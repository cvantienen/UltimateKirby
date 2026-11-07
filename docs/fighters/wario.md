# Wario

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **33** (`wario`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Dtilt; **Ftilt**; Fsmash |
| Jump | **Nair**; Fair; Uair; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield Up-B**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair; Low smash (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `wario.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `wario.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `wario.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `wario.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `wario.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `wario.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `wario.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral Air | 4/20 start; active 4–12 / 13–17 / 20–26 / 27–42; 45 total; 7 landing |
| Forward Air | 5 start; active 5–6 / 7–17; 37 total; 10 landing |
| Back Air | 9 start; active 9–11; 49 total; 16 landing |
| Up Air | 8 start; active 8–9; 46 total; 7 landing |
| Down Air | 9/11/13/15/17/19/21 start; active 9 / 11 / 13 / 15 / 17 / 19 / 21; 47 total; 18 landing |
| Forward Tilt | 12 start; active 12–15; 39 total; -- landing |
| Up Tilt | 8 start; active 8–10 / 11–15; 27 total; -- landing |
| Down Tilt | 5 start; active 5; 17 total; -- landing |
| Dash Attack | 5 start; active 5–8 / 9–23; 48 total; -- landing |
| Forward Smash | 18 start; active 18–19; 65 total; -- landing |
| Up Smash | 11 start; active 11–12 / 13; 57 total; -- landing |
| Down Smash | 8 start; active 8–17 / 18–27 / 28–36; 61 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 8–10 |
| Dash grab | frames 11–13 |
| Pivot grab | frames 12–14 |
| Standing reach reference | 14.0 world units |
| Shield-grab startup | 12 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 107 |
| Walk speed | 0.889 |
| Initial dash | 1.837 |
| Run speed | 1.65 |
| Air speed | 1.271 |
| Gravity | 0.107 |
| Fall speed | 1.61 |
| Fast fall | 2.576 |
| Short-hop height | 14.7 |
| Full-hop height | 30.5 |

## Execution and calibration

Option definitions and input delays come from `fighters/wario.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 20 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 29 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 20 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:19, release:1` |
| Jump / `nair` | 1 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 3 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 3 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:11, release:1` |
| Roll / `grab` | 23 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 26 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
| Drop / `down_smash` | 0 | `face:opponent, down_smash:1` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
