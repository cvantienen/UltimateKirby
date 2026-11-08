"""Workbench tests: temporary local files/fake FTP only; no Switch/build operations."""
import contextlib
import ftplib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import sequence_transfer as transfer
from workbench_history import History, receipts
import workbench

SOURCE = b"[kirby.neutral.new]\nstart_delay=0\nactions=a:4\n[kirby.settings]\ngrab_range=9\n"
DEST = b"# keep other fighters\n[yoshi.roll.perfect]\nstart_delay=0\nactions=b:4\n[kirby.roll.old]\nstart_delay=0\nactions=r:4\n"


class FakeFTP:
    def __init__(self, files):
        self.files = files
        self.commands = []
        self.fail_active = False
        self.fail_backup = False
        self.failed = False

    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def retrbinary(self, command, consume):
        self.commands.append(command)
        path = command.removeprefix("RETR ")
        if path not in self.files:
            raise ftplib.error_perm("550 absent")
        consume(self.files[path])

    def nlst(self, folder):
        return list(self.files)

    def storbinary(self, command, data):
        self.commands.append(command)
        path = command.removeprefix("STOR ")
        if path.endswith(".bak") and self.fail_backup:
            raise OSError("backup denied")
        payload = data.read()
        if path == transfer.ACTIVE and self.fail_active and not self.failed:
            self.failed = True
            self.files[path] = payload[:4]
            raise OSError("connection interrupted")
        self.files[path] = payload


class TransferTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source.txt"
        self.destination = self.root / "destination.txt"
        self.source.write_bytes(SOURCE)
        self.destination.write_bytes(DEST)
        self.local = transfer.Endpoint("local", str(self.source))
        self.dest = transfer.Endpoint("local", str(self.destination))
        self.validator = patch.object(transfer, "validate_sequences")
        self.validator.start()
        self.addCleanup(self.validator.stop)

    def test_local_fighter_merge_preserves_others_and_copies_settings(self):
        plan = transfer.prepare(self.local, self.dest, "kirby")
        result = transfer.commit(plan, self.root / "backups")
        data = self.destination.read_bytes()
        self.assertIn(b"[yoshi.roll.perfect]\nstart_delay=0\nactions=b:4", data)
        self.assertIn(b"[kirby.settings]", data)
        self.assertNotIn(b"kirby.roll.old", data)
        self.assertEqual(self.source.read_bytes(), SOURCE)
        self.assertEqual((Path(result["backup"]) / "destination-before.txt").read_bytes(), DEST)

    def test_source_or_destination_changed_after_preview_blocks_mutation(self):
        for changed_path in [self.source, self.destination]:
            self.source.write_bytes(SOURCE)
            self.destination.write_bytes(DEST)
            plan = transfer.prepare(self.local, self.dest)
            changed_path.write_bytes(b"changed")
            before = self.destination.read_bytes()
            with self.assertRaisesRegex(RuntimeError, "changed after preview"):
                transfer.commit(plan, self.root / "backups")
            self.assertEqual(self.destination.read_bytes(), before)
            self.assertFalse((self.root / "backups").exists())

    def test_invalid_merge_is_rejected_before_write(self):
        with patch.object(transfer, "validate_sequences", side_effect=[None, None, RuntimeError("invalid merge")]):
            with self.assertRaisesRegex(RuntimeError, "invalid merge"):
                transfer.prepare(self.local, self.dest, "kirby")
        self.assertEqual(self.destination.read_bytes(), DEST)

    def test_new_local_whole_file_and_atomic_replace_failure(self):
        self.destination.unlink()
        plan = transfer.prepare(self.local, self.dest)
        transfer.commit(plan, self.root / "backups")
        self.assertEqual(self.destination.read_bytes(), SOURCE)
        self.destination.write_bytes(DEST)
        plan = transfer.prepare(self.local, self.dest)
        with patch.object(transfer.os, "replace", side_effect=OSError("rename denied")):
            with self.assertRaises(OSError):
                transfer.commit(plan, self.root / "backups")
        self.assertEqual(self.destination.read_bytes(), DEST)
        self.assertEqual(list(self.root.glob("*.workbench-*")), [])

    def test_remote_preview_readonly_backup_failure_and_failed_write_rollback(self):
        remote = transfer.Endpoint("silent", transfer.ACTIVE)
        ftp = FakeFTP({transfer.ACTIVE: DEST})
        with patch.object(transfer, "connect", return_value=ftp):
            plan = transfer.prepare(self.local, remote, "kirby")
            self.assertFalse(any(c.startswith("STOR") for c in ftp.commands))
            ftp.fail_backup = True
            with self.assertRaises(OSError):
                transfer.commit(plan, self.root / "backups")
            self.assertEqual(ftp.files[transfer.ACTIVE], DEST)
            self.assertNotIn("STOR " + transfer.ACTIVE, ftp.commands)
            ftp.fail_backup = False
            ftp.fail_active = True
            with self.assertRaisesRegex(RuntimeError, "original restored and verified"):
                transfer.commit(plan, self.root / "backups")
            self.assertEqual(ftp.files[transfer.ACTIVE], DEST)

    def test_switch_to_local_and_remote_to_remote_whole_file(self):
        source = transfer.Endpoint("diagnostic", transfer.ACTIVE)
        destination = transfer.Endpoint("silent", transfer.ACTIVE)
        a, b = FakeFTP({transfer.ACTIVE: SOURCE}), FakeFTP({transfer.ACTIVE: DEST})
        with patch.object(transfer, "connect", side_effect=lambda endpoint: a if endpoint.kind == "diagnostic" else b):
            local_plan = transfer.prepare(source, self.dest)
            transfer.commit(local_plan, self.root / "backups")
            self.assertEqual(self.destination.read_bytes(), SOURCE)
            remote_plan = transfer.prepare(source, destination)
            transfer.commit(remote_plan, self.root / "backups")
            self.assertEqual(b.files[transfer.ACTIVE], SOURCE)
            self.assertFalse(any(c.startswith("STOR") for c in a.commands))

    def test_unchanged_no_write_and_invalid_endpoints(self):
        self.destination.write_bytes(SOURCE)
        plan = transfer.prepare(self.local, self.dest)
        result = transfer.commit(plan, self.root / "backups")
        self.assertTrue(result["unchanged"])
        self.assertFalse((self.root / "backups").exists())
        with self.assertRaises(ValueError):
            transfer.Endpoint("silent", "/arbitrary.nro")
        with self.assertRaises(ValueError):
            transfer.Endpoint("silent", transfer.ACTIVE, 0)
        with self.assertRaises(RuntimeError):
            transfer.prepare(self.local, self.local)


