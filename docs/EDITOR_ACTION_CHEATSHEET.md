# Editor action cheat sheet

For the complete current text-file grammar, aliases, settings and copy/paste
examples, use the [sequence syntax cheat sheet](SEQUENCE_SYNTAX_CHEATSHEET.md).

For automatic mode conditions and triggers, see the
[reaction-rule cheat sheet](REACTION_RULE_CHEATSHEET.md) and its
[styled searchable table](REACTION_RULE_CHEATSHEET.html).

All **52 unique fixed editor entries**, plus dynamic saved inputs and timing/activation settings. [Open the styled, searchable table](EDITOR_ACTION_CHEATSHEET.html).

All actions simulate inputs/check observations; they do not edit physics, hitboxes or move data. FRAMES is input-hold duration, not move startup or animation length. Every example uses explicit hold duration; do not assume a :1 is replaced by the editor’s 4-frame initial value. Strength accepts decimals (0.5) or percentages (50%) and is stick magnitude, not guaranteed movement speed. X/Y are left-stick axes except cstick, which is the right stick. +X is right for input, forward relative to step-start facing for facing_input; +Y is up. DIR: opponent, away_from_opponent, ledge, inward, outward, facing, backward, saved_ledge, away_from_ledge. Ledge directions need a usable ledge anchor. Check WAIT=0 requires the condition immediately; positive WAIT waits up to that many frames, then ends the remaining sequence on failure. Movement/range steps also respect spatial_timeout and safety checks.

Duplicate input aliases have been consolidated: Forward A covers Fair/Fsmash (and an A input while already dashing), Up A covers Uair/Usmash, Down A covers Dair/Dsmash, and Neutral A covers jab/Nair/continuous A hold. The picker has one category per fixed action. To build a dash attack, choose **Dash** with a 1-frame duration, then **Forward A**; a simultaneous full-stick+A input alone does not insert a dash first. Native state determines the resulting move.

**Controller input** combines any of A/B/X/Y/L/R/ZL/ZR and both sticks in one frame. Select **Buttons → A** to open eight independent toggles (A or Left/Right toggles; B returns). Set left and right X references independently to **screen**, **facing**, or **opponent**, then configure both sticks and hold frames. Opponent references require an observed target; their horizontal direction is captured at step entry. A facing change/crossing during that hold does not flip the input. Vertical axes remain screen up/down.

The picker no longer repeats Y/L/ZL or specific A+B/X+B shortcuts: use the button toggles. Until-manual hold is a setting on ordinary button inputs, not another duplicate shield action. Legacy `range:ignore`, `shield_until_manual` and ledge fallback directions remain loadable in saved recipes but are not new fixed picker choices. Nothing rewrites your existing actions or durations.

Examples are `actions = ...` lines inside a valid section. Replace the fighter/category/ID for your recipe:

```ini
[kirby.neutral.example]
name = Example
start_delay = 0
timeout = 3600
spatial_timeout = 60
default = false
actions = face:opponent, wait:2, down_tilt:4
```

## Attacks / aerials / tilts

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Neutral A / Nair | Neutral A input: jab while grounded, neutral aerial while airborne. Holds A for the chosen duration. | `nair:FRAMES` | `actions = nair:4` |
| Forward A (Fair / Fsmash) | A + full forward stick, relative to starting facing. Ground/air state and native input rules determine the move. | `fair:FRAMES` | `actions = fair:4` |
| Up A (Uair / Usmash) | A + full upward stick. Intended for up aerial airborne or up smash grounded. | `uair:FRAMES` | `actions = uair:4` |
| Down A (Dair / Dsmash) | A + full downward stick. Intended for down aerial airborne or down smash grounded. | `dair:FRAMES` | `actions = dair:4` |
| Ftilt | A + 60% forward left-stick input, relative to starting facing. Intended as forward tilt. | `forward_tilt:FRAMES` | `actions = forward_tilt:4` |
| Utilt | A + 60% up left-stick input. Intended as up tilt. | `up_tilt:FRAMES` | `actions = up_tilt:4` |
| Dtilt | A + 60% down left-stick input. Intended as down tilt. | `down_tilt:FRAMES` | `actions = down_tilt:4` |
| C-stick | Sends right-stick X/Y for FRAMES. This is the right stick; all ordinary input/stick actions use the left stick. | `cstick:X:Y:FRAMES` | `actions = cstick:0:-0.6:4` |
| Bair | A + full backward stick, relative to starting facing. Intended for back air when airborne; it does not jump first. | `bair:FRAMES` | `actions = bair:4` |

