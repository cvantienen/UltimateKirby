#!/usr/bin/env python3
"""Build/package logging variants, optionally uploading both after packaging.

This supports Python 3.9+; Python versions before 3.11 use the tomli backport
from requirements-docs.txt. Run it from any working directory; ROOT comes from
this script's location, not the shell's current directory. Cargo/Skyline must
already be installed. An optional --cargo-home can select an isolated cache if
the normal cache has permission problems.
Pass -upload to run the fixed multi-Switch uploader after both builds succeed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import sys
from pathlib import Path
import subprocess
from typing import BinaryIO
import zipfile
from sequence_sources import assemble, write_generated
from toml_compat import tomllib

ROOT = Path(__file__).resolve().parents[1]


def read_release_version() -> str:
    """Read the literal version without parsing the full Cargo manifest."""
    manifest = (ROOT / "Cargo.toml").read_text()
    package = re.search(r"(?ms)^\[package\][ \t]*\n(.*?)(?=^\[|\Z)", manifest)
    version = re.search(r'^version\s*=\s*"([^"]+)"', package[1], re.MULTILINE) if package else None
    if not version or not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version[1]):
        raise RuntimeError("Cargo.toml [package].version must be a literal semantic version")
    return version[1]


RELEASE_VERSION = read_release_version()
RELEASE_SUMMARY_NAME = f"RELEASE_SUMMARY-{RELEASE_VERSION}.txt"
PLUGIN = Path("atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro")
CONFIG = Path("ultimate/kirby_assist/config.txt")
SEQUENCES = Path("ultimate/kirby_assist/sequences.txt")
SOURCE_ARCHIVE_EXCLUDES = {
    ".mod-workbench",
    ".sequence-sync-backups",
    ".git",
    ".mypy_cache",
    "releases",
    "target",
    "__pycache__",
    ".venv-docs",
    "site",
}
SOURCE_ARCHIVE_EXCLUDED_FILES = {".DS_Store"}
# These are our explicit logger/message markers, not a test for all possible
# metadata, crash reporting, or telemetry in the console/runtime.
LOG_MARKERS = (
    b"sd:/ultimate/kirby_assist/status.log",
    b"sd:/ultimate/kirby_assist/performance.log",
    b"[kirby_assist]",
    b"tech observation:",
    b"ledge observation:",
    b"ledge mapper:",
    b"tech ZR applied:",
    b"NOT INSTALLED:",
)


def read_release_summary() -> bytes:
    """Use the Cargo package description verbatim, including multiline strings."""
    with (ROOT / "Cargo.toml").open("rb") as manifest:
        description = tomllib.load(manifest).get("package", {}).get("description")
    if not isinstance(description, str) or not description.strip():
        raise RuntimeError("Cargo.toml [package].description must be a nonempty string")
    return description.encode("utf-8")


def variant_release_summary(variant: str) -> bytes:
    """Label the packaged summary while preserving the Cargo description body."""
    if variant not in ("silent", "diagnostic"):
        raise ValueError("Release summary requires a single build variant")
    heading = f"Kirby Assist {RELEASE_VERSION}\nBuild: {variant.upper()}\n\n"
    return heading.encode("utf-8") + read_release_summary()


def documentation_files() -> dict[str, bytes]:
    """Package the maintained guides and assets without a hard-coded page list."""
    docs = ROOT / "docs"
    if not (docs / "index.md").is_file():
        raise RuntimeError(f"Documentation homepage missing: {docs / 'index.md'}")
    files = {
        path.relative_to(ROOT).as_posix(): path.read_bytes()
        for path in sorted(docs.rglob("*"))
        if path.is_file() and not path.is_symlink()
        and not any(part.startswith(".") for part in path.relative_to(docs).parts)
    }
    for name in ("mkdocs.yml", "requirements-docs.txt"):
        files[name] = (ROOT / name).read_bytes()
    return files


def variant_features(diagnostic: bool, silent_probe: bool = False) -> list[str]:
    """Normal releases stay unchanged; the one-match probe is explicitly opt-in."""
    if diagnostic and silent_probe:
        raise ValueError("silent-probe cannot be combined with diagnostic logging")
    return ["imgui-menu"] + (["diagnostic-logging"] if diagnostic else []) + (["silent-probe"] if silent_probe else [])


def build_variant(name: str, env: dict[str, str], *, silent_probe: bool = False) -> Path:
    """Compile once, then snapshot the result before the next build overwrites it."""
    # Assemble current editable sources before compiling or packaging either variant.
    sequences = assemble(ROOT / "deployment" / SEQUENCES)
    from sequence_validation import validate_sequences
    validate_sequences(sequences)
    write_generated(sequences, ROOT / "deployment" / SEQUENCES.parent / "assembled/sequences.txt")
    diagnostic = name == "diagnostic"
    summary = variant_release_summary(name)
    if silent_probe:
        summary += b"\nTEMPORARY: one-match SD aggregate probe enabled (silent-probe).\n"
    command = ["cargo", "skyline", "build", "--release", "--no-default-features"]
    command += ["--features", ",".join(variant_features(diagnostic, silent_probe))]
    # Stage 1: Cargo/Skyline compiles Rust and produces the NRO. check=True
    # stops this script on any build failure; it does not package an older NRO
    # as if the current build succeeded. Cargo itself does not create our ZIP.
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    # Keep the exact output bytes in memory before building the other variant,
    # since both feature selections use this same target filename.
    binary = (ROOT / "target/aarch64-skyline-switch/release/libkirby_assist.nro").read_bytes()
    if binary[0x10:0x14] != b"NRO0":
        raise RuntimeError("Build did not produce the expected NRO format")
    markers = {marker.decode(): marker in binary for marker in LOG_MARKERS}
    if any(present != diagnostic for present in markers.values()):
        raise RuntimeError(f"Unexpected {name} logging marker results: {markers}")

    if (b"ONE_MATCH_PROBE_V1" in binary) != silent_probe:
        raise RuntimeError("Unexpected one-match probe marker; refusing to package")

    # Both variants share all gameplay settings. Only tech_debug differs, and a
    # silent binary ignores it regardless because its logging code is absent.
    config = (ROOT / "deployment" / CONFIG).read_text()
    config = config.replace("tech_debug = true", f"tech_debug = {str(diagnostic).lower()}")
    heading = (
        "# SILENT: plugin file/console logging is compiled out; tech_debug cannot enable it.\n"
        if not diagnostic else
        "# DIAGNOSTIC: file/console logging is compiled in; tech_debug enables tech details.\n"
    )
    if silent_probe:
        heading = "# TEMPORARY SILENT PROBE: one-match aggregate SD log; no console/network output.\n"
    config = (heading + config).encode()
    install = f"""Kirby Assist {RELEASE_VERSION} - {name.upper()} BUILD

