# DI and SDI input assistance

Both features are **experimental and independently OFF by default**. They request
controller inputs only during observed victim damage hitstop; they do not change
knockback, damage, physics, hitboxes, status or move definitions. Their predictions
are bounded heuristics, not guarantees of survival or globally optimal choices.

Enable `Auto DI (experimental)` and/or `Auto SDI (experimental)` in the assist
menu. Each toggle updates that fighter's settings immediately. To persist, open
**Fighter settings** in Training and press **Y: Save**. The existing
`[fighter.settings]` section in `sequences.txt` stores `auto_di_default` and
`auto_sdi_default` as booleans. No new file format or automatic disk writes are
introduced. Unset preferences inherit startup `auto_di`/`auto_sdi` (both OFF by
default); explicit saved false overrides startup true. Resetting the tuning field
restores its startup value. Unsaved changes do not survive a restart.

- `auto_di_mode = survival` is the default. Existing `adaptive` configs now use
  the same survival selection; launch speed alone no longer enables outward
  combo-escape DI. `escape` is still an explicit separation-focused option and
  can make survival worse. The legacy `auto_di_survival_speed` field still parses
  for file compatibility but no longer switches Adaptive into Escape.
- With valid blast-zone bounds and fighter data, Survival evaluates a bounded,
  collision-free trajectory forecast for neutral and multiple stick directions,
  favoring predicted clearance. The live adapter abstains when that data is
  missing or invalid, including after an earlier valid decision. The standalone
  launch-only scoring helper remains available for model tests, but is not the
  live missing-data fallback.
- The live damage response uses exclusive ownership: once selected, its inputs
  and neutral gaps do not yield to physical stick movement. Disable the relevant
  menu toggles to retain manual control over those responses.
- SDI requests flicks away from a known attacker, separated by neutral and at
  least four native hitstop frames. Valid observed blast-zone geometry may bend
  the flick inward or upward. Missing/invalid environment data or unknown attacker
  direction makes SDI abstain. The eight-unit candidate step is only a scoring
  approximation, not a measured displacement or a collision sweep. Candidates
  outside the observed bounds are rejected. SDI currently abstains above 60%
  damage and stops
  for the final three hitstop frames, leaving the launch-input phase to DI or
  manual input when no other owner is active.
- Capture mash takes priority; only an urgent actual tech press can interrupt
  final DI. Non-press tech preparation/release yields to final DI; other active
  tech input takes priority over early SDI. An already held
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
DI/SDI handoff, independent menu choices, and persistence. No terrain model is
invented: floor/wall/ceiling rays elsewhere do not prove these trajectories
collision-free. Native DI sampling order, delivery lead time, zero-count release,
per-hit SDI eligibility/multipliers and displacement still need on-device
validation. Host tests establish input policy and copied-data behavior only.
No hardware build or deployment is implied; source changes do not update an
installed build.

## Host verification

Focused host checks passed **291 tests** across damage policies, native damage
adapter stubs, menu navigation/rendering, independent preferences, tuning saves,
and diagnostics. The final combined integration check passed **96 tests**
(43 collision checks plus 53 damage/menu checks). These are host-model results,
not on-device certification.

Broader menu suites retain baseline failures reproduced from an isolated,
unchanged HEAD: `per_fighter_auto_cache` had 33 passing / 5 failing tests;
diagnostic `live_menu_diagnostics` had 35 passing / 3 failing tests. The primary
failure is the existing Training-editor fixture's invalid short reaction syntax
(`short reaction syntax requires an auto-react event`); subsequent failures are
poisoned shared test locks. These unrelated fixtures were not changed.
