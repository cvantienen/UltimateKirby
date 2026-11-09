# Mega Man

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **49** (`mega_man`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Dtilt**; Utilt; Arm cannon (travel) |
| Jump | **Fair**; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Up-smash**; Shield > Fair |
| Drop | **Shield**; Dtilt > Shield; Near Fair |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Forward Air | `mega_man.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `mega_man.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up Smash | `mega_man.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up Smash | `mega_man.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up Smash | `mega_man.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `mega_man.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up Smash | `mega_man.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Forward Air | 9 start; active 9–11 / 12–17; 40 total; 11 landing |
| Back Air | 4/7/10 start; active 4–5 / 7–8 / 10–11; 44 total; 20 landing |
| Down Air | 23 start; active 23–25 / 26–35; 64 total; 14 landing |
| Forward Tilt | 7/19/31 start; active 7–8 / 9–12 / 13–36 / 19–20 / 11–24 / 25–42 / 31–32 / 33–36 / 37–54; 33/45/57 total; -- landing |
| Up Tilt | 6 start; active 6 / 7–9 / 10–16; 70/52/54 total; -- landing |
| Down Tilt | 5 start; active 5–8 / 9–21; 46 total; -- landing |
| Dash Attack | 7/10/13/16/19/22/25/35 start; active 7 / 10 / 13 / 16 / 19 / 22 / 25 / 35; 59 total; -- landing |
| Forward Smash | 19 start; active 19–42 / 79–108 / 199–222; 54 total; -- landing |
| Up Smash | 8/11/15/19/23/27/31 start; active 8 / 11–12 / 15–16 / 19–20 / 23–24 / 27–28 / 31; 68 total; -- landing |
| Down Smash | 17 start; active 17 / 18–19 / 20–33; 78 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 12.5 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 102 |
| Walk speed | 1.132 |
| Initial dash | 1.958 |
| Run speed | 1.602 |
| Air speed | 1.155 |
| Gravity | 0.107 |
| Fall speed | 1.8 |
| Fast fall | 2.88 |
| Short-hop height | 11.49 |
| Full-hop height | 32.8 |

## Execution and calibration

Option definitions and input delays come from `fighters/mega_man.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `down_tilt` | 21 | `opponent_input:a:0:-0.6:1` |
| Neutral / `up_tilt` | 23 | `opponent_input:a:0:0.6:1` |
| Neutral / `area_smash` | 0 | `opponent_input:a:0:-1:13, release:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_tilt` | 3 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:10, release:1` |
| Roll / `grab` | 25 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 18 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, opponent_input:a:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |

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