class HistoryAndLauncherTests(unittest.TestCase):
    def test_journal_restarts_failures_and_exact_receipts(self):
        with tempfile.TemporaryDirectory() as folder:
            h = History(Path(folder))
            archive = h.archive(b"payload")
            details = {"target": "silent", "version": "2.2.2", "artifacts": [archive]}
            def upload_output():
                print(f"  /plugin.nro (7 bytes; SHA-256 {archive['sha256']})")
                print("Uploaded and verified /plugin.nro")
                print("FTP update complete: all 1 files verified by read-back SHA-256.")
                return {"exit_code": 0}
            with contextlib.redirect_stdout(io.StringIO()):
                h.perform("upload", details, upload_output)
                with self.assertRaisesRegex(RuntimeError, "failure"):
                    h.perform("upload", {}, lambda: (_ for _ in ()).throw(RuntimeError("failure")))
            rows = History(Path(folder)).rows()
            self.assertEqual([r["status"] for r in rows], ["success", "failed"])
            self.assertTrue(rows[0]["receipt"]["verified_deployment"])
            self.assertEqual(rows[0]["receipt"]["payloads"][0]["archive"], archive["archive"])
            self.assertEqual(Path(archive["archive"]).read_bytes(), b"payload")
            h.append({"id": "unfinished", "status": "started"})
            self.assertEqual(h.rows()[-1]["status"], "started")

    def test_planned_manifest_alone_is_not_shipped_and_lock_excludes_second_session(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "log"
            path.write_text("  /plugin.nro (3 bytes; SHA-256 " + "a" * 64 + ")\n")
            self.assertFalse(receipts(path)["verified_deployment"])
            with workbench.session_lock(Path(folder)):
                with self.assertRaisesRegex(RuntimeError, "already running"):
                    with workbench.session_lock(Path(folder)):
                        self.fail("second instance admitted")

    def test_helpers_keep_fixed_scripts_and_upload_routes(self):
        self.assertEqual(workbench.helper("upload_switch.py", "$(not-a-shell)")[-1], "$(not-a-shell)")
        with self.assertRaises(ValueError):
            workbench.helper("../bad.py")
        self.assertEqual(dict(workbench.TARGETS)["192.168.1.105"], "silent")

    def test_build_back_cancels_and_build_failure_cannot_upload(self):
        with tempfile.TemporaryDirectory() as folder:
            app = workbench.Workbench(History(Path(folder)))
            with patch.object(workbench, "choose", return_value=0), patch.object(workbench, "ask", return_value=""), patch.object(workbench, "confirm", return_value=False), patch.object(workbench, "command") as run:
                self.assertFalse(app.build())
                run.assert_not_called()

    def test_loop_survives_ftp_denial(self):
        with tempfile.TemporaryDirectory() as folder:
            app = workbench.Workbench(History(Path(folder)))
            with patch.object(workbench, "choose", side_effect=[4, None]), patch.object(app, "sequences", side_effect=ftplib.error_perm("550 denied")), contextlib.redirect_stdout(io.StringIO()):
                app.loop()  # Returns to menu and exits instead of crashing.

    def test_build_failure_does_not_fall_through_to_upload(self):
        with tempfile.TemporaryDirectory() as folder:
            app = workbench.Workbench(History(Path(folder)))
            with patch.object(workbench, "choose", side_effect=[2, None]), patch.object(app, "build", side_effect=RuntimeError("build failed")), patch.object(app, "upload") as upload, contextlib.redirect_stdout(io.StringIO()):
                app.loop()
            upload.assert_not_called()

    def test_build_delegates_original_flags_and_archives_local_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            release = root / "release"
            release.mkdir()
            (release / "config.txt").write_bytes(b"config")
            app = workbench.Workbench(History(root / "state"))
            with patch.object(workbench, "choose", side_effect=[0, 0]), patch.object(workbench, "ask", return_value=""), patch.object(workbench, "confirm", return_value=True), patch.object(workbench, "version", return_value="2.2.2"), patch.object(workbench.upload_switch, "selected_source", return_value=release), patch.object(workbench, "command", return_value={"exit_code": 0}) as run, contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(app.build())
            argv = run.call_args.args[0]
            self.assertIn("build_variants.py", Path(argv[1]).name)
            self.assertEqual(argv[-2:], ["--variant", "all"])
            self.assertNotIn("--upload", argv)
            self.assertEqual(app.preferences["last_built_version"], "2.2.2")
            row = app.history.rows()[-1]
            self.assertEqual(len(row["result"]["artifacts"]), 2)
            self.assertEqual(row["status"], "success")

    def test_upload_decline_prevents_write_commands(self):
        with tempfile.TemporaryDirectory() as folder:
            app = workbench.Workbench(History(Path(folder)))
            with patch.object(workbench, "ask", return_value="2.2.2"), patch.object(workbench, "choose", return_value=0), patch.object(workbench, "confirm", return_value=False), patch.object(app, "run") as preview, patch.object(app.history, "perform") as execute:
                app.upload()
            self.assertEqual(preview.call_count, 2)
            self.assertTrue(all("--dry-run" in call.args[1] for call in preview.call_args_list))
            execute.assert_not_called()

    def test_upload_both_attempts_fixed_routes_even_after_one_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            app = workbench.Workbench(History(Path(folder)))
            with patch.object(workbench, "ask", return_value="2.2.2"), patch.object(workbench, "choose", return_value=0), patch.object(workbench, "confirm", return_value=True), patch.object(app, "run"), patch.object(app.history, "perform", side_effect=[RuntimeError("silent unavailable"), {}]) as execute, contextlib.redirect_stdout(io.StringIO()):
                app.upload()
            self.assertEqual(execute.call_count, 2)
            first, second = [call.args[1] for call in execute.call_args_list]
            self.assertEqual((first["host"], first["target"]), ("192.168.1.105", "silent"))
            self.assertIn("-s", first["command"])
            self.assertEqual((second["host"], second["target"]), ("192.168.1.142", "diagnostic"))
            self.assertIn("-d", second["command"])

    def test_real_child_failure_keeps_its_output_and_failed_status(self):
        import sys
        from workbench_history import command
        with tempfile.TemporaryDirectory() as folder:
            history = History(Path(folder))
            argv = [sys.executable, "-c", "print('test child output');raise SystemExit(7)"]
            with contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(RuntimeError, "exited 7"):
                    history.perform("test", {"command": argv}, lambda: command(argv))
            row = history.rows()[-1]
            self.assertEqual(row["status"], "failed")
            self.assertIn("test child output", Path(row["log"]).read_text())
