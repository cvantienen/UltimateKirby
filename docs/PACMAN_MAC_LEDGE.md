# Pac-Man and Little Mac: resources and grounded choices

Pac-Man uses `pacman` (kind 56); Little Mac uses `littlemac` (52). Both have
independent profile facts and file choices. Old long-grab/ghost planning windows,
adaptive aerial rules and release-on-startup handlers are no longer compiled
recipe behavior.

## Pac-Man resources

A Fruit label does not create stored fruit, choose its identity, install a hydrant
or configure trampoline color. Inspect the neutral-B duration: without stored
fruit it can start Bonus Fruit charging; a longer hold can cycle rather than
throw. Projectile emission is not contact, and no unverified fruit/trampoline
interaction becomes an automatic KO. Author only supported resource checks and
prepare unsupported setup manually. Capture returns manual pummel/throw control.

## Little Mac grounded versus aerial

A Jump-response label is an opponent-event row, not a command to jump. The actual
actions may remain grounded. Little Mac's qualitative aerial hints are deliberately
short coverage, not measured detection limits. Selecting an aerial in Auto Space
requires already-airborne state; it does not spend a jump. KO meter, charge and
counter behavior need verified native adapters rather than guessed recipe names.
An offstage script does not recover automatically.

## Configure, save and expand

Read the current generated reference for [Pacman](fighters/pacman.md) and [Littlemac](fighters/littlemac.md). Its tables are repository
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
