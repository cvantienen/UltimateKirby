# Parked whiff-punish prototype

Automatic whiff punishing is **removed from the live build and menu for now**.
There is no Whiff punish row, opponent-observation pass, queued whiff starter or
runtime whiff state in live.rs. It cannot issue live inputs.

The portable prototype and its host fixtures remain in whiff_punish.rs, with
its native adapter source retained but not registered as a live module. The
[compiled reference facts](OPPONENT_DATA.md) and explicit developer data importer
are preserved for future work. Their presence does not enable automation.

Kirby/Yoshi's manually triggered Y [finishers](FINISHERS.md) remain available.
Future recovery automation must use the common [snapshot/progress APIs](OBSERVATIONS_AND_TIMING.md)
and [action ownership](ACTION_OWNERSHIP.md), with native acceptance and measured
spacing/time checks before reconnecting it to gameplay.
