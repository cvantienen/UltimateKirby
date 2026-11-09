# Sequence text-file syntax cheat sheet

[Open the searchable / printable HTML version](SEQUENCE_SYNTAX_CHEATSHEET.html).
Checked against the current source parser. These are controller inputs and
observation checks; they do not change native physics, hitboxes or move data.

## Most-used copy/paste examples

| Want to… | Paste into the sequence |
| --- | --- |
| Hold shield for exactly four input frames | `actions = shield:4` |
| Hold Grab for four input frames | `actions = press:r:4` |
| Wait four frames | `actions = wait:4` |
| Immediately cancel unless within this fighter's grab reach, then Grab | `actions = grab:fighter_range_instant` |
| Immediately require enemy within 20 horizontal units | `actions = check:enemy_within:20:0, press:r:4` |
| Turn away from opponent | `actions = face:away_from_opponent` |
| Up-B followed by 69 input frames of full opponent-directed drift | `actions = up_b:4, drift:opponent:1:69` |
| Run to 15 units inward from the opponent's ledge | `actions = run_to_ledge:15:1:1:6:180` |
| React to an enemy roll within 40 X / 12 Y | `reaction = auto_roll:40:12` |

**Important:** `grab:4` means **range-check 4 horizontal units, then send one
frame of Grab**. It does not hold Grab for four frames. Use `press:r:4`,
`hold:r:4` or `input:r:0:0:4` for a four-frame input hold.

## File layout and basic rules

| Location / syntax | Meaning |
| --- | --- |
| `deployment/ultimate/kirby_assist/fighters/bowser.txt` | Bowser-only sections; use the corresponding fighter file for each character |
| `deployment/ultimate/kirby_assist/sequences.txt` | Universal settings and generic editor templates |
| `assembled/sequences.txt` | Combined generated bundle; edit the source files above |
| `[bowser.neutral.my_option]` | `[FIGHTER.SLOT.UNIQUE_ID]`; use the fighter's file ID, such as `bowser`, `yoshi`, `lucina` |
| `[bowser.settings]` | Fighter-specific policy values; exactly one settings section per fighter |
| `[universal.settings]` | Shared startup settings; at most one |
| `[template.my_starter]` | Named inert Training-editor starter; not automatically executed |
| `# comment` | Whole-line or trailing comment; `#` starts a comment even inside a quoted name |
| `actions = ACTION, ACTION, ACTION` | One line, in execution order; no empty items or trailing comma |

Use lowercase canonical spelling and `true` / `false`. IDs use letters, digits
and underscores, up to 48 characters. Keep each fighter file owned by its named fighter. No runtime
include/import syntax exists; tooling assembles the files before upload.

**SLOT values:** `neutral`, `jump`, `roll`, `attack`, `drop`, `x`, `y`,
`dpad_up`, `dpad_down`, `dpad_left`, `dpad_right`, `l3`, `r3`.
Aliases: `neutral_getup`, `jump_getup`, `roll_getup`, `getup_attack`, `ledge_drop`,
`button_x`, `button-x`, `button_y`, `button-y`.
L/R/ZL/ZR (including LZ/RZ aliases) section bindings and old universal X/Y binding sections are retired
and ignored; they are not additional active button slots.

```ini
[bowser.neutral.example]
name = Example ledge response
default = false
start_delay = 0
timeout = 120
spatial_timeout = 60
ground_only = false
while_held = false
actions = shield:4, grab:fighter_range_instant
```

## Sequence properties

| Key | Accepted values / default | Meaning |
| --- | --- | --- |
| `name` / `label` | Optional text, 1–64 UTF-8 bytes; matching outer double quotes allowed | Menu label; avoid control characters and `#` |
| `start_delay` | **Required** for action recipes/templates; integer 0–3600 | Explicit initial delay; 0 begins immediately |
| `actions` | Required nonempty comma-separated action line | At most **32 expanded actions**; some aliases expand into several steps |
| `default` | `true` / `false`; false | Startup choice; at most one true per fighter/slot |
| `timeout` | Integer 1–3600; 3600 | Total native-progress sequence timeout |
| `spatial_timeout` | Integer 1–3600; 60 | Timeout for supported spatial/range steps |
| `ground_only` | `true` / `false`; false | Stop when airborne; automatic direct Up-B has its existing tracking exception |
| `while_held` | `true` / `false`; false | Supported held-button playback; Auto-react responses are one committed execution |
| `reaction` | Short or long form below; omit for no automatic trigger | Observation trigger and distance limits |
| `builtin` | `normal`, `short_hop`, `ledge_assist`, `turbo_teabag` | Built-in button role; no actions, reaction, loop, ground-only flag or nonzero delay |
| `mode`, `activation` | Compatibility-only; see final section | Not a current way to bind automatic helpers to gameplay buttons |

