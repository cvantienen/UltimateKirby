# Peach

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **13** (`peach`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; Club f-smash |
| Jump | **Nair**; Uair; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
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
| `auto_air_dodge` | — | Air dodge > Neutral Air | `peach.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `peach.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `peach.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `peach.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `peach.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `peach.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `peach.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 2 start; active 2; 27 total; -- landing |
| Neutral Air | 5 start; active 5–8 / 9–19; 48 total; 7 landing |
| Forward Air | 16 start; active 16–20; 57 total; 13 landing |
| Back Air | 6 start; active 6–7; 53 total; 11 landing |
| Up Air | 10/15 start; active 10 / 11 / 12–13 / 15 / 16–19; 45 total; 7 landing |
| Down Air | 12/18/24/30 start; active 12–13 / 18–19 / 24–25 / 30–31; 38 total; 8 landing |
| Forward Tilt | 7 start; active 7 / 8–9 / 10–15; 36 total; -- landing |
| Up Tilt | 9 start; active 9–13; 37 total; -- landing |
| Down Tilt | 8 start; active 8–9; 23 total; -- landing |
| Dash Attack | 6/17 start; active 6–9 / 17–19; 37 total; -- landing |
| Forward Smash | 15 start; active 15–17 / 15–17 / 15–17; 45 total; -- landing |
| Up Smash | 14 start; active 14–16 / 17–20; 44 total; -- landing |
| Down Smash | 6/10/14/18/22/26/30 start; active 6–7 / 10–11 / 14–15 / 18–19 / 22–23 / 26–27 / 30–31; 54 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 12.0 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 89 |
| Walk speed | 0.924 |
| Initial dash | 1.826 |
| Run speed | 1.595 |
| Air speed | 1.029 |
| Gravity | 0.07 |
| Fall speed | 1.19 |
| Fast fall | 1.904 |
| Short-hop height | 14.5 |
| Full-hop height | 30.03 |

## Execution and calibration

Option definitions and input delays come from `fighters/peach.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 19 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 25 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 23 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 0 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 3 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:27, release:1` |
| Roll / `grab` | 25 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 23 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
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
