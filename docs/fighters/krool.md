# King K. Rool

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **69** (`krool`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; CAN; **Ftilt**; Belly flop |
| Jump | **Nair**; DASH; Fair |
| Roll | **Dsmash**; Grab; CANON; GRAB2 |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair |
| Drop | **Shield**; Dtilt > Shield; Spike (risky); Flop (risky); BAIR |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `krool.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `krool.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `krool.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `krool.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `krool.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `krool.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `krool.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 4 start; active 4–5; 24 total; -- landing |
| Neutral Air | 7 start; active 7–8 / 9–30; 45 total; 9 landing |
| Forward Air | 11 start; active 11 / 12–14 / 15–17; 54 total; 11 landing |
| Back Air | 18 start; active 18–20; 49 total; 14 landing |
| Up Air | 7 start; active 7–13 / 14–19; 71 total; 13 landing |
| Down Air | 14 start; active 14–15 / 16–21; 61 total; 14 landing |
| Forward Tilt | 12 start; active 12–13; 42 total; -- landing |
| Up Tilt | 5 start; active 5–6 / 7–13; 37 total; -- landing |
| Down Tilt | 13 start; active 13 / 14–15; 42 total; -- landing |
| Dash Attack | 7 start; active 7–14 / 15–25; 60 total; -- landing |
| Forward Smash | 19 start; active 19–21; 61 total; -- landing |
| Up Smash | 6/19/22 start; active 6–9 / 10–11 / 19 / 20 / 21–23 / 22–23; 71 total; -- landing |
| Down Smash | 22 start; active 22–24 / 25; 63 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 8–10 |
| Dash grab | frames 11–13 |
| Pivot grab | frames 12–14 |
| Standing reach reference | 20.0 world units |
| Shield-grab startup | 12 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 133 |
| Walk speed | 0.903 |
| Initial dash | 1.936 |
| Run speed | 1.485 |
| Air speed | 0.945 |
| Gravity | 0.105 |
| Fall speed | 1.7 |
| Fast fall | 2.72 |
| Short-hop height | 13.5 |
| Full-hop height | 33 |

## Execution and calibration

Option definitions and input delays come from `fighters/krool.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 20 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 21 | `opponent_input:a:0.6:0:1` |
| Neutral / `belly_flop` | 0 | `opponent_input:a:0:-1:11, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 0 | `opponent_input:a+x:0:1:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:9, release:1` |
| Roll / `grab` | 23 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `menu_added_3` | 0 | `face:away_from_opponent, facing_input:b:0:0:60` |
| Roll / `menu_added_4` | 0 | `walk:away_from_opponent:12:0.8, wait:1, range:fighter_range, facing_input:r:0:0:4` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |
| Drop / `belly_flop` | 0 | `face:opponent, down_smash:1` |
| Drop / `menu_added_6` | 0 | `face:away_from_opponent, wait:1, facing_input:x:0:0:1, facing_input:a:-1:0:2, wait:6, drift:away_from_opponent:1:8, wait:4, facing_input:none:0:-1:4` |

See [this fighter's detailed execution notes](../KROOL_ISABELLE_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

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


## Belly flop and lingering coverage

The belly-flop option uses the grounded down-smash body hit, not an aerial
recovery or a new move. Its later shockwave is not counted as continuous body
coverage. Nair is the more lingering general option. Roll defaults to Grab;
capture returns manual pummel/throw control. The Drop spike/belly-flop choices
need spacing and can carry K. Rool offstage without automatic recovery.
