import contextlib
import ftplib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import sync_sequences as sync

SOURCE = b"[yoshi.roll.grab]\nstart_delay=0\nactions=grab\n"
ORIGINAL = b"# silent custom recipes\n"


class FakeFTP:
    def __init__(self, data, *, readonly=False, corrupt_backup=False, fail_active=False,
                 corrupt_active=False, fail_restore=False):
        self.files = {sync.ACTIVE: data}
        self.readonly = readonly
        self.corrupt_backup = corrupt_backup
        self.fail_active = fail_active
        self.corrupt_active = corrupt_active
        self.fail_restore = fail_restore
        self.commands = []
        self.active_writes = 0

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False

    def connect(self, host, port, timeout):
        self.connection = (host, port)

    def login(self):
        pass

    def retrbinary(self, command, consume):
        self.commands.append(command)
        path = command.removeprefix("RETR ")
        data = self.files[path]
        if path == sync.REMOTE_BACKUP and self.corrupt_backup:
            data = b"bad backup"
        if path == sync.ACTIVE and self.active_writes == 1 and self.corrupt_active:
            data = b"bad active"
        consume(data)

    def storbinary(self, command, source):
        assert not self.readonly, "Diagnostic must never be written"
        self.commands.append(command)
        path = command.removeprefix("STOR ")
        data = source.read()
        if path == sync.ACTIVE:
            self.active_writes += 1
            if self.active_writes == 1 and self.fail_active:
                self.files[path] = data[:5]
                raise OSError("connection lost")
            if self.active_writes > 1 and self.fail_restore:
                raise OSError("restore unavailable")
        self.files[path] = data


class SequenceSyncTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.backups = Path(self.temp.name) / "snapshots"
        self.source = FakeFTP(SOURCE, readonly=True)
        self.destination = FakeFTP(ORIGINAL)

    def run_sync(self, *, dry_run=False, invalid=False, source_variant="diagnostic"):
        with patch.object(sync.ftplib, "FTP", side_effect=[self.source, self.destination]) as connect, \
                patch.object(sync, "validate_sequences", side_effect=RuntimeError("invalid sequence") if invalid else None) as validate, \
                contextlib.redirect_stdout(io.StringIO()):
            sync.sync_sequences(source_variant=source_variant, dry_run=dry_run, backup_root=self.backups)
        validate.assert_called_once_with(SOURCE)
        return connect.call_count

    def test_copies_diagnostic_active_to_silent_after_verified_backup(self):
        self.assertEqual(self.run_sync(), 2)
        self.assertEqual(self.source.connection, ("192.168.1.142", 5000))
        self.assertEqual(self.destination.connection, ("192.168.1.105", 5000))
        self.assertEqual(self.source.commands, [f"RETR {sync.ACTIVE}"])
        self.assertEqual(self.destination.files[sync.ACTIVE], SOURCE)
        self.assertEqual(self.destination.files[sync.REMOTE_BACKUP], ORIGINAL)
        self.assertLess(self.destination.commands.index(f"RETR {sync.REMOTE_BACKUP}"),
                        self.destination.commands.index(f"STOR {sync.ACTIVE}"))
        snapshot, = self.backups.iterdir()
        self.assertEqual((snapshot / "diagnostic-sequences.txt").read_bytes(), SOURCE)
        self.assertEqual((snapshot / "silent-sequences.txt").read_bytes(), ORIGINAL)

    def test_dry_run_only_downloads_and_validates_without_writing(self):
        self.assertEqual(self.run_sync(dry_run=True), 1)
        self.assertFalse(self.backups.exists())
        self.assertEqual(self.destination.commands, [])

    def test_reverse_sync_keeps_silent_source_and_backs_up_diagnostic_destination(self):
        self.assertEqual(self.run_sync(source_variant="silent"), 2)
        self.assertEqual(self.source.connection, ("192.168.1.105", 5000))
        self.assertEqual(self.destination.connection, ("192.168.1.142", 5000))
        self.assertEqual(self.source.commands, [f"RETR {sync.ACTIVE}"])
        self.assertEqual(self.destination.files[sync.ACTIVE], SOURCE)
        self.assertEqual(self.destination.files[sync.REMOTE_BACKUP], ORIGINAL)
        snapshot, = self.backups.iterdir()
        self.assertEqual((snapshot / "silent-sequences.txt").read_bytes(), SOURCE)
        self.assertEqual((snapshot / "diagnostic-sequences.txt").read_bytes(), ORIGINAL)

    def test_reverse_failure_restores_diagnostic_and_reverse_dry_run_never_connects_to_it(self):
        self.assertEqual(self.run_sync(source_variant="silent", dry_run=True), 1)
        self.assertFalse(self.backups.exists())
        self.destination.fail_active = True
        with self.assertRaisesRegex(RuntimeError, "original diagnostic sequences restored and verified"):
            self.run_sync(source_variant="silent")
        self.assertEqual(self.destination.files[sync.ACTIVE], ORIGINAL)

    def test_invalid_direction_stops_before_any_connection(self):
        with patch.object(sync.ftplib, "FTP") as ftp:
            with self.assertRaisesRegex(RuntimeError, "Source must be"):
                sync.sync_sequences(source_variant="newest")
        ftp.assert_not_called()

    def test_invalid_source_never_connects_to_silent(self):
        with self.assertRaisesRegex(RuntimeError, "invalid sequence"):
            self.run_sync(invalid=True)
        self.assertEqual(self.destination.commands, [])
        self.assertFalse(self.backups.exists())

    def test_failed_backup_leaves_active_untouched(self):
        self.destination.corrupt_backup = True
        with self.assertRaisesRegex(RuntimeError, "SHA-256"):
            self.run_sync()
        self.assertEqual(self.destination.active_writes, 0)
        self.assertEqual(self.destination.files[sync.ACTIVE], ORIGINAL)

    def test_partial_write_and_bad_readback_restore_original(self):
        for failure in ["fail_active", "corrupt_active"]:
            with self.subTest(failure=failure):
                self.destination = FakeFTP(ORIGINAL, **{failure: True})
                with self.assertRaisesRegex(RuntimeError, "original silent sequences restored and verified"):
                    self.run_sync()
                self.assertEqual(self.destination.files[sync.ACTIVE], ORIGINAL)

    def test_failed_restore_reports_manual_recovery_and_keeps_backups(self):
        self.destination = FakeFTP(ORIGINAL, fail_active=True, fail_restore=True)
        with self.assertRaisesRegex(RuntimeError, "Restore .*silent-sequences.txt"):
            self.run_sync()
        self.assertEqual(self.destination.files[sync.REMOTE_BACKUP], ORIGINAL)
        snapshot, = self.backups.iterdir()
        self.assertEqual((snapshot / "silent-sequences.txt").read_bytes(), ORIGINAL)

    def test_identical_files_skip_all_writes_and_backups(self):
        self.destination = FakeFTP(SOURCE)
        self.run_sync()
        self.assertEqual(self.destination.commands, [f"RETR {sync.ACTIVE}"])
        self.assertFalse(self.backups.exists())

    def test_local_backup_failure_prevents_sd_writes(self):
        self.backups.write_bytes(b"not a directory")
        with self.assertRaises(OSError):
            self.run_sync()
        self.assertEqual(self.destination.commands, [f"RETR {sync.ACTIVE}"])

    def test_concurrent_destination_change_is_not_overwritten(self):
        original_read = self.destination.retrbinary
        reads = 0
        def changed(command, consume):
            nonlocal reads
            if command == f"RETR {sync.ACTIVE}":
                reads += 1
                if reads == 2:
                    self.destination.files[sync.ACTIVE] = b"# newer edit\n"
            return original_read(command, consume)
        self.destination.retrbinary = changed
        with self.assertRaisesRegex(RuntimeError, "Destination changed"):
            self.run_sync()
        self.assertEqual(self.destination.files[sync.ACTIVE], b"# newer edit\n")
        self.assertEqual(self.destination.active_writes, 0)


