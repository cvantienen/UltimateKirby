# Bowser Jr.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **58** (`bowser_jr`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | NAIR PLAIN; Dtilt; **Ftilt**; Utilt; Fsmash |
| Jump | Nair; Fair; **Uair**; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield > Up-smash**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
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
| `auto_air_dodge` | — | Air dodge > Up Air | `bowser_jr.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `bowser_jr.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up Smash | `bowser_jr.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up Smash | `bowser_jr.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up Smash | `bowser_jr.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `bowser_jr.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up Smash | `bowser_jr.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–19 |
| Down tilt | 4–6 |
| Forward tilt | 7–9 |
| Down smash | 12–14 |
| Up Tilt | 7–14 |
| Forward Smash | 18–18 |
| Forward Air | 10–23 |
| Up Air | 6–13 |
| Down Air | 15–15 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 11 |
| Dash startup | 14 |
| Pivot startup | 15 |
| Shield-grab startup | 15 |
| Active duration | 3 |
| Standing reach reference (world units) | 18.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 108 |
| Walk speed | 0.924 |
| Initial dash | 1.76 |
| Run speed | 1.566 |
| Air speed | 1.134 |
| Gravity | 0.092 |
| Fall speed | 1.65 |
| Fast fall | 2.64 |
| Short-hop height | 16.59 |
| Full-hop height | 34.4 |

[Move source](https://ultimateframedata.com/bowser_jr) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

Option definitions and input delays come from `fighters/bowser_jr.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 18 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 29 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 26 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 23 | `opponent_input:a:0:0.6:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:21, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 2 | `opponent_input:a+x:0:1:1` |
| Jump / `up_tilt` | 4 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:20, release:1` |
| Roll / `grab` | 20 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 26 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a:0:1:1` |
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
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/bowser_jr.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/bowser_jr/timing.rs`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
