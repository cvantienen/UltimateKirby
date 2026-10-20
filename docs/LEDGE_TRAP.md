# Hold Y: timed ledge responses with manual spacing

Hold **Y** to engage; release it to disengage. Y is consumed, and the automatic
jump uses **X**. L3 remains turbo teabag. Use A=Attack, X=Jump, R=Grab, ZR=Shield,
and Tap Jump off.

These controls apply to every human-controlled fighter on the selected player
input slot in all battle modes. Move timings and ranges still use Kirby defaults.

The default `ledge_distance_agnostic=true` policy leaves normal/jump/get-up-attack
spacing to you. Opponent acquisition and those responses ignore horizontal and
vertical range gates. Distance still ranks eligible ledge targets; it cannot
prevent acquiring the only eligible target. The assist selects a response from
the opponent's observed ledge option and schedules inputs using vulnerability,
recovery and native action readiness.

| Opponent option | Response |
| --- | --- |
| Hang | Keep manual capped walking. |
| Normal get-up | Short-hop nair, targeting the first active hitbox three frames before known invincibility ends. Neutral stick avoids facing-dependent aerials. |
| Jump get-up | Prepare a short hop, release X during squat, then neutral-stick A as soon as aerial input is legal after the committed takeoff. Ignore reach and height. |
| Roll | Lock one projected endpoint, move to a nearby standing area, brake and wait; pulse down-stick+A within the vulnerability window, with no follow-up chase. |
| Get-up attack | Hold shield; after a confirmed block, buffer grab regardless of distance or target protection metadata. If facing away, jump directly out of shield and request neutral air at the first observed aerial-ready update; do not turn or grab. No whiff-only grab in this mode. |
| Generic ledge drop | Hold shield as fallback. Recognized ledge options take priority. |
| Confirmed capture | End autopilot immediately; physical controls choose pummels, throws and follow-ups. Release and hold Y again to rearm. |

Manual horizontal input stays capped at `ledge_walk_strength` while waiting.
Only roll approaches automatically; normal-getup turning is a bounded facing
correction. No automatic return to a standing spot occurs. Actual punish recipes
briefly own their necessary inputs, then restore player spacing. L3 movement
remains independent, with automatic punish inputs taking priority.

A confirmed get-up-attack grab returns all buttons and uncapped stick input on
the first capture observation, including catch-pull. No automatic pummel, throw,
stance return or walk cap remains active after that capture, even if Y stays
held. Release Y and hold it again to rearm ledge assistance. Old
`ledge_throw_outward` values are still parsed for config compatibility and do
not re-enable automatic throws.

## Configuration

```text
ledge_trap = true
ledge_manual_spacing = true
ledge_distance_agnostic = true
ledge_walk_strength = 0.35
ledge_drop_defense = true
ledge_getup_attack = neutral_air
ledge_roll_dash_range = 10.0
ledge_roll_hit_window_frames = 5
ledge_follow_range = 40.0
```

Distance-agnostic mode defaults to true even if an old config omits the key.
Setting it false restores the previous reach-gated normal/jump/grab policy.
Legacy tilt, grab, anti-air and acquisition distances remain parsed but do not
gate those responses in the default mode.

Roll interception applies to every character. Auto mode projects an endpoint
once from observed horizontal travel and the native remaining roll time, then
locks it for that roll. Animation-root displacement supplements kinetic velocity;
if travel/timing is unknown, it waits instead of pursuing the current root.
The estimate does not read an exact future animation path and can be inaccurate
when a roll's speed changes. A measured `ledge_roll_distance` can override the
endpoint distance inward from a remembered ledge.

