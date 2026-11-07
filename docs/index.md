# Kirby Assist

Kirby Assist supplies raw stick/button input for the selected local human
controller before Smash applies the player's controller mapping. It includes
ledge responses, automatic mash/tech/SDI and manual controller shortcuts.

Start with [current controls](CONTROLS.md), [fighters](FIGHTERS.md), or
[installation and configuration](INTEGRATION.md).

## Current source and installed builds

The release version is recorded in `Cargo.toml`. These maintained docs describe the
current working source, including later Kirby/Yoshi nair and roll-grab edits.
An existing 1.5.3 NRO or source ZIP may predate those edits: editing source or docs
does not update an installed plugin. Check the built package and diagnostic
startup header when reporting behavior.

Supplied Switch logs confirm mapped Grab inputs, native grab startup and some
Kirby/Yoshi captures in the tested version. They also exposed delayed approaches
and a mismatched common roll estimate. The latest source changes address those
paths, but have not been compiled or tested on Switch here. Formatting and a
successful documentation build do not establish gameplay or online accuracy.

## Controls at a glance

| Input | Action |
| --- | --- |
| Hold L | Ledge autopilot |
| Minus | Cycle compact selector → modes → closed |
| Hold L3 without L | Turbo teabag; release to stop |
| R3, compact selector | Edit the highlighted recipe; R3 or Minus returns to compact |
| D-pad, compact/modes | Up/Down selects a row; Left/Right changes its value |
| D-pad, recipe editor | Up/Down selects a field; Left/Right changes its value |
| D-pad, menu hidden | Fighter-specific input sequence; Normal button passes through |
| Auto-mash row, Left/Right | Toggle mash ON/OFF |
| X | Existing short-hop shortcut |
| Press Y | Normal mapped game input |
| Hold L | Saved ledge responses and configured walking/ledge-distance limits |

Required mapping, menu behavior and automatic tools are documented in
[the full control reference](CONTROLS.md). The older combo engine and B-reverse
binding remain removed. The old Y Finisher binding remains removed. X/Y/D-pad input sequences are fighter-specific; modes are ON/OFF menu settings only; Normal button restores X short hop or ordinary Y. Recipe menus are battle-only. The A+B jab-to-grab shortcut is removed. Ledge moves, menu choices, defaults and input delays come only from `sequences.txt`. Existing
ledge-choice recipes may be overridden through startup-only
[`sequences.txt`](SEQUENCE_FILES.md); shared startup settings live at the top of that master file in `[universal.settings]`. `config.txt` is only a fallback for older files without that section.

## Fighters and ledge responses

[Fighters](FIGHTERS.md) collects the individual guides and full non-DLC roster.
Each fighter has saved choices for Neutral, Jump, Roll, Attack and Drop. Recipes deliver the configured controller inputs; the game determines native action acceptance. Reach,
root travel and online input arrival remain estimates; a correctly requested
move does not guarantee a hit.

[Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md) documents the brief nair nudges, earlier
scheduling, Kirby bair and the latest walking-grab changes.

## Builds and diagnostics

Silent and diagnostic are optimized variants with the same gameplay policy.
Silent excludes this plugin's explicit logger; diagnostic records startup,
input/action evidence and bounded roll-grab traces. See
[build variants](BUILD_VARIANTS.md) for packaging and the separate uploader.

Diagnostic creates `SD:/ultimate/kirby_assist/status.log` at startup, before a
match. The uploader defaults to silent unless `-d` is supplied. Restart Smash
after replacing the NRO. Missing files and stale artifacts are covered in
[diagnostic setup](INTEGRATION.md#read-the-status-log-before-changing-code).

## Understand or extend the code

Read [the source map](CODE_MAP.md), [file-authored fighter options](FIGHTER_CONFIGS.md),
[startup sequence files](SEQUENCE_FILES.md),
[execution and calibration](FIGHTER_EXECUTION.md), and the
[Rust beginner guide](README_BEGINNER.md). The [reliability notes](RUST_RELIABILITY_REVIEW.md)
distinguish mapped requests, native acceptance and measured outcomes.

## Build and check

From the repository root with the Skyline toolchain configured:

```sh
# Package optimized silent and diagnostic variants; does not upload.
python3 tools/build_variants.py

# Portable Rust logic tests; compiling tests is separate from a docs build.
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml

# Validate and generate documentation only.
.venv-docs/bin/python -m mkdocs build --strict
```

Run mod builds or uploads only when intended. See [maintaining the docs](MAINTAINING.md)
for Python setup and previewing, and [TODO](TODO.md) for the work template.
