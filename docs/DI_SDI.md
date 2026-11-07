# DI and SDI input assistance

Both features simulate left-stick inputs only during observed victim damage
hitstop. They do not change knockback, damage, physics, hitboxes, status or move
definitions. They are heuristics, not a blast-zone/trajectory survival solver.

- `auto_di_mode = survival` is the default. Existing `adaptive` configs now use
  the same survival selection; launch speed alone no longer enables outward
  combo-escape DI. `escape` is still an explicit separation-focused option and
  can make survival worse. The legacy `auto_di_survival_speed` field still parses
  for file compatibility but no longer switches Adaptive into Escape.
- Survival evaluates the existing nine stick candidates and reduces the
  dominant launch component. It rejects candidates whose modeled LSI would
  increase total launch speed. Nearly vertical launches with unknown side
  information, invalid launches and unavailable decisions abstain.
- A physical stick at least 20% from neutral immediately takes priority, even
  when held before the hit. Automation stays out for that hit after you release
  the stick too; a separately observed new hit can start a new decision.
- SDI flicks away from a known attacker earlier in hitstop only with a neutral
  player stick. It never guesses stage center when the attacker is unknown.
  It stops for the final three hitstop frames, or earlier for a known input-delay
  cutoff, allowing stable launch DI/manual input. There is no final-frame outward
  SDI fallback when DI has no usable launch data. Invalid stick data abstains.

The previous Adaptive behavior could choose greater separation on weak launches
without knowing whether a blast zone was close, while a held pre-hit stick could
be overridden. Those are plausible contributors to early KOs, not proof of the
cause of any particular death. Automatic outward SDI remains position-unaware;
manual input takes priority when you need a particular survival direction.

The local prediction uses the existing
[DI/LSI calculator formulas](https://github.com/rubendal/SSBU-Calculator/blob/master/js/formulas.js)
as a small approximation. It is not a game-physics modification. No extra game
queries, stage rays, worker or trajectory simulation is added. Existing per-hit
DI caching and mapper ownership remain. Host regressions check weak/strong
launch directions, modeled speed increases, held/manual input, duplicate samples,
new-hit reset and final DI/SDI handoff. On-device results still need comparison;
source changes alone do not update an installed build.
