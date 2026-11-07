#!/usr/bin/env python3
"""Copy active sequences.txt between the diagnostic and silent Switches.

Close Smash and open FTP on both Switches first. Unlike ordinary uploads, this
explicit sync replaces the destination Switch's ACTIVE file. Source is read-only.
Use --from diagnostic (default) or --from silent to choose which copy to keep.
--fighter kirby replaces only that fighter; omit it to copy the whole file.
--interactive asks for direction and fighter.
--dry-run previews/validates the transfer without writing anything.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import ftplib
import hashlib
import io
import re
from pathlib import Path
import tempfile

from sequence_validation import MAX_BYTES, validate_sequences
from upload_all_switches import TARGETS
from upload_switch import read_existing

ACTIVE = "/ultimate/kirby_assist/sequences.txt"
REMOTE_BACKUP = "/ultimate/kirby_assist/sequences.sync.bak"
BACKUP_ROOT = Path(__file__).resolve().parents[1] / ".sequence-sync-backups"


def targets(source_variant: str = "diagnostic") -> tuple[str, str]:
    if source_variant not in ("diagnostic", "silent"):
        raise RuntimeError("Source must be diagnostic or silent")
    destination_variant = "silent" if source_variant == "diagnostic" else "diagnostic"
    sources = [host for host, variant in TARGETS if variant == source_variant]
    destinations = [host for host, variant in TARGETS if variant == destination_variant]
    if len(sources) != 1 or len(destinations) != 1 or sources[0] == destinations[0]:
        raise RuntimeError("Expected one distinct diagnostic Switch and silent Switch")
    return sources[0], destinations[0]


def fighter_id(value: str) -> str:
    result = re.sub(r"[\s-]+", "_", value.strip().lower())
    if not re.fullmatch(r"[a-z][a-z0-9_]*", result) or result in ("universal", "template"):
        raise RuntimeError("Use a fighter name such as kirby, yoshi or captain_falcon")
    return result


def sections(data: bytes) -> list[tuple[str, int, int]]:
    """Track raw spans, including mixed newlines, without rewriting recipe text."""
    starts = []
    offset = 0
    decoration_start = None
    for line in data.splitlines(keepends=True):
        stripped = line.strip()
        if stripped == b"# ========================================================================" or (stripped.startswith(b"# ---- ") and stripped.endswith(b" ----")):
            if decoration_start is None:
                decoration_start = offset
        header = line.split(b"#", 1)[0].strip()
        if header.startswith(b"[") and header.endswith(b"]"):
            name = header[1:-1].split(b".", 1)[0].decode("utf-8").strip()
            starts.append(("universal" if name.lower() in ("universal", "template") else fighter_id(name), decoration_start if decoration_start is not None else offset))
            decoration_start = None
        offset += len(line)
    return [(name, start, starts[i + 1][1] if i + 1 < len(starts) else len(data))
            for i, (name, start) in enumerate(starts)]


def replace_fighter(source: bytes, destination: bytes, fighter: str) -> tuple[bytes, int, int]:
    fighter = fighter_id(fighter)
    # Legacy universal sections are not fighter configs, but must be preserved.
    source_sections = sections(source)
    selected = [source[start:end] for name, start, end in source_sections if name == fighter]
    if not selected:
        raise RuntimeError(f"Source has no sections for {fighter}; nothing changed")
    destination_sections = sections(destination)
    previous = [destination[start:end] for name, start, end in destination_sections if name == fighter]
    if selected == previous:
        return destination, len(selected), len(previous)
    replacement = b"".join(block if block.endswith(b"\n") else block + b"\n" for block in selected)
    result = bytearray()
    cursor = 0
    removed = 0
    for name, start, end in destination_sections:
        if name != fighter:
            continue
        result.extend(destination[cursor:start])
        if removed == 0:
            result.extend(replacement)
        removed += 1
        cursor = end
    result.extend(destination[cursor:])
    if not removed:
        if result and not result.endswith(b"\n"):
            result.extend(b"\n")
        result.extend(replacement)
    updated = bytes(result)
    if any(line.strip() == b"[universal.settings]" for line in destination.splitlines()):
        from sequence_validation import organize_sequences
        updated = organize_sequences(updated)
    return updated, len(selected), removed


def store_verified(ftp: ftplib.FTP, path: str, data: bytes) -> None:
    ftp.storbinary(f"STOR {path}", io.BytesIO(data))
    digest = hashlib.sha256()
    ftp.retrbinary(f"RETR {path}", digest.update)
    if digest.digest() != hashlib.sha256(data).digest():
        raise RuntimeError(f"SHA-256 read-back verification failed: {path}")


def sync_sequences(*, source_variant: str = "diagnostic", port: int = 5000, dry_run: bool = False,
                   backup_root: Path = BACKUP_ROOT, fighter: str | None = None) -> None:
    fighter = fighter_id(fighter) if fighter is not None else None
    source_host, destination_host = targets(source_variant)
    destination_variant = "silent" if source_variant == "diagnostic" else "diagnostic"
    print("Close Smash on both Switches and keep FTP open before transferring.", flush=True)
    print(f"Scope: {fighter or 'ALL fighters (whole file)'}", flush=True)
    print(f"Source: {source_variant} {source_host}:{port}{ACTIVE}", flush=True)
    print(f"Destination: {destination_variant} {destination_host}:{port}{ACTIVE}", flush=True)
    data = bytearray()

    def receive(block: bytes) -> None:
        if len(data) + len(block) > MAX_BYTES:
            raise RuntimeError("Source sequences.txt exceeds 512 KiB; nothing changed")
        data.extend(block)

    with ftplib.FTP() as ftp:
        ftp.connect(source_host, port, timeout=15)
        ftp.login()
        ftp.retrbinary(f"RETR {ACTIVE}", receive)
    source = bytes(data)
    validate_sequences(source)
    if dry_run and fighter is None:
        print(f"Dry run: {source_variant} file validated; no {destination_variant} connection, backups or writes.")
        return
    with ftplib.FTP() as ftp:
        ftp.connect(destination_host, port, timeout=15)
        ftp.login()
        original = read_existing(ftp, ACTIVE)
        if fighter is not None:
            if original is None:
                raise RuntimeError("Destination active file is missing; fighter transfer stopped")
            validate_sequences(original)
            updated, added, removed = replace_fighter(source, original, fighter)
            validate_sequences(updated)
            print(f"{fighter}: replace {removed} destination sections with {added} source sections; other fighters preserved.", flush=True)
            if dry_run:
                print("Dry run: merged file validated; no backups or Switch writes.")
                return
        else:
            updated = source
        if original == updated:
            print(f"{destination_variant.capitalize()} already matches the requested transfer; nothing changed.")
            return
        # Keep unique local snapshots before any SD write, including if backup
        # publication or the active transfer subsequently fails.
        backup_root.mkdir(parents=True, exist_ok=True)
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-")
        snapshot = Path(tempfile.mkdtemp(prefix=stamp, dir=backup_root))
        (snapshot / f"{source_variant}-sequences.txt").write_bytes(source)
        if original is not None:
            (snapshot / f"{destination_variant}-sequences.txt").write_bytes(original)
        (snapshot / "transfer-result.txt").write_bytes(updated)
        print(f"Local snapshots: {snapshot}", flush=True)
        # Reject a concurrent in-game/FTP edit before touching the active file.
        if read_existing(ftp, ACTIVE) != original:
            raise RuntimeError("Destination changed during preparation; nothing overwritten")
        if original is not None:
            # A failed backup must never be followed by an active write.
            try:
                store_verified(ftp, REMOTE_BACKUP, original)
            except (OSError, EOFError, ftplib.Error, RuntimeError) as error:
                raise RuntimeError(f"{destination_variant.capitalize()} backup failed; its active file is unchanged. {error}") from error
        if read_existing(ftp, ACTIVE) != original:
            raise RuntimeError("Destination changed before replacement; nothing overwritten")
        try:
            store_verified(ftp, ACTIVE, updated)
        except (OSError, EOFError, ftplib.Error, RuntimeError, KeyboardInterrupt) as error:
            if original is not None:
                try:
                    store_verified(ftp, ACTIVE, original)
                except (OSError, EOFError, ftplib.Error, RuntimeError) as restore_error:
                    raise RuntimeError(
                        f"Sync failed ({error}); restoring {destination_variant} also failed ({restore_error}). "
                        f"Its active file may be incomplete. Restore {snapshot / (destination_variant + '-sequences.txt')} "
                        f"or SD {REMOTE_BACKUP} before starting Smash."
                    ) from error
                raise RuntimeError(f"Sync failed; original {destination_variant} sequences restored and verified: {error}") from error
            raise RuntimeError(
                f"Sync failed; {destination_variant} had no previous active file. Its new file may be incomplete. "
                f"Retry before starting Smash. Source snapshot: {snapshot}. {error}"
            ) from error
    scope = f"{fighter} recipes match" if fighter else "active sequences.txt matches"
    print(f"Done: {destination_variant} {scope} {source_variant} (SHA-256 verified). Restart Smash on {destination_variant}.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--from", dest="source_variant", choices=("diagnostic", "silent"),
                        default=None, help="Switch with the copy to keep (default: diagnostic)")
    parser.add_argument("--fighter", help="replace all sections for only this fighter, e.g. kirby")
    parser.add_argument("--interactive", action="store_true", help="ask for transfer direction and fighter")
    parser.add_argument("--port", type=int, default=5000)
    parser.add_argument("--dry-run", action="store_true", help="validate/preview; fighter transfers read both Switches, never write")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("--port must be 1..65535")
    if args.interactive:
        if args.source_variant or args.fighter:
            parser.error("Use --interactive alone, or specify --from and --fighter")
        try:
            direction = input("Transfer: 1 = diagnostic -> silent; 2 = silent -> diagnostic: ").strip()
            if direction not in ("1", "2"):
                parser.error("Choose 1 or 2; nothing transferred")
            args.source_variant = "diagnostic" if direction == "1" else "silent"
            args.fighter = input("Fighter (e.g. kirby, yoshi, captain_falcon): ").strip()
            if not args.fighter:
                parser.error("A fighter is required; nothing transferred")
        except (EOFError, KeyboardInterrupt):
            parser.exit(1, "Transfer cancelled; nothing transferred.\n")
    try:
        sync_sequences(source_variant=args.source_variant or "diagnostic", port=args.port,
                       dry_run=args.dry_run, fighter=args.fighter)
    except (OSError, EOFError, ftplib.Error, RuntimeError) as error:
        parser.exit(1, f"Sequence sync failed: {error}\n")


if __name__ == "__main__":
    main()
