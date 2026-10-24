# Explicit action ownership

One final selector in the mapper decides which feature owns automated buttons
and sticks. Its state lives in core action_owner.rs. It sits before the existing
single ManualOverlay edge history; it neither adds an input hook nor inserts
another neutral/primer frame. Every live input path participates: ledge/recovery,
roll reads, trip/fair confirms, Y finishers, shielding, slides, trump, X shortcut,
turbo and native safety responses (mash/tech/SDI).

## Interruption rules

| Situation | Rule |
| --- | --- |
| Committed sequence, including neutral release samples | Retains ownership while its proposal remains present/committed |
| Passive shielding, walking or turbo | Cannot replace a committed punish |
| Mash, tech or SDI safety response | Can interrupt ordinary action sequences |
| Recognized trip follow-up | Can take over for its confirmed forward-smash follow-up |
| Automatic fair confirmation | Can replace automatic ledge/roll/preparation; does not steal a manual Y route |
| Explicit Y, slide, trump or X tool | Can interrupt automatic ledge/roll/preparation; cannot replace a committed hit-confirm follow-up |
| Completion, interruption, expired observation or invalid scope | Releases ownership; queued losing actions are canceled |

A passive ledge setup may yield to a feasible roll punish. A committed ledge
response cannot. Ledge shielding/recovery setup also checks ownership before
arming a competing proposal. Manual Y has its own proposal, separate from the
automatic fair follow-up, so the same aerial cannot leave two finishers queued.
The C-stick turnaround overlay is only available to the selected ledge owner.

Cancellation preserves physical edge history for Y, Down and X. A losing held
button cannot silently restart its canceled sequence. Roll cancellation retains
the used-roll marker. Unconfirmed hit watchers are not marked spent merely
because no action owns that observation. Native actionability, session fences,
input freshness and existing move timings remain in force; this does not cancel
a native move already running in the game.

Selection uses a fixed eleven-intent stack array and enum comparisons, without
allocation, opponent scans, logging, networking or delay estimation. Hardware
cost and actual online inputs still require Switch testing.

## Verification and extensions

Run core integration tests first, then:

\x60\x60\x60sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --test action_ownership --offline
python3 tools/check_action_ownership.py
\x60\x60\x60

The script host-compiles/tests the actual selection and cancel_non_owner blocks
extracted from live.rs. It catches passive shield takeover, discarded stick-only
preparation, accidentally spent hit watchers, wrong trip direction and held-Y
restarts after cancellation. It does not build a Switch plugin.

New features must publish a separate proposal, define commitment/interruption
rules and cancel losing plans through this same selector. Do not add a second
button overlay or send independent queued inputs after ownership is resolved.

Shortcut proposals reach the final selector directly; preliminary assignments
to the shared output are unnecessary and are overwritten by that selection.

Kirby's optional posture proposal has the lowest priority (5) and never locks
ownership. Idle crouch owns only the stick. Every roll punish, ledge response,
manual tool and safety action wins over it. Dtilt facing only adjusts the stick
for the current physical attack input; there is no queued turnaround to resume.
Physical Dtilt-facing presses use the existing Shortcut owner so ledge preparation
cannot replace their aim; idle/recovery-only posture remains lowest priority.

Kirby **A+B grab** uses the existing Shortcut owner from the neutral chord
through jab release/recovery and grab delivery. B is consumed before mapping,
preventing A+B Smash. A alone stays physical. Native jab 1 startup releases A;
ordinary jab recovery and catch readiness gate Grab. Releasing the chord does
not cancel; directional input or another button does. Safety and confirmed
follow-ups retain priority. After catch startup, only original held A/B are
suppressed until release to prevent automatic pummels or specials.

Yoshi down-tilt chase uses **JabLock**, priority 75. It can interrupt background
RollRead/Ledge/Defense/Turbo after an attributed down-tilt hit. Once committed,
background rolls cannot steal its movement or jab-release frames. Safety,
confirmed follow-ups and explicit manual tools retain interruption rights.
Losing a committed chase cancels its target instead of queueing a later attack.

RollRead shield-release preparation is committed. Its neutral release owns
shield buttons and cannot be replaced by passive automatic shielding. Native
safety and explicit higher-priority tools keep their interruption rules.
