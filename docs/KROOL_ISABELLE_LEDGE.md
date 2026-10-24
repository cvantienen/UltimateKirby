# King K. Rool and Isabelle

Both have independent compiled profiles, native move data and five saved lists.
Controls: [L autopilot and R3 menu](LEDGE_MENU.md). Y is reserved for configured
held modes; B-reverse is currently unbound.
Bold entries below are defaults.

| Row | King K. Rool | Isabelle |
| --- | --- | --- |
| Neutral | **Nair**, down tilt, forward tilt, belly flop | **Nair**, down tilt, forward tilt, party popper, slingshot |
| Jump | **Nair**, up-air, forward-air | **Nair**, up-air, slingshot |
| Roll | Down-smash, **grab** | **Down-smash**, net grab, slingshot |
| Attack | **Shield → nair**, grab / rear nair, shield → up-air | **Shield → nair**, grab / rear nair, shield → up-air, shield → slingshot |
| Drop | **Shield**, down tilt → shield, spike, belly flop | **Shield**, down tilt → shield, turnip spike, slingshot |

Nair defaults have continuous active spans: K. Rool 7–30, Isabelle 5–25. Normal
get-up coverage targets eight frames before protection expiry using the existing
numeric adjustment. K. Rool's roll grab starts sooner than his frame-22 smash.
Isabelle's front/rear down-smash windows are separately 8–9 and 12–13; their gap
is never counted as coverage. Successful grabs return control for pummels/throws.

The playful grounded choices are K. Rool's down-smash body hit (22–24; the later
shockwave is excluded) and Isabelle's forward-smash party popper (14–19).
Extra jump choices use K. Rool up-air 7–19/fair 11–17, or Isabelle up-air 6–31.
Her slingshot selects fair (first projectile phase 10–12) when the target is
in front, or bair (13–15) when behind. Travel time remains spacing dependent. Spike choices use down-air: K. Rool 14–21, Isabelle 11–28.
Sources: [K. Rool measurements](https://ultimateframedata.com/king_k_rool),
[Isabelle measurements](https://ultimateframedata.com/isabelle).

Nair is the forgiving general preset; up-air aims above the fighter and fair
needs facing/spacing. These are coverage candidates, not proven optimal against
every opponent. Airborne height, shield pushback, platform geometry and online
input-delay changes can still defeat a correctly scheduled animation. The short
smash windows tolerate less timing error than nair.

Extra aerials pair X+A, release X in jump squat (including frozen observations),
and hold directional A until the exact motion starts. They then release attack
inputs, avoid fast-fall, and hand back after the chosen active window or landing.
Drop spikes drift outward and do not recover automatically: use them selectively.
Grounded smash variants prime a neutral release, pair direction+A, then hand back
on native startup to avoid charging. All extra choices make only one move;
interruptions, wrong moves, invalid clocks, timeout and releasing L cancel them.

The same cached UI/config selection and active-tool observation gates are reused.
There is no combo engine, extra roster scan, file loading, dependency, network or
service call. Inactive profiles execute no response. Formatting/static source
checks and regression fixtures were added; no build, Rust tests, upload or device
performance/online measurements were run for this update.

## Isabelle slingshot in all five lists

Neutral, Jump, Roll, Get-up Attack, and Drop each have a Slingshot choice.
Existing choice indices and defaults are preserved. Neutral and Roll short-hop
in place and stage projectile emission against vulnerability using the chosen
fair/bair startup. Roll aims toward the usual ledge-relative roll endpoint,
without running there. Jump retains her native adaptive-hop policy. Get-up
Attack holds shield until a confirmed block and native jump readiness; Drop
requests an immediate short-hop shot. All shots use zero post-emission drift.

The chosen facing selects fair or bair without a turnaround, and bair starts
with a neutral hop to preserve that facing. OneShot continues through lost
opponent tracking and stops ownership after the first projectile phase; the
native pellet continues flying. Projectile speed, travel, and collision are not
modeled, so emission timing does not guarantee contact before the enemy can act.
These options require spacing and Switch validation. No new scans or runtime
config loading are added.
