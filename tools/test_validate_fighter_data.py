import json
import unittest
from pathlib import Path

from tools import validate_fighter_data


class FighterDataValidationTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads(validate_fighter_data.DATA_FILE.read_text(encoding="utf-8"))

    def test_canonical_file_contains_full_roster_and_move_tables(self):
        fighter_count, move_count = validate_fighter_data.validate(self.data)
        self.assertEqual(fighter_count, 90)
        self.assertGreater(move_count, 4_000)
        self.assertEqual(self.data["fighters"][6]["name"], "kirby")

    def test_every_roster_kind_is_present_once_and_sorted(self):
        self.data["fighters"].pop(0)
        with self.assertRaisesRegex(ValueError, "exactly 90"):
            validate_fighter_data.validate(self.data)

        self.data = json.loads(validate_fighter_data.DATA_FILE.read_text(encoding="utf-8"))
        self.data["fighters"][0]["kind"] = self.data["fighters"][1]["kind"]
        with self.assertRaisesRegex(ValueError, "every supported kind"):
            validate_fighter_data.validate(self.data)

    def test_active_frame_gaps_are_preserved_and_invalid_ranges_rejected(self):
        mario = self.data["fighters"][0]
        down_smash = next(move for move in mario["moves"] if move["name"] == "Down Smash")
        self.assertEqual(down_smash["active"], [[5, 6], [14, 14]])
        down_smash["active"] = [[14, 5]]
        with self.assertRaisesRegex(ValueError, "invalid active span"):
            validate_fighter_data.validate(self.data)

    def test_unknown_and_compound_values_remain_text_not_invented_numbers(self):
        mario = self.data["fighters"][0]
        self.assertIsInstance(next(stat for stat in mario["stats"]
                                   if stat["key"] == "fallspeed.increase")["value"], str)
        self.assertFalse(any("source" in fighter or "sha256" in fighter
                             for fighter in self.data["fighters"]))
        self.assertNotIn("stats_source", self.data)


if __name__ == "__main__":
    unittest.main()
