# Training handoff and honest monitoring

This guide covers supported controller handoff and how to interpret diagnostics.
It is not a record of one console's crash, installed host hash or a promised GPU
fix. No monitor, overclock preset, renderer patch or Switch upload is installed
by a documentation update.

## Menu versus Training passthrough

Use [Controls](CONTROLS.md) for current navigation. Minus opens the ledge home or
closes a page. Actual Training match entry starts hidden; non-Training lightweight
selection/navigation is different and does not expose editing/save/test tools.
Do not assume hidden X/Y always mean an assisted hop/finisher: selected saved
bindings determine their behavior.

Recognized Training Modpack save/load shortcuts hand physical input through
while held and resume on release. Sticky menu/recording shortcuts retain the
handoff until you finish that external interaction and make a fresh Minus press.
Standard ZL+Up/Down save/load is not a claim that arbitrary custom shortcuts are
recognized. Keep monitor and Training shortcuts distinct from assist bindings.
An OS overlay capturing input can prevent the game from receiving it at all;
this mapper cannot restore events never delivered to Smash.

On entering passthrough, `src/live/training.rs::enter_passthrough` cancels pending
assist output once, then maintains controller association/timestamps while the
handoff continues. `finish_passthrough` resumes saved state for ordinary release
or requests the menu after modal handoff. It does not rearm every consumed button
recipe or force a paused native move. Global/mode/fighter selection still applies.

## Enable diagnostic observation, not a physics experiment

Silent and diagnostic builds share input-only behavior; silent excludes diagnostic
logging/instrumentation. A debug flag cannot create logs in silent. Use the
Performance gauge/diagnostic controls documented in [Controls](CONTROLS.md) and
[Settings reference](SETTINGS_REFERENCE.md). Copy evidence before restarting:
diagnostic log files reset at plugin startup. Buffered queues can drop messages;
a missing line is not proof the path did not execute.

A repeatable trial:

1. Fix stage, fighter, opponent, positions, damage and chosen recipe. Disable
   unrelated helpers and record that baseline selection.
2. Reproduce one feature's real trigger, then compare its OFF and ON runs. Merely
   standing idle does not exercise collision prediction or receipt delivery.
3. Keep menu/editing windows separate from gameplay. Change one range/delay/flag
   at a time and allow a fresh measurement window.
4. Compare request, mapped acknowledgment and actual native move observation.
   They are different events: neither a proposal nor mapping proves success.
5. Compare diagnostic instrumentation ON/OFF, then silent independently. Instrumentation
   has its own overhead and cannot certify unchanged whole-system performance.

## Read-only telemetry dashboard

Launching `python3 tools/workbench.py` starts the local telemetry receiver alongside
the browser workbench. Open **Training telemetry** in the sidebar to use the
embedded dashboard, or choose **Open full dashboard**. First startup can compile
the existing Rust host example using offline Cargo; opening it never builds or
uploads a Switch release. Receiver startup failures appear in the page. Closing
Workbench stops its receiver.

No SD pairing file or manual key is needed. Enter the Diagnostic Switch's private
IPv4 and telemetry UDP port (4243), then activate **Live telemetry UDP** explicitly
in Training. Receiver subscription does not activate the Switch mode. The stream
is unauthenticated and unencrypted by default: use only a trusted private LAN.
Training LAN editing is a separate service and is not needed to view telemetry.

- **Object** selects an observed battle-object ID, not a controller/player slot.
  Coverage is bounded; absence does not prove an object does not exist.
- **Publication group** selects object state, controller input, or attack slot
  0–7. Groups are separate publications, not an atomic whole-frame snapshot.
- **Field filter** searches names, wire types and source labels in displayed rows.
  It does not change what the Switch samples. **Available values only** hides
  missing values; unavailable is not zero.
- **Schema reference** shows the field catalog even without a live observation.
  Select a field for source/type interpretation and exact raw bits. Native numeric
  IDs are not guessed into names. Physical input precedes mapping; mapped input
  may include existing assistance and is not physical input.
