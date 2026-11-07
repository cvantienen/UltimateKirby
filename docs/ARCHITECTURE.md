# Runtime architecture

Kirby Assist turns read-only observations of a match into simulated controller
inputs for the selected local human fighter. The runtime does not change native
fighter status, moves, hitboxes, physics, or animation rates. This page follows a
single frame from startup and observation through policy evaluation and input
mapping, then describes how the project shares state and separates platform
integration from portable decision logic.

For the user-facing setup and saved recipe format, see
[installation and compatibility](INTEGRATION.md),
[controls](CONTROLS.md), and [sequence configuration](SEQUENCE_FILES.md).
For source navigation see the [source map](CODE_MAP.md); for input ownership
details see [action ownership](ACTION_OWNERSHIP.md).

## System boundaries

| Component | Responsibility | Main source |
| --- | --- | --- |
| Switch plugin | Startup, Skyline hooks, native observations, runtime coordination, and controller mapping | `src/live.rs` and `src/live/` |
| Portable core | Platform-independent observations, configuration, policy/state machines, and controller decisions | `crates/kirby_assist_core/src/` |
| Fighter data | Native fighter identifiers, profile tuning, and roster facts | `crates/kirby_assist_core/src/fighter_profiles/`, `fighter_kinds.rs`, and `data/Ultimate_stats.json` |
| Recipe configuration | Startup settings and user-authored sequence/action definitions | `deployment/ultimate/kirby_assist/sequences.txt` |
| Documentation generation | Fighter references and standalone HTML cheat sheets derived from maintained sources | `tools/update_fighter_docs.py` and `tools/update_cheatsheets.py` |

The core crate deliberately has no dependency on Skyline, Smash memory, or HID
access. It can therefore be tested on a host machine. The plugin is the adapter
that translates native game facts into core observations and translates logical
decisions back into controller input. The core's adapter/runtime abstraction is
described in `crates/kirby_assist_core/src/runtime.rs`; the plugin's startup and
hook wiring live in `src/live.rs`.

## Startup and configuration

`install()` in `src/live.rs` initializes fighter data, parses and validates the
startup settings and sequence catalog, checks prerequisites, and only then
installs the required hooks. Startup configuration and catalogs are loaded once;
the match loop does not repeatedly read the sequence file.

The local master recipe file is
`deployment/ultimate/kirby_assist/sequences.txt` (installed on the SD card at
`/ultimate/kirby_assist/sequences.txt`). It contains universal settings, fighter
settings, ledge/button recipes, and separate assist-rule sections. `config.txt`
is a compatibility fallback for older sequence files that do not contain
`[universal.settings]`. External file edits take effect after restarting Smash.
Training saves are a separate in-session path and update the live configuration
through the serialized save worker. The exact editor controls and fields are in
[fighter settings](FIGHTER_SETTINGS.md) and the
[sequence syntax reference](SEQUENCE_FILES.md).

## One frame: observation to controller input

The Switch path is split into observation and mapping callbacks. This keeps
native reads out of the controller-mapping transaction and makes the data
flow explicit:

```text
startup configuration
        |
        v
match readiness / hook setup
        |
        v
read-only native observations
        |
        v
copied, fenced observation + fresh human input
        |
        v
feature policies and assist evaluation
        |
        v
intent / ownership arbitration
        |
        v
temporary controller overlay -> native mapper -> restore physical memory
```

### 1. Prepare the live hooks

`observe_commands()` in `src/live.rs` tracks whether the game is ready for
assistance and lazily installs the mapper hook. The mapper hook is not treated as
an always-valid source of game state; match and runtime gates determine when
assistance may be evaluated.

### 2. Capture a valid fighter observation

`observe_fighter()` verifies object identity and activity, local controller
ownership, human participation, and the current session ticket before reading
facts. The observation layer reads only what is needed for that callback:
status, situation, motion and motion frame/rate, position and velocity, facing,
team/stop state, capture links, floor support, roster and opponent facts, ledge
context, attack geometry, and damage trajectory.

