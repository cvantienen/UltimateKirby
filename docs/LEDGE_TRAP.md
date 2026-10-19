# Toggle L3: Kirby ledge assist

The separate [D-pad Down ledge-trump attempt](LEDGE_TRUMP.md) runs off a nearby
occupied ledge and returns control at catch. L3 remains this tool's trigger.

The v17 source fixes a status-range bug that mistook normal idle for ledge hanging, adds recovery of missed normal-get-up/roll transitions, removes extra tilt retry delay, and predicts shield-grab reach and protection at startup. Confirmed captures can throw outward, and Kirby returns to his original ledge stance after a punish. The supplied config selects downward-angled forward tilt for normal get-up. It retains analog-stick mash, tech, and turbo behavior. Source checks pass, but this revision has not been tested on a Switch. Release packages have not been rebuilt.

## Controls and responses

Use the selected human Kirby with **A=Attack, B=Special, X=Jump, R=Grab, ZR=Shield, Tap Jump off**. Click **L3 to enable nearest-ledge preparation** while grounded; it also arms in the air and waits for landing. An opponent need not be hanging. Click L3 again to return physical control. Accepted animations continue under the game's normal rules. A ledge jump is met with a predicted short-hop neutral air; no directional aerial automation is present.

| Opponent option | Response |
| --- | --- |
| Hang | Position inward, face the ledge, and prepare Up without attacking |
| Normal get-up | Stay planted; time the configured tilt against reach and vulnerability. Forward tilt sends direction and A together |
| Roll | Follow predicted travel, turn toward the endpoint, and request R when its grab window can overlap vulnerability |
| Jump | Remember short jump transitions, follow predicted movement, and request short-hop neutral air for its predicted active window |
| Ledge release into fall/double jump/aerial | With `ledge_drop_defense=true`, stop moving and hold shield; click L3 again for a manual punish |
| Get-up attack | Shield, then attempt a startup-aware grab from confirmed block evidence or observed attack hitboxes clearing |
| Confirmed grab | At catch-wait, request a horizontal throw toward the acquired ledge anchor |
| Confirmed hit / recovered throw | Return to the original stance on supported ground; rearm on the next ledge visit |

Down takes priority over side-D-pad turbo. Mash, tech, and damage retain their existing priority; ledge assistance does not own airborne movement. Reserved D-pad taunts are suppressed as described in TURBO_CONTROLS.md.

## Positioning and movement

While the opponent hangs, the native cliff anchor establishes the ledge and stage side. The default stance is now **8 world units inward**, allowing more room to react to roll travel than the previous 6-unit stance. A short prediction of Kirby's own momentum helps avoid repeatedly overshooting that position. This is a tunable starting point, not an optimal spot covering every escape.

Normal get-up now holds position rather than chasing or turn-walking. If the opponent is behind Kirby or outside the configured tilt window, it waits instead of crossing through them. Roll movement commits to one interception point; later opponent samples affect hit timing, not the movement destination. Prediction stops extending a grounded roll/get-up's measured motion beyond its animation end.

Some ledge travel comes from animation rather than kinetic velocity. When the same native animation clock advances, root displacement supplies an observed world-space velocity. Duplicate polls of the same animation frame retain the estimate. Motion changes, invalid timing, and unavailable data fall back to sanitized native kinetic readings. This avoids treating an animation-driven roll as stationary just because its kinetic speed reads zero.

Ground moves still require support probes. Missing support blocks a requested move/turn. Stage slopes, platforms, acceleration, collisions, and future choices are not fully modeled. Acquisition remains local; tracked follow-up permits 40 horizontal / 48 vertical units and expires after 48 non-stopped mapper updates outside ledge statuses.

## Timing and invincibility

`AttackWindow` checks predicted relative position during the move's active frames, instead of treating the target's current root as where the attack will land. It compares those frames with native vulnerability metadata. Timing is converted using:

```text
remaining game frames = (native deadline - current animation frame) / animation rate
```

Invalid/unknown deadlines are not guessed. If the target is protected and expiry is unknown, an attack waits for actual vulnerability. A known expiry can allow starting during invincibility so a later active frame overlaps the first reported hittable moment.

