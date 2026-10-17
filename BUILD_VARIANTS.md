# Silent and diagnostic builds

The two variants share the same gameplay code. Logging is selected at compile time with the `diagnostic-logging` Cargo feature; it is independent of Rust's optimized `--release` profile. Both supplied builds are optimized releases.

| Variant | Explicit plugin file/console logging | Detailed tech logs | Config reading |
| --- | --- | --- | --- |
| Silent (default feature set) | Compiled out | Compiled out, even with `tech_debug = true` | Enabled |
| Diagnostic | Enabled | Enabled when `tech_debug = true` | Enabled |

Silent means this plugin's explicit logger and diagnostic collection are absent. It does **not** hide the loaded NRO or hooks, erase old logs, suppress crashes, or control Skyline/OS/game reporting. It is not a ban-prevention or hardware-indistinguishability guarantee.

## Build commands

From the project root with a configured Skyline toolchain:

```sh
# Silent, optimized plugin. This is also the default for cargo skyline build --release.
cargo skyline build --release --no-default-features

# Diagnostic, optimized plugin.
cargo skyline build --release --no-default-features --features diagnostic-logging
```

Each command writes `target/aarch64-skyline-switch/release/libkirby_assist.nro`. Copy that result before building the other variant, or use the helper below to keep them separate automatically.

```sh
python3 tools/build_variants.py
```

The helper builds each variant sequentially, verifies the NRO header and selected logger/message markers, snapshots the binary/config, and checks the ZIP contents. It does not flash or install anything. It requires no third-party Python packages. If your Cargo cache needs an isolated directory, pass `--cargo-home /path/to/cache`; this changes only the build process's environment.

Outputs:

```text
releases/kirby_assist-silent.zip
releases/kirby_assist-diagnostic.zip
releases/silent/atmosphere/...
releases/diagnostic/atmosphere/...
```

Use `--variant silent` or `--variant diagnostic` to rebuild just one.

## Install exactly one variant

1. Fully close Smash.
2. Extract your chosen ZIP and copy its `atmosphere/` and `ultimate/` directories to the SD-card root.
3. Replace the existing plugin and config. Both variants install as `libkirby_assist.nro` at the same path. Do not rename them and load both.
4. Restart Smash. Keep the Pro Controller first (`npad_id = 0`), select Kirby, and map ZR to Shield.

The supplied gameplay settings are identical, including [directional turbo crouch](TURBO_CONTROLS.md) on D-pad Left/Right. Silent's config has `tech_debug = false`; diagnostic's has it set to `true`. Changing that key cannot bring logging back into a silent binary. Logging can affect callback timing/IO overhead even though the gameplay policy is shared.

## What silent actually excludes

The logger module, explicit console printing, SD log path, startup directory/file creation, queued tech messages, and logging-only counters are guarded by compile-time `cfg` attributes. A disabled log macro also removes its message expression, so diagnostic formatting is not evaluated in the silent build.

The plugin still reads `SD:/ultimate/kirby_assist/config.txt`, loads through Skyline, scans the executable, installs its hooks, and changes raw button states plus the left stick during the held turbo control. Normal native calls and runtime failure paths still exist. Do not interpret an absent diagnostic string as proof that a module or gameplay change cannot be observed.

Silent does not delete/truncate a previous `status.log`; an old file may therefore remain unchanged after switching variants. If config is invalid, the mod is disabled, or the executable signature is unsupported, the silent build returns without giving a diagnostic. Switch to the diagnostic build to investigate.

Diagnostic writes to console and `SD:/ultimate/kirby_assist/status.log`. Its first line identifies `Kirby Assist diagnostic build v18`. `tech_debug = false` disables detailed tech messages but leaves basic/startup logging enabled.

## Verification and remaining limits

The helper checks that our selected log markers are absent from the silent NRO and present in the diagnostic NRO. This is a binary/source check, not a measurement of Nintendo telemetry, panic output, other plugins, or every dependency's failure behavior. `VALIDATION.json` records hashes and the exact feature selection for each package.

On hardware, compare the existing SD log's timestamp/content before and after running the silent variant; it should not be created or changed by this plugin. Then install diagnostic and verify a fresh log appears. Compare ordinary movement, mash, tech, release, pause/resume, and controller reconnect behavior with each variant. Those two-variant hardware checks have not been performed here.

Core checks remain:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
cargo clippy --manifest-path crates/kirby_assist_core/Cargo.toml --all-targets -- -D warnings
cargo skyline clippy --no-default-features
cargo skyline clippy --no-default-features --features diagnostic-logging
```

The next source build identifies v18 and includes [predictive grounded ledge assistance](LEDGE_TRAP.md) plus [analog-stick mashing](ANALOG_MASH.md). Ledge fixes include exact hanging-status checks, missed-option recovery, startup-aware shield grabs, and tilt retries after a release without an extra timer. The supplied config selects forward tilt for normal get-up and outward throws after capture; post-punish recovery returns to the ledge stance. Host checks do not refresh release ZIPs; install the rebuilt plugin and updated config together.

Use A=Attack, R=Grab, ZR=Shield, and Tap Jump off. Set `ledge_getup_attack = forward_tilt` to try the downward-angled forward tilt for normal get-up. Other ledge options retain up tilt or Grab responses. Both variants share the gameplay policy; silent keeps logging compiled out. Device testing remains required, including the Pichu-on-FD small-character baseline. No guaranteed frame-perfect coverage is claimed.

v18 adds smoother homing, early catch-state anchor reads, two-observation target retention without stale output, and separate tilt/grab approach distances. Copy the updated config to get the 9-unit stance. Source checks do not regenerate release packages.
