# Onstage Roll read

This is separate from the **Roll** ledge response. Kirby and Yoshi's **Roll read** row has
Off, Down smash (default) and Grab. It runs without holding L. Unsupported
fighters do not inherit this onstage policy. Menu edits/session changes cancel
pending inputs; the Training handoff and normal control bindings remain.

## Down smash recognition

Only native grounded forward/back dodge rolls qualify: ESCAPE_F (0x20) and
ESCAPE_B (0x21). Spot dodges, air dodges and ledge get-ups are excluded. The
opponent must be observed moving toward Kirby, or have previously approached
or crossed him during this same roll. A newly observed stationary or away-moving
roll alone does not trigger. A toward roll may stop short and still qualify;
it does not have to pass through Kirby's center.

Recognition uses the existing eight-slot roster, within a 64-unit horizontal
and 6-unit root-height watch region. This is a watch limit, not attack range.
The destination is a bounded projection from current position/velocity, using
at most six future frames and rechecking every observation. It is not an exact
animation endpoint or a full hitbox model.

Before taking control, the policy requires either a viable standing contact or
enough estimated time to walk into reach, brake, prime and start down smash
before roll recovery ends. Movement uses the existing 0.65 walking stick,
validated native walk/brake data and safe-floor rays. Missing physics/floor
support rejects an out-of-range chase; an already reachable standing punish
can still work. Stops are bounded to avoid walking through the projected point.

**Timing alone no longer triggers an out-of-range down smash.** Each candidate
active frame from Kirby's configured 7–19 interval must overlap actual known
vulnerability/recovery AND predicted reach (14-unit root-distance reference)
on that same frame. One neutral primer precedes Down+A. The input sample
rechecks timing, geometry and native smash readiness; the request is never
issued merely because the protection countdown reached a threshold.

Delivery lasts at most three advancing native frames until native startup
accepts. Target/protection loss and out-of-range changes cancel even during a
duplicate own animation frame. A frozen own idle pose can use the selected
opponent's native roll clock/rate to detect progress; repeated controller polls
cannot advance the primer. Hitstop does not spend the delivery window.

A roll has one bounded attempt, identified by object, status and native age.
Roster-slot reordering and higher-priority interruption cannot reissue that
attempt. A genuinely new roll resets its native age and can qualify again.

## Ownership and interruption

The [action owner](ACTION_OWNERSHIP.md) protects a committed primer/request from
passive movement or shielding. Safety responses, explicit manual tools and
confirmed follow-ups have defined interruption rules. Losing roll plans are
canceled, not held behind another action. Native attack lag is never bypassed.

## Grab and configuration

Grab remains a standing onstage punish with its existing facing, timing and
native capture handback. It does not chase, pummel or throw. Its watch distance
is 32 units and reach reference 12.5; this update does not change **ledge** Roll
→ Grab movement for Kirby/Yoshi.

The compiled entries are ROLL_READ_DOWN_SMASH / ROLL_READ_GRAB in Kirby's
fighter timing module; shared policy is in roll_read.rs and roll_read/smash.rs.
Unknown protection/recovery and insufficient startup time cause a skip.
No item scans, new hooks, file reads, delay learning or status/velocity writes
are added. Reach/projection remain estimates and need Switch calibration.

## Yoshi-specific windows

Yoshi uses front down-smash frames **7–8** and rear frames **22–23**, each
checked on its corresponding side of native facing. The gap is never modeled
as active. Reach is an estimated 13-unit root-space gate. Standing tongue grab
uses **14–21** and its 26-unit range reference. These share recognition, bounded
walking/primer delivery and ownership, not Kirby's move timing. In-range timing
still must overlap known vulnerability; this cannot punish an invincible roll.

## Punishing from shield

Down-smash Roll read now recognizes a punishable approaching/crossing roll while
in shield entry, hold or release. It reserves the same roll, releases all shield
buttons (including a physically held shield), and retains ownership through the
release. Automatic L shielding no longer excludes this Down smash route. Grab
mode keeps its existing rules.

Eligibility includes the universal **11-frame shield drop**, documented by
[Ultimate Frame Data](https://ultimateframedata.com/smash), plus attack startup
and any required approach. Entry adds a conservative remaining three-frame
commitment estimate; exit subtracts native animation progress at its rate.
These are eligibility estimates: Down+A still waits for native smash readiness.
The shield-release step already serves as the neutral smash primer. It never
sends Down/A while still shielding, avoiding a spot dodge or shield grab.

Shieldstun remains excluded. Unknown timing, a late/unreachable roll, target loss
or safety takeover cancels/skips the plan. This does not bypass shield-drop lag
or guarantee every online roll is punishable. Host tests cover held-shield
release, recovery/readiness and rejection; Switch validation is still needed.
