# Ness

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **10** (`ness`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | SIDE B; Fair; NAIR; **Ftilt**; Utilt; Bat reflector |
| Jump | **Nair**; Fair; SPIKE; Ftilt; DWN B |
| Roll | Yo-yo sweep; Grab; **PK** |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Dtilt; Dair; Afair; PK; Yo-yo sweep |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `ness.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `ness.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `ness.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `ness.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `ness.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `ness.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `ness.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3; 19 total; -- landing |
| Neutral Air | 5 start; active 5–12 / 13–15; 35 total; 8 landing |
| Back Air | 10 start; active 10–11 / 12–18; 35 total; 10 landing |
| Down Air | 18 start; active 18–20 / 21–24; 52 total; 12 landing |
| Forward Tilt | 7 start; active 7–10; 34 total; -- landing |
| Up Tilt | 5 start; active 5–6 / 7–8; 28 total; -- landing |
| Down Tilt | 3 start; active 3–4; 11 total; -- landing |
| Dash Attack | 8/15/23 start; active 8 / 15–16 / 23–24; 39 total; -- landing |
| Forward Smash | 21 start; active 21–22; 54 total; -- landing |
| Up Smash | 10 start; active 10–32; 52 total; -- landing |
| Down Smash | 12/17/23/31 start; active 12–15 / 17–18 / 23–25 / 31–32; 45 total; -- landing |

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
| Weight | 94 |
| Walk speed | 0.907 |
| Initial dash | 1.826 |
| Run speed | 1.609 |
| Air speed | 1.007 |
| Gravity | 0.077 |
| Fall speed | 1.31 |
| Fast fall | 2.096 |
| Short-hop height | 16.65 |
| Full-hop height | 34.48 |

## Execution and calibration

Option definitions and input delays come from `fighters/ness.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `menu_custom` | 5 | `face:opponent, wait:2, facing_input:x:0:0:1, facing_input:b:1:0:1` |
| Neutral / `nair` | 21 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 30 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 25 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 27 | `opponent_input:a:0:0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:22, release:1` |
| Jump / `nair` | 1 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 3 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 3 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 8 | `opponent_input:a:0:0.6:1` |
| Jump / `menu_added_5` | 0 | `facing_input:x:0:0:1, drift:opponent:0.7:5, facing_input:b:0:-1:4` |
| Roll / `down_smash` | 0 | `walk:away_from_opponent:15:0.95, wait:2, facing_input:a:0:-1:20` |
| Roll / `grab` | 25 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 28 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `face:opponent, wait:1, facing_input:a:0:-0.6:1, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4, wait:1, facing_input:a:0:-0.6:4` |
| Drop / `drop_nair` | 0 | `facing_input:x:0:0:1, wait:1, facing_input:a:0:-0.6:1, wait:25, facing_input:none:0:-1:1` |
| Drop / `forward_air` | 0 | `face:opponent, facing_input:x:0:0:1, facing_input:a:0.6:0:1, drift:away_from_opponent:1:15` |
| Drop / `short_hop_dair` | 0 | `facing_input:x:0:0:1, facing_input:b:1:0:4` |
| Drop / `down_smash` | 0 | `face:opponent, facing_input:a:0:1:50` |

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

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


## Yo-yo choice

Yo-yo sweep is an uncharged down-smash attempt. It does not pre-position a
charging yo-yo at ledge or guarantee a reaction punish on every drop. Hop aerials
stay near the player unless their handler explicitly adds drift. Height,
protection and native action readiness remain relevant.
