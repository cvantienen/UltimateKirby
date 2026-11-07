# Kirby Assist

Rust Skyline controller-input assists for Super Smash Bros. Ultimate. The mod
reads game observations and requests simulated inputs for the selected local
human fighter; recipes do not alter native moves, hitboxes or physics.

## Documentation

```sh
.venv-docs/bin/python -m mkdocs serve
```

Open the URL printed by MkDocs. Validate links and navigation with:

```sh
.venv-docs/bin/python -m mkdocs build --strict
```

Start with [controls](docs/CONTROLS.md), [menus](docs/LEDGE_MENU.md),
[sequence syntax](docs/SEQUENCE_FILES.md), [assist rules](docs/REACTION_RULES.md)
and the [fighter overview](docs/FIGHTERS.md). The
[project backlog](docs/TODO.md) tracks remaining calibration and device testing.

## Configuration and controls

`deployment/ultimate/kirby_assist/sequences.txt` is the local master. On SD it is
`/ultimate/kirby_assist/sequences.txt`. Universal startup settings are at the top,
then fighter settings, ledge/button recipes and separate assist-rule groups.
`config.txt` is only a fallback for older files without `[universal.settings]`.
External file changes require restarting Smash; Training saves update the current
session through the verified single-worker save path.

| Control | Current behavior |
| --- | --- |
| Minus | Open/close the menu. |
| Hold L | Walking limits and ledge assist; toggling ledge mode ON keeps tracking active without L. |
| R3 outside Training | Switch lightweight ledge versus Buttons / Assists selectors. |
| R3 on a Training recipe | Edit its actions, delays and movement values. |
| L3/R3 on a sequence-driven assist in Training | Open that assist's separate priority rule list. |
| Y in an editor | Explicit save; wait for completion. |
| X/Y/D-pad with menu closed | Selected fighter-specific recipe or Normal button. |

Roll reactions, Spot punish, Out of shield, Grab dodge + punish and Ledge regrab punish have
separate ON/OFF controls and up to eight rules each. Assist rules never enter the
normal ledge/button selectors. Acquired ledge setups retain control through their
response, then other assists may react to the current situation. Native safety
and game action eligibility still apply.

## Build, validate and transfer

The interactive workbench keeps the original helpers available:

```sh
python3 tools/workbench.py
python3 tools/sequence_validation.py
```

Your normal build command remains:

```sh
python3 tools/build_variants.py --cargo-home /private/tmp/kirby-assist-build-cargo
```

Use the workbench or original upload helper to review/upload a built release.
Builds and uploads are separate operations; the source version comes from
`Cargo.toml`. See [build variants](docs/BUILD_VARIANTS.md) and
[workbench transfers/history](docs/WORKBENCH.md).

Release uploads stage recipes as `sequences.upload.txt` and preserve the active
SD configuration. To replace it deliberately, use the workbench's Transfer /
replace sequences operation, selecting the direction and a fighter or whole file.
Review the preview and confirmation. No FTP writing should occur while Smash can
save the same file.

## Source and documentation maintenance

Portable tests use the core crate; choose the relevant suite for a change:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline
python3 tools/update_fighter_docs.py
python3 tools/update_fighter_docs.py --check
.venv-docs/bin/python tools/update_cheatsheets.py
.venv-docs/bin/python tools/update_cheatsheets.py --check
```

[Maintaining the docs](docs/MAINTAINING.md) describes the generated pages.
Diagnostic logs and host tests help investigate behavior, but do not prove actual
Switch timing, online compatibility, crash resolution or system temperature.
