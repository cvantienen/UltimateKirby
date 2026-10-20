# Down-tilt trip follow-up and auto SDI

Both tools apply to the selected human player's controller without holding Y.
They respect the global `enabled` setting and existing controller/session checks.
No parry feature or ZL/ZR chord was added.

```text
down_tilt_trip_smash = true
auto_sdi = true
```

## Tilt trip to forward smash

This tool is Kirby-specific and independent of held Y. When an opponent is
observed in a native trip state during Kirby's down tilt, it queues forward
smash. The existing forward-tilt starter remains supported too. Recognition
uses only the tilt and trip states; it does not require a separate hit flag,
last-attacker attribution, trip-age limit, distance, spacing, or facing check.

From recognition onward, the macro overrides physical attack, jump, shield,
grab, taunt, and stick inputs. It first releases the tilt input to create a fresh
stick flick, then sends full horizontal stick in Kirby's existing facing
direction + A, alternating with neutral release samples. These inputs begin
while tilt lag remains, using the game's normal buffer; no separate smash
readiness check delays them and no six-attempt limit exhausts them. Repeated
identical native observations reuse the current output.

Once queued, the follow-up does not recheck the opponent's state or require the
roster to remain available. Smash startup releases the macro's A and stick so it
does not deliberately charge. Damage, leaving the ground, another unsupported
player action, disabling the feature, or a controller/session change cancels it.
Ordinary controls return afterward. The macro does not cancel native attack lag;
the game decides the first legal frame on which buffered forward smash starts.
The existing `down_tilt_trip_smash` config key is unchanged.

## SDI

Only the damage recipient's hitstop enables this tool. It does not act during
ordinary hitstun, the attacker's hitlag, shield freeze or global stop. The native
remaining-hitstop timer distinguishes advancing samples even when the animation
is frozen. Repeated mapper polls at one timer value reuse one output.

The stick alternates full deflection and neutral at each observed timer change,
creating repeated fresh flicks. Hold the physical stick at least 0.4 magnitude
to choose direction. With a neutral physical stick it uses horizontal movement
away from the last attributed opposing fighter. Missing attacker/roster data
falls back toward world X=0; this is a stage-center heuristic.

Buttons remain physical, and fresh physical stick input returns immediately
when damage hitstop ends. With a learned input delay, flicks stop that many frames
earlier so queued SDI inputs have time to clear before normal control resumes.
This compensation is an estimate, not access to the online input queue. The macro does not force SDI displacement, modify move
SDI multipliers, bypass native SDI restrictions, or change launch physics. Actual
accepted SDI rate depends on the game, move, observation cadence and online input
delay. An outward default cannot escape every combo and may be wrong for some
situations; use the stick to select another direction or disable the tool.

## Validation

Host regressions cover trip-state recognition, full control takeover through
hitstop and tilt lag, continued input beyond six attempts, missing target data,
smash release/acceptance, duplicate polls, SDI direction selection and control
restoration. Native symbols and constants are checked against the pinned
skyline-smash bindings. Switch testing is required for actual hit/SDI outcomes.