`src/live/frame.rs` provides `FrameReads`, a callback-scoped cache for native
facts and shared queries. Portable observation types are in
`crates/kirby_assist_core/src/observation.rs`. Native pointers are not retained
as cross-frame state: the callback publishes copied observations and requests.
This keeps repeated feature reads consistent within a frame and avoids carrying
stale native references into later callbacks.

### 3. Publish through the shared-state boundary

The live `State` in `src/live/state.rs` owns mutable session and feature
orchestration: routing, timing, manual controls, action ownership, feature
instances, outputs, pending requests, and observation/evaluation fences.
`src/live/runtime.rs` resets policies, routes, menus, diagnostics, and pending
inputs when a session is reset. Runtime disable clears participation and pending
work before changing the master gate; it is guarded against unsafe active-match
transitions.

The synchronization mechanisms have separate purposes:

* `STATE: Mutex<State>` protects mutable multi-feature runtime state.
* Atomics carry narrow gates such as runtime, match, and participation state.
* `OnceLock` holds startup data that is initialized once and then read.
* Session tickets, settings epochs, observation sequence numbers, and request
  timestamps fence stale work when ownership or settings change.

Observation setup captures its settings fence together with fighter settings.
If settings or the mapper update change while native facts or the roster are
being read, the callback discards that setup rather than publishing its results.
Controller cancellation also clears pending dash-pivot history.

The current declarations and state fields are in `src/live.rs` and
`src/live/state.rs`; reset and enable/disable behavior are in
`src/live/runtime.rs`.

### 4. Evaluate feature policy

The portable runtime expresses the generic cycle as
`GameAdapter` -> fresh context, human input, and observation ->
`AssistEngine::evaluate()` -> `FrameDecision` -> adapter output. See
`crates/kirby_assist_core/src/runtime.rs`.

`FighterObservation` can represent up to three concurrent scenario samples.
When more than one applies, scenario priority is:

1. Tech
2. Clatter
3. Tumble
4. Hitstun
5. Parry
6. Footstool
7. Ledge trump
8. Landing
9. Ledge
10. Grounded
11. Airborne

`AssistEngine::evaluate()` checks whether the engine and scope are enabled,
selects the highest-priority applicable scenario, checks its rule and timing,
then arbitrates its controller recipe against current human input. A missing
prerequisite leaves human input intact. Rule storage is fixed-size
(`MAX_SCENARIO_RULES = 11`); a defined scenario or future playback slot is not
by itself a wired gameplay feature. See `crates/kirby_assist_core/src/state.rs`,
`assist.rs`, and `config.rs`.

The live plugin also coordinates stateful features that produce their own
controller outputs. Those outputs join the same ownership/arbitration path;
individual features do not bypass the mapper with independent native writes.

### 5. Arbitrate intent and ownership

Feature policies describe desired controller actions, while ownership rules
resolve conflicts and respect the human's input. `ActionOwnership`, `Intent`,
and `Owner` make the winning source explicit. Feature outputs are merged before
the native controller mapper runs. This is the boundary to use when adding a
feature: produce a logical output and register its ownership rather than
mutating the controller directly.

See `src/live/state.rs`, `src/live.rs`, and the detailed
[action ownership guide](ACTION_OWNERSHIP.md).

### 6. Apply and restore the controller overlay

`map_controls()` reads fresh physical input for the selected controller,
combines it with pending policy and manual overlays, invokes the native mapper,
and restores the original controller memory afterward. The mapped result is
what persists; the temporary input overlay does not replace the physical
controller's stored history.

`src/live/controller.rs` pins the controller ABI with `repr(C)`, field offsets,
and compile-time layout assertions. It updates the four button-history words
(`held`, `previous`, `pressed`, `released`) consistently so native edge
detection sees a coherent frame. The mapper transaction is implemented in
`src/live.rs` and the controller helpers.

