# Link

## Configure Link

This page belongs to profile **`link`** (`fighter_kinds::LINK`), not a
roster-position index. Use `[link.settings]` for overrides and
`[link.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

This identity uses the shared file-authored runner, not a dedicated compiled move routine. No standalone Lucina counter, Game & Watch reflector or Kirby-specific follow-up is inherited. Universal held tools have their own eligibility gates; registering a fighter does not bypass them.

To expand this fighter, add a uniquely named option under **`link`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **2** (`link`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Down tilt; Forward tilt |
| Jump | **Nair** |
| Roll | **Down smash** |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield > Up-B** |
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
| `auto_air_dodge` | — | Air dodge > Up B | `link.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `link.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `link.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `link.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `link.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `link.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 7 start; active 7–8; 23 total; -- landing |
| Neutral Air | 7 start; active 7–8 / 9–31; 38 total; 6 landing |
| Forward Air | 16/24 start; active 16–17 / 24–25; 51 total; 11 landing |
| Back Air | 6/15 start; active 6–8 / 15–17; 30 total; 6 landing |
| Up Air | 11 start; active 11–13 / 14–40; 59 total; 14 landing |
| Down Air | 14 start; active 14–19 / 20–64; 79 total; 19 landing |
| Forward Tilt | 15 start; active 15 / 16–19; 39 total; -- landing |
| Up Tilt | 8 start; active 8–12; 35 total; -- landing |
| Down Tilt | 10 start; active 10–11; 30 total; -- landing |
| Dash Attack | 20 start; active 20–23; 56 total; -- landing |
| Up Smash | 10/25/41 start; active 10–13 / 14–15 / 25–28 / 29–30 / 41–45; 77 total; -- landing |
| Down Smash | 12/24 start; active 12–13 / 24–25; 56 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 11.0 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 104 |
| Walk speed | 1.247 |
| Initial dash | 1.98 |
| Run speed | 1.534 |
| Air speed | 0.924 |
| Gravity | 0.096 |
| Fall speed | 1.6 |
| Fast fall | 3.04 |
| Short-hop height | 13.38 |
| Full-hop height | 27.8 |

## Execution and calibration

Option definitions and input delays come from `fighters/link.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 23 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 17 | `opponent_input:a:0.6:0:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:23, release:1, else_facing, opponent_input:a:0:-1:11, release:1, end_facing` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |

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
