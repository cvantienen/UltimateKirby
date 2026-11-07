# Auto tech input scheduling

Auto-tech is available in silent and diagnostic builds as an ON/OFF menu row
for every fighter. It starts ON
with the supplied config. On the Modes page, Up/Down selects the row and
Left/Right toggles it.
Selections last for the current match; `auto_tech` supplies
the initial value (an explicit false is respected). OFF skips collision prediction entirely. ON only predicts
in eligible damage/tumble states, and stops probing once it finds minimum-time
contact. Linear refinement rebuilds only the ray being shortened. Descending
floor prediction can use the bounded acceleration adjustment described below;
wall/ceiling rays and linear fallback retain their existing geometry.
No cross-frame cache or reduced prediction frequency is introduced. These
shortcuts preserve the existing prediction result; they do not establish the
cause of a reported whole-match FPS drop.

`auto_tech = true` applies to the selected human controller in every supported
battle mode. The configured layout's simulated guard button must map to Shield
(ZR for Default, ZL for Brother). The implementation supplies controller
inputs; it does not force tech status or override native lockout/untechable rules.

The predictor reads **ALL kinetic reserves**, including damage knockback.
During the final eligible damage-hitstop frames it uses a finite, nonzero native
reserved launch vector instead of frozen/old kinetic motion. A zero or invalid
reservation falls back to the ordinary velocity. DI and tech share those two
read-only values within the same observation; no persistent launch cache is kept.
The previous MAIN-only query could miss the velocity driving a stage collision.
Swept floor, wall and ceiling body probes estimate the first contact time by
bounded ray refinement. The one-frame contact ray is checked before the longer
sweep, because a long diagonal can miss a narrow platform that the immediate
ray reaches. Genuine slow motion down to a small numerical-rest tolerance
(0.0001 units/frame) is eligible for floor, wall and ceiling prediction; the old
0.1 threshold could skip slow contacts. Only descending floor probes include pass-through
platforms; wall and ceiling probes exclude them to avoid false early predictions.
The adapter withholds automatic guard when native damage-recovery flags and the
air-escape transition indicate it could become an air dodge. The scheduler also
cancels stale contact requests outside supported tumble/capture statuses.
`tech_lookahead_frames` remains a base horizon of 1–5;
the live adapter supplies zero extra input delay, so it adds no learned-delay
extension to that horizon. Explicit core callers can still supply a delay.

For an eligible damage-air/tumble state, the live scheduler compares contact
time with the configured native lead and remaining hitstop. Live extra input
delay is zero; no per-action samples or medians are collected. The final five
known damage-hitstop frames can run the existing bounded collision forecast
(previously only the final two). This gives the mapper more lead for a floor
impact immediately after a spike. Remaining stop time is included in the forecast;
the scheduler still requires contact inside its existing arrival window.
Early/global stop, invalid prediction and the native passive-disabled flag do
not generate a tech request. A valid damage-hitstop timer can supply timing even
when the motion clock is frozen or not yet available.

## Descending floor/platform adjustment

The linear predictor can miss an approaching floor within its lookahead when
current descent is slow but still accelerating. Eligible, non-stopped tumble
observations now retain one copied velocity/motion sample. Matching native
motion progress over 0.5–2 frames can establish a small downward acceleration
trend (0.005–0.3 world units/frame squared). No gravity/velocity is written.

With that evidence, a descending fighter (including zero current vertical speed
or a slow rise that turns downward within the horizon) uses up to five one-frame floor sweeps
at the three existing horizontal body offsets: at most 15 floor rays, stopping
at the first predicted contact. This covers floors and one-way platforms along
the estimated curved descent. It is limited to the existing live horizon of at
most five frames; no horizon extension or new stage/object scan is added.
Non-hit floor queries can increase from six linear-path queries to fifteen short rays
in this branch, though the old worst-case refined floor path used eighteen.