Close Smash fully. Copy atmosphere/ and ultimate/ to the SD-card root, replacing
this mod's plugin/config/sequence file. Install ONE variant at a time: both use the same .nro
name and destination. Do not rename them and install both together.

Use the Player 1 controller (player_index = 0), a registered human fighter, and ZR mapped to Shield.
Compiled fighter catalogs cover the non-DLC roster, including echoes, Pokemon forms and Miis, plus separate Pyra/Mythra identities with editable starter menus.
Existing profiles are retained; new choices use fighter-specific windows and native movement forecasts.
See docs/NON_DLC_ROSTER.md for defaults, projectile/resource exceptions and validation limits.
Gameplay policy is shared between the silent and diagnostic variants.
Fighter menu choices retain compiled fallbacks. sequences.txt may override an
existing choice's actions at startup; config.txt supplies shared settings.
Hold L for ledge autopilot; tap Y for neutral-B reverse. No combo or mode selection.
Automatic mash starts OFF in the supplied config; enable it from the Auto-mash menu row.
Auto-tech and Spot punish are available and start ON in both variants; each has an ON/OFF menu row.
Hold R3 alone for fast crouch/walk-cancel turbo teabag; press X for paired X+ZL short hop (both mapped to Jump).
Hold L and click R3 to show/hide the bottom-left menu, which shows mash ON/OFF.
While open, D-pad Up/Down
selects a row and Left/Right cycles its saved options. While closed, sides
slide, Down starts ledge trump, and Up keeps normal input.
Both builds include the imgui-smash battle menu.
Reuse ONE matching libimgui_smash.nro host (the same library as Online Deluxe).
The host is not bundled or installed by this packager. See docs/LEDGE_MENU.md.
The menu shows saved ledge choices only, with no live statistics or control hints.
Requires A=Attack, X=Jump, R=Grab, ZR=Shield, Tap Jump OFF.
Native readiness and vulnerability gate automatic responses. You space normal
and jump get-up responses; roll approaches automatically. See the controls docs.
Collision coverage and frame-perfect timing require device testing.
Upload separately with tools/upload_switch.py IP -v {RELEASE_VERSION} -s (silent)
or -d (diagnostic). Before replacing the installed config, sequences, release
summary, optional source ZIP and plugin NRO, the uploader saves previous installed
bytes to ultimate/kirby_assist/bak/ and verifies every backup.
It then overwrites and verifies the new files at their original destinations.
The flat bak directory keeps one previous installed set, overwritten each upload.
Other files remain untouched. Building never uploads.


