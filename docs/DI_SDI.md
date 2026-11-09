# DI and SDI input assistance

Both features request controller inputs only during observed victim damage
hitstop. They do not change knockback, damage, physics, hitboxes, status or move
definitions. Their predictions are heuristics, not guarantees of survival.

- `auto_di_mode = survival` is the default. Existing `adaptive` configs now use
  the same survival selection; launch speed alone no longer enables outward
  combo-escape DI. `escape` is still an explicit separation-focused option and
  can make survival worse. The legacy `auto_di_survival_speed` field still parses
  for file compatibility but no longer switches Adaptive into Escape.
- With valid blast-zone bounds and fighter data, Survival evaluates a bounded,
  collision-free trajectory forecast for neutral and multiple stick directions,
  favoring predicted clearance. Without that data it falls back to nine stick
  candidates that reduce the dominant launch component without increasing
  modeled speed. Invalid launches and unavailable decisions abstain; the
  fallback also abstains for nearly vertical launches with unknown side.
- The live damage response uses exclusive ownership: once selected, its inputs
  and neutral gaps do not yield to physical stick movement. Disable `auto_di`
  and `auto_sdi` in the startup configuration to retain manual control over
  those responses.
- SDI requests flicks away from a known attacker, separated by neutral and at
  least four native hitstop frames. Available blast-zone geometry may bend the
  flick inward or upward. Unknown attacker direction makes SDI abstain. It stops
  for the final three hitstop frames, leaving the launch-input phase to DI or
  manual input when no other owner is active.
- Capture mash takes priority; an urgent tech press can interrupt final DI.
  Other active tech input takes priority over early SDI. An already held
  game-visible stick is released to neutral before SDI's first flick.

Escape is explicitly separation-focused and requires a known attacker direction;
it can make survival worse. The forecast does not model stage collisions,
recovery options or the game's KO decision.

The local prediction uses the existing
[DI/LSI calculator formulas](https://github.com/rubendal/SSBU-Calculator/blob/master/js/formulas.js)
as a small approximation. It is not a game-physics modification. The live
adapter reads the native death area and fighter movement parameters, caching
the bounds per hit and refreshing them at the final DI handoff. Host regressions
cover the forecast, SDI flick timing, duplicate samples, new-hit reset and
DI/SDI handoff. On-device results still need comparison; source changes alone
do not update an installed build.
