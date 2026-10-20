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

The helper defaults to building both silent and diagnostic sequentially. `-v silent` or `-v diagnostic` selects just one. It verifies the NRO header and selected logger/message markers, snapshots the binary/config, and checks the ZIP contents. It never installs or uploads anything; uploading is a separate command. It requires Python 3.11 or newer and no third-party Python packages. If your Cargo cache needs an isolated directory, pass `--cargo-home /path/to/cache`; this changes only the build process's environment.

Outputs:

```text
releases/kirby_assist-<version>-silent.zip
releases/kirby_assist-<version>-diagnostic.zip
releases/silent/atmosphere/...
releases/diagnostic/atmosphere/...
```

The ZIP version is read from `Cargo.toml`, for example
`kirby_assist-1.1.8-silent.zip`. Different release versions keep separate archives.

After the selected builds finish, the helper creates separate flat folders in
`Latestbuild-<version>`:

```text
Latestbuild-<version>/
    silent/
        libkirby_assist.nro
        config.txt
        RELEASE_SUMMARY-<version>.txt
        kirby_assist-<version>-source.zip
    diagnostic/
        libkirby_assist.nro
        config.txt
        RELEASE_SUMMARY-<version>.txt
        kirby_assist-<version>-source.zip
```

Build both folders with `python3 tools/build_variants.py -v all`. A single-variant
build refreshes only its own folder; it leaves the other variant unchanged.
Each NRO/config is copied from that variant's staged release, not from the shared
Cargo output. Each summary is labeled with its version and SILENT or DIAGNOSTIC variant, and the source ZIP is generated
once and copied byte-for-byte with SHA-256 verification. The separate uploader
selects one of these folders by version and variant. Building never uploads.

The source backup contains the project under a `kirby_assist_v2/` folder.
Generated `target`, `releases`, cache, and Git metadata directories are excluded.
The helper reopens the archive and verifies its CRC and file list. Older backups
and legacy flat files at the Latestbuild root are left untouched; use the new
variant subfolders for current outputs.
The output directory is beside the project and uses `Latestbuild-<RELEASE_VERSION>`;
change `[package].version` in the root `Cargo.toml` to create a separate release directory.
That is the single release-version source: the builder reads it and Rust uses
`CARGO_PKG_VERSION` for the diagnostic startup message. Cargo updates its lockfile
when building; the core library has its own independent package version.

All maintained guides and assets are packaged under `docs/`, alongside the
root `mkdocs.yml` and `requirements-docs.txt`. New pages are picked up
automatically. See [maintaining the docs](MAINTAINING.md) for preview and build
commands. Generated `site/` HTML and `.venv-docs/` are excluded from source ZIPs.

Set `[package].description` in the root `Cargo.toml` to the desired release
summary. On each run, the builder generates or overwrites
`releases/RELEASE_SUMMARY-<version>.txt` using exactly that parsed string, with no
extra heading or appended newline. TOML multiline strings are supported. A
missing, empty or whitespace-only description stops the build before compilation.
Each packaged summary prepends a version and build label to that description:

```text
Kirby Assist 1.1.0
Build: DIAGNOSTIC

Huge Update
```

Silent packages use `Build: SILENT`. The labeled summary is included in the
variant ZIP and copied to its matching Latestbuild folder. The top-level
`releases/RELEASE_SUMMARY-<version>.txt` and source ZIP retain the common description
without a variant label because the source is shared. The uploader transfers
the selected Latestbuild variant's summary without modifying it.

With no `-v` flag, both variants are built, including when using `--cargo-home`. Use `-v silent` or `-v diagnostic` to build just one variant. The long alias `--variant` remains accepted.

## Separate FTP uploader

Close Smash, start the Switch FTP server, then select the built version and variant:

```sh
# Silent build (also the default if -s/-d is omitted):
python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s

# Diagnostic build; asks for yes before connecting:
python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -d

# Validate the selected local files and display paths without connecting:
python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s --dry-run

# Compare installed bytes against that build without writing or backing up:
python3 tools/upload_switch.py 192.0.2.10 -v 1.1.2 -s --verify-only
```

Replace `192.0.2.10` with your Switch's IP address; it is a documentation-only
example address.

`-v` / `--version` is required. `-s` / `--silent` and `-d` / `--diagnostic`
are mutually exclusive. The selected source is always beside the project:

```text
AutoSmashMod/Latestbuild-1.1.2/silent/
AutoSmashMod/Latestbuild-1.1.2/diagnostic/
```

The old `UPLOAD` folder is no longer used; no manual copying is needed. Paths do
not depend on the current working directory. The uploader never builds or extracts
archives. It validates the selected folder, release-summary version and Build label,
NRO header and diagnostic markers, and optional source ZIP structure/CRCs before FTP.
A missing version/variant folder or mismatched metadata stops the operation.

