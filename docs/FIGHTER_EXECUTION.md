# Ledge input execution and timing

All fighter options and input scheduling are authored in
[sequences.txt](SEQUENCE_FILES.md). There are no compiled fighter ledge recipes,
adaptive hop policies or native-acceptance retries behind the menu choices.

<a id="kirby-neutral-nair-walk-example"></a>

## Initial and per-action delays

```ini
[kirby.neutral.nair_wait]
name = NAIR LATE
start_delay = 2
actions = Short Hop, wait:1, Neutral Air, wait:2, up_special:1
```

Frame zero is recognition. This waits on frames 0–1, presses Jump on frame 2,
releases on frame 3, presses neutral A on frame 4, releases on frames 5–6, and
presses Up+B on frame 7. Omit a wait (or use `wait:0`) for the next input on the
next frame. Use `input:A+X:0:0:1` for simultaneous buttons on one frame.

A bare input is a one-frame press. `move:N` holds it for N frames. Adjacent inputs
that share a button keep that button held; use a released frame for a new edge.
A full hop needs an explicit hold such as `full_hop:4`, calibrated in the file.
The parser does not insert a fighter-specific jump-squat wait or change the game.
Early inputs can buffer or fail under Smash's ordinary rules.

`face`, distance-based movement, explicit grab range checks and
`shield_until_block:MAX_FRAMES` request shared observation logic. The block action
holds shield until a block and the end of native shieldstun, then advances in the
same observation. It cancels if the file-authored timeout expires. Simple inputs
never wait for these checks unless the recipe explicitly asks for them.

For `shield_until_block:90, up_b:1` (or a longer configured Up-B press), the
delivery adapter retains Shield with Up+B while the fighter is still in grounded
guard. This preserves the native out-of-shield cancel without a shield-drop
frame. The temporary guard bit is not stored in the input tape and disappears
after leaving guard, including duplicate-clock observations. Explicitly authored
shield inputs, pulse lengths and tracking steps keep their existing meaning.

## Online timing

The cursor advances from copied native motion progress divided by motion rate,
not controller-poll counts or wall-clock sleeps. Duplicate polls reuse output;
hitstop preserves held inputs without spending waits. No guessed online latency
is added to file delays. Invalid clocks and observation gaps over 1.25 native
frames cancel rather than burst overdue inputs into one frame.

For a zero-rate pose, the scheduler can use the locked opponent’s advancing
native motion clock instead. That fallback reads only the locked actor, without
a roster scan, hitbox/readiness lookup or ground ray. The two clocks are never
added together; missing samples/hitstop rebase the fallback rather than charging
a catch-up gap. Floating-point roundoff is normalized at exact frame boundaries.

This is an observation clock, not a verified global simulation counter. Online
input delay still affects game consumption like it does for physical inputs.
Host tests do not establish Switch delivery accuracy: test the installed build
both offline and online at repeatable positions before calibrating tight punishes.

## Validation and calibration

1. Edit the selected fighter/situation section and its explicit `start_delay`.
2. Set each hold and inter-action `wait` in the file; check the expected frame list.
3. Run `python3 tools/upload_sequences.py --dry-run` to validate both build filters.
4. After installing the updated mod and uploading the file, restart Smash and
   select the option. Check the first input frame, holds/releases, and target spacing.

Labels/defaults come from the file. The fighter documentation generator reads
those same sections. Old native frame tables are independent reference facts;
they do not import a schedule or change the game's actual move definitions.

## Chart-based roster configurations

The 74 fighter files other than Kirby, Bowser and Yoshi use these authored
ledge timing baselines. Those three fighter files retain their saved settings.

| Recognized ledge option | Target | Configuration |
| --- | --- | --- |
| Normal getup | F34 | Center a documented continuous active phase on the target |
| Jump getup | F14, with F13–15 coverage | Include the native three-frame jump squat for aerials |
| Ledge roll | F31 | Stage an attack or standing grab at a prepositioned roll endpoint |
| Getup attack | Confirmed block and shieldstun end | Hold shield, then issue a direct fighter-specific out-of-shield input |
| Ledge drop | No fixed getup window | Keep spacing/recovery choices separate from the getup schedule |

These are the user's chart targets, not universal opponent animation data.
Calculations assume recognition at opponent option F1, immediate native
acceptance, correct position/facing, and an already released shield for ordinary
ground attacks. Late recognition, shield-drop recovery, motion rates, differing
opponents, platforms and hitbox geometry can shift or defeat contact. The files
are configurations for calibration, not measured frame-perfect guarantees.
See [opponent ledge statistics](https://ultimateframedata.com/stats).

Each fighter file links its move-data source. Documented active intervals and
the three-frame jump squat supply the input lead. Disconnected multihit phases
are not treated as a continuous hitbox; uncertain rehit/extended/resource phases
use a conservative documented phase. Even-length normal-getup phases use the
later middle move frame to center their coverage over F33–34. Charged smashes use the published charge
point and spend the spare lead holding Attack, followed by an explicit release.
Native charge entry/release still needs device calibration. Front/rear
down-smash branches select their phase at entry, so position for the roll
endpoint; they do not predict a later crossover. Timed grabs abort immediately
if the target is behind or outside standing reach instead of walking and
missing the authored window.

Every in-scope fighter has one recipe for each of the seven current Auto-react
events. Ordinary roll/spot/air dodges can first be recognized partway through
their animation, so their responses stay immediate rather than borrowing ledge
delays. Close ground events use character-specific direct Up-B, up-smash or
jump attacks; airborne targets use upward/neutral coverage. Miis use attacks
that do not depend on the selected special loadout. The two generic automatic
grab recipes retain their ranges; Game & Watch's stray pre-grab drift is removed.
No automatic punish adds another spot dodge or Bowser's spinning drift.
Automatic admission remains grounded; configured aerial tapes may finish after
takeoff. New attack ranges are conservative starting envelopes, not measured
hurtbox reach or proofs of contact.

The existing entry limit also counts generic templates. Forty-four weaker
generic alternatives were retired to fit all events, retaining five populated
ledge rows and the grab choices for every fighter. Seven other generic choices
were replaced within their rows with the selected fighter-specific move.
Twenty-five spacing/resource/charge-hitbox alternatives remain explicitly
marked for calibration in the text files and are not selected as defaults.
This includes projectile travel, held-fruit requirements and Ness's yo-yo
charge hitboxes. Samus/Dark Samus shield Zair retains its airborne tether input;
an instant airborne check prevents a failed hop from becoming a ground grab.

No runtime code, native physics, move definitions or animation rates are changed
by these files. Parser checks and host input-runner tests cannot establish
on-device hit timing; validate the selected response in Training before relying
on it in matches.
