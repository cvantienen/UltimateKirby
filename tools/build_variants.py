#!/usr/bin/env python3
"""Build/package both logging variants without installing anything on a Switch.

This uses only Python 3.11+'s standard library. Run it from any working directory;
ROOT comes from this script's own location, not the shell's current directory.
Cargo/Skyline must already be installed. An optional --cargo-home can select
an isolated cache if the normal cache has permission problems.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def read_release_version() -> str:
    """Read the literal package version without requiring Python 3.11/tomllib."""
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
SOURCE_ARCHIVE_EXCLUDES = {
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


def build_variant(name: str, env: dict[str, str]) -> Path:
    """Compile once, then snapshot the result before the next build overwrites it."""
    diagnostic = name == "diagnostic"
    summary = read_release_summary()
    command = ["cargo", "skyline", "build", "--release", "--no-default-features"]
    if diagnostic:
        command += ["--features", "diagnostic-logging"]
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

    # Both variants share all gameplay settings. Only tech_debug differs, and a
    # silent binary ignores it regardless because its logging code is absent.
    config = (ROOT / "deployment" / CONFIG).read_text()
    config = config.replace("tech_debug = true", f"tech_debug = {str(diagnostic).lower()}")
    heading = (
        "# SILENT: plugin file/console logging is compiled out; tech_debug cannot enable it.\n"
        if not diagnostic else
        "# DIAGNOSTIC: file/console logging is compiled in; tech_debug enables tech details.\n"
    )
    config = (heading + config).encode()
    install = f"""Kirby Assist {RELEASE_VERSION} - {name.upper()} BUILD

Close Smash fully. Copy atmosphere/ and ultimate/ to the SD-card root, replacing
this mod's plugin/config. Install ONE variant at a time: both use the same .nro
name and destination. Do not rename them and install both together.

Use the first Pro Controller (npad_id = 0), human Kirby, and ZR mapped to Shield.
Gameplay policy is shared: left-stick mash, tech, and directional turbo crouch.
Mash cycles Right/Up/Left/Down every distinct mashable input update, with no
automated action buttons. Supplied takeover blocks physical action buttons and
C-stick during mash, then restores them. See docs/ANALOG_MASH.md for device checks.
Hold D-pad Left/Right for turbo crouch with slight movement that direction.
Both held cancels movement; release restores physical stick. Side taunts are
consumed during valid gameplay. Runs only in normal grounded movement for Kirby.
Tune turbo_phase_frames / turbo_walk_strength in the config and restart Smash.
Hold the left-stick click (L3) near an opponent hanging at ledge for reactive ledge assistance.
Tap D-pad Down for a run-off ledge-catch/trump attempt, with no opponent required.
The nearest floor edge is detected independently (up to 100 world units).
Control returns at ledge catch; choose your own follow-up. The feature defaults
on; auto_ledge_trump=false restores Down taunt. This remains Kirby-only.
See docs/LEDGE_TRUMP.md for starting conditions and device-test limits.
With ledge_drop_defense=true (default), L3 also shields witnessed ledge-drop
double-jump/aerial approaches without walking in or grabbing blindly.
Release L3 to choose a manual punish after blocking. See docs/LEDGE_TRAP.md.
Requires A=Attack, X=Jump, R=Grab, ZR=Shield, Tap Jump OFF.
Start near the ledge; coverage depends on spacing and native action timing. See docs/LEDGE_TRAP.md for scope/limitations and docs/TURBO_CONTROLS.md for turbo.
The ledge policy needs hardware testing. 1.0.0 predicts active-window reach and
invincibility expiry, pursues normal get-up/roll/jump travel, and retries attacks.
The supplied stance is 12 units inward: copy the new config with the new plugin.
The supplied responses are downward-angled forward_tilt for normal get-up,
neutral_air for ledge jump, and down_smash after shielding get-up attack.
Down tilt, neutral air, and shield grab remain selectable in config.txt.
1.0.0 fixes idle being mistaken for hanging, recovers missed option transitions,
removes extra tilt retry delay, and predicts shield-grab reach/protection.
With ledge_throw_outward=true, a confirmed capture gets a directional throw
toward the remembered ledge. After recovery Kirby returns to his ledge stance.
1.0.0 acquires anchors in catch/catch-move as well as hang, retains identity
through two missing mapper observations without acting on stale data, tapers
movement near targets, and uses stance hysteresis to reduce oscillation.
Normal get-up uses a configurable minimum/approach/outer spacing band. Roll uses
approach, brake, face, and ready phases so a failed timing gate stays neutral.
Tilt, jump, and grab pursuit separations are independently configurable.
No frame-perfect/unavoidable coverage is claimed. See docs/LEDGE_TRAP.md.


