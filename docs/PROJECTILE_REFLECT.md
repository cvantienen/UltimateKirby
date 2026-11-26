# Projectile defense: Mr. Game & Watch Oil Panic admission

The current projectile-reflection adapter supports **Mr. Game & Watch only**.
It requests ordinary Down+Special for a classified reflectable/absorbable incoming
projectile while Oil Panic's gauge is not full. It does not grant a universal
reflector to every fighter, change a projectile, or intentionally discharge a
full bucket as automatic defense.

## Configure and activate

An explicit `reflect_default` override is required for the supported fighter;
omission is not a generic enable. Schema fallback is false. The supplied Game
& Watch file enables it, with startup15, margin8, max-hold45 and estimated body
half-width15/height16/half-depth15. Schema fallbacks are startup6, margin2,
max-hold45 and body5/16/5. See [Fighter settings](FIGHTER_SETTINGS.md) for exact
key bounds and [Controls](CONTROLS.md) for current row/mode admission.

```ini
[mr_game_and_watch.settings]
reflect_default = false
```

This is an explicit opt-out example using the canonical `mr_game_and_watch`
profile namespace. The source support identity is GAMEWATCH.
[Sequence files](SEQUENCE_FILES.md) owns file grammar and precedence. Do not
assume numeric `reflect_max_hold` means the adapter will hold a native bucket
for that long—it does not.

## Read-only threat selection

The live adapter reuses a bounded ATTACK-source snapshot, not a new whole-stage
projectile search. It validates active hostile weapon/item IDs and projectile
classification, finite velocity, situation mask and geometry. Only indirect
hitboxes marked reflectable or absorbable qualify. At most16 expensive geometry
slots are read; incomplete scans abstain rather than selecting guessed safety.
The controlled fighter must be normal scale (0.99–1.01), movable/eligible and
not captured. Shield state is excluded. Gauge must be finite in **[0,3)**;
full/invalid gauge produces no automatic defense.

Forecast uses estimated body and startup+margin+fixed two-frame allowance,
within a bounded40-frame horizon. Ground facing correction can propose a0.6
turn input; it cannot force facing or erase motion. Missing pre-delivery threat
or prolonged turn ends an unsent attempt. Frozen projectile/native stop does
not advance toward catalog startup or authorize a new special.

## Delivery, native confirmation and cancellation

The policy emits a bounded Down+B request. Once a winning mapped packet is
acknowledged, it sends **no further input** and retains a bounded receipt.
Observed native bucket state ends assist ownership immediately; it does not
hold B for45 frames or manage bucket release for you. Choose subsequent input
manually. A disappearing threat after delivery does not instantly forget a
possibly queued special, but invalid movement/status, missing acceptance,
24-tick/receipt expiry or delivery budget ends that record.

Unsent attempts abort after a short four-frame budget; ground turning has a
short two-frame bound. `max_hold` is checked before delivery, not promised native
bucket duration. Stop returns empty new input. Remembered source generation
prevents the same projectile being answered repeatedly; a new generation reusing
an ID can rearm. Disabling mode/config, lost controller/session, menu/handoff,
capture or invalid gauge/geometry cancel admission. Mapping is not proof of a
reflection/absorption or optimal timing.

## Player experiment and Rust extension

Start with one known projectile in Training, then test both facings, no incoming
projectile, full bucket, stopped/invalid threat and immediate manual handback on
bucket startup. Change only one startup/body estimate at a time. Absorbability
and reflection still depend on native move/attack behavior and online queues.

Rust readers can follow `src/live/reflect.rs::observe` for copied native geometry
and the Game & Watch reflector policy for attempt/receipt state.
`ProjectileKey` includes generation so reused object IDs are not confused.
`Option<Threat>` means validated candidate or no candidate, not an invitation
to use the nearest unverified item. Adding another fighter needs verified
reflector readiness/gauge/status/input semantics and focused classification,
generation, receipt, scale and cancellation tests. It must remain input-only;
adding a tuning key alone cannot supply native reflector behavior.
