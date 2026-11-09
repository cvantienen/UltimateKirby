# Kirby Assist documentation

Kirby Assist is an input-only Rust Skyline plugin for Super Smash Bros.
Ultimate. It observes gameplay, evaluates configured/controller-only assists,
and requests simulated inputs for the selected local human fighter. It does not
change native moves, hitboxes, physics, or fighter status.

## Find your path

### Set up and use the mod

1. Read [installation and compatibility](INTEGRATION.md).
2. Learn the [controls](CONTROLS.md) and [ledge menu](LEDGE_MENU.md).
   The [shortcuts and turbo controls](TURBO_CONTROLS.md) guide covers the
   additional controller bindings.
3. Configure saved responses using [sequence files](SEQUENCE_FILES.md) and
   [fighter settings](FIGHTER_SETTINGS.md).
4. Browse the current [fighter roster](FIGHTERS.md) and its generated guides.

### Understand a feature

Start with [assist rules](REACTION_RULES.md) for automatic reactions, or use the
specific guides for [ledge autopilot](LEDGE_TRAP.md),
[shield pressure](SHIELD_PRESSURE.md), [out of shield](OUT_OF_SHIELD.md),
[grab dodge](GRAB_DODGE.md), [automatic tech](AUTO_TECH.md),
[DI/SDI](DI_SDI.md), and
[training/monitoring](TRAINING_MONITOR.md). The assist guides distinguish
automatic rules from manually selected recipes and explain their own settings
and timing.

### Develop or extend the project

Read the [runtime architecture](ARCHITECTURE.md) for the observation-to-input
flow and shared-state boundaries, and [runtime safety](RUNTIME_SAFETY.md) for
freshness, cancellation, and live feature limits. Then use the
[beginner and extending guide](DEVELOPMENT.md) to trace a feature, reproduce and
correct issues, implement changes, run tests, and maintain configuration,
fighter references, and documentation.

## Project map

```text
src/                         Switch plugin and native integration
  live.rs                    Startup and top-level hook/mapper callbacks
  live/                      Observation, mapper, state, and native adapters
crates/kirby_assist_core/    Portable policy, configuration, and tests
deployment/ultimate/...      Canonical startup sequence configuration
data/                        Canonical roster/fighter facts
docs/                        User guides and developer references
tools/                       Validation, documentation, build, and workbench tools
```

The [source map](CODE_MAP.md) lists the main modules. The
[build and release guide](BUILD_VARIANTS.md) covers packaging; the
[interactive workbench](WORKBENCH.md) covers its local tooling.

## Validation and support caveats

Portable tests and strict documentation builds can validate host-side logic,
configuration, and links. They do not prove Switch timing, online compatibility,
crash resolution, or temperature behavior. See
[extending the project](DEVELOPMENT.md#test-and-validate) for test commands and
[installation and compatibility](INTEGRATION.md) for runtime troubleshooting.
