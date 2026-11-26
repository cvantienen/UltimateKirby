# Bowser

## Configure Bowser

This page belongs to profile **`bowser`** (`fighter_kinds::BOWSER`), not a
roster-position index. Use `[bowser.settings]` for overrides and
`[bowser.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
Existing choices, supplied defaults and literal inputs are shown in the generated
snapshot below. Your installed file and unsaved Training draft may differ.

Start with [the complete fighter configuration walkthrough](../FIGHTER_CONFIGS.md),
then use [all 83 tuning keys](../FIGHTER_SETTINGS.md) and
[execution/calibration](../FIGHTER_EXECUTION.md). These shared guides explain
units, ranges, current versus startup effects, save/restart, and cancellation.
Choose or edit this fighter's row in Training; activate its configured Ledge
Assist binding to run it. A missing selected timeline produces no fallback move.

### Supplied fighter overrides

These are the current repository file values, **not schema fallbacks**.
Omitted keys retain resolved shared/reference defaults; retired fields remain
parseable but do not restore removed behavior.

| Key | Supplied value |
| --- | --- |
| `walk_strength` | `0.49999985` |
| `grab_range` | `22` |
| `vertical_range` | `20` |
| `follow_range` | `45` |
| `follow_vertical_range` | `45` |
| `stand_distance` | `15` |
| `manual_spacing` | `true` |
| `auto_position` | `true` |
| `grab_dodge_default` | `true` |
| `out_of_shield_default` | `true` |
| `grab_dodge_lookahead` | `8` |
| `shield_body_half_width` | `10` |
| `shield_body_height` | `20` |
| `shield_body_half_depth` | `10` |
| `shield_forecast` | `10` |
| `shield_hold` | `2` |
| `grab_dodge_padding` | `15` |
| `grab_dodge_height` | `10` |
| `tech_ceiling_origin` | `5` |
| `tech_floor_padding` | `2` |

### Support and extension boundary

Bowser has explicit supplied held/button bindings and six reaction event records; he does not receive a hardcoded spinning drift or automatic unshielded grab. Ranges and follow-through come from each record. Native capture returns control for manual throws; no label supplies a combo.

To expand this fighter, add a uniquely named option under **`bowser`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **15** (`bowser`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | FIRE BREATH; DWN B; UP B; **F SMASH** |
| Jump | BAIR; F TILT; **UP-TILT** |
| Roll | UP SMASH; GRAB; UP B; **F SMASH** |
| Attack | Shield DSMASH; **Grab**; Shield > Up-B; SIDE B |
| Drop | Shield; FADE AWAY NAIR; FADE AWAY FAIR; **DOWN SMASH**; Dair; DWN B |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge - Up-B chase | `bowser.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | After dodging enemy grab | `bowser.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll - Up-B chase | `bowser.roll.auto_roll` |
| `auto_shield_backup` | — | Shield grab backup | `bowser.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `bowser.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge - Up-B chase | `bowser.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 7 start; active 7–8; 25 total; -- landing |
| Neutral Air | 8/14/18/** start; active 8–29 / 14–29 / 18–29 / 18–29; 47 total; 15 landing |
| Forward Air | 11 start; active 11–14; 41 total; 14 landing |
| Back Air | 9 start; active 9–11; 44 total; 24 landing |
| Up Air | 9 start; active 9–13; 44 total; 17 landing |
| Down Air | 17 start; active 17–24 / 25–49 / 1–6; 77 total; 34 landing |
| Forward Tilt | 10 start; active 10–14; 37 total; -- landing |
| Up Tilt | 11 start; active 11–16; 40 total; -- landing |
| Down Tilt | 10/15 start; active 10–12 / 15–17; 45 total; -- landing |
| Dash Attack | 11 start; active 11–14 / 15–20; 55 total; -- landing |
| Forward Smash | 22 start; active 22–23 / 24–27; 69 total; -- landing |
| Up Smash | 16/37 start; active 16–23 / 37; 57 total; -- landing |
| Down Smash | 12/28 start; active 12–14 / 28–30; 72 total; -- landing |

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
| Weight | 135 |
| Walk speed | 0.901 |
| Initial dash | 2.255 |
| Run speed | 1.971 |
| Air speed | 1.155 |
| Gravity | 0.125 |
| Fall speed | 1.77 |
| Fast fall | 2.832 |
| Short-hop height | 15.7 |
| Full-hop height | 33 |

## Execution and calibration

Option definitions and input delays come from `fighters/bowser.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `down_tilt` | 0 | `face:opponent, facing_hold_until_manual:b:0:0` |
| Neutral / `forward_tilt` | 30 | `opponent_input:b:0:-1:4` |
| Neutral / `up_tilt` | 10 | `facing_input:b:0:1:4, drift:opponent:1:70` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:5` |
| Jump / `nair2` | 0 | `face:away_from_opponent, wait:1, facing_input:x:0:0:1, facing_input:a:-1:0:4` |
| Jump / `up_air` | 0 | `opponent_input:a:0.6:0:1` |
| Jump / `up_tilt` | 0 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `drift:away_from_opponent:1:1, facing_input:a:0:1:20` |
| Roll / `grab` | 0 | `walk:away_from_opponent:5:0.8, range:fighter_range, input:r:0:0:1` |
| Roll / `down_tilt` | 0 | `walk:away_from_opponent:5:0.8, facing_input:b:0:1:4, drift:opponent:1:70` |
| Roll / `menu_added_4` | 0 | `wait:1, opponent_input:none:1:0:4, opponent_input:a:-1:0:5` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a:0.6:-1:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, if_facing_opponent, range:25, facing_input:r:0:0:4, else_facing, opponent_input:b:1:0:4, end_facing` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, facing_input:b:0:1:1, drift:opponent:1:70` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:b:1:0:4` |
| Drop / `shield` | 0 | `facing_hold_until_manual:zr:0:0` |
| Drop / `drop_nair` | 0 | `facing_input:a+x:0:0:4, drift:away_from_opponent:0.4:20` |
| Drop / `forward_air` | 0 | `face:opponent, wait:1, facing_input:x:0:0:1, facing_input:a:1:0:1, drift:away_from_opponent:0.3:10` |
| Drop / `down_smash` | 0 | `opponent_input:a:0:-1:1` |
| Drop / `menu_added_6` | 0 | `facing_input:x:0:0:1, facing_input:a:0:-1:4` |
| Drop / `menu_added_7` | 0 | `facing_input:b:0:-1:4` |

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
