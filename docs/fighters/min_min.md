# Min Min

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **87** (`min_min`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Down tilt; **Forward tilt**; Up tilt |
| Jump | Nair; **Up air**; Grounded up tilt |
| Roll | **Grab**; Down tilt |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield > Uair** |
| Drop | **Shield**; Down tilt > shield; Hop nair (near); Low smash (spacing) |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `min_min.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up Air | `min_min.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up Air | `min_min.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up Air | `min_min.roll.auto_shield_backup` |
| `auto_spot_dodge` | — | Spot dodge > Up Air | `min_min.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 5 start; active 5–6; 24 total; -- landing |
| Up Air | 7 start; active 7–13; 38 total; 8 landing |
| Down Air | 15 start; active 15–18 / 19–33 / 1–3; 54 total; 25 landing |
| Up Tilt | 5 start; active 5–6 / 7–11 / 6–9 / 10–15; 41 total; -- landing |
| Down Tilt | 10 start; active 10–14 / 15–19; 34 total; -- landing |
| Dash Attack | 7 start; active 7–10 / 11–16; 38 total; -- landing |
| Up Smash | 8 start; active 8–10 / 11–13 / 14–16; 53 total; -- landing |
| Down Smash | 6 start; active 6–8; 48 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 18–24 |
| Dash grab | frames 18–24 |
| Pivot grab | frames 18–24 |
| Standing reach reference | 40.0 ~ world units |
| Shield-grab startup | 22 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 104 |
| Walk speed | 1.05 |
| Initial dash | 1.9 |
| Run speed | 1.55 |
| Air speed | 0.85 |
| Gravity | 0.11 |
| Fall speed | 1.52 |
| Fast fall | 2.432 |
| Short-hop height | 16 |
| Full-hop height | 30.7 |

## Execution and calibration

Option definitions and input delays come from `fighters/min_min.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 21 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 23 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 19 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 28 | `opponent_input:a:0:0.6:1` |
| Jump / `nair` | 1 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 4 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 8 | `opponent_input:a:0:0.6:1` |
| Roll / `grab` | 13 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 21 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
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
