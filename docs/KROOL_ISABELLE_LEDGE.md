# King K. Rool and Isabelle: melee and projectile configuration

K. Rool's profile prefix is `krool` (kind 69); Isabelle uses `isabelle` (70).
Their supplied lists/defaults are independent. Neither inherits compiled
belly-flop/slingshot scheduling, automatic fair-versus-bair selection or adaptive
hop behavior. Actions, not move names, determine delivery.

For K. Rool body/shockwave or front/rear smash coverage, keep separate data phases
separate. For Isabelle slingshot, write the hop, directional attack, wait and
drift explicitly; projectile emission is not projectile arrival. A Roll row
alone does not cause a chase to an endpoint. A shield-shot sequence should wait
for a confirmed block only if `shield_until_block` is actually authored.
Net grab uses independent reference facts and capture hands back control.
Drop spikes can take either fighter offstage; no label adds recovery.

## Configure, save and expand

Read the current generated reference for [Krool](fighters/krool.md) and [Isabelle](fighters/isabelle.md). Its tables are repository
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
