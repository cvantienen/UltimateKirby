# Luigi and Captain Falcon: independent profiles

Luigi uses `luigi`; Captain Falcon uses `captain_falcon` (the documentation filename
is captain.md). Their exact native kinds are 9 and 11. Neither borrows Kirby
reference data, option defaults or a compiled combo builder. The supplied pages
show each current file's rows and actual delays; older lead/clamp/shield-to-Nair
claims were retired with the compiled ledge engine.

A shorter native startup is a useful calibration fact, not an automatic input
lead. In a multihit aerial, inspect separate active spans rather than joining the
gap. A grab recipe needs each fighter's own reach/facing/state; a successful
capture does not automatically pummel or throw.

## Configure, save and expand

Read the current generated reference for [Luigi](fighters/luigi.md) and [Captain](fighters/captain.md). Its tables are repository
snapshots, not your installed file or unsaved Training draft. Follow
[the full configuration walkthrough](FIGHTER_CONFIGS.md) to choose/edit the exact
fighter row, set explicit start delay/actions, save with Y alone and wait for
success. External file edits require restart; current mode activation is separate
from saved startup preference. Use [all 83 tuning keys](FIGHTER_SETTINGS.md) for
valid ranges, units, menu visibility and active/legacy effects.

New choices should reuse existing input primitives under the exact profile prefix.
New native behavior requires an explicitly reviewed shared adapter; profile
registration or extra sliders alone cannot implement it. Keep all assistance
input-only. [Rust extension contracts](FIGHTER_CONFIGS.md#extend-the-rust-code)
explain the stable schema, exact-kind validation and focused tests.

## Verify the intended outcome

Check label versus actual action, facing on both sides, grounded/airborne state,
resource setup, emitted press/release and native acceptance separately. Move-frame
facts are [canonical reference data](FIGHTER_DATA.md), not timing commands.
Then [calibrate](FIGHTER_EXECUTION.md) from repeatable positions, including
hitstop, target loss, interruption and online play. No asset table or host check
proves contact, safety or optimal timing. This documentation update does not
build/upload to Switch, alter canonical assets or change executable Rust.
