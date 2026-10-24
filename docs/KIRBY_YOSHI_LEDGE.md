# Kirby and Yoshi

These notes describe the current source changes under package version **1.5.3**.
They include edits made after the supplied diagnostic log; an older 1.5.3 NRO
does not necessarily include them. Rebuild/install separately before testing.
See [controls](CONTROLS.md) and [fighter execution](FIGHTER_EXECUTION.md).

## Saved choices

Default choices are first in each cell.

| Row | Kirby | Yoshi |
| --- | --- | --- |
| Neutral | Nair; down tilt; forward tilt; Stone; Inhale; bair | Nair; down tilt; forward tilt; up tilt; headbutt f-smash; bair (hit 2) |
| Jump | Nair; bair | Nair; up-air; meteor fair |
| Roll | Down-smash; grab | Down-smash; tongue grab |
| Attack | Grab / rear nair; grab / rear shield | Shield → nair; grab / rear nair; shield → up-air |
| Drop | Shield; down tilt → shield; short-hop dair | Shield; down tilt → shield; meteor fair; dair chase |

## Neutral and Jump nair

Yoshi uses **full 1.0 horizontal stick toward the opponent**, only after exact
native nair acceptance and while aerial animation frame is below **2**, for both
Neutral and Jump nair. Kirby Neutral nair keeps its 0.25 nudge below frame 4;
Kirby Jump nair uses full 1.0 forward stick after acceptance through frame 32,
then neutral at frame 33 or landing. Its launch timing remains unchanged.
The stick then goes neutral. Stick magnitude is not physical air speed;
momentum and native air acceleration still determine travel. Startup
inputs remain neutral so the nudge does not select fair or bair.

| Response | Current timing |
| --- | --- |
| Kirby Neutral nair | Configured lead plus 2 frames: supplied 8 becomes 10 frames before protection expiry |
| Kirby Jump nair | Protection forecast permits 2 frames earlier; an immediate launch cue is still immediate |
| Yoshi Neutral nair | Immediate short hop/drift with X+ZL; delay neutral A to target first activity 16 frames before predicted vulnerability |
| Yoshi Jump nair | Face ledge, confirm native facing, then immediate short-hop nair |

Native nair activity is unchanged: Kirby **8–32**, Yoshi **3–25**. The four-frame
nudge can occur during aerial startup; it does not extend active hitboxes.
Late recognition, unavailable jump readiness or landing can shorten coverage.

Yoshi Neutral nair first faces the remembered ledge, confirms native facing,
then immediately sends X+ZL without A, with a brief full-stick forward nudge.
Both buttons must map to Jump and release in squat for a short hop. Takeoff
ignores spacing and protection timing. Neutral A waits
`max(0, protection remaining - 3 startup - 16 lead)` native game frames from
takeoff request. Duplicate polls/hitstop do not spend the wait. Unknown
protection withholds A; landing/interruption cancels rather than jumping again.
This snapshot schedule cannot guarantee coverage if the short hop lands before
the desired attack. A must map to Attack. Confirmed descending
nair fast-falls from animation frame 3; its full-stick nudge ends at frame 2
or landing/interruption. Jump nair does not gain the fast fall.


## Kirby bair and drop dair

Jump bair uses **0.35** forward stick after native bair acceptance. Neutral bair
uses a short hop, **zero drift**, and a three-frame early scheduling lead.
Neutral bair turns away immediately on recognition, confirms native facing,
then launches with neutral X+ZL, releases Jump in squat and selects backward+A after takeoff. It never
requests a grounded tilt or C-stick attack. Already back-facing skips the turn.
X/ZL must map to Jump and A to Attack. Jump bair retains its paired X+ZL route.
Native `attack_air_b` confirms acceptance; hardware testing is still needed.

Kirby Drop dair launches with neutral X only, then releases Jump and buffers
down+A during native squat/takeoff, following Yoshi's directional aerial input
pattern. This avoids selecting nair in the initial Jump request.
It does not automatically recover. Stone and Inhale return control on native
special startup; captured opponents are not automatically pummeled or thrown.

## Roll grabs

The supplied log showed both delivered Grab inputs and native grab startup on
misses. One Kirby miss requested around enemy roll frame **31.9**, versus
**23.1** on a nearby successful trial. Repeated braking outside the navigation
area delayed arrival; the common endpoint also differed from actual root travel.

Kirby's experimental Roll → Grab runs immediately to a fixed **20-unit inset
from the witnessed ledge**, brakes with native stopping-distance checks and uses
a 0.3 walking correction near arrival. He stops within one unit, faces center
stage and waits for the timed catch. Arrival is required; an earlier local catch
does not interrupt the run. There is no opponent-based destination extension or
grab standoff subtraction. Yoshi has returned to the shared tracked 0.65-stick
walking approach, which may stop for a legal timed catch before its destination.

Shared grab requests check predicted range, facing, height and vulnerability on
the **same active frame**. Timing includes Kirby's two active grab frames and
Yoshi's eight. Native catch readiness, a movement-release sample and bounded
input delivery still apply. Capture hands control back. Down-smash navigation
is unchanged; Yoshi's down-smash target remains two units toward the ledge from
the configured point.

This remains a root-based forecast, not animated grab/hurtbox collision. The
fixed-inset change has passing host regression fixtures for both ledges, but
has not been Switch-validated.


## Yoshi Neutral dair

Neutral Dair reacts immediately without a vulnerability-time or spacing gate.
It faces the remembered ledge, confirms native facing, sends neutral X+ZL and
releases both in squat before selecting down+A after takeoff. Drift begins only
after exact dair acceptance, using full forward stick until animation frame 30
or landing/interruption. No automatic fast fall is added for dair. Its native
multihits and gaps remain unchanged; drifting does not extend hitbox activity.
Neutral Bair, Forward tilt and Up tilt have been removed from Yoshi's catalog.

Yoshi Neutral nair enforces Short at execution, even if a stale profile setting
requests Full. It uses neutral X+A without injecting ZL, releases X in native
jump squat and retains its drift and descending fast fall without a timing wait.
Map X to Jump and A to Attack. The appended Neutral **Down smash** choice uses
Yoshi's front activity at frames 7–8, with a neutral smash primer before Down+A
and attack release on native startup. It does not move Yoshi into range.

Kirby Neutral down smash also uses the existing wait for a 0.65-stick walk
toward the six-unit ledge inset. Attack preparation at 16 frames and Down+A
at 15 frames before vulnerability take priority over arrival or range. This is
two frames earlier than the former midpoint target, aiming for protection expiry
at move frame 15 inside activity 7–19. Jump
nair uses the same grounded fast-walk helper only while its existing gate waits.
