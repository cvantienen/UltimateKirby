# Configured out-of-shield responses

## Shield grab and Up-B fallback

Ordinary shield grabs and grabs against shielding opponents remain generic with
fighter-specific ranges. Every non-grab follow-up requires that fighter’s own
configured recipe. Missing `auto_shield_backup` means no backup attack. The
supplied split catalog preserves the existing explicit backup entries for all
77 fighters. Those saved entries are fighter configuration, not an implicit
Bowser fallback; remove or edit a fighter's backup to change that behavior.


With **Automatic grabs / Out of shield** enabled, native shielding uses one
immediate choice: grab a vulnerable, grabbable opponent in front when legal;
otherwise request Up+B+Shield for a vulnerable opponent close to either side.
The fallback does not require a dodge or a prior shield hit. It also permits a
vulnerable downed opponent that cannot be grabbed; captures, ledge visits and
noncombat targets remain excluded. A close attacker
can be punished on the first shield-ready observation after a hit, provided it
is still vulnerable and native Up-B is available. No facing step, shield-drop
wait or range-wait is added.

An explicitly configured ordinary backup (for example, up-smash or a jump-based
recipe) uses legal shield-control readiness rather than requiring Up-B to be
available. Only the configured recipe reaches the mapper; the fallback's internal
target selection never emits a placeholder Up-B. A custom movement response gets
ground-support observations even with Auto react switched off. Ordinary actions
that require shield drop retain the sequence engine's native handoff behavior;
their authored timing and native move restrictions still apply.

The saved sequences are **Auto react → Automatic grab (shielded)**
(`auto_shield_grab:25:10`) and **Shield grab backup**
(`auto_shield_backup:25:15`) in the current Bowser file. Each has its own X/Y
limits and editable actions. The **Automatic grabs** switch still controls both;
Auto react's switch continues to control only enemy roll/spot/air responses.
A legal front grab wins over a rear backup. Active ledge mode disables the entire
automatic group and its searches. Safety, committed-owner and roll priority still apply.

The unaccepted B request ends after four delivered mapper attempts or four
native-progress frames; duplicate mapper acknowledgments do not extend it.
It does not press a new Up-B through shieldstun or hitstop. For Bowser, an actual
final mapped **Special + Up** packet also arms a passive native-receipt record.
If shield hitstop/readiness changes or the four-frame pulse ends before native
Up-B is visible, it sends no further B, owns no buttons/sticks and retains only
punish priority and the locked target for at most 24 distinct mapper updates
(including delivery). Native confirmation begins the configured drift without
replaying B; the whole 69-frame drift budget remains available. Failed mapping
cannot arm this record. Ledge L, capture/damage, target loss, an unconfirmed
airborne transition, another native move or receipt expiry clears it.

For Bowser only, confirmed native Up-B startup replaces B/Shield with up to
**69 native-progress frames of full horizontal drift toward the tracked opponent**.
This includes air spinning after a grounded start, follows target crossings and
does not recheck the initial proximity or vulnerability after a hit. Native spin
end, ledge-trap L, safety/scope changes or target loss release tracking early.
Physical gameplay buttons and both sticks do not cancel committed Bowser Up-B
startup or tracking; mode switches apply after that punish ends.
No fresh airborne Up-B is started. Other fighters retain release on native startup.

## Current automatic grabs

When you are **not shielding**, **Automatic grabs** requires an opponent visibly
shielding (shield startup, hold or shield damage), currently vulnerable and in
front. When you **are shielding**, the enemy does not need to shield: any
currently vulnerable, grabbable front target inside `auto_shield_grab` X/Y can
receive the shielded recipe on the first native-ready observation.
Invulnerable, captured, knocked-down, ledge and noncombat targets are excluded.
Your fighter must be grounded and able to grab. When you hold shield, it sends a
direct grab with neutral stick, only within its saved `auto_shield_grab` X/Y limits;
it never drops shield to dash. Bowser's current shielded range is **25 X / 10 Y**.
When idle without shield, it sends one full-strength opponent-directed input
pulse, followed by R for at most four delivered input frames. Ground movement
and normal landing send R immediately instead. Bowser's
**unshielded grab sequence** is `auto_grab:35:24`. Both grab sequences live
in the Auto react list. Their `actions = input:r:0:0:4` entries preserve the
existing native delivery path; edited actions replace its final input after the
same acquisition/movement checks. No preliminary Grab is emitted before the
configured action. Ordinary shield attacks still require native shield-drop
recovery when the requested action cannot start directly from shield.