FTP defaults to port **5000**, anonymous login, and SD root **/**. Optional settings
are `--port 21`, `--root /sdcard`, `--user NAME`, and
`--password-env SWITCH_FTP_PASSWORD`. Diagnostic uploads still require typing
`yes` at the confirmation prompt; dry runs and verification do not prompt.

The fixed installation destinations remain:

```text
/ultimate/kirby_assist/config.txt
/ultimate/kirby_assist/RELEASE_SUMMARY.txt
/ultimate/kirby_assist/kirby_assist-source.zip  # when present in the selected build
/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro
```

Before replacing any installed file, the uploader retrieves **all existing selected
files**, writes their original bytes directly to the fixed backup folder,
and verifies every backup by read-back SHA-256:

```text
/ultimate/kirby_assist/bak/
    config.txt
    RELEASE_SUMMARY.txt
    kirby_assist-source.zip  # if selected and previously installed
    libkirby_assist.nro
```

These are backups of the files installed on the Switch, not the incoming local
build. Each run overwrites these same backup filenames, keeping one previous
installed set with no timestamp subfolder. The script
creates `/ultimate/kirby_assist/bak` if needed; installation directories must already exist.
A read, backup write, or backup verification failure stops before any installed
file is overwritten. Files confirmed absent by a parent directory listing need no
backup. An unreadable existing file is an error, not a missing file.

After **all** backups pass, each replacement is uploaded and verified by read-back
SHA-256; the NRO goes last. A failure during replacements can still leave a partial
update. The error reports progress and the backup path; no automatic rollback or
cleanup occurs. Keep Smash closed until the update is completed or the saved files
are restored to the original destinations listed above.

Only the selected installation files and the fixed backup files are written.
Other plugins, logs and versioned summaries remain untouched. There is no folder
sync, deletion, rename, or ZIP extraction. `--root /sdcard` prefixes both installation
and backup paths. `--verify-only` and `--dry-run` never create backups.

Tests use temporary input files and fake FTP without building or connecting:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

## Install exactly one variant

1. Fully close Smash.
2. Extract your chosen ZIP and copy its `atmosphere/` and `ultimate/` directories to the SD-card root.
3. Replace the existing plugin and config. Both variants install as `libkirby_assist.nro` at the same path. Do not rename them and load both.
4. Restart Smash. Use Player 1 (`player_index = 0`), select Kirby, and map ZR to Shield.

The supplied gameplay settings are identical, including [held-L3 turbo teabag](TURBO_CONTROLS.md), held-Y ledge assist and [D-pad slide attempts](SLIDE_CONTROLS.md). Silent's config has `tech_debug = false`; diagnostic's has it set to `true`. Changing that key cannot bring logging back into a silent binary. Logging can affect callback timing/IO overhead even though the gameplay policy is shared.

## What silent actually excludes

The logger module, background writer thread/queue, explicit console printing, SD log path, startup directory/file creation, queued tech messages, and logging-only counters are guarded by compile-time `cfg` attributes. A disabled log macro also removes its message expression, so diagnostic formatting is not evaluated in the silent build.

The plugin still reads `SD:/ultimate/kirby_assist/config.txt`, loads through Skyline, scans the executable, installs its hooks, and changes raw button states plus the left stick during the held turbo control. Normal native calls and runtime failure paths still exist. Do not interpret an absent diagnostic string as proof that a module or gameplay change cannot be observed.

Silent does not delete/truncate a previous `status.log`; an old file may therefore remain unchanged after switching variants. If config is invalid, the mod is disabled, or the executable signature is unsupported, the silent build returns without giving a diagnostic. Switch to the diagnostic build to investigate.

Diagnostic writes to console and `SD:/ultimate/kirby_assist/status.log`. Its first line identifies `Kirby Assist diagnostic build <version>`. `tech_debug = false` disables detailed tech messages but leaves basic/startup logging enabled.

## Diagnostic performance

Gameplay hooks no longer open or write the SD log. They submit messages with
`try_send` to a 128-message queue; one diagnostic-only worker keeps the file open,
buffers console/file output and flushes about every 250 ms. If output stalls and
the queue fills, new messages are dropped and a later `diagnostic queue: dropped`
line reports the count. Gameplay never waits for the file/console writer.

Ledge observations are formatted on meaningful changes (option/target, readiness,
action phase, motion, input request or hit confirmation), rather than every
animation frame. `tech_debug=true` also enables a waiting-state snapshot at most
twice per second. `periodic=true` identifies those snapshots;
`coalesced_updates` counts suppressed unchanged observations. Exact animation
frames, positions, protection/recovery metadata and native gates remain in each
emitted snapshot. Actual `ledge mapper` input-edge and tech-edge logs remain.
Set `tech_debug=false` for event-only ledge logging and fewer tech details.

This reduces logging overhead but is not an on-device FPS guarantee. The final
buffered messages can be lost in a crash/power loss; allow at least a moment for
the worker to flush before collecting the log. Silent has no logger thread/queue.
The source change needs a new diagnostic NRO to take effect.

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

The source includes [predictive grounded ledge assistance](LEDGE_TRAP.md) plus [analog-stick mashing](ANALOG_MASH.md). Ledge fixes include exact hanging-status checks, missed-option recovery, startup-aware shield grabs, and tilt retries after a release without an extra timer. The supplied config selects forward tilt for normal get-up and outward throws after capture; post-punish recovery returns to the ledge stance. Host checks do not refresh release ZIPs; install the rebuilt plugin and updated config together.

Use A=Attack, R=Grab, ZR=Shield, and Tap Jump off. Set `ledge_getup_attack = forward_tilt` to try the downward-angled forward tilt for normal get-up. Ledge jumps use short-hop back air; roll uses uncharged forward smash and get-up attack uses Grab. Both variants share the gameplay policy; silent keeps logging compiled out. Device testing remains required, including the Pichu-on-FD small-character baseline. No guaranteed frame-perfect coverage is claimed.
