#!/usr/bin/env python3
"""Upload a selected built version after backing up installed files on the SD.

Examples (run from the project root; default FTP port 5000):
    python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s
    python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -d
    python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s --dry-run
    python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s --verify-only
    python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s --root /sdcard

Reads AutoSmashMod/Latestbuild-<version>/silent or /diagnostic (default: silent).
Never builds, extracts archives, or reads the old UPLOAD folder.
Close Smash and start the Switch FTP server first.
Diagnostic uploads still require typing yes before connecting.
Before replacing anything, retrieve all existing selected destination files,
copy them to /ultimate/kirby_assist/bak/, and read back every backup to
verify SHA-256. Failed backup means no installed files are overwritten.
Missing files have nothing to back up; directory listings must confirm absence.
Each run overwrites the previous backup files in that same flat directory.
Only selected installed files and their backup copies are written. No directory sync, deletion, or rename.
--dry-run never connects; --verify-only reads without making backups or changes.
"""
from __future__ import annotations

import argparse
import ftplib
import hashlib
import io
import ipaddress
import os
import re
import zipfile
from pathlib import Path, PurePosixPath

DEFAULT_RELEASE_ROOT = Path(__file__).resolve().parents[2]
PLUGIN = PurePosixPath("atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro")
CONFIG = PurePosixPath("ultimate/kirby_assist/config.txt")
SUMMARY = PurePosixPath("ultimate/kirby_assist/RELEASE_SUMMARY.txt")
SOURCE_ZIP = PurePosixPath("ultimate/kirby_assist/kirby_assist-source.zip")

# Match the builder's explicit diagnostic markers; no Cargo/build imports needed.
LOG_MARKERS = (
    b"sd:/ultimate/kirby_assist/status.log",
    b"[kirby_assist]",
    b"tech observation:",
    b"ledge observation:",
    b"ledge mapper:",
    b"tech ZR applied:",
    b"NOT INSTALLED:",
)


def verify_build_variant(binary: bytes, summary: bytes) -> str:
    """Reject mislabeled or partially diagnostic prepared NROs before FTP."""
    try:
        text = summary.decode("utf-8")
    except UnicodeDecodeError as error:
        raise RuntimeError("Release summary must be UTF-8 text") from error
    labels = re.findall(r"^Build:[ \t]*([^\r\n]+)", text, re.MULTILINE | re.IGNORECASE)
    if len(labels) > 1:
        raise RuntimeError("Release summary contains multiple Build labels")
    expected = labels[0].strip().lower() if labels else None
    if expected not in (None, "silent", "diagnostic"):
        raise RuntimeError("Release summary Build label must be SILENT or DIAGNOSTIC")
    present = [marker in binary for marker in LOG_MARKERS]
    if any(present) and not all(present):
        missing = ", ".join(marker.decode() for marker, found in zip(LOG_MARKERS, present) if not found)
        raise RuntimeError(f"Incomplete diagnostic NRO markers; refusing upload. Missing: {missing}")
    actual = "diagnostic" if all(present) else "silent"
    if expected is not None and actual != expected:
        raise RuntimeError(f"Build mismatch: summary says {expected.upper()}, NRO markers indicate {actual.upper()}")
    if actual == "diagnostic" and expected is None:
        raise RuntimeError("Diagnostic NRO requires a matching summary with Build: DIAGNOSTIC; copy both from the diagnostic folder")
    print(f"Local build check passed: {actual.upper()} NRO "
          f"({'all diagnostic markers present' if actual == 'diagnostic' else 'diagnostic markers absent'}).", flush=True)
    return actual


def ftp_root(value: str) -> str:
    if (not value.startswith("/") or "\\" in value
            or any(ord(char) < 32 or ord(char) == 127 for char in value)
            or any(part in (".", "..") for part in value.split("/"))):
        raise argparse.ArgumentTypeError("FTP root must be an absolute path without traversal/control characters")
    return str(PurePosixPath(value))


