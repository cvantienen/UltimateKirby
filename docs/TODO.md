# Maintainer backlog

This is a non-user-facing working list, not a feature promise or authoritative
release plan. Confirm each item against current issues, source, and device
results before scheduling work; the list can lag the implementation.

## Status conventions

**Backlog/unverified** is the default below. A source feature or host regression
may exist without closing a reported gameplay issue. Close an item only with
its reproducible scenario and relevant acceptance evidence; retain the original
question and historical notes when updating status. This documentation review
performed no hardware checks and did not resolve the listed gameplay reports.

For a future item record: trigger/setup, fighter/controller and mode, configuration,
expected/actual result, relevant source owner, focused host evidence, separately
measured device evidence, and remaining uncertainty. Do not attach an invented
issue number, release promise, or passing result.

## Investigation and validation candidates (backlog/unverified)

- Review Kirby Bair routing and fighter-specific roll/grab behavior.
- Calibrate Yoshi and other fighter-specific responses on hardware.
- Investigate reported doubles behavior and verify Training Modpack integration.
- Measure callback/frame-time behavior on the target device.
- Compare offline and Arena behavior where those environments are in scope.
- Record Switch acceptance and compatibility separately from host tests or
  compilation.

## Source-status notes, not closed issues

- Private test bodies have moved into package `tests/unit/` scopes; this is code
  organization, not proof that a device report is resolved.
- Recovery-punish implementation is `policies/option_read.rs`; compatible public
  aliases do not constitute a new universal combo backend.
- Diagnostic health counters are in `src/diagnostics/performance/health.rs`; they
  measure selected callback/report work, not GPU/system temperature.
- `tools/check_action_ownership.py` still expects moved mapper symbols in
  `src/live.rs`. Updating that legacy harness remains a separate executable-code
  task; this docs-only review records the limitation, not a completed repair.

Removed combo playback is not an active feature or a committed future item.
See [runtime safety and feature limits](RUNTIME_SAFETY.md) for the distinction
between current input recipes and reserved playback abstractions.
