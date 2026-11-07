# Ledge autopilot

> Current ledge options and all input scheduling now come only from [sequences.txt](SEQUENCE_FILES.md). The compiled-engine descriptions below are historical/reference material, not active move timing. See the fighter pages for the current file-authored schedules.


## Input freshness

The original wall-clock watchdogs apply to every ledge input: requests expire
after 35 ms and an observation gap of 250 ms resets the controller association.
The stall-preserving committed-input exception has been removed. No automatic
delay learning or compensation is enabled; configured fighter timings remain.

## Held-L defensive shield

With `autopilot_shield = true` (the default), held ledge assist (X by default) can request normal
ZR shielding on the ground against a nearby fighter's active or known imminent attack.
The opponent need not have grabbed the ledge. Releasing L releases this assist.

The check uses the existing bounded fighter roster and validated live accessors.
A five-frame swept root envelope (24 horizontal / 28 vertical world units)
provides the cheap active-attack warning. Unambiguous compiled active windows
also warn for mapped jabs, tilts, smashes and aerials, with the existing profile
fallback for supported move families. Unknown timings do not borrow Kirby data.

For active hitboxes beyond the root envelope, a bounded read of native attack
geometry tests a swept conservative standing envelope. This covers extended
attacks and approaching/descending aerials that a root-only distance test misses.
A broad spatial filter precedes these reads; at most 16 active slots receive
geometry work across the roster (at most two joint transforms per slot). Reaching
that cap requests shield conservatively. These are read-only hitbox observations,
not changes to attacks, and use the existing ABI without a new observer hook.
Projectile/item objects are not scanned. Estimated body bounds and linear motion
remain approximations; coverage is not guaranteed.

Shield input requires native guard availability or an existing guard/shieldstun
state. Native guard availability is checked without an idle/walk-only status
filter, so crouching and other legitimately shield-ready states qualify too.
Committed punishes (including onstage Roll read), capture/manual handback,
mash/tech, slide and trump
retain priority. Defense overrides idle/preparation movement only; movement is
neutral while shielding to avoid adding dodge direction. An existing defensive
shield is held through native shield hitstop. A fixed-size hold bridges small
gaps for up to four advancing native frames after the last warning; repeated
polls and hitstop do not spend that interval. Release, menu/tool cancellation,
or lost native shield readiness still stop it immediately. Once that tail ends it hands
control back. The separate optional [Out of shield](OUT_OF_SHIELD.md) mode can
respond after an actual approach block; this shield safeguard alone does not
automatically attack or parry.

Passive ledge tracking, including jump approaches, no longer disables this
safeguard merely because an opponent is being tracked. Actual get-up-attack
responses and committed text timelines remain in control: this feature cannot
replace their shield, action or configured wait steps.

This cannot shield through hitstun, shield break or uncancellable move recovery.
It does not parry, detect grabs, or guarantee collision coverage.

The added native reads run only for grounded, guard-eligible, L-held ledge
autopilot observations. There are no new hooks, threads, weapon scans or live
file reads. Runtime cost and online effectiveness still require Switch testing.
Diagnostic ledge snapshots label the action `autopilot_defensive_shield`.
Disable with `autopilot_shield = false` in `[universal.settings]` of startup `sequences.txt`, then restart.

## Kirby auto-duck

**Currently disabled**, including its Y binding, source tracking and ATTACK
observer-hook installation. The legacy config cannot restore it. Y instead
attempts the selected [Finisher](FINISHERS.md). The retained implementation
below documents the former behavior for future offline calibration.
The live path no longer uses a Falcon Punch/move-name allowlist. It reads active
hitbox sphere/capsule endpoints in world coordinates from nearby enemy fighters
and cached projectile/item objects, including size and root velocity.

A fixed six-frame swept envelope predicts incoming contact. It requests normal
down-stick crouching when the detected hitbox overlaps an estimated standing
Kirby body but clears the estimated crouching body. Multiple high hitboxes may
be ducked together; a low/large/descending hitbox intersecting the crouching
bound chooses shield if native shielding is ready. Native crouch readiness is
required; an already raised shield is not replaced with down+shield. Damage,
committed punishes and other higher-priority tools still take precedence.
Down is not requested on passable platforms or for nonstandard Kirby sizes.

The calibration model in `autopilot_guard/duck_geometry.rs` uses standing height
12, crouching top 4.5 and horizontal/depth half-width 4 world units. **These are
estimates, not live native hurtboxes or universal clearance guarantees.** Capsule
bounds are deliberately conservative. Native crouch startup, changing joint
poses, slopes, hitstop and delayed inputs can still defeat the prediction.
Only currently active attacks are visible; this is not prediction of attacks
that have not spawned hitboxes yet. Grabs are not covered.

One deferred `sv_animcmd::ATTACK` observer, installed only when auto-duck is
configured on, calls the existing hook chain unchanged and remembers IDs in a
32-slot atomic ring while the selected fighter is Kirby. It never edits attack
arguments. This enables weapon/item discovery without traversing an unknown
object-manager layout. The cache is retained while Y is released so a projectile
already in flight can be considered when Y is pressed. Battle end clears it.
Only held-Y observations scan geometry: up to eight cached fighters and 32
weapon/item IDs, eight active slots per object and at most 32 active hitbox
transforms per observation. There are no new threads, per-observation allocations,
render effects or per-frame log strings. This is bounded work, not measured
Switch performance. The additional ATTACK hook needs on-device coexistence
validation with Training Modpack; its Lua arguments and original call are untouched.