Do not repeat properties in one section. Builtins do not require `start_delay`;
if supplied it must be 0. The file limit is 512 KiB; the catalog limit is 2112
parsed sequence/template entries, including at most 64 generic templates. Menu capacity is bounded per fighter/slot;
validate the assembled file before uploading.
An editor Empty step has no text execution token; remove unused steps.

## Parameters, directions and button names

| Parameter | Meaning / valid values |
| --- | --- |
| `F` | Input-hold duration, normally integer 1–255. It is not move startup, recovery or guaranteed animation duration |
| `WAIT` | Integer 0–255; 0 checks immediately, positive values explicitly wait for a condition |
| `X`, `Y` / `LX`, `LY` | Left-stick axes −1 through 1; Y positive = up |
| `RX`, `RY` | Right-stick axes −1 through 1 |
| `S` | Stick strength, not a native speed change. Decimal `0.5` or percentage `50%`; each action's bounds are listed below |
| `UNITS`, `GAP`, `OFFSET` | Game-world distances, not frames or pixels |
| `DIR` | `opponent`, `away_from_opponent`, `facing`, `backward`, `ledge`, `inward`, `outward`, `saved_ledge`, `away_from_ledge` |
| `REF` | `screen`, `facing`, `opponent`; changes horizontal reference only |
| `BUTTONS` | `a`, `b`, `x`, `y`, `l`, `r`, `zl`, `zr`; combine with `+`, e.g. `a+x` |

Button aliases: `attack` = A; `special` = B; `jump` = X; `grab` = R;
`shield` = ZR; `none` / `0` = no buttons. These are the mod's canonical
controller inputs; your native controller mapping still matters.

Direction aliases: `enemy`, `toward`, `towards`, `toward_opponent`, `towards_opponent`, `toward_enemy`,
`towards_enemy` = `opponent`; `away`, `away_from_enemy` =
`away_from_opponent`; `forward` = `facing`; `back` = `backward`;
`toward_saved_ledge` = `saved_ledge`; `away_from_saved_ledge` = `away_from_ledge`.
Ledge-relative directions require an acquired ledge anchor and are rejected
inside onstage reaction recipes. Opponent/facing directions are suitable there.

## Literal inputs, holds, release and repeated presses

| Syntax | Example | Behavior |
| --- | --- | --- |
| `press:BUTTONS[:F]` | `actions = press:a+x:4` | Buttons with neutral stick; default F=1 |
| `hold:BUTTONS[:F]` | `actions = hold:zr:4` | Same timed input semantics as press; use explicit duration |
| `a[:F]`, `b[:F]`, `x[:F]`, `y[:F]`, `l[:F]`, `r[:F]`, `zl[:F]`, `zr[:F]` | `actions = zr:4` | Individual canonical button; default F=1 |
| `input:BUTTONS:X:Y:F` | `actions = input:r:0:0:4` | Absolute screen-relative left stick |
| `facing_input:BUTTONS:X:Y:F` | `actions = facing_input:a:1:0:4` | X relative to facing at step entry |
| `opponent_input:BUTTONS:X:Y:F` | `actions = opponent_input:a:1:0:4` | X relative to current target side at entry; needs a fresh target |
| `controller_input:BUTTONS:LX:LY:RX:RY:F[:LEFT_REF[:RIGHT_REF]]` | `actions = controller_input:a+b:0.5:0:-1:0:4:opponent:facing` | Both sticks plus buttons; omitted references are `screen` |
| `stick:X:Y:F` | `actions = stick:-0.5:0:4` | Left stick only, absolute screen direction |
| `cstick:X:Y:F` | `actions = cstick:0:1:4` | Right-stick input |
| `tap:BUTTONS:X:Y:HOLD:GAP:TOTAL` | `actions = tap:a:0:0:1:2:12` | Repeated pulses; HOLD/GAP 1–255, TOTAL 1–3600 |
| `wait[:F]` / `release[:F]` | `actions = wait:4` | Neutral/released sequence input; default 1; 0 is accepted as an immediate step |
| `hold_until_manual:BUTTONS:X:Y` | `actions = hold_until_manual:a:0:0` | Nonempty held buttons until manual takeover; no F parameter |
| `facing_hold_until_manual:BUTTONS:X:Y` | `actions = facing_hold_until_manual:a:1:0` | Same, with facing-relative X |
| `shield_until_manual` | `actions = shield_until_manual` | Grounded shield until manual takeover |
| `shield_until_block:F` | `actions = shield_until_block:90` | Hold shield for the hit/block handoff, bounded by F |

