# Kirby

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **6** (`kirby`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Dsmash; Stone; Inhale; Bair |
| Jump | **Nair**; Bair |
| Roll | **Dsmash**; Grab |
| Attack | **Grab/rear Nair**; Grab/rear Guard |
| Drop | **Shield**; Dtilt > Shield; SH Dair |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 8–32 |
| Down tilt | 4–6 |
| Forward tilt | 5–8 |
| Down smash | 7–19 |
| Jump Bair | 6–12 |
| Grounded down-special startup | 11 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 6 |
| Dash startup | 9 |
| Pivot startup | 10 |
| Shield-grab startup | 10 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.5 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
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

[Move source](https://ultimateframedata.com/kirby) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares short hops for common nair responses.

See [this fighter's detailed execution notes](../KIRBY_YOSHI_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/kirby.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/kirby/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/kirby.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

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
The inset is `ROLL_GRAB_LEDGE_INSET` in Kirby's `timing.rs`.

**Neutral → Down smash** replaces the former Forward tilt menu choice. On
recognition it walks toward the remembered ledge at 0.65 stick during the
existing wait, aiming six units inside with native braking/floor checks. It
stops walking at the attack boundary even if still far away. The timing target
now starts two frames earlier than the old midpoint alignment: protection expiry
at animation frame **15**, within
of Kirby's active frames **7–19** (clean 7–11, late 12–19;
[Ultimate Frame Data](https://ultimateframedata.com/kirby)). Preparation begins
16 frames before predicted vulnerability, including one neutral smash-primer
update; down+A follows with 15 frames remaining. Nominally the hitbox begins
eight frames before protection ends and remains active four frames after it.
Native startup releases A to
avoid charging. Arrival/range is never required before the attack; native action
readiness still applies. `NEUTRAL_DOWN_SMASH_EARLY_FRAMES = 2` in Kirby's
`timing.rs` advances the selected move's midpoint alignment;
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

These constants live in
`crates/kirby_assist_core/src/fighter_profiles/kirby/timing.rs`.
The sequence is in `respond_to_neutral_getup_nair` and
`walk_before_neutral_nair` in
`crates/kirby_assist_core/src/ledge_trap/fighters/kirby.rs`.
Jump nair also opts into the shared movement helper while its existing timing
checks are waiting. Immediate jump cues still launch immediately, leaving no
grounded walking time. Horizontal digital flick aliases are suppressed during
these walk preparations; other response movement remains independently configured. Shared movement lives in `fighters/common.rs`.
See the
[fighter-specific sequence example](../FIGHTER_EXECUTION.md#kirby-neutral-nair-walk-example).

Neutral nair keeps its quarter-stick nudge below animation frame 4. Jump nair
now uses **full 1.0 forward stick** after exact native nair acceptance through
its configured active window (8–32), returning horizontal stick to neutral at
frame 33 or earlier on landing/interruption. Launch stays neutral Jump+A, so
forward drift cannot select fair. Neutral nair adds two frames to its configured
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

## Y finisher attempts

Select **Fair 1 > Fsmash** or **Dair > Dsmash** in the **Finisher** row and press Y.
Both use two-button short hops and native hit confirmation before a landing
smash. Holding Y does not repeat. See [sequences and limits](../FINISHERS.md).
These are not guaranteed true combos or kills. Auto-duck and its hook/scans
are disabled; its former Y binding is removed.

## Reference facts

Automatic whiff punish is removed from the live build/menu. The compiled
[opponent reference](../OPPONENT_DATA.md) remains available for future policies;
no files load during battle.

## Optional posture and Fair starter

Idle crouch, Dtilt face and A+B grab menu toggles start ON. Holding L
suspends idle crouch until release. Tap neutral A+B for one jab 1 into a grab
when native recovery allows it; A alone stays normal. They are described in
[shared inputs and posture](../INPUT_RECIPES.md); they do not reactivate the
disabled incoming-hitbox auto-duck. The Fair finisher now waits for the short-hop
apex, selects fair in air and fast-falls from accepted fair frame 4 while descending.
Any confirmed hit from this Y-started fair now permits landing up-angled Fsmash
toward the opponent; later fair frames do not cancel it. Dair is unchanged.
See [finishers](../FINISHERS.md).

Dtilt face now prepares its directional Down angle before A, while Down is held,
and retains manual-tool priority through native down-tilt recovery between taps.
Attack presses remain physical; pre-attack aiming yields to automatic punishes.

Input-delivery corrections: Dtilt face retains its angle during native down-tilt
hitstop and preserves physical tilt-stick Down. Finisher attempts keep queued
Jump alive for up to 24 advancing frames without holding/repeating it. Native
takeoff status/situation overlap no longer cancels the aerial. Configured ledge
timings and fair-apex selection are unchanged; arena testing is still required.

Ordinary Roll read → Down smash can now release physical or automatic shield
for a reachable roll, budgeting shield-drop recovery before attacking. It waits
for native smash readiness and keeps ownership through shield release. Late
rolls and shieldstun are skipped; see [Roll read](../ROLL_READ.md).
