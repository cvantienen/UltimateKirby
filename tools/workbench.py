#!/usr/bin/env python3
"""Local GUI and terminal build/upload/sequence workbench. No extra packages.

Usage from the project directory:
    python3 tools/workbench.py
    python3 tools/workbench.py --cli
    python3 tools/workbench.py --history

Default opens a local browser GUI: enter a version (or +1), select Switches,
confirm Smash closed/FTP open once, then click Build + Upload. The build version
is written to Cargo.toml before building. Existing build/upload helpers perform
the work and retain their validation/backups. GUI opening never builds/uploads.
Edit fighter changes one fighter's sections against a single snapshot, with a
diff preview before writing; Sequences has a preview and explicit confirmation
before replacement; Helpers exposes original scripts. --cli keeps the original
numbered prompt below.

In --cli, replace the active sequences.txt on a Switch:
    Choose "Transfer / replace sequences", select the local source file,
    select the destination Switch and "ACTIVE sequences.txt", then choose
    "Whole file". Review the preview and type TRANSFER to confirm replacement.
    Close Smash and leave FTP open before transferring. Whole-file replacement
    overwrites all active recipes/settings; choose "One fighter" instead to
    replace only that fighter while preserving the others. Validation, recovery
    snapshots and destination readback verification run before completion.

Sources and destinations can also be computer files or the other Switch.
STAGED sequences.upload.txt is inactive: uploading a release preserves active
sequences.txt, and restarting does not activate the staged copy.

The prompt stays open until Exit. Local history/preferences/backups live in
.mod-workbench/ (override with --state-dir). Existing standalone helpers remain
available; no build or Switch write occurs merely by opening this launcher.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import ftplib
import json
import os
import ipaddress
from pathlib import Path
import shlex
import sys
import tempfile

import fighter_editor
import sequence_sources as sources
import sequence_transfer as transfer
from toml_compat import tomllib
import upload_switch
from upload_all_switches import TARGETS
from workbench_history import History, ROOT, STATE, command

HELPERS = ("build_variants.py", "upload_all_switches.py", "upload_switch.py",
           "upload_sequences.py", "sync_sequences.py", "sequence_validation.py",
           "sequence_sources.py")
# The assembled bundle is the default transfer file. It is the exact file that
# is deployed and is regenerated from the canonical fighter sources.
DEFAULT_FILE = ROOT / "deployment/ultimate/kirby_assist/assembled/sequences.txt"


def choose(title, items, *, back=True, back_label="Back"):
    print(f"\n{title}")
    for index, item in enumerate(items, 1):
        print(f"  {index}. {item}")
    if back:
        print(f"  0. {back_label}")
    while True:
        answer = input("> ").strip().lower()
        if back and answer in ("0", "b", "back", "q"):
            return None
        if answer.isdigit() and 1 <= int(answer) <= len(items):
            return int(answer) - 1
        print("Choose a listed number.")


def ask(label, default=""):
    value = input(f"{label}{' [' + str(default) + ']' if default != '' else ''}: ").strip()
    return value or str(default)


def confirm(label, token):
    print(label)
    return input(f"Type {token} to continue; anything else cancels: ").strip() == token


def version():
    # Re-read so editing the manifest while the launcher stays open is supported.
    return str(tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"])


def helper(name, *args):
    if name not in HELPERS:
        raise ValueError("Unknown helper")
    return [sys.executable, str(ROOT / "tools" / name), *map(str, args)]


class Workbench:
    def __init__(self, history: History):
        self.history = history
        self.preferences_path = history.directory / "preferences.json"
        self.preferences = {}
        self.noninteractive = False
        if self.preferences_path.exists():
            try:
                self.preferences = json.loads(self.preferences_path.read_text())
                if not isinstance(self.preferences, dict):
                    raise ValueError("preferences must be an object")
            except (ValueError, OSError) as error:
                print(f"Preferences could not be read; using defaults: {error}")
                self.preferences = {}

    def save_preferences(self):
        temp = self.preferences_path.with_suffix(".tmp")
        with temp.open("w") as handle:
            json.dump(self.preferences, handle, indent=2)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp, self.preferences_path)

    def local_path(self, label, default=None, *, force_default=False):
        default = str(default or DEFAULT_FILE)
        saved = default if force_default else self.preferences.get("sequences", default)
        path = Path(ask(label, saved)).expanduser().resolve()
        self.preferences["sequences"] = str(path)
        self.save_preferences()
        return path

    def port(self, target):
        port = int(self.preferences.get(target + "_port", 5000))
        if not 1 <= port <= 65535:
            raise ValueError("FTP port must be 1..65535")
        return port

    def host(self, target):
        saved = self.preferences.get(target + "_host")
        host = str(saved or next(host for host, name in TARGETS if name == target))
        try:
            address = ipaddress.ip_address(host)
        except ValueError as error:
            raise ValueError(f"{target.title()} address must be a valid IP address") from error
        if address.version != 4:
            raise ValueError("Switch addresses must be IPv4")
        if target == "silent" and host == self.host("diagnostic"):
            raise ValueError("Silent and Diagnostic Switch addresses must be different")
        return host

    def targets(self):
        return tuple((self.host(name), name) for name in ("silent", "diagnostic"))

    def prepare_silent_sequences(self):
        """Download and validate Silent's active file without mutating repository files."""
        endpoint = transfer.Endpoint("silent", transfer.ACTIVE, self.port("silent"), self.host("silent"))
        downloaded = transfer.read(endpoint)
        if downloaded is None:
            raise RuntimeError("Silent active sequences.txt is missing; repository was not changed")
        current = sources.assemble(sources.SOURCE)
        restored = sources.restore_comments(current, downloaded)
        transfer.validate_sequences(restored)
        plan = sources.plan_breakdown(sources.SOURCE, restored)
        plan["origin"] = endpoint.label()
        plan["original"] = current
        plan["original_files"] = {str(path): path.read_bytes() for path in [sources.SOURCE, *sources.SOURCE.parent.joinpath('fighters').glob('*.txt')] if path.is_file()}
        plan["original_fighters"] = {str(path): data for path, data in ((Path(name), value) for name, value in plan["original_files"].items()) if path != sources.SOURCE}
        return {key: value for key, value in plan.items() if key != "original_files"}

    def rollback_silent_sequences(self, plan):
        """Restore split sources and assembled output after an unsuccessful deployment."""
        current = sources.assemble(sources.SOURCE)
        if current == plan["original"]:
            return
        rollback = Path(tempfile.mkdtemp(
            prefix="silent-sync-rollback-", dir=self.history.directory))
        sources.replace_sources(sources.SOURCE, plan["original"], rollback, expected=current)
        for filename, data in plan.get("original_fighters", {}).items():
            path = Path(filename)
            if path.exists() and path.is_file():
                sources.write_generated(data, path)
        for path in sources.SOURCE.parent.joinpath('fighters').glob('*.txt'):
            if str(path) not in plan.get("original_fighters", {}):
                path.unlink()
        sources.write_generated(plan["original"])

    @contextmanager
    def stage_silent_sequences(self, plan):
        """Temporarily expose a reviewed Silent snapshot to build/upload preflight."""
        current = sources.assemble(sources.SOURCE)
        if current != plan["original"]:
            raise RuntimeError("Repository sequence sources changed after Silent preview; nothing overwritten")
        if not plan.get("data"):
            plan["data"] = current
        generated = sources.GENERATED
        previous_generated = generated.read_bytes() if generated.exists() else None
        scratch = Path(tempfile.mkdtemp(prefix="silent-sequence-stage-", dir=self.history.directory))
        sources.replace_sources(sources.SOURCE, plan["data"], scratch / "before")
        try:
            sources.write_generated(plan["data"])
            yield
        finally:
            sources.replace_sources(sources.SOURCE, plan["original"], scratch / "restore")
            for filename, data in plan.get("original_fighters", {}).items():
                path = Path(filename)
                if path.exists():
                    sources.write_generated(data, path)
            for path in sources.SOURCE.parent.joinpath('fighters').glob('*.txt'):
                if str(path) not in plan.get("original_fighters", {}):
                    path.unlink()
            if previous_generated is None:
                generated.unlink(missing_ok=True)
            else:
                sources.write_generated(previous_generated)

    def apply_silent_sequences(self, plan):
        """Apply a reviewed Silent snapshot to split sources and assembled deployment file."""
        current = sources.assemble(sources.SOURCE)
        if current != plan["original"]:
            raise RuntimeError("Repository sequence sources changed after Silent preview; nothing overwritten")
        if current == plan["data"]:
            return {"unchanged": True, "source": plan["origin"], "destination": str(sources.SOURCE)}
        backup_root = self.history.directory / "source-breakdowns"
        backup_root.mkdir(parents=True, exist_ok=True)
        backup = Path(tempfile.mkdtemp(
            prefix=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-"), dir=backup_root))
        sources.replace_sources(sources.SOURCE, plan["data"], backup, expected=current)
        try:
            sources.write_generated(plan["data"])
        except BaseException:
            rollback = Path(tempfile.mkdtemp(prefix="silent-sync-rollback-", dir=backup_root))
            sources.replace_sources(sources.SOURCE, current, rollback)
            raise
        print(f"Synchronized Silent active sequences into {plan['fighters']} fighter sources; backup: {backup}")
        return {"source": plan["origin"], "destination": str(sources.SOURCE), "backup": str(backup),
                "fighters": plan["fighters"], "sections": plan["sections"], "bytes": plan["bytes"]}

    def run(self, action, argv, details=None):
        details = {**(details or {}), "command": argv}
        print("\nCommand:", shlex.join(argv))
        return self.history.perform(action, details, lambda: self.execute(argv))

    def execute(self, argv):
        return command(argv, interactive=False) if self.noninteractive else command(argv)

    def build(self):
        selected = choose("Build which variants?", ["Both", "Silent", "Diagnostic"])
        if selected is None:
            return False
        variant = ("all", "silent", "diagnostic")[selected]
        cache = ask("Cargo home (Enter = shown value; - = normal cache)", self.preferences.get("cargo_home", "/private/tmp/kirby-assist-build-cargo" if Path("/private/tmp/kirby-assist-build-cargo").is_dir() else ""))
        if cache == "-":
            cache = ""
        self.preferences["cargo_home"] = cache
        self.save_preferences()
        argv = helper("build_variants.py", "--variant", variant)
        if cache:
            argv += ["--cargo-home", str(Path(cache).expanduser())]
        print("\n", shlex.join(argv))
        if not confirm("Build only; this will NOT upload.", "BUILD"):
            return False
        return self.build_release(variant, cache)

    def build_release(self, variant, cache):
        """Prompt-free build shared by GUI and CLI; archive only successful outputs."""
        if variant not in ("all", "silent", "diagnostic"):
            raise ValueError("Invalid build variant selection")
        argv = helper("build_variants.py", "--variant", variant)
        if cache:
            argv += ["--cargo-home", str(Path(cache).expanduser())]
        built_version = version()
        variants = ["silent", "diagnostic"] if variant == "all" else [variant]
        details = {"command": argv, "version": built_version, "variants": variants}
        def operation():
            result = self.execute(argv)
            artifacts = []
            for name in variants:
                folder = upload_switch.selected_source(built_version, name)
                for path in sorted(folder.iterdir()):
                    if path.is_file() and not path.is_symlink():
                        artifacts.append({"variant": name, "source": str(path), **self.history.archive(path.read_bytes())})
            return {**result, "artifacts": artifacts}
        self.history.perform("build", details, operation)
        self.preferences["last_built_version"] = built_version
        self.save_preferences()
        return True

    def upload(self, *, verify=False):
        release = upload_switch.release_version(ask("Built version", self.preferences.get("last_built_version", version())))
        selected = choose("Target", ["Both saved Switches", "Silent only", "Diagnostic only"])
        if selected is None:
            return
        routes = self.targets()
        targets = list(routes) if selected == 0 else [(host, name) for host, name in routes if name == ("silent" if selected == 1 else "diagnostic")]
        if not verify and not confirm("Silent active sequences.txt becomes the repository source of truth. Close Smash and open FTP on Silent and the selected Switches; selected active files will be replaced after verified backups.", "UPLOAD"):
            return False
        return self.upload_release(release, targets, verify=verify)

    def upload_release(self, release, targets, *, verify=False, sequences_synced=False, sequence_plan=None):
        """Fixed route uploads, per-target history and independent failure handling."""
        upload_switch.release_version(release)
        if not targets or any(target not in self.targets() for target in targets):
            raise ValueError("Only the saved Switch/variant routes may be used")
        if not verify and not sequences_synced:
            sequence_plan = self.prepare_silent_sequences()
            with self.stage_silent_sequences(sequence_plan):
                for host, name in targets:
                    preview = helper("upload_switch.py", host, "-v", release,
                                     "-s" if name == "silent" else "-d", "--port", self.port(name),
                                     "--replace-sequences", "--allow-diagnostic-on-silent-ip", "--dry-run")
                    self.run("upload-preview", preview, {"version": release, "target": name, "host": host})
            self.history.perform("silent-sequence-sync", {"source": sequence_plan["origin"], "destination": str(sources.SOURCE)}, lambda: self.apply_silent_sequences(sequence_plan))
            sequences_synced = True
        failures = []
        uploaded = []
        for host, name in targets:
            argv = helper("upload_switch.py", host, "-v", release, "-s" if name == "silent" else "-d", "--port", self.port(name), "--replace-sequences", "--allow-diagnostic-on-silent-ip")
            if verify:
                argv.append("--verify-only")
            else:
                argv.append("--smash-closed")
            details = {"version": release, "target": name, "host": host, "command": argv}
            def operation(argv=argv, details=details, name=name):
                # Keep local payload snapshots; actual uploaded hashes are read
                # separately from the original helper's output, never guessed.
                details["artifacts"] = [{"remote": "/" + str(path), **self.history.archive(data)}
                    for path, data in upload_switch.upload_payloads(upload_switch.selected_source(release, name), replace_sequences=True)]
                return self.execute(argv)
            try:
                self.history.perform("verify" if verify else "upload", details, operation)
                if not verify:
                    uploaded.append(name)
                    if name == "silent" and sequence_plan:
                        sequence_plan["silent_uploaded"] = True
            except (OSError, RuntimeError) as error:
                failures.append(name)
                print(f"{name}: {error}")
                if not verify and name == "silent" and sequence_plan:
                    try:
                        self.rollback_silent_sequences(sequence_plan)
                        sequences_synced = False
                    except (OSError, RuntimeError) as rollback_error:
                        print(f"Repository rollback failed after Silent upload failure: {rollback_error}")
        if failures and not verify and sequences_synced and sequence_plan and not sequence_plan.get("silent_uploaded"):
            try:
                self.rollback_silent_sequences(sequence_plan)
            except (OSError, RuntimeError) as error:
                print(f"Repository rollback failed after upload failure: {error}")
        print("Finished." if not failures else "Failed/unconfirmed targets: " + ", ".join(failures))
        return not failures

    def endpoint(self, label):
        item = choose(label, ["Local computer file", "Silent active/staged sequences", "Diagnostic active/staged sequences"])
        if item is None:
            return None
        if item == 0:
            # Transfers always start from the deployed assembled bundle; a
            # different path can still be entered explicitly at the prompt.
            return transfer.Endpoint("local", str(self.local_path("Local file", DEFAULT_FILE, force_default=True)))
        name = "silent" if item == 1 else "diagnostic"
        file = choose("Which sequence file?", ["ACTIVE sequences.txt (in-game saves)", "STAGED sequences.upload.txt (not active)"])
        if file is None:
            return None
        return transfer.Endpoint(name, transfer.ACTIVE if file == 0 else transfer.STAGED,
                                 self.port(name), self.host(name))

    def sequences(self):
        source = self.endpoint("Transfer FROM")
        if source is None:
            return
        destination = self.endpoint("Transfer TO")
        if destination is None:
            return
        scope = choose("Transfer scope", ["One fighter: replace ALL their recipes/settings", "Whole file: replace every fighter/settings"])
        if scope is None:
            return
        fighter = ask("Fighter ID (e.g. yoshi, kirby, captain_falcon)", self.preferences.get("fighter", "")) if scope == 0 else None
        if scope == 0 and not fighter:
            print("No fighter selected; cancelled.")
            return
        if fighter:
            self.preferences["fighter"] = fighter
            self.save_preferences()
        plans = []
        def preview():
            plan = transfer.prepare(source, destination, fighter)
            plans.append(plan)
            print(f"FROM: {source.label()}\nTO:   {destination.label()}\n{plan.description}\nResult: {len(plan.updated)} bytes; SHA-256 {transfer.digest(plan.updated)}")
            print("Validation uses the current source parser; installed plugin compatibility is not inferred.")
            return plan.details()
        self.history.perform("sequence-preview", {"source": source.label(), "destination": destination.label(), "fighter": fighter}, preview)
        plan = plans[0]
        if not confirm("Replace the previewed destination with verified backups?\nFor any Switch involved, close Smash and leave FTP open first.", "TRANSFER"):
            return
        self.history.perform("sequence-transfer", plan.details(),
            lambda: transfer.commit(plan, self.history.directory / "sequence-backups"))

    def breakdown_plan(self, endpoint, source=None):
        """Read a combined file and describe the fighter files it would produce."""
        source = Path(source or sources.SOURCE)
        if not sources.is_split_source(source):
            raise RuntimeError(
                "Choose the repository deployment/ultimate/kirby_assist/sequences.txt; "
                "its fighters/ directory receives the individual files")
        data = transfer.read(endpoint)
        if data is None:
            raise RuntimeError(f"No sequence file to break down at {endpoint.label()}")
        transfer.validate_sequences(data)
        if source.resolve() == sources.SOURCE.resolve():
            data = sources.restore_comments(sources.assemble(source), data)
            transfer.validate_sequences(data)
        plan = sources.plan_breakdown(source, data)
        plan["origin"] = endpoint.label()
        return plan

    def breakdown_commit(self, plan):
        """Replace the split sources from a reviewed plan, with backups and readback."""
        source = Path(plan["source"])
        backup_root = self.history.directory / "source-breakdowns"
        backup_root.mkdir(parents=True, exist_ok=True)
        backup = Path(tempfile.mkdtemp(
            prefix=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-"), dir=backup_root))
        sources.replace_sources(source, plan["data"], backup)
        written = len(plan["created"]) + len(plan["updated"])
        print(f"Wrote {written} source files; {len(plan['removed'])} removed. Backup: {backup}")
        return {"source": str(source), "origin": plan.get("origin"), "backup": str(backup),
                "created": [e["fighter"] for e in plan["created"]],
                "updated": [e["fighter"] for e in plan["updated"]],
                "removed": [e["fighter"] for e in plan["removed"]],
                "sections": plan["sections"], "bytes": plan["bytes"]}

    def breakdown(self):
        """Terminal flow: pick a combined file, review the split, then confirm."""
        endpoint = self.endpoint("Break down WHICH combined sequence file")
        if endpoint is None:
            return
        destination = Path(ask("Repository universal source", str(sources.SOURCE))).expanduser().resolve()
        plans = []
        def preview():
            plan = self.breakdown_plan(endpoint, destination)
            plans.append(plan)
            print("\n" + sources.describe_breakdown(plan))
            return {k: v for k, v in plan.items() if k != "data"}
        self.history.perform("breakdown-preview",
                             {"source": endpoint.label(), "destination": str(destination)}, preview)
        plan = plans[0]
        if not confirm("Replace the repository fighter sources with this breakdown?\n"
                       "Existing files are backed up first.", "BREAKDOWN"):
            return
        self.history.perform("breakdown", {"source": endpoint.label(), "destination": str(destination)},
                             lambda: self.breakdown_commit(plan))

    def edit_fighter(self):
        """Edit one fighter's sections in $EDITOR against a single read snapshot."""
        endpoint = self.endpoint("Edit which sequence file")
        if endpoint is None:
            return
        session = fighter_editor.Session(self.history.directory / "fighter-edits")
        metadata = self.history.perform("fighter-load", {"source": endpoint.label()},
                                        lambda: session.load(endpoint))
        print(f"Loaded {metadata['bytes']} bytes; {len(metadata['fighters'])} fighters: "
              + ", ".join(metadata["fighters"]))
        fighter = ask("Fighter ID to edit", self.preferences.get("fighter", ""))
        if not fighter:
            print("No fighter selected; cancelled.")
            return
        current = session.get(fighter, session.load_id)
        self.preferences["fighter"] = current["fighter"]
        self.save_preferences()
        if not current["sections"]:
            print(f"{current['fighter']} has no sections in this file; new sections may be added.")
        scratch = session.directory / f"{current['fighter']}.edit.txt"
        scratch.parent.mkdir(parents=True, exist_ok=True)
        scratch.write_text(current["text"])
        editor = os.environ.get("VISUAL") or os.environ.get("EDITOR")
        if editor:
            command([*shlex.split(editor), str(scratch)])
        else:
            print(f"\nSet $EDITOR to open this automatically. Edit then save:\n  {scratch}")
            input("Press Enter when the file is saved: ")
        preview = self.history.perform(
            "fighter-preview", {"source": endpoint.label(), "fighter": current["fighter"]},
            lambda: session.prepare_text(current["fighter"], scratch.read_text(),
                                         session.load_id, "cli"))
        print(f"\n{preview['description']}")
        print(preview["diff"] or "No textual change in this fighter.")
        if preview["diff_truncated"]:
            print("(diff truncated)")
        if not confirm(f"Write only {current['fighter']} to {endpoint.label()}?"
                       + ("\nClose Smash on that Switch and leave FTP open first." if session.requires_ready() else ""),
                       "SAVE"):
            session.discard()
            return
        self.history.perform("fighter-save",
            {"destination": endpoint.label(), "fighter": current["fighter"], "sha256": preview["sha256"]},
            lambda: session.commit(preview["plan_id"], confirmed=True, ready=True))

    def show_history(self):
        rows = self.history.rows()
        if not rows:
            print("No workbench operations recorded yet.")
            return
        shipped = {}
        for row in rows:
            details = row.get("details", {})
            if row.get("action") == "upload" and row.get("status") == "success" and row.get("receipt", {}).get("verified_deployment"):
                shipped[details.get("target", "unknown")] = row
        for target, row in shipped.items():
            print(f"Last VERIFIED {target} upload: {row['details'].get('version')} at {row['time']}")
        print("\nRecent operations (started with no completion = unknown/interrupted):")
        for row in rows[-20:]:
            details = row.get("details", {})
            print(f"{row['id']} | {row['action']} | {row['status']} | {details.get('version','')} {details.get('target','')}")
            if details.get("source") or details.get("destination"):
                print(f"  {details.get('source','')} -> {details.get('destination','')} | {details.get('fighter') or 'whole file'}")
            print("  log:", row["log"])
        print("Full history:", self.history.path)
        print("Artifact snapshots:", self.history.directory / "artifacts")

    def advanced(self):
        item = choose("Original helpers (all original options remain available)", list(HELPERS))
        if item is None:
            return
        name = HELPERS[item]
        args = shlex.split(ask("Arguments (use --help to see original options)", "--help"))
        readonly = any(flag in args for flag in ("--help", "-h", "--dry-run", "--verify-only", "--check-only", "--check")) or name == "sequence_validation.py"
        writes = not readonly and (name.startswith("upload") or name == "sync_sequences.py" or any(flag in args for flag in ("--upload", "-upload")))
        imports = not readonly and name == "sequence_sources.py" and any(a.startswith("--import-file") for a in args)
        argv = helper(name, *args)
        print(shlex.join(argv))
        if writes:
            prompt, token = "Confirm Smash CLOSED before any Switch write. Original helper confirmations remain enabled.", "WRITE"
        elif imports:
            prompt, token = "This replaces the repository fighter sources from the given file; backups are kept.", "IMPORT"
        else:
            prompt, token = "Run this original helper?", "RUN"
        if not confirm(prompt, token):
            return
        self.run("advanced", argv)

    def settings(self):
        self.preferences["sequences"] = str(self.local_path("Default local sequences file"))
        self.preferences["cargo_home"] = ask("Default Cargo home (Enter = shown; - = normal)", self.preferences.get("cargo_home", ""))
        if self.preferences["cargo_home"] == "-":
            self.preferences["cargo_home"] = ""
        for name in ("silent", "diagnostic"):
            configured_host = ask(f"{name.title()} Switch IPv4", self.host(name))
            try:
                if ipaddress.ip_address(configured_host).version != 4:
                    raise ValueError
            except ValueError as error:
                raise ValueError("Switch addresses must be valid IPv4 addresses") from error
            self.preferences[name + "_host"] = configured_host
            port = int(ask(f"{name} FTP port", self.port(name)))
            if not 1 <= port <= 65535:
                raise ValueError("Port must be 1..65535")
            self.preferences[name + "_port"] = port
        if self.host("silent") == self.host("diagnostic"):
            raise ValueError("Silent and Diagnostic Switch addresses must be different")
        self.save_preferences()

    def build_then_upload(self):
        if self.build():
            self.upload()

    def validate(self):
        self.run("validate", helper("sequence_validation.py", self.local_path("File to validate")))

    def menu(self):
        """Named menu so adding an operation cannot renumber the others."""
        return (
            ("Ship: build, then review/upload", self.build_then_upload),
            ("Build only", self.build),
            ("Upload a built release", self.upload),
            ("Verify installed release (read-only)", lambda: self.upload(verify=True)),
            ("Edit one fighter's sequences", self.edit_fighter),
            ("Break down a combined file into fighter sources", self.breakdown),
            ("Transfer / replace sequences (computer / Switches)", self.sequences),
            ("Validate a sequences file", self.validate),
            ("History / shipped versions / file logs", self.show_history),
            ("Preferences", self.settings),
            ("Original helpers / advanced options", self.advanced),
        )

    def loop(self):
        while True:
            try:
                print(f"\nKIRBY ASSIST WORKBENCH | source {version()}")
                entries = self.menu()
                item = choose("Choose an operation", [label for label, _ in entries], back_label="Exit")
                if item is None:
                    return
                entries[item][1]()
            except EOFError:
                return
            except KeyboardInterrupt:
                print("\nCancelled. Check the operation log if a transfer/upload had started.")
            except (OSError, RuntimeError, ValueError, ftplib.Error, argparse.ArgumentTypeError) as error:
                print(f"\nOperation stopped: {error}\nThe workbench remains open; inspect history before retrying writes.")


@contextmanager
def session_lock(directory):
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / "session.lock").open("a") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise RuntimeError("Another workbench is already running with this history directory")
        yield


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--state-dir", type=Path, default=STATE, help="local history/preferences/backups directory")
    parser.add_argument("--history", action="store_true", help="show history and exit; no build/FTP")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--cli", action="store_true", help="original numbered terminal prompt")
    mode.add_argument("--gui", action="store_true", help="local browser GUI (default)")
    args = parser.parse_args()
    try:
        state_dir = args.state_dir.expanduser().resolve()
        if state_dir.is_relative_to(ROOT) and ".mod-workbench" not in state_dir.relative_to(ROOT).parts:
            raise RuntimeError("Keep history in .mod-workbench or outside the project so build source archives cannot include it")
        with session_lock(state_dir):
            app = Workbench(History(state_dir))
            if args.history:
                app.show_history()
            elif args.cli:
                app.loop()
            else:
                from workbench_gui import launch
                launch(app)
    except (OSError, RuntimeError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
