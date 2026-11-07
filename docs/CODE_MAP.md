# Reading and extending the source

The Switch package connects to Smash. The smaller `kirby_assist_core` crate
contains ordinary Rust values and decisions. Start with the core if raw game
pointers and hooks are new to you.

Ledge recognition and controller routing are shared Rust code. Every fighter
option, initial delay and ordered input schedule is authored only in
`sequences.txt`; there are no fighter option tables or executable menu handlers.
See [fighter configuration](FIGHTER_CONFIGS.md) and [sequence syntax](SEQUENCE_FILES.md).

## Start with one ledge response

Read these files in order. Paths are relative to the repository root; open them
in your editor (the documentation site does not publish Rust source files).

1. `sequence_config.rs` defines the fixed-size recipe data; `sequence_config/parser.rs` reads and validates file sections at startup.
2. `fighter_profiles.rs` freezes file entries into menu lists; `settings_menu.rs` caches selected timelines.
3. `ledge_trap.rs` recognizes the opponent's option and starts only that row's file timeline.
4. `ledge_trap/file_timeline.rs` executes ordered inputs/waits and explicit spatial/block gates.
5. `src/live.rs` publishes requests through the existing input owner/overlay; it does not force moves.

## Where each part lives

Paths below are relative to the project root. Core paths begin with
`crates/kirby_assist_core/src/`.

| File | Responsibility |
| --- | --- |
| `docs/FIGHTER_ADDITIONS.md` | Luigi/Falcon source, timing, performance and validation notes. |
| Core `settings_menu.rs` | Shared per-fighter menu selections, ledge choices, feature-mode rows, bindings, recipe editing and trigger reservation. |
| Core `settings_menu/editor.rs` | Recipe editing, action presets, field selection, and the editor's displayed schedule. |
| Core `settings_menu/editor/commands.rs` | Input-driven copy, paste, reorder, undo, and delete commands for recipe editing. |
| Core `settings_menu/editor/presentation.rs` | Maps each action to editable fields and renders recipe schedule details. |
| Core `button_modes.rs` | Retained legacy mode definitions; the old Y Finisher selector/binding is removed. |
| Core `finisher.rs` | Parked portable starter/follow-up implementation; no live binding or per-frame state. |
| Core `fighter_data.rs` | Exact-kind stats and move-frame facts from the canonical roster JSON, plus native recovery validation. |
| Core `whiff_punish.rs` and `src/live/whiff.rs` | Parked prototype, not wired into the live build/menu. |
| Core `action_owner.rs` | One committed input owner, explicit interruption rules and losing-plan cancellation. |
| `data/Ultimate_stats.json` | Sole source for roster-wide fighter stats and move-frame data; embedded and parsed once at startup. |
| Core `autopilot_guard/duck.rs` | Held-Y crouch/shield ownership; legacy move fixtures remain separate from live geometry. |
| Core `autopilot_guard/duck_geometry.rs` | Allocation-free swept hitbox/body-envelope tests and calibration bounds. |
| `src/live/duck_hits.rs` | Disabled reference implementation; its native hitbox scans, source tracking and ATTACK hook are not enabled. |
| `src/lib.rs` | Skyline entry point; calls installation once. |
| `src/live.rs` | Hook orchestration, profile selection, shared session state and tool priority. |
| `src/live/config.rs` | Startup-only master `sequences.txt` loading/validation; `config.txt` fallback for older files. |
| `src/live/controller.rs` | Native controller layout, offset checks and button-history reads/writes. |
| Core `training_test.rs` | Diagnostic/Training Minus-menu handoff and isolated test settings. |
| `src/live/observations.rs` | Read-only fighter, opponent, physics, roster and floor queries. |
| `src/live/tech.rs` | Native tech eligibility and predicted contact/capture timing. |
| `src/game_ffi.rs` | Declarations of functions supplied by Smash; their memory/calling contract. |
| `src/logging.rs` / `src/grab_diagnostics.rs` | Diagnostic-only queued writer / bounded roll-grab recorder. |
| Core `live_input/config.rs` | Startup settings, defaults, text parsing and validation. |
| `src/calibration.rs` | Diagnostic-only fixed observation recorder, completed result formatting and cached card publication. |
| Core `ledge_trap/diagnostics.rs` | Read-only response metadata for all five rows and detailed roll-grab snapshots. |
| Core `live_input.rs` | Mash/tech request policy and compatibility helpers. |
| Core `gameplay.rs` | Shared `GameplayContext`, `ControllerOutput`, opponent observations, motion timing, and recovery-frame helpers. |
| Core `roll_read.rs` / `roll_read/smash.rs` | Shared roll target facts/constants; fixed punish prototype is parked. Live roll responses use reaction_rules.rs. |
| Core `ledge_trap.rs` | Held-trigger visit history, response selection and native action confirmation. |
| Core `ledge_trap/types.rs` | Ledge-specific settings and opponent range checks. Shared gameplay values live in `gameplay.rs`. |
| Core `fighter_profiles.rs` | Shared per-fighter profile declarations, generic grab references, and startup-only file menu construction; individual public module paths remain available. |
| Core `fighter_tuning.rs`, `fighter_tuning/schema.rs`, and `fighter_tuning/analysis.rs` | Public tuning facade, stable saved-setting/menu schema, and fighter-derived policy defaults. |
| Core `ledge_trap/attempt.rs` | Native-progress clock for ordered input timelines. |
| Core `sequence_config.rs`, `sequence_config/parser.rs`, and `sequence_config/parser/actions.rs` | Keep fixed-size recipe data and the public catalog API in the parent module; parse catalog sections and action tokens in focused startup-only modules. |
| Core `ledge_trap/file_timeline.rs` | File-ordered input holds/waits, native-progress clock, explicit spatial/block predicates. |
| Core `ledge_position.rs` / `ledge_trump.rs` | Grounded stance preparation / run-off ledge trump. |
| Core `manual_controls.rs` / `native_input.rs` | Trigger reservation, shared button-ownership masks, input overlays, and raw button conversion. |
| Core `combat_assist.rs` | Either tilt trip to forward smash, and damage-hitstop SDI. |
| Core `tech_prediction.rs` / `tech_input.rs` | Pure collision timing / shield request and acknowledgment scheduling. |
| Core `timing.rs` / `timing/progress.rs` | Share motion-progress classification, event deduplication, simulation-frame arithmetic and forecast overlap. No learned compensation. |
| Core `observation.rs` / adapter `live/frame.rs` | Lazy fixed-size facts shared across policies for one observation; see [shared observations and timing](OBSERVATIONS_AND_TIMING.md). |
| Core `slide.rs` | Parked portable slide prototype; no live state, observations or input binding. |
| Core `session.rs` / `diagnostics.rs` | Invalidate stale requests between matches / limit diagnostic snapshots. |
| Core `state.rs` / `input.rs` | Generic observation contracts / logical buttons, sticks and merging. |
| Core `config.rs` / `assist.rs` / `tools.rs` | Generic scenario rules / stateless engine / mash and tech presets. |
| Core `runtime.rs` / `sequence.rs` | Generic adapter loop used by examples / sequence helpers. |