`wait_4`, `wait_4_frame`, `wait_4_frames` are aliases for `wait:4`.
Unlimited hold syntax is separate: `input:...:0` and `shield:0` are invalid.

## Attack, special, jump and dodge shortcuts

Unless stated otherwise, append `[:F[:S]]`; default F=1 and S=1, S range 0–1.
Directional shortcuts use facing-relative X. Native game state and input mapping
determine the actual move; these are not forced move commands.

| Canonical action and aliases | Example | Input / restriction |
| --- | --- | --- |
| `jab`, `attack` | `actions = jab:4` | Neutral A |
| `forward_tilt`, `f_tilt`, `ftilt` | `actions = forward_tilt:4` | A with 0.6 forward stick before S scaling |
| `up_tilt`, `u_tilt`, `utilt` | `actions = up_tilt:4` | A with 0.6 up stick |
| `down_tilt`, `d_tilt`, `dtilt` | `actions = down_tilt:4` | A with 0.6 down stick |
| `forward_smash`, `f_smash`, `fsmash` | `actions = forward_smash:4` | A with full forward stick |
| `up_smash`, `u_smash`, `usmash` | `actions = up_smash:4` | A with full up stick |
| `down_smash`, `d_smash`, `dsmash` | `actions = down_smash:4` | A with full down stick |
| `nair`, `neutral_air`, `neutral_aerial` | `actions = nair:4` | Neutral A; requires appropriate airborne state for Nair |
| `fair`, `forward_air`, `forward_aerial` | `actions = fair:4` | Forward A |
| `bair`, `back_air`, `back_aerial` | `actions = bair:4` | Backward A |
| `uair`, `up_air`, `up_aerial` | `actions = uair:4` | Up A |
| `dair`, `down_air`, `down_aerial` | `actions = dair:4` | Down A |
| `neutral_special`, `neutral_b`, `special` | `actions = neutral_special:4` | Neutral B |
| `side_special`, `side_b` | `actions = side_b:4` | Forward B |
| `up_special`, `up_b`, `final_cutter` | `actions = up_b:4` | Up B; native character/state determines the special |
| `down_special`, `down_b`, `stone` | `actions = down_b:4` | Down B |
| `short_hop`, `shorthop`, `jump` | `actions = short_hop:1` | X pulse; actual jump height depends on native input duration/state |
| `full_hop`, `fullhop` | `actions = full_hop:4` | Explicit F is required; the name alone is invalid |
| `shield` | `actions = shield:4` | ZR |
| `dash` | `actions = dash:1` | Full forward stick, no attack |
| `crouch`, `fast_fall` | `actions = fast_fall:4` | Down stick; fast-fall requires native aerial eligibility |
| `roll_forward` | `actions = roll_forward:4` | Shield + forward stick |
| `roll_backward` | `actions = roll_backward:4` | Shield + backward stick |
| `spot_dodge` | `actions = spot_dodge:4` | Shield + down stick |
| `air_dodge` | `actions = air_dodge:4` | Neutral shield input; requires appropriate airborne state |
| `dash_attack:DASH_F[:ATTACK_F]`, `dashattack` | `actions = dash_attack:1:4` | Two expanded steps: dash then forward A; default durations 1 / 1 |
| `short_hop_nair`, `short_hop_fair`, `short_hop_bair`, `short_hop_uair`, `short_hop_dair` | `actions = short_hop_bair:4` | Simultaneous X + directional A, not a separate delayed aerial |
| `short_hop_neutral_air`, `short_hop_forward_air`, `short_hop_back_air`, `short_hop_up_air`, `short_hop_down_air` | `actions = short_hop_neutral_air:4` | Long-name paired aliases; `shorthop_` prefix is also accepted |
| `turnaround_ACTION` / `turn_around_ACTION` | `actions = turnaround_short_hop_nair:4` | Prepends `face:backward` to a supported shortcut; consumes another expanded step |

