import argparse
import json
import re
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description="Build Jiuheng's audited GeoNames CN location seed")
parser.add_argument("--source", required=True, type=Path, help="Extracted GeoNames CN.txt path")
parser.add_argument(
    "--output",
    type=Path,
    default=PROJECT_ROOT / "data" / "locations.seed.json",
    help="Generated JSON path",
)
parser.add_argument("--download-date", default="2026-08-30", help="YYYY-MM-DD provenance date")
args = parser.parse_args()
SOURCE = args.source.resolve()
OUTPUT = args.output.resolve()
DATASET_VERSION = f"geonames-cn-downloaded-{args.download_date}"
LEVELS = {"ADM1": 1, "ADM2": 2, "ADM3": 3, "ADM4": 4}
EXPECTED_SUFFIXES = {
    "ADM1": ("自治区", "特别行政区", "省", "市"),
    "ADM2": ("自治州", "地区", "市", "盟", "州"),
    "ADM3": ("自治县", "自治旗", "特区", "林区", "区", "县", "市", "旗"),
    "ADM4": ("民族乡", "街道", "办事处", "管理区", "苏木", "镇", "乡"),
}
TRADITIONAL_HINTS = set("區縣鄉鎮臺灣廣東龍門長樂烏魯齊瀋陽寧夏內蒙古")
CJK = re.compile(r"[\u3400-\u9fff]")
ALLOWED_NAME = re.compile(r"^[\u3400-\u9fff·•—（）()]+$")


def chinese_name(fields):
    code = fields[7]
    candidates = []
    for raw in [fields[1], *fields[3].split(",")]:
        value = raw.strip()
        if not value or not CJK.search(value) or not ALLOWED_NAME.match(value):
            continue
        suffix = any(value.endswith(item) for item in EXPECTED_SUFFIXES[code])
        official = any(item in value for item in ("自治", "省", "市", "地区", "盟", "区", "县", "旗", "街道", "镇", "乡", "苏木"))
        traditional = sum(char in TRADITIONAL_HINTS for char in value)
                                                                            
                                                            
        score = (suffix, official, -traditional, len(value))
        candidates.append((score, value))
    return max(candidates, default=(None, None), key=lambda item: item[0])[1]


records = []
with SOURCE.open("r", encoding="utf-8", errors="strict") as handle:
    for line in handle:
        fields = line.rstrip("\n").split("\t")
        if len(fields) < 19 or fields[7] not in LEVELS:
            continue
        name = chinese_name(fields)
        if not name:
            continue
        records.append(
            {
                "id": fields[0],
                "name": name,
                "latitude": float(fields[4]),
                "longitude": float(fields[5]),
                "feature_code": fields[7],
                "a1": fields[10],
                "a2": fields[11],
                "a3": fields[12],
                "a4": fields[13],
                "modified": fields[18],
            }
        )

adm1 = {(r["a1"],): r for r in records if r["feature_code"] == "ADM1"}
adm2 = {(r["a1"], r["a2"]): r for r in records if r["feature_code"] == "ADM2"}
adm3 = {(r["a1"], r["a2"], r["a3"]): r for r in records if r["feature_code"] == "ADM3"}
adm2_by_id = {r["id"]: r for r in records if r["feature_code"] == "ADM2"}
adm3_by_id = {r["id"]: r for r in records if r["feature_code"] == "ADM3"}

rows = []
skipped_incomplete_parent = 0
skip_reasons = {"province": 0, "city": 0, "district": 0}
for record in records:
    level = LEVELS[record["feature_code"]]
    province_record = record if level == 1 else adm1.get((record["a1"],))
    city_record = (
        record
        if level == 2
        else adm2_by_id.get(record["a2"]) or adm2.get((record["a1"], record["a2"]))
    )
    district_record = (
        record
        if level == 3
        else adm3_by_id.get(record["a3"])
        or adm3.get((record["a1"], record["a2"], record["a3"]))
    )
    if province_record is None or (level >= 3 and city_record is None) or (level == 4 and district_record is None):
        skipped_incomplete_parent += 1
        if province_record is None:
            skip_reasons["province"] += 1
        if level >= 3 and city_record is None:
            skip_reasons["city"] += 1
        if level == 4 and district_record is None:
            skip_reasons["district"] += 1
        continue

    province = province_record["name"]
    city = province if level == 1 else city_record["name"]
    district = record["name"] if level == 3 else (district_record["name"] if level == 4 else None)
    street = record["name"] if level == 4 else None
    precision = "city_center" if level <= 2 else ("district_center" if level == 3 else "street_center")
    rows.append(
        {
            "administrative_code": f"GN:{record['id']}",
            "code_system": "geonames_id",
            "province": province,
            "city": city,
            "district": district,
            "street": street,
            "longitude_deg": record["longitude"],
            "latitude_deg": record["latitude"],
            "coordinate_precision": precision,
            "source": f"GeoNames CN dump {record['feature_code']} feature center; center approximation, not boundary/address",
            "source_record_id": record["id"],
            "source_url": "https://download.geonames.org/export/dump/CN.zip",
            "source_license": "CC-BY-4.0",
            "source_modified_date": record["modified"],
            "dataset_version": DATASET_VERSION,
        }
    )

rows.sort(
    key=lambda row: (
        row["province"],
        row["city"],
        row["district"] or "",
        row["street"] or "",
        row["administrative_code"],
    )
)
codes = [row["administrative_code"] for row in rows]
assert len(codes) == len(set(codes))
assert len({row["province"] for row in rows if row["district"] is None and row["province"] == row["city"]}) == 31
assert all(-180 <= row["longitude_deg"] <= 180 and -90 <= row["latitude_deg"] <= 90 for row in rows)

OUTPUT.write_text(json.dumps(rows, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
print(json.dumps({"written": len(rows), "skipped_incomplete_parent": skipped_incomplete_parent, "skip_reasons": skip_reasons, "bytes": OUTPUT.stat().st_size}, ensure_ascii=False))
