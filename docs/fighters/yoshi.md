# Yoshi

## Down-tilt chase / jab lock

The separate **Jab lock** ON/OFF row starts OFF. It chases only Yoshi's own
confirmed down-tilt hits: missed techs get a jab-lock attempt into down smash;
techs get grab attempts. It takes priority over background Roll read while active.
See [setup, limits and calibration](../JAB_LOCK.md).

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **5** (`yoshi`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Head Fsmash; Dsmash; Dair |
| Jump | **Nair**; Uair; Meteor fair (spacing) |
| Roll | Dsmash; **Tongue grab** |
| Attack | Shield Nair; **Grab/rear Nair**; Shield Uair |
| Drop | **Shield**; Dtilt > Shield; Meteor (risky); Dair (risky) |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 3–25 |
| Down tilt | 8–10 |
| Forward tilt | 5–7 |
| Down smash | 7–8 |
| Down-smash rear window | 22–23 |
| Up Tilt | 8–15 |
| Forward Smash | 14–16 |
| Up Air | 5–6 |
| Forward Air | 16–20 |
| Back Air First | 11–11 |
| Back Air Middle | 14–14 |
| Back Air Finish | 18–19 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 14 |
| Dash startup | 16 |
| Pivot startup | 17 |
| Shield-grab startup | 18 |
| Active duration | 8 |
| Standing reach reference (world units) | 26.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 104 |
| Walk speed | 1.208 |
| Initial dash | 1.98 |
| Run speed | 2.046 |
| Air speed | 1.344 |
| Gravity | 0.08 |
| Fall speed | 1.29 |
| Fast fall | 2.064 |
| Short-hop height | 14.43 |
| Full-hop height | 36.09 |

[Move source](https://ultimateframedata.com/yoshi) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/yoshi.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/yoshi/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/yoshi.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Tracked ledge-roll grab

The fixed 20-unit experiment has been reverted. Roll → Grab uses shared tracked
inward walking, with Yoshi's own grab range and active window. A legal timed
catch can interrupt the approach; capture hands back control. Actual spacing
still needs Switch calibration.

## Yoshi-specific timing and movement

Yoshi has **Bair drag > Fsmash** on Y. It turns before the short hop and only
an early bair hit can queue the landing Fsmash. This is not a verified true
combo. Automatic whiff punishing is removed from the live build/menu.
See [finisher sequences](../FINISHERS.md) and the compiled
[movement/weight/move reference](../OPPONENT_DATA.md).

Neutral nair reacts immediately to recognized Neutral get-up: face the remembered
ledge (turn with 0.6 stick if needed), confirm native facing, then send **X+ZL**
without A and a brief full-stick forward nudge. Both buttons must map to Jump;
both release in squat for a short hop. Neutral A waits until the observed
protection countdown minus nair startup (3) minus `NEUTRAL_NAIR_LEAD` (16) has
elapsed in advancing native game frames. For 30 protection frames remaining,
this waits 11 game frames before requesting A. Attack selection is neutral,
avoiding fair/bair. Unknown protection withholds A; landing/interruption or an
invalid/expired clock cancels the attempt. This uses a snapshot at takeoff,
not delay learning or per-frame opponent tracking. Native readiness/control
still apply. A must map to Attack. Accepted descending nair may fast-fall from
animation frame 3. Jump nair now also turns toward the remembered ledge and confirms facing before
launching its immediate short-hop nair. Already facing correctly skips the turn.
Native jump readiness permits launch during the turnaround animation.
Both use a full 1.0 forward-stick nudge below confirmed nair animation frame 2;
Neutral also nudges during the first two game frames of its takeoff/wait.
Horizontal input is neutral while selecting A and after the nudge windows.
Neutral nair retains its descending fast fall while drifting. Existing momentum
can continue afterward; drift does not extend the move’s active hitboxes.
Up-air and fair retain adaptive jump selection.

Roll down-smash stands two units toward the ledge from the configured point.
Tongue grab uses the shared active-window/range check and 0.65 inward walking;
its longer startup and eight active frames are distinct from Kirby's grab.
The fixed 20-unit experiment has been removed: the shared tracked endpoint can
extend inward from observed roll travel, and Yoshi stops inside his usable grab
area rather than at one fixed coordinate. A legal timed catch can interrupt the
walk. Control returns on capture, without an automatic pummel or throw. Offstage fair/dair choices
remain risky chases with their own longer drift, not the brief nair nudge.


### Immediate Neutral dair

Neutral **Dair** locks the remembered ledge direction, turns if needed and
confirms facing before jumping. It sends neutral X+ZL, releases both in squat,
then selects down+A after native takeoff, creating a short-hop dair without a
vulnerability-time or spacing wait. Full forward drift begins only after exact
dair acceptance and lasts until animation frame 30 or landing/interruption.
No automatic dair fast fall is added. The player controls spacing.
Neutral Bair, Forward tilt and Up tilt have been removed; current choices are
Nair, Down tilt, Forward smash, Down smash and Dair.

## Onstage Roll read

The separate Roll read row offers Off, Down smash and Grab, without holding L.
Down smash checks its front 7–8 and rear 22–23 windows against native facing;
tongue grab uses 14–21 with the 26-unit reach reference. Both share the bounded
[onstage recognition policy](../ROLL_READ.md), separate from ledge Roll get-up.
Reach estimates still need Switch calibration.

Finisher and jab-lock requests now allow up to 24 advancing frames for native
acceptance without delaying accepted actions. Shared ledge aerials also tolerate
native takeoff status preceding the situation-flag update. Configured attack
startup and ledge timings are unchanged; arena testing remains necessary.
