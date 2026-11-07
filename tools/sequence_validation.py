"""Shared host-only validation of the exact sequence bytes selected for upload."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
MAX_BYTES = 512 * 1024


def validate_sequences(data: bytes) -> None:
    """Use the current mod's Rust parser and catalog checks before any FTP work."""
    if not data.strip() or len(data) > MAX_BYTES:
        raise RuntimeError("sequences.txt must be nonempty and at most 512 KiB; nothing uploaded.")
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--offline", "--manifest-path",
         str(PROJECT_ROOT / "crates/kirby_assist_core/Cargo.toml"),
         "--example", "validate_sequences"],
        input=data, capture_output=True, cwd=PROJECT_ROOT,
    )
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        match = re.search(r"line (\d+):", detail)
        if match:
            lines = data.decode("utf-8", errors="replace").splitlines()
            index = int(match.group(1)) - 1
            if 0 <= index < len(lines):
                section = next((line.strip() for line in reversed(lines[:index + 1])
                                if line.strip().startswith("[")), "")
                detail += f"\n{section}\n  {lines[index].strip()}"
        raise RuntimeError((detail or "Sequence validation failed.") + "\nNothing uploaded.")
    print(result.stdout.decode("utf-8", errors="replace").strip())


def organize_sequences(data: bytes) -> bytes:
    """Format an explicitly transferred master file; never runs during gameplay."""
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--offline", "--manifest-path",
         str(PROJECT_ROOT / "crates/kirby_assist_core/Cargo.toml"),
         "--example", "organize_sequences"],
        input=data, capture_output=True, cwd=PROJECT_ROOT,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.decode("utf-8", errors="replace").strip())
    return result.stdout


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Validate sequences.txt locally; no FTP connection or mod build.")
    parser.add_argument("file", nargs="?", type=Path,
                        default=PROJECT_ROOT / "deployment/ultimate/kirby_assist/sequences.txt",
                        help="file to check (defaults to the deployment sequences.txt)")
    args = parser.parse_args()
    try:
        if not 0 < args.file.stat().st_size <= MAX_BYTES:
            raise RuntimeError("sequences.txt must be nonempty and at most 512 KiB")
        # Bound the read too, in case another program changes the file after stat.
        from sequence_sources import assemble
        validate_sequences(assemble(args.file))
    except (OSError, RuntimeError) as error:
        parser.exit(1, f"{error}\n")
    print(f"Checked: {args.file.resolve()}\nNo Switch connection or upload.")


if __name__ == "__main__":
    main()
