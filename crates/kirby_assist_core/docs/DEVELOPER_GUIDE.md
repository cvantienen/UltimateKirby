# Kirby Assist Core developer guide

`kirby_assist_core` is the portable decision-making part of Kirby Assist. It
contains policy, configuration, input recipes, and tests; it does not install
game hooks, read game memory, or write native controller structures. Keeping
those boundaries separate lets the crate run in normal Rust tests and examples.

Start with `crates/kirby_assist_core/examples/walkthrough.rs` (paths in this
guide are repository-relative unless otherwise stated). It is a small,
host-only frame loop that shows the most compact version of the core API.
This is the canonical beginner Rust tutorial; the maintained site workflow in
`docs/DEVELOPMENT.md` explains how to choose a change and validate it.

## Rust vocabulary, using this project

### Package, crate, module, and re-export

`Cargo.toml` describes a **package**: dependencies, edition, features, and build
targets. A **crate** is one compilation unit, such as a library or example.
The root package builds the Skyline library; its path dependency imports the
separate core library. `cargo test --manifest-path
crates/kirby_assist_core/Cargo.toml` selects the core package, not the plugin.
It is not a workspace-wide test command. Both packages use Rust edition 2021;
that is a language edition, not the plugin version or a minimum compiler claim.

A **module** is a namespace and visibility boundary inside a crate. In core
`lib.rs`, `pub mod shared { pub mod timing; ... }` loads
`src/shared/timing.rs`; inline modules such as `runtime::session` remain inside
their parent source file. `mod` does not run a file or create a thread.
`pub use shared::timing` exposes the same module as `kirby_assist_core::timing`;
it does not copy state or create a second implementation. Root re-exports keep
existing callers working after organization changes. `crate::` starts at the
current crate, `super::` at the parent module, and `self::` at the current one.
`pub(crate)` exposes a name inside its crate; `pub(super)` exposes it to the
parent scope. A private helper need not become public just to be tested.

### Structs, enums, and explicit absence

A **struct** groups fields: `GameplayContext` carries copied facts;
`ControllerOutput` carries a proposal; `AssistRuntime` stores configuration and
permissions. An `impl` block supplies methods for that type. Multiple `impl`
blocks or files do not make multiple instances. `Default` constructs the type's
defined initial value, which is not necessarily an enabled feature or an
available native observation. `ControllerOutput::NONE` is a const-compatible
empty proposal; calling ordinary `Default::default()` inside `const fn` is not
interchangeable with it.

An **enum** is one of named alternatives, optionally with data.
`MotionProgress::Advanced(f32)` carries elapsed game-frame progress;
`Duplicate`, `Transition`, `Rewind`, and `Invalid` have distinct meaning.
A `match` makes handling alternatives explicit. Do not convert invalid clocks
to a default successful transition just to satisfy an exhaustive match.

`Option<T>` means `Some(value)` or `None`, not “zero.”
`ControllerOutput.stick = None` proposes no stick override;
`Some([0.0, 0.0])` actively requests neutral. `ground_brake: None` means no
validated braking value. `frames_until` returns `None` for invalid/unbounded
forecasts; `Some(0.0)` is a valid due-now deadline. The final exclusive-output
boundary can deliberately turn an absent stick into neutral for a committed
sequence, so the proposal and delivered packet need not have identical ownership.

`Result<T, E>` means `Ok(value)` or `Err(error)`: it carries failure information
rather than merely absence. A mutex `lock()` returns a `Result`; callback code
commonly handles `if let Ok(guard)` or `.lock().ok()?` instead of panicking on a
poisoned lock. Configuration parsing is another place where errors need their
line/context, not silent conversion to an enabled default. `?` returns early
from the enclosing function: for `Option` it propagates `None`, for `Result` it
propagates an error. It neither retries nor logs automatically.

### Borrowing, ownership, copies, and traits

Rust **ownership** determines who may use and drop a value. It is different
from gameplay **action ownership**, which chooses the assist allowed to control
buttons. `&T` borrows for reading; `&mut T` borrows exclusively for mutation.
`process_frame(&self, adapter: &mut impl GameAdapter)` reads runtime settings
and permits the adapter's output write. The borrow ends when its scope/use ends;
returning a copied `FrameDecision` does not keep the adapter borrowed forever.

