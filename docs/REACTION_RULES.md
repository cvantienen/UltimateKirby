# Configurable reaction rules

## Start here: a recipe is not a mode

A **recipe** is an ordered tape of controller inputs. A **reaction** says which
observed opponent event may start that tape. A **mode** enables a family of
observations; selecting or saving a recipe does not enable every assist.
All behavior remains input-only: no native physics, hitboxes, animation rates or
fighter statuses are edited.

The live unified controller uses **six** simple `auto_*` events. Older seven/eight-
field priority rules still parse and have retained core/editor support, but they
are not the live unified Auto react event list. `auto_grab` is retired and rejected;
`ledge_regrab` is readable legacy metadata with no live punish path. Do not turn
an old example into a live rule just by changing its label or category.

## Current simple Auto react mode

Use `reaction = EVENT:MAX_X:MAX_Y`. Each distance is 0–100 game-world units.
X is absolute horizontal fighter-root separation; Y is absolute vertical root
separation, not height above the stage or diagonal distance. MAX_Y=0 permits
approximately equal heights, not necessarily grounded fighters. These are
admission limits, not hitbox/reach changes or guaranteed punish windows.

| Current event | Existing mode switch | What starts the response |
| --- | --- | --- |
| `auto_roll` | Auto react | An eligible current opponent forward/back roll |
| `auto_spot_dodge` | Auto react | An eligible current opponent spot dodge |
| `auto_air_dodge` | Auto react | An eligible current opponent air dodge |
| `auto_shield_grab` | Automatic grabs | A qualifying vulnerable, grabbable front target while you shield; enemy shield is not required |
| `auto_grab_dodged` | Grab avoidance | Recovery after your assisted grab dodge was natively accepted; not an arbitrary enemy spot dodge |
| `auto_shield_backup` | Automatic grabs | A configured shield response when no legal automatic grab wins |

The currently supplied Bowser source has maximum X/Y pairs: roll 25/12,
spot dodge 25/10, air dodge 25/19, shield Grab 22/10, assisted-grab-dodge
follow-up 25/10, shield backup 20/12. These are verified authoring-file values,
not Rust defaults or a guarantee about a user's active SD file. They are not
shared implicitly with another fighter.

There is at most one sequence per simple event per fighter. The header category
stores its identity; the reaction field establishes the automatic event. Active
reaction entries are excluded from ordinary ledge/button choices. There is no
implicit cross-fighter copy. Missing roll/dodge/follow-up recipes mean no such
custom response; a missing shield backup means no backup attack. The fixed
shielded-grab fallback has its own code/range settings; it is not a seventh event.
Current live automatic grabs do not begin an unshielded walking/dash chase.

These examples are **valid authored starters**, not a snapshot of the supplied
saved file and not calibrated advice. Replace a fighter's existing event rather
than adding a duplicate:

```ini
[yoshi.roll.my_roll_response]
name = Roll response starter
start_delay = 0
timeout = 180
spatial_timeout = 30
reaction = auto_roll:30:12
actions = check:enemy_in_front:0, range:fighter_range_instant, press:r:4

[yoshi.roll.my_grab_dodge_followup]
name = After my assisted grab dodge
start_delay = 0
timeout = 180
reaction = auto_grab_dodged:25:10
actions = up_b:4, drift:opponent:25%:12
```

The first tape cancels immediately if the target is behind or beyond the fighter's
estimated standing-grab reach; it does not search for another response after that
cancellation. The second requests Up+B then drift; native Up-B availability and
startup confirmation still govern automatic direct-Up-B handoff. The names do
not guarantee a grab, a hit, or native special acceptance.

## Configure in Training

1. Open Minus → Assist modes. Toggle **Auto react**, **Automatic grabs**, or
   **Grab avoidance** deliberately; each controls its own family.
2. Highlight Auto react and press L3/R3 to browse the current fighter's simple
   events. Automatic grabs opens Shield Grab/Shield Backup; Grab avoidance opens
   its follow-up subset. These are different views of the same recipes.
3. Select a recipe and use A/R3 to edit. On **+ Add**, Left/Right chooses a
   missing event; A/R3/X creates its draft. An already configured event must be
   edited instead. New editor presets are starters, not saved-file defaults.
4. Change actions and their holds/delays. Below the action list, edit **Max X
   distance** and **Max height difference**. The simple editor does not offer
   legacy priority, minimum distance, own-state, enemy-side or toggle groups.
   Nor does it offer Stop if airborne for a simple event; a saved `ground_only`
   property is distinct from initial grounded admission.
5. X explicitly tests the draft; Y alone explicitly saves. Wait for the result
   before another save/delete. B/L3 leaves the list for the same Assist row;
   R3 finishes editing. A saved rule is not a ledge/button startup default.

Outside Training, only the selectors and mode switches are available: no test,
editor, rename/add/delete or save. Session selections and edits do not persist
across confirmed match end. Saved text persists; mode startup follows the actual
configuration/resolver, not a blanket “all ON” or “all OFF” claim.

## Admission, priority and ownership

A current simple event is considered only while its family is enabled, its target
is fresh/valid and native control readiness permits starting. Grounded ordinary
responses use controllable admission; a direct first Up-B input uses its specific
native availability checks. An opponent event observed during your own dodge,
shieldstun or uninterruptible recovery is not queued. If that same opponent event
is still current when readiness returns, it may become eligible then. Finished
or consumed events are not replayed. The bounded target history is not a
persistent cross-match event queue.

