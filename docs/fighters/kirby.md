# Kirby

Standing C-stick correction: Dtilt face reads the physical right-stick axes in
the same mapper update, including the first Down flick before digital aliases
appear. With a neutral left stick it prepares the down-turn angle on both sticks;
no crouch prerequisite or waiting frame is added. Side/up C-stick inputs, aerials
and attack button edges remain unaffected. Native input acceptance still decides
whether the requested turn precedes an attack. C-stick must use Tilt Attack.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **6** (`kirby`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Rapid Jab; Dsmash; Stone; Inhale; Bair; Cross-up grab; NAIR LATE; Immediate Nair; Immediate Nair Max Drift; Dash Attack; [TEST] Face ledge > walk 5 > grab; [TEST] Face foe > run 10 > grab; [TEST] Full-hop Bair + drift; [TEST] Short-hop Nair + delay/drift; [TEST] Turnaround short-hop Bair; [TEST] Grab (verify profile range); [TEST] Grab (ignore range); [TEST] Grab (verify 20 units); [TEST] Grounded down smash; [TEST] Grounded down special |
| Jump | **Nair**; Inhale; MAX NAIR; HALF NAIR |
| Roll | **Dsmash**; Grab; Dash Attack |
| Attack | **Grab/rear Nair**; Grab/rear Guard |
| Drop | **Shield**; Dtilt > Shield; SH Dair; jump UP B; UP B |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_grab` | — | Automatic grab (unshielded) | `kirby.roll.auto_grab` |
| `auto_shield_backup` | — | Shield grab backup | `kirby.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `kirby.roll.auto_shield_grab` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 2 start; active 2; 14 total; -- landing |
| Neutral Air | 8 start; active 8–9 / 10–13 / 14–18 / 19–32; 52 total; 6 landing |
| Forward Air | 10/17/25 start; active 10–11 / 17–18 / 25–27; 47 total; 8 landing |
| Back Air | 6 start; active 6–8 / 9–12; 40 total; 10 landing |
| Up Air | 8 start; active 8–13; 35 total; 7 landing |
| Down Air | 18/21/24/27/30/34 start; active 18–19 / 21–22 / 24–25 / 27–28 / 30–31 / 34; 54 total; 16 landing |
| Forward Tilt | 5 start; active 5–8; 23 total; -- landing |
| Up Tilt | 4 start; active 4–5 / 6–10; 20 total; -- landing |
| Down Tilt | 4 start; active 4–6; 20 total; -- landing |
| Dash Attack | 9 start; active 9–17 / 18–26 / 27–34; 51 total; -- landing |
| Forward Smash | 13 start; active 13–15 / 16–19; 47 total; -- landing |
| Up Smash | 12 start; active 12–14 / 15–16 / 17; 45 total; -- landing |
| Down Smash | 7 start; active 7–11 / 12–19; 50 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 9–10 |
| Pivot grab | frames 10–11 |
| Standing reach reference | 12.5 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 79 |
| Walk speed | 0.977 |
| Initial dash | 1.9 |
| Run speed | 1.727 |
| Air speed | 0.84 |
| Gravity | 0.064 |
| Fall speed | 1.23 |
| Fast fall | 1.968 |
| Short-hop height | 12.24 |
| Full-hop height | 25.37 |

## Execution and calibration

Option definitions and input delays come from `fighters/kirby.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 0 | `facing_input:x:0:0:1, wait:10, drift:opponent:0.99:5, wait:4, facing_input:a:0:0:4` |
| Neutral / `down_tilt` | 1 | `walk_to_opponent:0.8:1:10, facing_input:a:0:0:50` |
| Neutral / `down_smash` | 15 | `wait:1, face:opponent, facing_input:a:-0.02:-1:1` |
| Neutral / `stone` | 15 | `face:opponent, facing_input:b:0:-1:1` |
| Neutral / `inhale` | 8 | `face:opponent, facing_input:b:0:0:1` |
| Neutral / `bair` | 0 | `wait:1, face:away_from_opponent, wait:2, facing_input:a+x:-1:0:4` |
| Neutral / `crossup_grab` | 0 | `face:opponent, wait:1, facing_input:zr:1:0:4, wait:20, check:enemy_in_front:0, wait:1, facing_input:r:0:0:4` |
| Neutral / `nair_wait` | 0 | `Short Hop, wait:5, Neutral Air, Drift:Opponent:25%:20` |
| Neutral / `nair_now` | 0 | `Short Hop, Nair, Drift:Opponent:25%:20` |
| Neutral / `nair_now_max_drift` | 0 | `Short Hop, Nair, Drift:Opponent:100%:20` |
| Neutral / `dash_attack` | 0 | `face:opponent, dash_attack` |
| Neutral / `test_walk_ledge_grab` | 0 | `Face Ledge, Walk:Ledge:5 units:45%, Grab No Range Check` |
| Neutral / `test_run_foe_grab` | 0 | `Face Opponent, Run:Opponent:10 units, Grab Range Verified` |
| Neutral / `test_full_hop_bair` | 0 | `full_hop:4, Bair, Drift:Opponent:50%:12` |
| Neutral / `test_short_hop_nair` | 0 | `Short Hop, Wait 3 frames, Nair, Drift:Opponent:25%:8` |
| Neutral / `test_turnaround_bair` | 0 | `Short Hop, Turn Around Bair` |
| Neutral / `test_grab_range` | 0 | `Grab Range Verified` |
| Neutral / `test_grab_ignore_range` | 0 | `Grab No Range Check` |
| Neutral / `test_grab_20_range` | 0 | `Grab:20 units` |
| Neutral / `test_down_smash` | 0 | `Down Smash` |
| Neutral / `test_down_special` | 0 | `Down Special` |
| Jump / `nair` | 0 | `facing_input:x+zl:0:0:1, facing_input:a:0:0:1, drift:opponent:1:4` |
| Jump / `bair` | 0 | `face:opponent, facing_input:b:0:0:4` |
| Jump / `nair_max_drift` | 0 | `Short Hop, Nair, Drift:Opponent:100%:10` |
| Jump / `nair_half_drift` | 0 | `Short Hop, Nair, Drift:Opponent:50%:10` |
| Roll / `down_smash` | 0 | `walk:away_from_opponent:14:0.9, wait:5, facing_input:a:0:-1:1` |
| Roll / `grab` | 10 | `walk:away_from_opponent:12:0.8, range:fighter_range, wait:1, input:r:0:0:4` |
| Roll / `dash_attack` | 20 | `face:opponent, wait:1, facing_input:none:1:0:4, facing_input:a:0.79:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, grab` |
| Attack / `shield_rear` | 0 | `shield_until_block:90, shield:255` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |
| Drop / `jump_up_b` | 0 | `Short Hop, wait:2, up_special` |
| Drop / `up_b` | 0 | `up_special` |

See [this fighter's detailed execution notes](../KIRBY_YOSHI_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


## Kirby-specific timing and inputs

**Roll → Grab** immediately runs to a fixed **20-unit inset from the witnessed
ledge**, on either side and every stage. Native braking slows the approach, with
a small walking correction near the spot. There is no opponent-based destination
extension or grab reach subtraction. Kirby stops within one unit, faces center
stage, then waits for a valid timed grab; it does not grab before reaching the
spot. Full-stick dash inputs close distance, with neutral braking and 0.3-stick
fine correction near arrival. Native grab readiness and predicted contact still
gate delivery; reaching the spot alone cannot force a catch. Yoshi instead uses
the shared tracked walking approach again. Host
regressions pass; spacing and actual grab success still need Switch testing.
The fixed-inset implementation and its timing profile have been retired.

**Neutral → Down smash** replaces the former Forward tilt menu choice. On
recognition it walks toward the remembered ledge at 0.65 stick during the
existing wait, aiming six units inside with native braking/floor checks. It
stops walking at the attack boundary even if still far away. The timing target
now starts two frames earlier than the old midpoint alignment: protection expiry
at animation frame **15**, within
of Kirby's active frames **7–19** (clean 7–11, late 12–19; see the canonical
[Fighter Data catalog](../FIGHTER_DATA.md)). Preparation begins
16 frames before predicted vulnerability, including one neutral smash-primer
update; down+A follows with 15 frames remaining. Nominally the hitbox begins
eight frames before protection ends and remains active four frames after it.
Native startup releases A to
avoid charging. Arrival/range is never required before the attack; native action
readiness still applies. The former compiled midpoint-alignment constant has
been retired with the fighter timing profiles;
positioning and actual online delivery still require calibration on Switch.

**Neutral → Nair** now has Kirby-only grounded preparation: once neutral get-up
is recognized, use the existing wait time to walk toward the remembered ledge
anchor. Input strength is `NEUTRAL_NAIR_WALK_STICK = 0.65`, the existing safe
fast-walk value used for Kirby's roll grab. Aim to stop
`NEUTRAL_NAIR_EDGE_INSET = 6.0` world units inside the ledge, braking earlier for
native momentum. Ground-support checks can stop the walk farther back. Unknown
ledge anchors/timing retain the old stationary behavior. It never chases past
the destination, runs off the edge deliberately, or waits for arrival before
launching nair. Starting too far away can still leave Kirby short of the target.

The former fighter timing constants and compiled sequence handlers have been
retired. Current responses and input timing are authored in the fighter's
sequence file; reference facts remain in the canonical
[Fighter Data catalog](../FIGHTER_DATA.md).
Jump nair also opts into the shared movement helper while its existing timing
checks are waiting. Immediate jump cues still launch immediately, leaving no
grounded walking time. Horizontal digital flick aliases are suppressed during
these walk preparations; other response movement remains independently configured. Shared movement lives in `fighters/common.rs`.
See the
[fighter-specific sequence example](../FIGHTER_EXECUTION.md#initial-and-per-action-delays).

Neutral nair keeps its quarter-stick nudge below animation frame 4. Jump nair
now uses **full 1.0 forward stick** after exact native nair acceptance through
its configured active window (8–32), returning horizontal stick to neutral at
frame 33 or earlier on landing/interruption. Launch and every delayed/retried
Attack request keep the stick neutral; drift starts only after native
`attack_air_n` acceptance, so forward drift cannot select Fair. Neutral nair adds two frames to its configured
lead (8 becomes 10); Jump nair keeps its existing launch timing and immediate cues.

Jump bair uses 0.35 drift. When facing the opponent, it sends a 0.6-stick
grounded turn away without C-stick or A, confirms native facing, then requests
X+ZL short hop, releases both in squat, then selects backward+A after takeoff. Already back-facing
Kirby skips the turn. This avoids the failed grounded-tilt cancel; it does not
assume every turn is delivered in exactly one frame. Both X/ZL must map to Jump.
Neutral bair now turns away immediately on recognition with 0.6 stick and
confirms native facing before its timed short-hop launch. The turn is explicitly
marked as preparation so manual spacing does not discard it. It sends neutral X+ZL, releases both in squat and selects backward+A after takeoff; no grounded
attack or C-stick tilt is used. Zero drift and three-frame early lead remain.

Neutral Inhale turns toward the remembered ledge as soon as neutral get-up is
recognized, using a 0.6-stick turn without B. After native facing confirms the
direction, the existing timed neutral-B request runs with neutral stick. Timing
and range checks are otherwise unchanged; no automatic walk/chase is added.

Roll Grab uses the fixed approach above; it does not extend the destination from
observed roll travel. Capture returns full control. Integrated held-L recovery cover, tilt-trip follow-up
and fair-first-hit landing follow-up are separate tools; see [controls](../CONTROLS.md).
These current movement edits still need device testing.

## Separate onstage Roll read

The additional **Roll read** row is independent of ledge Roll get-up. Choices
are **Off** / **Down smash** (default) / **Grab**; once enabled it watches ordinary ground
rolls without holding L. Down smash now accepts nearby rolls without a crossing
requirement and takes over immediately when a valid timed roll is recognized.
It may walk at 0.65 stick toward the short projected roll position while waiting,
using native walking/braking data and floor support. The attack window takes
priority over range or arrival: stop, prime, then Down+A. It uses the full
configured 7–19 active window, including the final two frames previously reserved.
`ROLL_READ_DOWN_SMASH` uses estimated reach 14, height tolerance 6 and observation
distance 64 world units; reach is a reference, not an attack gate in this mode.
It can therefore attempt a timed smash while still too far away to connect.
No other fighter inherits this mode until explicitly registered. See
[Roll read](../ROLL_READ.md) for priorities, testing limits and Training controls.
`ROLL_READ_GRAB` uses standing grab frames 6–7 and an estimated 12.5-unit root
reach. It can turn in place toward the projected contact side, but never chases;
native grab acceptance returns control without a pummel or throw. Ledge Roll
grab is a separate implementation using the fixed-inset experiment above.

## Manual X/Y controls

X/Y/D-pad input sequences belong only to this fighter and are edited through the in-game
[sequence editor](../SEQUENCE_FILES.md#fighter-specific-xy-recipes). Normal button
keeps X short hop or ordinary Y input. The old native Y Finisher remains removed.
Ledge recipes and other fighter policies retain their separate controls. Modes
are available only on the ON/OFF page; no button can Hold/Toggle a mode.

## Reference facts

Automatic whiff punish is removed from the live build/menu. The canonical
[Fighter Data catalog](../FIGHTER_DATA.md) remains available to current and
future policies; it is embedded and parsed at startup, with no battle-time
network or SD-card reads.

## Optional posture

Idle crouch and Dtilt face menu toggles start ON. Holding L suspends idle crouch
until release. See [shared inputs and posture](../INPUT_RECIPES.md). These do not
reactivate the disabled incoming-hitbox auto-duck.

Dtilt face now prepares its directional Down angle before A, while Down is held,
and retains manual-tool priority through native down-tilt recovery between taps.
Attack presses remain physical; pre-attack aiming yields to automatic punishes.
Down Tilt-stick also qualifies with a neutral left stick (Tilt Attack mapping).
The helper adds no wait; native acceptance can still use the old facing if turn
and attack arrive together. Holding Down before A prepares the angle sooner.

Input-delivery corrections: Dtilt face retains its angle during native down-tilt
hitstop and preserves physical tilt-stick Down. Configured ledge timings are
unchanged; arena testing is still required.

Ordinary Roll read → Down smash can now release physical or automatic shield
for a reachable roll, budgeting shield-drop recovery before attacking. It waits
for native smash readiness and keeps ownership through shield release. Late
rolls and shieldstun are skipped; see [Roll read](../ROLL_READ.md).
