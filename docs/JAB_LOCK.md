# Yoshi down-tilt chase

Yoshi's **Jab lock** menu row is **OFF by default**. R3 opens the menu;
Up/Down selects the row and Left/Right toggles it. This is independent of L's
ledge responses and ordinary onstage **Roll read**.

Only a damaging Yoshi down tilt (`attack_lw3`) arms the chase. Shield contact
does not qualify. The victim must be in damage/knockdown and its native
last-attacker entry must identify Yoshi. Ambiguous multiple victims are skipped.
The chase keeps that opponent's ID.

| Observed result | Attempt |
| --- | --- |
| Damage/tumble | Ground-checked walking toward the observed opponent |
| Missed tech, grounded knockdown | Face the opponent and attempt one jab 1 in estimated reach |
| Confirmed jab lock | Release A to avoid jab 2, then down smash when native recovery permits |
| Tech in place or tech roll | Face/chase the opponent and attempt a standing grab in its observed recovery window |
| Ordinary get-up or get-up attack | Attempt down smash if startup fits the observed vulnerability/recovery window |
| Target actionable, missing or out of range; Yoshi interrupted | End the sequence |

Jab-lock eligibility requires grounded DOWN state, no available native tech
transition, normal hurtbox and down-damage count below two. Jab input alone
does **not** confirm a lock: accepted jab 1 must be followed by attributed
DOWN_DAMAGE and an increased native lock counter. This version attempts one
jab into down smash. Grab startup ends automation for manual pummels/throws.

This sequence takes priority over background Roll read and ledge preparation.
Their selected configurations remain saved, but they cannot insert a smash
into this chase. Safety, confirmed follow-ups and explicit manual tools retain
their interruption rules. Menu/stock/session changes clear pending inputs.
One down-tilt hit cannot rearm the chase through repeated hitstop observations.

## Calibration and implementation

- Shared detector and policy runner: `crates/kirby_assist_core/src/jab_lock.rs`.
- Yoshi constants: `crates/kirby_assist_core/src/fighter_profiles/yoshi.rs`, `JAB_LOCK`.
- Native observations: `src/live/observations.rs`, `jab_lock_targets`.
- Scheduling: `src/live.rs`; ownership: `action_owner.rs`.

Startup values follow [Yoshi's Ultimate Frame Data](https://ultimateframedata.com/yoshi):
jab **3**, front down-smash sweep **7**, standing grab **14** frames.
Root-distance estimates are jab **7**, smash **12**, grab **23** world units.
The chase is bounded to **60** horizontal / **24** vertical units and **120**
advancing native frames. These distances are tuning estimates, not measured
live hitboxes. Moving-roll grabs also check estimated reach at startup.

Inputs require native transition readiness. Requests hold up to three advancing
frames, release on accepted startup, and allow up to **24 advancing frames**
for acceptance. This avoids abandoning queued inputs after six frames; accepted
actions continue immediately, without an added timing offset.
Accepted jab 1 releases A even in hitstop. Unknown vulnerability/recovery never
authorizes an attack. Timing uses native motion progress, not controller polls.

No new hook, status/physics writes, runtime file loads or per-frame logs are
added. Detailed target reads use the existing bounded fighter roster only on a
relevant hit or during the chase. Other fighters have no policy yet.

Host tests cover detection, inputs and ownership. Switch testing is still needed
for spacing, move acceptance, tech animations and arenas; this cannot guarantee
every chase or remove network delay. Test no-tech, tech-in-place, both tech rolls,
shield/whiff, facing, edges, interruptions and disabling the row separately.
