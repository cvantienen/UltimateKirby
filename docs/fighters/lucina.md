# Lucina

## Configure Lucina

This page belongs to profile **`lucina`** (`fighter_kinds::LUCINA`), not a
roster-position index. Use `[lucina.settings]` for overrides and
`[lucina.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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
| `walk_strength` | `0.48999986` |
| `grab_dodge_padding` | `10` |
| `tilt_turn_down` | `false` |
| `tilt_turn_forward` | `false` |
| `tilt_turn_up` | `false` |
| `counter_default` | `true` |
| `counter_startup` | `7` |
| `counter_margin` | `3` |
| `counter_half_width` | `9` |
| `counter_height` | `17` |
| `counter_half_depth` | `6` |
| `auto_spacing_minimum_distance` | `24` |
| `auto_spacing_ground_move` | `2` |
| `auto_spacing_ground_startup` | `10` |
| `auto_spacing_ground_active_frames` | `5` |
| `auto_spacing_ground_reach` | `28` |
| `auto_spacing_ground_height` | `10` |
| `auto_spacing_aerial_move` | `2` |
| `auto_spacing_aerial_startup` | `5` |
| `auto_spacing_aerial_active_frames` | `5` |
| `auto_spacing_aerial_reach` | `28` |
| `auto_spacing_aerial_height` | `20.5` |
| `auto_di_default` | `false` |
| `auto_sdi_default` | `false` |
| `auto_spacing_ground_speed` | `0.49` |
| `auto_spacing_maximum_drift` | `1` |
| `auto_spacing_minimum_drift` | `0.11` |
| `auto_spacing_ground_braking` | `false` |
| `auto_spacing_braking_distance` | `10` |
| `auto_defense_lead_frames` | `10` |

### Support and extension boundary

Standalone auto-counter and tilt-facing tuning rows are available for Lucina. The supplied file enables counter and disables all three tilt-facing flags; a live mode/binding still needs admission. Counter estimates never shorten native startup.

To expand this fighter, add a uniquely named option under **`lucina`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **22** (`lucina`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | SIDE B; B; Forward tilt; **F-tilt**; f-smash |
| Jump | N-air; **F-air**; F-TILT; F SMASH |
| Roll | **Down smash**; Grab; F SMASH |
| Attack | Shield > f Tilt; Shield > range-checked grab; **Shield > Up-B**; Shield > Uair; Shield > Fair |
| Drop | Shield; Down tilt > shield; Hop fair (near); **Hop dair (near)**; Low smash (spacing) |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `lucina.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `lucina.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `lucina.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `lucina.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `lucina.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `lucina.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 5 start; active 5–6; 25 total; -- landing |
| Neutral Air | 6/15 start; active 6–7 / 15–21; 49 total; 7 landing |
| Forward Air | 6 start; active 6–8; 37 total; 10 landing |
| Back Air | 7 start; active 7–11; 39 total; 10 landing |
| Up Air | 5 start; active 5–9; 45 total; 8 landing |
| Down Air | 9/11 start; active 9–13 / 11; 59 total; 14 landing |
| Forward Tilt | 8 start; active 8–11; 33 total; -- landing |
| Up Tilt | 6 start; active 6–8 / 9–12; 33 total; -- landing |
| Down Tilt | 7 start; active 7–8; 23 total; -- landing |
| Dash Attack | 13 start; active 13–16; 49 total; -- landing |
| Forward Smash | 10 start; active 10–13; 51 total; -- landing |
| Up Smash | 13 start; active 13–14 / 13–17; 58 total; -- landing |
| Down Smash | 6/21 start; active 6–7 / 21–23; 55 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 14.5 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 90 |
| Walk speed | 1.575 |
| Initial dash | 2.255 |
| Run speed | 1.964 |
| Air speed | 1.071 |
| Gravity | 0.075 |
| Fall speed | 1.58 |
| Fast fall | 2.528 |
| Short-hop height | 16.26 |
| Full-hop height | 33.66 |

## Execution and calibration

Option definitions and input delays come from `fighters/lucina.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 18 | `controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:0:-1:0:0:1:opponent:opponent` |
| Neutral / `down_tilt` | 0 | `opponent_input:b:0:0:26` |
| Neutral / `forward_tilt` | 0 | `wait:21, opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 20 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:20, wait:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:4` |
| Jump / `up_air` | 0 | `opponent_input:a:0.6:0:1` |
| Jump / `f_smash` | 0 | `opponent_input:a:1:0:1` |
| Roll / `down_smash` | 0 | `drift:away_from_opponent:0.5:4, facing_input:a:0:-1:20` |
| Roll / `grab` | 0 | `drift:away_from_opponent:0.5:8, range:fighter_range, input:r:0:0:1` |
| Roll / `f_smash` | 0 | `drift:away_from_opponent:1:15, controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:1:0:0:0:1:opponent:opponent, wait:10, controller_input:b:0:0:0:0:1:opponent:opponent` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, facing_input:x:0:0:1, tap:b:0:0:1:11:60` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, opponent_input:b:0.3:0.7:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:4` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:4` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `forward_air` | 0 | `opponent_input:a+x:1:0:4` |
| Drop / `short_hop_dair` | 0 | `facing_input:a+x:0:-1:4, drift:opponent:1:5` |
| Drop / `down_smash` | 0 | `face:opponent, down_smash:1` |

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
