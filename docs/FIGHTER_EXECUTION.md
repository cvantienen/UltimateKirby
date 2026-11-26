# Fighter input execution and calibration

All current ledge options are file-authored controller timelines. Profiles and
native frame references do not supply hidden recipes, adaptive hop selection,
compiled roll endpoints or native-acceptance retry schedules. Inspect actions
rather than trusting a move label. [Configure](FIGHTER_CONFIGS.md) and
[sequence syntax](SEQUENCE_FILES.md) are the edit/save references.

<a id="kirby-neutral-nair-walk-example"></a>

## Initial and per-action delays

This is a teaching example, not the supplied Kirby default:

```ini
[kirby.neutral.my_timing_example]
name = Timing example
start_delay = 2
actions = Short Hop, wait:1, Neutral Air, wait:2, up_special:1
```

Recognition is timeline frame zero. Frames 0–1 wait; frame 2 requests jump;
frame 3 releases; frame 4 requests neutral A; frames 5–6 release; frame 7 requests
Up+B. No jump-squat or move-startup wait is automatically imported from fighter data.
`wait:0` spends no released frame. Adjacent inputs sharing a button keep it held;
insert an explicit release if a new press edge matters. A full hop requires an
explicit hold, e.g. `full_hop:4`, and device calibration. Smash decides whether
inputs buffer, execute or fail.

Directions are part of the action. Opponent-relative X mirrors toward the observed
opponent; facing-relative X follows fighter facing; screen-relative X is absolute.
A `face` action can wait for observed facing, not forcibly set it. Movement/range
and `shield_until_block` request shared observation predicates; ordinary inputs
do not wait for them unless authored. `range:fighter_range` uses a policy estimate;
`range:20` uses an explicit number. A failed or missing fact is not a guessed hit.

For shield-to-Up+B, the input adapter can retain Shield while requesting Up+B
from grounded guard, preserving ordinary native out-of-shield input semantics.
That is input routing, not a move/status edit or a hidden delay. A shield backup
needs its own admitted reaction and range; a label saying Up-B is not evidence
that the actual action field contains Up+B.

## Native progress and online limits

The sequence cursor uses copied native motion progress/rate, not wall-clock sleep
or controller-poll counts. Duplicate observations do not spend another frame;
hitstop does not advance waits. A zero-rate pose may use the locked opponent's
advancing motion as fallback; the clocks are not added. Invalid progress or a
gap over the scheduler's allowed sample budget cancels rather than bursting
missed inputs. [Sequence timing](SEQUENCE_FILES.md) documents the exact clock
and receipt rules; [ownership](INPUT_RECIPES.md) explains competing tools.

Mapped input, native move startup and successful contact are three different
observations. A receipt can acknowledge delivered input without proving a hit.
Late recognition, state readiness, landing, shield-drop recovery, differing
motion rates, stage geometry and network delay can defeat an otherwise valid
tape. There is no learned delay that silently retunes your authored waits.

## Chart-based roster configurations

Earlier guides described F34 normal-getup/F14 jump/F31 roll chart targets as
if they were compiled scheduling. Those are calibration targets, not universal
opponent animation data or automatic runtime leads. The **current supplied
snapshot** for each fighter is in its generated page, including explicit delays
and actions. Kirby, Bowser, Yoshi, DLC/form starters and user-edited files can
have entirely different records; do not infer a roster-wide default count.

To plan coverage yourself: account for recognition offset, explicit tape holds/
waits, native acceptance/jump squat, move startup and the particular continuous
active phase. Separate front/rear or multihit phases; do not count their gap as
coverage. A charge-release attack needs explicit charge and release, not a larger
startup estimate. A projectile needs travel/resource setup; emission is not
arrival. A timed grab needs correct facing, vulnerability and actual range.

Current simple automatic events are six independent fighter records, not seven:
enemy roll, spot dodge, air dodge, shield grab, shield backup and after dodging
enemy grab. The old `auto_grab` token is rejected. Ledge Roll is separate from
an ordinary onstage roll; no ledge delay is automatically reused. Automatic grabs
are shield-only. [Reaction configuration](REACTION_RULES.md) explains current
range/admission; legacy priority rules are not the simple event editor.

## Repeatable calibration checklist

1. Record fighter kind, installed catalog/build, selected ID, explicit actions and
   current mode/binding state. Save the draft and wait for confirmation.
2. Fix stage, ledge side, opponent, position, facing and percent. Begin with
   one action; add movement/charge/resource setup only when understood.
3. Compute the expected frame list including releases. Verify input acknowledgement
   separately from native startup and contact; do not infer all three from one log.
4. Test both sides, misses, target loss, hitstop, landing/interruption and manual
   takeover. Ensure failed checks return control rather than chaining stale inputs.
5. Vary timing one value at a time. Increasing reach cannot fix failed input
   delivery; increasing hold may select another special or charge unexpectedly.
6. Repeat offline and online before claiming reliability. Host tests and generated
   tables prove neither on-device arrival nor match-up-optimal timing.

For catalog edits, the offline generator/check and parser tests validate syntax
and snapshots. No Switch build/upload, gameplay measurement or asset mutation is
part of this documentation-only update. See [data](FIGHTER_DATA.md) and
[diagnostic result cards](CALIBRATION_CARD.md) for the evidence boundaries.