`Copy` permits inexpensive value duplication for types implementing that trait:
`MotionStamp`, IDs, numeric contexts, and `ControllerOutput` can be passed by
value. `Clone` explicitly duplicates a value and can allocate for owned strings
or collections; do not assume every cloned configuration is cheap. `&str`
borrows text, while `String` owns it. Logger `write(&str)` copies into owned
text for the background queue, so do not describe queued logs as allocation-free.

A **trait** defines a contract implemented by a concrete type.
`GameObservationSource` reads observations; `InputBridge` reads human input and
writes merged input; `GameAdapter` requires both and adds `read_context`.
The walkthrough's `DemoAdapter` implements those contracts with ordinary Rust
fields. `impl GameAdapter` accepts any implementing adapter; it is not the
native Switch adapter. Trait bounds such as `K: PartialEq` in `DiagnosticGate`
require comparing snapshots without prescribing their concrete shape.

`Cell` in the callback-local fact cache permits narrowly scoped interior
mutation through `&self`; it is not a thread-safe lock. `Mutex<State>` provides
mutual exclusion and returns a guard released on drop. Atomics communicate
individual gates/counters; they do not freeze every field into one global
snapshot. `OnceLock` publishes one initialized value, but a value containing a
`Mutex<SequenceLibrary>` can still have explicitly synchronized mutable contents.

### Compile-time selection and native boundaries

`#[cfg(test)]` includes code only in a test compilation. A gated `#[path] mod`
or `include!` keeps test bodies under the implementation's private scope while
storing them in `tests/unit/`. `#[cfg(feature = "diagnostic-logging")]` includes
diagnostic code only when Cargo enables that feature. It is independent of
`--release` optimization and runtime settings such as `tech_debug`.
The plugin's feature forwards to the core feature; core-only diagnostic tests
need their own `--features diagnostic-logging` argument.

`unsafe` identifies operations whose preconditions Rust cannot prove. It does
not validate a pointer, establish a correct ABI, or grant permission to alter
fighter state. Native declarations in `src/game_ffi.rs` name game functions;
`extern "C"` chooses a calling convention, and `link_name` chooses a native
symbol. `#[repr(C)]` pins our declared field layout. Offset assertions test
our struct, not the actual executable. Keep native pointers callback-local and
copy validated facts into core values before retaining them.

## Mental model

The program follows this direction:

```text
current game facts + current physical input
                |
                v
native adapter / integration layer
                |
                v
observations, settings, and per-feature state machines
                |
                v
policies propose logical controller output
                |
                v
action ownership and input translation choose one safe result
                |
                v
native adapter applies controller input
```

The core owns everything from observations onward. The integration layer owns
game-specific reads, current input samples, and applying the final result. Do
not put native memory access, physics changes, hitbox changes, animation-rate
changes, or fighter-status changes in this crate. Assistance here must remain
controller-input automation based on read-only observations.

There are two related execution paths:

1. `AssistRuntime::process_frame` is the small generic path used by examples
   and host tests. It reads fresh human input and a `RunContext` plus
   `FighterObservation`, calls `AssistEngine::evaluate`, and writes the merged
   frame. `AssistEngine` is deliberately stateless: the caller supplies each
   scenario's age.
2. The live policy modules hold the richer state needed by gameplay features:
   target tracking, sequence playback, receipt checks, cooldowns, and input
   ownership. They consume native facts through plain Rust data types and emit
   logical requests. The integration layer performs the actual native reads and
   writes at the boundary.

Always keep the three frame values distinct: observed state, fresh physical
input, and proposed output. Never feed last frame's assisted output back in as
the next physical-input sample; that can turn a one-frame request into a stuck
hold.

## Map of the crate

