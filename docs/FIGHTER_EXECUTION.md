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

- Neutral short-hop attacks pair neutral X+A. Directional `OneShot` aerials
  launch with X+ZL, release both during squat, then select directional A in air.
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
for every opponent or a move’s reach. Kirby instead runs to a fixed 20-unit
ledge inset; Yoshi uses tracked walking again. See their calibration below.

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

Yoshi Neutral-getup nair now turns toward the ledge, confirms facing and launches
X+ZL (both mapped to Jump) without A immediately on recognition, with a brief
full-stick forward nudge. Both release in squat for a short hop. Neutral A is
delayed using the takeoff snapshot: protection countdown minus 3 startup minus
16 early-hit lead. The wait counts advancing native game frames; unknown
protection withholds A. After acceptance it also nudges below nair frame 2;
its confirmed descending nair retains fast fall. Native activity remains 3–25.
Kirby's neutral nair adds two frames to its configured lead, and its jump-nair
protection timing gate allows two frames earlier; an already-immediate launch
cue stays immediate. Kirby Neutral nair keeps its brief nudge; Jump nair uses full forward stick
through confirmed nair frame 32, neutral at frame 33 or landing. These are timing
requests, not guarantees of earlier contact when native readiness prevents it.
Roll down-smash subtracts two world units from the configured
coverage distance (25 becomes 23), placing Yoshi slightly toward the ledge. This
spacing is an on-device calibration choice, not proof of optimal collision reach.
Kirby's ledge Roll → Grab runs to 20 units inward from the witnessed ledge,
without opponent tracking or reach subtraction. Native stopping-distance checks
brake the run; a 0.3 walking correction handles the final approach. He stops
within one unit, faces center stage and waits for native timed contact. This
experimental destination applies on every stage and needs device calibration.
Yoshi has returned to shared tracked 0.65-stick walking, with an inward-extending
endpoint and his own grab reach/active span.

A tracked grab can proceed from an already useful position without reaching the
estimated point; Kirby's fixed-stance experiment requires arrival instead. All
shared roll grabs require the opponent's predicted position,
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
Its timing stages the move three frames before protection expiry. Neutral bair
now prepares a 0.6-stick turn immediately on recognition, marks it as owned
preparation and confirms native away-facing before launch. Neutral X+ZL starts the
short hop; native jump squat releases both Jump buttons and native takeoff
unlocks backward+A. No grounded A,
C-stick tilt or attack-cancel acknowledgment is involved. Already away-facing
skips the turn. Native bair activity remains 6–12. Jump bair keeps paired X+ZL.
These checks verify input/state sequencing, not guaranteed hardware contact.


## Ledge-roll down-smash spacing

This is separate from the optional onstage [Roll read](ROLL_READ.md). The
existing shared ledge down-smash path in `ledge_trap/roll.rs` immediately locks
an approach point on recognizing roll get-up. Today its base is
`ledge_roll_distance` or the controlled fighter profile's common distance
(usually 25); it is **not** a measured endpoint for the opponent's fighter.
Changing the controlled fighter's base cannot accurately describe every enemy.

For opponent-aware calibration, keep two independent quantities:

```text
landing_x = ledge_x + inward * opponent_roll_distance
stand_x   = landing_x + inward * own_down_smash_offset
```

Here `inward` is +1 at a left ledge and -1 at a right ledge. A negative move
offset stops nearer the ledge; a positive offset stops farther toward center.
Measure the opponent's roll-end root relative to its ledge anchor, then select
the controlled fighter's offset so its useful active hitbox overlaps that root.
Use an opponent-distance table or verified native motion translation for the
first estimate; projecting peak roll velocity over the entire animation can
overshoot badly. Commit the destination once and use braking before arrival.
Timing remains based on native protection/recovery, never on reaching a point.

This change adds the separate onstage mode; it does not replace the current
ledge distance with an unverified opponent-distance table. To calibrate the
existing implementation now, use numeric `ledge_roll_distance` for a fixed
opponent during testing (`auto` restores the fallback). Per-move offsets belong
only in that fighter's down-smash wrapper, like Yoshi's
`ROLL_DOWN_SMASH_STANDOFF = 2` subtraction; avoid changing grab/zair destinations
as a side effect. Opponent-specific measured roll distances remain separate
work before the formula above can replace the common fallback for the full cast.