Walking, running, turning, crouching and ordinary landing may start a grab as
soon as native standing/dash-catch readiness permits. These moving/landing
attempts send R in the first observation, without a preliminary neutral or
approach packet. The final mapper retains your physical horizontal stick at its
current strength and neutralizes vertical input. There is no wait to slow down.
No target/readiness means no input ownership, so movement stays manual.

Existing stationary attempts retain their one delivered approach pulse, with
floor support checked before that additional movement. Bowser's idle far-range
attempt (beyond the shielded/close range) continues full forward input until
native dash/run is actually observed, then sends Grab with full forward input.
A mapper acknowledgment or residual shield-drop velocity is not dash evidence.
No standing Grab is sent while waiting for that native transition. Four delivered
approach attempts bound failure; already-moving, close-range and shield grabs
retain their existing paths. Already-moving grabs add
no approach movement and need no extra floor ray. A/B/manual grab, C-stick or
strong vertical jump/drop input keeps manual priority. Held/native shield uses
the stricter tilt gate and always emits neutral-axis grabs; full roll input is
not intercepted. A new shield press cancels an unshielded attempt, never turning
it into a dash from shield. New air grabs are excluded; a still-valid target is
checked fresh on the first native-ready landing observation.

Direct shield grabs also check projected reach at the controlled fighter's shield
grab startup using current relative speed. Dropping shield, leaving range,
becoming invulnerable/ungrabbable or crossing behind you ends further grab input. Neither path turns around
to grab a target behind you. A native grab confirms
acceptance and releases input ownership immediately. Attempts stop sending R
after four distinct delivered grab packets or four native-progress frames, with
no extra neutral ownership frame. Once R has been delivered, a bounded receipt
record prevents Up-B replacing it while it may still be queued. This record owns
no movement or buttons while settling and ends on native grab startup, interruption
or 24 mapper updates/native-progress frames. Duplicate ticks do not spend the
mapper budget. Native grab/catch/hold/pummel/throw states cancel all punish and
ledge/button tapes, leaving throw selection manual.

An eligible Auto react Up-B is selected before a **new** grab, including rolls in
front, overlapping or behind your shield. It can also replace an unmapped grab
or approach immediately **before any R packet is delivered**, even if that
grab's target remains valid or another opponent rolls. The pending grab and its cached output are cleared before Up-B
is emitted in that same observation. No neutral handoff wait is added.
Readiness, event-consumption, range and height rules still apply; unavailable or
already-used reactions do not steal the grab. A native grab already in progress
is not force-cancelled. One OOS owner selects grab or shield Up-B; a committed
fallback cannot turn into a new grab midway through its Special pulse. There is
no queued approach/grab plan behind another response.

Holding the assigned ledge-assist button or enabling its toggle disables all
seven Auto react options and grab avoidance, including opponent acquisition.
Both must be OFF before automatic checks resume. Explicit recipe previews,
manual escape and other committed owners retain their gates. A physical Grab
press takes priority before native catch is visible, in either controller layout.
An already delivered automatic Grab retains only a bounded, input-free receipt;
competing scripts cannot add Up/B while that receipt is unresolved.

New reactions are observed only during controllable grounded movement, ordinary
landing or ready shield. Attack recovery, dodge/roll, shieldstun, tumble and
capture do not admit fresh checks. A confirmed response may follow its locked
opponent through its own configured action; confirmed Bowser Up-B keeps its air
tracking. Capture links exclude both actors from ordinary automatic acquisition.

A nonshielding aerial can qualify for your shield grab when the front-side,
grabbability, X/Y and startup-reach checks pass. If no legal grab wins, a
vulnerable opponent inside the separate backup X/Y limits can receive immediate
shield Up-B (or the edited backup recipe) when native readiness permits. This
includes rear targets, downed/tripped bodies and opponents outside grab limits
but inside backup limits. Otherwise no automatic input replaces your shield.
A failed direct Grab/Up-B request preserves a continuously held physical shield;
only guard actually suppressed by a committed action needs release-to-rearm.
Unconfirmed Up-B cancels before any drift or neutral release, including zero
horizontal drift. The mode does not raise shield or modify moves/physics.
The remainder below describes legacy recipe modes.

