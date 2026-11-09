# Runtime safety and feature limits

This guide documents the guards that prevent an observation or controller
request from leaking into the wrong fighter, match, or mapper frame. It also
clarifies which sequence concepts are reusable core primitives and which
gameplay capabilities are actually connected to live input.

Read this alongside the [runtime architecture](ARCHITECTURE.md),
[action ownership](ACTION_OWNERSHIP.md), and
[observations and timing](OBSERVATIONS_AND_TIMING.md).

## Fighter and match identity

Live fighter observations are accepted only after validating the object
pointer/accessor relationship, fighter activity, native controller ownership,
the configured physical mapper index, and human participation. The runtime
stores actor IDs and session metadata, not native accessor pointers, as
cross-callback routing state.

`MatchSession` changes its epoch across ready/loading transitions and issues
`SessionTicket`s. Work from an earlier ticket is rejected even if a later match
reuses the same object ID. Runtime reset also advances the observation sequence
before clearing routes, policies, and pending requests, so an in-flight
publication cannot make stale work current again.

Relevant implementation:

* `src/live.rs`: `observe_fighter()` actor/controller checks and
  `map_controls()` routing checks.
* `src/live/state.rs`: stored route IDs and timestamps.
* `crates/kirby_assist_core/src/session.rs`: match epoch and ticket validation.
* `src/live/runtime.rs`: reset fencing and runtime suspension.

## Request freshness and cancellation

A request is eligible for mapping only when it belongs to the current mapper
tick and is within the bounded request lifetime. A stale assisted output becomes
neutral while retaining enough previous output state to send releases. Mapper
polls do not renew a human fighter route; the route must still identify the
associated object, mapper, and recent fighter observation.

Before composing controller overlays, the mapper rechecks runtime state,
controller identity and connection, match ticket, route, and request freshness.
Runtime disable/suspension clears participation, policy state, pending overlays,
training handoffs, and diagnostics. Entering Training passthrough cancels active
policies; saved policy state resumes only when passthrough ends.

The menu-presence watchdog is a 250 ms fighter-callback lease. Mapper polling
alone cannot renew it. A separate active-match gate protects master-switch
transitions during a match. These are separate safeguards: the lease does not
replace actor/session identity or per-request freshness checks.

Implementation references:

* `src/live.rs`: mapper admission and pre-map freshness checks.
* `src/live/runtime.rs`: runtime disable and suspension.
* `crates/kirby_assist_core/src/shared/observations/observation.rs`:
  `input_update_current()` exact-tick check.
* `crates/kirby_assist_core/src/session.rs`: watchdog lease and match session.

## Native grab handoff

`GrabHandoff` is a bounded receipt shared by ledge/button sequences,
automatic responses, and manual Catch. It starts only from a fresh grab request
or a native Catch edge. While pending, it suppresses competing action owners
so a delivered Grab cannot be replaced by an unrelated Special/throw packet.
The receipt advances from native motion progress, not mapper polling, and has a
bounded acceptance timeout.

The grab packet is constrained to Grab plus an optional shield modifier, with
neutral vertical stick and short-hop disabled. A queued ledge Grab receipt is
retained if another ledge action arrives. While the receipt is pending, only
the owning bounded neutral Grab pulses and the explicitly permitted safety,
defense, manual-jump, or shield exceptions may proceed. The receipt clears when
native acceptance occurs, state becomes invalid, the sequence is interrupted,
or delivery is exhausted.

This is an input handoff: the code observes Catch/grab progression and emits
controller input. It does not write native throw or fighter status.
Implementation: `crates/kirby_assist_core/src/grab_handoff.rs`.

## Distinguish recipes from generic playback

Saved input recipes and feature-specific state machines are wired to the live
runtime through explicit configuration and ownership paths. That does not imply
there is a universal combo-player service:

* `InputSequence` is a frame-indexed logical sampler used by examples; it
  proposes controller recipes and does not guarantee game action acceptance.
* `ConfirmedSequence` models target/status/hit-confirmed reactive steps with
  timeout and cancellation support, but is not wired to live playback slots.
* `ScenarioAction::PlaybackSlot` is a reserved future backend. It preserves
  human input rather than running a sequence.
* Target acquisition is feature-specific. Some defense and
  spot-dodge policies retain target IDs, but there is no universal
  combo-target service.
* Damage observations support DI and survival decisions through launch
  trajectory/blast-zone facts. There is no general damage-event attribution or
  combo playback ledger.

Current code references are `crates/kirby_assist_core/src/sequence.rs`,
`assist.rs`, and `config.rs`; live damage facts are read in `src/live/damage.rs`.
When adding a new automated sequence, wire its trigger, state, cancellation,
ownership, and mapper output explicitly, and test that the live path actually
uses it.

## What validation proves

Portable tests validate core decisions, parsers, state transitions, and
integration contracts represented by those tests. A successful host test or
plugin compilation does not establish that a specific input is accepted by the
game on a Switch, nor does it certify online compatibility, crash behavior,
frame time, or temperature. Report measured device results separately from
source-level or host-test evidence.