The windows use [Kirby's measured frame data](https://ultimateframedata.com/kirby):

| Response | Active-frame window used |
| --- | --- |
| Up tilt | 4–10 |
| Down tilt | 4–6 |
| Forward tilt | 5–8 |
| Standing grab | 6–7 |
| Dash/run grab | 9–10 |
| Pivot/shield grab estimate | 10–11 |

The shield-grab value is a startup estimate, not an independently measured collision window. Native shieldstun and input-buffer behavior still determine acceptance. The near-end shieldstun buffer is retained. Shield-grab prediction adds the observed remaining shieldstun to the startup estimate, then checks future position/protection. It does not need to wait for current vulnerability if a known deadline fits the active window; unknown deadlines still wait.

Confirmed get-up-attack blocks now permit a grab request during that buffer even while the opponent still reports active attack hitboxes. Previously the active-hitbox check could delay the request past the buffer window. Shield stays held through requests and release gaps. Without a confirmed block, observed hitboxes must clear before a punish. A host regression covers the early request and immediate ready-state retry; on-device timing still needs validation.

Identical native observations (including both fighters’ motion clocks/state) reuse one input proposal across repeated mapper polls, instead of spending retries before the game advances. A real A release is required before another tilt request; there is no additional three-update retry timer. Observed animations still pause retries and both request/animation counts remain bounded. Unknown motion timing falls back to the existing mapper-update clock.

Jump selection no longer requires Kirby to be actionable on the brief jump-status sample. It remembers the option while he recovers, and native previous-status history can identify a jump that has already entered ordinary air movement. Up tilt uses the predicted rising position, including low-root fast-rise cases, rather than firing only when the current position enters the old box.

**This is not proof of a frame-perfect or unavoidable hit.** The model uses root-position gates and linear movement, not each character's exact hurtboxes, Kirby's animated hitbox shapes, or future decisions. Native metadata can be unavailable. Input polling, animation-rate changes, and buffering add uncertainty. A move cannot hit through protection or cover an opponent who has left its real reach.

## Grabs, whiffs, and confirmation

In-range shield-grab requests keep ZR held through native guard hitstop and between R pulses. Pushback pursuit drops shield once, stays committed through guard-off, then requests R directly when actionable and in reach. It does not raise shield again to wait for a grab.

For a get-up attack that misses shield, the policy waits to observe active hitboxes and then their absence before attempting the whiff punish. An empty startup snapshot alone is not treated as recovery. The adapter checks attack IDs 0–7. Clearing hitboxes is not proof that a multi-hit move cannot create another hitbox later; device tests across characters are still needed. Unknown attack metadata retains conservative block-based behavior.

An accepted tilt animation no longer ends the visit by itself. Native hit infliction confirms a hit; a recovered tilt whiff can retry. Grab startup similarly pauses requests, while actual capture completes the grab phase. Requests and observed animations are bounded to three attempts per visit. The native hit flag is not tied to a specific victim, so a multi-opponent match can provide ambiguous confirmation.

After a capture, `ledge_throw_outward=true` requests only horizontal stick in native catch-wait. The remembered stage side selects forward or back throw through the normal controller mapper; it does not press A for a pummel or force a throw status. It waits during grab pull/hitstop and leaves the throw animation to the game. No automatic throw is chosen without a detected stage or opponent cliff anchor. Set the option false for manual throw selection (click L3 again to take control).

After a confirmed hit or recovered throw, automatic positioning returns Kirby to the remembered ledge stance, including when the opponent has left tracking range. Support probes still gate movement. The next ledge visit resets attack budgets, even if its hanging sample was missed. This maintains the setup for another attempt; it does not guarantee that a hit/throw sends every opponent offstage at every damage percentage.

## Configuration

Edit `SD:/ultimate/kirby_assist/config.txt` and restart Smash. Install the rebuilt plugin **and updated config**: an old explicit stance value of 6 overrides the new default.

```text
ledge_trap = true
# Normal get-up: down_tilt or forward_tilt (angled down).
ledge_getup_attack = forward_tilt
ledge_anti_air_range = 8.0
ledge_anti_air_height = 18.0
ledge_trap_range = 12.0
ledge_trap_vertical_range = 10.0
ledge_tilt_range = 7.0
ledge_grab_range = 6.0
ledge_follow_range = 40.0
ledge_follow_vertical_range = 48.0
ledge_auto_position = true
ledge_stand_distance = 8.0
ledge_move_strength = 0.65
ledge_grab_buffer_frames = 3
ledge_grab_button = R
ledge_throw_outward = true
```

The supplied config uses `ledge_getup_attack = forward_tilt`. To restore the low poke, set `ledge_getup_attack = down_tilt`. Forward tilt supplies a moderate forward stick with a downward angle together with A on the attack update; it holds neutral while waiting and during retry release updates. A ledge jump uses neutral-stick X+A when Kirby's short-hop neutral-air active window is predicted to overlap the target; roll and shield follow-ups still use Grab. The Rust fallback default remains down tilt when the key is omitted; the supplied config explicitly overrides it. Forward tilt has different reach/startup and is not a universal replacement for every option.

All floating values must be finite. Attack/acquisition ranges accept 1–20; anti-air height 1–40; follow ranges 1–100; stance 2–15; move strength 0.1–1; grab buffer 0–6. Grab button accepts R/L/ZL and must match your profile. Proximity gates are not measured hitbox boundaries; increasing them can increase whiffs.

`ledge_drop_defense = true` (default) protects against a witnessed ledge release
into ordinary airborne movement, including double-jump aerial approaches. It
requires the tracked opponent's previous hanging sample or native hang history;
it does not turn every nearby airborne opponent into a ledge-drop threat.
Kirby holds neutral stick plus ZR through the approach, landing and guard
hitstop, within the existing tracking range and 48-update follow timeout.
Regrabbing the ledge or choosing a newly observed standard ledge option selects
the normal response again. Click L3 again to punish manually. There is no blind
airborne grab or automatic shield-drop pursuit in this branch. Shielding is not
invulnerability: grabs, shield pressure and attacks during Kirby's own recovery
can still beat it. Set the option false to restore the prior drop behavior.

Missing `ledge_trap` disables the tool. False restores ordinary Down taunt. The neutral-air jump response is fixed in source; obsolete `ledge_jump_response` and `ledge_fair_*` keys remain ignored. `ledge_auto_position=false` disables automatic pursuit while retaining response inputs.

## Rust walkthrough

`crates/kirby_assist_core/src/ledge_trap.rs` contains pure policy; its child `ledge_trap/tests.rs` runs without a Switch. `Opponent` and `LedgeContext` copy observations, `LedgeSettings` stores validated tuning, and `LedgeOutput` proposes buttons/stick. `Option<T>` represents unavailable metadata. No native game pointers are retained by the policy.

| Function/type | Purpose |
| --- | --- |
| `sample` | Handle release/reset and reuse duplicate-update output |
| `evaluate` | Track one target, observe outcomes, apply control gates, dispatch |
| `target_velocity` | Supplement kinetic speed with animation-timed root displacement |
| `frames_until` / `MotionTiming` | Convert native animation deadlines into prediction time |
| `observe_action` | Distinguish startup, actual hit/capture, and recovered whiffs, including target loss |
| `complete_punish` / `position_at_ledge` | Throw outward in catch-wait, then recover the original stance |
| `is_hanging` | Match actual cliff statuses instead of treating numerically lower idle as hanging |
| `choose_response` | Remember ledge options and block/attack-hitbox observations |
| `AttackWindow` / `predicted_offset` | Test approximate future reach against active frames and vulnerability |
| `watch_ledge` | Position while hanging and request a stationary grounded get-up tilt |
| `respond_to_jump` | Request predicted short-hop neutral air |
| `respond_to_roll` | Brake/pursue travel and time a standing-only Grab request |
| `respond_to_getup_attack` | Maintain shield or complete committed pursuit/punish |
| `try_tilt` / `try_grab` | Enforce bounded requests and release gaps |

`OptionPlan` selects the response. `Pending` records an input waiting for native confirmation. `&mut self` lets methods retain state across updates. Named status constants avoid memorizing hexadecimal values. `GetupAttack` is an enum with two grounded alternatives.

`src/live.rs` reads native observations outside STATE's mutex and applies output through the shared mapper overlay. Four read-only declarations were added for animation rate/end, own hit infliction, and opponent attack presence. Their symbols/signatures and HIT mask match the pinned [skyline-smash bindings](https://github.com/ultimate-research/skyline-smash/blob/d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435/src/cpp.rs). Compatibility still needs device validation. The mod does not force statuses, shorten timers, or generate controller hardware packets.

## Tests and device checks

Tests include idle after normal get-up, missed roll/climb history at acquisition, earliest released tilt retries, shieldstun/protection/reach forecasts, both outward throw directions and facings, disabled/unknown-anchor throw behavior, post-hit target loss/stance recovery/rearming, and low-root fast-rise trajectories, unknown/prolonged protection, different animation rates, short missed jump transitions, duplicate native observations/release edges, normal-get-up pursuit, selectable forward tilt, pre-expiry roll grabs, animation-driven travel with zero kinetic speed, both ledges, momentum/support checks, whiff retries, hit confirmation, and existing shield/mapper/mash/tech/turbo cases. Small-target trajectories are synthetic Pichu-like policy cases, not recorded Pichu collision data.

Use **Pichu on Final Destination** as the next small-character device baseline. Test both ledges, each option separately, then vary delay, damage, and inward/outward jump drift. Record gameplay frames around invincibility expiry and the first hit/grab. Compare down tilt and forward tilt for normal get-up. Recheck DK afterward. No Pichu or DK hardware validation was performed here.

Diagnostic builds identify **v17**. `ledge observation:` includes native target motion/rate, previous status, protection timing, observed velocity, selected grounded attack, and Kirby's hit flag. With `tech_debug=true`, snapshots can update on each target animation frame while Down is held. `ledge mapper:` reports actual raw edges and mapped output. Silent compiles logging out and uses the same policy. Host checks do not refresh NROs/ZIPs; rebuild and install one variant before testing.

## Standing-only roll grab (source change after 1.0.3 build)

The roll response now uses one input recipe: neutral stick plus Grab, requested
only from native idle (WAIT), with actionable state and the standing-catch
transition enabled. It uses the six-update standing-grab startup prediction and
checks both modeled active frames for reach, facing and vulnerability. Walking
must settle to idle before Grab; running, braking, turning and guard states get
neutral stick with shield released until native recovery finishes. No roll dash
or shield-grab request is made. Get-up attack shield-grab behavior is unchanged.

Roll approach uses a locked interception spot and facing. Kirby brakes at that
spot, then holds neutral while waiting for protection or native readiness. Grab is requested
on the first observed idle update satisfying readiness, reach and vulnerability,
without an extra preparation update. If settling finishes too late, the roll
punish can still be missed. Early positioning remains necessary.

Diagnostics include `roll_settle_for_standing_grab`,
`roll_wait_grab_ready`, `roll_brake_wait_vulnerability` and
`roll_request_standing_grab`. Expected grab status is standing catch; an accepted
different grab animation is reported as `roll_grab_type_mismatch`.

Position-based reach and motion forecasts are approximations, not measured
collision volumes. Hardware logs are needed to confirm input acceptance and
collision timing. Standing-only input simplifies timing but does not guarantee
inescapable coverage. These source changes have not been built or tested;
existing 1.0.3 packages and backups remain the earlier build.

## Ground-only nearest-ledge preparation (unbuilt source)

Click L3 once to enable; click again to disable. Holding L3 does not repeatedly
toggle it. If enabled in the air, the mode waits and supplies no navigation
inputs. Leaving the ground clears the saved destination. On landing, the first
eligible movement observation selects a fresh nearest edge and starts approach.
Damage, attacks, hitstop, mash and tech can delay movement until native recovery.
Menu buttons, disconnect and death/rebirth clear the toggle.

Selection uses the same short downward `ray_check` support probes as the
existing ledge-trump tool, scanning along Kirby's CURRENT floor up to 400 units
in each direction. It does not scan vertically through the stage body, request
floor normals, navigate in air, jump, use recovery specials or drop through
platforms. The previous hit-position/normal FFI additions have been removed.
An unsupported configured stance is rejected. No edge found means no movement;
failed searches retry at most once per 30 input updates while grounded.

Kirby walks toward `edge + inward * ledge_stand_distance`, brakes before the
spot, faces the edge and holds neutral. This is the configured waiting stance,
not proof of one optimal position covering every character and get-up option.
`ledge_auto_position=false` disables positioning. Ledge option reads continue
during approach and take priority over waiting-position corrections. Roll
punishes remain standing-grab-only. Ground support gates every approach input;
unsupported travel stops rather than requesting a jump.

The detector finds boundaries of the current floor, not verified grabbable
stage cliffs. On an overhead platform it can select that platform's edge;
manually return Kirby to the main stage for main-stage ledge trapping. Sloped,
moving or disconnected terrain is not handled by a general pathfinder.
Diagnostic `locked_ledge` and `stance_distance` show the chosen destination.

The source has new ground-only regressions for both stage sides, unsupported
stances, landing activation, neutral waiting and air handback. Formatting has
been checked, but no compilation, test execution, builds or packaging were run.

## Stationary normal-get-up tilt (unbuilt source)

The normal-get-up response does not chase or turn-walk after the option starts.
It waits for native idle, correct facing, predicted reach and vulnerability.
Forward tilt sends its direction and A together on the eligible attack update;
there is no forward-stick preparation update. Waiting and retry release updates
hold neutral. Down tilt retains downward preparation without horizontal pursuit.
If the opponent is behind or out of range, no attack is requested; initial stance
still determines coverage. Other option responses and configuration are unchanged.
New regression cases cover both facings, protection, native movement recovery,
retry release, out-of-range opponents and cross-ups. These have not been run;
only formatting was checked, with no compilation, build or packaging.

## Committed roll interception (unbuilt source)

On roll recognition, the policy locks one endpoint and one standing position
within `ledge_grab_range - 1.5` of it (minimum margin distance 0.5). The nearest
point in that interval is chosen, so Kirby does not run through the endpoint to
stand on the far side. The destination is not recalculated as the opponent moves.
Kirby moves there, brakes using own momentum, settles to native idle, faces the
locked endpoint and holds neutral. Ground support still gates movement.

`ledge_roll_distance = auto` is the default. It projects an endpoint ONCE from
observed horizontal roll velocity and remaining motion duration, then latches it.
If those facts are unavailable/zero before protection ends, it waits for usable
motion instead of inventing an endpoint. That estimate assumes constant remaining
speed and can be inaccurate for nonlinear animations. A completed roll can latch
its observed final position. Auto is not guaranteed to commit on the first sample.

For immediate commitment from a known ledge anchor, set `ledge_roll_distance` to
a measured distance inward from that ledge to the opponent's roll endpoint
(finite 2..100 world units). This is character-specific roll travel, not grab
reach and not the desired Kirby stance. No universal numeric distance is supplied.
The config accepts `auto` to restore one-time estimation.

At the standing spot, the policy still checks the opponent's predicted root
against the six/seven-update grab window and observed protection expiry. It can
request Grab before the roll ends when the modeled active frame overlaps that
vulnerability window. Native standing-catch readiness, idle, low momentum,
release gaps and bounded attempts remain required. A fixed destination does not
justify grabbing through protection or outside reach. Unknown protection does
not spend attempts. Diagnostics expose `roll_interception=[endpoint, stand_x,
facing]` and phases for endpoint data, movement, settling, facing and timing.

This replaces continuous roll-root pursuit. It adds no native hooks or FFI calls.
Regression cases were updated/added for committed destinations, one-time motion
estimation, standing-only readiness and pre-exit vulnerability timing. Formatting
was checked; no compilation, test execution, build or packaging was performed.
