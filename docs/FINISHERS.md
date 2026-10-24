# Kirby and Yoshi finisher attempts

The fighter-specific **Finisher** row replaces the former Y Auto-duck selection. Open the
menu with R3, select Finisher with Up/Down, and cycle with Left/Right:

| Choice | One press of Y |
| --- | --- |
| Fair 1 > Fsmash | Two-button short hop, apex fair, descending fast fall during startup, then upward-angled forward smash after any confirmed fair hit and landing |
| Dair > Dsmash | Two-button short hop, dair, descending fast fall from its first active hit, then down smash only after a confirmed dair/landing hit and landing |
| Yoshi: Bair drag > Fsmash | Confirm facing away, two-button short hop, backward A and descending fast fall; only early-hit confirmation can queue a landing turn and Fsmash |

Kirby has the first two choices; Yoshi has the third. Yoshi's route is also the
parked whiff-punish prototype, which is not active in the live build. Its first two hits
are frames 11 and 14; reaching the final hit before landing cancels the route.
Manual Y assumes you start facing the opponent. A successful landing confirm
turns Yoshi back toward them before buffering forward smash.

**X and ZL must both map to Jump**, A to Attack. Holding Y never loops the
starter; release and press again for another attempt. Start grounded and
actionable, facing the opponent; spacing remains yours. Selection edits,
damage/capture, pause/session changes and invalid/expired native progress
cancel. Releasing Y after the press does not cancel an ongoing attempt.

The Kirby short-hop request pairs X+ZL, then releases both on subsequent native
progress, including while waiting for delayed jump acceptance.
Both buttons must map to Jump and A must map to Attack. Squat releases both Jump
buttons and withholds directional A until actual airborne jump/fall status.
Kirby Fair additionally waits for vertical speed at or below 0.15 units/frame
(near the short-hop apex) before selecting forward+A. Dair and Yoshi Bair select
their directions immediately after takeoff. An unaccepted request holds A for three advancing native frames,
then releases for one before retrying; duplicate polls and hitstop do not spend
this budget. A proposal is never treated as proof of native attack acceptance.
No fast-fall down input is combined with the fair-selection request. Exact
native aerial acceptance releases A. Only descending accepted attacks fast-fall:
fair from animation frame 4, dair from frame 18. Fair descends during startup to
attempt landing after hit 1 and before hit 2; this needs height/spacing calibration.
Neither route adds drift,
walks toward an enemy, changes velocity/status, or reads opponent state.

Yoshi's bair starter uses this same airborne selection and native-progress retry
path after confirming his turnaround. Tests cover both target directions,
including missed A requests, without holding Jump or issuing a grounded tilt.

Host tests cover raw button edges through the shared manual overlay, missed
attack requests, squat/takeoff transitions and exact aerial confirmation. They
cannot prove delivery on a Switch; verify both routes after installation.

Fresh `AttackModule::is_infliction(HIT)` arms the landing follow-up. The Y Fair
route accepts any real hit from its exact fair motion, including a contact first
observed on the fair landing transition. Later fair frames/hits do not discard
confirmation. Landing buffers up-angled Fsmash toward the nearest valid opponent,
without range, percent, protection or readiness waits; native startup still
determines when the input can execute. No second aerial is thrown while waiting.
Dair is unchanged and checks its discrete active intervals and its own
frame-1 landing hit. A shield hit or whiff never arms a finisher. Landing buffers
ordinary smash inputs with release gaps; native startup releases A instead of
charging. The separate automatic fair-first-hit follow-up keeps its existing rules.

These are **attempts, not guaranteed true combos or kills**. The move windows
come from [Kirby's measured frame data](https://ultimateframedata.com/kirby)
and [Yoshi's measured frame data](https://ultimateframedata.com/yoshi).
Dair has landing lag, and smash startup still applies. Opponent percent, DI,
hurtboxes, spacing, landing height, hitlag and input delivery can allow escape.
The input-only sequences cannot extend airtime or bypass animation recovery.
Test both routes offline on Switch before judging coverage.

Compiled choices live in `button_modes.rs`; the bounded state machine is
`finisher.rs`; the live adapter routes its output through the existing fair
takeover and shared manual overlay. There are no new hooks, scans, allocations
in the sampling state machine, configuration-file reads or learned delay.
Native jump readiness, vertical speed and fresh hit observations are read only
when needed. Host regressions are not Switch performance measurements.

Auto-duck is disabled, its Y binding removed, its source tracking off and its
ATTACK-command observer hook no longer installed. `kirby_auto_duck` is retained
for config compatibility but cannot re-enable it in this version. Existing L
defensive shielding is independent and remains unchanged.

Queued jump acceptance now has a bounded **24 advancing-frame budget**, replacing
the old three-frame ground timeout. This adds no scheduling delay: the sequence
continues immediately when native takeoff is observed, with Jump released rather
than repeated while waiting. Native jump/fall/air-attack status takes precedence
over a temporarily grounded situation flag. Shared ledge aerial variants use the
same interpretation. Fair-apex selection, configured move timings, delay learning
(disabled), and route/session safety fences are unchanged. Arena testing remains
necessary; a longer acceptance budget cannot eliminate network delay.
