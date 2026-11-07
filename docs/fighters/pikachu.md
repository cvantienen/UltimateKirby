# Pikachu

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **8** (`pikachu`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Dtilt; **Forward Smash** |
| Jump | **Up Air** |
| Roll | **Dsmash** |
| Attack | **Shield > Nair**; Shield > range-checked grab |
| Drop | **Shield**; Dtilt > Shield |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `pikachu.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `pikachu.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `pikachu.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `pikachu.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `pikachu.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `pikachu.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `pikachu.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral Air | 3/9/15/21 start; active 3–6 / 9–12 / 15–18 / 21–22; 38 total; 9 landing |
| Back Air | 4/8/12/16/20/24 start; active 4–5 / 8–9 / 12–13 / 16–17 / 20–21 / 24–25 / 1–2; 43 total; 18 landing |
| Up Air | 4 start; active 4–6 / 7–8; 26 total; 14 landing |
| Down Air | 14 start; active 14–15 / 16–26 / 1–2; 47 total; 22 landing |
| Forward Tilt | 6 start; active 6–8; 29 total; -- landing |
| Up Tilt | 7 start; active 7–13; 26 total; -- landing |
| Down Tilt | 7 start; active 7–8; 18 total; -- landing |
| Dash Attack | 6 start; active 6–8 / 9–12; 35 total; -- landing |
| Forward Smash | 15 start; active 15–16 / 17–19 / 20–29; 53 total; -- landing |
| Up Smash | 10 start; active 10–12 / 13–14 / 15–17; 44 total; -- landing |
| Down Smash | 8/11/14/17/20/23 start; active 8–9 / 11–12 / 14–15 / 17–18 / 20–21; 65 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 11–12 |
| Pivot grab | frames 12–13 |
| Standing reach reference | 11.9 world units |
| Shield-grab startup | 11 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 79 |
| Walk speed | 1.302 |
| Initial dash | 1.98 |
| Run speed | 2.039 |
| Air speed | 0.957 |
| Gravity | 0.095 |
| Fall speed | 1.55 |
| Fast fall | 2.48 |
| Short-hop height | 17.12 |
| Full-hop height | 35.5 |

## Execution and calibration

Option definitions and input delays come from `fighters/pikachu.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 26 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 26 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:21, release:1` |
| Jump / `up_air` | 5 | `opponent_input:a+x:0:1:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:25, release:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
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

- Registration: `crates/kirby_assist_core/src/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
