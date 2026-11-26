# Reaction-rule cheat sheet

**Legacy priority-builder reference.** The current unified Auto-react controller
uses six `auto_*:MAX_X:MAX_Y` events. `auto_grab` is retired/rejected; `ledge_regrab` is inert. Start with the
[current sequence syntax cheat sheet](SEQUENCE_SYNTAX_CHEATSHEET.md); accepted
legacy triggers below are not active in that unified controller.

The tables below describe retained legacy grammar/core/editor consumers, not current live controls. Legacy feedback labels are not proof that the unified live controller uses these conditions. [Open the styled, searchable table](REACTION_RULE_CHEATSHEET.html).

[Setup guide](REACTION_RULES.md) · [Editor actions](EDITOR_ACTION_CHEATSHEET.md)

## Current live event quick reference

| Event | Mode / scope | Example |
| --- | --- | --- |
| `auto_roll` | Auto react; opponent ordinary roll | `reaction = auto_roll:30:12` |
| `auto_spot_dodge` | Auto react; opponent spot dodge | `reaction = auto_spot_dodge:30:12` |
| `auto_air_dodge` | Auto react; opponent air dodge | `reaction = auto_air_dodge:40:20` |
| `auto_shield_grab` | Automatic grabs; you shield, target vulnerable/grabbable/front | `reaction = auto_shield_grab:25:10` |
| `auto_grab_dodged` | Grab avoidance; after confirmed assisted dodge recovery | `reaction = auto_grab_dodged:25:10` |
| `auto_shield_backup` | Automatic grabs; configured response when no legal grab wins | `reaction = auto_shield_backup:25:15` |

Each event has at most one saved sequence per fighter. X/Y are maximum absolute
root-position separations, 0–100 world units. No minimum distance, numeric
priority, own-state selector, enemy-side selector or button-toggle control is
offered in the simple editor. Those legacy fields are not current live arbitration.
Native grounded/readiness gates still apply. A check inside the selected tape
can cancel it but does not choose a different response for that consumed event.

Training: Minus → Assist modes → Auto react → L3/R3. Select → A/R3 edits;
+ Add → Left/Right chooses a missing event → A/R3/X creates its draft. The
Automatic grabs and Grab avoidance rows open subsets of that same catalog.
Y alone saves; wait for the result. B/L3 returns to the same assist row.
Start delays/holds/actions are edited normally; saving does not enable a mode.
Current live automatic grabs are shield-only; the fixed fallback is not an
unshielded `auto_grab` recipe. Missing custom follow-up/backup means no such
attack. [Complete current setup and handoff](REACTION_RULES.md).

```ini
[yoshi.roll.roll_starter]
name = Roll starter, not a supplied default
start_delay = 0
timeout = 180
reaction = auto_roll:30:12
actions = check:enemy_in_front:0, range:fighter_range_instant, press:r:4
```

This valid example requests Grab only if its in-tape checks pass. Native input
acceptance and reach still decide the outcome. It does not turn the mode ON.
Do not add it alongside another `auto_roll` for the same fighter.

## Legacy grammar below — not the unified live event editor

This is **grammar notation, not a valid standalone recipe**. Replace uppercase
placeholders and omit the notation brackets around an optional control; put the
result inside a complete section with `start_delay` and `actions`.

```ini
reaction = TRIGGER:STATE:MIN_X:MAX_X:MAX_HEIGHT:SIDE:PRIORITY[:CONTROL]
```

- Menu and button groups are independent. Recipes sharing a button toggle together. That button is reserved even while its group is OFF; normal button sequences are preserved and return after the assignment is removed and the button released.
- Events cover opponent shields/shield-origin rolls, ordinary rolls, confirmed shield hits, assisted grab-dodge recovery. Auto-tech and missed-tech branches remain separate. Defensive events use their own ON/OFF mode, not Roll rules or button groups.
- Priority compares conditions matching now. A plain roll rule can fire before a higher-priority rule becomes true later. Use roll_vulnerable and roll_cross_vulnerable together when comparing those outcomes at vulnerability.
- One response at a time; one attempt per enemy roll. Once playback starts, it does not switch to a different rule mid-response. A cancellation does not rearm that same roll.
- Retained legacy builders permit eight rules per assist family; current simple events additionally enforce one recipe per event/fighter. Ranges are world units, not hitbox reach. Every action is input-only under native rules.
- Requires the updated mod on every Switch loading reaction metadata. Older builds cannot parse the new property. This reference changes no Switch files and adds no runtime load.

