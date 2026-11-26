# Auto tech: request Shield before a legal collision

A **tech** is Smash's native collision recovery from eligible damage/tumble.
This assist predicts or observes a bounded imminent floor/wall/ceiling contact,
then requests ordinary mapped Shield. It never forces passive status, removes
lockout, changes knockback, or makes an untechable impact techable.

## Activate and configure

Enable the **Auto-tech** menu row for the selected human fighter. It is independent
of holding ledge assist. Global OFF, controller/session invalidity, menu/handoff
and safety admission still apply; consult [Controls](CONTROLS.md) for mode routing.
Rust fallback `auto_tech` is **false**, whereas the currently supplied universal
section sets it **true**. An explicit saved/startup false is not ignored.

```ini
[universal.settings]
auto_tech = true
tech_lookahead_frames = 5
tech_cooldown_frames = 40
takeover = true
tech_debug = true
```

This is a focused configuration example, not an instruction to overwrite the
whole section. Rust fallback lookahead is 3 (accepted 1–5), cooldown 40 and
takeover/debug false. Supplied universal lookahead is 5 and takeover/debug true.
Only diagnostic builds produce debug evidence; the flag does not add logging to
silent. See [Fighter settings](FIGHTER_SETTINGS.md) for body-probe estimates and
startup preferences and [Sequence files](SEQUENCE_FILES.md) for settings precedence.
The guard button in the selected game mapping must really be Shield.

## Observation and scheduling

1. Read native status, situation, stop timer and passive transition flags.
2. Reject ordinary air-escape availability: an automatic Shield that could become
   an air dodge is unsafe. Passive-disabled/unsupported states cannot qualify.
3. Accept a verified native floor/wall/ceiling contact, or predict contact only
   within eligible damage/tumble and a bounded lookahead.
4. Propose/release mapped Guard according to the scheduler's contact deadline.
5. Commit artificial cooldown only when the original mapper confirms a **fresh
   GUARD edge**, not when a proposal was created.

A **transition term** is the game's permission for an action; it is an eligibility
check, not by itself a reason to press Shield. Prediction uses copied root/body
probe estimates and read-only stage rays. Only descending floor probes admit
pass-through platforms. The live delivery allowance is a fixed one frame; it
is not measured online delay or learned compensation. Known late damage hitstop
can forecast using a finite reserved launch vector rather than frozen velocity.
Remaining stop time belongs in contact time. Early/unknown stop does not fabricate
an advancing clock or a collision.

Descending-floor prediction may use one matched copied motion/velocity sample to
estimate acceleration. Invalid identity/motion/rate, large gaps, discontinuities
or hitstop discard that trend and use the bounded linear path. This approximation
can miss changing movement, slopes and moving platforms; it is not native physics.

## Grounded Ganondorf Flame Choke

Grounded `CATCHED_GANON` is handled separately. The adapter resolves the actual
capture link and verifies the linked Ganondorf's grounded catch state and motion
clock before estimating contact. Missing links/clocks mean no invented deadline.
A verified grounded-origin release can supply immediate legal floor contact.
Aerial capture/fall and release with aerial/unknown origin remain excluded; the
assist cannot make aerial Flame Choke techable. Timing depends on the native move
script and requires device verification.

## Shield edges, cooldown and cancellation

With takeover enabled, existing mapped Shield/Grab aliases can be released so a
new Guard edge is possible. Tech ownership is limited to that shield/grab family;
it is not blanket Jump/Attack/DI suppression. Additive mode cannot force a held
Shield release. Preparation belongs only to a bounded eligible contact, not the
whole fall. A held button suppressed during takeover needs release and a fresh
press on handback; an old hold cannot manufacture another edge.

Cooldown uses rate-normalized native progress, excludes hitstop and remains
conservative through unknown animation changes. Native tech status clears the
artificial cooldown for another collision. A fresh human Shield press can update
tracking too. Unmapped requests remain pending only while contact is still valid;
losing contact, eligibility, controller scope, menu/handoff or freshness removes
them. Repeated mapper polls do not advance a motion clock. Requests expire after
35 ms; the 250 ms controller-gap reset is a separate wall-clock safeguard.

Capture mash has first safety priority. Actual tech/preparation ownership and
DI/SDI competition depend on the live scheduler; a mere distant forecast is not
tech ownership. Button-only jump forwarding is not permission to start a new
exclusive authored tape during incapacitation.

## Diagnose before changing timing

In diagnostic builds with `tech_debug`, distinguish a collision forecast,
requested Shield, mapped fresh Guard edge and actual native tech status. Those
are four different observations. Check controller mapping and takeover first,
then contact/stop/eligibility and the probe estimates. Do not interpret a proposal
or mapper receipt as a successful tech. Silent provides behavior without these
logs; see [Training monitor](TRAINING_MONITOR.md).

For Rust readers, `src/live/tech.rs::assess_tech` reads eligibility/contact and
`input_feedback` reads consumed Guard and damage reaction time. The live scheduler
uses `tech_input.rs`; `policies/tools.rs::AutoTech` is a generic scenario preset,
not the native scheduler. `Option<f32>` contact means either a finite estimate or
no contact evidence. Adding a state requires verified native safety and focused
contact/acknowledgment/cooldown regressions, not simply extending a status range.

No host model or documentation check certifies zero misses on Switch. Native
lockout, untechable impacts, SDI, geometry changes and online queues remain limits.
