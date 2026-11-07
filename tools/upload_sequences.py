#!/usr/bin/env python3
"""Validate and upload sequences to both configured Switches; never builds the mod.

Default / --upload: write sequences.upload.txt; preserve active sequences.txt.
--replace --smash-closed: back up and replace active sequences.txt on both.
--dry-run and --check-only never connect, back up, or write to a Switch.

Examples (run from the repository root):
    python3 tools/upload_sequences.py --upload
    python3 tools/upload_sequences.py --replace --smash-closed
    python3 tools/upload_sequences.py --replace --file /path/to/sequences.txt --dry-run

Close Smash on BOTH Switches before active replacement so a game save cannot
race the upload. Replacement reuses the workbench's source/destination rechecks,
local and remote backups, SHA-256 read-back verification and recovery behavior.
The staged copy is not activated by restarting Smash.
"""
from __future__ import annotations

import argparse
import ftplib
import hashlib
import io
from pathlib import Path

from upload_all_switches import TARGETS
from sequence_validation import MAX_BYTES, validate_sequences
from sequence_sources import assemble

SOURCE = Path(__file__).resolve().parents[1] / "deployment/ultimate/kirby_assist/sequences.txt"
REMOTE = "/ultimate/kirby_assist/sequences.upload.txt"
ACTIVE = "/ultimate/kirby_assist/sequences.txt"
BACKUP_ROOT = SOURCE.parents[3] / ".sequence-sync-backups" / "upload-sequences"


def upload_one(host: str, data: bytes, port: int = 5000, *, replace: bool = False,
               source: Path | None = None) -> None:
    if replace:
        # Keep the proven backup/recheck/recovery implementation shared with
        # workbench transfers. Do not write active bytes through the staged path.
        from sequence_transfer import Endpoint, prepare, commit
        variant = next((variant for target, variant in TARGETS if target == host), None)
        if variant is None or source is None:
            raise RuntimeError("Active replacement requires a configured Switch and source file")
        plan = prepare(Endpoint("local", str(source)), Endpoint(variant, ACTIVE, port))
        if plan.source_data != data:
            raise RuntimeError("Source changed since validation; nothing replaced")
        result = commit(plan, BACKUP_ROOT)
        if result.get("backup"):
            print(f"{host}: backup: {result['backup']}", flush=True)
        return
    with ftplib.FTP() as ftp:
        ftp.connect(host, port, timeout=10)
        ftp.login()
        ftp.storbinary(f"STOR {REMOTE}", io.BytesIO(data))
        ftp.voidcmd("TYPE I")
        if ftp.size(REMOTE) != len(data):
            raise RuntimeError("uploaded file size does not match")
        digest = hashlib.sha256()
        ftp.retrbinary(f"RETR {REMOTE}", digest.update)
        if digest.digest() != hashlib.sha256(data).digest():
            raise RuntimeError("uploaded file failed SHA-256 read-back verification")


def upload_both(data: bytes, port: int = 5000, *, replace: bool = False,
                source: Path | None = None) -> bool:
    failures = []
    for host, _variant in TARGETS:
        try:
            if replace:
                upload_one(host, data, port, replace=True, source=source)
                print(f"{host}: active sequences.txt replacement verified", flush=True)
            else:
                upload_one(host, data, port)
                print(f"{host}: sequences.upload.txt uploaded and verified; active sequences.txt preserved", flush=True)
        except (OSError, EOFError, ftplib.Error, RuntimeError) as error:
            failures.append(host)
            print(f"{host}: FAILED: {error}", flush=True)
    if failures:
        print("Some uploads failed; both Switches were attempted. Rerun when their FTP servers are open.")
    return not failures


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=5000)
    parser.add_argument("--file", type=Path, default=SOURCE, help="sequences file to validate/upload")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="validate and show paths/IPs without connecting")
    mode.add_argument("--check-only", action="store_true", help="validate locally; no Switch connection")
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--upload", dest="replace", action="store_false", help="upload a separate sequences.upload.txt copy (default)")
    selection.add_argument("--replace", dest="replace", action="store_true", help="back up and replace active sequences.txt on both Switches")
    parser.set_defaults(replace=False)
    parser.add_argument("--smash-closed", action="store_true", help="confirm Smash is fully closed on BOTH Switches; required for --replace writes")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("--port must be 1..65535")
    if args.replace and not args.smash_closed and not (args.dry_run or args.check_only):
        parser.error("Close Smash on BOTH Switches, then use --replace --smash-closed. Use --upload for a separate copy.")
    try:
        if not 0 < args.file.stat().st_size <= MAX_BYTES:
            parser.error("sequences.txt must be nonempty and at most 512 KiB")
        data = assemble(args.file)
        validate_sequences(data)
    except (OSError, RuntimeError) as error:
        parser.error(str(error))
    if args.check_only:
        print("Check complete; no Switch connection or upload.")
        return
    print(f"Source: {args.file}")
    print(f"Destination: {ACTIVE if args.replace else REMOTE}")
    if args.dry_run:
        for host, _variant in TARGETS:
            print(f"Would upload {len(data)} bytes to {host}:{args.port}")
        return
    success = upload_both(data, args.port, replace=True, source=args.file) if args.replace else upload_both(data, args.port)
    if not success:
        raise SystemExit(1)
    if args.replace:
        print("Done. Active sequences.txt verified on both Switches. Start Smash to load it.")
    else:
        print("Done. Active sequences.txt is unchanged. The uploaded copy is sequences.upload.txt; restarting does not activate it.")


if __name__ == "__main__":
    main()
