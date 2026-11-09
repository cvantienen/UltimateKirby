# Lucas

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **40** (`lucas`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; Utilt; Stick reflector |
| Jump | Nair; Fair; **Uair**; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `lucas.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `lucas.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `lucas.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `lucas.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `lucas.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `lucas.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `lucas.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 2 start; active 2; 19 total; -- landing |
| Forward Air | 9 start; active 9–10 / 11–12; 41 total; 7 landing |
| Back Air | 15 start; active 15–19 / 19–22; 39 total; 9 landing |
| Up Air | 7 start; active 7–10; 31 total; 7 landing |
| Down Air | 10/18/26/34 start; active 10–12 / 18–20 / 26–28 / 34–35; 56 total; 16 landing |
| Forward Tilt | 7 start; active 7–9; 25 total; -- landing |
| Up Tilt | 4/7 start; active 4 / 7–10 / 11–14 / 15–16; 33 total; -- landing |
| Down Tilt | 3 start; active 3–4; 15 total; -- landing |
| Dash Attack | 13 start; active 13–17; 37 total; -- landing |
| Forward Smash | 14 start; active 14–15; 45 total; -- landing |
| Up Smash | 28/30 start; active 28 / 30–32 / 33–37 / 38–42 / 43–47 / 48–52 / 53–54; 98 total; -- landing |
| Down Smash | 20/29/39 start; active 20–22 / 29–31 / 39–41; 59 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 12–17 |
| Dash grab | frames 14–19 |
| Pivot grab | frames 15–20 |
| Standing reach reference | 22.9 world units |
| Shield-grab startup | 16 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 94 |
| Walk speed | 0.893 |
| Initial dash | 1.815 |
| Run speed | 1.65 |
| Air speed | 1.155 |
| Gravity | 0.09 |
| Fall speed | 1.37 |
| Fast fall | 2.192 |
| Short-hop height | 13 |
| Full-hop height | 29.41 |

## Execution and calibration

Option definitions and input delays come from `fighters/lucas.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 24 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 30 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 26 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 22 | `opponent_input:a:0:0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:26, release:1` |
| Jump / `nair` | 4 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 1 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 3 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 3 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:15, release:1` |
| Roll / `grab` | 19 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 28 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
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
