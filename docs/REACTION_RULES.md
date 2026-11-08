# Configurable reaction rules

## Current simple Auto react mode

The **Auto react** Training list groups all automatic response sequences for the
current fighter. Each has one saved recipe with `reaction = EVENT:MAX_X:MAX_Y`
and ordinary `actions`. Range/actions are editable like roll reactions; target
recognition, movement preparation, native readiness, ownership and cancellation
remain in code. Current Bowser entries are:

| Event token | X / Y | Recognition and existing mode switch |
| --- | --- | --- |
| `auto_roll` | 30 / 12 | Enemy ordinary roll; **Auto react** |
| `auto_spot_dodge` | 25 / 10 | Enemy spot dodge; **Auto react** |
| `auto_air_dodge` | 35 / 19 | Enemy air dodge; **Auto react** |
| `auto_grab` | 35 / 24 | Enemy shielding, vulnerable, in front; you are unshielded; **Automatic grabs** |
| `auto_shield_grab` | 25 / 10 | Enemy vulnerable, grabbable and in front; you hold shield; enemy shield not required; **Automatic grabs** |
| `auto_grab_dodged` | 25 / 10 | Your assisted dodge was natively confirmed, then native recovery permits acting; **Grab avoidance** |
| `auto_shield_backup` | 25 / 15 | Your shield is ready, a vulnerable enemy is close on either side, and no legal grab wins; **Automatic grabs** |

The first three and grab-dodge follow-up currently send Up-B for four frames,
then up to 69 drift frames toward the opponent. The two grab recipes send
`input:r:0:0:4`; unshielded grabs retain their existing movement preparation but
send neutral stick during the Grab request. Shield backup retains immediate native-startup
handoff to the next authored action. These are the exact current file values;
the configuration migration did not retune them. Any drift after Up-B comes
from the recipe's configured actions.

Training: **Assist modes → Auto react → L3/R3** shows the whole group. The
Automatic grabs and Grab avoidance rows also open their relevant subsets. On
**+ Add**, Left/Right chooses a missing event, then A/R3/X creates it. X/Y range
and action editing, caching and saving use the existing sequence editor. Legacy
priority lists do not compete with these entries. Duplicate event recipes are
rejected instead of arbitrarily choosing one.

Old files remain compatible: missing new grab/backup entries use the previous
native grab defaults and legacy grab-range keys. A missing `auto_grab_dodged`
allows grab avoidance without a follow-up; it never borrows `auto_spot_dodge`.
The supplied master now has
explicit grab/backup entries for every fighter, and Bowser has a separate
grab-dodge follow-up. This does not enable new roll/dodge reactions for fighters
that previously had none. Use the existing mode switches to disable responses.

Other fighters use their saved ranges/recipes. These are observation thresholds,
not hitbox changes or guaranteed punish coverage. The shield backup shares the
Automatic grabs / Out of shield mode; it does not require Auto react or a dodge.
For newly eligible offense, the order is usable roll/dodge reaction, legal grab,
then shield Up-B backup. Existing safety/ledge/committed-owner gates still apply.

Ordinary custom roll/spot/air-dodge responses use grounded control readiness;
they do not depend on native Up-B availability. Direct Up-B recipes retain their
specific native readiness, delivery and tracking checks. Ordinary responses keep
the existing stationary admission restriction, so they do not brake walking or
running. Shield control queries run only for an in-range configured candidate
and share one result across the three event types in that observation.

The revision cache also records which enabled automatic groups require ground
support or an Up-B availability query. Grab, shield-backup and grab-dodge
movement checks follow their own switches, independently of Auto react.
Unconfigured/grab-only fighters do not construct a roll-reaction engine, and
enabled engines are reused in place. These checks add no reaction waits or
sequence timing changes. Raw configured actions still obey native shield-drop,
move-startup and character restrictions.

For new offensive responses, a usable Auto react sequence wins before a new
automatic grab. Ordinary rolls are covered without facing or vulnerability filters,
including rolls into/across/behind your shield. A usable immediate Up-B can also
replace an automatic grab or approach in the same observation **before R is
delivered**, even when the grab's target remains valid. The cached grab output is cleared first.
Accepted native grabs and protected ledge/other committed responses remain
uninterruptible. Unshielded automatic grabs require visibly shielding, vulnerable
opponents in front. Shielded grabs require a vulnerable, grabbable front opponent
without requiring enemy shield. Target loss, loss of required enemy shield,
invulnerability or crossing behind ends further grab input. A delivered R remains protected from Up-B until native acceptance or
a bounded delivery expiry; settling owns no movement or buttons. No wait is added for a future grab opportunity. No reaction is queued while you cannot act; a still-current dodge can become eligible when readiness returns.

