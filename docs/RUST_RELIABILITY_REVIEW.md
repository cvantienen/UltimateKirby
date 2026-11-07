# Runtime reliability and validation

The normal-match audit below describes source changes under package version
2.5.4 on 2026-11-04. Earlier device findings remain historical evidence, not
validation of this source. See [current controls](CONTROLS.md) and
[the source map](CODE_MAP.md).

## Normal-match runtime audit

The reviewed live route is local-controller selection → synchronous native
observation → per-observation copied facts → enabled core policies → one action
owner → shared raw-input overlay → original native mapper → physical-memory
restoration and delivery receipts. Match/session tickets, active object/accessor
identity, configuration/observation revisions and observation freshness fence
publication. Menu changes cancel the old observations before they can publish.
Training passthrough and editing retain their separate gates; ordinary matches
use the same cached recipes without live file parsing or save I/O.

The audit followed automatic grabs, incoming-grab defense and its follow-up,
roll/spot/air-dodge Auto react, shield Up-B backup, ledge reservation/preparation,
tech, mash, DI/SDI, Kirby follow-ups, and their final arbitration with manual
helpers. Geometry/roster work has fixed budgets and native reads occur before
policy publication. A pending state is revisited on later observations; no
reaction function spins waiting for a future animation, range or confirmation.
Authored waits, until-manual holds and native recovery remain intentional.
Mutex contention and native-call cost still require device profiling.

Four gaps were corrected:

- Unconfirmed roll/grab-dodge Up-B now has a fallback limit of **24 distinct
  delivered mapper attempts** if native animation clocks cannot advance.
  Duplicate observations, repeated receipts for the same tick and receipts for
  another target do not consume it. Native Up-B confirmation wins at the limit
  and disables this watchdog, preserving the normal four-frame input and full
  configured tracking duration. Expiry releases control rather than starting a
  blind chase. This is a failure bound, not a delay before acting.
- If you leave the ground before native Up-B startup is confirmed, the
  automatic B/Up request cancels. A grounded proposal alone cannot authorize an
  airborne special. Confirmed startup still retains Bowser's air-spin tracking.
- All automatic Up-B owners share physical shield/jump escape handling,
  including the grab-dodge follow-up and shield-backup tracking. The initial
  grounded B pulse stays protected; a fresh airborne shield input may hand back
  for a manual air dodge. Shield grabs and authored tapes retain their controls.
- Consumed roll events survive missing/reordered observations, avoiding a
  duplicate punish. Missing identities use spare history slots after current
  actors are recorded, so a different fighter reusing a roster slot cannot erase
  an old consumed event. Already-read native status changes clear old roll/grab
  markers so a subsequent event can rearm even if first observed later in its
  animation. Missing data alone never proves an event ended. This reuses the
  existing fact cache and adds no native queries, roster pass or allocation.

Regression coverage includes frozen clocks/hitstop, duplicate mapper calls,
wrong-target receipts, confirmation at the delivery limit, 69 drift frames,
roster gaps/reordering, known versus unknown event endings, both control layouts,
native grab/throw handoff, ledge/safety ownership and scope changes. Portable
checks do not prove native input acceptance, zero false positives/negatives or
Switch frame-time performance. Unobserved transitions, root-distance forecasts
and online input queues remain practical limits. No mod build or upload was
performed for this audit.

## Snapshot compilation fix

The live snapshot groups roll and grab-dodge target IDs into one nested pair,
keeping its outer tuple within Rust's 12-element `Default` implementation limit.
This fixes E0277 without changing any field values, decisions or input timing.
The exact source destructuring pattern was host-compiled against the real core
`Tuning` type and checked with both missing and populated snapshots. This check
is separate from a Switch mod build; the mod was not built or uploaded here.

## Auto react configuration refactor

Package 2.5.5 groups seven editable Auto react sequence types. Unshielded/shielded
grab and shield backup defaults were made explicit for all 77 fighters using
their pre-refactor effective ranges; Bowser's grab-dodge follow-up was copied
exactly into a separate sequence. Every pre-existing recipe and remaining
setting was compared against the original master. The two legacy grab-range
keys moved into the corresponding recipes; old external files retain fallbacks.