## Jumps / jump attacks

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Short hop | New selections use a brief 1-frame X pulse. Existing saved X durations remain unchanged. Native jump rules apply. | `short_hop:FRAMES` | `actions = short_hop:1` |
| Full hop hold | New selections hold X for 4 frames. Choosing Short/Full hop uses that preset’s pulse rather than inheriting another move’s hold; height still follows native rules. | `full_hop:FRAMES` | `actions = full_hop:4` |
| Short hop Nair | Sends X+A together with a neutral stick for the paired short-hop attack. | `short_hop_nair:FRAMES` | `actions = short_hop_nair:4` |
| Short hop Fair | X+A with full forward stick in the same input step; forward is relative to starting facing. | `short_hop_fair:FRAMES` | `actions = short_hop_fair:4` |
| Short hop Bair | X+A with full backward stick in the same input step. | `short_hop_bair:FRAMES` | `actions = short_hop_bair:4` |
| Short hop Uair | X+A with full up stick in the same input step. | `short_hop_uair:FRAMES` | `actions = short_hop_uair:4` |
| Short hop Dair | X+A with full down stick in the same input step. | `short_hop_dair:FRAMES` | `actions = short_hop_dair:4` |

## Specials

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Neutral B | B with a neutral left stick. Grounded/airborne behavior comes from the fighter. | `neutral_b:FRAMES` | `actions = neutral_b:4` |
| Side B | B + full forward stick, relative to starting facing. | `side_b:FRAMES` | `actions = side_b:4` |
| Up B | B + full up stick. No grounded-only gate is added by this input step. | `up_b:FRAMES` | `actions = up_b:4` |
| Down B | B + full down stick. | `down_b:FRAMES` | `actions = down_b:4` |

## Shield / dodge / grab

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Shield / air dodge | Shield input (ZR). The native situation decides shield or air dodge; this is not an automatic threat check. | `shield:FRAMES` | `actions = shield:4` |
| Grab / Zair | Canonical R grab input with neutral stick. The native situation decides grounded grab or aerial tether/other behavior. | `facing_input:BUTTONS:X:Y:FRAMES` | `actions = facing_input:r:0:0:4` |
| Roll forward | Shield + full forward stick, relative to starting facing. Native grounded eligibility is required for a roll. | `roll_forward:FRAMES` | `actions = roll_forward:4` |
| Roll backward | Shield + full backward stick, relative to starting facing. | `roll_backward:FRAMES` | `actions = roll_backward:4` |
| Spot dodge | Shield + full down stick. Native state decides whether spot dodge can begin. | `spot_dodge:FRAMES` | `actions = spot_dodge:4` |

