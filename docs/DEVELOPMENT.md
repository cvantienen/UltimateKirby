# Extending Kirby Assist

This guide describes the supported path for changing the Rust implementation
without breaking the read-only-observation/controller-input boundary. Read the
[runtime architecture](ARCHITECTURE.md) first, then use the
[source map](CODE_MAP.md) to find the owning module.

## Development boundaries

The repository has two Rust layers:

* `crates/kirby_assist_core/` contains portable state, configuration, policy,
  and controller decisions. It can be exercised with host tests and should not
  depend on Skyline or Smash memory.
* `src/` is the Switch plugin integration. It reads native game facts, validates
  runtime/session/controller ownership, connects core decisions to live feature
  state, and applies controller output through the native mapper.

Keep automation input-only. Read native state to detect a situation, but express
the response as controller input. Do not alter native fighter status, physics,
hitboxes, move definitions, or animation rates.

## Beginner guide: follow the code before changing it

The safest way to understand a feature is to follow one input from its trigger
to the final controller packet. The project is split between portable policy
and Switch integration; a feature is not complete just because its policy
function exists.

### Read the execution path

1. **Start at `src/lib.rs`.** Skyline calls `live::install()` from the plugin
   entry point.
2. **Read `src/live.rs` for hook setup.** `install()` loads and validates
   configuration, checks executable and manager prerequisites, and registers
   hooks. `observe_commands()` forwards command flags unchanged and asks the
   observation adapter to run only when assistance is enabled.
3. **Follow the observation callback.** `src/live/observation.rs` admits a
   fighter callback, snapshots policy/configuration, collects copied facts,
   evaluates live feature state, and publishes only if the session and input
   sample are still current. `src/live/frame.rs` caches reads for that callback;
   `src/live/observations.rs` and `src/live/fighter_roster.rs` hold native
   read-only queries.
4. **Find the policy owner.** Portable decisions belong in
   `crates/kirby_assist_core/src/` wherever possible. For example, feature
   policies live in `policies/`, shared controller contracts live in
   `shared/`, and fighter-specific decisions live in `fighters/`.
5. **Follow ownership and delivery.** `src/live/state.rs` retains runtime state.
   `src/live/mapper.rs` refreshes and admits the selected controller, arbitrates
   and applies requests. `src/live/controller.rs` owns native controller
   access. `map_controls()` in `src/live.rs` brackets the original mapper call
   with temporary overlay and physical-input restoration.
6. **Check cancellation and configuration.** Match/session resets are in
   `src/live/runtime.rs` and `src/live/state.rs`; menu/save behavior is in
   `src/live/settings_menu.rs` and its children. Startup parsing is in
   `src/live/config.rs` and the core `live_input.rs`/`sequence_config.rs`.

The complete callback lifecycle, data fences, and ownership boundaries are in
[runtime architecture](ARCHITECTURE.md). Use the [source map](CODE_MAP.md) to
jump to the current module owner.

### Change a feature or fix a behavior

1. **Reproduce and classify the issue.** Record the selected fighter/controller,
   match mode, relevant setting or recipe, expected result, actual result, and
   whether it happens consistently. Separate “trigger not detected,” “policy
   rejected,” “lost ownership,” “stale/cancelled request,” and “native game did
   not accept the input”; these are different failure points.
2. **Trace the path in order.** Search from the UI/config entry to the live
   observation, portable decision, ownership result, and mapper application.
   Follow existing tests and documentation before adding a new abstraction.
3. **Fix the owning layer.** Put general logic over copied Rust values in the
   core. Keep native reads, pointers, ABI details, and callback/session
   integration in the plugin. Avoid parallel implementations of range checks,
   clocks, parser behavior, or ownership.
4. **Preserve the boundaries.** Native reads are observations, not permission
   to write game state. Emit controller intent through the existing output and
   ownership path. Honor physical input, feature toggles, selected-controller
   routing, session/settings fences, and cancel/reset behavior.
5. **Add a regression test at the narrowest boundary.** Test portable policy
   decisions in the core crate's unit/integration tests. Test file parsing and
   configuration with the parser tests. For plugin wiring, native query
   constraints, or input publication, use a matching source-contract guard
   under `tools/` when one exists; these guards target specific code patterns,
   so verify its assumptions before relying on it. Do not assume the Skyline
   plugin can be host-built. Include the negative case that previously failed
   and the adjacent no-trigger/stale-input case.
6. **Update documentation at the source of truth.** Settings, defaults, and
   units belong in the configuration/feature guides. Update generated fighter
   pages through their generator, never by editing generated sections directly.
7. **Run targeted checks, then broader relevant checks.** Start with the core
   crate tests or specific existing test target, validate changed configuration,
   and run strict documentation checks for doc changes. Host success does not
   prove Switch timing; do not claim hardware behavior without device evidence.

When stuck, use the test harness and diagnostic build to observe each boundary
in order. Do not start by adding logs or native reads to a hot callback:
diagnostic sampling is conditional and rate-limited, and repeated native scans
can change performance or callback behavior. Check [runtime safety](RUNTIME_SAFETY.md),
[observations and timing](OBSERVATIONS_AND_TIMING.md), and
[action ownership](ACTION_OWNERSHIP.md) before changing these contracts.

## Add a gameplay capability

Use this sequence for a new assist or a substantial extension:

1. **Define the observable trigger.** Identify the native facts required and
   where they are read in `src/live/frame.rs`,
   `src/live/observations.rs`, or `src/live/fighter_roster.rs`. Read them once
   through the callback-scoped observation layer; do not retain native pointers
   or add repeated scans without a measured need.