Silent: no explicit plugin file/console logger, startup log creation, or detailed
message collection. It still reads config. tech_debug cannot enable logging.
It does not delete old files or change Skyline/OS/game crash reports or telemetry.
A missing/invalid config or unsupported signature produces no silent diagnostic;
use the diagnostic variant to troubleshoot instead.

Diagnostic: startup/basic logs plus detailed tech logs with tech_debug = true.
Output goes to console and SD:/ultimate/kirby_assist/status.log. The first line
identifies 'Kirby Assist diagnostic build {RELEASE_VERSION}'. tech_debug = false reduces details,
but does not disable startup/basic logging in this variant.

Neither variant hides the plugin or guarantees indistinguishability/ban protection.
Read docs/BUILD_VARIANTS.md for scope, build commands, and hardware verification steps.
"""
    # Stage 2: describe what this script actually checked. SHA-256 identifies
    # bytes for later comparison; it says nothing about in-game input timing.
    validation = {
        "build_id": RELEASE_VERSION,
        "variant": name,
        "release_optimized": True,
        "cargo_features": ["diagnostic-logging"] if diagnostic else [],
        "explicit_plugin_logging": diagnostic,
        "logging_marker_checks": markers,
        "plugin_sha256": hashlib.sha256(binary).hexdigest(),
        "config_sha256": hashlib.sha256(config).hexdigest(),
        "hardware_logging_check_performed": False,
        "system_runtime_telemetry_audited": False,
    }
    files = {
        str(PLUGIN): binary,
        str(CONFIG): config,
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
    output = ROOT / "releases" / f"kirby_assist-{name}.zip"
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
    print(f"Verified {output.name}: {len(binary):,}-byte NRO; explicit logging={diagnostic}", flush=True)
    return output

def copy_release_output(source: Path, destination: Path) -> None:
    """Refresh silent NRO/config without deleting earlier source backups."""
    destination.mkdir(parents=True, exist_ok=True)
    for file in (PLUGIN, CONFIG):
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

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--variant", choices=("all", "silent", "diagnostic"), default="all")
    parser.add_argument("--cargo-home", type=Path, help="Optional isolated Cargo cache")
    # --variant silent/diagnostic selects one package; the default "all" builds
    # both in order. --cargo-home changes dependency cache location only, not
    # runtime configuration or which gameplay features the plugin contains.
    options = parser.parse_args()
    summary = read_release_summary()
    summary_path = ROOT / "releases" / RELEASE_SUMMARY_NAME
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_bytes(summary)
    env = os.environ.copy()  # Do not mutate the caller's shell configuration.
    if options.cargo_home:
        env["CARGO_HOME"] = str(options.cargo_home.expanduser().resolve())
    variants = ("silent", "diagnostic") if options.variant == "all" else (options.variant,)
    latest_build = ROOT.parent / f"Latestbuild-{RELEASE_VERSION}"
    for variant in variants:  # Keep builds sequential: their Cargo output path is shared.
        build_variant(variant, env)
        if variant == "silent":
            copy_release_output(
                ROOT / "releases" / variant,
                latest_build,
            )
    latest_build.mkdir(parents=True, exist_ok=True)
    (latest_build / RELEASE_SUMMARY_NAME).write_bytes(summary)
    print(f"Copied release summary to {latest_build / RELEASE_SUMMARY_NAME}", flush=True)
    create_source_backup(latest_build)


if __name__ == "__main__":
    main()
