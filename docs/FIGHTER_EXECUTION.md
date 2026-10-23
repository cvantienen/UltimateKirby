# Fighter-specific execution and calibration

Move startup values alone do not determine an optimal sequence. A response also
needs a hop choice, input delivery order, target height, approach/braking, facing,
reach, and a real actionable/vulnerability deadline.

Each compiled fighter registration now declares `aerial_execution`, independent
of its move windows. It chooses `Short`, `Full`, or `Adaptive` independently for
normal-get-up nair, jump coverage, and shield nair. Kirby explicitly keeps its
original short-hop plan. Other profiles declare adaptive jump coverage; normal
and shield nairs stay short. Little Mac's default jump handler is grounded
up-tilt and does not turn into a jump because of this policy.

- Short hop pairs X+A, then releases X during squat and retains A until acceptance.
- Full hop holds X through native squat. Only actual takeoff unlocks A. Its forecast
  accounts for the extra delivery leg rather than treating X+A as a full hop.
- Adaptive prefers a short hop that intersects the modeled target trajectory; if
  that fails, it checks full hop. It uses native mini-jump/full-jump speed, gravity,
  terminal speed, attack active frames, protection/recovery, with zero extra live input-delay compensation.
- Generic adaptive jump responses wait for launch evidence from zero-speed
  initial ledge-jump poses. Yoshi’s dedicated immediate Jump nair is an exception. Missing native
  physics explicitly falls back to the existing short-hop behavior. Neither
  missing data nor a failed forecast proves an optimal contact.

The newer up-air/fair single-move variants use the same jump policy. Their own
animation identity, stick direction, active span and drop drift remain compiled
per move. Drop variants retain their explicit short-hop recipes. Smashes, tilts,
grabs and specials keep their separate readiness/acceptance paths; grabs and
specials return manual control as previously configured. Roll movement already
uses the controlled fighter's live velocity/braking. Its shared 25-unit coverage
point is an initial heuristic about opponent travel, not a measured endpoint
for every opponent or a move’s reach. Kirby/Yoshi walking grabs can extend the
approach inward from observed roll travel; see their current calibration below.

## What remains uncalibrated

The full non-DLC roster is now registered; [the per-fighter data audit](NON_DLC_ROSTER.md)
lists its compiled frame intervals, movement references and exceptions.
This is **not a claim that every profile is device-calibrated or universally optimal**.
The current reach/height envelopes are coarse configuration bounds, not full
per-move hitbox/hurtbox data. `ledge_distance_agnostic=true` still leaves horizontal
spacing to the player; adaptive height prediction does not override that setting.
The forecast cannot fully predict opponent steering or a changing online queue.
No arbitrary per-fighter drift/delay numbers were added just to make sequences
look different.

Calibrating each selected response still requires observations of first accepted
input, takeoff, move start, target protection expiry, height/spacing and contact.
That evidence determines whether a fixed full hop, a delayed aerial, different
movement or a different move is preferable in a particular matchup. Verified move
windows remain useful inputs, but they do not establish those missing values.

No file loading, network changes or new background roster scans were introduced.
The added native jump-speed query runs only during active ledge observation;
forecast integration has constant cost per candidate active frame. Regression
source covers hop choice, held-X takeoff, paired short-hop preservation, missing
physics and duplicate observations. No build, Rust tests, upload or device
calibration was performed for this update.


### Current Kirby and Yoshi calibration

Yoshi's menu and executable option handlers are in
`crates/kirby_assist_core/src/ledge_trap/fighters/yoshi.rs`. Native move data and
small Yoshi-only tuning constants are in `fighter_profiles/yoshi/timing.rs`.
The profile in `fighter_profiles/yoshi.rs` connects those handlers and defaults.

Jump-getup nair uses a dedicated immediate short-hop handler, bypassing generic
height/protection waits and adaptive full-hop selection. It uses the existing
`OneShot` helper to pair neutral jump+A, release jump in squat, and keep attack
neutral until native nair starts. Then it requests horizontal stick magnitude
0.25 toward the opponent's position captured at recognition, only while native
nair animation frame is below 4. Afterward the automatic stick is neutral;
existing momentum can continue under native air physics. Other Yoshi jump
aerial options retain their adaptive-hop behavior. Jump nair remains immediate
on recognition, so it cannot be advanced another two frames.

