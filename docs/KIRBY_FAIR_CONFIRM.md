# Kirby fair first-hit landing follow-up

This Kirby-only tool watches a **fresh hurtbox hit before fair motion frame17**,
then attempts the fast-fall first-hit landing route into an upward-angled forward
smash. Shield-only contact, another aerial and fair's later hits cannot arm it.
It observes fresh hit infliction, not a persistent historical hit flag.

## Activate and understand the defaults

`kirby_fair1_smash = true` is the Rust startup fallback. It is independent of
trip follow-up and does not require a ledge hold, but selected human controller,
mode/global enabled and native safety/owner admission still apply. Disable the
startup key or relevant current mode through [Controls](CONTROLS.md); startup
file precedence is explained in [Settings reference](SETTINGS_REFERENCE.md).

```ini
[universal.settings]
kirby_fair1_smash = false
```

Merge into your existing section and use its documented reload path; a file edit
is not an immediate native-state write. Fair route internals are compiled policy,
not editable arbitrary follow-up actions. A saved button recipe is a separate
literal tape and cannot register a fresh-hit observer.

## Recognition, input and native timing

A landing-fair observation can preserve an earlier first-hit frame through a late
poll. Once armed, the tool owns competing gameplay inputs. It requests downward
stick only after finite vertical velocity is below-0.05: it does not force descent
or fast-fall physics. Landing must occur before fair reaches its second hit.
The queued air phase has a short16-update bound; a wider total age bound also
prevents permanently queued input.

On landing, it alternates ordinary releases and **facing+A with upward angle
`[facing, 0.5]`**, buffering through native landing lag. This does not make the
smash actionable early. Native forward-smash startup releases A while preserving
angle; charge/selected attack ends ownership. No percent, opponent range/DI or
additional target scan is required after first-hit confirmation. That absence
of checks is not a promise the smash will connect.

Native progress spends timing; duplicate observations reuse output and stop
does not create extra pulses. Reach/frame17 before landing, damage/capture,
unrelated action, invalid clock/identity, route disable, queued expiry and
stronger safety/explicit Held Defense/Counter admission cancel. Ordinary physical
input does not automatically cancel an already-owned route. The underlying move
may continue after assist hands back; only requested controls are removed.

## Practice and Rust extension

Test fresh first-hit versus shield and late-hit cases, descent versus ascent,
landing versus staying airborne, both facings and manual control after smash
startup. Delivery can miss native acceptance; spacing, DI, percent and queues
still determine outcome. No on-device success or zero-miss certification is
implied.

Rust beginners can follow `src/live/kirby_followups.rs::fair_hit_window`,
`fair_context` and `fair`: the adapter only admits/read-copies the hit facts;
`FairFollowup` in the Kirby policy owns progression. `Option<FairFollowup>` is
created only while enabled and cleared when disabled. Add focused fresh-hit,
late-hit, landing, velocity, duplicate/stop and interruption tests when extending
this route. Do not alter animations, hitboxes, physics or status to force it.
See [Trip follow-up](TRIP_AND_SDI.md) for the different confirmed-trip route and
[Input recipes](INPUT_RECIPES.md) for literal versus native-confirmed playback.