Once admitted, one response owns the gameplay buttons and both sticks. Other
Auto react events are not queued and global assist button requests are not mixed
into its inputs. Moving into the air does not hand the controls back midway
through a committed response. Damage, invalid observation/scope, native safety,
and ledge priority can still end it. Hanging opponents stop general reactions;
manual ledge recipes remain independent. Already-delivered continuous drift
can survive a skipped observation within the same step; missing a button edge
or crossing a step boundary still aborts instead of bursting delayed inputs.

There is no queued reaction during your own dodge, shieldstun or other
uninterruptible action. The mode takes no input ownership until native Up-B is
usable. Events observed while grounded but unable to act (including smash
endlag, shieldstun and your own dodge) take no input ownership. Once readiness
returns, an opponent's still-current dodge may trigger immediately. A finished
dodge is never queued or replayed, and a dodge already used for a response stays
consumed. This reuses the existing bounded per-opponent event history. A consumed event
survives a missing/reordered observation; an already-read native status change
clears its marker for a later event. Missing data does not prove an event ended.
Running, walking and ordinary landing can react immediately when Up-B is usable and the recipe begins
with a direct Up-B input, without a preliminary neutral waiting phase. Recipes
beginning with a grab, ordinary attack or approach cannot interrupt movement. The mod cannot cancel
uninterruptible game states or remove online input delay. No hitbox or physics
changes are made.

Unconfirmed automatic Up-B also has a 24-distinct-mapper-attempt fallback
limit, so frozen native clocks cannot hold B/Up forever. Duplicate observations
and duplicate/wrong-target receipts do not spend it. Native startup confirmation
disables this watchdog and advances to the next authored sequence action. There
is no wait before the first request. Leaving the ground before native startup
is confirmed cancels B/Up rather than allowing a stray airborne Up-B.

For a direct Up-B Auto react recipe, follow-through movement requires observed
native Up-B startup. If startup is not recognized before horizontal follow-through,
the response cancels instead of becoming a walking chase; it does not add a retry
or waiting phase. Generic Up-B entry and fighter-specific native continuations are
recognized. An observed roll/jump/wrong special cancels that response, and its
authored drift ends once ordinary movement/shield/fall control resumes. Fresh
physical input also releases the optional drift tail after native Up-B is confirmed,
including on the first drift observation; the critical initial B pulse stays
protected. The consumed dodge is not replayed. All ledge/button tapes also release immediately in native grab/catch/hold/
pummel/throw states, so they cannot select a throw or resume an old punish. Other custom
special-state transitions may need recognition if the generic entry was missed.
For direct Up-B reactions and automatic grabs, final mapped output also removes jump or
Guard flags unless the current step requested them. Requested X/Y/ZL jumps,
R-grab's native Guard/Attack components, and explicit OOS shield remain intact.
Authored ledge/button inputs retain their normal behavior outside native grab
interactions.

Holding a fixed native jump trigger suspends automatic punish modes and cancels
their input ownership: ZL on Default, R/ZR on Brother. A fresh native shield press
also cancels an active automatic Up-B response (Auto react, grab-dodge follow-up
or shield backup), except during its initial grounded
B input, so it cannot interrupt the requested OOS special. In the air, fresh shield
still cancels for a manual air dodge. A shield already held permits OOS reactions. The same dodge is not replayed afterward. This hands back
fresh physical input rather than mixing it into a running tape. Committed ledge
and explicit button tapes retain their separate ownership semantics.

Vulnerable-ledge/regrab punishment has been removed. Older `ledge_regrab`
recipes remain readable for compatibility but have no live detector, menu or
punish path. The older rule vocabulary below is a legacy reference; its
priorities do not apply to the current Auto react sequence group.

[Open the styled, searchable cheat sheet](REACTION_RULE_CHEATSHEET.html), or use
the [Markdown reference table](REACTION_RULE_CHEATSHEET.md), for every setting,
choice, file-field position and feedback label.

