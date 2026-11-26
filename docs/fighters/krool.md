# King K. Rool

## Configure King K. Rool

This page belongs to profile **`krool`** (`fighter_kinds::KING_K_ROOL`), not a
roster-position index. Use `[krool.settings]` for overrides and
`[krool.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

Belly-flop and down-smash names do not extend native body/shockwave activity. Keep disconnected phases separate when calibrating. Drop movement can carry the fighter offstage; no label adds recovery automatically.

To expand this fighter, add a uniquely named option under **`krool`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **69** (`krool`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; CAN; **Forward tilt**; Belly flop |
| Jump | **Nair**; DASH; Forward air |
| Roll | **Down smash**; Grab; CANON; GRAB2 |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair |
| Drop | **Shield**; Down tilt > shield; Spike (risky); Belly flop (risky); BAIR |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `krool.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `krool.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `krool.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `krool.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `krool.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `krool.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 4 start; active 4–5; 24 total; -- landing |
| Neutral Air | 7 start; active 7–8 / 9–30; 45 total; 9 landing |
| Forward Air | 11 start; active 11 / 12–14 / 15–17; 54 total; 11 landing |
| Back Air | 18 start; active 18–20; 49 total; 14 landing |
| Up Air | 7 start; active 7–13 / 14–19; 71 total; 13 landing |
| Down Air | 14 start; active 14–15 / 16–21; 61 total; 14 landing |
| Forward Tilt | 12 start; active 12–13; 42 total; -- landing |
| Up Tilt | 5 start; active 5–6 / 7–13; 37 total; -- landing |
| Down Tilt | 13 start; active 13 / 14–15; 42 total; -- landing |
| Dash Attack | 7 start; active 7–14 / 15–25; 60 total; -- landing |
| Forward Smash | 19 start; active 19–21; 61 total; -- landing |
| Up Smash | 6/19/22 start; active 6–9 / 10–11 / 19 / 20 / 21–23 / 22–23; 71 total; -- landing |
| Down Smash | 22 start; active 22–24 / 25; 63 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 8–10 |
| Dash grab | frames 11–13 |
| Pivot grab | frames 12–14 |
| Standing reach reference | 20.0 world units |
| Shield-grab startup | 12 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 133 |
| Walk speed | 0.903 |
| Initial dash | 1.936 |
| Run speed | 1.485 |
| Air speed | 0.945 |
| Gravity | 0.105 |
| Fall speed | 1.7 |
| Fast fall | 2.72 |
| Short-hop height | 13.5 |
| Full-hop height | 33 |

## Execution and calibration

Option definitions and input delays come from `fighters/krool.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 20 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 21 | `opponent_input:a:0.6:0:1` |
| Neutral / `belly_flop` | 0 | `opponent_input:a:0:-1:11, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 0 | `opponent_input:a+x:0:1:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:9, release:1` |
| Roll / `grab` | 23 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `menu_added_3` | 0 | `face:away_from_opponent, facing_input:b:0:0:60` |
| Roll / `menu_added_4` | 0 | `walk:away_from_opponent:12:0.8, wait:1, range:fighter_range, facing_input:r:0:0:4` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |
| Drop / `belly_flop` | 0 | `face:opponent, down_smash:1` |
| Drop / `menu_added_6` | 0 | `face:away_from_opponent, wait:1, facing_input:x:0:0:1, facing_input:a:-1:0:2, wait:6, drift:away_from_opponent:1:8, wait:4, facing_input:none:0:-1:4` |

See [this fighter's detailed execution notes](../KROOL_ISABELLE_LEDGE.md) for configuration advice,
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