- **Freeze values** freezes the displayed snapshot, not the receiver or game.
  Freshness still ages. Records are stale at 250 ms and expire at one second.
- Numeric charts retain at most 240 browser-observed points over 60 seconds;
  throughput charts retain at most 120. Browser polling can miss publications.
  Approximate graph numbers do not replace exact integer/bit strings in the table.
  **Clear charts** affects only browser history. **Export visible CSV** exports
  the current filtered rows, not all observations or chart history.
- `sample_us` measures sender publication microseconds since activation, and
  `sequence` orders all session publications. Neither is an engine-frame counter,
  synchronized clock, ping or end-to-end latency. Sequence gaps count unseen
  positions; sender drops report a separate queue/worker counter. Do not add them
  into a purported network-loss percentage.

## What measurements mean

- **Microseconds** measure sampled local elapsed time;1000 microseconds is1 ms.
  A nominal60 FPS frame is about16.67 ms shared with Smash and other components.
- Sampled hook/observation times may include nested/original game work. Inclusive
  rows overlap: adding percentages double-counts them.
- Draw-callback gaps include pauses/loading/missed callbacks; they are not a
  verified game FPS counter or simulation-frame clock.
- Retained recipe/menu payload estimates are not total process RAM or allocator
  overhead; growth is a clue, not proof of a heap leak.
- Queue depth/drops/flush time explain logging pressure, not network latency.
- A manually entered external-delay label labels a trial; it does not inject,
  read, learn or compensate Training/online delay.

## Labelled latency and workload trials

A diagnostic external-delay label records the delay chosen in another tool; it
never introduces delay or reads that tool's private configuration. Keep the same
stage, fighter, opponents, recipe and enabled helpers while comparing labelled
trials. Separate setup/menu windows from the repeated interaction, record both
successful and failed attempts, and preserve the corresponding diagnostic windows.
A fixed Training delay does not reproduce online jitter or complete system load.
Mapper acknowledgment may precede a separate queued-input hook, so it cannot
certify final engine consumption or measure network ping.

External CPU/GPU/temperature/fan displays are a separate system measurement.
Temperature stabilizes slowly and cannot attribute cost to one helper. A sampled
CPU-side mod timing cannot diagnose GPU command lifetime, clock stability or a
system-thread abort. Avoid changing clock plugins, renderer hosts, recipes and
monitoring components simultaneously when isolating a problem.

## Freshness and ordering when investigating missed input

The live mapper prepares under shared state, drops the lock before the original
mapper, then rechecks publication before final adjustment/acknowledgment and
restores raw controller storage. Native observations are copied/validated before
policy sampling. A stale35 ms request can be suppressed; a250 ms controller
sample gap resets association. These wall-time safeguards are not online ping
or native move time.

Safety runs first; ordinary committed owners retain admission, while Held
Defense/Counter explicitly preempt nonsafety owners. A mapped Grab receipt can
prevent a competing Special from becoming an accidental throw. Paused/duplicate
native motion must not be mistaken for missed progress. See [Auto tech](AUTO_TECH.md),
[DI/SDI](DI_SDI.md) and [Out of shield](OUT_OF_SHIELD.md) for their different
acknowledgment/confirmation lifecycles.

## Rust extension and useful evidence

A beginner can follow `enter_passthrough` and `finish_passthrough`: they coordinate
state, not file I/O or native physics. Preserve the first-entry cancellation and
session-alive timestamps when changing handoff. Tests should cover ordinary
release, sticky fresh-Minus exit, controller identity and no hidden recipe rearm.
Diagnostics changes belong in their dedicated owner/module, not extra gameplay
polling or synchronous logs here.

For a report, provide build variant/version, controller layout, fighter, active
mode/binding, exact reproducible event, expected versus native result and relevant
bounded diagnostic windows. Protect secrets/private addresses and retain original
crash reports locally. Logs and file validation can narrow a problem; they cannot
prove an installed host matches source or a crash is fixed. Any build, external
source inspection, host replacement or Switch write requires its own requested
workflow.
