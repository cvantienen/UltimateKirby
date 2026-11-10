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
4. `policies/ledge_trap/file_timeline.rs` executes ordered inputs/waits and explicit spatial/block gates.
5. `src/live/observation.rs` publishes current, copied observations to feature state; `src/live/mapper.rs` resolves ownership and prepares controller requests for the overlay. Neither path forces moves.

## Where each part lives

Paths below are relative to the project root. Core paths begin with
`crates/kirby_assist_core/src/`.

Core layout: `shared/` (controller, observation, native-input, timing and survival
contracts), `fighters/` (reference data, profiles, native states, fighter-specific
rules), `policies/` (ledge, out-of-shield, reaction and other assistance policies),
plus `config.rs`, `sequence_config.rs`, `settings_menu.rs`, `manual_controls.rs`,
`runtime.rs`, `live_input.rs` and `diagnostics.rs`. Small helper modules are inline
in their parent file; `lib.rs` re-exports every module at its original crate-root path.

The Switch plugin keeps live adapters flat in `src/live/`, with menu rendering
and persistence in `src/live/settings_menu/`, and diagnostic-only recording,
logging and performance instrumentation in `src/diagnostics/`. Tech capture
timing, menu text caching and health counters are grouped with their parents.

| File | Responsibility |
| --- | --- |
| `src/lib.rs` | Skyline plugin entry point; calls `live::install()`. |
| `src/live.rs` | Hook declarations, startup setup, top-level hook callbacks, and the original mapper transaction. |
| `src/live/config.rs` | Startup-only configuration and sequence loading; compatibility fallback handling. |
| `src/live/observation.rs` | Fighter observation admission, policy snapshot, callback orchestration, and publication fencing. |
| `src/live/frame.rs` | Callback-scoped `FrameReads` cache; avoids repeating native reads within one observation. |
| `src/live/observations.rs` / `src/live/fighter_roster.rs` | Read-only native fighter/opponent, roster, movement, floor, and route observations. |
| `src/live/mapper.rs` | Controller callback admission, menu/input updates, assist proposals, ownership arbitration, and overlay preparation/finalization. |
| `src/live/state.rs` / `src/live/runtime.rs` | Retained feature/session state and cancellation, reset, and runtime participation rules. |
| `src/live/controller.rs` | Native controller layout, selection, button-history access, and overlay restoration. |
| `src/live/settings_menu.rs` and `src/live/settings_menu/` | Live menu settings, rendering, and serialized save-result handling. |
| `src/live/training.rs` / Core `runtime.rs` | Training passthrough lifecycle / generic portable adapter loop. |
| `src/live/tech.rs` / `src/live/kirby_followups.rs` / `src/live/reflect.rs` | Native observations and adapters for tech, Kirby follow-ups, and reflection. |
| `src/live/attack_sources.rs` / `src/live/projectile_sources.rs` | Bounded read-only attack-source tracking for ledge defense and projectile reflection. |
| `src/game_ffi.rs` | Native Smash function declarations and ABI boundary. |
| `src/diagnostics.rs` and `src/diagnostics/` | Diagnostic-build-only logging, calibration, grab recording, and performance tools. |
| `crates/kirby_assist_core/src/lib.rs` | Portable core modules and compatibility re-exports. |
| Core `config.rs`, `live_input.rs` (`live_input/config.rs` holds `LiveConfig`), and `sequence_config.rs` | Settings and recipe data models, startup parsing, validation, and persistence. |
| Core `settings_menu.rs` and `settings_menu/` | Portable menu state, choice navigation (`input.rs` input state machine, `panel.rs` recipe panel), recipe editor, commands, and presentation. |
| Core `shared/controller.rs` / `shared/native_input.rs` | Logical controller actions, ownership, arbitration, and native-independent button translations. |
| Core `shared/observations.rs` / `shared/timing.rs` / `shared/survival/` | Copied gameplay facts, timing contracts, and reusable survival policies. |
| Core `fighters.rs` and `fighters/` | Fighter catalog, profile/tuning data, native status tables, and fighter-specific policies. |
| Core `fighters/native_states/fighter_status.rs` | Fighter-specific native status IDs; classification uses the verified per-fighter tables in `native_states/kind_statuses.rs`. |
| Core `policies.rs` and `policies/` | Assist policies, including ledge, reaction, out-of-shield, posture, dodge, and defense. |
| Core `policies/ledge_trap.rs` / `policies/ledge_trap/` | Ledge response selection, visit ownership, native-progress clocks, and file-authored timeline execution. |
| Core `policies/out_of_shield.rs` / `policies/out_of_shield/` | Out-of-shield target validation, grab response, and shield Up-B fallback. |
| Core `policies/autopilot_guard.rs` | Shared hitbox shape, incoming-attack envelope, and held-shield timing. |
| Core `manual_controls.rs` / `training_test.rs` / `runtime.rs` | Manual input behavior, training test policy, and portable runtime coordination. |
| Core `fighter_data.rs` / `fighters/fighter_profiles.rs` / `fighters/fighter_kinds.rs` | Canonical roster facts, fighter profiles, and native fighter identifiers. |
| `data/Ultimate_stats.json` | Canonical roster-wide fighter stats and move-frame facts, parsed at startup. |
| `deployment/ultimate/kirby_assist/sequences.txt` | Canonical user-authored sequence and settings configuration. |
| `docs/DEVELOPMENT.md` / `docs/ARCHITECTURE.md` | Beginner change/debug workflow / runtime data flow and synchronization boundaries. |

