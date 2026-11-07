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

## Add a gameplay capability

Use this sequence for a new assist or a substantial extension:

1. **Define the observable trigger.** Identify the native facts required and
   where they are read in `src/live/frame.rs` or
   `src/live/observations.rs`. Read them once through the callback-scoped
   observation layer; do not retain native pointers or add repeated scans
   without a measured need.
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
`src/live/state.rs`, reset semantics in `src/live/runtime.rs`, and portable
evaluation in `crates/kirby_assist_core/src/assist.rs` and `runtime.rs`.
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
