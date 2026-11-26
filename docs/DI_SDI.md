# DI and SDI input assistance

**DI** changes the launch direction through ordinary stick sampling; **SDI** uses
stick flicks during victim damage hitstop. They are independent experimental
features. The mod requests input only: it never writes knockback, damage, native
physics, hitboxes, status or animation rate. Bounded estimates are not guaranteed
survival or globally optimal decisions.

## Enable and persist deliberately

Enable the separate Auto DI / Auto SDI rows for your selected fighter, then save
fighter settings through the Training workflow in
[Fighter settings](FIGHTER_SETTINGS.md). `auto_di_default` and `auto_sdi_default`
are optional fighter startup overrides; an explicit false overrides universal
true, while omission inherits. Current selection and saved startup are different.

The **live plugin's** Rust fallback `auto_di` and `auto_sdi` are **false**.
The standalone DI policy's `Settings::default()` has `enabled = true`, survival
mode and speed3; `LiveConfig` explicitly overrides its enabled flag to false.
That internal model default is not live startup admission. The supplied universal
section sets both live features **true**, but a fighter file may explicitly
disable either. For example:

```ini
[lucina.settings]
auto_di_default = false
auto_sdi_default = false
```

That is a valid per-fighter opt-out, not a universal command. Check the actual
saved section rather than assuming all fighters use the universal value.
[Controls](CONTROLS.md) owns mode admission; [Sequence files](SEQUENCE_FILES.md)
owns startup/file precedence. Global OFF, invalid controller scope, menu and
Training passthrough still suspend automation.

## DI decision and its limits

`auto_di_mode = survival` is the Rust fallback and supplied universal mode.
Adaptive uses the same survival selection; legacy `auto_di_survival_speed` does
not switch it into escape at high launch speed. Explicit Escape is separation-
focused, requires known attacker direction and may make survival worse.

With valid copied blast bounds and native fighter movement data, survival scores
neutral and bounded stick candidates using a local trajectory approximation.
The live adapter abstains on invalid/missing environment data, even after an
earlier valid decision. A launch-only helper used by model tests is not a live
missing-data fallback. The forecast omits stage collisions, recovery choices and
the game's exact KO decision; do not call it a collision-safe path.

`src/live/damage.rs` caches copied bounds for a hit and refreshes them at the final
launch-input handoff. It retains no game pointer. Upward launch sectors select
native damage-fly-top gravity/fall parameters before candidate scoring. Invalid
or unbounded damage-stop observations return no geometry/trajectory.

## SDI flicks and handoff

Only the **victim's damage hitstop** qualifies—not attacker hitlag, ordinary
hitstun, shield freeze or global stop. Remaining native hitstop distinguishes
updates while animation is frozen; repeated polls reuse the same decision.
A game-visible held stick gets neutral release before the first flick. Flicks
have neutral rearm gaps and at least four native hitstop frames between them.
SDI abstains above 60% damage and stops for the final three hitstop frames,
leaving launch input to DI or manual input when no other owner is active.

Known attacker direction and valid environment evidence are required. Geometry
may bend a flick inward/upward; out-of-bounds candidates are rejected. The
candidate eight-unit step is scoring only, not measured displacement or a terrain
sweep. When selected, the response owns its stick including neutral gaps;
physical movement does not silently cancel it. Disable the row to keep that
response manual.

Capture mash has first safety priority. The native tech scheduler's actual
Guard-preparation/press ownership can take precedence over DI; otherwise DI
precedes SDI. A contact forecast alone is not permission to override. Losing
hitstop/eligibility, disabled selection, invalid environment/session or safety
cancellation removes the proposal. No wall-time catch-up or learned online-delay
compensation is added.

## Player experiment versus Rust extension

Test one feature at a time in Training with repeatable damage/launch, including
near-blast-zone, unknown attacker and final-hitstop cases. A mapped stick is not
proof Smash sampled DI/SDI at the desired instant. Native SDI multipliers, per-hit
restrictions, sampling order, queues and actual displacement need device checks.
Do not use old host test counts as current certification.

Rust beginners should distinguish `Option<Trajectory>` (valid copied prediction
or absence) from a cached pointer. `src/live/damage.rs::observe` is read-only and
bounded; the shared policies choose input. Extend scoring only with finite-data,
missing-data, duplicate-hitstop and handoff tests, and never change native physics
to make a predicted outcome true. See [Trip follow-up](TRIP_AND_SDI.md) for the
separate Kirby tool; enabling SDI does not enable a trip punish.
