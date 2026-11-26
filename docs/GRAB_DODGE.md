# Grab avoidance and configured follow-up

Grab avoidance predicts an **ordinary standing/dash/pivot grab**, requests one
spot dodge, then optionally plays your fighter's saved **After dodging enemy
grab** response. Avoidance and enemy Spot punish are different tools. Native
spot-dodge startup is confirmation of the attempted dodge—not proof the grab
missed or the punish will hit.

## Enable and tune

Use the current Grab avoidance row and its Training list in
[Controls](CONTROLS.md). The portable menu starts it ON; Rust `auto_spot_dodge`
and the supplied universal value are true. Optional fighter
`grab_dodge_default` overrides startup. The row's current state and saved startup
are distinct; see [Fighter settings](FIGHTER_SETTINGS.md).

```ini
[kirby.settings]
grab_dodge_lookahead = 6
grab_dodge_padding = 2
grab_dodge_height = 8
```

These are schema fallback values, not a read of your saved Kirby section.
Lookahead accepts 1–8 native frames. Padding adds an estimated horizontal body
allowance to the attacker's reference reach; height is root-height tolerance.
Larger values can dodge sooner/wider but may dodge too early. They do not resize
hurtboxes or change native grab activity. The current policy's ground-status gate
admits controllable movement/ordinary landing/ready shield candidates; final native
spot-dodge readiness and physical/manual intent checks still determine admission.
Do not rely on older documentation's idle-only description.

## How a threat becomes input

The bounded fighter snapshot supplies ordinary grab motion, active window,
position, velocity, facing and reach. Horizontal and vertical interval overlap
must occur at the **same future time** inside activity. Unknown grab data, command
grabs, invalid clocks, capture links, air or unavailable dodge readiness do not
invent threats. Current active ledge hold/toggle suppresses these new searches.

The earliest qualifying target receives neutral-horizontal **Down+Shield**.
Before mapper delivery the request remains bounded/revalidated. One delivered
packet is followed by neutral input while waiting for native `ESCAPE`; it does
not repeatedly flick Down and queue a second dodge. Duplicate receipts do not
spend the budget. Native progress or at most 24 distinct delivered/attempted
mapper updates bound an unanswered attempt. Stationary held shield can borrow
already-observed attacker motion progress; controller polls are not game frames.
Stop pauses native progress.

A temporarily lost forecast/readiness before delivery releases the request
without necessarily marking that grab handled; it can qualify again while still
threatening. Delivery, native dodge confirmation or exhausted attempt marks the
observed grab event used. The identity survives missing observations until an
already-read changed status confirms the old grab ended.

## After-dodge response example

```ini
[bowser.roll.practice_grab_dodged]
name = Practice grab-dodge follow-up
start_delay = 0
timeout = 120
reaction = auto_grab_dodged:25:10
actions = input:b:0:1:4
```

The supplied Bowser event currently also uses maximum X25/Y10; the actions above
are an illustrative short upward-Special experiment, not a claim about its full
saved tape. Use [Sequence files](SEQUENCE_FILES.md) for selection, exact grammar
and adding an explicit drift/wait/check. The simplified event exposes maximum
X/height, not legacy side/priority fields. Missing your fighter's own
`auto_grab_dodged` entry means dodge only; it never borrows an enemy spot-dodge or
another fighter's punish.

Only the pending assisted dodge's observed native startup arms follow-up against
its original grabber. Recovery owns no speculative attack. At the first native
recovery opportunity, the original enemy must still be valid and inside the
saved limits; a miss is discarded rather than queued to chase later. Direct
Up-B keeps its native readiness and bounded receipt; generic configured actions
use native control readiness. Once Up-B confirms, only the remaining authored
steps run—no Bowser spin/drift tail is invented.

## Cancellation and extension

Airborne start/unconfirmed takeoff, damage/capture, lost target, disabled mode,
invalid scope/clock/timeout and competing safety cancel. Held Defense/Counter
can explicitly preempt a nonsafety committed response. A new warning cannot
resume an old hidden punish after cancellation. Physical Grab and native
catch/pull/hold/pummel/throw hand control back so throw choice remains manual.
A mapped upward Special may retain a short input-free receipt through readiness
changes; it never holds B while waiting or proves successful startup.

Rust beginners: `policies/spot_dodge.rs` separates forecasting, mapped delivery,
native `ESCAPE` confirmation and `punish_automatic`. `Option<u32>` target means
an acquired opponent or none; `used` remembers an answered motion event.
`grab_handoff.rs` prevents delivered Grab becoming a Special/throw packet.
Players tune estimates and saved actions; developers adding command-grab support
need verified native timing/scope and focused duplicate/receipt/interruption
tests. Input-only approximations cannot guarantee avoidance or a punish online.
