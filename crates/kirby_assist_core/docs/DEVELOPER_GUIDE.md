# Kirby Assist Core developer guide

`kirby_assist_core` is the portable decision-making part of Kirby Assist. It
contains policy, configuration, input recipes, and tests; it does not install
game hooks, read game memory, or write native controller structures. Keeping
those boundaries separate lets the crate run in normal Rust tests and examples.

Start with [`examples/walkthrough.rs`](../examples/walkthrough.rs). It is a
small, host-only frame loop that shows the most compact version of the core
API.

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
| `policies/` | Independent gameplay decisions: ledge, reactions, out-of-shield, defensive responses, posture, and grabs. | You are changing when the assist should respond. |
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
6. Add a focused regression test under `tests/` (or a colocated unit test when
   it tests a private helper).

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