Silent: no explicit plugin file/console logger, startup log creation, or detailed
message collection. It still reads config. tech_debug cannot enable logging.
It does not delete old files or change Skyline/OS/game crash reports or telemetry.
A missing/invalid config or unsupported signature produces no silent diagnostic;
use the diagnostic variant to troubleshoot instead.

Diagnostic: startup/basic logs plus detailed tech logs with tech_debug = true.
Output goes to console; ordinary evidence goes to SD:/ultimate/kirby_assist/status.log,
while enabled Perf timing/health reports go to SD:/ultimate/kirby_assist/performance.log. The first line
identifies 'Kirby Assist diagnostic build {RELEASE_VERSION}'. tech_debug = false reduces details,
but does not disable startup/basic logging in this variant.

Neither variant hides the plugin or guarantees indistinguishability/ban protection.
Read docs/BUILD_VARIANTS.md for scope, build commands, and hardware verification steps.
"""
    if silent_probe:
        install = install.replace(
            "Silent: no explicit plugin file/console logger, startup log creation, or detailed\nmessage collection.",
            "TEMPORARY SILENT PROBE: creates silent_probe_once.log for one match only;\n10-second summaries, 32KiB/10-minute cap. No console/network output. Existing file\nprevents another run. Rebuild without --silent-probe after the test."
        )
    # Stage 2: describe what this script actually checked. SHA-256 identifies
    # bytes for later comparison; it says nothing about in-game input timing.
    validation = {
        "build_id": RELEASE_VERSION,
        "variant": name,
        "release_optimized": True,
        "cargo_features": variant_features(diagnostic, silent_probe),
        "requires_imgui_smash_host": True,
        "explicit_plugin_logging": diagnostic or silent_probe,
        "temporary_silent_probe": silent_probe,
        "probe_marker_present": b"ONE_MATCH_PROBE_V1" in binary,
        "logging_marker_checks": markers,
        "plugin_sha256": hashlib.sha256(binary).hexdigest(),
        "config_sha256": hashlib.sha256(config).hexdigest(),
        "sequences_sha256": hashlib.sha256(sequences).hexdigest(),
        "hardware_logging_check_performed": False,
        "system_runtime_telemetry_audited": False,
    }
    files = {
        str(PLUGIN): binary,
        str(CONFIG): config,
        str(SEQUENCES): sequences,
        "INSTALL.txt": install.encode(),
        RELEASE_SUMMARY_NAME: summary,
        "VALIDATION.json": (json.dumps(validation, indent=2) + "\n").encode(),
    }
    files.update(documentation_files())

    # Separate folders prevent a later feature build from overwriting the earlier
    # variant's installable binary. The ZIP contains the same reviewed bytes.
    stage = ROOT / "releases" / name
    for relative, data in files.items():
        target = stage / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    # Remove only legacy generated guide copies after their docs/ replacements
    # have been written. Historical release directories/archives are untouched.
    for legacy_name in ("README.md", "README_BEGINNER.md", "BUILD_VARIANTS.md", "TURBO_CONTROLS.md", "LEDGE_TRAP.md", "LEDGE_TRUMP.md", "ANALOG_MASH.md", "INTEGRATION.md"):
        legacy = stage / legacy_name
        if legacy.is_file():
            legacy.unlink()
    output = ROOT / "releases" / f"kirby_assist-{RELEASE_VERSION}-{name}.zip"
    # Stage 3: Python compresses the already compiled files here. The ZIP is
    # an SD-card folder bundle, not another program or another compilation.
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
        for relative, data in files.items():
            archive.writestr(relative, data)
    # Reopen the finished archive and verify CRCs and every stored file's bytes.
    # This validates packaging, not Switch installation or gameplay behavior.
    with zipfile.ZipFile(output) as archive:
        if archive.testzip() is not None:
            raise RuntimeError(f"ZIP CRC check failed: {output}")
        for relative, data in files.items():
            if archive.read(relative) != data:
                raise RuntimeError(f"Packaged bytes differ: {relative}")
    print(f"Verified {output.name}: {len(binary):,}-byte NRO; explicit logging={diagnostic or silent_probe}; temporary probe={silent_probe}", flush=True)
    return output

def copy_release_output(source: Path, destination: Path) -> None:
    """Copy one variant's plugin, startup configs and summary to its flat folder."""
    destination.mkdir(parents=True, exist_ok=True)
    # Retire only this known former generated payload; preserve unrelated files.
    obsolete = destination / "fighter_configs.json"
    if obsolete.is_file() or obsolete.is_symlink():
        obsolete.unlink()
    for file in (PLUGIN, CONFIG, SEQUENCES, Path(RELEASE_SUMMARY_NAME)):
        target = destination / file.name
        target.write_bytes((source / file).read_bytes())
    print(f"Copied {source} to {destination}", flush=True)