Reaction recipes work for all supported fighters. Events include ordinary onstage
rolls, vulnerable opponent spot dodges, confirmed shield hits, recovery after an assisted spot dodge,
and [opponent shields / shield-origin rolls](SHIELD_PRESSURE.md). The same
editor configures conditions and inputs. [Out of shield](OUT_OF_SHIELD.md) and
[Grab dodge + punish](GRAB_DODGE.md) have their own ON/OFF modes and startup defaults;
Roll reactions start ON in each fresh fighter menu. They run matching saved
roll rules and can still be switched OFF in Assist modes; button-controlled
groups retain their separate activation controls. Branching tech chases and
arbitrary new event families remain outside this builder.

## Configure in Training

1. Open **Assist modes** and highlight **Roll reactions**, **Shield pressure**, **Spot punish**, **Out of shield**,
   **Grab dodge + punish**. Use the row's ON/OFF
   setting to control that assist during the match.
2. Press **L3 or R3** in Training to open its separate priority-ordered rule list.
   Use Up/Down to select, A/R3 to edit, X to add, Y to manage
   (rename/add/duplicate/delete), and B/L3 to return to Assist modes.
3. Configure the rule's own state, minimum/maximum X distance, maximum height,
   opponent side and priority. Priority 1 is checked first. Equal priorities
   retain category/file order. Distances refer to fighter roots, not hitboxes.
   Roll rules can choose roll/cross/vulnerable/cross-vulnerable; Shield pressure
   can choose its eight shield/escape events. Spot punish, OOS and grab-dodge can choose
   their own event variants; they never move into another assist's list. New rules use Menu control; existing saved roll
   button-toggle groups remain compatible.
4. Configure the actions, Delay before and Start delay. Press Y in the editor
   to save. R3/L3 returns to the rule list; R+Left/Right cycles that assist's rules.
   Native move eligibility and timing still apply to the simulated inputs.
5. These rules are not ledge/button choices. They are all considered by their
   enabled assist, independently of the currently selected ledge/button recipe.
   A miss does not cause historical events to replay. Existing ledge ownership
   protects an acquired setup and releases it when its response finishes.

Ordinary ledge/button editors no longer expose Auto reaction tagging. Existing
reaction-tagged sections remain readable under their original saved IDs and are
automatically routed to the proper assist list. Saving does not require manually
moving or renaming old sections. Standalone helper modes without configurable
input sequences keep their ON/OFF rows rather than empty rule editors.

## Conditions

| Setting | Values and meaning |
| --- | --- |
| Auto reaction | `off`; `roll` = any eligible point during an ordinary forward/back roll; `roll_cross` = relative X position crossed during that roll; `roll_vulnerable` = native forecast says roll intangibility has ended; `roll_cross_vulnerable` requires both crossing and vulnerability; `shield_hit` = confirmed hit after native shieldstun; `grab_dodged` = native assisted spot dodge followed by actionable recovery |
| My state | `any`, `grounded`, `airborne`, `standing`, `walking`, `running`, `shielding` |
| Min / max X distance | Inclusive horizontal distance bounds, 0–100 units |
| Max height difference | Absolute vertical separation, 0–100 units |
| Enemy side | `any`, `front`, `behind` |
| Priority | 1–8, lower first among enabled groups |
| Mode control | `menu` (default), `toggle_x`, `toggle_y`, `toggle_up`, `toggle_down`, `toggle_left`, `toggle_right` |

Crossing uses relative positions, so your own movement also affects it. Rules
can wait for their conditions to become true while a roll is still observed;
`roll` does not mean only the exact first animation frame. Once a response starts,
conditions are not reevaluated to switch to a different rule halfway through it.
Priority is applied to conditions that match **now**, not predicted future
conditions. A plain `roll` response can fire before a later crossing. Use
`roll_vulnerable` and `roll_cross_vulnerable` together when you want to compare
those outcomes at vulnerability rather than respond at the start.
There is one attempt per enemy roll within a monitoring session;
preemption/cancellation does not rearm the same roll. New rolls rearm it. Turning
all groups OFF then back ON starts a fresh monitoring session. At most one response plays at once.

## Text syntax and example

