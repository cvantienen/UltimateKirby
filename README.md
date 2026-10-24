# Kirby Assist

Kirby Assist is a Rust Skyline plug-in for Super Smash Bros. Ultimate. It adds
controller-input assists for the selected local human fighter, including compiled
fighter-specific ledge responses, auto mash, auto tech, SDI and manual shortcuts.

This README is the quick reference for controls, documentation, testing, building
and uploading. The longer guides explain the implementation and calibration in
detail.

> Host tests and successful builds cannot prove gameplay timing or compatibility
> on a Switch. Use the diagnostic build and on-device testing for that work.

## TODO

### Character-specific changes needed

- [ ] Fix and device-test Kirby's forward-facing attack-cancel bair route; it can
  still turn into forward tilt instead of bair.
- [ ] Continue fighter-by-fighter roll-grab calibration for approach distance,
  facing, grab range and the vulnerable-frame target.
- [ ] Finish on-device calibration of Yoshi's Neutral short-hop nair and bair
  choices, including hit timing, fast fall, drift and handback.
- [ ] Validate each compiled fighter's ledge choices on hardware and record any
  timing, spacing or move-delivery adjustments on that fighter's page.
- [ ] Add or tune fighter-specific response choices as testing exposes gaps.

### System-level changes

- [ ] Investigate the doubles connection error.
- [ ] Confirm diagnostic R3-menu/full-feature Training testing with stock Training Modpack v7.1.1 in
  Training Mode on the diagnostic Switch.
- [ ] Measure silent and diagnostic callback cost on the Switch and keep logging,
  result-card rendering and observations within the frame-time budget.
- [ ] Compare offline and Arena timing with controlled diagnostic captures; keep
  network delay findings separate from local input-delivery bugs.
- [ ] Revisit live combo playback later; it is currently removed.

## What is loaded at runtime

- Shared settings load once from `SD:/ultimate/kirby_assist/config.txt` when
  Smash starts.
- Fighter profiles, menu choices, timings and action sequences are compiled into
  the NRO. There is no runtime fighter JSON.
- Editing `config.txt` requires a Smash restart.
- Editing Rust requires a new build and upload.
- Editing Markdown only requires rebuilding the documentation.
- Silent and diagnostic builds install to the same path. Install one variant at
  a time.

See [installation and configuration](docs/INTEGRATION.md),
[all fighters](docs/FIGHTERS.md) and the [source map](docs/CODE_MAP.md).

## Current controls

