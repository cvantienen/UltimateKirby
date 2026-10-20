# Rust reliability review — 2026-10-19

Reviewed the Rust integration in `src/`, every module in
`crates/kirby_assist_core/src/`, their host tests, and the existing examples.
The silent and diagnostic plugins use the same gameplay policy.

## Findings addressed

| Finding | Change |
| --- | --- |
| Native hit observation omitted forward smash | Include native forward-smash attack status in hit observation; startup/charge alone cannot confirm a hit |
| Jump response lost control at takeoff | Continue only a policy-started hop through jump squat, ordinary jump/fall, attack and landing; damage aborts it |
| Aerial status did not identify the move | Require `attack_air_b` motion as well as native aerial status for back-air confirmation |
| Directional ground attack could select the wrong aerial | Face away, neutralize, pulse X, release during jump squat, then use rear stick + A after native takeoff |
| Independent native button overlays kept separate edge histories | Combine tech, mash and ledge requests through one `ManualOverlay` history; test takeover and additive tech |
| Repeated mapper callbacks could spend cooldown/phase updates | Deduplicate mash/tech/turbo from native animation progress; native clatter time also advances the clock when the trapped pose is frozen |
| Cached proposals could survive changed eligibility within one tick | Damage, target loss, navigation disable and scheduler cancellation override cached output |
| Frozen-clock resume could replay an old tech pulse | Clear cached proposals during stop/unknown timing without discarding cooldown history |
| Old pending attacks could survive target loss or plan changes | Clear pending moves and attempt/animation counters with their visit/plan; preserve ledge geometry and last target status where required |
| Airborne prediction used only two frames of Kirby's velocity | Project both fighters to the candidate aerial hit frame; ground prediction retains its short momentum estimate |
| Roll and other attacks duplicated vulnerability checks | Share `ActiveWindow::first_contact`, including optional input delay and an exclusive actionable deadline |
| Generic sequences advanced without action/hit evidence | Add `ConfirmedSequence`, with readiness, action/hit gates, release frames, interruption/target/clock cancellation and timeouts |
| Missing fighter data could leave generic adapter output latched | Restore fresh physical input if the context/observation disappears; missing human input still needs the adapter's watchdog |
| Invalid native velocities could be treated as stationary | Reject invalid ledge inputs and overflowing tech-probe motion |
| Ledge navigation accepted invalid spots or turned without support | Validate locked spots and support-gate turns; cancellation precedes cached movement |
| Trump attempt could cancel in ordinary movement hitstop | Freeze the run during stop; damage/unsupported states still abort, including within a repeated tick |
| Tests assumed old editable config values and three retries | Use current supplied settings and the existing six-attempt policy; preserve boundary/whiff assertions |
| Arbitration reported already-held buttons as additions | Report only bits actually added to the physical input |

## Back-air response

