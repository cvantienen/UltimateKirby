# UltimateKirby

Auto mash and auto tech for human-controlled Kirby, using **raw Pro Controller stick/button states** before Smash's controller mapping. There is no training-mode restriction.

**v6 adds [directional turbo crouch](TURBO_CONTROLS.md):** hold D-pad Left or Right to crouch/release rapidly while moving slightly that direction. Side taunts are suppressed; releasing restores physical stick control. Timing and movement strength are configurable. This is included in both builds.

**Two optimized builds:** silent (plugin file/console logging compiled out) and diagnostic (startup/activity logs plus optional detailed tech logs). Gameplay policy is shared. See [build variants and installation](BUILD_VARIANTS.md). The existing tech retry fix, analog-stick mashing, and prediction remain in both.

The plugin now contains a raw controller-mapping hook and fighter-state observation. **This is a compiled on-device test candidate, not a hardware-verified release.** Auto tech uses a collision-prediction heuristic; its timing and compatibility need testing on the target Switch.

Start with [installation, settings, and the status-log guide](INTEGRATION.md). The build helper generates `releases/kirby_assist-silent.zip` and `releases/kirby_assist-diagnostic.zip` locally. Generated packages, build output, logs, caches, and machine-specific editor settings are excluded from Git. Install exactly one variant.

## Source checkout

```sh
git clone https://github.com/cvantienen/UltimateKirby.git
cd UltimateKirby
```

The repository/folder is **UltimateKirby**; the existing Rust package and installable plugin remain `kirby_assist` / `libkirby_assist.nro` so installation paths stay compatible. After installing the Rust/Skyline prerequisites in [BUILD_VARIANTS.md](BUILD_VARIANTS.md), build a silent package with:

```sh
python3 tools/build_variants.py --variant silent
```

For Cargo cache permission problems, add `--cargo-home /private/tmp/kirby-assist-build-cargo`. Source checks do not refresh release NROs/ZIPs.

## What it does

- Auto mash: cycle the left stick Right → Up → Left → Down while the game's clatter timer is positive; one full-strength direction per observed input update.
- Auto tech: raw ZR pulse near a predicted stage collision in an eligible tumble state.
- Mash takeover suppresses physical action buttons and C-stick during the left-stick cycle. Tech still owns only ZR. Physical controls return when the active tool ends; manual turbo owns the left stick while held in eligible movement.
- Manual control: hold D-pad Left/Right for directional turbo crouch during normal grounded movement.
- Targets one configured Pro Controller and its human-controlled Kirby.
- Leaves control mapping, input recognition, tech eligibility, and lockouts to Smash.

This temporarily overlays raw controller button memory and its press/release history before the game maps it, then restores the physical input structure. It does not generate actual USB/Bluetooth hardware packets or claim universal indistinguishability from physical controller signals.

Runs directly on your Switch with your Pro Controller; no extra controller hardware, network connection, or computer is needed during play.

The default controller is the first Pro Controller (`npad_id = 0`). ZR must be mapped to Shield. Change individual tools in `SD:/ultimate/kirby_assist/config.txt` and restart Smash.

## Understand or extend the code

Read the [Rust beginner guide](README_BEGINNER.md) for structs, enums, traits, rules, timing, and the logical frame flow. Run its pretend-adapter example locally:

```sh
cargo run --manifest-path crates/kirby_assist_core/Cargo.toml --example walkthrough
```

