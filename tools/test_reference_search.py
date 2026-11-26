import contextlib
import io
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest

import reference_search as reference


FUNCTIONS = """Synthetic fixture, not a game reference.
skyline-smash | example/bindings | commit abc123 | commit date 2026-01-01
FUNCTION 00001 | skyline-smash | src/cpp.rs:12 | app :: WorkModule :: get_int
  signature: pub fn get_int(id: i32) -> i32
  source: https://example.invalid/bindings#L12

FUNCTION 00002 | smash-rs | src/work.rs:9 | impl WorkModule :: get_int
  signature: pub fn get_int(id: WorkId) -> i32
  source: https://example.invalid/other#L9

FUNCTION 00003 | skyline-smash | src/cpp.rs:20 | app :: WorkModule :: get_float
  signature: pub fn get_float(id: i32) -> f32

FUNCTION POINTER FIELDS
POINTER | skyline-smash | src/work.rs:4 | VTable::get_int | type fn(i32) -> i32
  https://example.invalid/pointer#L4
MACRO DEFINITIONS
MACRO | skyline_macro | src/lib.rs:8 | hook
  https://example.invalid/macro#L8
EXPORT | getRegionAddress | https://example.invalid/exports
END OF CATALOG
"""
STATES = """Synthetic fixture.
STATE 00001 | skyline-smash | lua_const::FIGHTER_STATUS_KIND_WAIT | LuaConst literal
  value/expression: 0x0 (decimal 0)
  category: status identifiers
  source: https://example.invalid/state#L1

RECOVERED PARAMETER AND RESOURCE LABEL DICTIONARIES
LABEL SOURCE: ParamLabels.csv
https://example.invalid/labels
LABEL | ParamLabels.csv:1 | 0x123 | wait | source hash; MATCH
LABEL | motion_list/Labels.txt:1 | 0x456 | wait.nuanmb | COMPUTED hash
END OF CATALOG
"""


class ReferenceSearchTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        (self.directory / reference.CATALOGS[0]).write_text(FUNCTIONS)
        (self.directory / reference.CATALOGS[1]).write_text(STATES)
        self.database = self.directory / "index.sqlite3"
        self.summary = reference.build_index(self.directory, self.database)

    def connect(self):
        connection = reference.open_index(self.database)
        self.addCleanup(connection.close)
        return connection

    def test_streaming_preserves_entries_and_excludes_section_headers(self):
        self.assertEqual(self.summary["total"], 9)
        rows = reference.search_index(self.connect(), "get_int", exact=True)
        self.assertEqual(len(rows), 3)
        function = next(row for row in rows if row["kind"] == "function")
        self.assertIn("signature:", function["body"])
        self.assertIn("https://example.invalid", function["body"])
        self.assertNotIn("FUNCTION POINTER FIELDS", function["body"])
        self.assertEqual(function["line"], 3)
        self.assertEqual(function["owner"], "app :: WorkModule")

    def test_exact_filters_and_literal_prefix_search(self):
        connection = self.connect()
        exact = reference.search_index(connection, "GET_INT", exact=True,
                                       library="skyline-smash", kind="function")
        self.assertEqual(len(exact), 1)
        broader = reference.search_index(connection, "WorkModule get", kind="function")
        self.assertEqual(len(broader), 3)
        self.assertEqual(reference.search_index(connection, "nonexistent OR get_int"), [])
        self.assertEqual(len(reference.search_index(connection, "wait", kind="label")), 2)

    def test_pagination_and_budget_do_not_truncate_entries(self):
        rows = reference.search_index(self.connect(), "get_int", exact=True, limit=1)
        result = reference.bounded_result(rows, "get_int", 1, 0, 3000)
        self.assertTrue(result["more"])
        self.assertEqual(result["next_offset"], 1)
        self.assertEqual(result["results"][0]["body"], rows[0]["body"])
        oversized = dict(rows[0], body="x" * 3000)
        result = reference.bounded_result([oversized], "get_int", 1, 0, 1000)
        self.assertEqual(result["results"], [])
        self.assertTrue(result["more"])
        self.assertIsNone(result["next_offset"])
        self.assertLessEqual(len(json.dumps(result)), 1000)

    def test_change_detection_and_failed_rebuild_preserve_old_index(self):
        before = self.database.read_bytes()
        (self.directory / reference.CATALOGS[1]).write_text("unrecognized format\n")
        _, problems = reference.source_status(self.connect(), self.directory)
        self.assertTrue(problems)
        with self.assertRaisesRegex(ValueError, "No recognized entries"):
            reference.build_index(self.directory, self.database)
        self.assertEqual(self.database.read_bytes(), before)
        self.assertEqual(list(self.directory.glob(".reference-index-*")), [])
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            code = reference.main(["--data-dir", str(self.directory), "--db",
                                   str(self.database), "search", "get_int"])
        self.assertEqual(code, 1)
        self.assertIn("run build", output.getvalue())

    def test_unrelated_database_is_not_overwritten(self):
        unrelated = self.directory / "unrelated.sqlite3"
        with sqlite3.connect(unrelated) as connection:
            connection.execute("CREATE TABLE user_data(value TEXT)")
        before = unrelated.read_bytes()
        with self.assertRaises(sqlite3.Error):
            reference.build_index(self.directory, unrelated)
        self.assertEqual(unrelated.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
