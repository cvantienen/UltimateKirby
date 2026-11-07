# Lucas

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **40** (`lucas`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

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
| Neutral air | 7–7 |
| Down tilt | 3–4 |
| Forward tilt | 7–9 |
| Down smash | 20–22 |
| Up Tilt | 4–4 |
| Forward Smash | 14–15 |
| Forward Air | 9–12 |
| Up Air | 7–10 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 12 |
| Dash startup | 14 |
| Pivot startup | 15 |
| Shield-grab startup | 16 |
| Active duration | 6 |
| Standing reach reference (world units) | 22.9 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
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

[Move source](https://ultimateframedata.com/lucas) · [Stats source](https://ultimateframedata.com/stats.php)

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
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/lucas.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/lucas/timing.rs`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
