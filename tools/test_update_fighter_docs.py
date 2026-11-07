"""File-catalog documentation checks; no mod builds or networking."""
import configparser
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import update_fighter_docs as docs

AUTO_EVENTS = (
    "auto_roll", "auto_spot_dodge", "auto_air_dodge", "auto_grab",
    "auto_shield_grab", "auto_grab_dodged", "auto_shield_backup",
)


class FighterDocsTests(unittest.TestCase):
    def test_source_strips_comments_without_changing_strings(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "profile.rs"
            path.write_text('name: "A, B", url: "https://example.test", // removed\n')
            self.assertEqual(docs.source(path), 'name: "A, B", url: "https://example.test", \n')

    def test_hex_identity_is_not_truncated_to_mario(self):
        for module, kind in [("sonic", 0x29), ("isabelle", 0x46), ("krool", 0x45)]:
            self.assertEqual(docs.render(module)[1], kind)

    def test_move_frames_and_baseline_stats_come_from_the_fighter_record(self):
        rendered, kind = docs.render("kirby")
        fighter = docs.fighter_data_snapshot()[kind]
        neutral_air = next(move for move in fighter["moves"] if move["name"] == "Neutral Air")
        active = " / ".join(
            str(first) if first == last else f"{first}–{last}"
            for first, last in neutral_air["active"]
        )
        self.assertIn(f"| Neutral Air | {neutral_air['raw']['startup']} start; active {active};", rendered)
        weight = next(stat["value"] for stat in fighter["stats"] if stat["key"] == "weight.weight")
        self.assertIn(f"| Weight | {weight} |", rendered)

    def test_all_seven_short_auto_tokens_and_legacy_priorities_are_documented(self):
        catalog = configparser.RawConfigParser()
        for event in AUTO_EVENTS:
            catalog[f"bowser.roll.{event}"] = {"reaction": f"{event}:35:24", "name": event}
        catalog["bowser.attack.legacy"] = {"reaction": "shield_hit:grounded:0:22:12:front:3"}
        with patch.object(docs, "sequence_catalog", return_value=catalog):
            rules = docs.file_assists("bowser")
        self.assertEqual({r[0] for r in rules}, set(AUTO_EVENTS) | {"shield_hit"})
        self.assertTrue(all(priority == 1 for event, priority, *_ in rules if event in AUTO_EVENTS))
        self.assertEqual(next(r[1] for r in rules if r[0] == "shield_hit"), 3)

    def test_auto_sequences_never_become_manual_choices_or_ledge_defaults(self):
        catalog = configparser.RawConfigParser()
        for row in docs.ROWS:
            catalog[f"bowser.{row.lower()}.manual"] = {
                "start_delay": "0", "actions": "input:a:0:0:1", "default": "true",
            }
        for event in AUTO_EVENTS:
            catalog[f"bowser.roll.{event}"] = {
                "reaction": f"{event}:35:24", "start_delay": "0",
                "actions": "input:r:0:0:4", "default": "true",
            }
        with patch.object(docs, "sequence_catalog", return_value=catalog):
            menu = docs.file_menu("bowser")
        for row in menu:
            self.assertEqual([choice["id"] for choice in row["options"]], ["manual"])
            self.assertEqual(row["selected"], 0)

    def test_all_registered_profiles_have_five_valid_lists_and_current_pages(self):
        pages = docs.outputs()
        self.assertEqual(len(pages) - 1, 79)
        for path, text in pages.items():
            self.assertEqual(path.read_text(), text, str(path))
        self.assertIn("## Kirby-specific timing and inputs", pages[docs.ROOT / "docs/fighters/kirby.md"])

if __name__ == "__main__":
    unittest.main()