Examples of strength scaling: `actions = side_special:4:50%`.
Use `press:a+x:F` or `controller_input` for explicit simultaneous combinations.
Turnaround prefixes belong to the shortcut family, not arbitrary tokens such as
`turnaround_walk`. Plain `grab` is the separate range syntax below.

## Facing, movement and range

| Syntax | Example | Limits / behavior |
| --- | --- | --- |
| `face:DIR` | `actions = face:away_from_opponent` | Instant if already facing correctly; otherwise inputs a turn until facing matches or spatial timeout |
| `face_DIR` | `actions = face_opponent` | Underscore alias for `face:DIR` |
| `walk:DIR:UNITS:S[:GAP]` | `actions = walk:opponent:8:50%:1` | UNITS 0–100; S 0.1–0.95; optional GAP 0–60 only with `opponent` |
| `run:DIR:UNITS` | `actions = run:opponent:8` | UNITS 0–100; distance completion, not a frame count |
| `drift:DIR:S:F`, `drift_for` | `actions = drift:opponent:1:69` | S 0–1; timed horizontal stick, not a distance target |
| `walk_to_opponent:S:F:GAP` | `actions = walk_to_opponent:50%:30:10` | S 0.1–0.95; GAP 0–60; stops on reaching the gap or duration |
| `drift_to_opponent:S:F:GAP` | `actions = drift_to_opponent:0.5:12:3` | Air approach; S 0.1–1; GAP 0–60 |
| `chase_if_far:GAP:S:F` | `actions = chase_if_far:10:1:14` | GAP 0–100, S 0–1; skip if already near, otherwise timed chase |
| `run_to_ledge:OFFSET:S[:TOLERANCE[:BRAKE[:MAX_F]]]` | `actions = run_to_ledge:15:1:1:6:180` | OFFSET 0–200 inward; S 0.3–1; tolerance 0–20 (default 1); brake 0–100 (default 6); MAX_F 1–3600 (default 180) |
| `range:ignore` | `actions = range:ignore, press:r:4` | No distance restriction |
| `range:N` | `actions = range:12, press:r:4` | N 0–60 horizontal units; can wait up to spatial timeout |
| `range:fighter_range` | `actions = range:fighter_range, press:r:4` | Can wait for this fighter's configured standing grab reach |
| `range:fighter_range_instant` | `actions = range:fighter_range_instant, press:r:4` | Fresh target required; cancel immediately if outside standing reach |
| `grab` / `grab:ignore` | `actions = grab` | One-frame R input, no range check |
| `grab:N` | `actions = grab:12` | Waiting horizontal range check (0–60), then one-frame R |
| `grab:fighter_range` | `actions = grab:fighter_range` | Waiting fighter-range check, then one-frame R |
| `grab:fighter_range_instant` | `actions = grab:fighter_range_instant` | Immediate fighter-range check, then one-frame R |

Grab aliases: `grab:no_range_check`, `grab:unverified`, `grab_no_range_check`
= unchecked; `grab:range_verified`, `grab:verified`, `grab_range_verified`
= waiting fighter-range check. Numeric/fighter-range checks do not themselves
prove vulnerability, front-facing geometry or native grab availability.

## Instant conditions and explicit condition waits

