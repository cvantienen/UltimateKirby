"""Release-wrapper options and routing; no builds or FTP connections."""
import contextlib
import io
import unittest
from types import SimpleNamespace
from unittest.mock import patch

import upload_all_switches as uploader


class UploadAllTests(unittest.TestCase):
    def test_replacement_requires_closed_games_but_read_only_modes_do_not(self):
        args = ["-v", "2.3.1", "--replace-sequences"]
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            uploader.parse_options(args)
        for extra in ["--smash-closed", "--dry-run", "--verify-only"]:
            self.assertTrue(uploader.parse_options(args + [extra]).replace_sequences)
        self.assertFalse(uploader.parse_options(["-v", "2.3.1"]).replace_sequences)

    def test_both_targets_receive_replacement_and_selected_read_only_flags(self):
        for mode in [[], ["--dry-run"], ["--verify-only"]]:
            options = uploader.parse_options(["-v", "2.3.1", "--replace-sequences", "--smash-closed"] + mode)
            with patch.object(uploader, "parse_options", return_value=options), \
                    patch.object(uploader.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run, \
                    contextlib.redirect_stdout(io.StringIO()):
                uploader.main()
            self.assertEqual(run.call_count, 2)
            for call, (host, variant) in zip(run.call_args_list, uploader.TARGETS):
                command = call.args[0]
                self.assertIn(host, command)
                self.assertIn("-s" if variant == "silent" else "-d", command)
                self.assertIn("--replace-sequences", command)
                self.assertIn("--smash-closed", command)
                for flag in mode:
                    self.assertIn(flag, command)


if __name__ == "__main__":
    unittest.main()
