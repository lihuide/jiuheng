                                                                             

                                                                  
                                                        
   

import argparse
import ast
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "data/research/kintaiyi-counts-static-reference.json"


def checked_bytes(path, size, digest):
                                                                         
    if path.stat().st_size != size:
        raise ValueError(f"unexpected byte length: {path.name}")
    content = path.read_bytes()
    if hashlib.sha256(content).hexdigest() != digest:
        raise ValueError(f"SHA-256 mismatch: {path.name}")
    return content


def extract_tables(content):
    tree = ast.parse(content.decode("utf-8"))
    functions = [
        node for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name == "find_cal"
    ]
    if len(functions) != 1:
        raise ValueError("find_cal must have exactly one definition")
    result = {}
    for dun, variable in (("yang", "yangcal"), ("yin", "yingcal")):
        assignments = [
            node for node in functions[0].body
            if isinstance(node, ast.Assign)
            and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
            and node.targets[0].id == variable
        ]
        if len(assignments) != 1:
            raise ValueError(f"{variable} must have exactly one literal assignment")
                                                                           
                                                                              
        rows = ast.literal_eval(assignments[0].value)
        if type(rows) is not list or len(rows) != 72:
            raise ValueError(f"{variable} must contain exactly 72 rows")
        for row in rows:
            if (type(row) is not list or len(row) != 3
                    or any(type(value) is not int or not 1 <= value <= 40 for value in row)):
                raise ValueError(f"invalid count row in {variable}")
        result[dun] = [row[:2] for row in rows]
    return result


def verify(source_path, license_path, fixture):
    source = fixture["source"]
    content = checked_bytes(source_path, source["bytes"], source["sha256"])
    license_info = fixture["license"]
    license_bytes = checked_bytes(
        license_path, license_info["raw_github_bytes"], license_info["raw_github_sha256"]
    )
                                                                         
                                                                            
    crlf_bytes = license_bytes.replace(b"\n", b"\r\n")
    if (len(crlf_bytes) != license_info["original_manifest_bytes"]
            or hashlib.sha256(crlf_bytes).hexdigest() != license_info["original_manifest_sha256"]):
        raise ValueError("LICENSE line-ending bridge does not match pinned manifest")
    notice = (ROOT / license_info["notice_path"]).read_bytes().replace(b"\r\n", b"\n")
    if notice.rstrip(b"\n") != license_bytes.rstrip(b"\n"):
        raise ValueError("distributed MIT notice differs from pinned license")
    tables = extract_tables(content)
    for dun in ("yang", "yin"):
        if tables[dun] != fixture[dun]:
            raise ValueError(f"committed {dun} reference differs from pinned source")
    return {"source_hash_verified": True, "license_hash_verified": True,
            "license_notice_verified": True, "static_rows_verified": 144,
            "retained_cells_verified": 288, "third_party_code_executed": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--license", type=Path, required=True)
    args = parser.parse_args()
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    try:
        print(json.dumps(verify(args.source, args.license, fixture), ensure_ascii=False))
    except (ValueError, OSError, SyntaxError) as error:
        parser.exit(1, f"static reference verification failed: {error}\n")


if __name__ == "__main__":
    main()
