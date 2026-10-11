                                                                                  

                                                                             
                                                                              
                                                                             
   

import argparse
import collections
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--source", required=True, type=Path)
parser.add_argument("--locations", type=Path, default=ROOT / "data/locations.seed.json")
parser.add_argument("--output", type=Path, default=ROOT / "data/streets.seed.json")
args = parser.parse_args()

expected_source_sha256 = "d03463a39014b72e6b4cf093af8327e2539aa9efbb010c8d2eb59a054cb632fe"
expected_locations_sha256 = "daf969babe5123db49bb5aec4c0c8005c9e812f30fe74ca8857b82e0bcca24b2"
assert hashlib.sha256(args.source.read_bytes()).hexdigest() == expected_source_sha256
assert hashlib.sha256(args.locations.read_bytes()).hexdigest() == expected_locations_sha256

source = json.loads(args.source.read_text(encoding="utf-8"))
assert source["meta"]["table_name"] == "Xzqh20251231"
assert source["meta"]["is_partial"] is False
records = source["data"]
assert len(records) == source["meta"]["total"] == 41935
assert sum(item["level"] == 4 for item in records) == 38723
by_code = {item["code"]: item for item in records}
assert len(by_code) == len(records)

geonames = json.loads(args.locations.read_text(encoding="utf-8"))
by_hierarchy = {}
by_province_district = collections.defaultdict(list)
for location in geonames:
    if location["district"] is None or location["street"] is not None:
        continue
    key = (location["province"], location["city"], location["district"])
    by_hierarchy.setdefault(key, []).append(location)
    by_province_district[(location["province"], location["district"])].append(location)

excluded_provinces = {"香港特别行政区", "澳门特别行政区", "台湾省"}
output = []
reasons = collections.Counter()
for item in records:
    if item["level"] != 4:
        continue
    district = by_code.get(item["parent_code"])
    if district is None or district["level"] != 3:
        reasons["missing_county_parent"] += 1
        continue
    parent = by_code.get(district["parent_code"])
    if parent is None:
        reasons["missing_city_or_province_parent"] += 1
        continue
    if parent["level"] == 1:
        province = city = parent["name"]
    elif parent["level"] == 2:
        province_record = by_code.get(parent["parent_code"])
        if province_record is None or province_record["level"] != 1:
            reasons["missing_province_parent"] += 1
            continue
        province, city = province_record["name"], parent["name"]
    else:
        reasons["invalid_parent_level"] += 1
        continue
    if province in excluded_provinces:
        reasons["outside_mainland"] += 1
        continue

    exact = by_hierarchy.get((province, city, district["name"]), [])
    if len(exact) == 1:
        match, basis = exact[0], "exact_hierarchy"
    else:
        province_matches = by_province_district[(province, district["name"])]
        if len(province_matches) != 1:
            reasons["no_unique_district_center"] += 1
            continue
        match, basis = province_matches[0], "unique_province_district"
    assert match["coordinate_precision"] == "district_center"
    assert 73 <= match["longitude_deg"] <= 135
    assert 17 <= match["latitude_deg"] <= 54
    assert item["code"][:6] == district["code"]
    output.append(
        {
            "administrative_code": item["code"],
            "province": province,
            "city": city,
            "district": district["name"],
            "street": item["name"],
            "coordinate_reference_code": match["administrative_code"],
            "match_basis": basis,
        }
    )
    reasons[basis] += 1

output.sort(key=lambda row: row["administrative_code"])
assert len({row["administrative_code"] for row in output}) == len(output)
assert len(output) >= 34000
args.output.write_text(
    json.dumps(output, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
)
print(json.dumps({"written": len(output), "all_streets": 38723, "reasons": reasons, "bytes": args.output.stat().st_size}, ensure_ascii=False))