def ftp_ip(value: str) -> str:
    try:
        return str(ipaddress.ip_address(value))
    except ValueError as error:
        raise argparse.ArgumentTypeError("Supply the Switch IP address, without ftp:// or a port") from error


def upload_payloads(source: Path) -> tuple[tuple[PurePosixPath, bytes], ...]:
    """Read explicit flat-folder files and an optional source ZIP; never extract."""
    summary = source / "RELEASE_SUMMARY.txt"
    candidates = list(source.glob("RELEASE_SUMMARY-*.txt"))
    if summary.exists() or summary.is_symlink():
        candidates.append(summary)
    if len(candidates) != 1:
        raise RuntimeError(f"Place exactly one RELEASE_SUMMARY.txt or RELEASE_SUMMARY-<version>.txt in {source}")
    paths = (source / "config.txt", candidates[0], source / "libkirby_assist.nro")
    payloads = []
    for path, remote in zip(paths, (CONFIG, SUMMARY, PLUGIN)):
        if path.is_symlink() or not path.is_file():
            raise RuntimeError(f"Required upload file is missing, not a regular file, or a symlink: {path}")
        data = path.read_bytes()
        if not data.strip():
            raise RuntimeError(f"Upload file is empty: {path}")
        payloads.append((remote, data))
    if payloads[-1][1][0x10:0x14] != b"NRO0":
        raise RuntimeError("libkirby_assist.nro does not have the expected NRO header")
    verify_build_variant(payloads[-1][1], payloads[1][1])
    # Fighter responses/combos are compiled. Ignore obsolete profile files in
    # older prepared folders; they are not part of the upload allowlist.
    archives = list(source.glob("kirby_assist-*-source.zip"))
    stable_archive = source / "kirby_assist-source.zip"
    if stable_archive.exists() or stable_archive.is_symlink():
        archives.append(stable_archive)
    if len(archives) > 1:
        raise RuntimeError("Keep only one kirby_assist source ZIP in the selected build folder; refusing to guess")
    if archives:
        archive_path = archives[0]
        if archive_path.is_symlink() or not archive_path.is_file():
            raise RuntimeError(f"Source ZIP must be a regular file, not a symlink: {archive_path}")
        archive_bytes = archive_path.read_bytes()
        try:
            with zipfile.ZipFile(io.BytesIO(archive_bytes)) as archive:
                if not archive.namelist() or archive.testzip() is not None:
                    raise RuntimeError("Source ZIP is empty or has a CRC failure")
        except (zipfile.BadZipFile, RuntimeError, NotImplementedError) as error:
            raise RuntimeError(f"Invalid source ZIP: {archive_path}: {error}") from error
        payloads.insert(-1, (SOURCE_ZIP, archive_bytes))
    # Freeze byte snapshots before connecting. Read the prepared source ZIP as
    # one file; never extract it, build a new archive, or regenerate a release.
    return tuple(payloads)


def release_version(value: str) -> str:
    if not re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,3}(?:[-+][A-Za-z0-9][A-Za-z0-9.-]*)?", value):
        raise argparse.ArgumentTypeError("Version must look like 1.1.2, without a path or Latestbuild- prefix")
    return value


def selected_source(version: str, variant: str) -> Path:
    source = DEFAULT_RELEASE_ROOT / f"Latestbuild-{version}" / variant
    if not source.is_dir() or source.is_symlink() or source.parent.is_symlink():
        raise RuntimeError(f"Selected build folder missing or a symlink: {source}. Build this version/variant first.")
    return source


def verify_selection(payloads, version: str | None, variant: str | None) -> None:
    summary = next(data for remote, data in payloads if remote == SUMMARY).decode("utf-8")
    if variant is not None:
        labels = re.findall(r"^Build:[ \t]*([^\r\n]+)", summary, re.MULTILINE | re.IGNORECASE)
        if len(labels) != 1 or labels[0].strip().lower() != variant:
            raise RuntimeError(f"Selected {variant.upper()} folder must contain a matching Build label")
    if version is not None:
        versions = re.findall(r"^Kirby Assist[ \t]+([^\s]+)[ \t]*$", summary, re.MULTILINE)
        if versions != [version]:
            raise RuntimeError(f"Selected version {version} does not match the release summary")


