# Mr. Game & Watch

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **28** (`mr_game_and_watch`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Ftilt; Up Smash; Torch f-smash |
| Jump | **Nair**; Dash attacke |
| Roll | **Bury hammers**; Grab |
| Attack | Shield > Nair; **Shield Up-B** |
| Drop | **Shield**; beans; Bury hammers |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `mr_game_and_watch.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `mr_game_and_watch.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `mr_game_and_watch.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `mr_game_and_watch.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `mr_game_and_watch.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `mr_game_and_watch.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `mr_game_and_watch.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–10 |
| Down tilt | 6–8 |
| Forward tilt | 8–20 |
| Down smash | 12–16 |
| Up Tilt | 10–12 |
| Forward Smash | 17–18 |
| Forward Air | 10–13 |
| Up Air | 9–9 |
| Grounded up-special startup | 3 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 6 |
| Dash startup | 9 |
| Pivot startup | 10 |
| Shield-grab startup | 10 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.5 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 75 |
| Walk speed | 1.18 |
| Initial dash | 1.98 |
| Run speed | 1.679 |
| Air speed | 1.176 |
| Gravity | 0.08 |
| Fall speed | 1.24 |
| Fast fall | 1.984 |
| Short-hop height | 13.26 |
| Full-hop height | 27.51 |

[Move source](https://ultimateframedata.com/mr_game_and_watch) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

Option definitions and input delays come from `fighters/mr_game_and_watch.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 0 | `facing_input:x:0:0:2, wait:8, opponent_input:a:0.3:0:4, wait:13, facing_input:none:0.15:-1:1` |
| Neutral / `forward_tilt` | 18 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 0 | `face:away_from_opponent, opponent_input:a:0:1:24, wait:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1, drift:opponent:0.8:10` |
| Jump / `up_tilt` | 0 | `drift:opponent:1:2, opponent_input:a:1:0:4` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:28, wait:1` |
| Roll / `grab` | 0 | `drift:away_from_opponent:0.8:10, wait:15, range:fighter_range_instant, facing_input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, facing_input:a:0:-1:4` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield_until_block:90, up_special:1` |
| Drop / `down_tilt_shield` | 0 | `face:opponent, hold_until_manual:b:0:0` |
| Drop / `down_smash` | 0 | `face:opponent, down_smash:1` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/mr_game_and_watch.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/mr_game_and_watch/timing.rs`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Automatic punish configuration

All seven Auto-react events now have explicit recipes in this fighter's text file.
The current saved roll and spot-dodge recipes use downward A for 4 frames within
30 horizontal / 20 relative vertical units; the dodged-grab response uses the same
input within 30 / 10. Their saved names still say Up B, but their action fields
control delivery. Air-dodge punishment uses `input:b:0:1:4` within 30 / 30.
Each starts without an authored delay and requires
grounded admission. These are starting configuration limits, not measured hit guarantees.
The existing automatic grabs and shield-backup Up-B retain their saved values.
No Bowser spinning/drift sequence is added to Mr. Game & Watch.

## Automatic Bucket defense

The separate Auto-reflect Assist starts ON through this fighter's explicit
`reflect_default = true`. It predicts eligible incoming projectiles and requests
Down+B, with a six-frame startup estimate and two-frame early allowance. Energy
shots may be absorbed rather than reflected. The mode pauses at a full bucket
and never spends its oil automatically.

Repeated Samus charge shots/missiles can reuse a projectile object ID. Each new
native attack has its own duplicate guard, so a partially filled Bucket can react
again as soon as the native move is available. No timed rearm cooldown is added;
a full bucket still requires the player to empty it manually.

Grounded ready movement can turn toward a rear shot before using Bucket.
Ordinary airborne jump/fall can use it for front-facing shots only. Shield,
recovery, capture, tumble, attacks and existing punish/grab ownership block a new
attempt. Held/toggled ledge trapping, master OFF and spectating suspend the mode.
No sequence wait or physics change is added. Geometry/trajectory prediction and
native acceptance still need in-game calibration; this cannot guarantee every shot.

Edit the `reflect_*` Fighter settings in Training or this fighter's settings
section. The default prediction envelope is half-width 5, height 16 and depth 5;
maximum native-progress hold is 45 frames. These are detection estimates, not
native reflector/hurtbox modifications. See the
[full configuration and limits](../SEQUENCE_FILES.md#automatic-projectile-reflection).
