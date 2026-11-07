# Maintainer backlog

This is a non-user-facing working list, not a feature promise or authoritative
release plan. Confirm each item against current issues, source, and device
results before scheduling work; the list can lag the implementation.

## Investigation and validation candidates

- Review Kirby Bair routing and fighter-specific roll/grab behavior.
- Calibrate Yoshi and other fighter-specific responses on hardware.
- Investigate reported doubles behavior and verify Training Modpack integration.
- Measure callback/frame-time behavior on the target device.
- Compare offline and Arena behavior where those environments are in scope.
- Record Switch acceptance and compatibility separately from host tests or
  compilation.

Removed combo playback is not an active feature or a committed future item.
See [runtime safety and feature limits](RUNTIME_SAFETY.md) for the distinction
between current input recipes and reserved playback abstractions.