2. **Choose the policy boundary.** If the decision can be expressed from copied
   observations and logical input, implement it in the core crate. Use a
   live-only feature state machine only when the policy depends on a
   Switch-specific integration detail.
3. **Model state and configuration explicitly.** Put portable configuration and
   policy state with the core feature. Put live session bookkeeping in
   `src/live/state.rs`. Reuse existing timing, settings, and sequence helpers
   rather than adding a second parser or clock.
4. **Wire observation to evaluation.** A scenario declaration or a new module
   does not activate itself. Connect detection, policy evaluation, reset/cancel
   behavior, and configuration loading in the live orchestration path.
5. **Produce a logical controller decision.** Route output through the existing
   `Intent`/`Owner`/`ActionOwnership` arbitration and mapper pipeline. Define
   what happens when the user is holding a conflicting input, another assist
   owns the action, prerequisites are missing, or the feature is disabled.
6. **Fence stale work.** Selection changes, match transitions, runtime disable,
   and session reset must invalidate observations and pending requests that
   could otherwise emit an input later.
7. **Add tests at both boundaries.** Test portable policy behavior in the core
   crate, then test observation/wiring or lifecycle behavior in the relevant
   live integration tests. Cover the negative path (no trigger, missing facts,
   stale request, and human-input conflict) as well as a successful response.
8. **Document operation.** Update the relevant user guide with its trigger,
   eligibility, controls, settings, timing model, and limitations; link it from
   the feature hub or navigation if it is a major feature.

The integration points are visible in `src/live.rs`, live feature state in
`src/live/observation.rs`, `src/live/mapper.rs`, and `src/live/state.rs`, reset
semantics in `src/live/runtime.rs`, and portable evaluation in
`crates/kirby_assist_core/src/policies/assist.rs` and `runtime.rs`.
See [action ownership](ACTION_OWNERSHIP.md) and
[observations and timing](OBSERVATIONS_AND_TIMING.md) before introducing new
arbitration or clock behavior.

## Extend a configured input recipe

For a change expressible as a saved action sequence, prefer the existing recipe
system over a new hard-coded feature:

1. Edit the canonical file at
   `deployment/ultimate/kirby_assist/sequences.txt`.
2. Follow the established section and action syntax in
   [sequence files](SEQUENCE_FILES.md) and verify action names in the
   [sequence syntax cheat sheet](SEQUENCE_SYNTAX_CHEATSHEET.md).
3. Check that the selected fighter and scenario use the intended section, and
   that the action remains a controller request rather than a native game-state
   edit.
4. Run the sequence validator and any relevant parser/core tests before
   distributing the configuration.

Universal settings, fighter-specific settings, manual ledge/button responses,
and reaction assists are distinct configuration surfaces. Reaction-rule lists
must remain separate from normal ledge/button selectors. See
[reaction rules](REACTION_RULES.md) for rule-specific behavior.

## Customize settings and timing

Use the maintained configuration and feature pages as the source of truth:

* Universal and fighter settings are loaded at startup from the sequence file.
  `config.txt` is only a fallback for older sequence files without
  `[universal.settings]`.
* External file edits require restarting Smash. Training saves update the
  running session through the serialized save path.
* Recipe delays and movement values are action/configuration values; reaction
  rules have their own trigger, scope, priority, and timing semantics.
* The runtime uses native motion progress, scenario-active ages, mapper ticks,
  request timestamps, and session/observation fences for different purposes.
  Do not assume one frame count or wall-clock timer applies to all features.
* Document feature-specific defaults and units beside the feature. When a
  setting is added or renamed, update parsing, validation, generated references
  where applicable, the user-facing settings guide, and tests together.

See [fighter settings](FIGHTER_SETTINGS.md),
[sequence configuration](SEQUENCE_FILES.md), and
[observations and timing](OBSERVATIONS_AND_TIMING.md).

## Add or update a fighter reference

Fighter pages are generated from the registered profile catalog, canonical
fighter data, and saved sequence choices. Do not hand-edit generated sections:
update the source data or generator inputs and regenerate. Keep authored
technique/calibration notes outside generated markers. The generator rejects
unknown profile syntax and supports a check-only mode.

```sh
python3 tools/update_fighter_docs.py
python3 tools/update_fighter_docs.py --check
```

The roster overview is [FIGHTERS.md](FIGHTERS.md), and generation rules are in
[maintaining the docs](MAINTAINING.md).

## Test and validate

Run the smallest relevant check first. Portable Rust policy and parser tests use
the core crate:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline
```

Validate sequence configuration with:

```sh
python3 tools/sequence_validation.py
```

Build the documentation site and verify its links/navigation with:

```sh
.venv-docs/bin/python -m mkdocs build --strict
```

Check generated pages and standalone cheat sheets for drift:

```sh
python3 tools/update_fighter_docs.py --check
.venv-docs/bin/python tools/update_cheatsheets.py --check
```

These checks do not prove Switch callback timing, online compatibility, crash
resolution, or hardware behavior. The portable core example can be run with
`cargo run --manifest-path crates/kirby_assist_core/Cargo.toml --example walkthrough`.

## Build and troubleshoot

Build/release commands and variant packaging belong in
[build variants](BUILD_VARIANTS.md); workbench transfers are described in
[the workbench guide](WORKBENCH.md). For unexpected gameplay behavior, first
confirm the active configuration and restart requirements in
[installation and compatibility](INTEGRATION.md), then collect the diagnostic
data described there. Host tests and logs are evidence about code paths, not a
substitute for a measured device result.

Do not build the Switch plugin or upload to a console unless that work is
explicitly requested.
