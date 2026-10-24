# Shared observations and game progress

The live adapter creates one `FrameReads` inside each validated local-fighter
observation. All assistance policies share this instance. It lazily copies
status, situation, motion/frame/rate, position, total velocity, facing, team,
stop, previous-status and animation-end facts into eight fixed identity slots.
Repeated field reads reuse their first value. No heap allocation, new hook,
worker, network activity or eager full-roster read is added. Unexpected extra
identities read through without evicting cached fighters.

The snapshot belongs to **one callback**, not one mapper tick or match. It is
dropped on return; the next observation reads fresh facts even if its mapper
tick is unchanged. Only copied values and IDs are stored, never native pointers.
Existing session, identity and routing checks remain. This is a reuse boundary,
not a promise that individual native reads form an atomic engine snapshot.

## Coverage

- All fighters' ledge responses, grab movement and recovery coverage.
- Slides, ledge trump and onstage roll reads (whiff punishing is disconnected).
- Autopilot shielding, shortcuts, finishers and fair/trip confirmations.
- Tech/captor timing, SDI, and the currently disabled auto-duck reader.

Readiness/transition terms, hit confirmations, active attacks, collision queries
and protection metadata remain live, demand-driven reads. The snapshot never
crosses the existing observation/publication fence.

## Consistent progress without changing timings

`timing::motion_progress` classifies initial observations, duplicate poses,
advancement, status/motion transitions, rewinds and invalid clocks. Elapsed
motion time uses the previous sample's rate. Each caller retains its explicit
initial/transition, hitstop, rewind and timeout policy. Tech cooldowns retain
their stable-rate basis; ledge velocity estimates retain their current-rate
basis. These policies are not silently recalibrated.

Ledge attempts/grab delivery, finishers, fair confirms, shortcuts, guard holds,
roll reads, the parked whiff prototype and recovery cover share this arithmetic. The
`AnimationClock` uses it for event deduplication, retaining native clatter-timer
and identity changes. Mash, slides, turbo and ledge trump receive that event
clock as before. Generic confirmed sequences supplied actual simulation frame
numbers use `simulation_elapsed`. Direct attack-frame and native hitstop-timer
checks remain direct checks, rather than being converted into a different clock.

This is not a global simulation counter: a motion change still contributes the
existing one-frame transition allowance. Animation data cannot reconstruct every
unseen game update. Owning policies handle hitstop before spending sequence time.

No waiting frames, learned delay offsets or move-timing changes are added. The
35 ms request freshness and 250 ms observation/session fences are unchanged.
Network packets and arena delay settings are untouched. Switch testing remains
necessary to measure processing cost and actual online input acceptance.

## Extending the system

1. Pass the current `&FrameReads` to new native observers, rather than creating
   an independent cache. Reacquire and validate accessors as usual.
2. Keep readiness/contact checks live; publish copied portable contexts.
3. Use `motion_progress` / `motion_elapsed` for motion-relative time and define
   stopped, initial, transition and rewind behavior explicitly.
4. Use `simulation_elapsed` only for a supplied real simulation-frame counter.
5. Test duplicate polls, hitstop, rate changes, loops, identity changes, rejected
   inputs and the existing attack deadline. Do not introduce callback-count
   timers or sleeps for action sequencing.

## Verification limits

`tests/shared_observations.rs` covers lazy reuse, fresh callbacks, overflow,
re-entry, unchanged valid elapsed arithmetic, event deduplication and simulation
counter separation. Existing action tests cover request timing and acceptance.
Native helpers are also host metadata-checked against actual FFI signatures.

Action ownership now resolves committed proposals before the final input
overlay; see [rules and actual live-wiring tests](ACTION_OWNERSHIP.md). This does
not add timing offsets or replace native readiness checks. No Switch plugin
was built as part of the host-only verification.

The broader legacy lib suite currently has 18 behavioral expectation failures.
A temporary control using previous elapsed arithmetic reproduces the same 18
failures (343 passes). The legacy diagnostic-logger harness also lacks report
module imports. Focused regressions run separately; the full suite is not clean.
