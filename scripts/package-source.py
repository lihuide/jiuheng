                                                                      

                                                                          
                                                                          
   

import argparse
import ast
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import sys
import tokenize
import tomllib
from zipfile import ZIP_DEFLATED, ZipFile


SOURCE_ROOTS = (".cargo", "apps", "crates", "data", "docs", "packaging", "scripts")
REQUIRED_FILES = ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "README.md", "LICENSE.md")
MANIFEST_NAME = "SOURCE-MANIFEST.json"
SCHEMA_VERSION = 1
EXCLUDED_DIRS = {"__pycache__", ".pytest_cache", "node_modules", "target", ".git"}
STANDARD_KIND = "source_snapshot_not_a_windows_build"
NO_COMMENTS_KIND = "source_snapshot_code_comments_removed_not_a_windows_build"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def source_inventory(root):
    root = Path(root).resolve(strict=True)
    for name in REQUIRED_FILES:
        if not (root / name).is_file():
            raise ValueError(f"Required source file missing: {name}")
    candidates = [item for item in root.iterdir() if item.is_file()]
    for name in SOURCE_ROOTS:
        directory = root / name
        if not directory.is_dir() or directory.is_symlink():
            raise ValueError(f"Required source directory missing or linked: {name}")
        candidates.extend(directory.rglob("*"))
    files = {}
    for path in candidates:
        relative = path.relative_to(root)
        if any(part in EXCLUDED_DIRS for part in relative.parts):
            continue
        if path.suffix.lower() in {".pyc", ".pyo"}:
            continue
        if path.is_symlink():
            raise ValueError(f"Linked source entry is not allowed: {relative}")
        if not path.is_file():
            continue
        if not path.resolve(strict=True).is_relative_to(root):
            raise ValueError(f"Source entry resolves outside project: {relative}")
        name = relative.as_posix()
        if name == MANIFEST_NAME:
            raise ValueError(f"Reserved generated filename exists: {name}")
        files[name] = path
    return dict(sorted(files.items()))


def project_version(root):
    with (Path(root) / "Cargo.toml").open("rb") as handle:
        return tomllib.load(handle)["workspace"]["package"]["version"]


def safe_member(name):
    path = PurePosixPath(name)
    return (
        bool(name)
        and "\\" not in name
        and ":" not in name
        and not path.is_absolute()
        and not any(part in {".", ".."} for part in name.split("/"))
        and path.as_posix() == name
        and not any(part in EXCLUDED_DIRS for part in path.parts)
        and path.suffix.lower() not in {".pyc", ".pyo"}
    )


def blank_except_newlines(value):
    return "".join(character if character in "\r\n" else " " for character in value)


def strip_rust_comments(data):
    source = data.decode("utf-8")
    output = []
    index = 0
    while index < len(source):
        raw_start = index + (1 if source.startswith("br", index) else 0)
        if source.startswith("r", raw_start):
            hashes_end = raw_start + 1
            while hashes_end < len(source) and source[hashes_end] == "#":
                hashes_end += 1
            if hashes_end < len(source) and source[hashes_end] == '"':
                delimiter = '"' + source[raw_start + 1 : hashes_end]
                close = source.find(delimiter, hashes_end + 1)
                if close < 0:
                    raise ValueError("Unterminated Rust raw string")
                end = close + len(delimiter)
                output.append(source[index:end])
                index = end
                continue
        if source[index] == '"':
            end = index + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            else:
                raise ValueError("Unterminated Rust string")
            output.append(source[index:end])
            index = end
            continue
        if source.startswith("//", index):
            end = index + 2
            while end < len(source) and source[end] not in "\r\n":
                end += 1
            output.append(blank_except_newlines(source[index:end]))
            index = end
            continue
        if source.startswith("/*", index):
            end = index + 2
            depth = 1
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            if depth:
                raise ValueError("Unterminated Rust block comment")
            output.append(blank_except_newlines(source[index:end]))
            index = end
            continue
        output.append(source[index])
        index += 1
    return "".join(output).encode("utf-8")