| File | Purpose |
| --- | --- |
| `src/lib.rs` | Starts the live integration. |
| `crates/kirby_assist_core/src/manual_controls.rs` | D-pad triggers, crouch/move phases, taunt suppression, and stick hand-back. |
| `src/logging.rs` | File/console logger, compiled only into the diagnostic variant. |
| `tools/build_variants.py` | Builds, verifies, and packages separate silent/diagnostic artifacts. |
| `src/live.rs` | Raw controller-memory hook, controller association, fighter observation, collision prediction. |
| `crates/kirby_assist_core/src/native_input.rs` | Pinned controller ownership chain and raw held/pressed/released overlay. |
| `crates/kirby_assist_core/src/tech_prediction.rs` | Swept collision probes around the approximate fighter footprint. |
| `crates/kirby_assist_core/src/ledge_trap.rs` | Hold-Down ledge acquisition, tilt preparation, shield response, and cancellation (source addition). |
| `src/game_ffi.rs` | Narrow game ABI declarations checked against pinned upstream bindings. |
| `crates/kirby_assist_core/src/live_input.rs` | Live settings, analog mash/ZR request policy, duplicate-observation handling, signature scanning. |
| `crates/kirby_assist_core/src/tools.rs` | Auto-tech and auto-mash logical presets. |
| `crates/kirby_assist_core/src/config.rs` | Scenario rules, timing, toggles, fighter/controller scope. |
| `crates/kirby_assist_core/src/assist.rs` | Stateless priority selection and logical decisions. |
| `crates/kirby_assist_core/src/input.rs` | Logical controller values and additive arbitration. |
| `crates/kirby_assist_core/src/runtime.rs` | Generic adapter runner used by the teaching example. |
| `crates/kirby_assist_core/src/sequence.rs` | Sequence sampling; recorded scenario playback remains unimplemented. |

The Switch mapping path uses `RawAutoControls` and `AssistEngine`; it does not use the generic runner to rebuild a native controller. Live mash translation produces normalized left-stick floats and matching direction bits, while tech produces raw ZR. The older button-group/interval config entries are ignored. [ANALOG_MASH.md](ANALOG_MASH.md) explains the cycle, takeover, cancellation, and device tests. Actual escape speed has not been benchmarked on Switch.

## Build and check

From this project folder, build the Switch plugin with:

```sh
# Default feature set is silent.
cargo skyline build --release

# To include the plugin logger for testing:
cargo skyline build --release --features diagnostic-logging

# Build/package both variants in separate folders:
python3 tools/build_variants.py
```

`cargo skyline` must already be installed and its Switch toolchain configured. `--release` selects the optimized build. Ordinary `cargo build` is not the Switch plugin build command.

The command creates `target/aarch64-skyline-switch/release/libkirby_assist.nro`. It does not install it on your Switch or refresh the prebuilt ZIP. See [rebuilding the SD-card package](INTEGRATION.md#rebuild-the-sd-card-packages) for the copy/ZIP commands.

For checks and formatting:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
cargo clippy --manifest-path crates/kirby_assist_core/Cargo.toml --all-targets -- -D warnings
cargo fmt --all
cargo fmt --manifest-path crates/kirby_assist_core/Cargo.toml
cargo skyline clippy
cargo skyline build --release
```

Release output: `target/aarch64-skyline-switch/release/libkirby_assist.nro`.

The integration guide records the exact upstream ABI sources, current limitations, deployment paths, and on-device checks. The `.nro` build verifies compilation; a hardware test is required to establish that Smash accepts the intended inputs.

## Hold-Down ledge assist (source only)

Hold D-pad Down before a nearby opponent chooses a ledge option. The v18 source uses an 9-unit inward stance with momentum-aware positioning, pursues normal-get-up/roll travel, and predicts target position and invincibility expiry across attack active frames. It remembers short jump transitions and retries recovered tilt whiffs until a hit is confirmed, within a bounded budget. Shield holds and direct R follow-ups remain. Use **A=Attack, R=Grab, ZR=Shield, Tap Jump off**.

The supplied config now selects downward-angled forward tilt for normal get-up; `ledge_getup_attack = down_tilt` remains available. v18 fixes normal idle being treated as hanging, recovers missed get-up/roll transitions, removes the extra tilt retry timer, and predicts shield-grab reach/vulnerability. Confirmed captures throw toward the remembered ledge with `ledge_throw_outward = true`; after a punish Kirby returns to the original stance while Down stays held. Jump still uses up tilt, and roll/shield follow-ups use Grab. [LEDGE_TRAP.md](LEDGE_TRAP.md) explains the model and the planned Pichu-on-FD device checks. Synthetic policy tests do not establish collision results or frame-perfect coverage. Rebuild and install the plugin plus updated config; existing ZIPs were not refreshed for v18.

The v18 positioning update acquires cliff anchors during catch/catch-move, tolerates two missing mapper observations while suspending output, and separates pursuit distances from attack range. Movement tapers near the target and the stance uses different start/stop thresholds to reduce oscillation. Tune `ledge_tilt_approach_distance` and `ledge_grab_approach_distance` separately from hit/grab range; the supplied stance is 9 world units inward.