Objects not seen through the normal ATTACK command, evicted cached objects,
secondary attack banks, unusual scaling rules and stage hazards can be missed.
No claim is made that every hitbox can be ducked, that every projectile is
tracked, or that this is exact collision prediction. Validate offline before
relying on it. L's separate fighter-only shield policy is unchanged.

## Controls

Hold **L** for the selected fighter's saved ledge responses and configured walking
limits. With Ledge trap mode ON, tracking stays active without L; L still adds
walking/known-ledge stopping limits. An acquired setup is protected from competing
configured responses until it finishes. Plus/Minus, menus and native safety keep
their documented controls; see [current controls](CONTROLS.md).

Minus opens/closes the menu. Outside Training, R3 switches between ledge and
Buttons / Assists selectors. In Training, R3 edits recipes; L3/R3 on a
sequence-based assist opens its separate rule list. X/Y/D-pad run the selected
fighter-specific sequence or Normal button. Hard-coded slide/B-reverse/Down-trump
bindings and the native Y Finisher selector are removed.

Use the [controller mapping](CONTROLS.md#controller-mapping). Configure exact
attack inputs, hold durations, checks and delays in the master file rather than
relying on a compiled fighter recipe. Auto-mash is OFF in the supplied master.

These controls apply to a human-controlled fighter with a loaded ledge profile
on the selected local controller in valid battles. All non-DLC forms have
compiled profiles. Valid unconfigured kinds fall back to Kirby; invalid routing
stays inactive. See [fighters](FIGHTERS.md).

Kirby's **Drop → Short-hop dair** option reacts to a witnessed ledge release
with **neutral X first**, then **down+A during native jump squat/takeoff**,
releasing Jump for a short hop. The attack-selection stick is 0.6 toward the
ledge and -1 down. Separating the initial Jump from Attack prevents the first
request from selecting nair. It requests the action as soon as native grounded jump/actionability
allows, without a spacing, facing-turn, reach or vulnerability timing gate.
Down+A stays queued through takeoff, even
if the opponent disappears from tracking. Once the actual dair starts, A/down
release and full horizontal drift continues through startup. At dair animation
frame 18 ([first active frame](https://ultimateframedata.com/kirby)), control
hands back so you choose recovery. A different aerial, landing, interruption or
request timeout also ends this one-shot attempt. Release/rehold L to rearm.
Spacing is yours; it can drift offstage and does not guarantee a spike.

The default `ledge_distance_agnostic=true` policy leaves normal/jump/get-up-attack
spacing to you. Opponent acquisition and those responses ignore horizontal and
vertical range gates. Distance still ranks eligible ledge targets; it cannot
prevent acquiring the only eligible target. The assist selects a response from
the opponent's observed ledge option and schedules inputs using vulnerability,
recovery and native action readiness.

| Opponent option | Kirby default response |
| --- | --- |
| Hang | Keep manual capped walking. |
| Normal get-up | Short-hop nair, targeting first activity ten frames before known protection expiry with the supplied eight-frame setting plus the current two-frame lead. Neutral stick avoids facing-dependent aerials. |
| Jump get-up | Pair neutral X+A for short-hop nair; release X in squat while A remains buffered. A timing-gated request permits two frames earlier; immediate recognition stays immediate. |
| Roll | Down-smash approaches the common coverage point and pulses down+A during vulnerability. Selecting Grab uses the separate range/active-frame logic below. |
| Get-up attack | Hold shield; after a confirmed block, buffer grab regardless of distance or target protection metadata. If facing away, pair neutral jump+A directly out of shield; do not turn or grab. No whiff-only grab in this mode. |
| Generic ledge drop | Shield by default; alternatives are one native-confirmed down tilt then shield, or short-hop dair. There are no Off entries. Recognized ledge options take priority. |
| Confirmed capture | End autopilot immediately; physical controls choose pummels, throws and follow-ups. Release and hold L again to rearm. |

Manual horizontal input has two caps while waiting: the
`ledge_walk_strength` baseline minus 0.02 normally, or a fixed **0.50** when
horizontal stick magnitude is at least 0.95 and vertical magnitude at most 0.15.
The supplied baseline gives **0.33 normal / 0.50 near-full horizontal**.
The fast-walk cap is 50% stick deflection, not a guaranteed 50% of native walk
velocity. Full analog deflection is no longer sent for manual autopilot spacing.
Weak inputs remain proportional and vertical input is preserved. Digital
stick-flick aliases are cleared; native fast-walk versus dash behavior still
needs device validation. These are stick strengths, not world-units/frame;
each fighter's native walk curve determines movement speed. This affects manual
spacing only, not recipe-controlled roll approaches, ledge-walk preparation,
punish drift or attack input timing.
Only roll approaches automatically; normal-getup turning is a bounded facing
correction. No automatic return to a standing spot occurs. Actual punish recipes
briefly own their necessary inputs, then restore player spacing. R3 teabag
remains independent, with automatic punish inputs taking priority.

A confirmed get-up-attack grab returns all buttons and uncapped stick input on
the first capture observation, including catch-pull. No automatic pummel, throw,
stance return or walk cap remains active after that capture, even if L stays
held. Release L and hold it again to rearm ledge assistance. Old
`ledge_throw_outward` values are still parsed for config compatibility and do
not re-enable automatic throws.

Kirby also has an optional neutral-get-up **Down-B / Stone** recipe in the
[per-fighter saved lists](FIGHTER_CONFIGS.md). Its grounded
startup is configured separately (11 by default), with zero early lead; nair
keeps its original lead of 8. This predicts an input time rather than proving
a frame-perfect online hit. Stone confirmation hands controls back so B remains
available to leave Stone normally. Release/rehold L to rearm.

## Configuration

The menu selects compiled fighter response functions. An optional startup-only
[`sequences.txt`](SEQUENCE_FILES.md) section can replace an existing ordinary
choice's action recipe while retaining its compiled fallback. Global activation, shared
shared startup settings and per-fighter overrides live in the master `sequences.txt`.
Config files cannot register functions or define their action sequence.


```text
ledge_trap = true
ledge_manual_spacing = true
ledge_distance_agnostic = true
ledge_walk_strength = 0.35
ledge_drop_defense = true
ledge_follow_range = 40.0
```

Distance-agnostic mode defaults to true even if an old config omits the key.
Setting it false makes `ledge_trap_range` and
`ledge_trap_vertical_range` constrain opponent acquisition. The old
fighter-independent tilt, grab, anti-air and roll ranges remain accepted and
validated for config compatibility, but do not affect live responses; grab
recipes use the selected fighter's profile reach instead.

Use the selected fighter's saved sequence to define the roll response. Stand
positioning remains controlled separately by `ledge_stand_distance`; a legacy
`ledge_roll_distance` value does not tune it.

Native speed and ground braking guide early stick release; live extra delay is zero.
The final short approach slows down near the fixed point. Missing braking data
uses a short coasting estimate instead of blocking movement. Arrival, not an
opening timing window, ends navigation. Opponent movement cannot drag the spot
farther away. A missed punish window never becomes a chase.

At the spot, the policy releases horizontal input and sends down+A when the
native down-smash transition is ready and the hit window overlaps vulnerability
before roll recovery ends. Separate velocity, endpoint-reach and height vetoes
have been removed from this timing path. Rejected down+A inputs get release gaps
and retries during the window; only one actual down-smash animation is attempted.

Roll grabs use the fighter's own configured grab reach, startup and active
length. Kirby's experimental Roll → Grab runs directly to **20 units inward
from the witnessed ledge**, brakes, uses a small walking correction near arrival,
stops within one unit and faces center stage. Arrival precedes the timed catch;
his destination does not extend from enemy movement or subtract grab reach.
Yoshi instead uses shared tracked 0.65-stick walking again. Existing timed grab
checks remain; a fixed inset is not stage-specific or a guaranteed catch.
Other fighters retain their existing navigation style.

The approach area extends up to 60% of configured standing reach. The request
itself requires predicted opponent reach, facing, height AND vulnerability on the
same active grab frame, using the full configured standing range. An already
useful tracked position can request without reaching the common point; Kirby's
fixed-stance experiment instead requires arrival. These are root
position/velocity estimates, not an animated grab/hurtbox model. Diagnostic gate
`roll_wait_actual_grab_reach` means timing is available but forecast geometry fails.

The whole configured grab-active interval is checked against a target three
frames after protection expiry (midpoint for narrow vulnerable windows, two
frames if the recovery deadline is unknown). Unknown protection still waits.
Only activity before the known recovery deadline counts. Native can_grab,
movement release, bounded delivery and capture handback remain in effect.
There is no learned input-delay compensation. The area and roll endpoint remain
estimates; they do not extend native hitboxes or guarantee a catch.

`ledge_roll_hit_window_frames` defaults to **5**, planning contact during attack
frames **7–11**. Set 1–13 to extend or shorten that window through frame 19.
Live extra input-delay compensation is zero. Unknown protection still waits for observed
vulnerability; these settings do not change actual hitboxes or guarantee
opponent-relative frame-perfect timing online.

## Timing and limits

Saved sequences start at the first eligible observation of the recognized option,
then deliver the configured inputs and waits. Move startup is additional native
move time, not a hidden sequence delay. There is no learned online-delay
compensation or automatic retry behind literal attack inputs.

Only explicit spatial/check/wait-for-block actions add their documented waits.
Missing targets, unsafe movement or invalid scope can end a sequence. Inputs
cannot bypass shieldstun, landing lag, recovery or native action restrictions.
Timing perfected in Training is not proof of identical online input acceptance.
Use [input timing](SEQUENCE_FILES.md), [fighter references](OPPONENT_DATA.md) and
[diagnostic observations](OBSERVATIONS_AND_TIMING.md).
