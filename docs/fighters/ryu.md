# Ryu

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **60** (`ryu`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Fsmash; **Down Tilt** |
| Jump | **Nair**; Fair; Uair |
| Roll | **Dsmash**; Grab |
| Attack | **Shield Up-B**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Near Nair; Near Fair; Near Dair |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `ryu.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `ryu.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `ryu.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `ryu.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `ryu.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `ryu.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `ryu.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral Air | 4 start; active 4–6 / 7–31; 35 total; 5 landing |
| Forward Air | 8 start; active 8–9 / 10–14; 36 total; 11 landing |
| Back Air | 8 start; active 8–9; 41 total; 10 landing |
| Up Air | 6/9 start; active 6–7 / 9–11; 35 total; 11 landing |
| Down Air | 8 start; active 8–12; 45 total; 15 landing |
| Dash Attack | 7 start; active 7–9 / 10–15; 40 total; -- landing |
| Forward Smash | 15 start; active 15–17; 45 total; -- landing |
| Up Smash | 9 start; active 9 / 10–12; 44 total; -- landing |
| Down Smash | 5 start; active 5–6; 41 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 11.6 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 103 |
| Walk speed | 0.75 |
| Initial dash | 1.76 |
| Run speed | 1.6 |
| Air speed | 1.12 |
| Gravity | 0.12 |
| Fall speed | 1.6 |
| Fast fall | 2.24 |
| Short-hop height | 15 |
| Full-hop height | 26 |

## Execution and calibration

Option definitions and input delays come from `fighters/ryu.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Neutral / `down_tilt` | 31 | `opponent_input:a:0:-0.6:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 1 | `opponent_input:a+x:0:1:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:28, release:1` |
| Roll / `grab` | 25 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/ryu.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
