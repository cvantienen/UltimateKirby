# Reading and extending the source

The Switch package connects to Smash. The smaller `kirby_assist_core` crate
contains ordinary Rust values and decisions. Start with the core if raw game
pointers and hooks are new to you.

The source is split into observation, configuration, prediction and response
layers. The existing input recipes now receive fighter-specific timings and
response choices from compiled fighter modules. Static menu entries bind labels
to named Rust functions; numeric settings are separate. See [fighter profiles](FIGHTER_CONFIGS.md) for editing it.

## Start with one ledge response

Read these files in order. Paths are relative to the repository root; open them
in your editor (the documentation site does not publish Rust source files).

1. **Ledge data types** — `crates/kirby_assist_core/src/ledge_trap/types.rs`:
   `LedgeSettings` is tuning; `LedgeContext` describes us; `Opponent` describes
   the target; `LedgeOutput` is the requested stick/buttons.
2. **Ledge state machine** — `crates/kirby_assist_core/src/ledge_trap.rs`:
   `sample` handles activation and duplicate observations. `evaluate` checks
   confirmations, follows one opponent and dispatches a response.
3. **Aerial responses** — `crates/kirby_assist_core/src/ledge_trap/aerial.rs`:
   `respond_to_normal_nair` schedules neutral get-up coverage;
   `respond_to_jump` starts ledge-jump coverage; `continue_committed_nair`
   completes a requested hop using our own native action state.
4. **Input overlay** — `crates/kirby_assist_core/src/manual_controls.rs`:
   `ManualOverlay::apply_controls` combines the selected proposal with fresh
   physical input and consistent button history.
5. **Live integration** — `src/live.rs`: `observe_fighter` collects observations
   and publishes requests; `map_controls` temporarily applies them around the
   game's original controller mapper, then restores physical controller memory.

```text
startup: config.txt + compiled Rust catalogs -> validated settings/functions
                                               |
                                 controlled fighter kind -> profile

game observations -> LedgeContext + Opponent
                               |
                               v
                       LedgeTrap::sample
                               |
                 confirm / select / respond
                               |
                               v
                         LedgeOutput
                               |
          physical input -> shared overlay -> native mapper
                                                 |
                            mapped-input acknowledgment
                                                 |
                                  later native action observation
```

An input request, a mapped input, an accepted animation and a confirmed hit are
four different events. Sending A does not prove an attack started. Seeing an
attack animation does not prove it connected. Keeping those events separate
is especially important when controller mapping and fighter observations occur
at different times online.

## Where each part lives

Paths below are relative to the project root. Core paths begin with
`crates/kirby_assist_core/src/`.

| File | Responsibility |
| --- | --- |
| `docs/FIGHTER_ADDITIONS.md` | Luigi/Falcon source, timing, performance and validation notes. |
| Core `ledge_menu.rs` | Five saved response choices, L+R3 menu chord, Minus mash toggle and trigger reservation. |
| `src/lib.rs` | Skyline entry point; calls installation once. |
| `src/live.rs` | Hook orchestration, profile selection, shared session state and tool priority. |
| `src/live/config.rs` | Startup-only I/O for config.txt; initialize/validate compiled fighter catalog. |
| `src/live/controller.rs` | Native controller layout, offset checks and button-history reads/writes. |
| `src/live/observations.rs` | Read-only fighter, opponent, physics, roster and floor queries. |
| `src/live/tech.rs` | Native tech eligibility and predicted contact/capture timing. |
| `src/game_ffi.rs` | Declarations of functions supplied by Smash; their memory/calling contract. |
| `src/logging.rs` / `src/grab_diagnostics.rs` | Diagnostic-only queued writer / bounded roll-grab recorder. |
| Core `live_input/config.rs` | Startup settings, defaults, text parsing and validation. |
| `src/calibration.rs` | Diagnostic-only fixed observation recorder, completed result formatting and cached card publication. |
| Core `ledge_trap/diagnostics.rs` | Read-only response metadata for all five rows and detailed roll-grab snapshots. |
| Core `live_input.rs` | Mash/tech request policy and compatibility helpers. |
| Core `ledge_trap.rs` | Held-trigger visit history, response selection and native action confirmation. |
| Core `ledge_trap/types.rs` | Settings, copied observations, output values and animation/physics helpers. |
| Core `fighter_profiles.rs` | Typed compiled fighter data, profile selection and catalog validation. |
| Core `fighter_profiles/kirby.rs` | Built-in Kirby compatibility data. |
| Core `ledge_trap/attempt.rs` | Native-progress timing, bounded retries and action acceptance for new response variants. |
| Core `ledge_trap/fighters/kirby/variants.rs` | Grounded Stone and one down tilt followed by shield. |
| Core `ledge_trap/fighters/` | Resolve our fighter’s handler and dispatch its configured get-up/drop responses. |
| Core `ledge_trap/aerial.rs` | Ledge-jump and neutral-get-up nair requests and hop completion. |
| Core `ledge_trap/roll.rs` | Fixed-point smash navigation, inward walking-grab tracking, reach/timing checks and input requests. |
| Core `ledge_trap/shield.rs` | Get-up-attack block, grab response and rear-side hop. |
| Core `ledge_trap/grounded.rs` | Hanging stance, legacy get-up tilts and shared ground movement. |
| Core `ledge_trap/prediction.rs` | Approximate reach, braking and attack-window overlap calculations. |
| Core `ledge_position.rs` / `ledge_trump.rs` | Grounded stance preparation / run-off ledge trump. |
| Core `manual_controls.rs` / `native_input.rs` | Trigger reservation and shared input overlay / raw button conversion. |
| Core `combat_assist.rs` | Trip-to-forward-smash follow-up and damage-hitstop SDI. |
| Core `tech_prediction.rs` / `tech_input.rs` | Pure collision timing / shield request and acknowledgment scheduling. |
| Core `timing.rs` | Forecast overlap and deduplicate native progress. Learned input-delay compensation has been removed. |
| Core `slide.rs` | D-pad slide sequence. |
| Core `session.rs` / `diagnostics.rs` | Invalidate stale requests between matches / limit diagnostic snapshots. |
| Core `state.rs` / `input.rs` | Generic observation contracts / logical buttons, sticks and merging. |
| Core `config.rs` / `assist.rs` / `tools.rs` | Generic scenario rules / stateless engine / mash and tech presets. |
| Core `runtime.rs` / `sequence.rs` | Generic adapter loop used by examples / sequence helpers. |