def read_existing(ftp: ftplib.FTP, remote: str) -> bytes | None:
    data = io.BytesIO()
    try:
        ftp.retrbinary(f"RETR {remote}", data.write)
    except (ftplib.error_perm, ftplib.error_temp) as error:
        if str(error)[:3] not in ("450", "550"):
            raise
        # Switch servers may use 450 or 550 for missing files, but also for
        # permission/transient errors. Only skip when a successful parent
        # listing confirms this filename is absent; never silently skip a denial.
        name = PurePosixPath(remote).name
        entries = ftp.nlst(str(PurePosixPath(remote).parent))
        if any(PurePosixPath(entry.rstrip("/")).name == name for entry in entries):
            raise RuntimeError(f"Existing destination cannot be read for backup: {remote}") from error
        print(f"Not installed, no backup needed: {remote}", flush=True)
        return None
    return data.getvalue()


def backup_installed(ftp: ftplib.FTP, plan, backup: PurePosixPath) -> None:
    # Fetch every original before making any backup or replacing any installed file.
    originals = []
    for remote, _ in plan:
        try:
            data = read_existing(ftp, remote)
        except (OSError, EOFError, ftplib.Error, RuntimeError) as error:
            raise RuntimeError(f"Reading installed file for backup failed: {remote}: {error}") from error
        if data is not None:
            originals.append((remote, data))
    if not originals:
        print("No selected files previously installed; no backup required.", flush=True)
        return
    base = backup
    try:
        ftp.cwd(str(base))
    except (ftplib.error_perm, ftplib.error_temp) as error:
        if str(error)[:3] not in ("450", "550"):
            raise RuntimeError(f"Checking backup directory failed: {base}: {error}") from error
        try:
            ftp.mkd(str(base))
        except ftplib.Error as mkdir_error:
            raise RuntimeError(f"Creating backup directory failed: {base}: {mkdir_error}") from mkdir_error
    print(f"Backing up installed files to {backup}", flush=True)
    for remote, data in originals:
        destination = str(backup / PurePosixPath(remote).name)
        ftp.storbinary(f"STOR {destination}", io.BytesIO(data))
        digest = hashlib.sha256()
        ftp.retrbinary(f"RETR {destination}", digest.update)
        if digest.digest() != hashlib.sha256(data).digest():
            raise RuntimeError(f"Backup read-back verification failed: {destination}")
        print(f"Backed up and verified {remote} -> {destination}", flush=True)
    print("All existing selected files backed up and verified; starting replacements.", flush=True)