| Syntax | Example | Meaning |
| --- | --- | --- |
| `check:CONDITION[:WAIT]` | `actions = check:enemy_in_front:0, press:r:4` | Boolean condition; WAIT omitted = 0 |
| `check:enemy_within:DISTANCE[:WAIT]` | `actions = check:enemy_within:20:0, press:r:4` | Horizontal separation at most DISTANCE, 0–100 |
| `check:enemy_outside:DISTANCE[:WAIT]` | `actions = check:enemy_outside:20:0, shield:4` | Horizontal separation at least DISTANCE; 0–100 (inclusive) |
| `require:CONDITION[:WAIT]` | `actions = require:self_grounded:0, shield:4` | Alias for check; distance conditions keep the distance argument |

Boolean CONDITION values: `self_grounded`, `self_airborne`, `enemy_grounded`,
`enemy_airborne`, `enemy_shielding`, `enemy_not_shielding`, `enemy_in_front`,
`enemy_behind`. Distance conditions: `enemy_within`, `enemy_outside`.
With WAIT=0, failure cancels the remaining sequence immediately; success continues
in that observation. A positive WAIT deliberately waits up to that many native
frames. Example: `actions = check:enemy_shielding:8, press:r:4`.

## Facing conditionals: no turning or decision-wait frames

| Token | Meaning |
| --- | --- |
| `if_facing_opponent` | Execute the block when facing the observed opponent |
| `if_back_to_opponent` | Execute the block when facing away from the opponent |
| `if_facing_ledge` | Execute when facing the saved ledge at sequence start; needs a ledge anchor |
| `if_back_to_ledge` | Execute when facing away from the saved ledge at sequence start |
| `else_facing` | Optional opposite branch; at most one per block |
| `end_facing` | Required closing token |

```ini
[bowser.neutral.facing_example]
start_delay = 0
actions = if_facing_opponent, press:r:4, else_facing, up_b:4, end_facing
```

Blocks cannot nest. Opponent-facing conditions use the current opponent when
evaluated; ledge-facing conditions retain their sequence-start reference.
These choose a branch; `face:DIR` is the separate action that actually turns.

## Current automatic reactions

Use `reaction = EVENT:MAX_X:MAX_Y`. Distances are 0–100 world units.
MAX_Y is the absolute vertical difference **between the fighters**, not height
above stage. Equal-height platforms can therefore have a difference of 0;
MAX_Y=0 does not prove that either fighter is grounded.

| EVENT | Example | Controlled by / meaning |
| --- | --- | --- |
| `auto_roll` | `reaction = auto_roll:40:12` | Auto react; enemy forward/back roll |
| `auto_spot_dodge` | `reaction = auto_spot_dodge:35:12` | Auto react; enemy spot dodge |
| `auto_air_dodge` | `reaction = auto_air_dodge:40:24` | Auto react; enemy air dodge |
| `auto_grab` | `reaction = auto_grab:35:24` | Automatic grabs; unshielded approach requires shielding opponent |
| `auto_shield_grab` | `reaction = auto_shield_grab:25:10` | Automatic grabs; shielded front grab against vulnerable/grabbable target |
| `auto_grab_dodged` | `reaction = auto_grab_dodged:35:12` | Grab avoidance; follow-up after the assisted grab dodge |
| `auto_shield_backup` | `reaction = auto_shield_backup:25:15` | Automatic grabs; configured shield backup when the grab does not qualify |

```ini
[bowser.roll.auto_roll_example]
name = Roll punish example
start_delay = 0
timeout = 240
reaction = auto_roll:40:12
actions = up_b:4, drift:opponent:1:69
```

Only one recipe per automatic event per fighter. The slot stores the recipe;
the reaction field establishes its trigger. Other fighters need their own
roll/spot/air-dodge and grab-dodge follow-up recipes; no Bowser template is
implicitly inherited. A missing shield backup means no backup attack.
Ledge-trap activation disables ordinary automatic punishes. Safety, native
readiness, target validity and the single committed-input owner still apply.
Raw recipe inputs retain their native character/move restrictions.

## Legacy reaction grammar: accepted does not mean active

The current unified in-game controller uses the seven `auto_*` events above.
Older priority-rule tokens below remain parseable for compatibility and older
core consumers; they are **not executed by the unified Auto-react list**.
Do not use them to add a new current automatic punish.

