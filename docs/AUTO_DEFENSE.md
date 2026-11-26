# Auto defense: held universal Shield or neutral air dodge

Auto defense is a **universal input-only defense**, unlike Lucina's counter.
It reads incoming active melee/projectile contact and requests neutral Shield on
the ground or neutral Shield/air-dodge input in air when native permission allows.
It does not parry, counter, modify intangibility, remove air-dodge lockout or
shield through native incapacitation.

## Activation and settings

Assign Auto defense hold or toggle in [Controls](CONTROLS.md). It has the explicit
Held Defense owner, above Counter and nonsafety committed scripts; native Safety
can still preempt. A held role stops on release; toggle retains activation until
changed/reset. Global enabled, supported battle/controller scope and menu/handoff
must also permit it.

`auto_defense_lead_frames` is a fighter setting, schema fallback **4**. The supplied
Lucina file uses10; do not transfer that override to every fighter. Body estimates
use shared shield half-width/height/half-depth fallbacks6/18/6, scaled by native
size, not actual hurtbox edits:

```ini
[lucina.settings]
auto_defense_lead_frames = 4
```

This example resets the explicit lead to the schema value, not the supplied
Lucina value. [Fighter settings](FIGHTER_SETTINGS.md) explains accepted bounds,
reset versus explicit override and saving. Lead plus configured input-delay
allowance forms a bounded horizon capped at40; it does not learn latency.

## Native admission and contact evidence

`defense_response::native_allowed` requires valid ground/air, caller owner
admission, no capture/grab and no fighter-specific sequence interruption.
Ground Shield requires existing guard or native guard-on permission (fresh
permission is not read as readiness during stop). Air dodge requires native
escape-air permission. A transition term alone never means an incoming attack.

`src/live/counter.rs::observe_defense` scans bounded hostile fighter/projectile
snapshots and active hit geometry, comparing relative motion against the scaled
body. Ground/air situation masks exclude irrelevant hitboxes; known active
windows can bound forecasts, unknown attacks use their **already active**
geometry. Projectile ownership is not required for ordinary defense. Invalid
geometry is not guessed; a known contact can remain usable despite an unrelated
unreadable source. This reader reuses a solver, **not** counter fighter admission.

## Ground versus air lifecycle

While admitted with no contact, defense reserves the Guard input family rather
than generating an attack. An imminent ground contact with guard readiness
requests neutral-axis Shield and sequence input ownership. Releasing admission
or losing readiness/contact stops the proposal. This is not a fixed guaranteed
hold-through-every-hit routine or a shield-break escape.

In air, one pending neutral Guard packet is requested only with imminent contact,
readiness and nonstopped play. Mapper acknowledgment clears repeated requested
bits and waits input-free for native air-dodge startup. Delivery is not acceptance.
A missed startup releases between bounded retries; a successful observed air
dodge marks the opportunity used. It cannot spam another while native readiness
is unavailable. A later unavailable-to-ready transition or grounding resets the
cycle under the policy's rules. Duplicate packets are not extra native actions.

## Cancellation and beginner extension

Hold/toggle OFF, lost caller admission, invalid situation/identity, capture,
damage, native interruption or session/menu cancellation resets state. Air
pending requests are bounded by distinct native-derived tick accounting plus
configured delay; stop does not authorize a new air-dodge press. Shared watchdogs
still expire input and restore physical memory after mapping.

For Rust readers, `Context` is copied native permission, `Contact` is estimated
incoming evidence, and `Defense::sample` turns both into `ControllerOutput`.
`Option<Contact>` explicitly represents unknown/no contact. `mapped_air_dodge`
must be called only for the winning delivered packet. Players tune lead/body
estimates and activation; developers changing retry/admission need focused
no-contact, invalid/stop, ground/air, mapped-but-unaccepted, readiness-reset and
capture handback regressions. Never replace native permission checks with a
status write. Test no-contact and interrupted cases before evaluating success;
geometry and online queues can still defeat defense.
