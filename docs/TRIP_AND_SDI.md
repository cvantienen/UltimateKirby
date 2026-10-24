# Down-tilt trip follow-up and auto SDI

Both tools apply to the selected human player's controller without holding the ledge-assist button.
They respect the global `enabled` setting and existing controller/session checks.
No parry feature or ZL/ZR chord was added.

```text
down_tilt_trip_smash = true
auto_sdi = true
```

## Tilt trip follow-up

This Kirby-only tool is independent of L and Y. **Down tilt → Forward smash**;
forward tilt also uses forward smash. Recognition accepts an
opponent in native trip states `0x86..=0x88`, including an already-tripped target.
It also accepts grounded damage (`0x47`) or downed damage (`0x5a`) immediately
following one of those trip states. Re-hits can replace the trip status before
the next observation, so a new trip transition is not required. Unrelated damage,
air launch, and trip get-up/roll states do not qualify through this fallback.
There is no extra attribution, spacing, age or hit-confirm requirement.

Recognition owns competing controls, releases the old tilt input, then alternates
neutral releases with **facing-direction+A** for either tilt. These are ordinary buffered inputs during native tilt recovery; they cannot
cancel native lag. No additional smash-readiness gate or six-pulse limit is added.
Repeated identical observations reuse output. Native startup of the selected
smash releases A so it does not deliberately charge. Damage/air/unsupported own
states, disabling the feature and route/session changes still cancel.

Each new tilt can rearm the follow-up even when the opponent remains in trip
wait across attempts. The existing `down_tilt_trip_smash` config key is unchanged.

## SDI

Only the damage recipient's hitstop enables this tool. It does not act during
ordinary hitstun, the attacker's hitlag, shield freeze or global stop. The native
remaining-hitstop timer distinguishes advancing samples even when the animation
is frozen. Repeated mapper polls at one timer value reuse one output.

The automatic stick alternates horizontal full deflection away from the attacker
and neutral at each observed timer change. A direction already held when hitstop
starts does not override it. Moving the stick at least 0.4 from its entry position,
with at least 0.4 magnitude, hands full physical control back for the rest of
that hitstop. Releasing to neutral and pressing a direction again also counts.
SDI resumes automatically on a later hitstop session; ordinary DI after hitstop
is always physical.

Direction uses the attributed attacker's current horizontal position. If that
attribution is unavailable and exactly one opponent is known, it uses that
opponent. If direction is unknown or both roots overlap, it leaves the physical
stick alone instead of guessing toward center stage. Buttons remain physical.
Live learned-delay compensation has been removed. This does not change native
SDI multipliers, restrictions or launch physics; actual movement still depends
on the move, native acceptance and observation cadence.

## Validation

Host regressions cover trip-state recognition, full control takeover through
hitstop and tilt lag, continued input beyond six attempts, missing target data,
smash release/acceptance, duplicate polls, SDI direction selection and control
restoration. Native symbols and constants are checked against the pinned
skyline-smash bindings. Switch testing is required for actual hit/SDI outcomes.