```ini
[yoshi.attack.legacy_example]
start_delay = 0
reaction = shield_hit:grounded:0:23:10:front:1
actions = press:r:4
```

| Format / field | Meaning |
| --- | --- |
| `TRIGGER:STATE:MIN_X:MAX_X:MAX_Y:SIDE:PRIORITY[:CONTROL]` | Seven or eight colon-separated fields |
| `STATE` | `any`, `grounded`, `airborne`, `standing`, `walking`, `running`, `shielding` |
| `MIN_X`, `MAX_X`, `MAX_Y` | 0 ≤ MIN_X ≤ MAX_X ≤ 100; MAX_Y 0–100 |
| `SIDE` | `any`, `front`, `behind` |
| `PRIORITY` | Integer 1–8; lower first in legacy consumers |
| `CONTROL` | `menu`, `toggle_x`, `toggle_y`, `toggle_up`, `toggle_down`, `toggle_left`, `toggle_right` |

All retained TRIGGER tokens: `off`, `roll`, `roll_cross`, `roll_vulnerable`,
`roll_cross_vulnerable`, `shield_hit`, `shield_hit_vulnerable`, `grab_dodged`,
`grab_dodged_vulnerable`, `ledge_regrab`, `spot_dodge`, `spot_dodge_any`,
`enemy_shield`, `shield_roll`, `shield_roll_cross`, `shield_roll_vulnerable`,
`shield_roll_cross_vulnerable`, `shield_roll_away`, `shield_roll_away_vulnerable`,
`shield_pressure_vulnerable`.
`ledge_regrab` is inert compatibility syntax. To omit a trigger, omit `reaction`;
bare `reaction = off` is **not valid**. Its full compatibility form is
`reaction = off:grounded:0:0:0:any:1`.
The seven automatic events also parse in the long format, but normalize to their
short semantics: grounded admission, zero minimum, either side, Menu control and
priority 1. Extra legacy fields do not customize those automatic events.

## Generic Training templates and built-in button roles

```ini
[template.quick_shield]
name = Four-frame shield
start_delay = 0
actions = shield:4
```

Templates may contain actions, timings and `ground_only`. No active reaction, `default=true`,
mode, builtin or `while_held=true`. They appear in the Training editor's generic
template picker; choosing one copies its recipe into your fighter's sequence.

```ini
[yoshi.x.short_hop_button]
name = Short hop button
start_delay = 0
actions = short_hop:1
default = true
```

Builtins: `normal`, `ledge_assist`, `turbo_teabag`, `ledge_trump`. Use a recipe
action such as `short_hop:1` for short-hop input. Ledge trump starts only when
assigned to and pressed from a button slot.
Use only button slots X/Y/D-pad/L3/R3. Do not combine builtin with actions.

Compatibility-only mode values: `ledge_trap`, `auto_tech`,
`auto_mash`, `idle_crouch`, `tilt_turn`, `spot_punish`, `out_of_shield`.
The parser accepts `mode = VALUE` followed by `activation = hold` or
`activation = toggle`, with no actions/nonzero delay, only on supported fighter
button rows. Idle crouch is Kirby-only; tilt turn Kirby/Yoshi.
These legacy bindings are not the current assist ON/OFF controls. Use the
`builtin = ledge_trump` button action for a button-mapped ledge trump.

## Fighter-setting keys

Place under `[FIGHTER.settings]`; omit a key to retain its default. Numbers here
are ordinary decimal numbers, **not percentage strings**. Booleans use true/false.
Detailed meanings and fighter restrictions: [fighter settings](FIGHTER_SETTINGS.md).
Seven retired punish/roll keys still parse but do not tune current Auto-react
recipes; edit the corresponding `reaction = auto_*:X:Y` entry instead.

```ini
[yoshi.settings]
grab_dodge_default = true
out_of_shield_default = true
```