Defensive startup switches and the 1–8 frame grab forecast are saved in **Fighter settings**. Defensive recipes must use Menu control; that field names their specific ON/OFF mode.

In the retained legacy editor: Assist modes → highlight the assist → L3/R3 opens its separate priority list. A/R3 edits, X adds, Y manages, B/L3 returns. In the editor, Y saves and R3/L3 returns to the list. Activate the assist with its ON/OFF row. Outside Training the switch is on Buttons / Assists, reached with R3 from the ledge selector.

## Editor settings

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Auto reaction | In a separate Assist rule editor, choose a roll variant or one of the retained Shield pressure events; other defensive assists keep their own event. Ordinary ledge/button editors do not expose this setting. | `Field 1 · TRIGGER` | `reaction = roll_vulnerable:grounded:0:20:12:any:1` |
| Stop if airborne | Opt-in throughout-playback safety; cancels if your situation becomes airborne. Different from My state, which filters startup. Existing recipes default OFF. | `ground_only = true` | `ground_only = true` |
| My state | Your state must match when the response starts. This does not force your fighter into that state. | `Field 2 · STATE` | `reaction = roll_vulnerable:walking:0:20:12:any:1` |
| Min X distance | Inclusive minimum absolute horizontal separation. World units, not frames or a percentage of the opponent’s hitbox. | `Field 3 · MIN_X · 0–100` | `reaction = roll_vulnerable:grounded:8:24:12:any:1` |
| Max X distance | Inclusive maximum horizontal separation. Must be at least Min X distance. | `Field 4 · MAX_X · MIN_X–100` | `reaction = roll_vulnerable:grounded:0:14:12:any:1` |
| Max height difference | Maximum absolute vertical separation. Horizontal and vertical limits are separate checks. | `Field 5 · MAX_HEIGHT · 0–100` | `reaction = roll_vulnerable:grounded:0:20:10:any:1` |
| Enemy side | Restrict the enemy’s current side relative to your facing. any includes either side and overlapping X positions. | `Field 6 · SIDE` | `reaction = roll_vulnerable:grounded:0:20:12:behind:1` |
| Priority | Lower numbers win among rules matching NOW. Equal values use category/file order; priority does not reserve a future response. | `Field 7 · PRIORITY · 1–8` | `reaction = roll_vulnerable:grounded:0:20:12:any:2` |
| Mode control | Choose Menu or one toggle button. Rules sharing a button toggle together; a reserved button replaces its ordinary jump/sequence input. | `Optional field 8 · CONTROL` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_x` |

## Trigger choices

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Auto reaction → off | File-only legacy setting: no automatic trigger. New assist rules keep their event family; delete a rule or switch its assist OFF to disable it. | `off` | `reaction = off:grounded:0:20:12:any:1` |
| Auto reaction → roll | Match during an observed ordinary forward/back roll when all other conditions hold. It can match after the first roll frame; it is not a ledge getup-roll trigger. | `roll` | `reaction = roll:grounded:0:20:12:any:1` |
| Auto reaction → roll_cross | Match after relative horizontal positions cross during this roll. Your movement can also create this crossing; it does not predict a future cross. | `roll_cross` | `reaction = roll_cross:grounded:0:20:12:any:1` |
| Auto reaction → roll_vulnerable | Wait until the native roll forecast indicates intangibility has ended, then check the remaining conditions. It does not time your move’s active hitbox automatically. | `roll_vulnerable` | `reaction = roll_vulnerable:grounded:0:20:12:any:1` |
| Auto reaction → roll_cross_vulnerable | Require BOTH a crossing during this roll and native roll vulnerability. Useful for comparing a crossed-roll response with other vulnerability-based rules. | `roll_cross_vulnerable` | `reaction = roll_cross_vulnerable:grounded:0:20:12:any:1` |
| Auto reaction → Spot dodge vulnerable | React once the opponent’s ordinary spot dodge is vulnerable and still recovering. Uses Spot punish ON/OFF and its own rule list. | `spot_dodge` | `reaction = spot_dodge:grounded:0:12:18:front:1` |
| Auto reaction → After shield hit | Run once after confirmed shield contact and native shieldstun, using Out of shield ON/OFF. | `shield_hit` | `reaction = shield_hit:grounded:0:12:18:any:1` |
| Auto reaction → After dodging grab | Follow the tracked grabber after native spot-dodge acceptance; run when recovery permits acting. Uses Grab dodge + punish ON/OFF. | `grab_dodged` | `reaction = grab_dodged:grounded:0:12:18:any:1` |
| Auto reaction → Enemy shielding | Enemy is raising/holding shield or taking shield damage; one response per shield visit. Uses the separate Shield pressure switch/list. | `enemy_shield` | `reaction = enemy_shield:grounded:0:20:12:any:1` |
| Auto reaction → Shield roll | Roll after an observed shield state. Uses the separate Shield pressure switch/list. | `shield_roll` | `reaction = shield_roll:grounded:0:20:12:any:1` |
| Auto reaction → Shield roll crosses | Shield-origin roll has crossed relative horizontal positions. Uses the separate Shield pressure switch/list. | `shield_roll_cross` | `reaction = shield_roll_cross:grounded:0:20:12:any:1` |
| Auto reaction → Shield roll vulnerable | Shield-origin roll with native vulnerability forecast at zero. Uses the separate Shield pressure switch/list. | `shield_roll_vulnerable` | `reaction = shield_roll_vulnerable:grounded:0:20:12:any:1` |
| Auto reaction → Shield cross vulnerable | Both crossing and vulnerability. Uses the separate Shield pressure switch/list. | `shield_roll_cross_vulnerable` | `reaction = shield_roll_cross_vulnerable:grounded:0:20:12:any:1` |
| Auto reaction → Shield roll away | Observed roll velocity moves away from your position at roll start. Uses the separate Shield pressure switch/list. | `shield_roll_away` | `reaction = shield_roll_away:grounded:0:20:12:any:1` |
| Auto reaction → Shield away vulnerable | Both rolling away and vulnerability. Uses the separate Shield pressure switch/list. | `shield_roll_away_vulnerable` | `reaction = shield_roll_away_vulnerable:grounded:0:20:12:any:1` |

[Shield pressure setup, cooperation and example recipes](SHIELD_PRESSURE.md).

## Additional event variants

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Shield / shield roll vulnerable | Combined pressure grab event: held shield or shield-origin roll, with native vulnerability. | `shield_pressure_vulnerable` | `reaction = shield_pressure_vulnerable:grounded:0:22:12:front:1` |
| After shield hit: enemy vulnerable | OOS event additionally requires the confirmed attacker’s native hittability. | `shield_hit_vulnerable` | `reaction = shield_hit_vulnerable:grounded:0:22:12:front:1` |
| After dodging grab: enemy vulnerable | After your assisted dodge recovery, requires the grabber’s native hittability. | `grab_dodged_vulnerable` | `reaction = grab_dodged_vulnerable:grounded:0:22:12:front:1` |
| Any enemy spot dodge | Can react before intangibility ends; does not change old `spot_dodge` rules. | `spot_dodge_any` | `reaction = spot_dodge_any:grounded:0:30:12:any:2` |

An immediate fallback can start before a future vulnerable-grab rule qualifies.

## Your state choices

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| My state → any | No additional own-state condition. Native input rules still determine whether the response can happen. | `any` | `reaction = roll_vulnerable:any:0:20:12:any:1` |
| My state → grounded | Your situation must be grounded; this includes more than idle standing. | `grounded` | `reaction = roll_vulnerable:grounded:0:20:12:any:1` |
| My state → airborne | Your situation must be airborne. The enemy can still be rolling on the ground below you. | `airborne` | `reaction = roll_vulnerable:airborne:0:20:12:any:1` |
| My state → standing | Grounded idle/WAIT specifically. Walking, crouching and attacking are not standing for this filter. | `standing` | `reaction = roll_vulnerable:standing:0:20:12:any:1` |
| My state → walking | Grounded WALK specifically. Walk braking is not included. | `walking` | `reaction = roll_vulnerable:walking:0:20:12:any:1` |
| My state → running | Grounded dash, run, turn-dash or turn-run. Ordinary turning and run braking are not included. | `running` | `reaction = roll_vulnerable:running:0:20:12:any:1` |
| My state → shielding | Grounded guard-on, guard, guard-off or shield-damage states. This is a state filter, not a shield-hit event trigger. | `shielding` | `reaction = roll_vulnerable:shielding:0:20:12:any:1` |

## Enemy side choices

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Enemy side → any | Accept either side, including almost overlapping X positions. | `any` | `reaction = roll_vulnerable:grounded:0:20:12:any:1` |
| Enemy side → front | Enemy must be in front relative to your current facing; near-zero horizontal separation does not count as front. | `front` | `reaction = roll_vulnerable:grounded:0:20:12:front:1` |
| Enemy side → behind | Enemy must be behind relative to your current facing; near-zero separation does not count as behind. | `behind` | `reaction = roll_vulnerable:grounded:0:20:12:behind:1` |

## Mode control choices

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Mode control → Menu | Follow the Roll rules Menu ON/OFF switch. This is the default for older seven-field rules. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `menu` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:menu` |
| Mode control → Toggle X | Fresh X presses toggle this group. X is reserved instead of jumping or playing its selected sequence. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_x` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_x` |
| Mode control → Toggle Y | Fresh Y presses toggle this group. Y is reserved instead of its ordinary action/sequence. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_y` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_y` |
| Mode control → Toggle D-pad Up | Fresh D-pad Up presses toggle this group with the menu closed. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_up` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_up` |
| Mode control → Toggle D-pad Down | Fresh D-pad Down presses toggle this group with the menu closed. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_down` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_down` |
| Mode control → Toggle D-pad Left | Fresh D-pad Left presses toggle this group with the menu closed. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_left` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_left` |
| Mode control → Toggle D-pad Right | Fresh D-pad Right presses toggle this group with the menu closed. Legacy button groups start OFF each match; holding does not repeat. Navigation/chords do not toggle. | `toggle_right` | `reaction = roll_vulnerable:grounded:0:20:12:any:1:toggle_right` |