## Movement / facing

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Dash | Full forward left stick without an attack button, relative to starting facing. | `dash:FRAMES` | `actions = dash:4` |
| Crouch / fast fall | Full downward left stick without a button. Crouch on the ground; fast fall only when native aerial conditions permit. | `crouch:FRAMES` | `actions = crouch:4` |
| Face direction | Faces the chosen direction/target. This step uses facing checks; it is not a timed walking step. | `face:DIR` | `actions = face:opponent` |
| Walk to opponent | Walks toward the opponent for at most FRAMES, stopping at the requested horizontal GAP. Strength is analog input, not a speed percentage. | `walk_to_opponent:STRENGTH:FRAMES:GAP` | `actions = walk_to_opponent:0.8:30:10` |
| Air approach | Timed horizontal air approach toward the opponent, stopping at GAP when reached. | `drift_to_opponent:STRENGTH:FRAMES:GAP` | `actions = drift_to_opponent:0.5:12:3` |
| Walk distance | Walks UNITS from the step starting position in DIR at STRENGTH. Optional GAP applies only to opponent direction. Units are distance, not frames. | `walk:DIR:UNITS:STRENGTH[:GAP]` | `actions = walk:opponent:8:50%:1` |
| Run distance | Uses running movement in DIR until UNITS of horizontal displacement. Distance is not a fixed number of frames. | `run:DIR:UNITS` | `actions = run:ledge:8` |
| Drift | Holds a horizontal analog direction for FRAMES. It is a timed stick action, not a distance target. | `drift:DIR:STRENGTH:FRAMES` | `actions = drift:opponent:25%:20` |

## Buttons / combinations

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Controller input | Simultaneous button combination, left stick and C-stick. Canonical gameplay buttons only; no menu/click/taunt recursion. Timed hold 1–255 frames. References independently control each stick’s horizontal sign. | `controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]` | `actions = controller_input:a+b:0.5:0:-1:0:4:opponent:facing` |
| Empty | Unused editor step. Remove it before saving; there is no Empty execution token. | `No file token — remove step` | `actions = down_tilt:4  # Empty removed` |

## Conditions / checks

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| If facing ledge | Run the block only if facing the saved ledge at sequence start. Close with End facing block. | `if_facing_ledge, ..., end_facing` | `actions = if_facing_ledge, grab:4, end_facing` |
| If back to ledge | Run the block only if facing away from the saved ledge at sequence start. | `if_back_to_ledge, ..., end_facing` | `actions = if_back_to_ledge, short_hop_nair:4, end_facing` |
| End facing block | Ends a facing block; adds no input or wait. Blocks cannot nest. | `end_facing` | `actions = if_facing_ledge, grab:4, end_facing` |
| Wait for shield hit | Holds shield until a block is observed and shield recovers. Ends the sequence if the FRAMES limit expires. | `shield_until_block:FRAMES` | `actions = shield_until_block:90, nair:4` |
| Check opponent distance | Waits until the opponent is within RANGE horizontally, subject to spatial_timeout; sends no attack itself. | `range:RANGE` | `actions = range:10, grab` |
| Check fighter grab reach | Waits until within this fighter’s configured standing-grab range. Does not press grab or guarantee a hit. | `range:fighter_range` | `actions = range:fighter_range, grab` |
| Check fighter grab reach instantly | Checks current horizontal standing-grab reach once; cancels the remaining recipe immediately if too far or the target is unavailable. Add Grab / Zair next to grab. | `range:fighter_range_instant` | `actions = range:fighter_range_instant, grab` |
| Check: I am grounded | Continue only while the controlled fighter is grounded. WAIT optionally waits for the condition before ending the sequence. | `check:self_grounded:WAIT` | `actions = check:self_grounded:0, down_tilt:4` |
| Check: I am airborne | Continue only while the controlled fighter is airborne. | `check:self_airborne:WAIT` | `actions = check:self_airborne:0, down_tilt:4` |
| Check: enemy grounded | Continue only when the observed enemy is grounded. | `check:enemy_grounded:WAIT` | `actions = check:enemy_grounded:0, down_tilt:4` |
| Check: enemy airborne | Continue only when the observed enemy is airborne. | `check:enemy_airborne:WAIT` | `actions = check:enemy_airborne:0, down_tilt:4` |
| Check: enemy shielding | Continue only when the observed enemy is shielding. | `check:enemy_shielding:WAIT` | `actions = check:enemy_shielding:0, down_tilt:4` |
| Check: enemy not shielding | Continue only when the observed enemy is not shielding. | `check:enemy_not_shielding:WAIT` | `actions = check:enemy_not_shielding:0, down_tilt:4` |
| Check: enemy in front | Continue only when the observed enemy is in front relative to your facing. | `check:enemy_in_front:WAIT` | `actions = check:enemy_in_front:0, down_tilt:4` |
| Check: enemy behind | Continue only when the observed enemy is behind relative to your facing. | `check:enemy_behind:WAIT` | `actions = check:enemy_behind:0, down_tilt:4` |
| Check: enemy X distance within | Continue only when absolute horizontal distance is at most RANGE. WAIT is the maximum condition wait. | `check:enemy_within:RANGE:WAIT` | `actions = check:enemy_within:10:0, down_tilt:4` |
| Check: enemy X distance outside | Continue only when absolute horizontal distance is at least RANGE. This does not measure diagonal/vertical distance. | `check:enemy_outside:RANGE:WAIT` | `actions = check:enemy_outside:10:0, down_tilt:4` |

