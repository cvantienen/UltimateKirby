# Out of shield: shield grab and configured backup

The current simplified **Out of shield / Automatic grabs** group is shield-only.
It considers a legal front grab first, then this fighter's explicitly saved
backup. It does **not** start an unshielded dash-grab chase, raise shield from
neutral, or automatically borrow Bowser's Up-B. A nearby vulnerable enemy can
qualify without previously hitting your shield: this differs from retained
legacy `shield_hit` rules.

## Enable, configure, save

Use the current mode row in [Controls](CONTROLS.md). In Training its rule list
contains **Automatic grab (shielded)** and **Shield grab backup**. The
`out_of_shield_default` fighter preference controls startup inheritance; the
ordinary row changes the current selection. See [Fighter settings](FIGHTER_SETTINGS.md)
for save/reset behavior and the difference between an absent override and false.
The portable menu starts this switch ON; startup/saved settings can change it.
No supplied catalog entries or values are assumed here.

A simplified rule stores just maximum horizontal separation and height:

```ini
[bowser.roll.practice_shield_grab]
name = Practice shield grab
start_delay = 0
timeout = 120
reaction = auto_shield_grab:20:10
actions = input:r:0:0:4

[bowser.roll.practice_shield_backup]
name = Practice shield backup
start_delay = 0
timeout = 120
reaction = auto_shield_backup:12:15
actions = input:b:0:1:4
```

These are **editable examples**, not Bowser's saved defaults or verified hitbox
reach. The first requests mapped Grab with neutral stick; the second requests
upward Special. Literal physical names must agree with the selected game layout.
Use [Sequence files](SEQUENCE_FILES.md) for grammar, selection and saving. Only
the chosen configured response reaches the mapper. Editing backup to an ordinary
non-Up-B recipe changes its action; native shield-control readiness still applies
and a needed shield-drop handoff remains. Missing backup means no backup attack.

## Admission: what must be true now?

Your fighter must be in native grounded shield startup/hold, not shieldstun,
hitstop, capture or unrelated recovery. Native catch readiness (or the held-shield
path) must permit grab. Physical shield intent alone does not authorize a new
unshielded grab while native state has not caught up.

The front grab target must be available, currently vulnerable, ordinarily
grabbable, inside saved X/Y bounds and in front of native facing. A moving target
additionally needs a valid grab-startup projection that stays within front reach.
Rear, invulnerable, captured or downed/tripped bodies cannot receive this ordinary
grab. The opponent need not be shielding.

The backup has independent X/Y limits and can select either side, including a
vulnerable downed body that is not grabbable. Direct upward Special requires
native Up-B readiness. Other configured backups use legal shield-control
readiness, not placeholder Up-B permission. Root distances are estimates;
neither range nor vulnerability guarantees the move will connect.

Active ledge hold/toggle disables new automatic-group acquisition. Safety and
committed owners retain priority, with explicit Held Defense/Counter preemption.
A new eligible immediate Auto-react Up-B can replace an **undelivered** grab in
the same observation; after delivery a receipt protects Grab from competing
Special/throw-producing inputs. Physical Grab also retains manual priority
before native catch becomes visible.

## Delivery is not move confirmation

A grab emits neutral-axis Grab for a bounded attempt: at most four distinct
delivered packets or four native-progress frames. Duplicates do not spend the
mapper budget. Losing shield/readiness/range/front-side before delivery cancels
without turning it into an approach. After delivery, an input-free receipt can
remain for up to 24 distinct mapper updates/native-progress frames while catch
is pending. Native catch/pull/hold/pummel/throw immediately releases all tapes,
leaving pummels and throws manual.

Direct Up-B sends its configured bounded initial pulse. A mapped Special+Up
packet can arm an input-free receipt; it is not proof the move started. The
receipt can survive short readiness/hitstop changes without sending more B,
then expires or confirms native Up-B. Confirmation resumes the next authored
step with its full duration. No fighter-specific spin/drift tail is appended.
Unconfirmed takeoff, wrong native move, capture/damage, lost target, mode/scope
loss and receipt expiry cancel. Failed mapping cannot arm a success receipt.

The automatic shield path has a separate **six native-progress-frame guard tail**
after an observed threat/shield hit. It requests neutral Shield only while shield
admission remains valid and no delivered response/receipt owns the path. Stop
pauses its clock. This differs from the ledge safeguard's tuning and literal
response waits; it does not make a fresh attack legal in shieldstun.

## Legacy compatibility, not current default behavior

Detailed `shield_hit` / `shield_hit_vulnerable` recipes and their portable
confirmed-contact helper remain readable for compatibility. They mean a reported
attacker hit **your** shield and have detailed conditions. Do not confuse them
with `auto_shield_grab` / `auto_shield_backup`, which can act without a prior
block. Consult [Reaction rules](REACTION_RULES.md) for retained control support.

The removed `auto_grab` event and old unshielded stationary/full-run chase are
not current simplified slots. `auto_grab_close_range` can still supply fallback
shield-grab range; the legacy approach argument is ignored by the current
shield-only policy. Editing an old chase range does not restore chase.

## Rust beginner walkthrough and safe experiments

`AutomaticConfig` in `policies/out_of_shield/automatic.rs` borrows optional
sequences. `simple.rs` acquires/revalidates the front target and tracks mapped
Grab packets. `shield_up_b.rs` handles bounded direct backup delivery;
`defense_response.rs` plays edited responses and resumes late native receipts.
`Option<&Sequence>` means a borrowed recipe or no recipe; never replace absence
with another fighter's move. A **receipt** remembers delivery without owning
buttons, whereas a **proposal** still asks for input.

Test in Training with only this mode enabled: front vulnerable target, rear
vulnerable target, leaving range, shieldstun, then capture handback. Change one
range or hold at a time. Adding an admission path in Rust requires native scope,
readiness and receipt/cancellation regressions, not merely a parser key. Inputs
obey native restrictions; no physics/status writes, automatic shield raising,
guaranteed punish or Switch verification is implied.