def create_source_backup(destination: Path) -> Path:
    """Archive the release's source tree without generated build artifacts."""
    destination.mkdir(parents=True, exist_ok=True)
    output = destination / f"kirby_assist-{RELEASE_VERSION}-source.zip"
    source_files = sorted(
        path
        for path in ROOT.rglob("*")
        if path.is_file()
        and not path.is_symlink()
        and path.name not in SOURCE_ARCHIVE_EXCLUDED_FILES
        and (
            path.relative_to(ROOT) == Path("releases") / RELEASE_SUMMARY_NAME
            or not any(part in SOURCE_ARCHIVE_EXCLUDES for part in path.relative_to(ROOT).parts)
        )
    )
    if not source_files:
        raise RuntimeError("No source files found to back up")

    # Keep one project directory at the archive root so extraction does not
    # scatter the source files into the user's current directory.
    archive_root = ROOT.name
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
        for source in source_files:
            relative = source.relative_to(ROOT)
            archive.write(source, Path(archive_root) / relative)

    with zipfile.ZipFile(output) as archive:
        if archive.testzip() is not None:
            raise RuntimeError(f"Source ZIP CRC check failed: {output}")
        expected = {
            (Path(archive_root) / source.relative_to(ROOT)).as_posix()
            for source in source_files
        }
        if set(archive.namelist()) != expected:
            raise RuntimeError(f"Source ZIP contents differ: {output}")
    print(f"Verified source backup {output}: {len(source_files)} files", flush=True)
    return output

def parse_options(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--variant", choices=("all", "silent", "diagnostic"), default="all",
                        help="Build variant (default: all — silent and diagnostic)")
    parser.add_argument("--cargo-home", type=Path, help="Optional isolated Cargo cache")
    parser.add_argument("-upload", "--upload", action="store_true",
                        help="After a successful all-variant build, upload using the saved Switch routes")
    parser.add_argument("--silent-probe", action="store_true",
                        help="Temporary one-match aggregate SD logging in silent only (no upload by default)")
    options = parser.parse_args(argv)
    if options.silent_probe and options.variant == "diagnostic":
        parser.error("--silent-probe requires -v silent or -v all")
    if options.upload and options.variant != "all":
        parser.error("-upload requires -v all so neither Switch receives a stale variant")
    return options


def upload_built_release() -> None:
    """Run the fixed uploader only after main has packaged both variants."""
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools" / "upload_all_switches.py"),
            "-v",
            RELEASE_VERSION,
        ],
        cwd=ROOT,
        check=True,
    )


def sha256_digest(file_obj: BinaryIO) -> bytes:
    """Hash a file in bounded chunks on Python versions without file_digest."""
    digest = hashlib.sha256()
    for chunk in iter(lambda: file_obj.read(1024 * 1024), b""):
        digest.update(chunk)
    return digest.digest()


def main() -> None:
    options = parse_options()
    summary = read_release_summary()
    summary_path = ROOT / "releases" / RELEASE_SUMMARY_NAME
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_bytes(summary)
    env = os.environ.copy()
    if options.cargo_home:
        env["CARGO_HOME"] = str(options.cargo_home.expanduser().resolve())
    variants = ("silent", "diagnostic") if options.variant == "all" else (options.variant,)
    latest_build = ROOT.parent / f"Latestbuild-{RELEASE_VERSION}"
    for variant in variants:  # Sequential: Cargo output path is shared.
        if options.silent_probe and variant == "silent":
            build_variant(variant, env, silent_probe=True)
        else:
            build_variant(variant, env)
    # Publish each selected variant separately. A single-variant build never
    # refreshes the other variant or copies its NRO from a shared target path.
    for variant in variants:
        copy_release_output(ROOT / "releases" / variant, latest_build / variant)
    # Source is identical for both feature selections. Archive it once, then
    # copy those exact verified bytes into the other selected variant folder.
    source_archive = create_source_backup(latest_build / variants[0])
    with source_archive.open("rb") as archive_file:
        source_hash = sha256_digest(archive_file)
    for variant in variants[1:]:
        target = latest_build / variant / source_archive.name
        shutil.copy2(source_archive, target)
        with target.open("rb") as archive_file:
            if sha256_digest(archive_file) != source_hash:
                raise RuntimeError(f"Source backup copy differs: {target}")
        print(f"Copied verified source backup to {target}", flush=True)
    if options.upload:
        upload_built_release()
    print(
        "REMINDER: revisit the deferred Training-to-online GPU crash. "
        "Compare fresh boot -> Elite with Training -> online.",
        flush=True,
    )



if __name__ == "__main__":
    main()
