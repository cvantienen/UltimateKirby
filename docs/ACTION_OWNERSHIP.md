# Explicit action ownership

One final selector chooses which assist owns automated buttons and sticks before
the shared input overlay. It uses the existing mapper and observations; it does
not add a hook, force an animation, cancel native move recovery or change physics.

## Interruption rules

| Situation | Behavior |
| --- | --- |
| Held ledge button / ledge mode ON | Suspends all seven Auto react events, automatic grabs, shield backup and grab avoidance immediately, including their opponent searches. They resume fresh checks only after the ledge binding and toggle are both OFF. Explicit recipe previews also suspend automatic responses. |
| Acquired ledge visit | Reserves its setup and configured response against competing configured actions. |
| Ledge response waits, release frames and final input | Remain protected until delivered or cancelled. |
| Response finishes, even if the attack misses | Releases its reservation so enabled assists can match the opponent's current state. Holding L or leaving ledge mode ON does not block them indefinitely. |
| Tech, DI/SDI or mash safety | Retains its safety priority when hit/captured or a native tech is needed. |
| Native-ready incoming-grab dodge | Sampled ahead of new offense. Cannot interrupt committed roll/OOS/ledge/safety input; native dodge startup releases Down/Guard; the saved spot-dodge Up-B recipe may run at first native-ready recovery, within its range. |
| Ready Auto react Up-B versus an unmapped automatic grab | Clears the pending grab/approach and cached grab output before emitting Up-B in the same observation. Already-delivered R, native grabs, ledge reservations and other committed modes keep their protection. |
| Shield-grab backup | A legal front grab wins; otherwise a close vulnerable target on either side can receive direct shield Up-B. The short B attempt is exclusive and bounded. Bowser follows confirmed startup with up to 69 frames of full opponent-directed drift; other fighters release on startup. |
| Manual input during confirmed Up-B drift | Releases the optional Auto react, grab-dodge follow-up or shield-backup drift tail in that observation. The initial B pulse and literal ledge/button tapes retain their protection. |
| Grounded moving grab | May acquire a front shielding, vulnerable target while walking/running/turning or normally landing when native catch readiness is present. Retains horizontal input; acquired ledge, safety and roll Up-B keep their priority. Shield grabs keep neutral axes and roll-safe intent gates. |
| Native grab/catch/hold/pummel/throw | All automatic ownership and ledge/button tapes release. Physical throw selection remains manual; no Up-B or follow-up is mixed into the grab. |
| Delivered automatic R awaiting acceptance | Cannot be replaced by Up-B. After the four-packet input budget, bounded settling owns no movement/buttons. Native grab startup or expiry clears it. |
| Fresh shield/jump during automatic Up-B | All three automatic Up-B owners share manual escape gates. The initial grounded B pulse stays protected; fresh airborne shield may hand back for a manual dodge. |
| Unconfirmed automatic Up-B with frozen clocks | A 24-distinct-delivery watchdog releases input; repeated observations/receipts do not count. Native startup disables this watchdog and retains the configured tracking duration. |
| Passive walking, crouching or shielding | Cannot steal a committed punish; ledge approach defense remains available during preparation. |
| Other committed sequence | Uses the explicit owner/interruption rules in `action_owner.rs`; unrelated rules do not freely overwrite its inputs. |
| Invalid scope, stale observation, disable or cancellation | Releases ownership and cancels losing plans rather than replaying them later. |

The ledge reservation gates all configured action owners, including future ones
unless explicitly classified as safety/passive. It protects ledge setup from
ordinary roll, OOS, grab-dodge/punish and button sequences. Regrab/vulnerable-hang
responses are part of ledge handling and can replace its drop defense.

Cancellation retains physical button-edge history so holding a button cannot
silently restart a cancelled recipe. Roll rules retain their used-roll marker.
An event that ended while another response owned control is not queued for a
late attack. A still-current qualifying event can match after handoff. Native
startup, shield-drop and recovery rules continue to decide whether an input takes
effect; reference timings do not bypass them.

Selection uses a fixed-size intent list and enum comparisons, without additional
fighter scans, ray queries, logging, network changes or delay estimation.

## Verification and extensions

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --test action_ownership
```

See [assist rules](REACTION_RULES.md), [ledge controls](LEDGE_MENU.md#l-and-the-autopilot-toggle)
and [input timing](SEQUENCE_FILES.md). Host tests cannot establish actual Switch
performance or prove that a specific input was accepted in-game.

## Configured automatic sequences

The Auto react editor groups roll, enemy spot/air dodge, automatic unshielded and
shielded grab, after dodging enemy grab, and shield backup. Grouping configuration
does not merge their enable switches or owners. Native grab/backup defaults keep
their existing delivery paths; edited actions commit one shared input timeline.
A committed edited response cannot be replaced by a competing new reaction.
Queued R in either grab mode or a configured grab-dodge punish preserves a
bounded receipt reservation with zero input while settling. It cannot fall
through to another B/Up step or resume a hidden script after native catch.

Shielded and unshielded grabs use separate target policies: only the unshielded
path requires enemy shield. A legal shielded front grab wins before the close
shield backup; rear/ungrabbable/out-of-grab-range targets can use backup only
inside its own X/Y limits. Existing usable roll/dodge reactions retain priority
before new grabs. A delivered Grab cannot switch to Special while its receipt
is unresolved, and a committed backup cannot switch back to Grab midway.
