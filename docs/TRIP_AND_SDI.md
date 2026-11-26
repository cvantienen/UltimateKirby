# Kirby confirmed-trip follow-up and separate SDI

The trip follow-up watches Kirby's grounded forward/down tilt, confirms a **new
nearby trip attributed to that attack**, then attempts facing forward smash.
Auto SDI is unrelated damage-hitstop stick assistance; enabling one does not
activate the other. Neither changes native physics, status, hitboxes or trip odds.

## Activate and configure

`down_tilt_trip_smash = true` is the Rust startup fallback. The route does not
require a ledge hold, but mode/global enabled, selected controller/Kirby and
native safety admission still matter. To disable it independently:

```ini
[universal.settings]
down_tilt_trip_smash = false
```

Use [Settings reference](SETTINGS_REFERENCE.md) for shared startup precedence
and [Controls](CONTROLS.md) for actual current admission. The internal confirmed-
trip timing/range is compiled, not a set of arbitrary saved-sequence sliders.
[Fighter settings](FIGHTER_SETTINGS.md) owns configurable estimates elsewhere;
a similarly named legacy field cannot change this state machine by assumption.

## Current recognition: stricter than older versions

The source requires your grounded `ATTACK_S3`/`ATTACK_LW3`, corroborated
`hit_confirmed` attacker-entry evidence and the same target changing from
non-trip to native `SLIP..SLIP_WAIT` **during the same tilt motion**. The target
must be grounded, in front within18 horizontal units, and within8 vertical units.
An already-tripped enemy does not newly arm it. Old documentation's trip re-hit,
`DOWN_DAMAGE` fallback, no-attribution and unlimited-pulse claims are retired.

Fresh recognition releases the old tilt input first. The queued target must
remain nearby/tripping with the same facing and your fighter in a supported
grounded tilt/idle state. The policy alternates neutral release and facing+A
requests, at most **six flicks**, bounded by **18 native updates**. Inputs buffer
through ordinary recovery; they cannot cancel it. Native smash startup releases
the proposal rather than charging or forcing attack selection.

Duplicates/stop do not add pulses. Air, damage/capture, wrong action, lost/changed
target/facing, disabled route, invalid scope/clock and budget expiry cancel,
returning a neutral release. Safety and explicit Held Defense/Counter can
preempt. A confirmed trip plus mapped Attack does not prove the smash connected.

## SDI belongs to its own policy

See [DI and SDI](DI_SDI.md) for the authoritative damage response, defaults,
geometry and ownership. Rust fallback SDI is false; supplied universal true may
be overridden by a fighter false. It requests flicks only in victim damage
hitstop, with neutral gaps and final-hitstop DI handoff; it does not run while
Kirby is attacking merely because a trip route is enabled.

## Beginner source map and extension boundary

`src/live/kirby_followups.rs::trip_enabled` admits only nonsuspended configured
Kirby; `trip_watch` requests observation during eligible tilts or an existing
queue. `trip_context` supplies copied native and target facts; `TripFollowup`
checks confirmation and progression. **Watching** is not yet input permission,
and **hit confirmation** is not equivalent to any nearby opponent being tripped.
`Option<TripFollowup>` distinguishes enabled state from absence.

Players can toggle the compiled route and practice spacing, or author separate
literal inputs in [Sequence files](SEQUENCE_FILES.md). Developers changing trip
recognition need transition/attribution, already-tripped rejection, range/facing,
six-pulse budget, duplicate/stop and smash-handback regressions. Never increase
native trip odds or shorten move lag. Test real trips and nontrips at both facings;
online delay/native acceptance remains a limitation.