| Key | Values | Menu meaning / scope |
| --- | --- | --- |
| `walk_strength` | 0.1–0.5 | Walking stick % |
| `grab_range` | 1–60 | Grab reach estimate |
| `horizontal_range` | 1–150 | Ledge acquire X |
| `vertical_range` | 1–150 | Ledge acquire Y |
| `follow_range` | 1–150 | Ledge follow X |
| `follow_vertical_range` | 1–150 | Ledge follow Y |
| `stand_distance` | 1–150 | Ledge stand distance |
| `move_strength` | 0.1–1 | Auto-position stick % |
| `manual_spacing` | `true` / `false` | Manual walking cap |
| `auto_position` | `true` / `false` | Auto ledge stance |
| `use_range_limits` | `true` / `false` | Use acquire/follow limits |
| `grab_dodge_default` | `true` / `false` | Grab avoidance at startup |
| `out_of_shield_default` | `true` / `false` | Automatic grabs at startup |
| `grab_dodge_lookahead` | 1–8 | Grab dodge forecast frames |
| `shield_warning_x` | 0–100 | Shield: warning X |
| `shield_warning_y` | 0–100 | Shield: warning Y |
| `shield_body_half_width` | 0–40 | Shield: body half-width |
| `shield_body_height` | 0–60 | Shield: body height |
| `shield_body_half_depth` | 0–40 | Shield: body half-depth |
| `shield_forecast` | 0–10 | Shield: forecast frames |
| `shield_hold` | 0–20 | Shield: release-tail frames |
| `grab_dodge_padding` | 0–30 | Grab dodge: body padding |
| `grab_dodge_height` | 0–60 | Grab dodge: height tolerance |
| `tech_half_width` | 0–30 | Tech probes: half-width |
| `tech_floor_origin` | 0–40 | Tech probes: floor origin Y |
| `tech_wall_low` | 0–40 | Tech probes: low wall Y |
| `tech_wall_mid` | 0–40 | Tech probes: middle wall Y |
| `tech_wall_high` | 0–60 | Tech probes: high wall Y |
| `tech_ceiling_origin` | 0–60 | Tech probes: ceiling origin Y |
| `tech_floor_padding` | 0–15 | Tech probes: floor extra reach |
| `tech_ceiling_padding` | 0–15 | Tech probes: ceiling extra reach |
| `punish_down_reach` | 0–100 | Legacy punish: down-smash reach — retired; compatibility only |
| `punish_forward_reach` | 0–100 | Legacy punish: forward-smash reach — retired; compatibility only |
| `punish_height` | 0–60 | Legacy punish: height tolerance — retired; compatibility only |
| `roll_smash_height` | 0–60 | Legacy roll: smash height — retired; compatibility only |
| `roll_smash_watch` | 0–150 | Legacy roll: smash watch distance — retired; compatibility only |
| `roll_grab_height` | 0–60 | Legacy roll: grab height — retired; compatibility only |
| `roll_grab_watch` | 0–150 | Legacy roll: grab watch distance — retired; compatibility only |
| `crouch_range` | 0–150 | Kirby crouch: proximity X — Kirby only |
| `crouch_height` | 0–100 | Kirby crouch: proximity Y — Kirby only |
| `shield_scan_x` | 1–300 | Shield: scan X cutoff |
| `shield_scan_y` | 1–300 | Shield: scan Y cutoff |
| `auto_grab_close_range` | 0–150 | Automatic grab: shielded range — legacy fallback; 0 means unavailable/disabled |
| `auto_grab_dash_range` | 0–150 | Automatic grab: approach range — legacy fallback; 0 means unavailable/disabled |
| `reflect_default` | `true` / `false` | Auto-reflect at startup — explicit reflector configuration; currently Game & Watch only |
| `reflect_startup` | 6–30 | Reflect/absorb startup in frames; Game & Watch's native minimum is 6 |
| `reflect_margin` | 0–8 | Early activation allowance before predicted projectile contact |
| `reflect_max_hold` | 6–120 | Maximum native-progress frames holding one bucket response |
| `reflect_half_width` | 1–30 | Estimated body half-width used by projectile contact prediction |
| `reflect_height` | 1–60 | Estimated body height above the root used by prediction |
| `reflect_half_depth` | 1–30 | Estimated depth allowance used by prediction |
| `tilt_turn_down` | `true` / `false` | Aim down tilts; defaults ON for Kirby/Yoshi, OFF for others |
| `tilt_turn_forward` | `true` / `false` | Aim side C-stick tilts; requires Tilt Attack mapping; defaults OFF |
| `tilt_turn_up` | `true` / `false` | Aim up C-stick tilts; requires Tilt Attack mapping; defaults OFF |