| Area | Responsibility | Start here when... |
| --- | --- | --- |
| `shared/` | Controller ownership, native-input translation helpers, observations, timing, and survival responses. | A feature needs common facts or must safely share input with another feature. |
| `policies/` | Independent gameplay decisions: ledge, reactions, out-of-shield, recovery punishes (`option_read.rs`), defensive responses, posture, and grabs. | You are changing when the assist should respond. |
| `fighters/` | Fighter IDs and data, status tables, profile selection, tuning, plus Kirby and Game & Watch rules. | Behavior depends on the controlled character. |
| `sequence_config/` | The validated sequence catalog, action language, parser, persistence, and playback model. | A saved recipe, template, or `sequences.txt` field changes. |
| `live_input.rs` | Portable parsing and scheduling for live auto-mash and auto-tech settings, including `LiveConfig`/`LiveSettings`. | A shared live input setting or its per-update timing changes. |
| `settings_menu/` | Menu state, bindings, choice lists, editor commands, and display data. | A player-facing menu option or edit flow changes. |
| `manual_controls.rs` | Physical-input history, manual takeover, turbo, and reservation helpers. | A feature must coexist with player input. |
| `config.rs` | Generic scenario rules and the mode/builtin identifiers used by bindings. | You are extending the small, host-testable rule engine. |
| `runtime.rs` | Generic adapter runner, session fencing, and runtime toggle helpers. | You need the host-testable frame runner rather than a gameplay policy. |
| `diagnostics.rs` | Explainable mode/ownership diagnostics. | A new owner or mode needs useful debug output. |

`lib.rs` declares the module tree and keeps compatibility re-exports at the
crate root. Prefer a module-qualified path in new code when it makes ownership
clear; use a root re-export when it is part of the intended public API.
The recovery-punish implementation lives in `policies/option_read.rs`; both
`config::option_read` and the root `option_read` path remain compatible.
Follow those compatibility declarations rather than assuming every policy is
listed by the inline `policies` module in `lib.rs`.
In the native adapter, repository-root `src/diagnostics/performance/health.rs`
owns health counters and report triage; `performance.rs` keeps observation
sampling and the same public `performance::health` entry point.

## How a feature makes a decision

Most gameplay features should follow this shape:

1. Read only the facts the feature needs and represent them with an existing
   observation/context type where possible.
2. Check admission conditions in the relevant policy. Invalid, stale, stopped,
   or unsupported state should produce no request.
3. Return an input request or advance a feature-local state machine. Do not
   write controller state here.
4. Register an `action_owner::Intent` when the request is committed or must
   reserve a release frame. `shared/controller.rs` selects one owner; a
   committed exclusive sequence blocks competing gameplay requests until it is
   released, cancelled, or safely interrupted.
5. Apply the selected logical result at the shared input boundary using
   `shared/native_input.rs` and `manual_controls.rs`. Preserve fresh player
   input outside the buttons and sticks owned by the selected action.
6. Add public-contract regression tests under `tests/`. Put private unit-test
   implementations under `tests/unit/`, retaining a `#[cfg(test)]` module with
   `#[path = "..."]` at the original production scope. For inline nested modules
   whose virtual directory does not exist, keep a minimal gated `include!`
   wrapper instead. Both approaches preserve private access and test names.
   Native unit-test bodies live in the repository-root `tests/unit/`; their
   original modules remain compiled by the existing exact-source host harnesses.

The priority is defined by `action_owner::Owner`, not by module call order.
Safety, committed reads, ledge work, and passive helpers intentionally have
different priority. If you add an owner, update its priority and exclusivity,
the place that constructs intents, diagnostics, and tests for conflicts.

## Character-specific behavior

Character-specific behavior belongs in `fighters/`, with policy orchestration
remaining in `policies/`:

- `fighter_kinds.rs`, `fighter_data.rs`, and `native_states/` provide IDs,
  reference facts, and named status classifications. Add a named constant or
  classifier there instead of scattering raw integer checks.
- `fighter_profiles.rs` combines common settings with the selected fighter's
  profile and builds menu choices from the parsed catalog.
- `fighter_tuning.rs` defines per-fighter policy values. These are input-policy
  adjustments, not changes to native fighter parameters.
