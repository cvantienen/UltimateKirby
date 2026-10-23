# Kirby Assist

Kirby Assist supplies raw stick/button input for the selected local human
controller before Smash applies the player's controller mapping. It includes
ledge responses, automatic mash/tech/SDI and manual controller shortcuts.

Start with [current controls](CONTROLS.md), [fighters](FIGHTERS.md), or
[installation and configuration](INTEGRATION.md).

## Current source and installed builds

The manifest currently identifies **1.5.3**. These maintained docs describe the
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
| L+R3 | Show/hide the five-row menu |
| D-pad, menu visible | Up/Down selects a row; Left/Right cycles its response |
| D-pad, menu hidden | Left/Right slides; Down ledge trump; Up normal |
| Minus | Toggle auto-mash |
| Hold R3 alone | Turbo teabag |
| X | Paired X+ZL short hop |
| Y | B-reverse opposite fighter facing |
| Hold L3 as Kirby | Down-tilt recovery cover |

Required mapping, menu behavior and automatic tools are documented in
[the full control reference](CONTROLS.md). Combo playback and Y-mode cycling
are removed. Moves and menu choices come from compiled fighter profiles;
only shared settings and supported numeric adjustments load from `config.txt`.

## Fighters and ledge responses

[Fighters](FIGHTERS.md) collects the individual guides and full non-DLC roster.
Each fighter has saved choices for Neutral, Jump, Roll, Attack and Drop. Active
recipes use fighter-specific frame data and native action acceptance. Reach,
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

Read [the source map](CODE_MAP.md), [compiled fighter profiles](FIGHTER_CONFIGS.md),
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
