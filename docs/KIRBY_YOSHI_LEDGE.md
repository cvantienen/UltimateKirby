# Kirby and Yoshi: recipes versus separate helpers

Kirby (`kirby`, kind 6) and Yoshi (`yoshi`, kind 5) use distinct profile data and
files. Current Nair/Bair/roll-grab labels do **not** invoke the removed conditional
walking, adaptive-hop, native-acceptance nudge or fixed-20-unit endpoint engine.
Every movement, jump, wait, direction and attack belongs to the actual tape.
Do not copy Yoshi's tongue-grab duration from Kirby's standing grab table.

Both expose down/forward/up tilt-facing tuning rows; down's schema fallback is
ON, forward/up OFF. The current Tilt face switch and native mapping still gate
them. Kirby additionally has independent optional idle crouch, trip confirmation
and first-hit-fair landing follow-ups. Yoshi does not inherit those. See
[trip](TRIP_AND_SDI.md), [fair confirmation](KIRBY_FAIR_CONFIRM.md) and
[posture/input rules](INPUT_RECIPES.md). These helpers are not Nair recipe steps.

To add Nair drift, author hop, neutral attack and explicit drift in that order.
A forward stick during attack selection can select fair, so do not infer neutral
attack from the label. A directional Bair needs explicit facing/input axes;
turnaround may require observed confirmation. A roll grab needs explicit movement
and an instant range/facing check when a delayed catch would be unsafe.
Offstage Dair/Fair or Stone input never implies automatic recovery.

## Configure, save and expand

Read the current generated reference for [Kirby](fighters/kirby.md) and [Yoshi](fighters/yoshi.md). Its tables are repository
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
