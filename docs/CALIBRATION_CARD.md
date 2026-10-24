# Frozen diagnostic ledge result

The diagnostic build records a **completed result card** for each recognized
ledge response attempt: **Neutral, Jump, Roll, Attack and Drop**, using the
selected fighter's saved response. Silent builds exclude the recorder, core
telemetry and card. This is a viewer, not automatic tuning: it changes no move,
input priority, spacing decision or response timing.

## Use it

1. Use a newly built diagnostic NRO with `imgui-menu` and its matching ImGui host.
   Source edits do not update existing releases or the installed plugin.
2. In battle, select a fighter response in the normal menu. Hold L and have the
   opponent perform that ledge option. Training is useful for repeatable tests.
3. Once the attempt ends, a compact card appears **to the right of the menu**.
   It freezes there while you try again; the next completed result replaces it.
4. **R3** hides/shows both panels. Results still record while hidden. Cards
   clear when the battle ends or the controlled fighter changes; stock loss
   alone does not clear the last completed card.
5. Read the full report in `SD:/ultimate/kirby_assist/status.log`. Search for
   `LEDGE RESULT`. This also works in diagnostic builds without the GUI feature.

A result finishes on capture, eligible hit evidence, response handback, selected
move exit/interruption, passive shield block, target option ending, cancellation
or a four-second watchdog. That watchdog only closes telemetry. A new option or
visit starts a new report; a continuously held completed response does not
produce duplicate cards every callback. Release/re-hold L to retry the same
setup. Queued results from a previous battle/fighter cannot reappear later.

## Read the fields

| Field | What it tells you |
| --- | --- |
| Fighter / row / choice | Which compiled response was selected. The full log also records the actual native fighter kind, so a fallback profile can be identified. |
| Config window | Configured first/last move frames, not a measurement of collision boxes. Missing bounds show unknown. For some specials only startup is known. |
| Hop / lead / drift / fast fall | Committed short/full hop, configured early lead where exposed, signed horizontal stick drift and its end frame when available. Positive drift is world-right, not always toward the opponent. Opted-in fast fall shows its minimum nair animation frame and descent requirement. Unknown or not applicable stays unknown. |
| Read | First observation of the recognized response, with opponent status, native motion frame and motion rate. |
| Protection off | First sampled observation of the opponent being vulnerable; absence does not prove they stayed invincible. |
| Request | First intended attack, grab, special or shield request. Preparatory jump/shield requests do not replace this spacing snapshot. |
| Mapped J/A/G/B/S | Whether requested Jump, Attack, Grab, Special or Shield was observed through the controller mapper. This is delivery evidence, not proof of native action acceptance. |
| Native jump / move | First observed jump-squat/jump or selected move state after a corresponding request. A different aerial does not count as the selected aerial. |
| Own last / window pose | Last native status/frame and first observed pose inside the configured move window. A smash charge state does not count as active attack pose. |
| Gap / height / ref | Signed root-position gap relative to facing, height difference and configured range/reference at the first intended move request; falls back to the last observation if no move was requested. Negative gap means behind. These are not body-edge or grab-hitbox distances. |
| Roll aim / stand | Existing projected roll endpoint and desired standing position, when available. These use world X coordinates, not distances from Kirby. |
| Wait / mapper | Most recent waiting phase and input priority/gate evidence. The last wait can describe an earlier preparation phase after an action starts. |
| End | Why collection stopped and elapsed wall-clock milliseconds. It is not a gameplay timeout. |

The full log additionally preserves separate first request stamps for all five
input channels, raw requested/applied/mapped button masks, readiness flags,
remaining protection/recovery estimates, own position/facing/velocity, exact
last own motion hash, hit evidence and whether the target changed motions.

## Interpret results carefully

- **CAPTURE confirmed:** the existing assisted-grab handback was observed.
- **HIT observed; victim unverified:** native hit evidence was sampled in the
  selected move's configured window. The recorder does not identify the victim.
- **START observed; hit not confirmed:** the selected native move started, but
  the available observations did not confirm a hit. This is not proof of a miss.
- **REQUESTED; no native move observed:** compare mapper delivery, readiness and
  the last own state before changing timing.
- **NO MOVE REQUEST:** start with the wait/gate, spacing and readiness evidence.
- Shield results distinguish shield startup from an observed block. Projectile
  startup does not prove projectile contact; results may end before it hits.

Every selected response uses the common recorder. Exact OneShot recipes expose
its configured window and native motion/status rules; shared nair, tilt, smash,
grab, Kirby special/bair/drop-dair and Samus zair paths expose their available
metadata. If a custom helper has no exported move metadata, its card still
records requests, spacing, gates and end reason, but marks the model/window
unknown. Do not treat unknown as frame zero or assume every custom move has a
complete active-window model. Passive shield has no attack window.

Native frames are **motion-local animation frames**, including fractional values
and motion rates; they reset on a motion change. The card/log's `+ms` is elapsed
wall time, including pauses or connection stalls. Do not subtract frames across
different motions or divide milliseconds by 16.67 to claim frame-perfect input
latency. The recorder samples existing observations: it can miss intermediate
states and does not read actual attack/grab hitboxes or guarantee online timing.

## Calibrate one thing at a time

First check request → mapped input → correct native move. If startup is absent,
changing spacing alone will not fix an input problem. Once startup is confirmed,
compare protection timing, configured active window and root spacing across
several identical attempts. For grabs, record both a successful setup and a
failed approach: aim/stand, request gap, facing, velocity and the last wait help
separate navigation, readiness and timing. Change one parameter, then repeat.

Use the [individual fighter page](FIGHTERS.md),
[execution guide](FIGHTER_EXECUTION.md) and
[response implementation guide](FIGHTER_CONFIGS.md) to locate the setting or
hardcoded sequence. The card does not write settings or edit functions.

## Cost and validation

Gameplay callbacks retain one fixed-size numeric report, copying facts already
used by the ledge engine. Completion allocates one queued report. The existing
bounded, nonblocking logger queue handles it; a full queue can drop a result.
The background logger formats the full log and compact card once per completion.
Rendering reads cached text with a nonblocking lock; it performs no fighter
queries, raycasts, hitbox scans, per-frame string formatting or file I/O.
Only the latest completed card is retained. Existing detailed grab diagnostics
continue separately, so a roll-grab can also produce its dedicated trace.

These changes have source/format and documentation checks only. No new mod build,
Rust test execution, Switch performance profiling or on-device visual validation
was performed for this change. Check panel fit and callback observations on the
Switch before relying on the readings for calibration.
