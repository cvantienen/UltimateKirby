# Parked whiff-punish prototype

Automatic whiff punishing is **removed from the live build and menu for now**.
There is no Whiff punish row, opponent-observation pass, queued whiff starter or
runtime whiff state in live.rs. It cannot issue live inputs.

The portable prototype and its host fixtures remain in whiff_punish.rs, with
its native adapter source retained but not registered as a live module. It can
read fighter-specific facts from the canonical
[Fighter Data dataset](FIGHTER_DATA.md); their presence does not enable automation.

The manually triggered Y [finisher binding](FINISHERS.md) is also retired;
current button recipes and the separate Kirby automatic fair follow-up remain.
Future recovery automation must use the common [snapshot/progress APIs](OBSERVATIONS_AND_TIMING.md)
and [action ownership](ACTION_OWNERSHIP.md), with native acceptance and measured
spacing/time checks before reconnecting it to gameplay.
