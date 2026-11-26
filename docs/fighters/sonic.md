# Sonic

## Configure Sonic

This page belongs to profile **`sonic`** (`fighter_kinds::SONIC`), not a
roster-position index. Use `[sonic.settings]` for overrides and
`[sonic.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
Existing choices, supplied defaults and literal inputs are shown in the generated
snapshot below. Your installed file and unsaved Training draft may differ.

Start with [the complete fighter configuration walkthrough](../FIGHTER_CONFIGS.md),
then use [all 83 tuning keys](../FIGHTER_SETTINGS.md) and
[execution/calibration](../FIGHTER_EXECUTION.md). These shared guides explain
units, ranges, current versus startup effects, save/restart, and cancellation.
Choose or edit this fighter's row in Training; activate its configured Ledge
Assist binding to run it. A missing selected timeline produces no fallback move.

### Supplied fighter overrides

No `[fighter.settings]` overrides are supplied in this fighter file. Shared
settings and exact-kind reference facts remain the baseline; this does not mean
every assist is active or every recipe is populated.

### Support and extension boundary

Qualitative Nair hints are mid-range display guidance, not a collision test. Spring or charge-special names do not imply damaging coverage, charge management or automatic recovery. Inspect the actual actions before selecting an option.

To expand this fighter, add a uniquely named option under **`sonic`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **41** (`sonic`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Down tilt; **Forward tilt** |
| Jump | **Nair** |
| Roll | **Down smash**; Grab |
| Attack | **Shield > Nair**; Shield > range-checked grab |
| Drop | **Shield**; Down tilt > shield |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `sonic.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `sonic.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `sonic.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `sonic.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `sonic.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `sonic.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3; 19 total; -- landing |
| Neutral Air | 6 start; active 6–9 / 10–19 / 20–38; 49 total; 10 landing |
| Forward Air | 5/7/9/11/13/15 start; active 5 / 7 / 9 / 11 / 13 / 15; 45 total; 16 landing |
| Back Air | 13 start; active 13–14 / 15–19; 37 total; 15 landing |
| Up Air | 5/14 start; active 5–7 / 14–15; 39 total; 13 landing |
| Down Air | 17 start; active 17–19 / 20–32; 45 total; 21 landing |
| Forward Tilt | 6/8 start; active 6 / 8–11; 35 total; -- landing |
| Up Tilt | 6(7)/13 start; active 6–8 / 7–8 / 13–14; 35 total; -- landing |
| Down Tilt | 6 start; active 6–7; 27 total; -- landing |
| Dash Attack | 5 start; active 5–8 / 9–20; 49 total; -- landing |
| Forward Smash | 18 start; active 18–20; 47 total; -- landing |
| Up Smash | 14... start; active 14 / 17 / 19 / 21 / 23 / 25 / 27 / 29–30; 61 total; -- landing |
| Down Smash | 12 start; active 12–13; 54 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 10–11 |
| Pivot grab | frames 11–12 |
| Standing reach reference | 12.0 world units |
| Shield-grab startup | 11 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 86 |
| Walk speed | 1.444 |
| Initial dash | 2.31 |
| Run speed | 3.85 |
| Air speed | 1.208 |
| Gravity | 0.09 |
| Fall speed | 1.65 |
| Fast fall | 2.64 |
| Short-hop height | 16.89 |
| Full-hop height | 35 |

## Execution and calibration

Option definitions and input delays come from `fighters/sonic.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 9 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 27 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 24 | `opponent_input:a:0.6:0:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:24, release:1` |
| Roll / `grab` | 24 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |

See [this fighter's detailed execution notes](../SONIC_LEDGE.md) for configuration advice,
input distinctions and limitations.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighters/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/policies/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
