#!/usr/bin/env python3
"""Build/package both logging variants without installing anything on a Switch.

This uses only Python's standard library. Run it from any working directory;
ROOT comes from this script's own location, not the shell's current directory.
Cargo/Skyline must already be installed. An optional --cargo-home can select
an isolated cache if the normal cache has permission problems.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PLUGIN = Path("atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro")
CONFIG = Path("ultimate/kirby_assist/config.txt")
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


def build_variant(name: str, env: dict[str, str]) -> Path:
    """Compile once, then snapshot the result before the next build overwrites it."""
    diagnostic = name == "diagnostic"
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
    install = f"""Kirby Assist v18 - {name.upper()} BUILD

Close Smash fully. Copy atmosphere/ and ultimate/ to the SD-card root, replacing
this mod's plugin/config. Install ONE variant at a time: both use the same .nro
name and destination. Do not rename them and install both together.

Use the first Pro Controller (npad_id = 0), human Kirby, and ZR mapped to Shield.
Gameplay policy is shared: left-stick mash, tech, and directional turbo crouch.
Mash cycles Right/Up/Left/Down every distinct mashable input update, with no
automated action buttons. Supplied takeover blocks physical action buttons and
C-stick during mash, then restores them. See ANALOG_MASH.md for device checks.
Hold D-pad Left/Right for turbo crouch with slight movement that direction.
Both held cancels movement; release restores physical stick. Side taunts are
consumed during valid gameplay. Runs only in normal grounded movement for Kirby.
Tune turbo_phase_frames / turbo_walk_strength in the config and restart Smash.
Hold D-pad Down near an opponent hanging at ledge for reactive ledge assistance.
Requires A=Attack, R=Grab, ZR=Shield, Tap Jump OFF.
Start near the ledge; coverage depends on spacing and native action timing. See LEDGE_TRAP.md for scope/limitations and TURBO_CONTROLS.md for turbo.
The ledge policy needs hardware testing. v18 predicts active-window reach and
invincibility expiry, pursues normal get-up/roll travel, and retries tilt whiffs.
Default stance is now 9 units inward: copy the new config with the new plugin.
Bundled ledge_getup_attack uses downward-angled forward_tilt for normal
get-up; down_tilt is also available. Jumps still use up tilt. Test Pichu on FD as a small-character baseline.
v18 fixes idle being mistaken for hanging, recovers missed option transitions,
removes extra tilt retry delay, and predicts shield-grab reach/protection.
With ledge_throw_outward=true, a confirmed capture gets a directional throw
toward the remembered ledge. After recovery Kirby returns to his ledge stance.
v18 acquires anchors in catch/catch-move as well as hang, retains identity
through two missing mapper observations without acting on stale data, tapers
movement near targets, and uses stance hysteresis to reduce oscillation.
ledge_tilt_approach_distance and ledge_grab_approach_distance independently tune
pursuit separation; changing attack range no longer sets pursuit distance.
No frame-perfect/unavoidable coverage is claimed. See LEDGE_TRAP.md.


Silent: no explicit plugin file/console logger, startup log creation, or detailed
message collection. It still reads config. tech_debug cannot enable logging.
It does not delete old files or change Skyline/OS/game crash reports or telemetry.
A missing/invalid config or unsupported signature produces no silent diagnostic;
use the diagnostic variant to troubleshoot instead.

Diagnostic: startup/basic logs plus detailed tech logs with tech_debug = true.
Output goes to console and SD:/ultimate/kirby_assist/status.log. The first line
identifies 'Kirby Assist diagnostic build v18'. tech_debug = false reduces details,
but does not disable startup/basic logging in this variant.

Neither variant hides the plugin or guarantees indistinguishability/ban protection.
Read BUILD_VARIANTS.md for scope, build commands, and hardware verification steps.
"""
    # Stage 2: describe what this script actually checked. SHA-256 identifies
    # bytes for later comparison; it says nothing about in-game input timing.
    validation = {
        "build_id": "v18-smooth-ledge-positioning",
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
        "VALIDATION.json": (json.dumps(validation, indent=2) + "\n").encode(),
    }
    for guide in ("BUILD_VARIANTS.md", "TURBO_CONTROLS.md", "LEDGE_TRAP.md", "ANALOG_MASH.md", "README.md", "README_BEGINNER.md", "INTEGRATION.md"):
        files[guide] = (ROOT / guide).read_bytes()

    # Separate folders prevent a later feature build from overwriting the earlier
    # variant's installable binary. The ZIP contains the same reviewed bytes.
    stage = ROOT / "releases" / name
    for relative, data in files.items():
        target = stage / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
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


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--variant", choices=("all", "silent", "diagnostic"), default="all")
    parser.add_argument("--cargo-home", type=Path, help="Optional isolated Cargo cache")
    # --variant silent/diagnostic selects one package; the default "all" builds
    # both in order. --cargo-home changes dependency cache location only, not
    # runtime configuration or which gameplay features the plugin contains.
    options = parser.parse_args()
    env = os.environ.copy()  # Do not mutate the caller's shell configuration.
    if options.cargo_home:
        env["CARGO_HOME"] = str(options.cargo_home.expanduser().resolve())
    variants = ("silent", "diagnostic") if options.variant == "all" else (options.variant,)
    for variant in variants:  # Keep builds sequential: their Cargo output path is shared.
        build_variant(variant, env)


if __name__ == "__main__":
    main()