## Repeated taps

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Repeated taps | Pulses A, B, forward tilt, up tilt or down tilt. HOLD is press time, GAP is released time, TOTAL is the overall frame budget. | `tap:BUTTONS:X:Y:HOLD:GAP:TOTAL` | `actions = tap:a:0:-0.6:1:11:120` |

## Saved inputs

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Opponent-relative input (saved input) | Literal buttons plus left stick: +X toward the observed opponent, −X away. Captures direction when each step starts; cancels if the target is unavailable. No facing-wait or move-readiness check. Imported recipes retain editable X/Y and duration. | `opponent_input:BUTTONS:X:Y:FRAMES` | `actions = opponent_input:b:-1:0:1, opponent_input:none:1:0:2` |
| Saved inputs (dynamic group) | Actions imported from the recipe you opened, with descriptive input/check labels. This list varies per recipe; it is not another fixed universal move. | `Original action token; commonly input:BUTTONS:X:Y:FRAMES or facing_input:BUTTONS:X:Y:FRAMES` | `actions = input:a+zr:0:0:4` |

## Input settings

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Buttons | A opens eight independent canonical button toggles. Works on regular, opponent-relative and controller inputs. No buttons means stick-only input. | `BUTTONS = a+b+x+y+l+r+zl+zr` or `none` | `actions = controller_input:a+zr:0.2:0:0:0:4` |
| Left stick X reference | Screen: +X right; Facing: +X forward at step start; Opponent: +X toward the observed enemy. Changing reference retains buttons/axes/hold, except until-manual inputs switch to a finite 4-frame hold when converted to opponent-relative. | `input`, `facing_input`, `opponent_input`; or `LEFT_REF` for controller input | `actions = opponent_input:b:1:0:4` |
| Right stick X reference | Independent screen/facing/opponent sign for a Controller input’s C-stick. | `RIGHT_REF` | `actions = controller_input:none:0.5:0:-1:0:4:opponent:facing` |
| Right stick X / Y | Controller input’s simultaneous C-stick axes. Left X/Y remain independently editable. | `RX:RY` | `actions = controller_input:none:0.5:0:0:1:4` |

Aerial drifting + back-air input, for example, can use the last facing-relative C-stick example; it does not jump for you. Diagonal air dodge can use `check:self_airborne:0, controller_input:zr:0.7:0.7:0:0:4:screen:screen`. These are physical input combinations, not guaranteed native moves or hit confirms.

