# Samus and Dark Samus: author tether and projectile inputs

Samus (`samus`, kind 3) and Dark Samus (`dark_samus`, kind 4) are separate
profiles, settings and catalogs. They may contain similar recipes, but there is
no runtime alias borrowing the other form's choices. Inspect each current
snapshot before copying a script.

## Tether safety

A ground R requests an ordinary grab; a tether recipe must actually hop and
verify airborne state before R if failed takeoff must not become a ground grab.
The supplied action table shows whether that check exists. Native zair motion,
active spans and root geometry remain read-only references; the file does not
inherit old AIR_LASSO acceptance loops or twelve-frame ownership schedules.
Author explicit holds/releases and calibrate takeoff/input acceptance.

## Missiles and Bomb

Homing/Super missile selection depends on actual directional B input and native
state, not the saved name. Side-B strength, neutral primers and facing waits
must be written explicitly. No hidden travel simulation predicts an arrival
frame. Ground missiles can miss jumps/drops or be shielded/reflected; projectile
limits, travel and stage geometry remain native.

Bomb input requests Down+B, not an immediate explosion. Generation/fuse/collision
are native properties and are not shortened by start_delay. A Drop or Roll row
alone adds neither automatic chase nor recovery. Read the action column rather
than historical compiled startup/release descriptions.

## Configure, save and expand

Read the current generated reference for [Samus](fighters/samus.md) and [Dark Samus](fighters/dark_samus.md). Its tables are repository
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
