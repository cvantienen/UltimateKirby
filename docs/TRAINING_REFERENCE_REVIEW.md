# Training Modpack reference review

Reviewed Ultimate Training Modpack at commit
[`077a8e43b11d1f61ce57169156fb82e92e784c79`](https://github.com/jugeeya/UltimateTrainingModpack/tree/077a8e43b11d1f61ce57169156fb82e92e784c79).
The source inventory was used to select runtime logic relevant to detection,
input timing and workload. This is a targeted review of the areas below, not a
line-by-line audit of its UI, updater, camera, character buffs or save-state code.

| Reference area | Finding and decision here |
| --- | --- |
| [Tech](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/tech.rs) | CPU tech interception checks passive eligibility at an impending ground/wall/ceiling transition, then substitutes a tech status. That is not a shield-input predictor. Keep our input-only approach; do not port status replacement, forced get-up states or motion changes. |
| [Hook dispatch](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/mod.rs) / [mash](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/mash.rs) | Gate work by mode, controller/fighter and command category. Our command observer already restricts policy entry to category 1, validates identity/session, and guards duplicate evaluation. Keep the pre-map refresh: removing it merely to imitate one fighter callback could delay input delivery. |
| [Frame counters](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/frame_counter.rs) | Distinguish in-game progress and real time. Our timing uses native motion/hitstop progress and mapper acknowledgment; retain those protections rather than count callback invocations or wall-clock time as game frames. |
| [Input recording](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/input_record.rs) | Bounded recording/playback and explicit release/flick handling are useful patterns already present in our bounded action timeline and final mapper overlay. Some playback paths directly start statuses; those are outside our input-only boundary. |
| [Input delay](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/input_delay.rs) | Its queue deliberately delays mapped controller inputs for training. Hook order may place that queue before or after our mapping acknowledgment, so the diagnostic trace labels its boundary and records native observations separately. A manual test-delay label groups comparisons without reading private Training settings, adding a delay queue or estimating online latency. |
| [Ledge](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/ledge.rs) | Pick an option once for a ledge visit rather than re-roll it every update. Our recognized-option/timeline state already latches the choice. Its direct status requests are not copied. |
| [Shield](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/shield.rs) | Edge-latch shield-stun events so repeated observations do not repeatedly spend a reaction count. Keep our existing native-stop and acknowledgment handling. Shield decay/damage parameter changes are outside scope. |
| [SDI](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/sdi.rs) / [DI](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/directional_influence.rs) | The reference can override an SDI result/direction and write internal DI work values. Retain our simulated-stick path; do not port those internal writes. |
| [Shared helpers](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/common/mod.rs) | Keep cheap situation/status/controller checks before detailed reads. We already share copied facts within one observation, without retaining native pointers between callbacks. |

## Applied changes

- The recipe editor displays one-based nominal input frames and a total input
  schedule. F1 is the recognition update, not necessarily the enemy animation's
  first frame. Distance/face/range/block checks remain explicitly variable.
  Calculation runs only while rebuilding the existing cached panel.
- Silent tech assessment no longer requests damage velocity merely for
  diagnostics when the eligibility gates have already rejected prediction.
  Eligible prediction reads, rays, geometry, query order, cooldown, arrival
  margin, mapper acknowledgment and air-dodge prevention are unchanged.
  Diagnostics retain the rejected-state velocity evidence.
- Raw-input calibration reports no longer use missing move models/readiness or
  optional opponent samples to claim an input did not run. Raw/applied/mapped
  evidence remains separate from observed native move starts and hits.

## Limits and remaining opportunities

The reference does not supply an input-only replacement that proves zero-miss
techs. Its collision-status interception observes a later decision point than
our pre-impact shield scheduling. Checking its transition gates blindly before
impact could suppress a valid early input, and copying its status substitution
would violate this project's boundary. No prediction window, cooldown, collision
shape or air-dodge guard was changed without such evidence.

Our predictor still approximates collision geometry and future movement. Native
collision-bound observations and platform motion are possible future research,
but require verified read-only APIs and on-device cases before changing coverage.
The two observation entry points can repeat some preliminary setup; final policy
publication is already guarded. Further deduplication must retain current-frame
roster discovery, identity validation and pre-map timing. No cross-frame live
fact cache or lower polling frequency was introduced.

Host tests cover schedule accounting, variable checks and misleading diagnostic
explanations, alongside the existing tech regressions. They cannot establish
online tech success or thermal performance on hardware. No mod build or Switch
upload is part of this review.
