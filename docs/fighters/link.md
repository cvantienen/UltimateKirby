# Link

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **2** (`link`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt |
| Jump | **Nair** |
| Roll | **Dsmash** |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield Up-B** |
| Drop | **Shield**; Dtilt > Shield |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `link.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `link.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `link.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `link.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `link.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `link.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `link.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–31 |
| Down tilt | 10–11 |
| Forward tilt | 15–19 |
| Down smash | 12–13 |
| Down-smash rear window | 24–25 |
| Grounded up-special startup | 7 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 6 |
| Dash startup | 9 |
| Pivot startup | 10 |
| Shield-grab startup | 10 |
| Active duration | 2 |
| Standing reach reference (world units) | 11.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 104 |
| Walk speed | 1.247 |
| Initial dash | 1.98 |
| Run speed | 1.534 |
| Air speed | 0.924 |
| Gravity | 0.096 |
| Fall speed | 1.6 |
| Fast fall | 3.04 |
| Short-hop height | 13.38 |
| Full-hop height | 27.8 |

[Move source](https://ultimateframedata.com/link) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

Option definitions and input delays come from `fighters/link.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 23 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 17 | `opponent_input:a:0.6:0:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:23, release:1, else_facing, opponent_input:a:0:-1:11, release:1, end_facing` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/link.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/link/timing.rs`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
