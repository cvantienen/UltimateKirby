# Auto spacing: manual movement caps, retreat and optional attacks

Auto spacing is a current input-only built-in, **not retired**. It modifies
controller movement proposals and can request a configured ground/aerial attack
when copied target motion predicts reach. It never sets velocity, braking,
hitboxes, recovery or animation rate.

## Activate and tune one feature at a time

Assign its hold or toggle in [Controls](CONTROLS.md). Global enabled, selected
fighter/controller, valid native control and competing-owner admission still
apply. Held Defense/Counter displace lower nonsafety ownership; ledge and manual
input conditions can exclude spacing. Releasing a hold or clearing a toggle
removes the spacing request, not a native move already accepted.

Schema fallback minimum distance is15 and engagement range20. Ground/aerial move
selectors default to **0 (disabled)**. Supplied fighters may enable attacks and
change measured reference values. See [Fighter settings](FIGHTER_SETTINGS.md)
for selector values, exact key bounds and save/reset behavior. A minimal experiment:

```ini
[lucina.settings]
auto_spacing_ground_move = 0
auto_spacing_aerial_move = 0
```

This disables spacing attacks for that fighter while letting you test movement
admission. It is not the supplied Lucina configuration (which enables both
selectors). Preserve other settings and use the live menu's current controls.

## What the movement does

Without an active target, manual movement caps can still apply while admitted.
Ground horizontal caps are limited by the configured value/native non-dash stick
threshold. Air manual caps are full strength **1**. Vertical manual input is
preserved by the cap helper. Physical Attack/Special/Grab/Shield or real C-stick
action bypasses movement-only caps so manual action selection does not inherit
an unwanted walk angle.

When the known enemy is too close now or in a five-frame relative-motion
projection, ground retreat requests an outward three-world-unit step, with floor
support required; it does not step off an unsafe floor. Air retreat requests
outward drift with a two-unit hysteresis band before releasing. **Hysteresis**
means the release threshold differs slightly from the start threshold to avoid
rapid on/off jitter. These are estimated root distances, not hitbox boundaries.
No automatic inward approach or guaranteed return-to-stance is implied.

Legacy speed/drift/braking settings remain stored for compatibility; do not
assume their old names all change the live caps. In particular maximum/minimum
air drift and legacy braking controls do not retune the fixed full-strength air
manual cap. Consult the authoritative key-effect table rather than treating a
stored field as proof of active behavior.

## Attack admission and input sequence

A selected nonzero attack needs positive finite startup/activity/reach/height/
commitment estimates, currently vulnerable target and predicted horizontal and
vertical overlap at the same active time. Native air physics can refine vertical
travel; missing physics uses linear travel, not invented gravity. Readiness,
facing, movement/brake status and existing retreat can veto a request.

Ground facing correction uses sub-dash stick and waits for observed native
facing before Attack. Ground tilts request A with a sub-dash main-stick direction,
not full C-stick; air selection uses A plus the intended main-stick direction.
No Jump is generated merely to make an aerial. A winning mapped packet starts
an acceptance wait. Bounded retries alternate release and Attack; duplicate
mapper receipts do not create an independent successful move. Air selection
keeps its directional input until native aerial motion confirms, landing,
reversed tick or its bounded expiry—even if a target forecast briefly vanishes.

## Clock, cancellation and extension

Distinct admitted native-derived ticks drive pending retry accounting; stop
does not spend progression. Target/side/attack configuration changes, missed
forecast, native attack startup, brake/retreat, invalid context, mode/hold loss,
capture/damage and timeouts release pending proposals. Some selection receipts
remain bounded to avoid changing a buffered aerial's direction before acceptance.
The shared mapper arbitrates one winning proposal, restores physical storage
and expires stale requests; spacing is not permanent controller mutation.

Rust beginners can follow `policies/auto_spacing.rs::Spacing::sample`,
`sample_retreat`, `predicts_hit` and `mapped`. `Output` separates controller input
from optional movement caps and attack intent. `AttackConfig` numbers describe
estimates, not native move edits. Players tune/save these estimates; developers
adding movement/attack paths need finite/unknown data, manual-action handback,
floor-support, mapped receipt and interruption tests. Test movement alone first,
then one configured attack on both sides. Root forecasts and online input queues
cannot guarantee safe spacing or a connected hit.
