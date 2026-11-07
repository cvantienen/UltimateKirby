# Bowser

## Direct Auto react profile

Bowser has one direct sequence for each event: roll within **30 X / 12 Y**,
spot dodge within **25 / 10**, and air dodge within **35 / 19**. Each runs
`up_b:4, drift:opponent:1:69`. Starts require grounded Bowser, including held
shield; the following drift uses full horizontal input toward the tracked
opponent. The close-range shield Up-B backup also tracks for up to 69
native-progress frames after confirmed native startup, including air spinning.
Physical buttons and both sticks cannot cancel an already committed automatic
Up-B or its tracking. Ledge-trap L still stops it immediately. Native move end,
target loss and safety still release it early. This changes controller input only, not Up-B's hitboxes or animation.
A verified Up+Special packet retains a bounded, input-free native receipt through
shield hitstop/readiness loss. Late confirmation resumes tracking without
replaying B or spending its 69-frame tail while waiting. It reserves punish
priority only, expires within 24 mapper updates, and still obeys L/safety/target loss.
Active ledge-trap mode disables all automatic punish searches and responses,
even before an opponent reaches the ledge. Hanging/captured opponents are
excluded from ordinary punish targets.

Bowser's idle far-range automatic grab must observe native dash/run before
pressing Grab. It keeps full forward input through the dash-grab packet, avoiding
a standing whiff after shield release. Already-moving, close and shield grabs
keep their existing behavior. Failed dash startup is bounded by four delivered
approach attempts. Manual ledge recipes are unchanged.

Unshielded automatic grabs require an enemy shielding in front and vulnerable.
While Bowser shields, his shield-grab recipe accepts any vulnerable, grabbable
front opponent within its separate X/Y limits; enemy shield is not required.
If no legal grab wins, the shield backup uses its own range and native readiness. The
incoming-grab defense sends one Down+Guard request, then uses its separate **25 / 10**
`auto_grab_dodged` Up-B recipe only after native dodge startup and native readiness.
Its confirmed Up-B tracks for 69 drift frames. A delivered R cannot be replaced
by Up-B while awaiting receipt. Native grab/catch/hold/pummel/throw releases all
punish and ledge tapes so throws remain manual.

Bowser's complete Auto react list also includes `auto_grab:35:24`,
`auto_shield_grab:25:15`, and `auto_shield_backup:25:15`. Their actions and X/Y
limits are editable alongside the three enemy-dodge responses and the separate
grab-dodge follow-up. The default recipes retain existing native delivery,
movement, shield safety, priorities and 69-frame tracking. The current supplied
roll/spot/air values above were preserved, not retuned by this refactor.

## Default D-pad reversals

The local sequence file selects **D-pad Left: Fire Breath reverse toward enemy**
and **D-pad Right: Flying Slam reverse toward enemy**. These run once per press.
Use them airborne with Bowser free to start a special; no jump is inserted.

Left uses one frame of stick away, one neutral-stick B press, then two frames of
stick toward the opponent. Right sends B + stick away for one frame, then stick
toward for two frames. `opponent_input` mirrors X from the observed opponent,
independently of Bowser's facing, with no blocking Face step. Each step captures
its direction on entry. Missing opponent observations end the sequence.

These are input recipes, not native move/momentum edits. The result depends on
Bowser's native reversal window and starting state; Training validation of the
actual move is still required. The new syntax requires the updated mod build;
uploading only this text to an older build is not supported.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **15** (`bowser`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | FIRE BREATH; F TILT; UP B; **F SMASH** |
| Jump | BAIR; F TILT; **UP-TILT** |
| Roll | UP SMASH; GRAB; UP B; **F SMASH** |
| Attack | Shield DSMASH; **Grab**; Shield Up-B; SIDE B |
| Drop | Shield; FADE AWAY NAIR; FADE AWAY FAIR; DOWN SMASH; **Dair**; UP B |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge - Up-B chase | `bowser.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `bowser.roll.auto_grab` |
| `auto_grab_dodged` | — | After dodging enemy grab | `bowser.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll - Up-B chase | `bowser.roll.auto_roll` |
| `auto_shield_backup` | — | Shield grab backup | `bowser.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `bowser.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge - Up-B chase | `bowser.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 8–29 |
| Down tilt | 10–12 |
| Forward tilt | 10–14 |
| Down smash | 12–14 |
| Down-smash rear window | 28–30 |
| Up Tilt | 11–16 |
| Forward Smash | 22–27 |
| Forward Air | 11–14 |
| Up Air | 9–13 |
| Grounded up-special startup | 6 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 8 |
| Dash startup | 11 |
| Pivot startup | 12 |
| Shield-grab startup | 12 |
| Active duration | 3 |
| Standing reach reference (world units) | 20.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 135 |
| Walk speed | 0.901 |
| Initial dash | 2.255 |
| Run speed | 1.971 |
| Air speed | 1.155 |
| Gravity | 0.125 |
| Fall speed | 1.77 |
| Fast fall | 2.832 |
| Short-hop height | 15.7 |
| Full-hop height | 33 |

[Move source](https://ultimateframedata.com/bowser) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

Option definitions and input delays come from `fighters/bowser.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `down_tilt` | 0 | `face:opponent, facing_hold_until_manual:b:0:0` |
| Neutral / `forward_tilt` | 18 | `opponent_input:a:0.6:0:4` |
| Neutral / `up_tilt` | 10 | `facing_input:b:0:1:4, drift:opponent:1:70` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:5` |
| Jump / `nair2` | 0 | `face:away_from_opponent, wait:1, facing_input:x:0:0:1, facing_input:a:-1:0:4` |
| Jump / `up_air` | 0 | `opponent_input:a:0.6:0:1` |
| Jump / `up_tilt` | 0 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `facing_input:a:0:1:25` |
| Roll / `grab` | 5 | `walk:away_from_opponent:5:0.8, range:fighter_range, input:r:0:0:1` |
| Roll / `down_tilt` | 0 | `walk:away_from_opponent:5:0.8, facing_input:b:0:1:4, drift:opponent:1:70` |
| Roll / `menu_added_4` | 0 | `opponent_input:a:-1:0:10` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a:0.6:-1:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, if_facing_opponent, range:25, facing_input:r:0:0:4, else_facing, opponent_input:b:1:0:4, end_facing` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, facing_input:b:0:1:1, drift:opponent:1:70` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:b:1:0:4` |
| Drop / `shield` | 0 | `facing_hold_until_manual:zr:0:0` |
| Drop / `drop_nair` | 0 | `facing_input:a+x:0:0:4, drift:away_from_opponent:0.4:20` |
| Drop / `forward_air` | 0 | `face:opponent, wait:1, facing_input:x:0:0:1, facing_input:a:1:0:1, drift:away_from_opponent:0.3:10` |
| Drop / `down_smash` | 0 | `wait:2, opponent_input:a:0:-1:1` |
| Drop / `menu_added_6` | 0 | `facing_input:x:0:0:1, facing_input:a:0:-1:4` |
| Drop / `menu_added_7` | 0 | `facing_input:b:0:1:4, drift:opponent:1:70` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/bowser.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/bowser/timing.rs`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
