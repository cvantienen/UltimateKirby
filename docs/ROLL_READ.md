# Roll and automatic dodge reactions

Current **Auto react** uses saved sequences for the selected fighter. Its simple
opponent-event slots are **Roll**, **Enemy spot dodge**, and **Enemy air dodge**.
Separate switches own shield grab/backup and after-grab-dodge responses; they are
not attacks enabled solely by Auto react. Open the corresponding Training list
using [Controls](CONTROLS.md); grammar/saving is in [Sequence files](SEQUENCE_FILES.md).

## A simple roll example

```ini
[kirby.roll.practice_roll]
name = Practice roll response
start_delay = 0
timeout = 120
reaction = auto_roll:18:12
actions = input:a:0:-1:2, wait:1
```

This illustrative row requests down+Attack for two native input frames. It is
not a bundled default, guaranteed down smash, or vulnerability forecast. The
simple editor exposes **maximum X distance and height**, not priority, crossing,
side or inferred roll-end timing. It admits the first ready, unconsumed observed
event in bounded target order. An eligible roll can be observed before
vulnerability. Supported tech/get-up roll statuses also count as roll events;
this is not limited to shield-origin rolls.

Your fighter must be grounded in controllable movement/ordinary landing or ready
shield, with no capture, hitstop or excluded recovery. Direct initial Up-B keeps
its native Up-B readiness check. The event is consumed on commitment; a later
closer target cannot steal the tape, and cancellation does not replay the same
remembered event. The active target must remain valid; no stale-event queue or
implicit turn/reach chase is added.

## Detailed legacy rules are different

Long-form `roll`, `roll_cross`, `roll_vulnerable` and `roll_cross_vulnerable`
retain detailed metadata. They are not synonyms for `auto_roll`; simplified UI
behavior must not be inferred from their priority/side fields. See
[Reaction rules](REACTION_RULES.md) for retained configuration/control support and
[Shield pressure](SHIELD_PRESSURE.md) for shield-history rules. The old compiled
Kirby/Yoshi fixed Roll-read/far-smash fallback is retired; hidden legacy timing
settings do not revive it.

## Input order, cancellation and extension

A committed response owns authored inputs, including neutral waits. Native
progress, not repeated mapper polls, spends duration; stop pauses it. Automatic
tapes can finish a previously delivered hold across a bounded observation gap
without bursting unseen actions. Safety, ledge admission and protected owners
can prevent a new response; Held Defense/Counter can explicitly replace a
nonsafety committed owner. Delivered Grab receipts cannot become Special;
immediate Up-B may replace only an undelivered automatic grab. Damage/capture,
mode/scope loss, invalid clocks/timeouts and invalid locked target cancel. Inputs
still cannot cancel native recovery or guarantee hits.

For Rust readers, `reaction_rules/automatic_watch.rs` separates readiness,
first-event selection, consumption and playback. `AutomaticEvent` is the stable
six-slot shared identity; `enabled_mask` admits families independently. Players
edit actions/ranges; adding an event requires parser, stable slot metadata,
native observation, editor and focused duplicate/consumption/interruption tests.
Adding a name alone cannot make the game emit input.
