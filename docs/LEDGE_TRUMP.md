# Ledge trump: one-shot run-off and inward catch attempt

Assign **Ledge trump** to a configurable button via [Controls](CONTROLS.md).
A fresh press starts it; release does not stop an in-flight one-shot. Holding
cannot restart it after catch, failure or timeout. Release and press again to
retry. It is not a hard-coded D-pad Down macro, persistent toggle or proof the
opponent was actually displaced from the ledge.

## Edge selection and exact input phases

The selected human fighter must be grounded and native-ready at activation.
Read-only support probes search both directions up to100 world units along the
current floor. Nearest detected boundary wins; equal distances prefer native
facing. Failed search means no movement and no repeated scan until a fresh
press. This detects collision support, **not native cliff grabbability**.

1. Request outward stick toward the chosen boundary: full strength while far,
   then0.55 within8 units to reduce outward carry.
2. On first native fall, request **inward+Down**, not the old Down-only phase.
3. Keep that input until native status/motion frame advances, then release Down
   and retain inward input.
4. Native ledge catch/catch-move/wait ends ownership immediately; choose follow-up
   yourself. No attack, ledge invulnerability or occupancy write is performed.

The attempt deliberately permits running offstage. It cannot make a platform
edge grabbable or erase ledge lockouts. Slopes, moving floors and stage shapes can
invalidate the selected boundary; no moving target chase or new edge search is
performed mid-attempt.

## Clock and cancellation

Repeated mapper polls are not Down-phase frames. Native status/motion/frame
changes advance a bounded120-observation attempt budget; stopped play pauses and
emits no new trump stick. This count is not a certified simulation-frame timer.
Unexpected attack/damage, invalid context/readiness, more than3 units past the
selected edge while grounded, more than16 vertical units from its anchor or more
than12 horizontal units after falling cancel. Approach reach is initial distance
plus4. Failed catch returns physical controls rather than teleporting to the edge.

Safety and protected committed inputs can outrank trump; explicit Held Defense/
Counter preempt nonsafety owners. Trump is not automatically above every ledge
response. Mode/menu/session cancellation and shared request freshness apply.
Only the configured role binding is reserved; no universal Down reservation is
implied. See [Sequence files](SEQUENCE_FILES.md) for assignments and
[Fighter settings](FIGHTER_SETTINGS.md) for separate movement estimates.

## Player practice versus code extension

Start near each edge of a simple flat stage, compare both facings and confirm
native catch plus immediate manual follow-up. Then test an empty ledge and an
occupied ledge; catch alone is not confirmed trump success. Change only one
binding/starting position per trial. No fastest or guaranteed online timing is
claimed.

For Rust readers, `policies/ledge_trump.rs::nearest_floor_edge` returns
`Option<[f32;2]>`: a bounded candidate or none. `LedgeTrump` retains press history
and an optional `Attempt`; its phases are Approach, Down and Return. Adding a
stage-aware technique requires verified read-only geometry, native readiness,
phase/cancellation and duplicate/stop tests—not position/velocity/status edits.
A saved input tape is a different tool and cannot register a new native catch
capability. No Switch build or certification accompanies this guide.
