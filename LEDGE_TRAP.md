# Hold Down: Kirby ledge assist

The v18 source improves homing with proportional movement, stance hysteresis, shared roll/grab prediction timing, and independent approach distances. It captures anchors during catch/catch-move as well as hanging and preserves target identity for two missing mapper observations while emitting no stale movement or attacks. The stance moves from 8 to 9 units inward. It also retains the fixes for a status-range bug that mistook normal idle for ledge hanging, adds recovery of missed normal-get-up/roll transitions, removes extra tilt retry delay, and predicts shield-grab reach and protection at startup. Confirmed captures can throw outward, and Kirby returns to his original ledge stance after a punish. The supplied config selects downward-angled forward tilt for normal get-up. It retains analog-stick mash, tech, and turbo behavior. Source checks pass, but this revision has not been tested on a Switch. Release packages have not been rebuilt.

## Controls and responses

Use the selected human Kirby with **A=Attack, R=Grab, ZR=Shield, Tap Jump off**. Hold **D-pad Down before the opponent chooses an option**, starting near the ledge. Release Down to return physical control. Accepted animations continue under the game's normal rules. No forward-air or other aerial automation is present.

| Opponent option | Response |
| --- | --- |
| Hang | Position inward, face the ledge, and prepare Up without attacking |
| Normal get-up | Close distance, prepare the selected grounded tilt, then time its active window against vulnerability |
| Roll | Follow predicted travel, turn toward the endpoint, and request R when its grab window can overlap vulnerability |
| Jump | Remember short jump transitions, follow predicted movement, and request prepared grounded up tilt |
| Get-up attack | Shield, then attempt a startup-aware grab from confirmed block evidence or observed attack hitboxes clearing |
| Confirmed grab | At catch-wait, request a horizontal throw toward the acquired ledge anchor |
| Confirmed hit / recovered throw | Return to the original stance on supported ground; rearm on the next ledge visit |

Down takes priority over side-D-pad turbo. Mash, tech, and damage retain their existing priority; ledge assistance does not own airborne movement. Reserved D-pad taunts are suppressed as described in TURBO_CONTROLS.md.

## Positioning and movement

While the opponent hangs, the native cliff anchor establishes the ledge and stage side. The default stance is now **9 world units inward**, one unit farther inward than v17. A short momentum lookahead requests neutral stick when further movement would overshoot. Stance adjustment starts beyond 1.25 units of actual/predicted error, settles within 0.5 units at low speed, and retains its phase between those thresholds. Movement magnitude tapers with distance and respects the configured maximum; turns are capped at 0.3. This is a tunable starting point, not an optimal spot covering every escape.

Once an option begins, the policy pursues its predicted position. Normal get-up no longer waits indefinitely outside tilt range. Roll pursuit uses the same standing/dash/pivot grab-startup horizon as the grab gate and subtracts Kirby's short momentum estimate. Prediction stops extending a grounded roll/get-up's measured motion beyond its animation end.

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

Identical native observations (including both fighters’ motion clocks/state) reuse one input proposal across repeated mapper polls, instead of spending retries before the game advances. A real A release is required before another tilt request; there is no additional three-update retry timer. Observed animations still pause retries and both request/animation counts remain bounded. Unknown motion timing falls back to the existing mapper-update clock.

Jump selection no longer requires Kirby to be actionable on the brief jump-status sample. It remembers the option while he recovers, and native previous-status history can identify a jump that has already entered ordinary air movement. Up tilt uses the predicted rising position, including low-root fast-rise cases, rather than firing only when the current position enters the old box.

**This is not proof of a frame-perfect or unavoidable hit.** The model uses root-position gates and linear movement, not each character's exact hurtboxes, Kirby's animated hitbox shapes, or future decisions. Native metadata can be unavailable. Input polling, animation-rate changes, and buffering add uncertainty. A move cannot hit through protection or cover an opponent who has left its real reach.

## Grabs, whiffs, and confirmation

In-range shield-grab requests keep ZR held through native guard hitstop and between R pulses. Pushback pursuit drops shield once, stays committed through guard-off, then requests R directly when actionable and in reach. It does not raise shield again to wait for a grab.

For a get-up attack that misses shield, the policy waits to observe active hitboxes and then their absence before attempting the whiff punish. An empty startup snapshot alone is not treated as recovery. The adapter checks attack IDs 0–7. Clearing hitboxes is not proof that a multi-hit move cannot create another hitbox later; device tests across characters are still needed. Unknown attack metadata retains conservative block-based behavior.

An accepted tilt animation no longer ends the visit by itself. Native hit infliction confirms a hit; a recovered tilt whiff can retry. Grab startup similarly pauses requests, while actual capture completes the grab phase. Requests and observed animations are bounded to six attempts per visit. The native hit flag is not tied to a specific victim, so a multi-opponent match can provide ambiguous confirmation.