## Universal startup keys

Place these under `[universal.settings]` in the main source file. These are
startup policy settings; they do not define fighter sequences or replace the
per-fighter Auto-react ranges. A parsed compatibility flag does not create an
active binding in the current menu.

```ini
[universal.settings]
enabled = true
control_layout = default
auto_tech = true
```

| Key(s) | Value syntax / accepted bounds |
| --- | --- |
| `enabled`, `auto_mash`, `auto_tech`, `takeover`, `tech_debug` | `true` / `false` |
| `autopilot_shield`, `down_tilt_trip_smash`, `kirby_fair1_smash` | `true` / `false`; helpers retain their fighter/scope restrictions |
| `auto_sdi`, `auto_spot_dodge`, `auto_di` | `true` / `false` |
| `control_layout` | `default` / `brother` |
| `auto_di_mode` | `survival`, `adaptive`, `escape`; current adaptive shares survival behavior |
| `auto_di_survival_speed` | Finite 0.1–30; retained legacy threshold, no longer switches adaptive modes |
| `player_index`, `npad_id` | Integer 0–7; aliases for selected physical controller |
| `tech_lookahead_frames` | Finite 1–5 |
| `tech_cooldown_frames` | Integer 0–65535 |
| `ledge_trap`, `ledge_drop_defense`, `ledge_distance_agnostic`, `ledge_manual_spacing`, `ledge_auto_position`, `ledge_throw_outward` | `true` / `false` |
| `ledge_trap_range`, `ledge_trap_vertical_range` | Finite 1–20 |
| `ledge_follow_range`, `ledge_follow_vertical_range` | Finite 1–100 |
| `ledge_walk_strength` | Finite 0.1–0.45 |
| `ledge_stand_distance` | Finite 2–15 |
| `ledge_move_strength` | Finite 0.1–1 |
| `ledge_grab_button` | `R`, `L`, `ZL` |
| `turbo_walk_strength` | Finite 0.05–0.6; horizontal creep amount for the selected Turbo t-bag built-in |
| `dpad_b_reverse`, `dpad_slide` | `true` / `false`; accepted for old-file compatibility but have no effect |
| `kirby_auto_duck` | `true` / `false`; accepted for old-file compatibility but has no effect |
Accepted legacy no-effect Turbo keys: `r3_turbo`, `l3_turbo`, `dpad_turbo` and
`turbo_phase_frames`. Boolean values are validated but cannot activate the
helper; `turbo_phase_frames` is range-checked to 1–10 but no longer changes its
cadence.

Other accepted **ignored** compatibility keys: `combo_enabled`, `combo_starter`,
`combo_max_start_percent`, `combo_grab_range`, `combo_fair_range`, `mash_interval_frames`, `alternate_mash`,
`mash_buttons_per_frame`, `ledge_nair_hit_lead_frames`,
`ledge_stone_hit_lead_frames`, `ledge_stone_startup_frames`,
`ledge_drop_request_timeout_frames`, `ledge_roll_hit_window_frames`,
`ledge_grab_buffer_frames`, `ledge_getup_attack`, `ledge_jump_response`,
`ledge_fair_range`, `ledge_fair_vertical_range`, `ledge_fair_delay_frames`,
`ledge_fair_lead_frames`, `ledge_fair_jump_squat_buffer`, `ledge_fair_buffer_frame`,
`ledge_roll_dash_range`, `ledge_roll_smash_range`, `ledge_roll_distance`,
`ledge_tilt_range`, `ledge_grab_range`, `ledge_anti_air_range`,
`ledge_anti_air_height`.
They do not configure a current automatic response. Unknown keys are rejected.

## Validate before uploading

```sh
python3 tools/sequence_validation.py deployment/ultimate/kirby_assist/fighters/bowser.txt
python3 tools/sequence_validation.py deployment/ultimate/kirby_assist/sequences.txt
```

The second command assembles and validates all fighter files for both builds.
These commands do not build the Switch mod or connect/upload to a Switch.
Parser validation cannot prove that a move will hit or that authored timing is
appropriate in a match.
