# Little Mac

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **52** (`littlemac`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Utilt; **Dtilt**; Ftilt; Armored f-smash; Low f-smash (precise); Tiny nair (joke) |
| Jump | **Ground Utilt**; Tiny Uair (joke); Tiny nair (joke) |
| Roll | **Dsmash**; Grab |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield Up-B** |
| Drop | **Shield**; Dtilt > Shield; Low sweep (spacing); F-tilt poke |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `littlemac.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `littlemac.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `littlemac.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `littlemac.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `littlemac.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `littlemac.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `littlemac.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 1 start; active 1; 16 total; -- landing |
| Neutral Air | 2 start; active 2; 15 total; 10 landing |
| Forward Air | 10 start; active 10–12; 36 total; 13 landing |
| Back Air | 11 start; active 11–12; 37 total; 16 landing |
| Up Air | 5 start; active 5–8; 41 total; 13 landing |
| Down Air | 7 start; active 7–10; 27 total; 18 landing |
| Forward Tilt | 4/12 start; active 4–5 / 12–13; 37 total; -- landing |
| Up Tilt | 4 start; active 4–10; 29 total; -- landing |
| Down Tilt | 3 start; active 3–4; 25 total; -- landing |
| Dash Attack | 7 start; active 7–9; 33 total; -- landing |
| Forward Smash | 14/14/15 start; active 14–15 / 14–15 / 15; 43/43/43 total; -- landing |
| Up Smash | 10 start; active 10 / 11–14; 47 total; -- landing |
| Down Smash | 10/17 start; active 10–11 / 17–18; 42 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 9–10 |
| Dash grab | frames 13–14 |
| Pivot grab | frames 14–15 |
| Standing reach reference | 12.9 world units |
| Shield-grab startup | 13 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 87 |
| Walk speed | 1.386 |
| Initial dash | 2.365 |
| Run speed | 2.464 |
| Air speed | 1.208 |
| Gravity | 0.09 |
| Fall speed | 1.95 |
| Fast fall | 3.12 |
| Short-hop height | 12.53 |
| Full-hop height | 26 |

## Execution and calibration

Option definitions and input delays come from `fighters/littlemac.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `up_tilt` | 27 | `opponent_input:a:0:0.6:1` |
| Neutral / `down_tilt` | 30 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 29 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Neutral / `low_forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Neutral / `nair` | 29 | `opponent_input:a+x:0:0:1` |
| Jump / `up_tilt` | 7 | `opponent_input:a:0:0.6:1` |
| Jump / `up_air` | 5 | `opponent_input:a+x:0:1:1` |
| Jump / `nair` | 9 | `opponent_input:a+x:0:0:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:26, release:1, else_facing, opponent_input:a:0:-1:19, release:1, end_facing` |
| Roll / `grab` | 22 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `sweep` | 0 | `face:opponent, down_smash:1` |
| Drop / `poke` | 0 | `face:opponent, forward_tilt:1` |

See [this fighter's detailed execution notes](../PACMAN_MAC_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

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

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


## Grounded coverage

Jump defaults to grounded up tilt, not an adaptive aerial. The tiny aerial
choices are explicit alternatives and should not be read as strong coverage.
Shield-to-Up-B is the default Attack response and waits for real block evidence.
KO Punch, charge/counter management and automatic offstage recovery are absent.