def strip_python_comments(data):
    encoding, _ = tokenize.detect_encoding(io.BytesIO(data).readline)
    if encoding.lower() not in {"utf-8", "utf-8-sig"}:
        raise ValueError(f"Unsupported Python source encoding: {encoding}")
    source = data.decode(encoding)
    lines = source.splitlines(keepends=True)
    for token in tokenize.tokenize(io.BytesIO(data).readline):
        if token.type == tokenize.COMMENT:
            row, start = token.start
            _, end = token.end
            line = lines[row - 1]
            lines[row - 1] = line[:start] + " " * (end - start) + line[end:]
    stripped_source = "".join(lines)
    stripped = bytearray(stripped_source.encode("utf-8"))
    byte_lines = stripped_source.encode("utf-8").splitlines(keepends=True)
    line_offsets = [0]
    for line in byte_lines:
        line_offsets.append(line_offsets[-1] + len(line))
    for node in ast.walk(ast.parse(stripped_source)):
        if not isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        if not node.body or not isinstance(node.body[0], ast.Expr):
            continue
        docstring = node.body[0]
        if not isinstance(docstring.value, ast.Constant) or not isinstance(docstring.value.value, str):
            continue
        start = line_offsets[docstring.lineno - 1] + docstring.col_offset
        end = line_offsets[docstring.end_lineno - 1] + docstring.end_col_offset
        for position in range(start, end):
            if stripped[position] not in (10, 13):
                stripped[position] = 32
        if len(node.body) == 1 and not isinstance(node, ast.Module):
            stripped[start : start + 4] = b"pass"
    return bytes(stripped)


def strip_toml_comments(data):
    source = data.decode("utf-8")
    return re.sub(
        r"(?m)^[ \t]*#[^\r\n]*",
        lambda match: blank_except_newlines(match.group()),
        source,
    ).encode("utf-8")


def source_bytes(name, data, without_comments):
    if not without_comments:
        return data
    if name.endswith(".rs"):
        return strip_rust_comments(data)
    if name.endswith(".py"):
        return strip_python_comments(data)
    if name.endswith(".toml") or name == "Cargo.lock":
        return strip_toml_comments(data)
    return data


def pack_source(root, output, without_comments=False):
    root = Path(root).resolve(strict=True)
    output = Path(output).resolve()
    inventory = source_inventory(root)
                                                                          
    if output.parent == root or any(output.is_relative_to(root / name) for name in SOURCE_ROOTS):
        raise ValueError("Output must be outside source roots and not at project top level")
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "product_version": project_version(root),
        "kind": NO_COMMENTS_KIND if without_comments else STANDARD_KIND,
        "code_comments_removed": without_comments,
        "files": {},
    }
                                                                             
    with ZipFile(output, "x", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for name, path in inventory.items():
            original = path.read_bytes()
            data = source_bytes(name, original, without_comments)
            archive.writestr(name, data)
            manifest["files"][name] = {
                "bytes": len(data),
                "sha256": sha256(data),
                "original_sha256": sha256(original),
            }
        archive.writestr(
            MANIFEST_NAME,
            json.dumps(manifest, ensure_ascii=False, sort_keys=True, indent=2).encode("utf-8"),
        )
                                                                              
                                                                    
    return verify_source(root, output, without_comments)


def verify_source(root, output, without_comments=False):
    inventory = source_inventory(root)
    with ZipFile(output, "r") as archive:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise ValueError("Duplicate archive members")
        if not all(safe_member(name) for name in names):
            raise ValueError("Unsafe or cached archive member")
        if set(names) != set(inventory) | {MANIFEST_NAME}:
            raise ValueError("Archive differs from the complete current source inventory")
        manifest = json.loads(archive.read(MANIFEST_NAME))
        if (
            manifest.get("schema_version") != SCHEMA_VERSION
            or manifest.get("product_version") != project_version(root)
            or manifest.get("kind")
            != (NO_COMMENTS_KIND if without_comments else STANDARD_KIND)
            or manifest.get("code_comments_removed") != without_comments
            or set(manifest.get("files", {})) != set(inventory)
        ):
            raise ValueError("Source manifest schema, version or inventory mismatch")
        for name, path in inventory.items():
            data = archive.read(name)
            original = path.read_bytes()
            expected = {
                "bytes": len(data),
                "sha256": sha256(data),
                "original_sha256": sha256(original),
            }
            if manifest["files"][name] != expected or source_bytes(name, original, without_comments) != data:
                raise ValueError(f"Source bytes mismatch: {name}")
            if without_comments and source_bytes(name, data, True) != data:
                raise ValueError(f"Code comment remained after stripping: {name}")
    output = Path(output)
    return {
        "status": "verified_against_current_workspace",
        "product_version": manifest["product_version"],
        "kind": manifest["kind"],
        "source_files": len(inventory),
        "archive": output.name,
        "archive_bytes": output.stat().st_size,
        "archive_sha256": sha256(output.read_bytes()),
        "temporary_source_copy_created": False,
        "windows_rebuilt": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("pack", "verify"))
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--without-comments", action="store_true")
    args = parser.parse_args()
    try:
        result = (pack_source if args.operation == "pack" else verify_source)(
            args.root, args.output, args.without_comments
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
    except (OSError, ValueError, KeyError) as error:
        print(f"Source delivery verification failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
