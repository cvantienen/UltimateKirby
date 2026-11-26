# Automatic counter: supported fighter, held/toggled admission

The current counter engine supports **Lucina only**. This is verified by
`auto_counter::supports`, `Config::for_fighter`, `Counter::sample` and the live
adapter's support filter. A general-looking counter setting or a new fighter
name does not enable another fighter. **Universal Auto defense is separate**:
it shares contact geometry but does not inherit Lucina-only counter admission.

## Activate and configure

Assign the Counter hold or Counter toggle built-in through
[Controls](CONTROLS.md). A hold releases admission when released; toggle retains
selection until changed/reset. Supported fighter, global/mode/session gates and
native down-Special readiness still apply. Counter/Held Defense have explicit
priority over nonsafety committed owners; native safety wins over them.

Rust fighter fallback `counter_default` is false. The supplied Lucina file
explicitly enables it and tunes startup7, margin3 and body half-width9/height17/
half-depth6. These are **supplied tuning**, not universal schema defaults:

```ini
[lucina.settings]
counter_default = true
counter_startup = 7
counter_margin = 3
counter_half_width = 9
counter_height = 17
counter_half_depth = 6
```

Merge into the existing section using [Fighter settings](FIGHTER_SETTINGS.md)'s
save/reset workflow. Schema fallbacks are startup6, margin2, body5/16/5. Startup
and margin estimate when a request might protect the body; they do not change
Lucina's actual move startup. The live counter's extra input delay is zero.
A saved startup preference is not itself a physical held/toggle request.

## How contact becomes Down+Special

The live adapter checks eligible native movement/air states, capture and
interruption, then reads bounded hostile active-hit geometry. It compares a
scaled estimated defender body with relative motion; it can include currently
active projectile contact. Known ordinary attack timing can forecast a future
root-envelope contact when no active geometry exists. Frozen attacker animation
cannot advance toward catalog startup. Unknown unrelated geometry does not
invalidate a known incoming contact, but it is never fabricated as a threat.

Valid threat evidence includes object/motion/status identity, finite frame and
contact time inside startup+margin. Counter proposes **Down+B**, with competing
sequence inputs owned. The actual physical B must mean Special in Smash.
Controller delivery is not native special startup or a successful counter.

## Receipt, retries and cancellation

Mapper acknowledgment stores a receipt without further B. Observing native
Down-Special consumes the attack and hands back control immediately; it does
not hold the counter for a configured hitbox duration. An unaccepted pulse gets
a release before another bounded attempt; time, receipt-poll and delivery limits
prevent permanent ownership. New attacker motion/generation evidence can rearm;
missing geometry alone does not erase remembered attempt identity.

Releasing hold/turning toggle OFF, loss of allowed owner/config, invalid clock,
stop, unavailable native readiness, capture/damage, target changes and bounded
expiry cancel input. A saved numeric startup cannot bypass native readiness.
Hitstop is not an invitation to count mapper polls as move frames. Request
freshness and controller association still use the shared watchdogs.

## Beginner model and extension boundary

A **threat** is copied evidence; an **output proposal** asks the overlay for input;
a **receipt** says the mapper delivered it; **native confirmation** says the move
started. None proves retaliation hit. `Option<Threat>` means contact or absence,
not permission to guess. Start at `fighters/auto_counter.rs` and
`src/live/counter.rs` when reading Rust. The same adapter's `observe_defense`
handles universal shield/air dodge and must not be confused with `observe`'s
counter support gate.

Players tune estimates/activation, not counter physics. Adding another fighter
requires verified native readiness, special status/startup/confirmation, safe
controller inputs and focused missing-data/receipt/retry/stop tests, not merely
extending `supports`. Test Lucina at both facings, with melee and projectiles,
missed contact and interrupted startup. Input-only forecast cannot guarantee a
counter or optimal timing online.
