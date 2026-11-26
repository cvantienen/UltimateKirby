# Ledge trap: recognize an option, play your saved inputs

Ledge trapping is a **file-authored response**, not a compiled Kirby/Yoshi move
picker. It observes an opponent's ledge option, selects the corresponding saved
row for your fighter, and plays that row's controller inputs. Missing or empty
rows mean no response. There is no compiled attack fallback, automatic two-frame
punish, inferred hitbox timing, or automatic throw.

## Vocabulary and activation

- **Native** means a fact reported by Smash, such as ledge-climb status or motion frame.
- **Anchor** is the copied world position of the witnessed cliff. It lets an
  authored `ledge` direction or movement step refer to that particular edge.
- **Timeline/tape** means an ordered list of button, stick, wait and check steps.
- **Ownership** means this proposal wins the shared input overlay; it does not
  alter the fighter's status or cancel native recovery.

Use the assigned **Ledge assist** hold or its toggle as described in
[Controls](CONTROLS.md). A held role stops when released; a toggle retains its
selection until changed/reset. Global OFF, menu editing, invalid controller scope
and safety can suspend either. Activation is not a guarantee that every saved
row can start: native observations, selected fighter configuration and the shared
owner gates must also admit it. Held ledge activation and the persistent mode are
not the same thing as a button recipe's `while_held` loop.

Active ledge admission excludes new automatic roll/spot/air, shield-grab/backup
and incoming-grab follow-up searches. A delivered Grab can retain its input-free
receipt; ledge activation cannot replace that queued Grab with Special or a throw.
Other committed/safety owners retain their protections. Held Defense/Counter can
explicitly displace a nonsafety committed owner. See [Controls](CONTROLS.md)
for actual role bindings rather than assuming L or X in every saved profile.

## Recognition and one response per visit

| Observed native option | Saved response slot |
| --- | --- |
| Normal climb | Neutral get-up |
| Ledge roll | Roll |
| Get-up attack | Attack |
| Ledge jump stages | Jump |
| Airborne release after a witnessed hang, not an excluded damage/capture state | Drop |
| Still hanging | Watch only; no automatic attack |

A sticky previous option status can preserve recognition across a late sample.
Tracking retains the target ID and, when supplied, its cliff anchor. A selected
row completes this attempt; it does not repeatedly attack while that enemy hangs
or retry a finished tape. Native catch/pull/hold/pummel/throw immediately hands
control back so pummels, throws and follow-ups remain physical choices. Rearm
through the normal new-visit/activation lifecycle. A preview deliberately plays
a chosen slot against the observed target; it does not forge a get-up status.

Distance-agnostic tracking ignores acquisition/follow X/Y bounds, but still
requires finite copied positions. With it disabled, initial hanging acquisition
uses the supplied cliff anchor when available; following uses the target root
and separate follow bounds. These are root-space limits, not measured hurtboxes.

## Edit a response: worked example

In Training, open your fighter's ledge response list and edit its **Neutral
get-up** row. Saving, file locations and choice selection are documented in
[Sequence files](SEQUENCE_FILES.md). For example:

```ini
[kirby.neutral.practice_nair]
name = Practice neutral get-up nair
start_delay = 0
timeout = 120
actions = input:x+a:0:0:1, wait:1
```

This is an illustrative input experiment, **not a bundled default or calibrated
punish**. It requests the named buttons with neutral left stick for one input
frame, then waits one. Your game's mapping must make them Jump/Attack. It does
not prove nair started, compute protection expiry, or wait for takeoff. Native
shield/attack restrictions may make the input ineffective. Use
[Sequence files](SEQUENCE_FILES.md) for literal grammar, explicit spatial/check
steps and facing blocks.

To make get-up-attack playback wait for a real block, author the documented
wait-for-block step; selecting Attack alone does not add one. To approach a roll,
author movement/check actions; Roll alone does not run a compiled braking or
fixed-inset grab routine. To choose dair after Drop, author your jump/attack/stick
sequence; no hidden Kirby frame-18 drift tail is appended.

## Clock, ordering and cancellation

The first admitted observation is timeline frame zero. Progress comes from
rate-normalized native motion evidence; identical mapper polls reuse output and
hitstop does not spend authored duration. A configured shared input-delay
allowance shifts only acquisition delay, clamped at zero; it never shortens every
hold or learns network delay. Its Rust default is zero.

An ordinary ledge tape rejects a native-progress gap over 1.25 frames unless the
current already-held input or explicit waiting predicate safely tolerates it.
It never bursts missed taps to catch up. Automatic-response tapes have additional
bounded continuous-progress handling; that exception is not universal ledge timing.
Fresh requests still expire after the shared 35 ms watchdog; a 250 ms controller
sample gap resets the association. These are wall-clock safety checks, not move
frames or online ping measurements.

Damage/capture/native interruption, global or activation scope loss, invalid
clock, timeout, ground-only takeoff and unavailable targets for target-dependent
steps end playback. Ordinary physical gameplay input is masked while a tape owns
it, not automatically treated as cancellation. Explicit until-manual holds are
the exception: fresh manual intent hands back that same sample. Losing one target
does not permit retargeting an active response to an unrelated fighter.

## Position preparation and shielding are separate

Optional ground preparation searches the current floor for a candidate edge,
remembers an inward standing offset and proposes damped stick movement. It yields
in air and on failed readiness/support; landing requires a fresh floor selection.
This collision edge is not proof of native cliff grabbability. Saved response
movement is a separate timeline and does not inherit an automatic return stance.

<a id="held-l-defensive-shield"></a>

### Ledge defensive shield

The preserved `held-l-defensive-shield` link is a historical anchor, not a claim
that every layout or saved binding uses L; use the current assigned ledge role.

`autopilot_shield = true` is the Rust startup fallback for the ledge shield
safeguard. It proposes ordinary neutral-axis Shield for an eligible incoming
attack, and a bounded hold can bridge warning gaps. It is not parry, grab defense
or a guaranteed punish. Body/forecast/hold estimates belong to
[Fighter settings](FIGHTER_SETTINGS.md); a saved file may override them. Portable
`Envelope::default()` and live tuning need not use the same forecast: do not treat
a helper constant as a universal device default.

## Extend in code, or tune as a player?

Players edit saved slots, inputs, checks and delays through the save/load workflow.
Rust beginners should start at `policies/ledge_trap.rs`: `response_slot` recognizes
an event, `start_timeline` requires its saved sequence, and
`ledge_trap/file_timeline.rs` plays it. `Option<Sequence>` distinguishes an absent
row from a configured one. Add an event only with verified native observation
and menu/config resolution; add an action only with parser, validation, playback
and focused regression coverage. Registering a fighter identity cannot generate
a new recipe.

All results remain controller inputs. Stage shape, moving platforms, startup,
invulnerability, online queues and input acceptance can defeat a Training setup.
This documentation does not establish a successful on-device punish.
