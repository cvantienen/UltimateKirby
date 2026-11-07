"""Fake FTP tests only; no Switch connection or build."""
import contextlib
import io
import unittest
import subprocess
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch

import upload_sequences as uploader
import sequence_validation


class FakeFTP:
    def __init__(self, corrupt=False):
        self.commands = []
        self.data = b""
        self.corrupt = corrupt

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False

    def connect(self, host, port, timeout):
        self.connection = (host, port, timeout)

    def login(self):
        pass

    def storbinary(self, command, source):
        self.commands.append(command)
        self.data = source.read()

    def voidcmd(self, command):
        self.commands.append(command)

    def size(self, path):
        self.commands.append("SIZE " + path)
        return len(self.data)

    def retrbinary(self, command, consume):
        self.commands.append(command)
        consume(b"wrong" if self.corrupt else self.data)


class UploadSequencesTests(unittest.TestCase):
    def test_validator_receives_exact_upload_bytes_and_runs_offline(self):
        data = b"[yoshi.neutral.test]\nactions = short_hop_nair\n"
        result = subprocess.CompletedProcess([], 0, b"Validated\n", b"")
        with patch.object(sequence_validation.subprocess, "run", return_value=result) as run:
            with contextlib.redirect_stdout(io.StringIO()):
                uploader.validate_sequences(data)
        self.assertEqual(run.call_args.kwargs["input"], data)
        self.assertIn("--offline", run.call_args.args[0])
        self.assertIn("validate_sequences", run.call_args.args[0])

    def test_validation_error_blocks_both_switches(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sequences.txt"
            path.write_bytes(b"bad structure")
            result = subprocess.CompletedProcess([], 1, b"", b"Invalid sequences.txt: line 1: invalid header")
            with patch.object(uploader, "SOURCE", path), \
                    patch.object(sys, "argv", ["upload_sequences.py"]), \
                    patch.object(sequence_validation.subprocess, "run", return_value=result), \
                    patch.object(uploader, "upload_both") as upload, \
                    contextlib.redirect_stderr(io.StringIO()) as errors:
                with self.assertRaises(SystemExit):
                    uploader.main()
            upload.assert_not_called()
            self.assertIn("line 1: invalid header", errors.getvalue())

    def test_check_only_validates_without_connecting(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sequences.txt"
            path.write_bytes(b"[yoshi.roll.grab]\nactions=grab\n")
            result = subprocess.CompletedProcess([], 0, b"Validated\n", b"")
            with patch.object(uploader, "SOURCE", path), \
                    patch.object(sys, "argv", ["upload_sequences.py", "--check-only"]), \
                    patch.object(sequence_validation.subprocess, "run", return_value=result) as validate, \
                    patch.object(uploader, "upload_both") as upload, \
                    contextlib.redirect_stdout(io.StringIO()):
                uploader.main()
            validate.assert_called_once()
            upload.assert_not_called()

    def test_real_parser_catches_recent_config_errors_with_section_context(self):
        for action in ("walk:opponent:4:0.5:", "walk_to_opponent:1.0:3",
                       "short_hop_nair:4,", "drift"):
            with self.subTest(action=action):
                data = f"[yoshi.roll.grab]\nstart_delay=0\nactions = {action}\n".encode()
                with self.assertRaises(RuntimeError) as error:
                    uploader.validate_sequences(data)
                self.assertIn("[yoshi.roll.grab]", str(error.exception))
                self.assertIn("Nothing uploaded", str(error.exception))
        data = b"[yoshi.neutral.a]\nstart_delay=0\ndefault=true\nactions=grab\n[yoshi.neutral.b]\nstart_delay=0\ndefault=true\nactions=grab\n"
        with self.assertRaisesRegex(RuntimeError, "multiple defaults"):
            uploader.validate_sequences(data)

    def test_upload_writes_only_sequence_file_and_verifies_it(self):
        ftp = FakeFTP()
        with patch.object(uploader.ftplib, "FTP", return_value=ftp):
            uploader.upload_one("192.168.1.105", b"actions=test")
        self.assertEqual(ftp.connection, ("192.168.1.105", 5000, 10))
        self.assertEqual(uploader.REMOTE, "/ultimate/kirby_assist/sequences.upload.txt")
        self.assertNotIn("STOR /ultimate/kirby_assist/sequences.txt", ftp.commands)
        self.assertEqual(ftp.commands, [f"STOR {uploader.REMOTE}", "TYPE I",
                                        f"SIZE {uploader.REMOTE}", f"RETR {uploader.REMOTE}"])

    def test_corrupt_readback_fails(self):
        with patch.object(uploader.ftplib, "FTP", return_value=FakeFTP(corrupt=True)):
            with self.assertRaisesRegex(RuntimeError, "SHA-256"):
                uploader.upload_one("192.168.1.142", b"actions=test")

    def test_unavailable_first_switch_does_not_skip_second(self):
        with patch.object(uploader, "upload_one", side_effect=[OSError("offline"), None]) as upload:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(uploader.upload_both(b"actions=test"))
        self.assertEqual([call.args[0] for call in upload.call_args_list],
                         ["192.168.1.105", "192.168.1.142"])

    def test_replace_reuses_workbench_backups_and_verification(self):
        from types import SimpleNamespace
        import sequence_transfer
        data=b"validated bytes";source=Path("selected.txt");plan=SimpleNamespace(source_data=data)
        with patch.object(sequence_transfer,"prepare",return_value=plan) as prepare, \
                patch.object(sequence_transfer,"commit",return_value={"verified":True,"backup":"saved-backup"}) as commit, \
                patch.object(uploader.ftplib,"FTP") as ftp, contextlib.redirect_stdout(io.StringIO()):
            uploader.upload_one(uploader.TARGETS[0][0],data,replace=True,source=source)
        ftp.assert_not_called()
        self.assertEqual(prepare.call_args.args[0].kind,"local")
        self.assertEqual(prepare.call_args.args[1].path,uploader.ACTIVE)
        commit.assert_called_once_with(plan,uploader.BACKUP_ROOT)

    def test_changed_source_never_reaches_active_commit(self):
        from types import SimpleNamespace
        import sequence_transfer
        with patch.object(sequence_transfer,"prepare",return_value=SimpleNamespace(source_data=b"changed")), \
                patch.object(sequence_transfer,"commit") as commit:
            with self.assertRaisesRegex(RuntimeError,"Source changed"):
                uploader.upload_one(uploader.TARGETS[0][0],b"original",replace=True,source=Path("selected.txt"))
        commit.assert_not_called()

    def test_replace_cli_requires_closed_acknowledgment_before_any_connection(self):
        with patch.object(sys,"argv",["upload_sequences.py","--replace"]), \
                patch.object(uploader,"validate_sequences") as validate, \
                patch.object(uploader,"upload_both") as upload, contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as error:uploader.main()
        self.assertEqual(error.exception.code,2)
        validate.assert_not_called();upload.assert_not_called()

    def test_replace_and_upload_cli_select_the_expected_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            source=Path(directory)/"selected.txt";data=b"selected bytes";source.write_bytes(data)
            for flags,replace in [(["--upload"],False),(["--replace","--smash-closed"],True)]:
                with self.subTest(flags=flags), \
                        patch.object(sys,"argv",["upload_sequences.py","--file",str(source),*flags]), \
                        patch.object(uploader,"validate_sequences") as validate, \
                        patch.object(uploader,"upload_both",return_value=True) as upload, contextlib.redirect_stdout(io.StringIO()):
                    uploader.main()
                validate.assert_called_once_with(data)
                if replace:upload.assert_called_once_with(data,5000,replace=True,source=source)
                else:upload.assert_called_once_with(data,5000)

    def test_replace_dry_run_never_connects_or_requires_closed_acknowledgment(self):
        with tempfile.TemporaryDirectory() as directory:
            source=Path(directory)/"selected.txt";source.write_bytes(b"selected bytes")
            out=io.StringIO()
            with patch.object(sys,"argv",["upload_sequences.py","--file",str(source),"--replace","--dry-run"]), \
                    patch.object(uploader,"validate_sequences"), patch.object(uploader,"upload_both") as upload, \
                    contextlib.redirect_stdout(out):uploader.main()
            upload.assert_not_called();self.assertIn(uploader.ACTIVE,out.getvalue())

    def test_replace_attempts_both_switches_after_a_connection_drop(self):
        with patch.object(uploader,"upload_one",side_effect=[EOFError("closed"),None]) as upload, \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertFalse(uploader.upload_both(b"selected",replace=True,source=Path("selected.txt")))
        self.assertEqual(upload.call_count,len(uploader.TARGETS))
        self.assertTrue(all(c.kwargs["replace"] for c in upload.call_args_list))


class ValidatorCliTests(unittest.TestCase):
    def test_local_cli_selects_exact_file(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "my sequences.txt"
            data = b"[yoshi.settings]\nwalk_strength=0.28\n"
            path.write_bytes(data)
            with patch.object(sys, "argv", ["sequence_validation.py", str(path)]), \
                    patch.object(sequence_validation, "validate_sequences") as validate, \
                    contextlib.redirect_stdout(io.StringIO()):
                sequence_validation.main()
            validate.assert_called_once_with(data)

    def test_upload_check_only_accepts_selected_file_without_ftp(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "selected.txt"
            data = b"[yoshi.settings]\nwalk_strength=0.28\n"
            path.write_bytes(data)
            with patch.object(sys, "argv", ["upload_sequences.py", "--file", str(path), "--check-only"]), \
                    patch.object(uploader, "validate_sequences") as validate, \
                    patch.object(uploader, "upload_both") as upload, \
                    contextlib.redirect_stdout(io.StringIO()):
                uploader.main()
            validate.assert_called_once_with(data)
            upload.assert_not_called()


if __name__ == "__main__":
    unittest.main()
