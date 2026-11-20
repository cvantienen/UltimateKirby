# Banjo And Kazooie

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **84** (`banjo_and_kazooie`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Down tilt; **Forward tilt**; Up tilt; Forward smash |
| Jump | Nair; Forward air; **Up air**; Grounded up tilt |
| Roll | **Down smash**; Grab; Down tilt |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield > Uair**; Shield > Fair |
| Drop | **Shield**; Down tilt > shield; Hop nair (near); Hop fair (near); Low smash (spacing) |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `banjo_and_kazooie.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `banjo_and_kazooie.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up Air | `banjo_and_kazooie.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up Air | `banjo_and_kazooie.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up Air | `banjo_and_kazooie.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `banjo_and_kazooie.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up Air | `banjo_and_kazooie.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 4 start; active 4–6; 27 total; -- landing |
| Neutral Air | 10/13/16/19/22/25/28/31 start; active 10–11 / 13–14 / 16–17 / 19–20 / 22–23 / 25–26 / 28–29 / 31–32; 47 total; 16 landing |
| Forward Air | 15 start; active 15–17 / 18; 51 total; 11 landing |
| Back Air | 8/12/16 start; active 8–9 / 12–13 / 16–17; 43 total; 18 landing |
| Up Air | 7/9 start; active 7–8 / 9–11; 33 total; 12 landing |
| Down Air | 15 start; active 15–17 / 18–45 / 1–2; 56 total; 27 landing |
| Forward Tilt | 7 start; active 7–9; 28 total; -- landing |
| Up Tilt | 11 start; active 11–14; 31 total; -- landing |
| Down Tilt | 12 start; active 12–17 / 18–21; 37 total; -- landing |
| Dash Attack | 9 start; active 9–12 / 13–20; 36 total; -- landing |
| Forward Smash | 19 start; active 19–21; 52 total; -- landing |
| Down Smash | 13 start; active 13–17; 46 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 10–11 |
| Pivot grab | frames 11–12 |
| Standing reach reference | 13.2 world units |
| Shield-grab startup | 11 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 106 |
| Walk speed | 1.06 |
| Initial dash | 1.68 |
| Run speed | 2.18 |
| Air speed | 0.92 |
| Gravity | 0.11 |
| Fall speed | 1.76 |
| Fast fall | 2.816 |
| Short-hop height | ** |
| Full-hop height | ** |

## Execution and calibration

Option definitions and input delays come from `fighters/banjo_and_kazooie.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 21 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 21 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 26 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 22 | `opponent_input:a:0:0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Jump / `nair` | 1 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 4 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 2 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:20, release:1` |
| Roll / `grab` | 24 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 19 | `opponent_input:a:0:-0.6:1` |
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

- Registration: `crates/kirby_assist_core/src/fighters/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/policies/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
