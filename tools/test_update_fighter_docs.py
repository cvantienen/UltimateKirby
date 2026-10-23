"""Documentation-source extraction checks; no Rust compilation or networking."""
import unittest
import update_fighter_docs as docs


class FighterDocsTests(unittest.TestCase):
    def test_split_preserves_nested_choices_and_commas_inside_labels(self):
        self.assertEqual(
            docs.split_top('SavedChoice::new("a", "A, B", f, g), BASE.lists[0]'),
            ['SavedChoice::new("a", "A, B", f, g)', 'BASE.lists[0]'],
        )

    def test_hex_identity_is_not_truncated_to_mario(self):
        for module, kind in [("sonic", 0x29), ("isabelle", 0x46), ("krool", 0x45)]:
            self.assertEqual(docs.render(module)[1], kind)

    def test_shared_menu_alias_and_overridden_default(self):
        self.assertEqual(docs.evaluate("samus", "MENU"), docs.evaluate("dark_samus", "MENU"))
        roll = docs.evaluate("samus", "MENU")[2]
        self.assertEqual(roll["options"][roll["selected"]]["id"], "grab")

    def test_choice_struct_update_retains_id_and_handler(self):
        choice = docs.evaluate("littlemac", 'SavedChoice { name: "Tiny", ..BASE.lists[0].options[0] }')
        base = docs.evaluate("littlemac", "BASE.lists[0].options[0]")
        self.assertEqual(choice["name"], "Tiny")
        self.assertEqual(choice["id"], base["id"])
        self.assertEqual(choice["handler"], base["handler"])

    def test_all_registered_profiles_have_five_valid_lists_and_current_pages(self):
        pages = docs.outputs()
        self.assertEqual(len(pages) - 1, 77)
        for path, text in pages.items():
            self.assertEqual(path.read_text(), text, str(path))
        self.assertIn("## Kirby-specific timing and inputs", pages[docs.ROOT / "docs/fighters/kirby.md"])

    def test_unknown_static_expression_is_rejected(self):
        with self.assertRaises(ValueError):
            docs.evaluate("kirby", "unknown_runtime_builder()")


if __name__ == "__main__":
    unittest.main()
