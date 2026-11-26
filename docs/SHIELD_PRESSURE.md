# Shield pressure: enemy shield history, not your shield hit

Shield pressure tracks **the opponent's shield**, then can recognize a roll
originating from that shield. Out of shield instead operates from your shielding
state; [Spot punish](SPOT_PUNISH.md) observes enemy spot dodges. These are distinct
rule families, not interchangeable event names.

## Configure a detailed rule

Use the Shield pressure Training list in [Controls](CONTROLS.md). The portable
menu starts its switch ON, but a matching saved rule is required. Detailed rules
have own-state, minimum/maximum X, height, side and priority; simple Auto react
rules do not acquire these fields. Save via [Sequence files](SEQUENCE_FILES.md).

```ini
[bowser.attack.practice_pressure]
name = Close enemy shield grab
start_delay = 0
timeout = 120
reaction = enemy_shield:grounded:0:10:12:front:1
actions = input:r:0:0:4
```

This is a valid starting example, not a bundled rule or measured grab reach.
It requests four frames of mapped Grab only after the event/state/range/side
conditions match. Grounded includes shield state but does not grant permission
to cancel every animation. Priority 1 is considered before larger numbers **among
currently matching conditions**; it never waits for a future higher-priority event.
A literal Grab does not secretly calculate hitbox reach.

## Event vocabulary

| Token | Required observed enemy context |
| --- | --- |
| `enemy_shield` | Shield startup, hold or shield damage |
| `shield_roll` | Roll after shielding was observed |
| `shield_roll_cross` | Shield-origin roll crosses your relative X position |
| `shield_roll_vulnerable` | Shield-origin roll with observed vulnerability |
| `shield_roll_cross_vulnerable` | Crossing and vulnerability |
| `shield_roll_away` | Shield-origin roll with observed velocity away from starting side |
| `shield_roll_away_vulnerable` | Away direction and vulnerability |
| `shield_pressure_vulnerable` | Vulnerable shield or shield-origin roll |

The mode must see shield **before** roll; enabling midway through roll does not
invent history. Your own movement can cause relative crossing. Zero velocity
cannot establish away direction. Shield release alone, jump and spot-dodge escape
have no shield-origin event here. Unknown vulnerability does not satisfy a
vulnerable variant.

## Ordering, clock and cancellation

At most eight remembered fighter snapshots identify visits/motions and used
responses. A shield visit/roll is answered once. Once committed, later rolls do
not replace that tape. With shield-roll rules configured, this mode can claim
shield-origin rolls; a nonmatching pressure condition does not automatically
fall back to a generic roll punish. Disabling pressure removes its claim.

New selection needs usable native action/facing/range facts and nonstopped play.
A hit on **your** shield cancels old pressure playback. Native safety, active
ledge admission and other committed owners retain their protections; explicit
Held Defense/Counter can preempt nonsafety playback. The shared tape uses native
progress, pauses on stop and masks its authored gameplay inputs. Damage/capture,
scope/mode loss, clock/timeout or unavailable target-dependent facts end it.
Delivered Grab receipts cannot turn into Special or an automatic throw.

## Tune and extend safely

Try one close-shield rule, then a separate crossing rule using the full token
format; change one range/delay at a time. Immediate crossing may still be
invulnerable, while waiting for vulnerability may be too late for a slow move.
The engine does not calculate an optimal punish or retime move hitboxes.
[Fighter settings](FIGHTER_SETTINGS.md) documents estimates/default inheritance;
selected actions belong to your saved sequence, not a universal Bowser fallback.

For Rust readers, `policies/shield_pressure.rs::Pressure` keeps `Seen` history,
selects a matching configured response and marks it used. `Response` plays the
tape. `Option<bool>` away distinguishes known true/false from unknown; guessing
false/true would lose that important distinction. Add event-history, own movement
crossing, duplicate, one-response and shield-hit cancellation cases when extending
this code. The policy takes copied facts and performs no native writes; geometry,
recovery and online input acceptance remain native limitations.
