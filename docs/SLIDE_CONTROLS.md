# D-pad Left/Right slide attempts

**Source change only: no NRO has been rebuilt for these controls.**

Tap D-pad Right or Left to attempt one short-hop/downward-angled directional air dodge
in that absolute direction. Hold the direction to repeat after the slide lands
and the native grounded jump transition is available. Both directions together
cancel the macro. A tap completes its one attempt after release.

The input sequence is:

1. If grounded, supported in that direction and jump-ready, press X with neutral stick.
2. Release X during jump squat to request a short hop.
3. At the earliest observed native jump frame with air dodge enabled (retrying
   through frame 6 if the first sample/readiness was missed), request
   ZR with stick **[0.9588, -0.2839]** or **[-0.9588, -0.2839]**.
4. Hold ZR and the angle until native dodge acceptance or the bounded timeout, then
   release ZR and
   keep neutral input through the dodge/landing. Do not walk
   against the slide or force a status change.
5. If still held, jump on the first observed grounded jump-ready update; otherwise
   return physical controls. Native landing recovery is respected; a pending tap starts as soon as jumping is legal.

A missed early-air window aborts, rather than executing an unwanted dodge high
above the floor. A tap during grounded recovery or an existing air-dodge slide queues one attempt for up to 60 advancing native
observations. Waiting does not own movement or buttons. Dash/run/turn/brake,
sliding and native cancel-ready actions can start on the first legal jump-ready
observation; returning to idle is not required. A new side tap replaces the queued
direction. Holding the side keeps a pending request alive through longer recovery.
A tap during hitstop is remembered until progression resumes. Damage,
unsupported states, both-side conflicts, disabled scope and loss of valid
controller routing cancel the sequence. Native hitstop freezes progression.
D-pad Down trump takes priority; mash and tech retain their normal priority.
Slide requests override held-L3 movement and suspend the ledge response policy.

The trigger is consumed before taunt mapping, including diagonal/release history.
The macro shares the same X/ZR edge history as other tools. It requires X=Jump
and ZR=Shield on the selected human player's controller.

```text
dpad_slide = true
```

This implements the requested inputs, not a proven perfect slide. A native
animation frame is not an exact height measurement. Input cadence and game
acceptance can miss the window, and actual floor collision/slide distance needs
Switch testing. Ultimate's directional air dodges have native landing recovery;
see [SmashWiki's wavedash explanation](https://www.ssbwiki.com/Wavedash). The
sequence does not bypass that recovery or promise an immediate actionable loop.

The downward angle is the requested raw controller vector, replacing the old
pure-horizontal recipe. It is not a verified optimal Ultimate angle or a proven
universal Y threshold. The linked 73.5-degree advice does not establish that
claim for Kirby in Ultimate; check actual dodge direction and floor contact on
the Switch before tuning distance further. Native landing recovery and a bounded early-airborne window remain enforced. This source change has not been rebuilt.

Moving starts now include dash/run/turn/brake states. Grounded attacks/specials
also require native cancel enablement in the adapter in addition to jump
transition readiness. Pressing D-pad does not force a game status or bypass an
uncancellable animation. The floor probe uses the selected world direction.

Online follow-ups retain the angled shield input until native dodge status is
observed. The takeoff acknowledgment budget is 24 advancing observations; dodge
acknowledgment allows eight plus the learned input delay (at most 12 extra).
These are bounded acknowledgment budgets, not a claim that observations are a
global frame clock. Later acceptance can reduce slide distance even when the
recipe completes.
