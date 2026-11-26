# Enemy Spot punish

Spot punish responds to the opponent's ordinary grounded spot dodge. It does
not dodge an incoming grab: that is [Grab avoidance](GRAB_DODGE.md). The detailed
Spot punish family and simplified **Auto react → Enemy spot dodge** are also
different configuration paths; select the one exposed in your current menu.

## Detailed example and simple counterpart

```ini
[bowser.attack.practice_spot]
name = Vulnerable spot-dodge grab
start_delay = 0
timeout = 120
reaction = spot_dodge:grounded:0:12:18:front:1
actions = input:r:0:0:4
```

This detailed example is not calibrated reach. `input:r:0:0:4` requests mapped
Grab for four input frames; `grab:NUMBER` is a range check, not button duration.
Detailed rules use state, X minimum/maximum, height, side and priority. See
[Sequence files](SEQUENCE_FILES.md) and [Reaction rules](REACTION_RULES.md) for
syntax/current list availability and [Controls](CONTROLS.md) for activation.

The simplified counterpart uses `reaction = auto_spot_dodge:25:10`: only maximum
X and height. The supplied Bowser automatic event currently has X25/Y10, but its
full saved action tape should be inspected in the fighter guide, not inferred
from this example. The simple event does not wait for vulnerability or apply
detailed priority/side metadata. Its switch is Auto react; retained detailed
Spot punish has its own family. No compiled smash fallback is supplied when a
matching configured response is absent.

## What recognition means

Detailed `spot_dodge` requires native ordinary `ESCAPE`, a valid motion/rate,
observed end of invulnerability and remaining recovery. `spot_dodge_any` can
match the same ordinary dodge before vulnerability or without a known recovery
forecast. It does not turn air dodge or unknown invalid clocks into spot dodge.
A lower-priority any-dodge rule can win **now** before a later vulnerability
rule becomes eligible; priority does not reserve a future opportunity.

Each observed dodge motion is answered once on commitment. Cancelling that tape
does not retry the same event; a genuine later motion can qualify. The detailed
selector uses the best currently matching priority across bounded candidates.
The simple automatic selector instead commits the first ready unconsumed event.
These differences are intentional, not alternative descriptions of one scheduler.

## Ownership, timing and cancellation

Own native readiness, mode admission, controller scope and nonstopped play still
apply. Grounded is not permission to cancel arbitrary recovery. Protected ledge
and committed/safety owners can prevent selection; Held Defense/Counter explicitly
preempt nonsafety owners. Authored playback masks competing gameplay input;
stop pauses its native-progress clock and duplicate mapper polls do not spend
holds. Target-dependent steps need a fresh valid locked target, not a substitute
nearby opponent. Damage/capture, mode/scope loss, clock/timeout and target loss
cancel. Delivered Grab hands back native catches/throws rather than appending B.

## Rust walkthrough and player tests

`policies/spot_punish.rs::Punisher` remembers target motion identity and `used`,
constructs copied recovery facts, selects a configured `Response` and plays it.
`option_read.rs` retains a portable recovery-punish helper and observation types;
its presence does not establish a live hard-coded smash fallback. Players edit
rule conditions and actions; developers must update event parsing/observation/
selection and focused tests when adding behavior.

Test vulnerable and any-dodge rules separately, both sides and just outside range,
then interrupt playback with damage and manual capture. A detected recovery
window or mapped Grab is not proof of a guaranteed hit. Native invulnerability,
move startup, spacing and online queues remain limits; no physics/status writes
or new device certification result from this documentation.
