# Shared move inputs and Kirby posture

`crates/kirby_assist_core/src/move_input.rs` names physical input recipes.
They pass through the existing single mapper overlay and action owner. They
do not force moves, change animation speed or add an input-delay estimate.

| Action | Request and acceptance rule |
| --- | --- |
| Neutral short-hop attack | Neutral X+A, release Jump at squat; confirm exact nair motion |
| Directional short-hop aerial | Neutral X+ZL, release both at squat, directional A after native takeoff |
| Full hop | Hold X through squat, select aerial only after takeoff |
| Fair / bair | Stick toward / opposite native facing plus A; never a grounded directional A |
| Turnaround bair | Grounded stick-only turnaround, confirm facing away, then short hop and airborne backward+A |
| Fast fall | Down after exact move acceptance and negative native vertical speed; recipes choose the earliest permitted move frame |
| Smash | Neutral/release primer, direction+A, release A on native startup; rejection retries stay bounded |

`OneShot` variants, manual finishers and onstage roll reads share these helpers.
The existing timed-nair and fighter-specific sequences retain their own state
machines and schedules. Adding a helper does not replace native acceptance
checks. Known working nair pairing remains neutral X+A. C-stick attack-cancel is
not used for Kirby bair: it previously produced grounded tilts. His Jump and
Neutral bair now share the stick-only confirmed turnaround and airborne selection.
Already back-facing Kirby skips the turn. Existing per-option drift is retained.

This assumes **A = Attack, X and ZL = Jump, R = Grab, ZR = Shield** in the selected
Smash controller profile. Manual X short hop deliberately uses X alone and releases
it in squat. A shared recipe cannot correct a different game mapping, a closed
transition term, an OS overlay consuming the controller, or an airborne wrong move.
Physical C-stick inputs are suppressed while an assisted sequence owns them.
Nothing promises frame-perfect arena execution despite stalls or packet delay.

## Kirby-only menu rows

These Kirby rows start **ON**, save per fighter in memory, and can be changed with the
normal menu controls. No new config-file keys or startup loads are needed.

- **Idle crouch:** When grounded in ordinary movement and physical buttons/stick
  are neutral, propose Down. Holding L suspends crouching; releasing L resumes it
  without changing the saved toggle. Any physical movement, attack,
  jump, shield or other button immediately removes that proposal. No action
  buttons are owned. Passable platforms are excluded to prevent platform drops.
  Roll read, ledge responses, safety and every committed action have priority.
  Down stays held through native crouch entry/wait/turn/rise states (0x11–0x15)
  and crouch-animation loop resets; entering crouch must never release it.
- **Dtilt face:** While holding Down, prepare the angle before pressing A. Keep Down fully held and
  angle the stick 0.80 toward the nearest enemy if facing away. Already facing
  the enemy uses straight Down. Attack presses/releases remain physical, so
  mashed down tilts keep their original edges. Horizontal native direction bits
  pass through instead of being suppressed by the walking filter. A physical
  down-tilt press and native down-tilt recovery use manual-tool priority over ledge preparation, while safety
  and confirmed finishers retain priority. This also runs during native
  down-tilt recovery (0x2c) to aim the next buffered tilt, using the latest enemy
  direction each sample. There is no standing-turn sequence, queued attack or
  eight-frame timer. Releasing Down removes the adjustment; during down-tilt
  recovery the angle remains through A-release gaps to prepare the next tilt.
  Pre-attack aiming stays passive so committed automatic punishes still win.
  It uses a bounded pass over at most eight already-known fighter IDs only while
  enabled, including when other opponent-reading assists are off. No item/hitbox
  scan is added. In a free-for-all, nearest means nearest
  root position, not the opponent you intend to target.

- **A+B grab:** Tap A+B together with a neutral stick to commit one jab 1 into
  an ordinary standing grab. A alone (tapped, held or mashed) stays normal.
  B is consumed immediately so the game's A+B Smash setting does not receive
  the chord. Starting crouched/dashing first releases the stick and waits for
  an ordinary standing/walking state, avoiding down tilt/dash attack. Native
  jab 1 acceptance releases A even in hitstop; R is not sent into an ongoing
  jab, even if its catch transition term is enabled. Grab waits for ordinary
  recovery and native catch readiness. Releasing A+B does not cancel the attempt;
  movement or other buttons do. One attempt per chord; release both before
  retrying. Grab startup returns movement/control while suppressing any still-held
  original A/B until release, preventing automatic pummels/specials. Jab/grab
  acceptance budgets are 24 advancing frames, with a 90-frame total bound.
  This does not bypass recovery or guarantee a true combo; spacing remains yours.
  The existing menu toggle stays ON by default. No new hook or hitbox scan.

These are posture helpers, separate from the disabled incoming-hitbox auto-duck.
To check them, enable each row separately and test movement/attack handback first.
Then test both facing directions and roll-read priority. Host regressions cover
these rules; Switch calibration is still necessary.

Dtilt facing retains its angle during native down-tilt hitstop, allowing the next
physical tilt to buffer with its direction intact. Down on a C-stick mapped to
Tilt Attack is permitted alongside held left-stick Down. Left-stick-only aiming
preserves the physical C-stick rather than erasing its attack; full assisted
sequences still suppress competing physical C-stick input. Neither A nor a
C-stick attack is generated by this helper.