Large test modules live beside the corresponding implementation, for example
`ledge_trap/tests.rs`, `ledge_trap/file_timeline/tests.rs`,
`live_input/tests.rs` and `tech_input/tests.rs`.
`#[cfg(test)] mod tests;` includes those files only for test builds. Tiny
existing test modules elsewhere may remain inline.

The generic `AssistRuntime` example is useful for learning the Rust concepts.
The live plugin uses separate callbacks and retained tool state; it does not
run every tool through that example's synchronous loop.

## Rust details that matter here

- A `struct` groups named values. `GameplayContext` is copied fighter data and
  `ControllerOutput` is a requested input packet; `LedgeTrap` remembers ledge
  history between observations.
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

The controlled fighter's kind selects its identity and compiled reference data.
`sequences.txt` supplies the actual ledge/button input recipes and assist rules;
there is no compiled fighter response-list fallback. Universal startup settings
are in `[universal.settings]`, with optional `[fighter.settings]` overrides.
Menu selection and cached resolved rules refer to stable saved recipe IDs.
See [fighter configuration](FIGHTER_CONFIGS.md), [master-file syntax](SEQUENCE_FILES.md)
and [assist rules](REACTION_RULES.md).

## Validation

Run the portable core tests for policy and editor changes:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline
```

For documentation changes, check generated pages and build the site:

```sh
python3 tools/update_fighter_docs.py --check
.venv-docs/bin/python tools/update_cheatsheets.py --check
.venv-docs/bin/python -m mkdocs build --strict
```

Host tests and documentation builds do not verify Switch timing or live-game
compatibility. Follow the relevant feature guide for device-validation details.

## Follow a menu choice

1. `settings_menu.rs` maps fresh menu D-pad presses to row/choice indices.
2. `src/live/settings_menu.rs` resolves cached response settings only on changes.
3. `src/live.rs` reads native context only when a tool needs it, then composes
   requests with the existing higher-priority mash/tech/SDI inputs.
4. `src/live/settings_menu/menu.rs` draws cached labels without game queries.

Live combo playback and fighter combo libraries have been removed. See
[the current controls](LEDGE_MENU.md).

## File-authored defensive responses

`defense_response.rs` in the core crate reuses `InputTimeline` for `shield_hit`
and `grab_dodged` recipes. `out_of_shield.rs` gates a confirmed shield-contact
response; `spot_dodge.rs` records native dodge acceptance before an optional
follow-up. `reaction_rules.rs` caches trigger/range metadata on recipe edits.
These events use their own mode rows, not the roll engine's Menu/button groups.
`fighter_tuning.rs` preserves the tuning API; `fighter_tuning/schema.rs` owns
the stable setting keys, labels, hints, and indices, while
`fighter_tuning/analysis.rs` owns fighter-derived defaults and policy
conversions. Existing `fighter_tuning::*` imports continue to work.
The live adapter follows one confirmed identity without adding a global object
scan, hook or live file read. See [OOS](OUT_OF_SHIELD.md) and
[Grab dodge](GRAB_DODGE.md).

`crates/kirby_assist_core/src/spot_punish.rs` owns configured spot-dodge responses; it reuses the recovery targets and shared input timeline. [Spot punish](SPOT_PUNISH.md) documents its separate list.

### Loader stack headroom

The native plugin loader has a 64 KiB stack. Full configuration loading runs on
one temporary 2 MiB-stack worker, joined before any hooks are installed. It exits
after parsing; thread creation/join errors leave the mod uninstalled. Startup
returns the large shared configuration in a temporary box and publishes it through
a non-inlined helper so the loader does not retain config-sized copies. The box
and worker stack are released at startup. There is no ongoing worker, scan or
per-frame allocation added by this change. Host tests cover the full deployment
catalog on that worker stack; console validation still requires a rebuilt plugin.