Missing/invalid clocks, identity/status/motion/rate changes, large sample gaps,
velocity discontinuities, hitstop and unavailable trends use the original linear
prediction. History resets with session cancellation or disabled tech. Ascending
segments do not get floor sweeps: an estimated apex is skipped, and only the
subsequent descending part can collide with one-way platforms. Wall/ceiling geometry, eligibility, air-dodge
suppression, input lead and mapped-edge acknowledgment are unchanged. The
scheduler now records the configured human-shield cooldown before rejecting an
unknown initial motion clock, so an early human shield press cannot accidentally
use a two-frame fallback instead of the configured lockout.
This addresses a demonstrable predictor gap, not proof of the cause of every
reported miss; native collision geometry and changing motion remain estimates.

## Ganondorf Side B

Grounded Flame Choke uses victim status `CATCHED_GANON` (`0xbe`), which the
ordinary airborne-tumble detector excluded. The adapter now resolves the actual
capture-linked fighter without depending on Arena player slots or the ledge
roster. It verifies Ganondorf's grounded catch state, then predicts the throw
from the linked captor's native motion frame/rate. The vanilla
[grounded throw script](https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/main/smashline/lua2cpp_ganon/ganon/SpecialS.txt)
applies its throw at motion frame 31. That is a move-specific forecast, not an
assumption that the victim has normal downward knockback.

The linked clock also drives scheduler progress when the victim's capture pose
is frozen. The shield request targets four frames before predicted impact after
any explicit core delay (zero in the live adapter); ordinary tumble retains its existing nine-frame lead.
Held shields still require release and a fresh mapped press. Invalid/missing
capture links or clocks do not fabricate a throw deadline. This timing assumes
the vanilla move script and still requires on-device verification.

