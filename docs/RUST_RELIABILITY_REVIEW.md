# Runtime reliability and validation

These notes describe the current source under package version 1.5.3, including
edits after the supplied diagnostic test. They replace an older review's
bindings and test counts; those counts did not validate subsequent changes.
See [current controls](CONTROLS.md) and [the source map](CODE_MAP.md).

## Input requests and native acceptance

The live path keeps four events distinct:

1. A response proposes raw buttons and stick.
2. The original controller mapper produces game-visible mapped input.
3. Native status/motion confirms the requested move started.
4. A hit or capture confirms an outcome.

One shared overlay keeps consistent press/hold/release history across tools.
Physical controller memory is restored after native mapping. Readiness, native
animation progression, target/session changes and bounded delivery govern
requests; mapper-poll count is not a global simulation-frame clock.

Short-hop nair pairs neutral X+A and releases X through squat. Other aerials
require their exact native motion. Full-hop recipes wait for takeoff before A.
Captures hand control back without a pummel or throw. Learned per-action input
delay histories, medians and compensation have been removed; live extra delay
is zero. Explicit core fixtures may still supply the scalar `input_delay`.

## Back-air response

Kirby's forward-facing bair uses the backward C-stick attack-cancel route,
including digital direction bits and mapped primer acknowledgment, then X+ZL
jump with forward stick. Already back-facing takes the direct route. Native
`attack_air_b` confirms the aerial; a wrong aerial does not count as success.
Jump bair drifts at 0.35 stick, Neutral bair uses zero drift and a short hop.
Read [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md) for current settings and mapping.

## Roll-grab findings

Supplied logs show that some misses had both mapped CATCH and native grab
startup. Repeated braking outside the useful area delayed requests, and the
common 25-unit point did not match this opponent's actual roll displacement.
Current Kirby/Yoshi walking continues into the area, can extend inward from
observed roll travel and does not chase back toward the ledge. Shared grab
requests check predicted range/facing and protection on the same active frame.
The last source changes have not yet been device-validated.

The diagnostic-only recorder stores a fixed number of numeric checkpoints and
retains request/start/wait evidence. A background worker formats reports; game
hooks do not wait for SD writes. General bulky ledge logs are rate-limited.
See [diagnostic performance](BUILD_VARIANTS.md#diagnostic-performance) for report
interpretation, queue drops and limits. No Switch frame-time profiling has been
performed; bounded logging does not prove zero overhead.

## Frozen result cards

The diagnostic-only [calibration card](CALIBRATION_CARD.md) copies existing
observations into one fixed-size report for all five ledge response families.
The logger worker formats completed text, and the GUI reads a cached card.
It separates configured windows from observed states and unknown metadata.
The source includes recorder/metadata fixtures, but they have not been compiled
or executed for this change. Rustfmt and documentation checks do not substitute
for compiling both feature variants or checking Switch behavior/performance.

## Reusable timing and combo APIs

`timing.rs` provides `ActiveWindow`, `RecoveryWindow` and `PredictedContact`.
It compares protection, reach and recovery at a candidate frame; the actionable
deadline is exclusive. Unknown protection stays unknown. Optional recovery
margins are not guarantees when no native deadline exists.

`sequence.rs` retains generic teaching and confirmed-sequence helpers. These
are not a live combo engine. Fighter combo definitions, combo playback, combo
menu rows and Y-mode cycling have been removed from gameplay. Future sequences
need real action/hit evidence and an appropriate native clock rather than raw
mapper update counts.

## Online warm-up transition follow-up

Session tickets, observation freshness, active object/accessor identity,
controller-chain validation and bounded roster data guard loading transitions.
Fighter discovery comes from callbacks. The mapper can revalidate and refresh
one recently identified local human fighter; it does not enumerate a roster.
Ledge/recovery observations are gated to active tools. A released slide/trump
may finish its committed sequence.

These checks address plausible stale-identity paths. They do not prove the
cause of an unprovided crash dump, certify all plugin combinations or establish
online enforcement safety. This plugin has no authoritative online queue-depth
or future input-execution API.

## Validation

The latest source changes have Rust formatting/static checks and added
regression fixtures. They have not been compiled or run as Rust tests here;
older successful builds and counts do not validate the current tree. The
current docs are checked separately with strict MkDocs.

Before treating a gameplay change as verified, build the intended variant,
confirm the installed artifact, and test both ledges, initial spacing, facing,
stock/match transitions and interrupted inputs. Compare request, mapped input,
native move and actual contact. Root forecasts do not model every animated
hitbox/hurtbox, acceleration, platform or opponent decision. Repeat online
measurements under the connection conditions that exposed the problem.
