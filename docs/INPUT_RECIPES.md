# Shared move inputs and Kirby posture

This page describes compiled helpers used by existing assists. Saved text actions
are literal input timelines and do not inherit these helper acceptance checks or
hidden waits. Use the [editor action reference](EDITOR_ACTION_CHEATSHEET.md) for
current sequence behavior and [controls](CONTROLS.md) for active bindings.

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

The table describes independent native-confirmed helpers such as manual finishers
and onstage roll reads. Ledge trapping uses [literal file timelines](SEQUENCE_FILES.md)
instead: no acceptance retry, native JumpMini marker, fighter-specific drift window
or inferred startup delay is added to its inputs.

This assumes **A = Attack, X and ZL = Jump, R = Grab, ZR = Shield** in the selected
Smash controller profile. Manual X short hop deliberately uses X alone and releases
it in squat. A shared recipe cannot correct a different game mapping, a closed
transition term, an OS overlay consuming the controller, or an airborne wrong move.
Physical C-stick inputs are suppressed while an assisted sequence owns them.
Nothing promises frame-perfect arena execution despite stalls or packet delay.

## Grounded posture menu rows

Idle crouch (Kirby) and Dtilt face (Kirby/Yoshi) start **ON**, save per fighter in memory, and can be changed with the
normal menu controls. No new config-file keys or startup loads are needed.

- **Idle crouch:** When grounded in ordinary movement and physical buttons/stick
  are neutral and an enemy is within **45 horizontal units and 20 vertical units**,
  propose Down immediately. No enemy or leaving that range removes the proposal.
  This deliberately generous approach distance starts crouching early. Holding L
  suspends crouching; releasing L resumes it
  without changing the saved toggle. Any physical movement, attack,
  jump, shield or other button immediately removes that proposal. No action
  buttons are owned. Passable platforms are excluded to prevent platform drops.
  Roll read, ledge responses, safety and every committed action have priority.
  Down stays held through native crouch entry/wait/turn/rise states (0x11–0x15)
  and crouch-animation loop resets; entering crouch must never release it.
- **Dtilt face (Kirby/Yoshi):** While holding Down, prepare the angle before pressing A. A physical down C-stick also qualifies while standing with a neutral left stick; configure that stick as Tilt Attack in the game. The live helper reads the pre-map right-stick axes immediately instead of waiting for a digital Down alias. Keep Down fully held and
  angle the stick 0.80 toward the nearest enemy if facing away. Already facing
  the enemy uses straight Down. Attack presses/releases remain physical, so
  mashed down tilts keep their original edges. For an actual down C-stick flick,
  the winning helper also angles that stick toward the enemy, keeping Down the
  dominant axis; this avoids leaving a contradictory straight-down C-stick
  direction for the mapper. Side/up flicks and released C-stick axes are not
  converted into down tilts. Both stick values are restored after mapping.
  Horizontal native direction bits
  pass through instead of being suppressed by the walking filter. A physical
  down-tilt press and native down-tilt recovery use manual-tool priority over ledge preparation, while safety
  and confirmed finishers retain priority. This also runs during native
  down-tilt recovery (0x2c) to aim the next buffered tilt, using the latest enemy
  direction each sample. There is no standing-turn sequence, queued attack or
  eight-frame timer. Releasing Down removes the adjustment; during down-tilt
  recovery the angle remains through A-release gaps to prepare the next tilt.
  Pre-attack aiming stays passive so committed automatic punishes still win.
  It uses a bounded pass over at most eight already-known fighter IDs only while
  enabled and Down intent or native down-tilt recovery is present, including when other opponent-reading assists are off. No item/hitbox
  scan is added. In a free-for-all, nearest means nearest
  root position, not the opponent you intend to target.
  Direction is proposed in the same poll with no added delay. Native input
  acceptance can still choose the old facing when turn and attack arrive
  together; input-only behavior cannot guarantee a reversal before every tilt.

The A+B jab-to-grab shortcut and its menu row are removed. A/B now follow
normal game behavior. Use universal X/Y input recipes for your own named button
sequences; these do not inherit native jab-confirmation or recovery checks.

Dtilt facing retains its angle during native down-tilt hitstop, allowing the next
physical tilt to buffer with its direction intact. Down on a C-stick mapped to
Tilt Attack is permitted alongside held left-stick Down. Left-stick-only aiming
preserves the physical C-stick attack edge while aiming its down angle; full assisted
sequences still suppress competing physical C-stick input. Neither A nor a
C-stick attack is generated by this helper.

Startup-loaded file choices now use [literal input timelines](SEQUENCE_FILES.md).
Their general move names compile to controller inputs and do not use these
native-confirmed compiled helpers; there is no `code:ID` fallback.
