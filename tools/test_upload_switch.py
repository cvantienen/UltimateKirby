"""Standalone uploader tests: temporary input files, fake FTP, no builds/network."""
import contextlib
import ftplib
import io
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest.mock import patch

import upload_switch as builder


class FakeFTP:
    def __init__(self, root="/", missing_parent=False, fail_store=None, corrupt_read=None, include_source=False, fail_backup=False, corrupt_backup=False):
        prefix = root.rstrip("/")
        self.allowed = {
            prefix + "/" + builder.CONFIG.as_posix(),
            prefix + "/" + str(builder.SUMMARY),
            prefix + "/" + builder.PLUGIN.as_posix(),
        }
        if include_source:
            self.allowed.add(prefix + "/" + str(builder.SOURCE_ZIP))
        self.files = {path: b"old content" for path in self.allowed}
        self.files.update({"/unrelated.txt": b"keep", "/ultimate/kirby_assist/status.log": b"keep log"})
        self.original = self.files.copy()
        self.stores = []
        self.parents = []
        self.directories = {str(builder.PurePosixPath(file).parent) for file in self.allowed}
        self.backup_base = prefix + "/ultimate/kirby_assist/bak"
        self.backup_stores = []
        self.events = []
        self.fail_backup = fail_backup
        self.corrupt_backup = corrupt_backup
        self.missing_parent = missing_parent
        self.fail_store = fail_store
        self.corrupt_read = corrupt_read

    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False

    def connect(self, host, port, timeout):
        self.connection = (host, port, timeout)

    def login(self, user, password):
        self.credentials = (user, password)

    def cwd(self, path):
        if self.missing_parent:
            raise ftplib.error_perm("550 Missing parent")
        if path not in self.directories:
            raise ftplib.error_perm("550 No such directory")
        self.parents.append(path)

    def storbinary(self, command, source):
        assert command.startswith("STOR ")
        path = command[5:]
        self.events.append(("store", path))
        if path.startswith(self.backup_base + "/"):
            assert str(builder.PurePosixPath(path).parent) in self.directories
            self.backup_stores.append(path)
            if self.fail_backup:
                raise OSError("Simulated backup write failure")
            self.files[path] = source.read()
            return
        assert path in self.allowed, f"Write outside allowlist: {path}"
        assert self.directories - {self.backup_base} >= {str(builder.PurePosixPath(file).parent) for file in self.allowed}
        self.stores.append(path)
        if len(self.stores) == self.fail_store:
            raise OSError("Simulated connection loss")
        self.files[path] = source.read()

    def retrbinary(self, command, callback):
        assert command.startswith("RETR ")
        path = command[5:]
        self.events.append(("read", path))
        assert path in self.allowed or path.startswith(self.backup_base + "/")
        if path not in self.files:
            raise ftplib.error_perm("550 No such file")
        data = self.files[path]
        corrupt = (self.corrupt_backup and path.startswith(self.backup_base + "/")) or len(self.stores) == self.corrupt_read
        callback(b"wrong data" if corrupt else data)

    def mkd(self, path):
        assert path == self.backup_base or str(builder.PurePosixPath(path).parent) == self.backup_base
        if path in self.directories:
            raise ftplib.error_perm("550 Already exists")
        self.directories.add(path)

    def nlst(self, parent):
        return [path for path in self.files if str(builder.PurePosixPath(path).parent) == parent]


class UploadTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.source = Path(self.temp.name)
        self.binary = b"\0" * 16 + b"NRO0" + b"prepared binary"
        (self.source / "libkirby_assist.nro").write_bytes(self.binary)
        (self.source / "config.txt").write_bytes(b"enabled = true\n")
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"Prepared update")
        self.output = io.StringIO()
        redirect = contextlib.redirect_stdout(self.output)
        redirect.__enter__()
        self.addCleanup(redirect.__exit__, None, None, None)

    def test_only_prepared_three_files_are_uploaded_and_other_files_remain_untouched(self):
        (self.source / "unrelated.nro").write_bytes(b"never upload")
        fake = FakeFTP()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(len(fake.stores), 3)
        self.assertEqual(set(fake.stores), fake.allowed)
        self.assertEqual(fake.files["/" + str(builder.PLUGIN)], self.binary)
        self.assertEqual(fake.files["/" + str(builder.SUMMARY)], b"Prepared update")
        self.assertEqual(set(fake.files) - set(fake.backup_stores), set(fake.original))
        self.assertEqual(len(fake.backup_stores), 3)
        for path in fake.backup_stores:
            self.assertEqual(fake.files[path], b"old content")
        for path in fake.original.keys() - fake.allowed:
            self.assertEqual(fake.files[path], fake.original[path])
        self.assertEqual(fake.connection, ("192.0.2.10", 5000, 30))

    def test_obsolete_fighter_json_is_ignored_and_not_uploaded(self):
        (self.source / "fighter_configs.json").write_bytes(b"no longer parsed, even if invalid")
        fake = FakeFTP()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(len(fake.stores), 3)
        self.assertFalse(any("fighter_configs" in path for path in fake.stores))

    def test_optional_source_zip_is_fourth_allowlisted_file_and_plugin_stays_last(self):
        source_zip = self.source / "kirby_assist-1.1.0-source.zip"
        with zipfile.ZipFile(source_zip, "w") as archive:
            archive.writestr("kirby_assist_v2/src/lib.rs", "// prepared source")
        fake = FakeFTP(include_source=True)
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(len(fake.stores), 4)
        self.assertEqual(set(fake.stores), fake.allowed)
        self.assertEqual(set(fake.backup_stores), {
            "/ultimate/kirby_assist/bak/config.txt",
            "/ultimate/kirby_assist/bak/RELEASE_SUMMARY.txt",
            "/ultimate/kirby_assist/bak/libkirby_assist.nro",
            "/ultimate/kirby_assist/bak/kirby_assist-source.zip",
        })
        self.assertEqual(fake.stores[-1], "/" + str(builder.PLUGIN))
        self.assertEqual(fake.files["/" + str(builder.SOURCE_ZIP)], source_zip.read_bytes())
        fake.stores.clear()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10", verify_only=True)
        self.assertEqual(fake.stores, [])
        self.assertIn("all 4 files verified", self.output.getvalue())

    def test_invalid_or_ambiguous_source_archive_stops_before_connecting(self):
        path = self.source / "kirby_assist-source.zip"
        path.write_bytes(b"not a ZIP")
        with patch.object(builder.ftplib, "FTP") as ftp:
            with self.assertRaisesRegex(RuntimeError, "Invalid source ZIP"):
                builder.upload_files(self.source, "192.0.2.10")
            ftp.assert_not_called()
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr("src/lib.rs", "// source")
        (self.source / "kirby_assist-1.0.0-source.zip").write_bytes(path.read_bytes())
        with self.assertRaisesRegex(RuntimeError, "only one"):
            builder.upload_payloads(self.source)

    def test_diagnostic_variant_is_checked_before_any_connection(self):
        binary = self.binary + b"|".join(builder.LOG_MARKERS)
        (self.source / "libkirby_assist.nro").write_bytes(binary)
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"Kirby Assist 1.1.0\nBuild: DIAGNOSTIC\n\nChanges")
        def connect_after_check():
            self.assertIn("Local build check passed: DIAGNOSTIC", self.output.getvalue())
            return FakeFTP()
        with patch("builtins.input", return_value="yes") as confirmation, \
                patch.object(builder.ftplib, "FTP", side_effect=connect_after_check):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertIn("Are you sure you want to upload diagnostic?", confirmation.call_args.args[0])

    def test_diagnostic_confirmation_decline_or_missing_input_never_connects(self):
        (self.source / "libkirby_assist.nro").write_bytes(self.binary + b"|".join(builder.LOG_MARKERS))
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"Build: DIAGNOSTIC")
        for answer in ["no", "", "y", EOFError(), KeyboardInterrupt()]:
            kwargs = {"side_effect": answer} if isinstance(answer, BaseException) else {"return_value": answer}
            with patch("builtins.input", **kwargs), patch.object(builder.ftplib, "FTP") as ftp:
                builder.upload_files(self.source, "192.0.2.10")
                ftp.assert_not_called()

    def test_diagnostic_dry_run_and_verify_only_do_not_prompt(self):
        binary = self.binary + b"|".join(builder.LOG_MARKERS)
        (self.source / "libkirby_assist.nro").write_bytes(binary)
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"Build: DIAGNOSTIC")
        fake = FakeFTP()
        for remote, data in builder.upload_payloads(self.source):
            fake.files["/" + str(remote)] = data
        with patch("builtins.input") as confirmation, patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10", dry_run=True)
            builder.upload_files(self.source, "192.0.2.10", verify_only=True)
            confirmation.assert_not_called()
        self.assertEqual(fake.stores, [])

    def test_mismatched_partial_or_unlabeled_diagnostic_files_never_connect(self):
        diagnostic = self.binary + b"|".join(builder.LOG_MARKERS)
        for binary, summary in [
            (self.binary, b"Build: DIAGNOSTIC"),
            (diagnostic, b"Build: SILENT"),
            (self.binary + builder.LOG_MARKERS[0], b"Build: DIAGNOSTIC"),
            (diagnostic, b"Unlabeled legacy summary"),
            (diagnostic, b"Build: DIAGNOSTIC\nBuild: SILENT"),
        ]:
            (self.source / "libkirby_assist.nro").write_bytes(binary)
            (self.source / "RELEASE_SUMMARY.txt").write_bytes(summary)
            with patch.object(builder.ftplib, "FTP") as ftp:
                with self.assertRaises(RuntimeError):
                    builder.upload_files(self.source, "192.0.2.10")
                ftp.assert_not_called()

    def test_versioned_summary_is_accepted_but_multiple_summaries_are_rejected(self):
        (self.source / "RELEASE_SUMMARY.txt").rename(self.source / "RELEASE_SUMMARY-9.0.0.txt")
        self.assertEqual(builder.upload_payloads(self.source)[1], (builder.SUMMARY, b"Prepared update"))
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"ambiguous")
        with patch.object(builder.ftplib, "FTP") as ftp:
            with self.assertRaisesRegex(RuntimeError, "exactly one"):
                builder.upload_files(self.source, "192.0.2.10")
            ftp.assert_not_called()

    def test_missing_empty_invalid_or_symlink_files_fail_before_connecting(self):
        nro = self.source / "libkirby_assist.nro"
        for contents in [b"", b"not an NRO"]:
            nro.write_bytes(contents)
            with patch.object(builder.ftplib, "FTP") as ftp:
                with self.assertRaises(RuntimeError):
                    builder.upload_files(self.source, "192.0.2.10")
                ftp.assert_not_called()
        nro.unlink()
        with self.assertRaises(RuntimeError):
            builder.upload_payloads(self.source)
        target = self.source / "other.nro"
        target.write_bytes(self.binary)
        nro.symlink_to(target)
        with self.assertRaises(RuntimeError):
            builder.upload_payloads(self.source)

    def test_dry_run_never_connects(self):
        with patch.object(builder.ftplib, "FTP") as ftp:
            builder.upload_files(self.source, "192.0.2.10", dry_run=True)
            ftp.assert_not_called()

    def test_all_files_are_snapshotted_before_ftp_connects(self):
        fake = FakeFTP()
        def connection():
            (self.source / "libkirby_assist.nro").write_bytes(b"changed after validation")
            return fake
        with patch.object(builder.ftplib, "FTP", side_effect=connection):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(fake.files["/" + str(builder.PLUGIN)], self.binary)

    def test_missing_remote_parent_leaves_every_remote_file_unchanged(self):
        fake = FakeFTP(missing_parent=True)
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            with self.assertRaisesRegex(RuntimeError, "Destination directory unavailable") as failure:
                builder.upload_files(self.source, "192.0.2.10")
        self.assertIn("No installed files were uploaded or changed", str(failure.exception))
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.files, fake.original)

    def test_interrupted_upload_or_failed_readback_stops_before_plugin(self):
        for fake in [FakeFTP(fail_store=2), FakeFTP(corrupt_read=1)]:
            with patch.object(builder.ftplib, "FTP", return_value=fake):
                with self.assertRaisesRegex(RuntimeError, "FTP update stopped"):
                    builder.upload_files(self.source, "192.0.2.10")
            plugin = "/" + str(builder.PLUGIN)
            self.assertNotIn(plugin, fake.stores)
            self.assertEqual(fake.files[plugin], fake.original[plugin])
            self.assertEqual(fake.files["/unrelated.txt"], b"keep")

    def test_verify_only_reads_all_three_and_never_writes(self):
        fake = FakeFTP()
        for remote, data in builder.upload_payloads(self.source):
            fake.files["/" + str(remote)] = data
        original = fake.files.copy()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10", verify_only=True)
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.files, original)
        self.assertIn("verification complete", self.output.getvalue())

    def test_verify_only_reports_mismatch_without_changes(self):
        fake = FakeFTP()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            with self.assertRaisesRegex(RuntimeError, "No installed files were uploaded or changed"):
                builder.upload_files(self.source, "192.0.2.10", verify_only=True)
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.files, fake.original)

    def test_custom_sd_root_and_credentials_keep_three_fixed_destinations(self):
        fake = FakeFTP(root="/sdcard")
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10", port=21,
                                 root="/sdcard", user="switch", password="secret")
        self.assertEqual(set(fake.stores), fake.allowed)
        self.assertEqual(fake.credentials, ("switch", "secret"))
        self.assertNotIn("secret", self.output.getvalue())

    def test_cli_selects_version_and_variant_without_upload_folder(self):
        options = builder.parse_options(["192.0.2.10", "-v", "1.1.2"])
        self.assertEqual(options.port, 5000)
        self.assertEqual(options.variant, "silent")
        self.assertEqual(options.version, "1.1.2")
        self.assertEqual(builder.parse_options(["192.0.2.10", "-v", "1.1.2", "-d"]).variant, "diagnostic")
        self.assertEqual(builder.parse_options(["192.0.2.10", "-v", "1.1.2", "-s"]).variant, "silent")
        for args in [[], ["192.0.2.10"], ["ftp://192.0.2.10", "-v", "1.1.2"],
                     ["192.0.2.10", "-v", "../../bad"],
                     ["192.0.2.10", "-v", "1.1.2", "-s", "-d"],
                     ["192.0.2.10", "-v", "1.1.2", "--port", "0"],
                     ["192.0.2.10", "-v", "1.1.2", "--root", "/sdcard/../other"]]:
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                builder.parse_options(args)

    def test_selection_uses_exact_latestbuild_folder_and_checks_version_and_variant(self):
        folder = self.source / "Latestbuild-1.2.3" / "silent"
        folder.mkdir(parents=True)
        with patch.object(builder, "DEFAULT_RELEASE_ROOT", self.source):
            self.assertEqual(builder.selected_source("1.2.3", "silent"), folder)
            with self.assertRaisesRegex(RuntimeError, "Selected build folder missing"):
                builder.selected_source("1.2.3", "diagnostic")
        for summary, version, variant in [
            (b"Kirby Assist 1.2.3\nBuild: SILENT\n", "1.2.4", "silent"),
            (b"Kirby Assist 1.2.3\nBuild: SILENT\n", "1.2.3", "diagnostic"),
            (b"Kirby Assist 1.2.3\n", "1.2.3", "silent")]:
            (self.source / "RELEASE_SUMMARY.txt").write_bytes(summary)
            with patch.object(builder.ftplib, "FTP") as ftp:
                with self.assertRaises(RuntimeError):
                    builder.upload_files(self.source, "192.0.2.10", version=version, variant=variant)
                ftp.assert_not_called()
        (self.source / "RELEASE_SUMMARY.txt").write_bytes(b"Kirby Assist 1.2.3\nBuild: SILENT\n")
        with patch.object(builder.ftplib, "FTP") as ftp:
            builder.upload_files(self.source, "192.0.2.10", version="1.2.3", variant="silent", dry_run=True)
            ftp.assert_not_called()

    def test_all_backups_are_verified_before_any_installed_file_is_overwritten(self):
        fake = FakeFTP()
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        first_replace = next(i for i, event in enumerate(fake.events) if event[0] == "store" and event[1] in fake.allowed)
        for backup in fake.backup_stores:
            self.assertLess(fake.events.index(("read", backup)), first_replace)
        first_backups = set(fake.backup_stores)
        previous = {builder.PurePosixPath(path).name: fake.files[path] for path in fake.allowed}
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(set(fake.backup_stores), first_backups)
        self.assertEqual(len(first_backups), 3)
        for path in first_backups:
            self.assertEqual(str(builder.PurePosixPath(path).parent), "/ultimate/kirby_assist/bak")
            self.assertEqual(fake.files[path], previous[builder.PurePosixPath(path).name])

    def test_backup_failure_or_corruption_preserves_all_installed_files(self):
        for fake in [FakeFTP(fail_backup=True), FakeFTP(corrupt_backup=True)]:
            with patch.object(builder.ftplib, "FTP", return_value=fake):
                with self.assertRaisesRegex(RuntimeError, "No installed files were uploaded or changed"):
                    builder.upload_files(self.source, "192.0.2.10")
            self.assertEqual(fake.stores, [])
            for path in fake.allowed:
                self.assertEqual(fake.files[path], fake.original[path])

    def test_missing_source_zip_is_confirmed_absent_and_other_files_are_backed_up(self):
        with zipfile.ZipFile(self.source / "kirby_assist-source.zip", "w") as archive:
            archive.writestr("src/lib.rs", "// source")
        fake = FakeFTP(include_source=True)
        del fake.files["/" + str(builder.SOURCE_ZIP)]
        with patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(len(fake.backup_stores), 3)
        self.assertEqual(len(fake.stores), 4)

    def test_unreadable_existing_file_is_not_mistaken_for_missing(self):
        fake = FakeFTP()
        original_retr = fake.retrbinary
        def deny_existing(command, callback):
            if command == "RETR /" + str(builder.CONFIG):
                raise ftplib.error_perm("550 Permission denied")
            original_retr(command, callback)
        with patch.object(fake, "retrbinary", side_effect=deny_existing), patch.object(builder.ftplib, "FTP", return_value=fake):
            with self.assertRaisesRegex(RuntimeError, "cannot be read for backup"):
                builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.backup_stores, [])
        self.assertEqual(fake.files, fake.original)

    def test_switch_450_for_missing_file_and_backup_directory_is_supported(self):
        with zipfile.ZipFile(self.source / "kirby_assist-source.zip", "w") as archive:
            archive.writestr("src/lib.rs", "// source")
        fake = FakeFTP(include_source=True)
        del fake.files["/" + str(builder.SOURCE_ZIP)]
        original_retr = fake.retrbinary
        original_cwd = fake.cwd
        def switch_retr(command, callback):
            try:
                original_retr(command, callback)
            except ftplib.error_perm:
                raise ftplib.error_temp("450 No such file or directory")
        def switch_cwd(path):
            try:
                original_cwd(path)
            except ftplib.error_perm:
                raise ftplib.error_temp("450 No such file or directory")
        with patch.object(fake, "retrbinary", side_effect=switch_retr), \
                patch.object(fake, "cwd", side_effect=switch_cwd), \
                patch.object(builder.ftplib, "FTP", return_value=fake):
            builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(len(fake.backup_stores), 3)
        self.assertEqual(len(fake.stores), 4)

    def test_switch_450_does_not_skip_unreadable_existing_file(self):
        fake = FakeFTP()
        with patch.object(fake, "retrbinary", side_effect=ftplib.error_temp("450 File unavailable")), \
                patch.object(builder.ftplib, "FTP", return_value=fake):
            with self.assertRaisesRegex(RuntimeError, "Reading installed file for backup failed: /ultimate/kirby_assist/config.txt"):
                builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.backup_stores, [])
        self.assertEqual(fake.files, fake.original)

    def test_other_temporary_ftp_errors_abort_without_attempting_to_skip_or_create(self):
        fake = FakeFTP()
        with patch.object(fake, "cwd", side_effect=ftplib.error_temp("421 Service unavailable")), \
                patch.object(builder.ftplib, "FTP", return_value=fake):
            with self.assertRaisesRegex(RuntimeError, "Destination directory unavailable"):
                builder.upload_files(self.source, "192.0.2.10")
        self.assertEqual(fake.stores, [])
        self.assertEqual(fake.backup_stores, [])


if __name__ == "__main__":
    unittest.main()
