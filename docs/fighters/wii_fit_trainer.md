# Wii Fit Trainer

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **50** (`wii_fit_trainer`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Dtilt; **Ftilt**; Utilt; Fsmash |
| Jump | Nair; Fair; **Uair**; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair; Near Dair; Low smash (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `wii_fit_trainer.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `wii_fit_trainer.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `wii_fit_trainer.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `wii_fit_trainer.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `wii_fit_trainer.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `wii_fit_trainer.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `wii_fit_trainer.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 4 start; active 4–5; 25 total; -- landing |
| Neutral Air | 9/12 start; active 9–11 / 12–13; 34 total; 8 landing |
| Forward Air | 9 start; active 9 / 10 / 11–15; 35 total; 7 landing |
| Back Air | 5 start; active 5 / 6–11; 48 total; 12 landing |
| Up Air | 6 start; active 6–10; 39 total; 9 landing |
| Down Air | 15 start; active 15–17 / 18–20; 39 total; 12 landing |
| Forward Tilt | 6 start; active 6–10; 32 total; -- landing |
| Up Tilt | 5 start; active 5–12; 29 total; -- landing |
| Down Tilt | 9 start; active 9–12; 38 total; -- landing |
| Dash Attack | 6 start; active 6–8 / 9–13; 38 total; -- landing |
| Forward Smash | 16 start; active 16–19; 56 total; -- landing |
| Up Smash | 11 start; active 11–12 / 13–19; 54 total; -- landing |
| Down Smash | 18 start; active 18–21; 46 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 11.5 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 96 |
| Walk speed | 1.224 |
| Initial dash | 2.024 |
| Run speed | 1.866 |
| Air speed | 1.019 |
| Gravity | 0.09 |
| Fall speed | 1.3 |
| Fast fall | 2.08 |
| Short-hop height | 17.18 |
| Full-hop height | 35.6 |

## Execution and calibration

Option definitions and input delays come from `fighters/wii_fit_trainer.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 20 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 23 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 26 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 25 | `opponent_input:a:0:0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:22, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 3 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 6 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:16, release:1` |
| Roll / `grab` | 25 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 21 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
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

- Registration: `crates/kirby_assist_core/src/fighters/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/policies/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
