                                                                                    

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import warnings
from zipfile import ZipFile


SPEC = importlib.util.spec_from_file_location(
    "source_package", Path(__file__).resolve().parents[1] / "package-source.py"
)
PACKAGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGE)


class SourcePackageTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="jiuheng-source-package-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "project"
        self.root.mkdir()
        for name in PACKAGE.REQUIRED_FILES:
            (self.root / name).write_text("fixture\n", encoding="utf-8")
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion = "0.75.0"\n', encoding="utf-8"
        )
        for name in PACKAGE.SOURCE_ROOTS:
            (self.root / name).mkdir()
            (self.root / name / "fixture.txt").write_text("中文源码\n", encoding="utf-8")
        self.output = self.root / "outputs" / "source.zip"
        self.output.parent.mkdir()

    def pack(self):
        return PACKAGE.pack_source(self.root, self.output)

    def mutation(self, transform):
        self.pack()
        mutated = self.output.parent / "mutated.zip"
        with ZipFile(self.output) as original, ZipFile(mutated, "x") as archive:
            for name in original.namelist():
                result = transform(name, original.read(name))
                if result is not None:
                    archive.writestr(name, result)
        return mutated

    def test_roundtrip_complete_source_and_hidden_cargo_without_staging(self):
        result = self.pack()
        self.assertEqual(result["source_files"], 12)
        self.assertFalse(result["temporary_source_copy_created"])
        self.assertFalse(result["windows_rebuilt"])
        self.assertEqual(result, PACKAGE.verify_source(self.root, self.output))
        with ZipFile(self.output) as archive:
            self.assertIn(".cargo/fixture.txt", archive.namelist())
            self.assertIn("SOURCE-MANIFEST.json", archive.namelist())
        self.assertFalse((self.root / "work").exists())

    def test_comment_free_archive_preserves_complete_inventory_and_strings(self):
        rust = b'''// file comment\nconst URL: &str = r#"https://example.org/a/*literal*/"#; /* outer /* nested */ done */\nfn main() { println!("// literal"); }\n'''
        python = b'''"""Module documentation."""\nVALUE = "# literal"  # comment\ndef only_docstring():\n    """Function documentation."""\n'''
        (self.root / "crates" / "fixture.rs").write_bytes(rust)
        (self.root / "scripts" / "fixture.py").write_bytes(python)
        (self.root / ".cargo" / "config.toml").write_bytes(
            b'# mirror note\n[source.crates-io]\nreplace-with = "mirror"\n'
        )
        result = PACKAGE.pack_source(self.root, self.output, without_comments=True)
        self.assertEqual(result, PACKAGE.verify_source(self.root, self.output, True))
        with ZipFile(self.output) as archive:
            self.assertEqual(len(archive.namelist()), 16)
            self.assertIn(b'https://example.org/a/*literal*/', archive.read("crates/fixture.rs"))
            self.assertIn(b'println!("// literal")', archive.read("crates/fixture.rs"))
            self.assertNotIn(b"file comment", archive.read("crates/fixture.rs"))
            self.assertNotIn(b"nested", archive.read("crates/fixture.rs"))
            self.assertIn(b'"# literal"', archive.read("scripts/fixture.py"))
            self.assertNotIn(b"# comment", archive.read("scripts/fixture.py"))
            self.assertNotIn(b"Module documentation", archive.read("scripts/fixture.py"))
            self.assertNotIn(b"Function documentation", archive.read("scripts/fixture.py"))
            self.assertIn(b"    pass", archive.read("scripts/fixture.py"))
            self.assertNotIn(b"mirror note", archive.read(".cargo/config.toml"))
            manifest = json.loads(archive.read("SOURCE-MANIFEST.json"))
            self.assertTrue(manifest["code_comments_removed"])
            self.assertNotEqual(
                manifest["files"]["crates/fixture.rs"]["sha256"],
                manifest["files"]["crates/fixture.rs"]["original_sha256"],
            )

    def test_comment_free_rust_rejects_unterminated_block(self):
        (self.root / "crates" / "bad.rs").write_text("fn main() {} /*", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Unterminated Rust block comment"):
            PACKAGE.pack_source(self.root, self.output, without_comments=True)

    def test_cache_and_outputs_are_not_source(self):
        cache = self.root / "scripts" / "__pycache__"
        cache.mkdir()
        (cache / "bad.pyc").write_bytes(b"cache")
        (self.root / "scripts" / "bad.pyo").write_bytes(b"cache")
        (self.output.parent / "old.zip").write_bytes(b"published artifact")
        self.assertEqual(self.pack()["source_files"], 12)

    def test_existing_archive_is_not_overwritten(self):
        self.pack()
        before = self.output.read_bytes()
        with self.assertRaises(FileExistsError):
            self.pack()
        self.assertEqual(self.output.read_bytes(), before)

    def test_missing_required_file_rejects_before_creation(self):
        (self.root / "Cargo.lock").unlink()
        with self.assertRaisesRegex(ValueError, "Required source file"):
            self.pack()
        self.assertFalse(self.output.exists())

    def test_source_root_and_project_root_output_rejected(self):
        for output in (self.root / "source.zip", self.root / "scripts" / "source.zip"):
            with self.assertRaisesRegex(ValueError, "Output must be outside"):
                PACKAGE.pack_source(self.root, output)
            self.assertFalse(output.exists())

    def test_changed_source_bytes_reject(self):
        self.pack()
        (self.root / "docs" / "fixture.txt").write_text("changed", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Source bytes mismatch"):
            PACKAGE.verify_source(self.root, self.output)

    def test_missing_archive_member_reject(self):
        mutated = self.mutation(lambda name, data: None if name == "Cargo.lock" else data)
        with self.assertRaisesRegex(ValueError, "complete current source"):
            PACKAGE.verify_source(self.root, mutated)

    def test_corrupted_archive_bytes_reject(self):
        mutated = self.mutation(lambda name, data: b"bad" if name == "Cargo.lock" else data)
        with self.assertRaisesRegex(ValueError, "Source bytes mismatch"):
            PACKAGE.verify_source(self.root, mutated)

    def test_manifest_product_version_reject(self):
        def transform(name, data):
            if name == PACKAGE.MANIFEST_NAME:
                manifest = json.loads(data)
                manifest["product_version"] = "99.0.0"
                return json.dumps(manifest).encode()
            return data

        mutated = self.mutation(transform)
        with self.assertRaisesRegex(ValueError, "manifest schema, version"):
            PACKAGE.verify_source(self.root, mutated)

    def test_extra_duplicate_unsafe_and_cache_members_reject(self):
        self.pack()
        for index, name in enumerate(("extra.txt", "Cargo.lock", "../escape", "scripts/__pycache__/a.pyc")):
            mutated = self.output.parent / f"bad-{index}.zip"
            with ZipFile(self.output) as original, ZipFile(mutated, "x") as archive:
                for member in original.namelist():
                    archive.writestr(member, original.read(member))
                with warnings.catch_warnings():
                    warnings.simplefilter("ignore", UserWarning)
                    archive.writestr(name, b"not source")
            with self.assertRaises(ValueError):
                PACKAGE.verify_source(self.root, mutated)


if __name__ == "__main__":
    unittest.main()