```ini
# reaction = TRIGGER:MY_STATE:MIN_X:MAX_X:MAX_HEIGHT:ENEMY_SIDE:PRIORITY[:CONTROL]
[yoshi.roll.near_grab_rule]
name = Near roll grab
start_delay = 0
timeout = 180
spatial_timeout = 30
default = false
reaction = roll_vulnerable:grounded:0:12:10:front:2
actions = face:opponent, grab

[yoshi.roll.cross_smash_rule]
name = Cross roll down smash
start_delay = 0
timeout = 180
spatial_timeout = 30
default = false
reaction = roll_cross_vulnerable:grounded:0:14:10:any:1
actions = down_smash:4

[yoshi.roll.far_smash_rule]
name = Far roll forward smash
start_delay = 0
timeout = 180
spatial_timeout = 30
default = false
reaction = roll_vulnerable:grounded:14:28:10:front:3
actions = face:opponent, forward_smash:4
```

These ranges/delays are examples to tune, not calibrated guarantees. Save each
recipe after editing. `default` still controls manual recipe selection; it does
not turn automatic rules on. A button's `while_held` affects its manual activation;
automatic reaction playback always runs one pass.

## Menu versus button activation

Mode control is saved with each recipe's conditions. Seven-field lines retain
Menu behavior. An optional eighth field selects activation, for example:

```ini
[yoshi.roll.button_cross_rule]
name = X toggle cross response
start_delay = 0
timeout = 180
spatial_timeout = 30
reaction = roll_cross_vulnerable:grounded:0:14:10:any:1:toggle_x
actions = down_smash:4
```

X controls every active-tag recipe assigned `toggle_x`, regardless of which X
recipe is selected. The assigned button is reserved while those rules are
configured, even when their group is OFF: it toggles the group instead of jumping
or running the ordinary button sequence. That sequence is preserved, not deleted.
The compact button row says **Toggle rules / saved: ...** to show the override.
Use an otherwise unused button if you want to preserve manual jump access.

A fresh, unchorded press toggles once; holding does not repeat. Only X, Y and the
four D-pad directions are offered. The menu must be closed; L/Minus/Plus chords,
simultaneous sequence buttons and menu navigation do not activate groups. After
assignment/spawn, release a held button before toggling. Removing the last rule
for a button clears its toggle and restores that button after release.

The Roll rules menu switch controls only **Menu** recipes. It is not the master
switch for button groups. Group states are shown together; use each button again
to switch its group OFF. Enabled groups share priorities and one response engine.
Changing controls/mode state cancels pending response ownership safely. Automatic
helpers such as auto-tech, mash and down-tilt facing remain menu-only; this feature
does not expose legacy helper-mode bindings. Activation adds no polling hook, worker, SD file or opponent-scan path; small
per-fighter button masks use existing controller samples. Ground-watch settings refresh only on configuration or
activation changes and ignore disabled groups.

## Bounds, safety and performance

- Eight rules maximum per assist (six families, at most 48 per fighter). Parsing and
  saving reject invalid bounds, unknown tokens, duplicates and excessive rules.
- With every group OFF there is no additional opponent scan or collision ray. ON reuses the existing bounded
  eight-fighter roster and a single roll/follow scan. It does not enumerate
  projectiles, hitboxes or the scene.
- Recipes are resolved only when enabled or when their configuration changes.
  Matching has no per-frame allocations, file reads or formatting. A runtime
  plan is lazily allocated and reused; visible UI remains the cached selector.
- Ground-support rays are used only for ground-movement recipes near a relevant
  roll or while their remaining movement needs support. Simple attacks add no
  such rays. Spatial steps require a fresh locked target; missing targets cancel.
- Onstage rules have no verified ledge anchor. Tagged recipes cannot use ledge,
  inward or outward directions; use opponent/away/facing/backward instead.
- Tech/mash safety, damage, native stop clocks and input ownership retain their
  existing priority. Active ledge trapping/defense cancels custom roll playback.
  Turning all custom roll groups OFF leaves roll reactions inactive. No fixed
  Kirby/Yoshi helper or generic far-roll smash fallback resumes.
- Rendering never evaluates rules, applies saves or frees fighter rosters. Match
  cleanup releases the rule cache with the other transient menu data. SD saves
  stay on the single immediate Training worker with verified backups/recovery.