The release status `CATCHED_AIR_END_GANON` (`0xc1`) is shared by grounded
and aerial versions. The previous implementation incorrectly rejected it in both
cases. One read of the native previous status now identifies release directly
from grounded `CATCHED_GANON`; only that verified branch can request shield for
immediate contact after any known damage stop. It can finish delivering an
unmapped capture request, or request one when the earlier capture forecast was
unavailable. An already acknowledged press retains its cooldown and is not
repeated. An unknown stopped clock withholds input. This uses no stage rays or
roster scan. See the [Training Modpack's grounded-origin check](https://github.com/jugeeya/UltimateTrainingModpack/blob/main/src/training/tech.rs).

Aerial capture/fall (`0xbf`, `0xc0`) and release with an aerial/unknown predecessor
remain excluded. Aerial Flame Choke produces an untechable knockdown.
Shield inputs cannot convert it into a legal tech.

## Input ownership and confirmation

With `takeover = true`, an already held mapped shield is released before a fresh
guard press. Tech owns only the configured layout's shield and grab inputs
(native grab also produces Guard), including their digital trigger aliases.
Jump, attack, special, C-stick and left-stick DI remain available. Pause/menu
inputs also remain available. Additive mode cannot force a held shield release.

Preparation begins during known eligible damage hitstop, or within two frames
of the existing predicted-contact press window. A missing or distant collision
forecast no longer reserves controls throughout a fall. Native tech eligibility
and air-escape safety gates still apply; preparation releases Guard without
pressing it early.

A proposal does not consume cooldown. The scheduler waits for the original
mapper to produce a fresh GUARD edge before committing the configured cooldown.
Unmapped requests remain pending only while the current collision is still
predicted. Fresh human shield presses in tumble also update cooldown tracking.

Cooldown advances from rate-normalized native motion progress, excludes hitstop,
and is conservative across animation changes or missing observations. An observed
native tech status clears the artificial cooldown for the next collision. While
the same collision remains imminent, takeover keeps shield/grab aliases released.
If that forecast disappears, general control returns immediately. A shield/grab
button held through the takeover stays suppressed until release and a fresh
press, preventing an old hold from manufacturing another Guard edge. Other
buttons are unaffected by this handback.

Built-in jump bindings remain bufferable during damage/tech safety. A saved
unconditional one-frame jump-only recipe also forwards its jump during damage
recovery instead of attempting a new exclusive tape. Ordinary committed sequences
still own their complete authored inputs and block competing jump bindings.
Damage/capture cancels a tape without an additional all-button neutral packet;
new auto-reactions do not start while the fighter is incapacitated.

These changes reuse existing observations, collision predictions and the existing
held-guard handback. No new scans, hooks, polling, geometry or physics changes are
added. Host regressions verify ownership and delivery; online hardware testing
is still needed to assess remaining missed techs.

The live adapter uses `tech_input.rs` for this acknowledgment-based scheduling;
`RawAutoControls` supplies analog mash independently. Existing pure-Rust legacy
tech examples/tests do not define the live timing path.

With diagnostics and `tech_debug = true`, compare:

- `collision_predicted`: forecast contact, stop time, delay and proposal.
- `tech ZR applied`: fresh raw ZR supplied to the mapper.
- `tech request timing`: current status, motion clock, contact estimate and input
  delay at that press; grounded Flame Choke uses the linked captor's clock.
- `ganon_ground_capture_timing_unavailable`: grounded capture was seen, but a
  verified linked captor/usable pre-impact clock was unavailable.
- `ganon_aerial_slam_untechable`: aerial capture/slam is deliberately excluded.
- `tech mapped: fresh GUARD edge confirmed`: mapped shield input acknowledgment.
- `tech_status_observed`: actual native tech state; a proposal alone is not success.
- `air_dodge_available`: native recovery permits air escape; automatic guard withheld.

Collision rays still approximate body geometry and future movement. SDI, changing
knockback, passable/moving/sloped surfaces and online delay jitter can invalidate
a forecast. Untechable impacts and native lockout remain untechable. Host tests
and compiler checks do not establish a zero-miss result on a Switch.

Grounded Flame Choke and shield-handback regressions have been added as source
cases. A host test cannot validate the native capture link or prove a zero-miss
result online. Source edits do not update an installed NRO; a separately
requested build/install and diagnostic console test are still needed.

## Training Modpack comparison

Training Modpack's CPU auto-tech substitutes a passive/tech status at an engine
collision transition after checking native eligibility. It does not predict a
shield press. That method cannot be copied while keeping this project input-only.
The [runtime safety guide](RUNTIME_SAFETY.md) documents the current identity,
freshness, cancellation, and input-only constraints.

Native gates, timing windows and mapper acknowledgment remain unchanged.
Silent skips rejected-state velocity reads used only by diagnostics. The
descending-floor adjustment above uses the same native velocity and shared motion
reads; wall and ceiling prediction retains its existing rays.

The immediate-ray-first ordering retains the maximum refinement/query budget and
can finish sooner on an imminent hit. A non-hit active probe can use an extra
short query. No extra fighter scans, hooks or per-frame allocations were added.
Host regressions cover slow floor/wall/ceiling approaches, narrow platforms,
observed acceleration from zero descent and initial human-shield cooldown.

Successful native tech statuses clear the scheduler's old cooldown even when the
new animation clock is temporarily unavailable. An unacknowledged proposal also
rechecks mapped Guard: if it became held, takeover releases it before requesting
a fresh press. This prevents a stale request from holding Guard without a new
edge. Neither case changes native eligibility, lockout duration, air-dodge gates,
collision queries or successful-delivery acknowledgment.

A native-verified safe contact can request a tech even when its animation clock
is frozen or temporarily unavailable. This requires the adapter's existing
tech-status, passive-disabled and air-escape checks; a raw unknown animation
alone never authorizes input. Without an advancing valid clock the scheduler
preserves cooldown rather than counting controller polls. No new stage rays,
body-size guesses or native writes are added by this delivery correction.
