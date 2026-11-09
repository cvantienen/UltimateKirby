# Yoshi

**Tilt face** starts ON, with down tilt selected by default as on Kirby. It angles physical Down or down Tilt-stick
toward the nearest enemy in the same input poll. It adds no attack delay and
never creates A presses or forces native facing. Native acceptance can still
use the old facing when turn and attack arrive together. Holding Down before A
provides the turn preparation sooner. Yoshi does not gain Idle crouch.
See [input details](../INPUT_RECIPES.md).

## Optional out-of-shield response

**Out of shield** starts ON with the supplied defaults. File-authored
`shield_hit` recipes prefer a front grab, then close short-hop Nair. Configure
their distance/facing/delay in the Training reaction editor; the old compiled
startup/recovery selection is removed. Ledge responses keep priority. See
[Out of shield](../OUT_OF_SHIELD.md) and [Grab dodge](../GRAB_DODGE.md).

Standing C-stick correction: Dtilt face reads the physical right-stick axes in
the same mapper update, including the first Down flick before digital aliases
appear. With a neutral left stick it prepares the down-turn angle on both sticks;
no crouch prerequisite or waiting frame is added. Side/up C-stick inputs, aerials
and attack button edges remain unaffected. Native input acceptance still decides
whether the requested turn precedes an attack. C-stick must use Tilt Attack.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **5** (`yoshi`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Nair2; Fsmash |
| Jump | **Nair (instant nair, max drift)**; Fw air; Meteor fair (spacing); SHNAIR 1/2 DRIFT; SHNAIR MAX DRIFT; runs SHNAIR |
| Roll | **Tongue grab**; grab shuffle |
| Attack | Shield Nair; **Grab/rear Nair** |
| Drop | **Shield**; Dtilt > Shield; D AIR; Fw AIR |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_grab` | — | Automatic grab (unshielded) | `yoshi.roll.auto_grab` |
| `auto_shield_backup` | — | Shield grab backup | `yoshi.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `yoshi.roll.auto_shield_grab` |
| `shield_hit` | 1 | AUTO OOS Front grab | `yoshi.attack.auto_oos_grab` |
| `shield_hit` | 2 | AUTO OOS Nair | `yoshi.attack.auto_oos_nair` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3; 17 total; -- landing |
| Neutral Air | 3 start; active 3–4 / 5–11 / 12–25; 44 total; 7 landing |
| Forward Air | 16 start; active 16–20; 43 total; 12 landing |
| Back Air | 11/14/18 start; active 11 / 14 / 18–19; 55 total; 11 landing |
| Up Air | 5 start; active 5–6; 36 total; 8 landing |
| Down Air | 16/18/20/22/24/26/28/30/32/34/36/38/41 start; active 16 / 18 / 20 / 22 / 24 / 26 / 28 / 30 / 32 / 34 / 36 / 38 / 41; 56 total; 17 landing |
| Forward Tilt | 5 start; active 5–7; 29 total; -- landing |
| Up Tilt | 8 start; active 8–15; 31 total; -- landing |
| Down Tilt | 8 start; active 8–10; 23 total; -- landing |
| Dash Attack | 10 start; active 10–12 / 13–20; 40 total; -- landing |
| Forward Smash | 14 start; active 14–16; 52 total; -- landing |
| Up Smash | 11 start; active 11–13 / 14–16; 46 total; -- landing |
| Down Smash | 7/22 start; active 7–8 / 22–23; 49 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 14–21 |
| Dash grab | frames 16–23 |
| Pivot grab | frames 17–24 |
| Standing reach reference | 26.0 ~ world units |
| Shield-grab startup | 18 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
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

## Execution and calibration

Option definitions and input delays come from `fighters/yoshi.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair_ledge` | 0 | `drift:opponent:0.1:25, facing_input:a+x:0:0:4, drift:opponent:0.5:1` |
| Neutral / `nair_oppenent` | 0 | `facing_input:x:0:0:1, drift:away_from_opponent:0.61:4, facing_input:a:0:0:2, wait:1, drift:opponent:1:10, wait:3, facing_input:none:0:-1:4` |
| Neutral / `nair_v2` | 0 | `facing_input:a:0:0:19` |
| Jump / `nair` | 0 | `facing_input:a+x:0.35:0:4, drift:opponent:1:20` |
| Jump / `up_air` | 0 | `facing_input:x:0:0:1, wait:5, facing_input:a:1:0:4, drift:away_from_opponent:1:6, wait:5, facing_input:none:0:-1:4` |
| Jump / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
| Jump / `nair_fast` | 0 | `short_hop_nair, drift:opponent:0.50:10` |
| Jump / `nair_fas-2` | 0 | `short_hop, nair, drift:opponent:1:10` |
| Jump / `nair_fas-3` | 0 | `drift:ledge:100%:4, short_hop_nair, drift:opponent:1:5` |
| Roll / `grab` | 0 | `drift:away_from_opponent:0.4:20, wait:10, range:30, facing_input:r:0:0:4` |
| Roll / `grab_shuffle` | 0 | `wait:1, face:away, wait:1, face:opponent, walk:opponent:10:0.4, grab:fighter_range` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, short_hop_nair:4` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, facing_input:r:0:0:4` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `menu_added_4` | 0 | `facing_input:x:0:0:1, facing_input:a:0:-1:4, drift:opponent:1:15` |
| Drop / `menu_added_5` | 0 | `facing_input:x:0:0:1, wait:9, facing_input:a:1:0:1, drift:away_from_opponent:1:4, wait:11, facing_input:none:0:-1:4` |

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

- Registration: `crates/kirby_assist_core/src/fighters/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/policies/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


## Tracked ledge-roll grab

The fixed 20-unit experiment has been reverted. Roll → Grab uses shared tracked
inward walking, with Yoshi's own grab range and active window. A legal timed
catch can interrupt the approach; capture hands back control. Actual spacing
still needs Switch calibration.

## Yoshi-specific timing and movement

Yoshi has **Bair drag > Fsmash** on Y. It turns before the short hop and only
an early bair hit can queue the landing Fsmash. This is not a verified true
combo. Automatic whiff punishing is removed from the live build/menu.
Yoshi's Y recipe is file-authored; see [sequence files](../SEQUENCE_FILES.md)
and the canonical [Fighter Data catalog](../FIGHTER_DATA.md).

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

The fixed roll-read selector is removed. Configure Yoshi's onstage responses
using [roll reaction sequences](../ROLL_READ.md), separate from ledge Roll
get-up responses.

Jab-lock requests allow up to 24 advancing frames for native
acceptance without delaying accepted actions. Shared ledge aerials also tolerate
native takeoff status preceding the situation-flag update. Configured attack
startup and ledge timings are unchanged; arena testing remains necessary.


## Manual X/Y controls

X/Y/D-pad input sequences belong only to this fighter and are edited through the in-game
[sequence editor](../SEQUENCE_FILES.md#fighter-specific-xy-recipes). Normal button
keeps X short hop or ordinary Y input. The old native Y Finisher remains removed.
Ledge recipes and other fighter policies retain their separate controls. Modes
are available only on the ON/OFF page; no button can Hold/Toggle a mode.