- `kirby.rs` and `game_and_watch.rs` contain the character-only state machines
  and observations currently required by those characters.

To add a character-specific feature, first decide whether it is a reusable
policy with character data or truly unique behavior. Put reusable triggering
and ownership in `policies/`, and add a fighter profile, tuning, data, or small
fighter module for the differing facts. Put a fully unique state machine in a
fighter module, expose the smallest useful input/output type, and let the
shared controller boundary arbitrate its result. Do not special-case a fighter
inside unrelated universal policies unless the exception is genuinely part of
that policy's contract.

`Mode::supports` currently marks only `IdleCrouch` as Kirby-only. When adding a
new restricted mode, update that support rule, its menu visibility, and its
tests together.

## Universal modes, saved sequences, and bindings

These terms describe different things:

- **Universal settings** are the `[universal.settings]` section in the sequence
  catalog. The parser accepts one such section; it stores common catalog-level
  settings rather than a playable fighter recipe.
- **Templates** use `[template.<id>]`. They are reusable editor starters, not
  live responses by themselves.
- **Fighter entries** use `[fighter.slot.choice]`, for example
  `[kirby.roll.safe]`. They hold a `Sequence` and appear only in that fighter's
  preset slot. `PresetSlot` lists ledge-response slots and button slots.
- **Mode bindings** are `ModeBinding` values: Ledge trap, Auto-tech, Auto-mash,
  Idle crouch, Tilt face, Spot punish, and Out of shield, each as Hold or
  Toggle. A sequence may carry a mode binding; a `Builtin` is instead a native
  helper selected without running a sequence tape.
- **Automatic reaction rules** are file-authored `reaction` values interpreted
  by `policies/reaction_rules.rs`, then played by the relevant policy such as
  `out_of_shield` or `defense_response`. They are separate from a physical
  button binding.

The catalog parser and its action-token parsing both live in
`sequence_config/parser.rs`; `Sequence` and `Action` are in
`sequence_config.rs`. Parse and validate at startup or save time, never in a
per-frame callback. The parser already bounds file size, entry count, action
count, and sequence duration. Extend the existing grammar through its parser,
validation, persistence, menu rendering, and tests as one change; accepting a
new key alone does not make it playable.

## Adding or extending a universal mode

For a new mode that is available across fighters:

1. Decide whether it belongs in the generic scenario engine, a live input
   utility, or a gameplay policy. Use `AssistEngine` only for a stateless,
   scenario-driven logical recipe; use `policies/` for gameplay state and
   arbitration.
2. Add the policy's data and decision function/state machine. Reuse
   `shared/observations`, `shared/timing`, and `shared/controller` before
   creating parallel helpers.
3. If players can bind it, add the mode identity in `config::button_bindings`,
   then update binding handling, menu labels/visibility, diagnostics, and
   persistence/parser support as needed.
4. Route its logical output through `action_owner` and native-input helpers.
   A mode is incomplete until it has a tested conflict/handback story.
5. Test inactive, eligible, cancelled, and competing-owner paths. Prefer the
   narrow test file closest to the behavior.

For a new scenario-engine rule, also update `Scenario`, its priority, and
`MAX_SCENARIO_RULES` when necessary. `AssistEngine` chooses the highest-priority
active scenario *before* checking whether that scenario has an enabled rule;
do not change that ordering without a regression test.

## Worked extension: a pure deadline-to-input helper

Before adding a new live feature, practice a small decision over copied values.
The following is a teaching helper, **not a currently wired feature**. It asks
for Guard only once a supplied valid motion deadline is due. It does not observe
a collision or establish that Guard is the correct response to that situation.

