                                                                          

from copy import deepcopy
import json
from pathlib import Path
import unittest

from scripts.verify_weather_facsimile import validate_exclusions


ROOT = Path(__file__).resolve().parents[2]
RESEARCH = ROOT / "data" / "research"


class WeatherExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        manifest = json.loads(
            (RESEARCH / "liuyao-weather-naj-facsimile.json").read_text(encoding="utf-8")
        )
        cls.pages = manifest["pages"]
        cls.exclusions = json.loads(
            (RESEARCH / "liuyao-weather-exclusions-2026-10-02.json").read_text(encoding="utf-8")
        )

    def test_excerpts_are_anchored_but_not_released(self):
        self.assertEqual(validate_exclusions(self.exclusions, self.pages), 2)

    def test_changed_image_hash_is_rejected(self):
        changed = deepcopy(self.exclusions)
        changed["excerpts"][0]["image_sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            validate_exclusions(changed, self.pages)

    def test_formal_rule_or_executable_shortcut_is_rejected(self):
        changed = deepcopy(self.exclusions)
        changed["formal_rule_ids"] = ["unsafe"]
        with self.assertRaises(ValueError):
            validate_exclusions(changed, self.pages)
        changed = deepcopy(self.exclusions)
        changed["excluded_shortcuts"][0]["release_status"] = "released"
        with self.assertRaises(ValueError):
            validate_exclusions(changed, self.pages)


if __name__ == "__main__":
    unittest.main()