No physics, hitboxes, move definitions, animation rates or fighter statuses are
modified. Host tests and compile checks do not establish on-device performance
or prove that the previous GPU crash is resolved. Build/upload remain separate.

This syntax requires the updated mod on every Switch that loads the file.
Older builds cannot parse `reaction` metadata; update their plugins before
transferring tagged sequences. The implementation does not rewrite or upload
your existing configurations automatically.

## Ledge handoff

An acquired ledge visit reserves its setup and configured response. Other
configured reaction, punish and button-sequence owners cannot replace it,
regardless of recipe priority. Existing ledge approach shielding remains available;
tech/mash/DI safety still takes priority when needed.

The ledge response releases ownership when finished or cancelled; its final
input frame remains protected. All seven automatic events and grab avoidance
remain disabled while ledge mode is ON or the ledge binding is held. Their
opponent searches resume only when both are OFF. Historical events are not
queued through ledge mode; the next eligible check uses current native state.

This handoff uses existing observations and constant-size ownership checks, with
no additional fighter scans, geometry queries or file access during play.

## Checking an unexpectedly early Up-B

Check every enabled response that explicitly contains Up-B, including Grab dodge
and OOS as well as roll rules. `roll` starts on a matching observed roll;
`roll_vulnerable` waits for observed vulnerability. A zero start delay sends the
first input immediately after that rule matches; it does not align the move's
active frames with the opponent's recovery automatically. A height tolerance of
zero requires effectively equal root heights. A front-only condition belongs in
the rule's Side field if it must affect rule selection: an `enemy_in_front` check
inside the selected action tape can abort that tape, but does not choose another
lower-priority rule for the same consumed event.

Sequences use native progress clocks and pause during stop; they do not measure
or automatically compensate for network delay. Skipped edges abort rather than
bursting late actions. Timing can therefore still differ from a Training setup;
these arbitration fixes do not establish identical online outcomes.

Matched roll targets carry native motion identity/frame/rate into the initial
sequence sample and later locked follow snapshots. This lets waits progress from
the observed opponent clock while your shield pose is static, including a
positive reported shield animation rate. Duplicate polls add no time; hitstop
and missing native samples retain their pause/rebase behavior.

## Manual handback and grounded playback

Active file-authored sequences exclusively own gameplay buttons and both sticks,
including start delays, waits and transitions. Physical action presses and stick
changes cannot mix into or cancel an ordinary sequence. Pause/menu controls and
existing safety arbitration remain available. Explicit **hold until manual takeover**
actions still inspect the original physical input and end on fresh manual intent.
Releasing a hold-to-run binding or ledge activation still ends its activation scope.
This changes controller input only; it cannot erase an input the game already
buffered before the sequence began or cancel an already-started native move.

A physically held shield that was suppressed during an automatic punish stays
suppressed until released; release/repress shield to use it afterward. This avoids
restoring an old hold as a new shield edge alongside movement or while airborne.
A direct shield Grab or Up-B that explicitly keeps Guard does not suppress that
human hold, so a failed request never quarantines the held shield. Physical Grab
takes priority before native catch is visible. Fresh shield presses remain allowed. The gate
uses existing controller samples and constant-size state, with no timer or scan.

**Stop if airborne** in the assist recipe editor saves `ground_only = true`.
It stops remaining/held inputs if the fighter becomes airborne. Its default is
false, preserving existing jump sequences and the meaning of My state: grounded
as a start condition. The revised Bowser profile opts in and uses no jump/Nair
responses. Saving retains the property. Updated builds are required for the key.

## Vulnerable-grab and immediate event variants

- `shield_pressure_vulnerable`: one Shield pressure rule covers a natively
  hittable held shield or a shield-origin roll whose vulnerability has opened.
- `shield_hit_vulnerable`: OOS requires the confirmed attacker to have normal
  native hit status when selecting the response. Own shield recovery still gates it.
- `grab_dodged_vulnerable`: the assisted-dodge follow-up requires a natively
  hittable grabber. It does not bypass your own dodge acceptance/recovery.
- `spot_dodge_any`: reacts to the observed ordinary spot-dodge status without
  requiring its vulnerability/recovery window to be open. Existing `spot_dodge`
  recipes retain their conservative vulnerability-window behavior.