## Response and timing

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Response actions | The recipe containing reaction metadata IS the response. It can be stored under any ledge/X/Y/D-pad category; it need not be the selected manual recipe. | `actions = ACTION, ACTION, ...` | `actions = face:opponent, grab` |
| Start delay | Delay the response after a rule matches. This is not a delay from the enemy roll’s first frame unless the rule actually matched there. | `start_delay = FRAMES` | `start_delay = 2` |
| Delay before an action | Pause before the following action by inserting wait. It is an editor setting, not another trigger condition. | `wait:FRAMES` | `actions = face:opponent, wait:3, forward_smash:4` |
| Hold buttons for | How long input is held. It does not define attack startup, active frames or guaranteed combo timing. | `ACTION:FRAMES or hold:BUTTONS:FRAMES` | `actions = down_smash:4` |
| Timeout | Overall response limit. Use a finite bound appropriate for the sequence. | `timeout = FRAMES` | `timeout = 180` |
| Spatial timeout | Bound for movement/range waits; missing target or movement safety can cancel sooner. | `spatial_timeout = FRAMES` | `spatial_timeout = 30` |
| Movement and facing | Use the existing actions. Onstage rules have no verified ledge anchor: use opponent, away_from_opponent, facing or backward. | `face:DIR · walk:DIR:UNITS:STRENGTH[:GAP]` | `actions = walk:opponent:4:50%:1, face:opponent, grab` |
| Range / checks inside response | Conditions inside actions still work. They are evaluated during playback, separately from the reaction conditions checked before starting. | `range:VALUE · check:CONDITION:WAIT` | `actions = range:fighter_range, face:opponent, grab` |

