#!/usr/bin/env python3
"""Upload one prepared release to the project's silent and diagnostic Switches.

Usage:
    python3 tools/upload_all_switches.py -v 1.7.6
    python3 tools/upload_all_switches.py -v 1.7.6 --dry-run
    python3 tools/upload_all_switches.py -v 1.7.6 --replace-sequences --smash-closed

The .105 Switch is permanently restricted to SILENT. The .142 Switch receives
DIAGNOSTIC. Each target is attempted even if another target is unavailable.
Sequences are staged by default; --replace-sequences replaces the active file
after verified backups, using the current assembled fighter configurations.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

import upload_switch


TARGETS = (
    ("192.168.1.105", "silent"),
    ("192.168.1.142", "diagnostic"),
)


def deployment_plan(version: str) -> tuple[tuple[str, str], ...]:
    """Return fixed safe routing; diagnostic can never target .105."""
    upload_switch.release_version(version)
    if any(host.endswith(".105") and variant != "silent" for host, variant in TARGETS):
        raise RuntimeError("Safety rule violated: 192.168.1.105 may receive SILENT only")
    return TARGETS


def parse_options(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--version", required=True, type=upload_switch.release_version)
    parser.add_argument("--replace-sequences", action="store_true", help="back up and replace active sequences.txt on both Switches")
    parser.add_argument("--smash-closed", action="store_true", help="confirm Smash is fully closed on both Switches")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--verify-only", action="store_true")
    options = parser.parse_args(argv)
    if options.replace_sequences and not options.smash_closed and not (options.dry_run or options.verify_only):
        parser.error("Close Smash on BOTH Switches, then use --replace-sequences --smash-closed")
    return options


def main() -> None:
    options = parse_options()
    failures: list[str] = []
    uploader = Path(__file__).with_name("upload_switch.py")
    for host, variant in deployment_plan(options.version):
        print(f"\n=== {host}: {variant.upper()} {options.version} ===", flush=True)
        command = [
            sys.executable,
            str(uploader),
            host,
            "-v",
            options.version,
            "-s" if variant == "silent" else "-d",
        ]
        if options.replace_sequences:
            command.append("--replace-sequences")
        if options.smash_closed:
            command.append("--smash-closed")
        if options.dry_run:
            command.append("--dry-run")
        elif options.verify_only:
            command.append("--verify-only")
        result = subprocess.run(command, check=False)
        if result.returncode != 0:
            failures.append(f"{host} ({variant}): uploader exited {result.returncode}")
            print(f"FAILED: {failures[-1]}", flush=True)
    if failures:
        raise SystemExit("\nDeployment finished with failures:\n  " + "\n  ".join(failures))
    print("\nDeployment complete: every configured Switch verified.", flush=True)


if __name__ == "__main__":
    main()
