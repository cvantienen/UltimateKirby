# Kirby forward-air first-hit landing follow-up

A fresh hurtbox hit during **fair frames 10–11** arms the follow-up. This uses
`AttackModule::is_infliction(HIT)`, not the persistent infliction-status flag.
Shield-only contact, nair and fair's later hits cannot arm it. The route is the
fast-fall first-hit landing confirm: land before fair reaches its second hit
(frame 17). There is no additional fast-fall flag, percent, spacing, DI, opponent
status or target scan required after first-hit confirmation.

After recognition, the tool owns competing gameplay inputs. It waits for landing
before sending forward+A at `[facing, 0.5]`, giving an upward angle. Pulses enter
the game's ordinary landing-lag buffer; this cannot bypass airborne restrictions
or landing lag. A is released when native forward-smash startup is observed,
while the angle is kept through startup until attack selection. It then hands
back control without charging the smash.

Reaching fair's second hit before landing, damage, unrelated actions, a malformed
clock, excessive delay or a higher-priority tech/mash interruption cancels the
follow-up. A confirmed hit is not a guarantee that forward smash will connect;
spacing, DI, percentage and input delivery still matter.

`kirby_fair1_smash = true` enables it at startup (default and supplied config).
Set it to false and reload the plugin/game to disable it independently of the
existing down-tilt trip -> forward-smash tool. The reader adds one fresh collision
query only during Kirby's first fair active interval, with no additional roster
scan, game-state write or live GUI activity.

Regression fixtures cover first-hit confirmation, shield/late-hit rejection,
landing buffering, both facing directions, angle retention, release on acceptance,
and interruption. They have not been compiled or executed in this source-only
revision. No build or upload was performed.
