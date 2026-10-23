# Left/Right: downward air-dodge slide

With the menu hidden, tap D-pad Left or Right for one short-hop/downward-angled
air-dodge attempt. No L modifier is required. Holding the side repeats after
landing when native jumping is ready. Both sides together cancel.

**L+R3** shows/hides the menu. Minus toggles auto-mash. While open, D-pad Up/Down selects a row and
Left/Right cycles options, including with L held. Opening the menu cancels any
queued/active slide or trump. Directions held when closing must be released
before they can start tools. The panel starts visible each match; use L+R3 to hide it before sliding. With the panel hidden, hidden Down starts ledge
trump. Hidden Up is normal; Y now attempts neutral-B reverse.

The input sequence is:

1. If grounded, supported in that direction and jump-ready, press X with neutral stick.
2. Release X during jump squat to request a short hop.
3. At the earliest observed native jump frame with air dodge enabled (retrying
   through frame 6 if the first sample/readiness was missed), request
   ZR with stick **[0.7071, -0.7071]** or **[-0.7071, -0.7071]**.
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
Down trump takes priority; mash and tech retain their normal priority.
Slide requests override R3 teabag and the X/Y shortcuts and suspend the ledge response policy.

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

The raw vector is now a steeper 45-degree downward diagonal. Both axes exceed
the overlay's digital-direction threshold, so the analog angle and its Down/
Left-or-Right flags agree. The angle remains paired with ZR until native dodge
acceptance. This is input tuning, not a guarantee of online floor contact or a
universally optimal slide distance. Native landing recovery still applies.

Moving starts now include dash/run/turn/brake states. Grounded attacks/specials
also require native cancel enablement in the adapter in addition to jump
transition readiness. Pressing D-pad does not force a game status or bypass an
uncancellable animation. The floor probe uses the selected world direction.

Online follow-ups retain the angled shield input until native dodge status is
observed. The takeoff acknowledgment budget is 24 advancing observations; dodge
acknowledgment allows eight advancing observations with the live extra-delay value set to zero.
These are bounded acknowledgment budgets, not a claim that observations are a
global frame clock. Later acceptance can reduce slide distance even when the
recipe completes.