All use the existing seven/eight-field `reaction` syntax and the same per-mode
ON/OFF and eight-rule budget. Priority 1 does not reserve a future grab: a lower
priority immediate event can fire first while the grab condition is still false.
Committed playback does not switch later. See [Bowser's simple profile](fighters/bowser.md).

## Shield-to-sequence handoff

The shared sequence player releases shield with neutral buttons and sticks before
ordinary attacks or movement that require dropping shield. It waits through native
shield-drop (`GUARD_OFF`) before starting that action's configured input duration.
This covers shield-until-hit, timed shield and reactions started while shielding;
you do not need to add a two-frame wait to avoid down+shield becoming a spot dodge.
The native shield-drop time still applies, and the overall sequence timeout remains
in force. Direct out-of-shield grab, jump, Up-B and up-smash inputs are allowed from
shield without an added release step; guard damage/drop must finish first. Explicit
shield+direction inputs retain their literal meaning, including intentional dodges.
This uses already-copied status and situation; it adds no native queries or scans.

## Final releases and interrupted actions

Auto react retains input ownership for its already-scheduled final neutral
release, including repeated mapper polls. A grab-dodge proposal cannot replace
that release. Completed Auto react/automatic-grab locks are released before
new proposals are sampled, so the next observation can hand control to another
mode immediately.
Native safety and new ledge acquisition retain their existing preemption.

Running tapes stop on capture/damage/death, knockdown (`DOWN` through
`DOWN_WAIT_CONTINUE`), knockdown damage/reflection, and the native jump-board
rebound damage state. Fixed automatic grabs also release on those interruptions,
even when the victim is still grounded. Normal get-ups, shielding, dodge, attacks
and specials retain their existing behavior. Performing your own throw (`THROW`)
continues a configured sequence; being the throw victim (`THROWN`) cancels it.
The old range incorrectly included the attacker's throw. These status checks use the copied
observation and add no rays, native queries, frame waits or physics changes.
Status identifiers are checked against the [pinned Skyline Smash bindings](https://github.com/ultimate-research/skyline-smash/blob/d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435/src/lua_const.rs).

Changing a menu selection invalidates observations that began with the previous
settings and clears their published requests. It preserves auto-tech's existing
acknowledged-edge/cooldown and floor-motion history; the next observation applies
current settings. Match/fighter/controller cancellation still clears the full
state. This avoids replaying an old response or resetting tech delivery merely
because a different ledge recipe was selected.

Holding the assigned ledge-assist binding or its ON toggle suspends general
automatic reactions, grabs and grab avoidance before any ledge acquisition,
throughout its response and after it finishes. The corresponding opponent
searches remain OFF until both ledge activation sources are OFF.
Committed grab and roll sequences also retain their existing mutual exclusion.
Explicit recipe previews suspend automatic reactions. Native tech, mash and
damage-input safety remain available. Handoff resumes fresh live checks;
an old reaction is not queued through a reserved ledge response.

For a roll-punish grab that should cancel rather than stand still when too far,
use **Check fighter grab reach instantly** followed by **Grab / Zair**, or
`grab:fighter_range_instant` in the file. The original reach check keeps its
bounded wait for ledge setups; see [sequence checks](SEQUENCE_FILES.md).

A full-strength initial Auto react chase owns its first horizontal flick even if
that direction was already held manually. This does not add a neutral frame or
repeat the flick on subsequent polls. A leading chase from shield still waits for
native shield-drop recovery; short observation gaps during that neutral wait do
not cancel the plan or shorten the configured chase. Automatic grabs and manual
ledge/button input histories are unchanged.

### Late native Up-B confirmation

Automatic Up-B responses retain an input-free receipt after a verified final
mapped Special+Up packet. Shield hitstop, readiness loss and a late native status
change can then continue the already-started sequence instead of discarding it.
Only the locked opponent is followed during unactable shield recovery; no new
reaction is admitted there. The receipt reserves automatic-punish priority but
owns no gameplay inputs, expires after 24 distinct mapper updates, and never
retries B. Native confirmation advances past the delivered B step to the next
authored action. Target loss/capture, damage, other moves, an unconfirmed
airborne transition and ledge-trap L still cancel it. Failed
mapper attempts retain their old delivery bound and cannot arm a receipt.