[Kirby's frame data](https://ultimateframedata.com/kirby) lists back air active
on frames 6–8 (early) and 9–12 (late). Neutral air lasts longer; back air starts
faster and provides rear coverage. The ground forecast includes the three-frame
jump squat (modeled hit window 9–15). Once airborne, the policy rechecks the
actual rear-side position, relative movement and protection against frames 6–12.

The default and supplied `ledge_anti_air_range` are now 12 world units instead
of 8. This is an estimated root-position gate, not a measured hitbox boundary.
Existing edited tilt/grab/stance/turbo settings were preserved. A target that
crosses to Kirby's front after takeoff does not receive a mistaken forward-air
request. Click L3 again to return physical control.

## Reusable timing and combo APIs

`timing.rs` provides `ActiveWindow`, `RecoveryWindow` and `PredictedContact`.
For each candidate active frame it checks protection, reach and recovery at the
same time. Unknown protection returns no contact. An actionable deadline is
exclusive. `recovery_margin=None` explicitly means no recovery guarantee was
established. Roll uses its observed remaining animation as a recovery estimate.

`ConfirmedSequence` in `sequence.rs` emits a step once, waits for its expected
native status and optional hit, releases, then waits for readiness before the
next step. It cancels on timeout, target changes, clock rewind or interruption.
Hitstop freezes its timeout. Duplicate simulation-frame samples reuse output.
A pre-existing animation cannot count as a newly requested step.

The old `InputSequence` remains a stateless teaching sampler. Live recording
slots and general combo playback remain unwired. The new runner requires a
verified simulation-frame ID and per-combo triggers/acceptance measurements; a
mapper callback count must not be substituted. It also needs takeover or an
adapter that can release physical conflicting buttons when a recipe ends.

## Source coverage

| Files | Review result |
| --- | --- |
| `src/lib.rs`, `src/game_ffi.rs`, `src/logging.rs` | Entry point, ABI layout/declarations and feature-gated logger reviewed; no new native ABI introduced |
| `src/live.rs` | Hit observation, identity reset, native progress scheduling, single input history, narrow aerial ownership and richer motion diagnostics |
| `ledge_trap.rs`, its tests | Back air, common timing checks, target/plan cleanup, native-clock deduplication and invalid-data handling |
| `ledge_position.rs`, `ledge_trump.rs` | Eligibility/cache ordering, spot/support validation, hitstop and damage cancellation |
| `live_input.rs`, `manual_controls.rs`, `native_input.rs` | Scheduler and ownership/edge composition reviewed; native controller layout/ownership-chain offsets retained |
| `input.rs`, `assist.rs`, `config.rs`, `state.rs`, `tools.rs` | Arbitration diagnostics fixed; rule priority, config validation, observation and preset semantics reviewed |
| `runtime.rs`, `sequence.rs`, `timing.rs` | Physical handback, confirmed sequence runner and shared timing/native-progress utilities |
| Existing examples and host tests | Teaching/live distinctions reviewed; tests cover the new APIs and changes |

## What remains necessary for frame-perfect validation

There is still no verified global simulation-frame counter in the live ABI.
`AnimationClock` counts distinct native progress samples, not elapsed frames;
loops, animation rates, skipped observations and auxiliary timer changes prevent
it from establishing a universal frame clock. It suppresses duplicate polling
and freezes on stop rather than claiming exact global time.

Root-position reach and linear velocity do not model animated hitbox/hurtbox
shapes, gravity, acceleration, stage/platform movement or all collision outcomes.
Animation end may differ from a character's earliest cancel/actionable frame.
Own hit infliction is not tied to a specific victim; multi-opponent confirmation
is ambiguous. Native input acceptance and the frame offset from a mapper request
to move startup also need on-device measurement. These limits prevent a claim
of guaranteed punish/combo coverage.

The next measurements should record both fighters' motion/status, vulnerability
expiry, first accepted move frame, first hit and earliest target defense on both
ledges. Use Pichu and DK as different-sized baselines, then vary drift, damage,
DI/SDI, animation rate and stage geometry. Store measured recovery/cancel and
hitbox profiles by fighter/motion. Feed those verified profiles and a verified
simulation clock into the shared timing and confirmed-sequence APIs before
adding a particular live combo trigger.

## Validation

- Host tests: 145 unit tests and 9 integration tests pass (154 total).
- Core Clippy: all targets, warnings denied, passes.
- Rust formatting: root and core pass.
- Silent and diagnostic Skyline release builds succeed.
- Both NRO headers and logging markers checked after building.
- No Switch gameplay tests, installation, version bump, or replacement of existing release packages.

## Online warm-up transition follow-up

See [the transition hardening notes](INTEGRATION.md#online-warm-up-transition-hardening-source-update).
The mapper now consumes copied snapshots only. Global/manager gameplay readiness,
active/accessor/entry identity checks, session-ticket invalidation and bounded
roster counts guard native observations. Ledge roster scans run only with Y held.
The code and host regressions address plausible crash paths; the reported online
crash has not been reproduced or diagnosed from a crash dump in this environment.