Native default execution, movement, shield safety, four-packet grab delivery,
receipt settling, priority and native grab/throw handoff remain in place.
Other edited actions use the same bounded input timeline. Configured receipt
tracking also prevents an R step in a grab-dodge punish from falling through
into B/Up before native acceptance. Recipe references/range metadata come from
the existing revision cache; there are no extra roster passes, native observation
hooks, live file reads or decision-wait frames. The normal native recovery and
explicit waits authored by the user still apply.

Host checks compile and execute the exact changed live snapshot pattern, menu
configuration getters and grab-publication closure against real core types.
This supplements portable parity/menu/safety tests, and specifically guards the
previous tuple-size failure. It is not a complete Switch build or device test.
No mod build or upload was performed for this refactor.

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
menu rows remain removed from gameplay. The current Y mode list selects compiled
held actions (Kirby Auto-duck), with B-reverse unbound. Future sequences
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

The normal-match audit runs the portable core suite plus focused tests with
`diagnostic-logging`; native adapter files receive Rust syntax/format parsing.
Fighter documentation is checked separately. The Switch mod is not built or
uploaded unless explicitly requested. Portable checks validate copied-state
logic; they cannot execute Smash or establish native hook timing.

Before treating a gameplay change as verified, build the intended variant,
confirm the installed artifact, and test both ledges, initial spacing, facing,
stock/match transitions and interrupted inputs. Compare request, mapped input,
native move and actual contact. Root forecasts do not model every animated
hitbox/hurtbox, acceleration, platform or opponent decision. Repeat online
measurements under the connection conditions that exposed the problem.

## Automatic admission and shield handoff follow-up

Ledge activation now suppresses all automatic-punish acquisition immediately.
A shared cheap native-state gate excludes fresh reactions in attack recovery,
dodge/roll, shieldstun, tumble and capture. Existing confirmed Up-B and configured
follow-through track only their locked actor when new acquisition is unavailable.
Capture-link observations are cached once per actor per observation and exclude
captured actors before detailed punish data is read.

Manual Grab suspends automatic requests in both layouts before native catch
confirmation, addressing a path that could mix Up/B into a manual grab packet.
Native catch/hold/throw still releases punish and ledge input tapes. Suspended
delivered Grab retains only its bounded zero-input receipt, preventing a newly
activated script from appending Up/B while that Grab is still in flight.

The guard handoff now quarantines only physical guard that the assist actually
suppressed. A failed direct shield Grab/Up-B which kept Guard cannot quarantine
that same continuous hold. Unconfirmed Up-B cancels at its first non-B output,
including overlapping zero-drift and final-neutral samples, and cancels on lost
readiness or shield damage. Confirmed spin tracking and suppressed-guard air-dodge
protection remain intact. These are identified failure paths, not proof of the
exact cause of an unrecorded console up-throw. No gameplay or frame-time hardware
measurements were performed for this follow-up.

## Shielded versus unshielded grab policy

The shared grab selector now receives the actual shielded-policy flag, separately
from its movement/approach flag. Only unshielded acquisition requires enemy
shield. Shielded acquisition accepts ordinary vulnerable front targets within
its own X/Y and projected-startup limits; downed/tripped bodies remain backup
candidates instead of whiffed grabs. The existing observation pass already
supplies shield-backup candidates on both sides, so this change adds no scans,
waits, input timers or native mutations. It does not alter sequence files.

Selection still runs grab before shield backup, with usable roll/dodge reactions
retaining their existing priority before new grabs. Delivered R keeps its bounded
receipt protection, committed B cannot switch to R, and native own-state gates
remain shared. Regressions cover front nonshield targets, both shield-entry and
held-shield policy, unshielded moving recognition on the first enemy-shield
observation, independent ranges, captured/invulnerable/busy states and exclusive
Grab/Special delivery. Hardware timing remains unmeasured.