**Diagnostic + Training Modpack + Training Mode:** click **R3** for Kirby
Assist's menu, then use the D-pad. Normal **L/Y/X** controls and all configured
assists remain enabled; R3 no longer needs to be held. Real Minus/Plus and
Training menu/recording chords hand input to Training. Default ZL+Up load and
ZL+Down save keep our panel visible and resume held-L practice on release,
without R3. Finish/close Training's menu or
recording, then tap R3 to resume Kirby Assist with its menu visible. All other
setups use R3 for our menu. No Training Modpack rebuild is required.
See [Training compatibility](docs/INTEGRATION.md#training-modpack-compatibility).

| Input | Action |
| --- | --- |
| Hold **L** | Run the selected fighter's ledge autopilot response |
| D-pad **Up/Down**, menu visible | Select ledge lists; Kirby/Yoshi Roll read; Kirby/Yoshi Finisher; shared Auto-mash |
| D-pad **Left/Right**, menu visible | Cycle the selected row's saved response |
| D-pad **Left/Right**, menu hidden | Start or repeat a downward air-dodge slide |
| D-pad **Down**, menu hidden | Attempt a ledge trump |
| D-pad **Up**, menu hidden | Normal mapped D-pad input |
| **R3** | Show/hide the menu |
| Hold **L3** without L | Turbo teabag; release to stop |
| Auto-mash row, Left/Right | Toggle mash ON/OFF |
| Press **X** | X short-hop tap while grounded |
| Tap **A+B** as Kirby, neutral stick | Jab 1 → grab attempt; A alone stays normal (A+B grab row ON) |
| Press **Y** as Kirby | Attempt selected Finisher: Fair 1 > Fsmash or Dair > Dsmash |
| Press **Y** as Yoshi | Bair drag > Fsmash attempt, only finishing on an early hit |
| Hold **L** as Kirby | Ledge responses plus in-range down-tilt recovery cover |

Held-L autopilot also has an approximate grounded shield safeguard against
nearby active or known imminent fighter attacks, including opponents who did not grab ledge.
It uses root positions and a three-frame velocity projection, not exact attack
shapes. It does not detect projectiles or grabs and does not interrupt committed
punishes. See [defensive shielding](docs/LEDGE_TRAP.md#held-l-defensive-shield).
Kirby's [Finisher row](docs/FINISHERS.md) selects a single Y starter and its
hit-confirmed landing smash. Auto-duck is disabled, including its hitbox-reader
hook. B-reverse is unbound. Roll read offers
Off / Down smash / Grab for ordinary onstage rolls.

Whiff punish is removed from the live build/menu while reliability is refined.
[Opponent reference profiles](docs/OPPONENT_DATA.md) remain available for future
development; no match-time file or website reads are introduced.

Required controller mapping:

- A = Attack
- B = Special
- R = Grab
- ZR = Shield
- X and ZL = Jump
- Tap Jump off
- C-stick = Tilt Attack for Kirby's forward-facing attack-cancel bair

The menu begins visible in the top-left of each battle and closes outside
battle. Kirby recovery cover is part of held-L autopilot, with no separate R3
binding. Press Y for one selected finisher attempt; holding it does not loop.
B-reverse and the older general combo engine remain removed. Read the full
[control reference](docs/CONTROLS.md) before diagnosing a missed shortcut.

## Start in the project folder

Run commands from the directory containing this README and `Cargo.toml`:

```sh
cd /Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2
```

Useful read-only checks:

```sh
git status --short
git diff --stat
git diff
```

## Documentation commands

Create the isolated documentation environment once:

```sh
/usr/bin/python3 -m venv .venv-docs
.venv-docs/bin/python -m pip install -r requirements-docs.txt
```

Use `/usr/bin/python3` here if Homebrew Python fails during `ensurepip` or cannot
import `pyexpat`.

Preview the documentation at `http://127.0.0.1:8000/`:

```sh
.venv-docs/bin/python -m mkdocs serve
```

Build all documentation and fail on warnings or broken links:

```sh
.venv-docs/bin/python -m mkdocs build --strict
```

Generated HTML goes to `site/`; it is local and is not uploaded to the Switch.

After editing fighter choices, defaults, names or timing declarations, regenerate
the 77 individual fighter pages and verify they are current:

```sh
python3 tools/update_fighter_docs.py
python3 tools/update_fighter_docs.py --check
```

Run the documentation/tooling tests without compiling the Switch plug-in:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

When adding a general guide, add it to `mkdocs.yml`. Fighter pages stay under
`docs/fighters/`, are linked from [FIGHTERS.md](docs/FIGHTERS.md), and do not get
individual sidebar entries. See [maintaining the docs](docs/MAINTAINING.md).

## Source checks and host tests

Check formatting without rewriting files:

```sh
cargo fmt --all -- --check
cargo fmt --manifest-path crates/kirby_assist_core/Cargo.toml -- --check
```

Run the platform-independent core tests:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
```

These tests do not run Smash, the Skyline hooks or Switch input mapping.

## Build and package

The recommended builder creates optimized silent and diagnostic variants,
validates their NRO headers and logging markers, packages release ZIPs and creates
a verified source backup. It does not upload anything.

Build both variants with the isolated Cargo cache normally used for this project:

```sh
python3 tools/build_variants.py --cargo-home /private/tmp/kirby-assist-build-cargo
```

Build only one variant:

```sh
python3 tools/build_variants.py -v silent --cargo-home /private/tmp/kirby-assist-build-cargo
python3 tools/build_variants.py -v diagnostic --cargo-home /private/tmp/kirby-assist-build-cargo
```

The version comes from `[package].version` in `Cargo.toml`. Output is written to:

```text
releases/kirby_assist-<version>-silent.zip
releases/kirby_assist-<version>-diagnostic.zip
../Latestbuild-<version>/silent/
../Latestbuild-<version>/diagnostic/
```

Direct Cargo commands are useful for troubleshooting but do not create the full
verified release package:

```sh
# Optimized silent NRO with the menu.
cargo skyline build --release --no-default-features --features imgui-menu

# Optimized diagnostic NRO with the menu and logger.
cargo skyline build --release --no-default-features --features imgui-menu,diagnostic-logging
```

See [silent and diagnostic builds](docs/BUILD_VARIANTS.md) for the exact package
contents and validation limits.

## Upload to the Switch

Close Smash completely and start the Switch FTP server before an actual upload.
The uploader reads `../Latestbuild-<version>/<variant>/`; it never builds.

Set the values for the build being shipped:

```sh
SWITCH_IP=192.168.1.142
VERSION=1.5.5
```

First print and validate the silent upload plan without connecting:

```sh
python3 tools/upload_switch.py "$SWITCH_IP" -v "$VERSION" -s --dry-run
```

Upload silent or diagnostic:

```sh
python3 tools/upload_switch.py "$SWITCH_IP" -v "$VERSION" -s
python3 tools/upload_switch.py "$SWITCH_IP" -v "$VERSION" -d
```

Diagnostic upload asks you to type `yes` before connecting. Before replacement,
the uploader reads the selected installed files, copies them to
`SD:/ultimate/kirby_assist/bak/`, and verifies those backups. Each uploaded file
is then verified by reading it back. The fixed backup folder stores one previous
copy and is overwritten by the next successful backup.

After uploading and restarting Smash, compare the installed files to the selected
local build without writing anything:

```sh
python3 tools/upload_switch.py "$SWITCH_IP" -v "$VERSION" -s --verify-only
python3 tools/upload_switch.py "$SWITCH_IP" -v "$VERSION" -d --verify-only
```

Use the flag matching the installed variant. If FTP exposes the SD card below a
prefix rather than `/`, add `--root /that/prefix`. Run `python3
tools/upload_switch.py --help` for port, login and password options.

## Diagnostic reference

### Training Modpack compatibility

Use the unmodified Training Modpack. In diagnostic Training, **click R3** to
open/close Kirby Assist's menu and use the D-pad to edit it. L/Y/X and all
configured gameplay features work normally. Real Minus/Plus and Training
menu/recording shortcuts hand raw input to Training; default ZL save/load keeps
our panel visible and resumes automatically on release. After finishing/closing
its menu or recording, tap R3 to resume with our menu visible. Leave R3
unassigned in Training. Silent builds and other modes use R3 for our menu.
Actual controller acceptance still needs Switch testing; see the compatibility
guide above. No extra Training NRO or rebuild is needed.

### Logs

The diagnostic build writes:

```text
SD:/ultimate/kirby_assist/status.log
```

Its first line identifies the diagnostic version. It also provides the frozen
ledge-calibration result card. Read [the calibration guide](docs/CALIBRATION_CARD.md)
before changing timing or spacing.

The silent build does not create or update this plug-in log. An older log can
remain on the SD card after switching to silent, so check its timestamp and first
line before treating it as current evidence.

## Release checklist

1. Update `[package].version` in `Cargo.toml` only when creating a new release.
2. Append the completed-change summary to `[package].description`; preserve all
   earlier summaries.
3. Update the relevant maintained guides and fighter pages.
4. Run the fighter-page check, Python tooling tests, core tests and strict MkDocs
   build appropriate to the change.
5. Build the required variant or both variants with `tools/build_variants.py`.
6. Inspect `releases/` and `../Latestbuild-<version>/`.
7. Run the uploader with `--dry-run`.
8. Close Smash, upload the intended variant, restart Smash and perform the
   on-device test.
9. Use `--verify-only` and preserve the diagnostic log when investigating a
   failure.

## Common paths

| Purpose | Path |
| --- | --- |
| Shared startup settings | `deployment/ultimate/kirby_assist/config.txt` |
| Installed settings | `SD:/ultimate/kirby_assist/config.txt` |
| Installed plug-in | `SD:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro` |
| Diagnostic log | `SD:/ultimate/kirby_assist/status.log` |
| Uploader backups | `SD:/ultimate/kirby_assist/bak/` |
| Maintained guides | `docs/` |
| Fighter pages | `docs/fighters/` |
| Packaged releases | `releases/` |
| Copyable latest builds | `../Latestbuild-<version>/` |

The [documentation homepage](docs/index.md) links every maintained guide,
including the [Rust beginner guide](docs/README_BEGINNER.md),
[fighter execution guide](docs/FIGHTER_EXECUTION.md),
[automatic tech](docs/AUTO_TECH.md), [analog mash](docs/ANALOG_MASH.md),
[ledge controls](docs/LEDGE_TRAP.md) and [build variants](docs/BUILD_VARIANTS.md).

Current input work: R3 alone toggles the top-left single-line menu. Kirby has
optional Idle crouch / Dtilt face rows; Yoshi has onstage roll-smash/grab. See
[shared input recipes](docs/INPUT_RECIPES.md) and
[Training/monitoring setup](docs/TRAINING_MONITOR.md). New sources require a later
build/upload and on-Switch validation.

Yoshi now has an optional **Jab lock** menu row (default OFF): chase confirmed
down-tilt hits, attempt grabs on techs or jab locks on missed techs, then down
smash on a confirmed lock. It takes priority over background Roll read.
[Calibration and limits](docs/JAB_LOCK.md).