Kirby heads toward a standing area 80% of `ledge_roll_dash_range` short of that
endpoint (eight units back with the default ten-unit range), leaving more slide
margin. The target remains locked for that roll. Before reaching it, the policy
estimates queued coasting from current speed and learned input delay, then sums
neutral deceleration using the native `ground_brake` instance parameter
([parameter labels](https://github.com/ultimate-research/param-labels/blob/master/ParamLabels.csv)).
It releases forward early when that stopping estimate reaches the target and
uses gentle walking earlier in the approach. A braking forecast is not treated
as proof of arrival; a short approach correction still aims at the same fixed
spot. Missing/invalid braking data uses gentle walking instead of a full dash.
Once inside the standing area, it waits for residual approach speed to settle
before requesting down smash inside the existing vulnerability window. An opening timing window never marks an
out-of-range Kirby as positioned; he continues toward the standing area.
Movement stays within `ledge_follow_range` from the starting position (eight
units in the legacy reach-gated mode) and respects the floor probe. If a distant
projection exceeds this budget, the standing destination is clamped to the budget
instead of blocking all movement. Once positioned,
he never resumes pursuit or reverses to chase a moved opponent. He attempts one
actual down-smash animation per roll; recovery or a whiff does not start a chase.
`ledge_roll_smash_range` remains an alias for `ledge_roll_dash_range`.
Stopping and endpoint calculations remain estimates: surface traction, slopes,
status multipliers, acceleration during queued input, and changing online delay
can affect the actual stop. Device validation is required.

`ledge_roll_hit_window_frames` defaults to **5**, planning contact during attack
frames **7–11**. Set 1–13 to use a shorter window or extend it through frame 19.
At least one planned frame, plus learned input delay, must overlap the opponent's
known vulnerability before native roll recovery ends. This is a contact window,
not a fixed delay after detecting roll, and it does not change actual hitboxes.
Kirby must be within the estimated endpoint's down-smash range. Horizontal input
is released first; native down-smash readiness prevents a dash attack. Rejected
A inputs get release gaps and retries while the window remains open, without
spending a six-attempt budget. Unknown protection waits; a missed window releases
control instead of chasing.

## Timing and limits

[Kirby's frame data](https://ultimateframedata.com/kirby) gives nair startup 8
and active frames 8–32, and down smash startup 7 with front/back active phases
through frame 19.
Nair is one lingering hit with weakening phases. A proposed contact must overlap
known vulnerability and precede any known action deadline. Grounded recovery uses
the earlier valid cancel/animation-end frame. Missing protection timing waits
for observed vulnerability; it does not assume protection ended.

Native transitions still control when Kirby can actually jump, attack or grab.
Normal/jump timing ignores collision geometry by request: an attack can come out
from across the stage and miss. The front-facing block-triggered grab ignores
reach. The rear-side response preserves shield through shieldstun, jumps directly
from guard, releases X in jump squat and requests neutral A on observed takeoff.

Ledge-jump cover now sends **neutral X+A together** as the built-in
[short-hop attack input](https://www.smashbros.com/en_US/howtoplay/technique.html).
This gives the game's normal input pipeline the jump and aerial together, rather
than waiting to observe a delayed jump before transmitting a separate attack.
X is released in native jump squat while A remains queued. A rejected grounded
pair gets a neutral release sample and another paired request. Sending the pair
is an input request, not proof that either action was accepted.

All committed nair hops now finish using Kirby's own native action state before
opponent tracking or option selection is evaluated. Target loss, a regrab,
changed recovery forecasts or the opponent landing cannot cancel the aerial
partway through a hop. There is no aerial-readiness-term or six-attempt gate once
a hop is committed. Neutral A stays held until the native aerial starts. A queued
A hold remains owned through hitstop, so physical stick/C-stick or shield inputs
cannot replace it. Damage, landing, Y release and session changes end the
sequence. One actual hop/nair is attempted per visit/block; an accepted aerial
alone does not count as a confirmed hit.

Normal get-up still schedules its initial hop from protection timing and sends A
on observed takeoff. It shares the same committed follow-through and retries an
unaccepted X press with release gaps. Its scheduling estimate includes the delay
of two separate requests; ledge-jump's paired input uses one request instead.

These changes require a newly built and installed plugin. Existing NROs do not
change when source files are edited. Native observations and mapped-input
confirmation improve reliability, but this integration has no authoritative
online queue depth or future execution-frame API. It cannot guarantee an exact
opponent-relative attack frame in every Arena connection. Offline host checks
cannot establish online behavior; device testing with the rebuilt diagnostic
plugin is required.

Input timing starts only after the original mapper produces the expected action
button. Clean native action starts measure extra delay using motion-frame
advancement divided by native motion rate. Mapper polls, wall time and hitstop
are not counted. Interrupted actions, motion loops, rate changes, repeated input
pulses and missed first-action frames are discarded rather than guessed. The
recent eight clean samples supply a bounded upper-envelope compensation shared
by these input recipes. Unknown delay starts at zero and resets with the gameplay
session. Diagnostics report clean samples and the applied compensation.

Every forecast is recomputed against the current opponent protection/recovery
window with that delay included. This is a sampled animation-based estimator,
not access to the game's online input queue or a global simulation clock. It
cannot learn from frozen/ambiguous animations, undo delay that already occurred,
or establish guaranteed hits. Switch testing is still required.

Damage, target loss, tracking timeout, release of Y, mash and tech retain their
cancellation/priority rules. Hitstop preserves shield. Exact native attack
status/motion plus a registered hit/capture confirm success; a button request
alone does not. Diagnostics retain their throttled asynchronous output.

These are source changes. Install a newly built NRO and the intended config, then
restart Smash to use them.

Normal get-up now defaults to `neutral_air` (`nair` is also accepted). The
bundled config selects it; an existing config explicitly selecting a legacy tilt
continues to select that tilt until updated. Hop scheduling includes three jump
squat frames, eight nair startup frames and the learned delay for each of the
two input requests. It launches when that total is within three frames of known
protection expiry, then releases X and requests neutral A once natively airborne.
Late recognition/action readiness can reduce that lead. Unknown protection waits
for observed vulnerability. An expired recovery estimate does not veto the
normal-getup hop, and previous-status evidence can recover a missed get-up
transition. The active hitbox still needs correct player spacing.