## Kirby Neutral nair walk example

Fighter-specific sequences live in executable handlers, while shared helpers
provide reusable mechanics. The menu's existing Kirby Neutral `nair` entry calls
`respond_to_neutral_getup_nair` in
`crates/kirby_assist_core/src/ledge_trap/fighters/kirby.rs`.
That handler first calls the existing nair response, which keeps authority over
when the jump/attack must start. If it has not committed, Kirby's own
`walk_before_neutral_nair` delegates to the shared opt-in preparation in
`ledge_trap/fighters/common.rs`, using only the remaining wait period for movement.
Yoshi Neutral nair instead short-hops immediately and schedules A separately. Kirby
Jump nair calls the shared movement-only `walk_toward_ledge` after its existing
jump timing check; an immediate launch leaves no time to walk. Never add a
position/arrival requirement to the attack gate or move walking ahead of it.

The sequence is:

1. Observe neutral get-up (`CLIFF_CLIMB`) on the tracked ledge opponent.
2. If protection expiry is known and the existing nair launch gate is still in
   the future, walk toward the remembered ledge anchor using 0.65 stick input.
3. Brake with native ground deceleration, stop about six world units inside the
   edge, or stop earlier if ground support is unsafe. Do not reverse/chase if
   already closer than the target. Unknown anchor/timing means no added walk.
4. At the original launch gate, stop this preparation and use the unchanged
   shared jump+A nair delivery, whether or not the walk reached its target.
5. Preserve the existing nair nudge and normal handback behavior afterward.

Kirby's approach constants are in
`crates/kirby_assist_core/src/fighter_profiles/kirby/timing.rs`:

```rust
pub const NEUTRAL_NAIR_WALK_STICK: f32 = 0.65;
pub const NEUTRAL_NAIR_EDGE_INSET: f32 = 6.0;
```

The stick value is the established fast-walk input below the dash flick
threshold, not a measured guarantee of maximum native walking speed. A smaller
inset requests a closer stop, subject to braking and ground support. Neither
constant changes nair startup, the configured early lead, or the jump buttons.
Validate the final spacing and momentum on Switch before tightening the inset.

Kirby Jump nair uses `JUMP_NAIR_DRIFT = 1.0` through confirmed animation frame
32; Neutral retains its 0.25 nudge below frame 4.
Yoshi Neutral also nudges at takeoff while waiting for its timed A; both Yoshi
nairs nudge below accepted animation frame 2. Its fast-fall settings remain.

For a different fighter, implement its own named handler in that fighter's
`ledge_trap/fighters/<fighter>.rs` module (or a child module for longer sequences)
and register it in that fighter's `SavedChoice`. Its handler may use shared jump,
movement, timing or native-acceptance helpers without sharing the whole action
sequence. Keep numeric tuning in its `fighter_profiles/<fighter>/timing.rs` and
state in its own sequence object when necessary. Avoid putting character-specific
branches into shared `aerial.rs` just to change one fighter. These are compiled
Rust changes; they require a future build/install, not runtime JSON loading.

## Frozen diagnostic calibration results

Diagnostic builds provide a [frozen result card](CALIBRATION_CARD.md) for all
five ledge response lists. The compact card appears beside the menu after an
attempt ends; R3 hides/shows both. It records available configured timing,
request/mapping/startup evidence, root spacing and waiting gates. Full detail
is written to `status.log`. Missing metadata stays unknown; configured frames
are not measured hitboxes. Silent builds exclude this feature.

Kirby Neutral down smash now opts into ledge walking at 0.65 stick during its
existing wait only. The same floor/braking/six-unit inset helper is used; it
never waits for arrival. At the 16-frame primer boundary, movement releases;
Down+A follows at 15 frames before predicted vulnerability, two frames earlier
than the old target. The timed attack takes priority even outside estimated range.