## Mode, save and lifetime

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| Roll rules → ON / OFF | Controls Menu-tagged recipes only. Button groups have independent states shown on this row. Legacy button groups start OFF; acquired ledge setups and native safety retain priority. | `Menu group: Assist modes. Button group: assigned button.` | `Training: Minus → Assist modes → Roll rules → ON` |
| Y Save | Save this recipe and its conditions immediately in Training using the existing single worker. Wait for the result before another save. No pending-save queue. | `Stored together in sequences.txt` | `reaction = roll_vulnerable:grounded:0:20:12:any:1`<br>`actions = face:opponent, grab` |
| Recipe name | Use the existing list manager to name each response. The label identifies the saved recipe, not a new event type. | `name = LABEL` | `name = Cross roll down smash` |
| Recipe category / identity | Choose a unique recipe ID within a fighter/category. Categories do not constrain which reaction condition can be attached. | `[FIGHTER.CATEGORY.ID]` | `[yoshi.roll.cross_smash_rule]` |
| Copy recipe as template | Copies actions/timings and preserves destination reaction conditions. It does not silently clone the source’s automatic rule. | `Destination reaction line remains unchanged` | `Copy source inputs → keep destination reaction conditions` |
| Duplicate recipe | The existing Duplicate command copies the recipe. Review its reaction conditions/priority and save deliberately; eight rules per assist family maximum. | `Unique new section with its own reaction line` | `[yoshi.roll.cross_smash_variant]` |
| Save as startup default | Not allowed for active assist rules: persistence rejects it. Use the mode ON/OFF control, not a manual startup default. | `default = true` is for manual recipes only | `No active reaction default` |
| Loop while held | Controls manual button activation only. Automatic reactions run a single pass regardless of this setting. | `while_held = true / false` | `while_held = false` |
| Match end / unsaved edits | Transient menu/rule state is discarded at confirmed match end. Saved recipe conditions survive. Save before leaving Training. | `Saved recipe persists; mode switch resets OFF` | `Save → wait for success → leave Training` |

