# Input recipes and posture: requests are not forced moves

This guide separates **saved literal input tapes** from **compiled helpers with
native confirmation**. A saved action does not inherit a helper's hidden recovery
wait or move acceptance check. Use [Sequence files](SEQUENCE_FILES.md) for text
syntax and [Controls](CONTROLS.md) for actual physical mappings/activation.

## Three kinds of input behavior

| Kind | What it does | What it does not promise |
| --- | --- | --- |
| Literal saved step | Requests the authored buttons/sticks/wait duration | That Smash starts the named move |
| Native-confirmed compiled helper | Uses copied status/motion evidence to bound delivery/release | Forced recovery cancellation or faster move startup |
| Posture adjustment | Changes proposed stick angle while preserving physical attack edges | An automatic Attack press or guaranteed tilt reversal |

`policies/move_input.rs` retains shared delivery and Up-B receipt helpers. A
**receipt** remembers mapped input without owning another B pulse; **acceptance**
means native move status/motion was actually observed. Its 24-packet acceptance
budget is not hitbox startup or a sequence timing offset. Ledge/button tapes are
played by `ledge_trap/file_timeline.rs`, not a universal compiled move engine.

## Posture: idle crouch and tilt facing

Idle crouch is Kirby-only. It proposes Down during grounded neutral when a known
enemy is within45 horizontal/20 vertical world units, with safe ground and no
competing physical input. Passable platforms are excluded to avoid a drop.
Physical intent, loss of target/range/ground/readiness, suspension and stronger
owners remove the proposal. It owns no Attack button. Down can remain through
native crouch entry/wait/turn/rise; crouch looping is not a reason to release.

Tilt facing is a stick helper, not an attack macro. The current configured
helper adjusts a **physical dominant C-stick** direction toward the known opponent.
Down/up uses a small horizontal component (0.2) with a dominant vertical0.8;
forward uses opponent direction with the physical horizontal magnitude. It
preserves the left stick and actual physical attack edges. Held left-stick
Down+A alone does not run the old 0.80 angled-main-stick reversal. Native tilt
recovery can retain eligibility, but there is no standing-turn tape or guaranteed
reversal before the game accepts the physical C-stick input.

Forward/up C-stick adjustments are opt-in and require the game's C-stick to be
**Tilt Attack**. The helper cannot read that controller setting: with Smash
Attack mapping, enabling them can redirect a smash. Only the real dominant
side/up C-stick intent qualifies; arbitrary left-stick+A attacks are not
converted. Physical C-stick axes/button history are restored after mapping.

Schema `tilt_turn_down` inherits true for Kirby, Yoshi **and Lucina**, not just
Kirby/Yoshi; forward/up inherit false. Supplied Lucina overrides all three false.
The global `auto_turn_tilts_active` fallback is false and `auto_turn_tilts`
directions default OFF; direction settings and current mode admission are distinct.
A direct startup example enables both shared admission and a direction:

```ini
[universal.settings]
auto_turn_tilts_active = true
auto_turn_tilts = down
```

The direction parser accepts comma/space/pipe-separated down/forward/up names
and `none`/`off` disables it. `down`, `d`, `dtilt`; `forward`, `side`, `f`,
`ftilt`; `up`, `u`, `utilt` are accepted aliases. Empty/unknown values are errors.
Keep the canonical [Fighter settings](FIGHTER_SETTINGS.md) workflow for per-fighter
save/reset; numeric tuning inheritance is not the same as toggling the current
helper ON. No universal key list is duplicated here.

## Worked literal input example

```ini
[kirby.x.practice_hop]
name = Practice neutral hop attack
start_delay = 0
timeout = 120
actions = input:x+a:0:0:1, wait:1
```

This file experiment requests those physical buttons with neutral main stick
for one frame, then waits. It does not wait for native takeoff, verify nair,
append fast fall or alter landing lag. Controller mapping determines meaning.
A player should choose explicit documented waits/checks instead of expecting a
move name to register a hidden callback. [Sequence files](SEQUENCE_FILES.md)
explains once/while-held looping, facing branches and zero-frame until-manual holds.

The old A+B jab-to-grab/native Y Finisher and hard-coded slide/B-reverse bindings
are retired. Ordinary A/B remain game inputs; current button roles and saved
recipes are documented in Controls. Legacy portable helper types or config fields
are not proof those old live actions remain enabled.

## Order, cancellation and extension

A committed literal tape masks ordinary physical gameplay inputs, including its
neutral gaps. Explicit until-manual holds are different: new manual intent ends
them. Native stop pauses progress; duplicate polls reuse output. Damage/capture,
invalid scope/clock, timeout and lost required target facts end tapes. Safety and
explicit Held Defense/Counter can preempt nonsafety owners. Posture yields to
other owners rather than silently changing their attack angle.

Rust beginners should keep **data**, **proposal**, **mapper receipt** and **native
confirmation** separate. `ControllerOutput` owns only requested controller input;
`Option<[f32;2]>` means stick proposal or no override. Players edit saved inputs
and posture preferences; developers adding an action must update parser,
validation, playback and targeted duplicate/cancellation/manual-handback cases.
Never alter move definitions, hitboxes, rates, physics or status to make a recipe
succeed. Training success still does not guarantee online acceptance.