## Timing / activation settings

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Wait BEFORE action (setting) | Per-step delay, not a selectable Wait move. In the file, insert wait before that action. | `wait:FRAMES` | `actions = short_hop:4, wait:5, nair:4` |
| Start delay (setting) | Wait before the entire sequence. This is a section property, not an actions token. | `start_delay = FRAMES` | `start_delay = 4` |
| Loop while held (button setting) | Repeats the whole button recipe while its physical binding is held. Only button recipes support this. | `while_held = true / false` | `while_held = true`<br>`actions = down_tilt:4, wait:8` |
| Timeout / spatial timeout (properties) | Overall sequence limit and the bound for movement/range waits. These do not change the fighter’s move startup. | `timeout = FRAMES; spatial_timeout = FRAMES` | `timeout = 3600`<br>`spatial_timeout = 60` |
| Startup default (saved choice) | Select this saved recipe when the next match loads this fighter/category. Keep at most one true default per category. | `default = true` | `default = true` |

Built from the actual editor presets and checked against the Rust parser/serializer. Spatial and input actions still follow native game eligibility; these examples do not guarantee a hit or a legal tech.

## Automatic reaction settings

These are recipe properties below the action list, not extra move choices.
See [reaction rules](REACTION_RULES.md) for all seven fields and priorities.

| Menu name | Description | File syntax | File example |
| --- | --- | --- | --- |
| Stop if airborne | Optional sequence safety switch, exposed for assist recipes. Stops pending/held inputs after leaving the ground. Defaults OFF so existing jump recipes are unchanged. | `ground_only = true` | `ground_only = true` followed by `actions = check:self_grounded:0, up_b:3` |
| Auto reaction / My state / Min-max X distance / Height / Enemy side / Priority | Available in separate Assist rule editors, not ordinary ledge/button recipes. Roll variants choose their condition; shield-hit, grab-dodged and ledge-regrab rules keep their own event family. Each assist has its ON/OFF control. | `reaction = TRIGGER:STATE:MIN:MAX:HEIGHT:SIDE:PRIORITY` | `reaction = roll_cross_vulnerable:grounded:0:14:10:any:1` |

Use movement **Direction → Toward saved ledge / Away from saved ledge** for
explicit anchor-only movement. `walk:away_from_ledge:4:50%` moves four world units
away; `run:saved_ledge:8` runs eight units toward it;
`drift:away_from_ledge:25%:20` sends that stick direction for twenty frames.
No known anchor means these new actions stop the sequence; legacy `ledge`
retains its existing opponent fallback. Facing blocks use sequence-start facing,
not the opponent's side, and cannot nest. Only the selected block's waits/inputs
consume time. They need a known ledge; on-stage reaction rules cannot use them.
See [facing-dependent recipes](SEQUENCE_FILES.md#facing-dependent-ledge-recipes).

## Movement direction choices

These are the **Direction** settings on Face, Walk distance, Run distance and Drift, not separate move-picker entries. Walk/Run distances are world units; Drift duration is frames.

| Menu direction | Meaning | DIR token | Example |
| --- | --- | --- | --- |
| Toward saved ledge | Toward the retained ledge anchor. Ends the sequence if no anchor is known. | `saved_ledge` | `actions = drift:saved_ledge:25%:20` |
| Away from saved ledge | Away from the retained ledge anchor. Ends the sequence if no anchor is known. | `away_from_ledge` | `actions = walk:away_from_ledge:4:50%` |
| Opponent | Toward the observed opponent. | `opponent` | `actions = walk:opponent:4:50%` |
| Away from opponent | Away from the observed opponent. | `away_from_opponent` | `actions = drift:away_from_opponent:25%:20` |
| Ledge (legacy saved/file input only) | Toward the ledge, with the existing opponent-position fallback if no anchor is known. | `ledge` | `actions = run:ledge:8` |
| Inward | Along the retained inward stage direction. | `inward` | `actions = walk:inward:4:50%` |
| Outward | Opposite the retained inward stage direction. | `outward` | `actions = drift:outward:25%:20` |
| Facing | Along the fighter’s facing direction. | `facing` | `actions = walk:facing:4:50%` |
| Backward | Opposite facing; backward drift retains its step-start direction. | `backward` | `actions = drift:backward:25%:20` |