## Feature families and their boundaries

The current user-facing guides are the authoritative feature walkthroughs. The
core crate also contains reusable or dormant policy modules; the presence of a
module alone does not mean that a feature is exposed in the menu or active in
the live callback.

| User-visible area | Typical work performed | User guide |
| --- | --- | --- |
| Configuration and saved responses | Parse startup settings and per-fighter input recipes; select and edit responses | [Controls](CONTROLS.md), [ledge menu](LEDGE_MENU.md), [sequence files](SEQUENCE_FILES.md) |
| Reaction assists | Evaluate situational rules separately from manually selected recipes | [Reaction rules](REACTION_RULES.md), [Roll read](ROLL_READ.md), [Spot punish](SPOT_PUNISH.md), [Out of shield](OUT_OF_SHIELD.md), [Grab dodge](GRAB_DODGE.md) |
| Ledge and recovery | Track ledge context and run configured/manual controller responses | [Ledge autopilot](LEDGE_TRAP.md), [ledge trump](LEDGE_TRUMP.md), [vulnerable ledge](TWO_FRAME.md) |
| Defense and hitstun | Use controller-only defense, tech, mash, and directional influence policies | [Shield pressure](SHIELD_PRESSURE.md), [automatic tech](AUTO_TECH.md), [analog mash](ANALOG_MASH.md), [DI/SDI](DI_SDI.md), [trip follow-up](TRIP_AND_SDI.md) |
| Combat and training | Run input recipes and training observations without changing native moves | [Input recipes](INPUT_RECIPES.md), [jab-lock chase](JAB_LOCK.md), [training monitor](TRAINING_MONITOR.md) |
| Roster behavior | Choose fighter profiles and inspect generated references and authored notes | [Fighter overview](FIGHTERS.md) |

Feature-specific delays, thresholds, toggles, and supported actions belong in
those individual guides and in the active configuration reference. Do not copy
old values into a new overview; update the canonical feature page when behavior
changes.

## Timing and reset model

There is no single clock that should be treated as universal. Different work is
fenced by native motion progress, scenario-active age, mapper ticks, match/session
tickets, request timestamps, and observation/evaluation sequence numbers.
Hitstop, duplicate observations, idle motion loops, and session resets affect
which clocks are allowed to advance. The shared policy is documented in
[observations and timing](OBSERVATIONS_AND_TIMING.md); feature-specific timings
belong beside the feature behavior.

Reset paths cancel in-flight decisions and clear pending overlays so an old
observation cannot authorize an input after a selection, session, or runtime
state change. When changing timing, update the relevant clock and fence tests;
do not substitute wall-clock or HID polling assumptions without verifying the
native callback semantics.

## Code map

| Concern | Source location |
| --- | --- |
| Hook installation and live callback orchestration | `src/live.rs` |
| Per-callback observation reads and caching | `src/live/frame.rs`, `src/live/observations.rs` |
| Shared live state and feature instances | `src/live/state.rs` |
| Reset and runtime gates | `src/live/runtime.rs` |
| Controller ABI and history | `src/live/controller.rs` |
| Portable policy/configuration modules | `crates/kirby_assist_core/src/lib.rs` |
| Core observations and scenario priority | `crates/kirby_assist_core/src/observation.rs`, `state.rs` |
| Assist evaluation and rule timing | `crates/kirby_assist_core/src/assist.rs`, `config.rs` |
| Portable runtime adapter contract | `crates/kirby_assist_core/src/runtime.rs` |

For a task-specific module map use [CODE_MAP.md](CODE_MAP.md). For the safe
workflow to add a capability, continue to
[extending the project](DEVELOPMENT.md). For actor/session checks, bounded
receipts, cancellation, and the distinction between recipes and generic
playback, see [runtime safety and feature limits](RUNTIME_SAFETY.md).