**Out of shield** is available for every fighter on the Modes page and starts ON
with the supplied defaults. It responds to an actual native shield hit, not merely
an opponent being nearby. It does not raise shield automatically. Manual shielding
and the separate ledge approach defense still supply the block; reserved ledge
responses retain priority.

## Configure in Training

Open **Assist modes → Out of shield → L3/R3** in Training. Add/name a rule
in its separate list, then configure maximum/minimum X distance,
height, side, priority and the action sequence. It does not need an initial
"if shielding" action or selection as the active ledge/button recipe. Defensive
reactions use their own ON/OFF mode; they do not use Roll rules or button toggles.
Assist rules do not change the selected manual ledge/button option; their priority determines which matching response runs.

The supplied Bowser example is:

```ini
[bowser.attack.auto_oos_up_b]
name = AUTO OOS Up-B
start_delay = 0
timeout = 120
spatial_timeout = 60
default = false
reaction = shield_hit:grounded:0:12:18:any:1
actions = up_b:4
```

This uses 12 units of horizontal separation and 18 units vertically, on either
side. These are initial estimates to tune in Training, not measured Up-B hitbox
coverage. Start delay begins when native shieldstun has ended and the response
conditions match. Action delays and holds use the normal recipe timeline.

Only the confirmed attacker is followed. After native hitstop/shieldstun ends,
the first matching recipe runs once. If no recipe matches, automated ownership
is released immediately; it does not hold shield waiting for the enemy to move
into range. A physically held shield remains your input. Unknown/projectile
attacker identities do not cause a nearby unrelated fighter to be punished.

This is configured input playback: it does not infer whether the selected move
is a guaranteed punish, wait for lingering hitboxes to disappear, or invent
startup/recovery timings. Tune the distance and delay for the intended move.
If the game cannot accept an input, the recipe cannot force the move to occur.

The supplied Yoshi examples prefer a front grab within 23 X / 10 Y units, then
short-hop Nair within 12 X / 18 Y units. They replace the former hard-coded move
selection; all response inputs and delays now live in `sequences.txt`.

## Startup defaults and cost

In Training, open **Fighter settings** and save **Out of shield at startup**.
This writes `out_of_shield_default = true` or `false` in `[fighter.settings]`.
It affects fresh match startup; the ordinary Modes row controls the current match.

Without an enabled mode and a configured `shield_hit` recipe, the OOS observer
is skipped. Otherwise it follows only the reported/pending attacker, sharing
native facts already read in the observation. Recipes resolve on edits; no file
reads, new hooks, network work or permanent workers run during response playback.
Switch verification remains necessary for native input acceptance and online delay.

A renewed confirmed shield hit interrupts an older OOS response, including a
held Up-B. The new contact supplies the attacker and readiness again; the old
sequence cannot keep advancing or buffer through renewed shieldstun. OOS and
grab-dodge follow-ups track their own attacker snapshots, sharing the native read
only when both IDs match. This adds no roster scan; distinct simultaneous targets
can require one additional bounded identity follow-up read.

`shield_hit_vulnerable` is an optional event variant in the same OOS rule editor.
It adds an enemy-native-hit-status condition to selection without delaying or
changing older `shield_hit` recipes. The current Bowser profile uses just one
front grounded grab rule within 22 X / 12 Y units, with no OOS Up-B fallback.
The event still waits for confirmed contact and your own shield recovery.

## Configuration compatibility and safe playback

`auto_grab_close_range` / `auto_grab_dash_range` remain accepted for old files.
Explicit `auto_shield_grab` / `auto_grab` recipes override them; their duplicate
Fighter settings sliders are hidden while explicit recipes exist. The two old
keys were relocated, not retuned, in the supplied master.

The default grab and shield-backup recipes use their original native delivery
paths. Bowser's backup input duration, drift duration/strength/direction and
response timeout can be edited without adding a wait for startup. Other edited
recipes use the existing bounded input timeline and native interruption rules.
Queued Grab inputs still reserve against competing automation until receipt or
expiry. A configured step after an unresolved Grab cannot inject B/Up or select
a throw; native grab/catch/hold/pummel/throw always hands control back. Custom
inputs cannot bypass native recovery or force a move to start.