Neutral-getup nair has a Yoshi-specific 17-frame lead before protection expiry,
two frames earlier than its previous 15-frame lead. Real startup/active frames
remain 3–25. It uses the same brief 0.25 nudge toward the opponent, starting only
after native nair acceptance. Jump and attack startup remain neutral.
Kirby's neutral nair adds two frames to its configured lead, and its jump-nair
protection timing gate allows two frames earlier; an already-immediate launch
cue stays immediate. Both Kirby nairs use the same brief nudge. These are timing
requests, not guarantees of earlier contact when native readiness prevents it.
Roll down-smash subtracts two world units from the configured
coverage distance (25 becomes 23), placing Yoshi slightly toward the ledge. This
spacing is an on-device calibration choice, not proof of optimal collision reach.
Yoshi and Kirby use inward walking for roll grabs. The common roll point is an
initial approach estimate, not an opponent-specific endpoint. Native observed
root travel can extend that point farther inward; it never pulls the approach
back toward the ledge. Navigation forecasts at most six frames ahead, avoiding
long tongue startup extrapolations that would overshoot the roll.

The navigation area uses 60% of configured standing reach (7.5 units for Kirby,
15.6 for Yoshi). Walking stays at 0.65 stick until inside; braking protects against
crossing the target rather than stopping outside the area's outer edge. This
removes the repeated tiny walk/brake loop visible in the supplied diagnostic log.

A grab can proceed from an already useful position without reaching the estimated
point. All shared roll grabs now require the opponent's predicted position,
facing, height and vulnerability to overlap the SAME configured active frame.
That check uses the configured standing grab range, while the smaller navigation
area supplies a positioning margin. These are root-based forecasts, not live
hitbox/hurtbox collision checks. Long grabs retain their full active duration.
Native catch readiness, movement release, bounded input delivery and capture
handback remain in use; down-smash navigation is unchanged. New diagnostic gate
`roll_wait_actual_grab_reach` distinguishes timing-ready but out-of-reach waits.
Validate prediction and spacing on Switch; this does not guarantee every catch.

Kirby's Jump Bair uses 0.35-stick drift after exact attack_air_b acceptance.
Neutral Bair uses a short hop with zero post-acceptance drift.
Its timing stages the move three frames before protection expiry, allowing one
extra setup frame when attack cancel is needed. Native bair activity remains 6–12.

When already back-facing, the handler uses the direct neutral short-hop input
and backward+A in squat. When front-facing, it requests backward C-stick using
both analog axes and raw r_left/r_right digital bits through the common button
edge history. The earlier analog-only request omitted those digital aliases.
The normal mapper must confirm attack+CStickOn in the requested direction before
native progress advances to jump. On the next advancing update it releases
C-stick and holds X+ZL jump (the player's existing jump mappings) plus forward
left stick. The two jump buttons select short hop. Jump is released once native
squat/takeoff is observed; the already-buffered attack is not replaced by an
additional A press. This requires C-stick Tilt Attack and ZL mapped to Jump.

No grounded turnaround animation is waited out. Early tilt frames 1–2 are the
only accepted cancel window; a late/missed cancel or wrong aerial ends the
attempt. Mash, tech, slides and trump retain priority. The mapper restores raw
physical axes and button words after its normal call. No native status or
hitbox changes are made. Source checks do not establish on-device reliability
or frame-perfect online delivery; validate the sequence on Switch.


## Frozen diagnostic calibration results

Diagnostic builds provide a [frozen result card](CALIBRATION_CARD.md) for all
five ledge response lists. The compact card appears beside the menu after an
attempt ends; L+R3 hides/shows both. It records available configured timing,
request/mapping/startup evidence, root spacing and waiting gates. Full detail
is written to `status.log`. Missing metadata stays unknown; configured frames
are not measured hitboxes. Silent builds exclude this feature.