def upload_files(source: Path, host: str, *, port: int = 5000, root: str = "/",
                 user: str = "anonymous", password: str = "", dry_run: bool = False,
                 verify_only: bool = False, version: str | None = None,
                 variant: str | None = None) -> None:
    root_path = PurePosixPath(ftp_root(root))
    payloads = upload_payloads(source)
    verify_selection(payloads, version, variant)
    plan = tuple((str(root_path / remote), data) for remote, data in payloads)
    operation = "verification" if verify_only else "update"
    backup = root_path / "ultimate/kirby_assist/bak"
    print(f"Source: {source}\nFTP {operation} to {host}:{port}; {len(plan)} selected files:", flush=True)
    for remote, data in plan:
        print(f"  {remote} ({len(data):,} bytes; SHA-256 {hashlib.sha256(data).hexdigest()})", flush=True)
    if not verify_only:
        print(f"Existing selected files will be backed up and verified in: {backup}", flush=True)
    if dry_run:
        print("Dry run: no FTP connection or remote changes.", flush=True)
        return
    binary = next(data for remote, data in payloads if remote == PLUGIN)
    if not verify_only and all(marker in binary for marker in LOG_MARKERS):
        try:
            answer = input("Are you sure you want to upload diagnostic? Type yes to continue [default: no]: ")
        except (EOFError, KeyboardInterrupt):
            answer = ""
        if answer.strip().lower() != "yes":
            print("Upload cancelled. No FTP connection or remote changes.", flush=True)
            return
    total = len(plan)
    completed: list[str] = []
    current: str | None = None
    upload_started = False
    try:
        with ftplib.FTP() as ftp:
            ftp.connect(host, port, timeout=30)
            ftp.login(user, password)
            # Installed destination directories must already exist.
            for parent in sorted({str(PurePosixPath(path).parent) for path, _ in plan}):
                try:
                    ftp.cwd(parent)
                except (ftplib.error_perm, ftplib.error_temp) as error:
                    raise RuntimeError(
                        f"Destination directory unavailable: {parent}. "
                        "Check the FTP root/permissions; if missing, create this directory once manually. "
                        f"Server: {error}"
                    ) from error
            if not verify_only:
                backup_installed(ftp, plan, backup)
            for remote, data in plan:  # Plugin last, after config and summary.
                current = remote
                if not verify_only:
                    upload_started = True
                    ftp.storbinary(f"STOR {remote}", io.BytesIO(data))
                digest = hashlib.sha256()
                ftp.retrbinary(f"RETR {remote}", digest.update)
                if digest.digest() != hashlib.sha256(data).digest():
                    raise RuntimeError(f"Read-back verification failed for {remote}")
                completed.append(remote)
                print(f"{'Verified' if verify_only else 'Uploaded and verified'} {remote}", flush=True)
                current = None
    except (OSError, EOFError, ftplib.Error, RuntimeError) as error:
        state = ("The selected destinations may be partially updated. " if upload_started
                 else "No installed files were uploaded or changed. ")
        raise RuntimeError(
            f"FTP {operation} stopped; verified {len(completed)}/{total} files. "
            f"{state}Current file: {current or 'none (preflight/connection/close)'}. "
            f"Backup location (may be incomplete if backup failed): {backup}. "
            f"Only selected destinations and this backup directory may have been written. {error}"
        ) from error
    print(f"FTP {operation} complete: all {total} files verified by read-back SHA-256.", flush=True)



def parse_options(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ip", type=ftp_ip, help="Switch FTP IP address")
    parser.add_argument("-v", "--version", required=True, type=release_version,
                        help="Built version to select, e.g. 1.1.2")
    variants = parser.add_mutually_exclusive_group()
    variants.add_argument("-s", "--silent", dest="variant", action="store_const", const="silent",
                          help="Upload silent variant (default)")
    variants.add_argument("-d", "--diagnostic", dest="variant", action="store_const", const="diagnostic",
                          help="Upload diagnostic variant (requires confirmation)")
    parser.set_defaults(variant="silent")
    parser.add_argument("--port", type=int, default=5000, help="FTP port (default: 5000)")
    parser.add_argument("--root", type=ftp_root, default="/", help="Server's SD-root prefix (default: /)")
    parser.add_argument("--user", default="anonymous", help="FTP login (default: anonymous)")
    parser.add_argument("--password-env", help="Read FTP password from this environment variable")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="Check files and print paths without connecting")
    mode.add_argument("--verify-only", action="store_true", help="Compare remote files to the selected build without writing anything")
    options = parser.parse_args(argv)
    if not 1 <= options.port <= 65535:
        parser.error("--port must be 1..65535")
    if options.password_env and options.password_env not in os.environ:
        parser.error(f"Password environment variable is unset: {options.password_env}")
    return options


def main() -> None:
    options = parse_options()
    try:
        upload_files(selected_source(options.version, options.variant), options.ip, port=options.port,
                     root=options.root, user=options.user,
                     password=os.environ[options.password_env] if options.password_env else "",
                     dry_run=options.dry_run, verify_only=options.verify_only,
                     version=options.version, variant=options.variant)
    except (OSError, RuntimeError) as error:
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