```rust
use kirby_assist_core::{gameplay::ControllerOutput, native_input::MAPPER_ZR};

fn guard_when_due(frame: f32, deadline: f32, rate: f32) -> ControllerOutput {
    let Some(remaining) = kirby_assist_core::gameplay::frames_until(frame, deadline, rate)
    else {
        return ControllerOutput::NONE;
    };
    if remaining > 0.0 {
        return ControllerOutput::NONE;
    }
    ControllerOutput {
        requested: MAPPER_ZR,
        owned: MAPPER_ZR,
        ..ControllerOutput::NONE
    }
}

assert_eq!(guard_when_due(4.0, 5.0, 1.0), ControllerOutput::NONE);
assert_eq!(guard_when_due(f32::NAN, 5.0, 1.0), ControllerOutput::NONE);
assert_eq!(guard_when_due(5.0, 5.0, 1.0).requested, MAPPER_ZR);
```

Here `let Some(...) else` rejects absence, `remaining > 0` distinguishes waiting
from due now, and struct-update syntax preserves the remaining empty fields.
`requested` and `owned` deliberately name the same Guard bit; no native pointer
or physical-input rewrite appears in this decision. A production scheduler would
also need event identity, stop handling, deduplication, receipt and cancellation;
a bare helper called every poll would repeatedly request Guard. Reuse the
existing `tech_input` scheduler rather than deploying this example as auto-tech.

To turn a genuinely new decision into a complete feature:

1. Add its portable context and policy in the existing owning module, not in
   `game_ffi`. Document units, missing facts, and whether output is a proposal.
2. Add private regressions for invalid/missing data, waiting, due-now, duplicate
   observation, interruption and release. Place bodies under `tests/unit/` and
   preserve a gated include/module at the implementation scope.
3. Find the existing native fact. A callback-local cached field requires a
   `Field`/`Fact` pairing and `cached_reads!` entry; argument-dependent readiness
   or hit/contact queries remain live. Never cache a permission across callbacks.
4. Gate native work, construct a copied context, store only IDs/values in `State`,
   and fence publication with the current session, sequence and mapper tick.
5. Submit an appropriate intent; test held-defense, ledge reservation, safety,
   a committed competing action, and fresh human input. A new priority number
   alone is not a cancellation or delivery policy.
6. Clear the policy and pending output on selection/route/session loss. If the
   feature needs acknowledgment, count only mapped delivery for the matching
   publication, not an observed proposal or repeated poll.
7. Document the setting/parser/menu path and run the matching host regression.
   Native acceptance remains a separately authorized hardware exercise.

## Host validation and private test layout

From the repository root, select the smallest existing test first:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --lib runtime::tests
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --lib timing::tests
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --test shared_observations
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --features diagnostic-logging --lib diagnostics
```

A positional filter matches test names, not source filenames. `--lib` runs core
library tests; `--test shared_observations` selects that integration target.
A file such as `tests/unit/runtime.rs` is **not** an independently discovered
integration target: `src/runtime.rs` includes it as its private `tests` module.
For the inline `runtime::session` module, the gated `include!` keeps private
session tests in that same module. Replacing this with a top-level integration
test would lose private access or force unnecessary public APIs. Keep the include
path relative to the source file; `#[path]` in nested inline modules follows
Rust's module path rules, so use the established wrapper rather than guessing.

Root native tests under `tests/unit/diagnostics/` likewise need the existing
exact-source harness that supplies native stubs. Running desktop tests for the
core does not execute Skyline callbacks. Some legacy `tools/check_*.py` source
extractors still assume code lives in `src/live.rs`; inspect the exact markers
before trusting a helper after a file move. No helper failure authorizes native
writes or changing assertions to hide a regression.

For API comment edits, use the core's focused doctest target when examples are
added to rustdoc:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --doc
```

Commands here are reference workflows, not claims that they ran during a docs
review. Do not build/upload the plugin, fetch dependencies, or run broad checks
without the task's authorization. A missing offline dependency is a blocked
check to report, not a reason to silently fetch it.

## Safe change checklist

- Keep native game observation at the integration boundary and input-only
  assistance in this crate.
- Parse configuration once, validate it, then use typed values in the frame
  loop.
- Treat non-finite or missing observations as unavailable, not as permission
  to act.
- Preserve player input unless the selected owner explicitly owns that control.
- Keep status constants and fighter facts centralized in `fighters/`.
- Add a focused test and run it before broader checks.
- Update this guide when a module's responsibility, execution flow, or
  extension point changes.
