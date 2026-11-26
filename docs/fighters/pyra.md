# Pyra

## Configure Pyra

This page belongs to profile **`pyra`** (`fighter_kinds::PYRA`), not a
roster-position index. Use `[pyra.settings]` for overrides and
`[pyra.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

Pyra and Mythra are separate exact-kind profiles. The supplied five ledge rows contain Custom starters with wait:0, not attacks. Save before swapping forms; a profile swap clears outgoing selections/tapes and resolves the incoming identity rather than borrowing another fighter's recipe.

To expand this fighter, add a uniquely named option under **`pyra`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **90** (`pyra`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Custom (edit in Training)** |
| Jump | **Custom (edit in Training)** |
| Roll | **Custom (edit in Training)** |
| Attack | **Custom (edit in Training)** |
| Drop | **Custom (edit in Training)** |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| — | — | No saved assist rules in the supplied master | — |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3–5; 21 total; -- landing |
| Neutral Air | 11 start; active 11–16 / 17–22; 56 total; 15 landing |
| Forward Air | 11 start; active 11–14; 48 total; 14 landing |
| Back Air | 16 start; active 16–17 / 18; 49 total; 14 landing |
| Up Air | 13 start; active 13–17; 57 total; 8 landing |
| Down Air | 17 start; active 17 / 18 / 19–20; 65 total; 14 landing |
| Forward Tilt | 12 start; active 12–13; 37 total; -- landing |
| Up Tilt | 11 start; active 11–16; 39 total; -- landing |
| Down Tilt | 9 start; active 9–11; 26 total; -- landing |
| Dash Attack | 17 start; active 17–18; 52 total; -- landing |
| Forward Smash | 20 start; active 20–23; 67 total; -- landing |
| Up Smash | 15/17 start; active 15–16 / 17–23 / 24–33; 66 total; -- landing |
| Down Smash | 12/18 start; active 12–14 / 18–20; 47 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 12–13 |
| Pivot grab | frames 13–14 |
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
| Weight | 98 |
| Walk speed | 0.92 |
| Initial dash | 1.69 |
| Run speed | 1.38 |
| Air speed | 1.1 |
| Gravity | 0.078 |
| Fall speed | 1.62 |
| Fast fall | 2.592 |
| Short-hop height | Unknown |
| Full-hop height | Unknown |

## Execution and calibration

Option definitions and input delays come from `fighters/pyra.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `custom` | 0 | `wait:0` |
| Jump / `custom` | 0 | `wait:0` |
| Roll / `custom` | 0 | `wait:0` |
| Attack / `custom` | 0 | `wait:0` |
| Drop / `custom` | 0 | `wait:0` |

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
