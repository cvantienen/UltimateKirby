# Villager

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **48** (`villager`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Dtilt; **Ftilt**; Utilt; Bowling ball (travel) |
| Jump | **Nair**; Uair; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Slingshot (travel); Near Dair; Low smash (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `villager.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `villager.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `villager.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `villager.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `villager.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `villager.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `villager.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3–4; 21 total; -- landing |
| Neutral Air | 3 start; active 3–10 / 11–23; 35 total; 8 landing |
| Forward Air | 10 start; active 10–12 / 13–18 / 19–23; 39 total; 14 landing |
| Back Air | 13 start; active 13–15 / 16–21 / 22–26; 35 total; 14 landing |
| Up Air | 6 start; active 6–7 / 8–21; 42 total; 12 landing |
| Down Air | 11 start; active 11–12 / 13–26; 42 total; 12 landing |
| Forward Tilt | 8 start; active 8–11; 33 total; -- landing |
| Up Tilt | 7/22 start; active 7–18 / 22–25; 47 total; -- landing |
| Down Tilt | 9 start; active 9–11; 37 total; -- landing |
| Forward Smash | 25 start; active 25–30 / 31–144; 49 total; -- landing |
| Up Smash | 12... start; active 12–13 / 17–18 / 21–22 / 25–26 / 29–30 / 33–34; 53 total; -- landing |
| Down Smash | 9/29 start; active 9 / 10–11 / 29 / 30–31; 49 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 14–16 |
| Dash grab | frames 16–18 |
| Pivot grab | frames 17–19 |
| Standing reach reference | 19.0 world units |
| Shield-grab startup | 18 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 92 |
| Walk speed | 1.092 |
| Initial dash | 1.815 |
| Run speed | 1.397 |
| Air speed | 0.987 |
| Gravity | 0.078 |
| Fall speed | 1.32 |
| Fast fall | 2.112 |
| Short-hop height | 15.69 |
| Full-hop height | 32.5 |

## Execution and calibration

Option definitions and input delays come from `fighters/villager.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 18 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 24 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 24 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 21 | `opponent_input:a:0:0.6:1` |
| Neutral / `area_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:27, release:1, else_facing, opponent_input:a:0:-1:7, release:1, end_facing` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 0 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 2 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:24, release:1, else_facing, opponent_input:a:0:-1:4, release:1, end_facing` |
| Roll / `grab` | 17 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 21 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
| Drop / `projectile_fair` | 0 | `face:opponent, short_hop_fair:1` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |
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
