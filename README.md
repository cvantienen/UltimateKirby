# Kirby Assist

Kirby Assist is a Rust plugin for Super Smash Bros. Ultimate. It adds configurable
controller-input assists for a human-controlled Kirby. The plugin targets a
Nintendo Switch running Skyline and uses a Pro Controller.

> **Testing status:** Host tests check the shared logic, but they do not verify
> gameplay or compatibility on a Switch. Test device changes carefully.

## Features

- Auto mash and auto tech
- Directional turbo crouch
- Configurable ledge assist and ledge trump
- Silent and diagnostic plugin builds

Settings are read from `SD:/ultimate/kirby_assist/config.txt` when Smash starts.
Restart the game after changing the file. Install only one build variant.

## Quick start

Run the host-side tests from the project root:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
```

Build the Switch plugin with a configured Skyline toolchain:

```sh
cargo skyline build --release
```

See [installation and configuration](docs/INTEGRATION.md) for setup details.

## TODO

- [ ] Fix ledge-trap homing issues.
- [ ] Fix predicted grab-startup issues.
- [ ] Investigate the 14 known host-test failures.
- [ ] Validate the ledge-drop shield-defense option.
- [ ] Add dynamic ledge-trap options loaded from config for each character.

## Documentation

- [Controls and ledge assist](docs/LEDGE_TRAP.md)
- [Directional turbo](docs/TURBO_CONTROLS.md)
- [Analog mash](docs/ANALOG_MASH.md)
- [Build variants](docs/BUILD_VARIANTS.md)
- [Rust beginner guide](docs/README_BEGINNER.md)
- [All project documentation](docs/index.md)



The main function is [`watch_ledge()` in ledge_trap.rs](/Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2/crates/kirby_assist_core/src/ledge_trap.rs:1045). It handles **waiting at the ledge and punishing normal get-up**.

The important distinction is that this code **doesn’t wait a fixed number of frames after recognizing get-up**. Every eligible observation, it asks: “If I press Attack now, could one of the attack’s predicted active frames reach a vulnerable opponent?”

I’ll go through its executable lines in order, then explain the timing helpers and how the other ledge options select their responses.

**1. The function’s inputs**

```rust
fn watch_ledge(
    &mut self,
    settings: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
```

| Input | Meaning |
|---|---|
| `&mut self` | The remembered ledge-trap state. `mut` permits changing things such as preparation flags and attempt counts. |
| `settings` | Config values: selected attack, reach limits, stance distance, movement strength, etc. |
| `ctx` | Kirby’s current observations: position, facing, native status, velocity, readiness. |
| `enemy` | Opponent observations: status, position, estimated velocity, vulnerability and animation timing. |
| `out` | The proposed stick/button input. This function edits the proposal; it does not directly force a game action. |

`return;` anywhere below ends this function for the current observation. It does not permanently disable the tool.

**2. Calculate the opponent’s horizontal position relative to Kirby**

```rust
let dx = enemy.position[0] - ctx.position[0];
```

`position[0]` means horizontal position, or X.

For example:

```text
Kirby X = 50
Enemy X = 55
dx = +5 → enemy is to Kirby’s right
```

A negative `dx` means the enemy is to his left. This alone doesn’t say whether they are in front: that also depends on Kirby’s facing.

**3. Position Kirby while the opponent is hanging**

```rust
if is_hanging(enemy.status)
    && self.position_at_ledge(settings, ctx, out)
{
    self.grounded_tilt_prepared = false;
    return;
}
```

- `is_hanging(enemy.status)` checks whether the opponent is catching or hanging on the ledge.
- `&&` means both conditions must be true.
- `position_at_ledge(...)` may supply movement or turning input. It returns `true` when it reserves this observation for positioning.
- `grounded_tilt_prepared = false` clears any previous tilt preparation because Kirby is repositioning.
- `return` prevents adding an attack on that positioning observation.

This is the older policy-level positioning path. In the current L3 mode, the ground-positioning controller in `ledge_position.rs` can replace the hanging-position output afterward. **After the opponent begins normal get-up, that hanging override no longer applies.**

**4. The remaining hanging behavior**

```rust
if is_hanging(enemy.status) && ctx.can_act {
    out.stick = Some([0.0, 0.6]);
    self.grounded_tilt_prepared = false;
    return;
}
```

If the opponent is still hanging and Kirby can act:

```rust
Some([0.0, 0.6])
```

proposes:

- X stick: `0.0` — no horizontal input.
- Y stick: `0.6` — upward input.

It does not press Attack.

Again, the current ground-positioning controller normally replaces this hanging output with its own positioning or neutral waiting input. This upward stick is **not** the forward-tilt punish.

**5. Once get-up begins, start with neutral stick**

```rust
out.stick = Some([0.0, 0.0]);
```

This explicitly proposes centered stick.

`Some(...)` means the assist is providing a stick value. It differs from `None`, which means no stick proposal.

This line is part of the walking fix: the normal-get-up branch starts neutral instead of carrying forward input while waiting.

```rust
if !ctx.can_act || ctx.status != 0 || enemy.situation != 0 {
    self.grounded_tilt_prepared = false;
    return;
}
```

Here:

- `!ctx.can_act` means Kirby is not currently considered actionable.
- `ctx.status != 0` means Kirby is not in native idle/WAIT.
- `enemy.situation != 0` means the opponent is not grounded.
- `||` means **any one** of these conditions is enough to stop.

If any condition fails, Kirby keeps the neutral-stick proposal and makes no attack request.

The native-idle requirement prevents starting this punish while Kirby is walking, turning, or braking. `can_act` is the adapter’s general eligibility flag; the game still decides whether it accepts the actual attack input.

**6. Select the attack and its timing model**

```rust
let (stick, pending, first, last) = match settings.getup_attack {
```

`match` chooses a branch based on:

```ini
ledge_getup_attack = forward_tilt
```

The chosen branch supplies four values:

| Value | Purpose |
|---|---|
| `stick` | Direction to use for the move. |
| `pending` | Which requested move the policy should watch for afterward. |
| `first` | First modeled active frame after the input. |
| `last` | Last modeled active frame after the input. |

For down tilt:

```rust
GetupAttack::DownTilt => (
    [0.0, -0.6],
    Pending::DownTilt,
    4,
    6,
),
```

This means:

- No horizontal input.
- Downward stick at `-0.6`.
- Remember the request as `DownTilt`.
- Test predicted connections at offsets **4, 5, and 6**.

The actual source writes that branch on one line; the expanded form above has the same meaning.

For forward tilt:

```rust
GetupAttack::ForwardTilt => (
    [ctx.facing.signum() * 0.6, -0.3],
    Pending::ForwardTilt,
    5,
    8,
),
```

Line by line:

```rust
ctx.facing.signum()
```

produces the facing direction’s sign: normally `+1` when facing right or `-1` when facing left.

```rust
ctx.facing.signum() * 0.6
```

therefore becomes either:

```text
Facing right → +0.6 horizontal stick
Facing left  → -0.6 horizontal stick
```

```rust
-0.3
```

adds a downward angle.

```rust
Pending::ForwardTilt
```

records which animation should confirm that the request was accepted.

```rust
5,
8,
```

make the predictor examine offsets **5, 6, 7, and 8**.

**These numbers describe the move’s modeled active window. They are not an extra delay before pressing A.** Changing them changes the prediction, not Kirby’s actual move speed.

**7. Build the reach and timing check**

```rust
let window = AttackWindow {
    first,
    last,
    range: settings.tilt_range,
    height: [-5.0, 3.0],
    front_only: true,
};
```

Each field controls a separate condition:

```rust
first,
last,
```

Use the timing offsets selected above.

```rust
range: settings.tilt_range,
```

Uses:

```ini
ledge_tilt_range = 15.0
```

from your current config.

This is a permitted **predicted root-position separation**. It does not enlarge the hitbox. A value that is too generous can authorize attacks that physically miss.

```rust
height: [-5.0, 3.0],
```

Allows the opponent’s predicted root to be between 5 units below and 3 units above Kirby’s predicted root.

These are world-position differences, not controller-stick values.

```rust
front_only: true,
```

Requires the predicted opponent position to be in front of Kirby, with a small tolerance explained below.

**8. Refuse to turn-walk after get-up starts**

```rust
if dx * ctx.facing < -0.5 {
    self.grounded_tilt_prepared = false;
    return;
}
```

Multiplying by facing converts left/right separation into front/behind separation.

Examples:

```text
Enemy right, Kirby facing right → positive → in front
Enemy right, Kirby facing left  → negative → behind
```

If the enemy is more than `0.5` units behind:

- Clear preparation.
- Keep neutral stick.
- Make no attack request.

This deliberately does **not** turn and chase. Changing this threshold doesn’t make an attack hit behind Kirby; it only changes when the policy permits an attempt.

The current stray comment:

```rust
/// attack for
```

has no timing or control effect.

**9. Forward tilt: direction and A together**

```rust
if settings.getup_attack == GetupAttack::ForwardTilt {
```

This selects the forward-tilt-specific input behavior.

```rust
self.grounded_tilt_prepared = false;
```

Forward tilt no longer uses the preparation flag. That was the forward-stick-before-A behavior that could make Kirby walk.

```rust
if self.climb_seen
    && window.may_connect(ctx, enemy)
    && self.try_tilt(pending)
{
```

All three conditions must pass, in this order:

1. **`self.climb_seen`** — normal get-up has been observed during this ledge visit.
2. **`window.may_connect(...)`** — at least one modeled active frame is both in reach and vulnerable.
3. **`self.try_tilt(pending)`** — a fresh attack request is permitted by release history and attempt limits.

Rust’s `&&` short-circuits. If the timing check fails, `try_tilt()` is not called, so waiting does not spend an attempt.

```rust
out.stick = Some(stick);
out.requested = MAPPER_A;
```

These propose the selected angled stick and Attack **on the same observation**.

```rust
return;
```

Ends the forward-tilt path.

If the three-condition check failed, the function still returns—but the stick remains neutral and A was not requested.

**10. Down tilt: retain downward preparation**

Everything after the forward-tilt `return` is the down-tilt path.

```rust
if predicted_offset(ctx, enemy, f32::from(first))[0].abs()
    > settings.tilt_range
{
    self.grounded_tilt_prepared = false;
    return;
}
```

Breaking down the expression:

- `f32::from(first)` converts the integer timing offset into a floating-point number.
- `predicted_offset(...)` estimates the opponent’s relative future position.
- `[0]` selects horizontal separation.
- `.abs()` removes the sign, giving distance.
- `> settings.tilt_range` asks whether that prediction is too far away.

If it is too far, Kirby stays neutral. He does not approach.

```rust
out.stick = Some(stick);
```

For down tilt, `stick` is:

```rust
[0.0, -0.6]
```

So this prepares downward input without horizontal walking.

```rust
let prepared =
    self.grounded_tilt_prepared
    && self.last_output.stick == Some(stick);
```

Both must be true:

- The previous state says tilt preparation was established.
- The previous output actually used the same stick direction.

That prevents stale preparation from surviving an intervening stick change.

```rust
if prepared
    && self.climb_seen
    && window.may_connect(ctx, enemy)
    && self.try_tilt(pending)
{
    out.requested = MAPPER_A;
}
```

This is the same attack gate as forward tilt, with one additional requirement: **downward preparation must already exist**.

```rust
self.grounded_tilt_prepared = true;
```

Records that this observation supplied the downward preparation.

Consequently, down tilt normally needs a preparation observation before the A request. Forward tilt currently does not.

**11. Where the vulnerability timing comes from**

The actual timing decision is in [`AttackWindow::may_connect()`](/Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2/crates/kirby_assist_core/src/ledge_trap.rs:1203):

```rust
(self.first..=self.last).any(|frame| {
```

`..=` includes both ends. For forward tilt, it tests `5`, `6`, `7`, and `8`.

`.any(...)` returns true as soon as one of those offsets passes every condition.

```rust
let lead = f32::from(frame);
```

Converts that offset for position/timing calculations.

```rust
let hittable = enemy.vulnerable
    || enemy.vulnerable_in.is_some_and(|remaining| {
        remaining.is_finite()
            && remaining >= 0.0
            && remaining <= lead
    });
```

The opponent is considered hittable at that future offset if either:

- They are vulnerable now; or
- A known, valid protection-expiry estimate says they will become vulnerable by then.

`Option` matters here:

```text
Some(6.0) → estimated protection expiry is 6 updates away
None      → expiry is unknown
```

Unknown does not mean zero. If protection is unknown and the opponent isn’t currently vulnerable, the attack waits.

```rust
let [x, y] = predicted_offset(ctx, enemy, lead);
```

Gets the predicted horizontal and vertical separation at that offset.

```rust
hittable
    && x.abs() <= self.range
    && (self.height[0]..=self.height[1]).contains(&y)
    && (!self.front_only || x * ctx.facing >= -0.5)
```

A connection is permitted only when the **same future offset** passes:

- Vulnerability.
- Horizontal range.
- Vertical range.
- Facing.

For example, with forward tilt’s modeled `5–8` window:

| Protection expires in… | Timing result, assuming reach/facing pass |
|---|---|
| 9 updates | Wait: none of offsets 5–8 is vulnerable. |
| 8 updates | May press A now: offset 8 can overlap vulnerability. |
| 6 updates | May press A now: offsets 6–8 can overlap vulnerability. |
| Unknown | Wait until actual vulnerability is observed. |

That is why the attack can be requested **before protection expires**.

**12. How future position is estimated**

[`predicted_offset()`](/Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2/crates/kirby_assist_core/src/ledge_trap.rs:1223) approximately calculates:

```text
Future separation =
    current enemy position − current Kirby position
    + estimated enemy travel
    − estimated Kirby travel
```

For normal get-up and roll, enemy travel is capped at the reported animation time remaining. It avoids continuing the animation’s measured movement indefinitely after the animation ends.

Kirby’s own momentum contribution is limited to two timing units:

```rust
finite(ctx.velocity[0]) * lead.min(2.0)
```

This is a prediction assumption, not a measured stopping simulation. The model does not calculate exact animated hitbox/hurtbox collision.

**13. What controls retries**

[`try_tilt()`](/Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2/crates/kirby_assist_core/src/ledge_trap.rs:410) rejects a new request when:

```rust
self.last_output.requested & MAPPER_A != 0
```

A was requested on the previous output. A release observation is required before another press.

```rust
self.attempts >= MAX_ATTEMPTS
```

The request budget has been reached.

```rust
self.tilt_animations >= MAX_ATTEMPTS
```

The accepted-animation budget has been reached.

`MAX_ATTEMPTS` is currently `6`.

If permitted:

```rust
self.attempts += 1;
self.pending = pending;
true
```

It counts the request and remembers which move should appear.

Elsewhere, `observe_action()` distinguishes:

```text
Requested A → observed tilt animation → observed hit confirmation
```

A request alone is not a confirmed attack or hit. An accepted tilt pauses further requests while its animation is in progress. A confirmed hit finishes the punish; a recovered whiff can reopen requests within the remaining limits.

**14. Where the other options select their controls**

[`choose_response()`](/Users/bidoof/Documents/Development/AutoSmashMod/kirby_assist_v2/crates/kirby_assist_core/src/ledge_trap.rs:711) selects the plan:

| Observed option | Plan | Response function |
|---|---|---|
| Normal get-up: `CLIFF_CLIMB` | `Watch` | `watch_ledge()` |
| Roll: `CLIFF_ESCAPE` | `Roll` | `respond_to_roll()` |
| Get-up attack: `CLIFF_ATTACK` | `Shield` | `respond_to_getup_attack()` |
| Standard ledge jump | `NeutralAir` | `respond_to_jump()` |
| Witnessed ledge drop, when enabled | `DropShield` | Shield response in `evaluate()` |

It uses current native ledge status when available, otherwise previous-status history. That helps recover an option whose initial transition was missed.

Normal-get-up detection is also remembered through:

```rust
self.climb_seen |=
    enemy.status == CLIFF_CLIMB
    || enemy.previous_status == CLIFF_CLIMB;
```

`|=` means “keep this true once either condition becomes true.” Thus the normal-get-up punish can continue after the opponent enters ordinary idle.

**15. Which values to adjust for each symptom**

| Symptom or desired change | Relevant control |
|---|---|
| Choose forward tilt versus down tilt | `ledge_getup_attack` |
| Change initial waiting position | `ledge_stand_distance` |
| Change how far away the predictor permits a tilt | `ledge_tilt_range` |
| Change forward tilt’s direction/angle | `[ctx.facing.signum() * 0.6, -0.3]` |
| Correct the attack’s startup/active-window model | Forward tilt `5, 8`; down tilt `4, 6` |
| Change allowed vertical separation | `height: [-5.0, 3.0]` |
| Change retry budget | `MAX_ATTEMPTS`, which also affects other responses |
| Change roll interception travel | `ledge_roll_distance`; unrelated to neutral-get-up tilt timing |

**Don’t change `5–8` merely to make the attack “faster.”** Increasing those offsets can make the predictor authorize an earlier input because it looks farther ahead, but the actual move still follows the game’s timing. Those values should represent the real input-to-active timing. A deliberate extra delay should be implemented separately rather than hidden inside incorrect move data.

Your current `ledge_tilt_range = 15.0` is also worth evaluating carefully: the code accepts it as a prediction threshold, but it has not established that every target is physically hittable from that separation.

No code was changed or built for this explanation.