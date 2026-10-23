# Kirby and Yoshi

These notes describe the current source changes under package version **1.5.3**.
They include edits made after the supplied diagnostic log; an older 1.5.3 NRO
does not necessarily include them. Rebuild/install separately before testing.
See [controls](CONTROLS.md) and [fighter execution](FIGHTER_EXECUTION.md).

## Saved choices

Default choices are first in each cell.

| Row | Kirby | Yoshi |
| --- | --- | --- |
| Neutral | Nair; down tilt; forward tilt; Stone; Inhale; bair | Nair; down tilt; forward tilt; up tilt; headbutt f-smash |
| Jump | Nair; bair | Nair; up-air; meteor fair |
| Roll | Down-smash; grab | Down-smash; tongue grab |
| Attack | Grab / rear nair; grab / rear shield | Shield → nair; grab / rear nair; shield → up-air |
| Drop | Shield; down tilt → shield; short-hop dair | Shield; down tilt → shield; meteor fair; dair chase |

## Neutral and Jump nair

Both fighters use a **0.25 horizontal stick nudge toward the opponent**, only
after exact native nair acceptance and while aerial animation frame is below
**4**. The stick then goes neutral. This is a stick magnitude, not 25% physical
air speed; momentum and native air acceleration still determine travel. Startup
inputs remain neutral so the nudge does not select fair or bair.

| Response | Current timing |
| --- | --- |
| Kirby Neutral nair | Configured lead plus 2 frames: supplied 8 becomes 10 frames before protection expiry |
| Kirby Jump nair | Protection forecast permits 2 frames earlier; an immediate launch cue is still immediate |
| Yoshi Neutral nair | 17-frame lead before protection expiry, previously 15 |
| Yoshi Jump nair | Immediate short-hop nair on recognition; no extra wait to remove |

Native nair activity is unchanged: Kirby **8–32**, Yoshi **3–25**. The four-frame
nudge can occur during aerial startup; it does not extend active hitboxes.
Late recognition, unavailable jump readiness or landing can shorten coverage.

## Kirby bair and drop dair

Jump bair uses **0.35** forward stick after native bair acceptance. Neutral bair
uses a short hop, **zero drift**, and a three-frame early scheduling lead.
Already back-facing uses a direct bair route; forward-facing uses backward
C-stick, mapped primer acknowledgment and paired X+ZL jump, with forward stick.
The native `attack_air_b` motion confirms the result. Use Tilt Attack C-stick
and both jump mappings; an attack request alone does not prove bair started.

Drop dair immediately attempts an outward short-hop spike with paired jump+A.
It does not automatically recover. Stone and Inhale return control on native
special startup; captured opponents are not automatically pummeled or thrown.

## Roll grabs

The supplied log showed both delivered Grab inputs and native grab startup on
misses. One Kirby miss requested around enemy roll frame **31.9**, versus
**23.1** on a nearby successful trial. Repeated braking outside the navigation
area delayed arrival; the common endpoint also differed from actual root travel.

Current Kirby/Yoshi approach keeps **0.65 walking stick** until inside a usable
area, braking to avoid crossing the opponent rather than braking outside the
area. The initial common point can extend inward from observed roll movement,
using at most six frames of navigation forecast at zero extra live delay. It
does not chase back toward the ledge. Already useful positions can request
without reaching an arbitrary standing point.

Shared grab requests check predicted range, facing, height and vulnerability on
the **same active frame**. Timing includes Kirby's two active grab frames and
Yoshi's eight. Native catch readiness, a movement-release sample and bounded
input delivery still apply. Capture hands control back. Down-smash navigation
is unchanged; Yoshi's down-smash target remains two units toward the ledge from
the configured point.

This remains a root-based forecast, not animated grab/hurtbox collision. The
latest changes have formatting/source checks and regression fixtures; those
fixtures have not been run and these edits have not been Switch-validated.