Separate test-named modules live under each package's `tests/unit/` directory,
with subdirectories mirroring their implementations. The implementations
include them with `#[cfg(test)]` and `#[path = "..."]`, preserving access to
private items without compiling them into the runtime. Existing inline tests
and other regression modules may remain in source; integration tests remain
directly under `tests/`.

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
3. `src/live/observation.rs` reads native context only when a tool needs it;
   `src/live/mapper.rs` composes requests with current ownership and mapper input.
4. `src/live/settings_menu/menu.rs` draws cached labels without game queries.

Compiled fighter combo libraries have been removed. Configured input timelines
remain available for supported ledge and button responses. See
[the current controls](LEDGE_MENU.md).

## File-authored defensive responses

`policies/defense_response.rs` in the core crate reuses `InputTimeline` for
`shield_hit` and `grab_dodged` recipes. `policies/out_of_shield.rs` gates a
confirmed shield-contact response; `policies/spot_dodge.rs` records native dodge
acceptance before an optional follow-up. `policies/reaction_rules.rs` caches
trigger/range metadata on recipe edits.
These events use their own mode rows, not the roll engine's Menu/button groups.
`fighters/fighter_tuning.rs` contains the public tuning API, stable setting
schema, and fighter-derived defaults and policy conversions. Existing
`fighter_tuning::*` imports continue to work.
The live adapter follows one confirmed identity without adding a global object
scan, hook or live file read. See [OOS](OUT_OF_SHIELD.md) and
[Grab dodge](GRAB_DODGE.md).

`crates/kirby_assist_core/src/policies/spot_punish.rs` owns configured spot-dodge responses; it reuses the recovery targets and shared input timeline. [Spot punish](SPOT_PUNISH.md) documents its separate list.

### Loader stack headroom

The native plugin loader has a 64 KiB stack. Full configuration loading runs on
one temporary 2 MiB-stack worker, joined before any hooks are installed. It exits
after parsing; thread creation/join errors leave the mod uninstalled. Startup
returns the large shared configuration in a temporary box and publishes it through
a non-inlined helper so the loader does not retain config-sized copies. The box
and worker stack are released at startup. There is no ongoing worker, scan or
per-frame allocation added by this change. Host tests cover the full deployment
catalog on that worker stack; console validation still requires a rebuilt plugin.
