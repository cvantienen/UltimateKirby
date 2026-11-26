# Mr. Game & Watch

## Configure Mr. Game & Watch

This page belongs to profile **`mr_game_and_watch`** (`fighter_kinds::MR_GAME_AND_WATCH`), not a
roster-position index. Use `[mr_game_and_watch.settings]` for overrides and
`[mr_game_and_watch.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
Existing choices, supplied defaults and literal inputs are shown in the generated
snapshot below. Your installed file and unsaved Training draft may differ.

Start with [the complete fighter configuration walkthrough](../FIGHTER_CONFIGS.md),
then use [all 83 tuning keys](../FIGHTER_SETTINGS.md) and
[execution/calibration](../FIGHTER_EXECUTION.md). These shared guides explain
units, ranges, current versus startup effects, save/restart, and cancellation.
Choose or edit this fighter's row in Training; activate its configured Ledge
Assist binding to run it. A missing selected timeline produces no fallback move.

### Supplied fighter overrides

These are the current repository file values, **not schema fallbacks**.
Omitted keys retain resolved shared/reference defaults; retired fields remain
parseable but do not restore removed behavior.

| Key | Supplied value |
| --- | --- |
| `reflect_default` | `true` |
| `reflect_startup` | `15` |
| `reflect_margin` | `8` |
| `reflect_max_hold` | `45` |
| `reflect_half_width` | `15` |
| `reflect_height` | `16` |
| `reflect_half_depth` | `15` |

### Support and extension boundary

The standalone projectile-defense adapter is Game & Watch-only. It requires explicit reflect configuration and a bucket gauge below 3. Mapped Down+B relinquishes input while awaiting native startup; bucket confirmation ends ownership. It never empties oil automatically. New projectile attack generations can rearm even when object IDs are reused.

To expand this fighter, add a uniquely named option under **`mr_game_and_watch`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **28** (`mr_game_and_watch`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | HAM; **Forward tilt**; Up Smash; Torch f-smash |
| Jump | **Nair**; Foward tilt |
| Roll | **Bury hammers**; Grab |
| Attack | Shield GRAB; **Shield > Up-B** |
| Drop | **Shield**; beans; Bury hammers |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `mr_game_and_watch.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `mr_game_and_watch.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `mr_game_and_watch.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `mr_game_and_watch.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `mr_game_and_watch.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `mr_game_and_watch.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral Air | 7/12/17/22 start; active 7–10 / 12–15 / 17–20 / 22–23; 42 total; 7 landing |
| Back Air | 10/14/18/22 start; active 10–13 / 14–17 / 18–21 / 22 / 1; 39 total; 18 landing |
| Up Air | 9/15/21/27/33/37 start; active 9 / 15 / 21 / 27 / 33 / 37–41; 42 total; 12 landing |
| Down Air | 12 start; active 12–13 / 14–38 / 1–2; 49 total; 22 landing |
| Forward Tilt | 8 start; active 8–9 / 10–20; 37 total; -- landing |
| Up Tilt | 10/20 start; active 10–12 / 20–22; 33 total; -- landing |
| Down Tilt | 6 start; active 6–8; 42 total; -- landing |
| Dash Attack | 6 start; active 6–9 / 10–19; 38 total; -- landing |
| Forward Smash | 17 start; active 17–18; 42 total; -- landing |
| Up Smash | 21 start; active 21–25; 38 total; -- landing |
| Down Smash | 12 start; active 12–16; 37 total; -- landing |

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
| Weight | 75 |
| Walk speed | 1.18 |
| Initial dash | 1.98 |
| Run speed | 1.679 |
| Air speed | 1.176 |
| Gravity | 0.08 |
| Fall speed | 1.24 |
| Fast fall | 1.984 |
| Short-hop height | 13.26 |
| Full-hop height | 27.51 |

## Execution and calibration

Option definitions and input delays come from `fighters/mr_game_and_watch.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:b:1:0:4` |
| Neutral / `forward_tilt` | 16 | `opponent_input:a:0.6:0:1` |
| Neutral / `up_tilt` | 0 | `walk_to_opponent:0.8:10:0, face:away_from_opponent, opponent_input:a:0:1:10, wait:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:22, wait:1` |
| Jump / `nair` | 0 | `opponent_input:x:0:0:1, facing_input:a:0:0:1, drift:opponent:0.8:4` |
| Jump / `up_tilt` | 0 | `opponent_input:a:0.6:0:1` |
| Roll / `down_smash` | 0 | `walk:away_from_opponent:5:0.85, opponent_input:a:0:-1:16, wait:1` |
| Roll / `grab` | 0 | `wait:10, drift:away_from_opponent:0.75:10, range:fighter_range, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `wait:10, facing_input:zr:0:0:30, facing_input:r:0:0:4` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield_until_block:90, up_special:1` |
| Drop / `down_tilt_shield` | 0 | `face:opponent, hold_until_manual:b:0:0` |
| Drop / `down_smash` | 0 | `face:opponent, down_smash:1` |

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