For newly eligible offense, a usable roll/spot/air-dodge response takes priority
before a new legal automatic grab, then the shield backup. Current simple event
selection is not ordered by the legacy numeric priority, vulnerability or
crossing fields. A direct immediate Up-B may replace an **undelivered** shield
Grab request; once R has a protected delivery receipt it cannot be replaced by
that new Up-B proposal. A recipe's `check:` is evaluated after selection; a
failed check cancels that tape rather than selecting a fallback for the same event.

One admitted response owns gameplay buttons and both sticks. Other events do
not queue or merge arbitrary assist buttons into its packet. The final scheduled
neutral release remains owned even on repeated mapper polls. Ledge activation
(held binding or ON toggle) suspends the ordinary automatic event/grab searches,
not only after a ledge is acquired. Editor previews also reserve their scope.
Tech/mash/damage-input safety and native interruptions retain higher-level gates.
Ledge and explicit manual button tapes have their separate ownership semantics.

## Cancellation and direct Up-B receipt

Target/scope loss, invalid observations, damage/capture/death and native safety
can stop an attempt. Releasing a hold-to-run binding ends that activation. A
physical press cannot undo a move the native game already accepted or buffered.
Automatic direct-Up-B tails have explicit manual handback: after confirmation,
fresh physical intent can release optional drift; the critical initial grounded
B pulse is protected. Fixed native jump triggers suspend automatic punish modes
(Default ZL; Brother R/ZR). A fresh shield press can cancel automatic Up-B outside
its protected initial grounded pulse; an already-held shield permits OOS use.

A successfully delivered final Special+Up packet can leave an **input-free
receipt** while awaiting late native Up-B startup. The receipt reserves automatic
priority but does not repeatedly send B or own movement. Its 24-distinct-mapper-
update bound is a watchdog, not a configured recipe delay. Duplicate updates and
wrong-target receipts do not spend it. Confirmation advances to the next authored
action with that action's full budget; there is no catch-up burst. Unconfirmed
airborne transitions, target loss, interruption or ledge takeover cancel it.
Ordinary raw `up_b` in a manual tape is still a proposed input, not a general
native acceptance/retry engine.

## Frames, waits and safe editing

`start_delay` waits after admission, not from the enemy animation's first frame.
`up_b:4` holds input four native-progress frames, not four startup frames.
`wait:4` releases owned buttons/sticks for four frames; `wait:0` consumes none.
Native hitstop pauses sequence progress. Duplicate observations reuse output;
invalid/gapped clocks can cancel rather than burst missed edges. The input
schedule is not a wall-clock timer or an online latency promise.

Edit one event at a time, test with a known target, then Save and wait. Save writes
its exact validated draft through the existing single worker, backup and verified
replacement path; it is not autosave. Closing the editor does not cancel a write
already started. A successful parser check proves grammar/capacity only, not an
installed plugin version, native acceptance, hit coverage or console performance.
See [sequence file lifecycle](SEQUENCE_FILES.md#in-game-save-safety) and
[all editor actions](EDITOR_ACTION_CHEATSHEET.md).

## Legacy priority builder: read this only for an existing legacy rule

The remainder explains retained grammar/core consumers, not the current unified
live event list. Legacy rules may appear in old catalogs and test/editor consumers;
parsing does not wire them into a live feature. Keep saved IDs when migrating.
Replace the relevant event with one simple `auto_*` recipe if you want current
unified behavior, and validate before saving. `ledge_regrab` remains inert.
A `mode = ...` button section is a separate retired binding mechanism; it is not
reaction activation and does not enable current helpers.

## Legacy conditions (compatibility reference only)

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

## Legacy text syntax and example

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
recipe after editing. Active reaction recipes cannot be saved as manual startup
defaults. `default` belongs to manual selection, not automatic enablement.
A button's `while_held` affects its manual activation;
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

Held/toggled ledge activation reserves its tracking/response scope and suspends
general automatic reactions, grabs and grab avoidance until both sources are OFF.
A recipe priority cannot steal an acquired ledge visit. Completion/cancellation
releases the response; its already-scheduled final release remains protected.
Historical events are not queued through this reservation. Use the next fresh
eligible observation after handoff, not a stale recorded dodge.

## Manual handback and grounded playback

Ordinary manual file tapes own their scheduled buttons/sticks, including waits.
The explicit `hold_until_manual` family is different: it watches physical intent
and ends on takeover. `ground_only = true` is a saved throughout-playback safety
flag, whereas the simple automatic grounded admission applies before starting.
No flag changes native gravity, recovery or jump behavior. For an instant
in-tape range gate use `range:fighter_range_instant` then `press:r:4`; the waiting
`range:fighter_range` remains useful when a bounded wait is intended.

## Checking an unexpectedly early Up-B

Find the actual event/recipe and mode, not only the menu label. Check its maximum
X/Y, start delay, leading input, native readiness and confirmation. A zero start
delay requests the first input immediately on admission; it does not automatically
align startup with enemy recovery. A legacy `roll_vulnerable` example is not the
same live trigger as `auto_roll`. Inspect delivered-input/startup evidence where
available; missing diagnostics cannot prove a successful move. Silent and
diagnostic builds share input semantics, but diagnostic evidence is not available
in silent output. See the [current/legacy reference](REACTION_RULE_CHEATSHEET.md).