Large test modules live beside the corresponding implementation, for example
`ledge_trap/tests.rs`, `live_input/tests.rs` and `tech_input/tests.rs`.
`#[cfg(test)] mod tests;` includes those files only for test builds. Tiny
existing test modules elsewhere may remain inline.

The generic `AssistRuntime` example is useful for learning the Rust concepts.
The live plugin uses separate callbacks and retained tool state; it does not
run every tool through that example's synchronous loop.

## Rust details that matter here

- A `struct` groups named values. `LedgeContext` is copied data; `LedgeTrap`
  remembers history between observations.
- `&mut self` lets a method update that history. Splitting `impl LedgeTrap`
  across files still operates on the same instance.
- `mod aerial;` loads a child module from a source file. It does not start a
  thread, reset state or run the file on every frame.
- `pub(super)` makes a helper available within its parent module's scope.
  `pub use` keeps an existing public import working after moving a definition.
- `Option<T>` explicitly represents a value or its absence. For a stick,
  `None` leaves physical movement alone; `Some([0.0, 0.0])` requests neutral.
  For a deadline, `None` means unknown; `Some(0.0)` means due now.
- Button masks combine bits with `|`. `requested` says what to hold; `owned`
  says what the assist may replace, including releasing a physical hold.
- World units measure positions/distances. Stick values are normalized
  directions. Native animation frames, mapper ticks and elapsed milliseconds
  are separate clocks; do not substitute one for another.
- `unsafe` marks operations whose assumptions Rust cannot verify, such as
  dereferencing a game pointer. `repr(C)` and padding describe native memory
  layout; normal cleanup rules do not permit reordering those fields.

## Fighter-specific code and numeric tuning

The controlled fighter’s native kind selects a compiled profile and its own
response function lists. Opponent get-up timing remains a native observation.
`ledge_trap/fighters/kirby.rs` registers functions such as
`respond_to_roll_down_smash()` directly in the static menu catalog.
`fighter_profiles/kirby/timing.rs` contains numeric move defaults. Five saved
indices in `ledge_menu.rs` choose compiled responses. See [the fighter source
guide](FIGHTER_CONFIGS.md) to add a response or fighter. Only shared settings and
supported numeric adjustments come from config.txt.

## Validation for the profile change

Formatting parses both Rust module trees. Catalog, selection and policy
regressions cover shipped baseline values, invalid profiles, disabled options
and changed move scheduling. They have been added but not compiled/run under
the standing no-build instruction. Dependency lockfiles were resolved offline.
The packaging/upload preparation tests use temporary fixtures, mocked builds
and fake FTP; no actual project package or Switch connection is created.

No plugin build, compiled Rust tests or deployment was performed. Static checks
do not establish runtime or Switch compatibility.

## Follow a menu choice

1. `ledge_menu.rs` maps fresh menu D-pad presses to row/choice indices.
2. `src/live/ledge_menu.rs` resolves cached response settings only on changes.
3. `ledge_trap/fighters/<fighter>.rs` dispatches the selected compiled function.
4. `src/live.rs` reads native context only when a tool needs it, then composes
   requests with the existing higher-priority mash/tech/SDI inputs.
5. `src/live/ledge_menu/menu.rs` draws cached labels without game queries.

Live combo playback and fighter combo libraries have been removed. See
[the current controls](LEDGE_MENU.md).