class FighterTransferTests(unittest.TestCase):
    def test_replaces_all_target_categories_and_buttons_preserving_other_sections(self):
        source = b"# source\n[kirby.neutral.new]\nactions=a\n[yoshi.roll.keep]\nactions=b\n[kirby.x.new]\nactions=x\n"
        destination = b"# destination\n[kirby.roll.old]\nactions=r\n[yoshi.roll.perfect]\nname=Perfect\nactions=b\n[kirby.y.old]\nactions=y\n[ness.jump.keep]\nactions=x\n"
        merged, added, removed = sync.replace_fighter(source, destination, "Kirby")
        self.assertEqual((added, removed), (2, 2))
        self.assertEqual(merged, b"# destination\n[kirby.neutral.new]\nactions=a\n[kirby.x.new]\nactions=x\n[yoshi.roll.perfect]\nname=Perfect\nactions=b\n[ness.jump.keep]\nactions=x\n")
        self.assertNotIn(b"kirby.roll.old", merged)
        self.assertNotIn(b"yoshi.roll.keep", merged)

    def test_missing_source_refuses_to_erase_destination_and_prefixes_do_not_match(self):
        with self.assertRaisesRegex(RuntimeError, "no sections for kirby"):
            sync.replace_fighter(b"[kirby_extra.x.a]\nactions=a\n", SOURCE, "kirby")
        for name in ["", "kirby.x", "universal", "../kirby"]:
            with self.assertRaises(RuntimeError):
                sync.fighter_id(name)

    def test_adds_absent_fighter_and_handles_crlf_comments_and_legacy_sections(self):
        source = b"  [Captain Falcon.neutral.a] # caption\r\nname=New\r\nactions=a\r\n"
        destination = b"[universal.x.archived]\r\nactions=old\r\n[yoshi.roll.grab]\r\nactions=grab"
        merged, added, removed = sync.replace_fighter(source, destination, "captain-falcon")
        self.assertEqual((added, removed), (1, 0))
        self.assertTrue(merged.startswith(destination + b"\n"))
        self.assertTrue(merged.endswith(source))

    def test_matching_fighter_does_not_reorder_interleaved_sections_or_other_fighters(self):
        source = b"[kirby.x.a]\nactions=a\n[yoshi.x.a]\nactions=b\n[kirby.y.b]\nactions=b\n"
        destination = source.replace(b"[yoshi.x.a]", b"[yoshi.x.different]")
        self.assertEqual(sync.replace_fighter(source, destination, "kirby")[0], destination)

    def transfer(self, *, dry_run=False, invalid_merged=False):
        self.source = FakeFTP(b"[kirby.x.new]\nactions=a\n", readonly=True)
        self.destination = FakeFTP(b"[yoshi.roll.perfect]\nactions=b\n[kirby.y.old]\nactions=x\n")
        self.original = self.destination.files[sync.ACTIVE]
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.backups = Path(self.temp.name) / "snapshots"
        validations = [None, None, RuntimeError("invalid merged") if invalid_merged else None]
        with patch.object(sync.ftplib, "FTP", side_effect=[self.source, self.destination]), \
                patch.object(sync, "validate_sequences", side_effect=validations) as validate, \
                contextlib.redirect_stdout(io.StringIO()):
            sync.sync_sequences(source_variant="silent", fighter="kirby", dry_run=dry_run, backup_root=self.backups)
        self.assertEqual(validate.call_count, 3)

    def test_fighter_transfer_validates_merge_backs_up_and_writes_only_destination(self):
        self.transfer()
        self.assertEqual(self.destination.files[sync.ACTIVE], b"[yoshi.roll.perfect]\nactions=b\n[kirby.x.new]\nactions=a\n")
        self.assertEqual(self.destination.files[sync.REMOTE_BACKUP], self.original)
        self.assertTrue(all(c.startswith("RETR ") for c in self.source.commands))

    def test_fighter_dry_run_reads_both_without_writes_or_backups(self):
        self.transfer(dry_run=True)
        self.assertEqual(self.destination.files[sync.ACTIVE], self.original)
        self.assertFalse(self.backups.exists())
        self.assertFalse(any(c.startswith("STOR ") for c in self.destination.commands))

    def test_invalid_merge_never_writes(self):
        with self.assertRaisesRegex(RuntimeError, "invalid merged"):
            self.transfer(invalid_merged=True)
        self.assertEqual(self.destination.files[sync.ACTIVE], self.original)
        self.assertEqual(self.destination.active_writes, 0)
        self.assertFalse(self.backups.exists())

    def test_interactive_requires_direction_and_fighter(self):
        with patch("sys.argv", ["sync_sequences.py", "--interactive", "--dry-run"]), \
                patch("builtins.input", side_effect=["2", "yoshi"]), \
                patch.object(sync, "sync_sequences") as transfer:
            sync.main()
        transfer.assert_called_once_with(source_variant="silent", port=5000, dry_run=True, fighter="yoshi")


if __name__ == "__main__":
    unittest.main()


class MasterLayoutTests(unittest.TestCase):
    def test_transfer_legacy_fighter_into_master_keeps_other_fighter_and_globals(self):
        destination = (b"[universal.settings]\nenabled = true\n"
                       b"[kirby.neutral.first]\nstart_delay = 0\nactions = grab:4\n"
                       b"[yoshi.roll.old]\nstart_delay = 0\nactions = grab:4\n")
        updated, added, removed = sync.replace_fighter(SOURCE, destination, "yoshi")
        self.assertEqual((added, removed), (1, 1))
        self.assertIn(b"[universal.settings]\nenabled = true", updated)
        self.assertIn(b"[kirby.neutral.first]\nstart_delay = 0\nactions = grab:4", updated)
        self.assertNotIn(b"[yoshi.roll.old]", updated)
        spans = sync.sections(updated)
        kirby = b"".join(updated[a:b] for name, a, b in spans if name == "kirby")
        yoshi = b"".join(updated[a:b] for name, a, b in spans if name == "yoshi")
        self.assertIn(b"KIRBY", kirby)
        self.assertNotIn(b"YOSHI", kirby)
        self.assertIn(b"YOSHI", yoshi)