After a capture, `ledge_throw_outward=true` requests only horizontal stick in native catch-wait. The remembered stage side selects forward or back throw through the normal controller mapper; it does not press A for a pummel or force a throw status. It waits during grab pull/hitstop and leaves the throw animation to the game. No automatic throw is chosen without a cliff anchor acquired while hanging. Set the option false for manual throw selection (release Down to take control).

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
ledge_tilt_approach_distance = 6.0
ledge_grab_approach_distance = 5.0
ledge_follow_range = 40.0
ledge_follow_vertical_range = 48.0
ledge_auto_position = true
ledge_stand_distance = 9.0
ledge_move_strength = 0.65
ledge_grab_buffer_frames = 3
ledge_grab_button = R
ledge_throw_outward = true
```

The supplied config uses `ledge_getup_attack = forward_tilt`. To restore the low poke, set `ledge_getup_attack = down_tilt`. It supplies a moderate forward stick with a downward angle, then A after preparation. Up tilt remains the jump response; roll and shield follow-ups still use Grab. The Rust fallback default remains down tilt when the key is omitted; the supplied config explicitly overrides it. Forward tilt has different reach/startup and is not a universal replacement for every option.

`ledge_tilt_approach_distance` and `ledge_grab_approach_distance` select the desired pursuit separations when movement is needed. They are independent of attack eligibility: an eligible active-window grab may still be requested before reaching the movement target. Each accepts finite 0.5–20 and must be smaller than its matching attack range. Omitted values in older configs are fitted inside their ranges automatically; explicit invalid combinations reject the config. Increasing a hit/grab range no longer automatically increases pursuit separation.

All floating values must be finite. Attack/acquisition ranges accept 1–20; anti-air height 1–40; follow ranges 1–100; stance 2–15; move strength 0.1–1; grab buffer 0–6. Grab button accepts R/L/ZL and must match your profile. Proximity gates are not measured hitbox boundaries; increasing them can increase whiffs.

Missing `ledge_trap` disables the tool. False restores ordinary Down taunt. Obsolete `ledge_jump_response` and `ledge_fair_*` keys are ignored; they cannot restore aerial automation. `ledge_auto_position=false` disables automatic pursuit while retaining response inputs.

## Rust walkthrough

`crates/kirby_assist_core/src/ledge_trap.rs` contains pure policy; its child `ledge_trap/tests.rs` runs without a Switch. `Opponent` and `LedgeContext` copy observations, `LedgeSettings` stores validated tuning, and `LedgeOutput` proposes buttons/stick. `Option<T>` represents unavailable metadata. No native game pointers are retained by the policy.

| Function/type | Purpose |
| --- | --- |
| `sample` | Handle release/reset and reuse duplicate-update output |
| `evaluate` | Track one target, observe outcomes, apply control gates, dispatch |
| `target_velocity` | Supplement kinetic speed with animation-timed root displacement |
| `frames_until` / `MotionTiming` | Convert native animation deadlines into prediction time |
| `observe_action` | Distinguish startup, actual hit/capture, and recovered whiffs, including target loss |
| `complete_punish` / `position_at_ledge` | Throw outward in catch-wait, then recover the original stance with hysteresis/braking |
| `is_hanging` | Match actual cliff statuses instead of treating numerically lower idle as hanging |
| `choose_response` | Remember ledge options and block/attack-hitbox observations |
| `AttackWindow` / `predicted_offset` | Test approximate future reach against active frames and vulnerability |
| `watch_ledge` | Position while hanging and pursue/prepare the grounded get-up tilt |
| `respond_to_jump` | Request predicted grounded up tilt |
| `respond_to_roll` | Pursue travel and time a standing/dash/pivot Grab request |
| `respond_to_getup_attack` | Maintain shield or complete committed pursuit/punish |
| `try_tilt` / `try_grab` | Enforce bounded requests and release gaps |

`OptionPlan` selects the response. `Pending` records an input waiting for native confirmation. `&mut self` lets methods retain state across updates. Named status constants avoid memorizing hexadecimal values. `GetupAttack` is an enum with two grounded alternatives.

`src/live.rs` reads native observations outside STATE's mutex and applies output through the shared mapper overlay. Four read-only declarations were added for animation rate/end, own hit infliction, and opponent attack presence. Their symbols/signatures and HIT mask match the pinned [skyline-smash bindings](https://github.com/ultimate-research/skyline-smash/blob/d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435/src/cpp.rs). Compatibility still needs device validation. The mod does not force statuses, shorten timers, or generate controller hardware packets.

## Tests and device checks

Tests include early catch anchors/fast get-up after a brief observation gap, stale-output suppression/cancellation, stance hysteresis/braking, proportional pursuit and floor checks, independent approach-distance validation/compatibility, matching roll/grab horizons, idle after normal get-up, missed roll/climb history at acquisition, earliest released tilt retries, shieldstun/protection/reach forecasts, both outward throw directions and facings, disabled/unknown-anchor throw behavior, post-hit target loss/stance recovery/rearming, and low-root fast-rise trajectories, unknown/prolonged protection, different animation rates, short missed jump transitions, duplicate native observations/release edges, normal-get-up pursuit, selectable forward tilt, pre-expiry roll grabs, animation-driven travel with zero kinetic speed, both ledges, momentum/support checks, whiff retries, hit confirmation, and existing shield/mapper/mash/tech/turbo cases. Small-target trajectories are synthetic Pichu-like policy cases, not recorded Pichu collision data.

Use **Pichu on Final Destination** as the next small-character device baseline. Test both ledges, each option separately, then vary delay, damage, and inward/outward jump drift. Record gameplay frames around invincibility expiry and the first hit/grab. Compare down tilt and forward tilt for normal get-up. Recheck DK afterward. No Pichu or DK hardware validation was performed here.

Diagnostic builds identify **v18**. `ledge observation:` includes native target motion/rate, previous status, protection timing, observed velocity, selected grounded attack, and Kirby's hit flag. With `tech_debug=true`, snapshots can update on each target animation frame while Down is held. `ledge mapper:` reports actual raw edges and mapped output. Silent compiles logging out and uses the same policy. Host checks do not refresh NROs/ZIPs; rebuild and install one variant before testing.
