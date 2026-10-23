# Ledge autopilot

Hold **L** for ledge autopilot. Tap **Y** for a neutral-B reverse attempt;
press **X** for a paired X+ZL short hop (both mapped to Jump). Hold **R3 alone**
for fast crouch/walk-cancel turbo teabag. **L+R3** shows/hides the menu; **Minus** toggles auto-mash. While visible, **Up/Down**
selects one of five rows (Neutral, Jump, Roll, Attack, Drop), and **Left/Right**
cycles its saved responses. While hidden, Left/Right slides, Down starts ledge
trump, and Up is normal. Directions held when closing must be released before
starting gameplay tools. L+R3 is reserved for menu toggling and does not teabag.
Auto-mash stays enabled in the supplied startup configuration.

Use A=Attack, B=Special, X=Jump, ZL=Jump, R=Grab, ZR=Shield and Tap Jump off.
Kirby attack-cancel bair also requires Tilt Attack C-stick. See [controls](CONTROLS.md).

These controls apply to a human-controlled fighter with a loaded ledge profile
on the selected local controller in valid battles. All non-DLC forms have
compiled profiles. Valid unconfigured kinds fall back to Kirby; invalid routing
stays inactive. See [fighters](FIGHTERS.md).

Kirby's **Drop → Short-hop dair** option reacts to a witnessed ledge release
with directional **X+A together**, down held to select dair, and drift toward
the ledge. It requests the action as soon as native grounded jump/actionability
allows, without a spacing, facing-turn, reach or vulnerability timing gate.
X releases during jump squat while down+A stays queued through takeoff, even
if the opponent disappears from tracking. Once the actual dair starts, A/down
release and full horizontal drift continues through startup. At dair animation
frame 18 ([first active frame](https://ultimateframedata.com/kirby)), control
hands back so you choose recovery. A different aerial, landing, interruption or
request timeout also ends this one-shot attempt. Release/rehold L to rearm.
Spacing is yours; it can drift offstage and does not guarantee a spike.

The default `ledge_distance_agnostic=true` policy leaves normal/jump/get-up-attack
spacing to you. Opponent acquisition and those responses ignore horizontal and
vertical range gates. Distance still ranks eligible ledge targets; it cannot
prevent acquiring the only eligible target. The assist selects a response from
the opponent's observed ledge option and schedules inputs using vulnerability,
recovery and native action readiness.

| Opponent option | Kirby default response |
| --- | --- |
| Hang | Keep manual capped walking. |
| Normal get-up | Short-hop nair, targeting first activity ten frames before known protection expiry with the supplied eight-frame setting plus the current two-frame lead. Neutral stick avoids facing-dependent aerials. |
| Jump get-up | Pair neutral X+A for short-hop nair; release X in squat while A remains buffered. A timing-gated request permits two frames earlier; immediate recognition stays immediate. |
| Roll | Down-smash approaches the common coverage point and pulses down+A during vulnerability. Selecting Grab uses the separate range/active-frame logic below. |
| Get-up attack | Hold shield; after a confirmed block, buffer grab regardless of distance or target protection metadata. If facing away, pair neutral jump+A directly out of shield; do not turn or grab. No whiff-only grab in this mode. |
| Generic ledge drop | Shield by default; alternatives are one native-confirmed down tilt then shield, or short-hop dair. There are no Off entries. Recognized ledge options take priority. |
| Confirmed capture | End autopilot immediately; physical controls choose pummels, throws and follow-ups. Release and hold L again to rearm. |

Manual horizontal input stays capped at `ledge_walk_strength` while waiting.
Only roll approaches automatically; normal-getup turning is a bounded facing
correction. No automatic return to a standing spot occurs. Actual punish recipes
briefly own their necessary inputs, then restore player spacing. R3 teabag
remains independent, with automatic punish inputs taking priority.

A confirmed get-up-attack grab returns all buttons and uncapped stick input on
the first capture observation, including catch-pull. No automatic pummel, throw,
stance return or walk cap remains active after that capture, even if L stays
held. Release L and hold it again to rearm ledge assistance. Old
`ledge_throw_outward` values are still parsed for config compatibility and do
not re-enable automatic throws.

Kirby also has an optional neutral-get-up **Down-B / Stone** recipe in the
[per-fighter saved lists](FIGHTER_CONFIGS.md#recipes-and-variants). Its grounded
startup is configured separately (11 by default), with zero early lead; nair
keeps its original lead of 8. This predicts an input time rather than proving
a frame-perfect online hit. Stone confirmation hands controls back so B remains
available to leave Stone normally. Release/rehold L to rearm.

## Configuration

The menu selects compiled fighter response functions. Global activation, shared
spacing and supported numeric timing adjustments remain in `config.txt`.
Config files cannot register functions or define their action sequence.


```text
ledge_trap = true
ledge_manual_spacing = true
ledge_distance_agnostic = true
ledge_walk_strength = 0.35
ledge_drop_defense = true
ledge_getup_attack = neutral_air
ledge_roll_dash_range = 10.0
ledge_roll_hit_window_frames = 5
ledge_roll_distance = 25.0
ledge_follow_range = 40.0
```

Distance-agnostic mode defaults to true even if an old config omits the key.
Setting it false restores the previous reach-gated normal/jump/grab policy.
Legacy tilt, grab, anti-air and acquisition distances remain parsed but do not
gate those responses in the default mode.

Roll down-smash coverage uses a fixed common spot: by default,
**25 world units inward from the remembered ledge**. `ledge_roll_distance` tunes
this coverage preset from 2–100; an old `auto` value also uses 25. This is a common
coverage area, not a measured endpoint or a guarantee that every character's
roll is caught. Correct spacing still needs device testing.

As soon as a roll is recognized, Kirby moves toward that point without waiting
for opponent velocity, root-travel forecasts or protection metadata. The ledge
anchor determines direction; if no anchor was observed, the first roll position
is the fallback origin. The destination is locked for that roll and bounded by
`ledge_follow_range`. Roll navigation runs even when legacy automatic positioning
is disabled. Floor probes still prevent movement across unsupported ground.

Native speed and ground braking guide early stick release; live extra delay is zero.
The final short approach slows down near the fixed point. Missing braking data
uses a short coasting estimate instead of blocking movement. Arrival, not an
opening timing window, ends navigation. Opponent movement cannot drag the spot
farther away. A missed punish window never becomes a chase.

At the spot, the policy releases horizontal input and sends down+A when the
native down-smash transition is ready and the hit window overlaps vulnerability
before roll recovery ends. Separate velocity, endpoint-reach and height vetoes
have been removed from this timing path. Rejected down+A inputs get release gaps
and retries during the window; only one actual down-smash animation is attempted.

Roll grabs use the fighter's own configured grab reach, startup and active
length. Their initial common endpoint is an approach hint. Kirby/Yoshi extend
that hint inward when observed roll travel projects beyond it, with a bounded
six-frame navigation forecast. They keep walking at 0.65 stick into a usable area
instead of repeatedly braking outside its edge. Braking still prevents crossing
the opponent; there is no chase back toward the ledge. Other fighters retain their
existing navigation style.

The approach area extends up to 60% of configured standing reach. The request
itself requires predicted opponent reach, facing, height AND vulnerability on the
same active grab frame, using the full configured standing range. An already
useful position can request without reaching the common point. These are root
position/velocity estimates, not an animated grab/hurtbox model. Diagnostic gate
`roll_wait_actual_grab_reach` means timing is available but forecast geometry fails.

The whole configured grab-active interval is checked against a target three
frames after protection expiry (midpoint for narrow vulnerable windows, two
frames if the recovery deadline is unknown). Unknown protection still waits.
Only activity before the known recovery deadline counts. Native can_grab,
movement release, bounded delivery and capture handback remain in effect.
There is no learned input-delay compensation. The area and roll endpoint remain
estimates; they do not extend native hitboxes or guarantee a catch.

`ledge_roll_hit_window_frames` defaults to **5**, planning contact during attack
frames **7–11**. Set 1–13 to extend or shorten that window through frame 19.
Live extra input-delay compensation is zero. Unknown protection still waits for observed
vulnerability; these settings do not change actual hitboxes or guarantee
opponent-relative frame-perfect timing online.

## Timing and limits

[Kirby's frame data](https://ultimateframedata.com/kirby) gives nair startup 8
and active frames 8–32, and down smash startup 7 with front/back active phases
through frame 19.
Nair is one lingering hit with weakening phases. A proposed contact must overlap
known vulnerability and precede any known action deadline. Grounded recovery uses
the earlier valid cancel/animation-end frame. Missing protection timing waits
for observed vulnerability; it does not assume protection ended.

Native transitions still control when Kirby can actually jump, attack or grab.
Normal/jump timing ignores collision geometry by request: an attack can come out
from across the stage and miss. The front-facing block-triggered grab ignores
reach. The rear-side response preserves shield through shieldstun, jumps directly
from guard with **neutral jump+A together**, then releases X while retaining A
through jump squat, hitstop and takeoff. Rejected ground inputs retry the pair.

Ledge-jump cover now sends **neutral X+A together** as the built-in
[short-hop attack input](https://www.smashbros.com/en_US/howtoplay/technique.html).
This gives the game's normal input pipeline the jump and aerial together, rather
than waiting to observe a delayed jump before transmitting a separate attack.
X is released in native jump squat while A remains queued. A rejected grounded
pair gets a neutral release sample and another paired request. Sending the pair
is an input request, not proof that either action was accepted.

All committed nair hops now finish using Kirby's own native action state before
opponent tracking or option selection is evaluated. Target loss, a regrab,
changed recovery forecasts or the opponent landing cannot cancel the aerial
partway through a hop. There is no aerial-readiness-term or six-attempt gate once
a hop is committed. Neutral A stays held until the native aerial starts. A queued
A hold remains owned through hitstop, so physical stick/C-stick or shield inputs
cannot replace it. Damage, landing, L release and session changes end the
sequence. One actual hop/nair is attempted per visit/block; an accepted aerial
alone does not count as a confirmed hit.

Normal get-up now also sends neutral jump+A together, using the same built-in
short-hop attack recipe as ledge-jump cover. Short-hop jump and attack share one input request; live extra input-delay
compensation is zero. Rejected ground requests retry the pair with release gaps. X is
released in jump squat while A stays buffered, including frozen squat samples.
Both options share the committed follow-through through target loss and takeoff.

These changes require a newly built and installed plugin. Existing NROs do not
change when source files are edited. Native observations and mapped-input
confirmation improve reliability, but this integration has no authoritative
online queue depth or future execution-frame API. It cannot guarantee an exact
opponent-relative attack frame in every Arena connection. Offline host checks
cannot establish online behavior; device testing with the rebuilt diagnostic
plugin is required.

Learned mapped-input delay compensation has been removed. There are no per-action
sample histories, medians, runtime delay sampling, or paired jump/attack estimates.
Live observations supply zero extra delay to ledge responses, slides, shortcuts,
SDI, and tech prediction. Configured move frames and response timing adjustments
remain in use, as do native readiness, hitstop handling, and acceptance checks.
The scalar `input_delay` field remains available for explicit core callers and
fixtures; the Switch adapter no longer learns or fills it with measured delay.
This removal does not change the game's own online input buffer or a separate
latency mod. Online behavior still needs device validation.

Damage, target loss, tracking timeout, release of L, mash and tech retain their
cancellation/priority rules. Hitstop preserves shield. Exact native attack
status/motion plus a registered hit/capture confirm success; a button request
alone does not. Diagnostics retain their throttled asynchronous output.

These are source changes. Install a newly built NRO and the intended config, then
restart Smash to use them.

Normal get-up now defaults to `neutral_air` (`nair` is also accepted). The
bundled config selects it; an existing config explicitly selecting a legacy tilt
continues to select that tilt until updated. Kirby hop scheduling includes three jump-squat frames and eight nair startup
frames, with zero extra live transport-delay compensation. Its Neutral nair
adds two frames to the configured lead: the supplied eight becomes ten. It
releases X while keeping neutral A buffered. With uninterrupted native activity,
that plans the first hitbox ten frames before protection expiry and the last
fourteen frames after expiry (nair frames 8–32). An expiry at get-up frame
32–34 therefore targets first activity around frame 22–24. Yoshi uses a separate
17-frame Neutral lead and immediate Jump nair. Both use a quarter-stick nudge
only before native nair animation frame 4; see [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md).
This is the planned animation window; spacing, landing, late recognition and
unmeasured online delay can reduce actual coverage.
Late recognition/action readiness can reduce that lead. Unknown protection waits
for observed vulnerability. An expired recovery estimate does not veto the
normal-getup hop, and previous-status evidence can recover a missed get-up
transition. The active hitbox still needs correct player spacing.