## Feedback on Roll rules row

| Menu name / choice | Description | Corresponding file syntax | Example |
| --- | --- | --- | --- |
| watching rolls | No response is currently playing; the enabled mode is monitoring the known fighter roster. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| self state | A candidate did not match My state. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| outside range | A candidate was outside its inclusive Min/Max X distance. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| height | Vertical separation exceeded the configured height limit. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| enemy side | Front/behind did not match the configured side. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| waiting for crossing | The required relative X crossing has not been seen for this roll. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| waiting for vulnerability | The required native roll-vulnerability condition has not been reached. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| matched / last priority N | A rule with that priority started. Last matched priority remains visible after playback; it does not prove a hit. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| preempted | Input ownership/safety or another eligibility guard cancelled or withheld the response. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| target lost | A spatial part of the response no longer had a fresh locked target; playback was cancelled. | `Display only — not a file setting` | `No reaction property is written for feedback` |
| finished | The response ended. There is no second attempt for the same roll. This is not hit confirmation. | `Display only — not a file setting` | `No reaction property is written for feedback` |

## Complete legacy Yoshi examples (parseable, not live unified starters)

These examples document legacy consumers only. Use the simple event example above for current live behavior; these ranges/delays are not calibrated guarantees. If more than one rule matches now, the lower priority number wins.

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

For ledge-relative movement direction settings and facing-dependent blocks, see the [editor movement direction table](EDITOR_ACTION_CHEATSHEET.md#movement-direction-choices). These require a ledge anchor and are not available to on-stage roll reaction rules.

Vulnerable-ledge/regrab punishment is retired. The current Auto react mode uses one maximum X/Y range and one committed sequence for each of roll, spot dodge and air dodge; legacy priority vocabulary above does not order these three responses. See [current behavior](REACTION_RULES.md#current-simple-auto-react-mode).
