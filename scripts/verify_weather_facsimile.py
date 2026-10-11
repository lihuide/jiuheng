                                                                        

                                                                            
   

import argparse
import hashlib
import json
from pathlib import Path


def verify(root: Path) -> tuple[int, int]:
    root = root.resolve(strict=True)
    research = root / "data" / "research"
    manifest_path = research / "liuyao-weather-naj-facsimile.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    pages = manifest["pages"]
    if manifest["schema_version"] != 1 or [p["canvas"] for p in pages] != list(range(38, 58)):
        raise ValueError("Unexpected facsimile manifest schema or chapter canvas range")
    if manifest["scope"]["released_rule_ids"] != 0 or manifest["scope"]["executable_seed_eligible"]:
        raise ValueError("Research-only facsimile manifest claims released rules")

    total_bytes = 0
    for page in pages:
        canvas = page["canvas"]
        expected_label = f"C{103347737500 + (canvas - 38) * 100}"
        expected_relative_path = f"facsimiles/zengshan-buyi-naj-vol3/canvas-{canvas:02}.jpg"
        if page["label"] != expected_label or page["path"] != expected_relative_path:
            raise ValueError(f"Unexpected locator for canvas {canvas}")
        path = research / expected_relative_path
        image = path.read_bytes()
        if not image.startswith(b"\xff\xd8") or not image.endswith(b"\xff\xd9"):
            raise ValueError(f"Invalid JPEG envelope: {path}")
        if len(image) != page["bytes"]:
            raise ValueError(f"Byte length mismatch: {path}")
        if hashlib.sha256(image).hexdigest() != page["sha256"]:
            raise ValueError(f"SHA-256 mismatch: {path}")
        total_bytes += len(image)
    exclusions = json.loads(
        (research / "liuyao-weather-exclusions-2026-10-02.json").read_text(encoding="utf-8")
    )
    validate_exclusions(exclusions, pages)
    return len(pages), total_bytes


def validate_exclusions(exclusions: dict, pages: list[dict]) -> int:
                                                                                         
    if (
        exclusions.get("schema_version") != 1
        or exclusions.get("status") != "single_reader_partial_transcription_non_executable"
        or exclusions.get("formal_rule_ids") != []
        or exclusions.get("may_generate_prediction") is not False
    ):
        raise ValueError("Weather excerpts must remain research-only")
    page_by_canvas = {page["canvas"]: page for page in pages}
    excerpts = exclusions.get("excerpts")
    if not isinstance(excerpts, list) or not excerpts:
        raise ValueError("Weather exclusion excerpts are missing")
    excerpt_ids = set()
    for excerpt in excerpts:
        excerpt_id = excerpt["excerpt_id"]
        if not excerpt_id or excerpt_id in excerpt_ids:
            raise ValueError("Weather excerpt IDs must be distinct")
        excerpt_ids.add(excerpt_id)
        page = page_by_canvas.get(excerpt["canvas"])
        if (
            page is None
            or excerpt["side"] not in {"left", "right"}
            or excerpt["image_sha256"] != page["sha256"]
            or not excerpt["text"].strip()
        ):
            raise ValueError(f"Weather excerpt is not anchored: {excerpt_id}")
    shortcuts = exclusions.get("excluded_shortcuts")
    if not isinstance(shortcuts, list) or not shortcuts:
        raise ValueError("Weather exclusions are missing")
    for shortcut in shortcuts:
        if (
            shortcut.get("release_status") != "excluded"
            or shortcut.get("basis_excerpt_id") not in excerpt_ids
            or not shortcut.get("reason", "").strip()
        ):
            raise ValueError("An excluded shortcut lacks evidence or is marked executable")
    return len(excerpts)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    count, byte_count = verify(args.root)
    print(
        f"Verified {count} facsimile canvases, {byte_count} bytes and research-only excerpts. "
        "No divination rule released."
    )
