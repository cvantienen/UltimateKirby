# Kirby

## Configure Kirby

This page belongs to profile **`kirby`** (`fighter_kinds::KIRBY`), not a
roster-position index. Use `[kirby.settings]` for overrides and
`[kirby.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
Existing choices, supplied defaults and literal inputs are shown in the generated
snapshot below. Your installed file and unsaved Training draft may differ.

Start with [the complete fighter configuration walkthrough](../FIGHTER_CONFIGS.md),
then use [all 83 tuning keys](../FIGHTER_SETTINGS.md) and
[execution/calibration](../FIGHTER_EXECUTION.md). These shared guides explain
units, ranges, current versus startup effects, save/restart, and cancellation.
Choose or edit this fighter's row in Training; activate its configured Ledge
Assist binding to run it. A missing selected timeline produces no fallback move.

### Supplied fighter overrides

No `[fighter.settings]` overrides are supplied in this fighter file. Shared
settings and exact-kind reference facts remain the baseline; this does not mean
every assist is active or every recipe is populated.

### Support and extension boundary

Kirby alone has the separate down-tilt-trip and first-hit-fair follow-ups, plus optional idle crouch. These are not automatic ledge recipe steps. Use the linked follow-up guides to configure their switches; a Nair label does not imply the retired walking/nudge engine.

To expand this fighter, add a uniquely named option under **`kirby`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **6** (`kirby`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Rapid Jab; Down smash; Down-B / Stone; Inhale; Bair; Cross-up grab; NAIR LATE; Immediate Nair; Immediate Nair Max Drift; Dash Attack; [TEST] Face ledge > walk 5 > grab; [TEST] Face foe > run 10 > grab; [TEST] Full-hop Bair + drift; [TEST] Short-hop Nair + delay/drift; [TEST] Turnaround short-hop Bair; [TEST] Grab (verify profile range); [TEST] Grab (ignore range); [TEST] Grab (verify 20 units); [TEST] Grounded down smash; [TEST] Grounded down special |
| Jump | **Nair**; Inhale; BAIR; HALF NAIR |
| Roll | **Down smash**; Grab; Dash Attack |
| Attack | **Grab / rear nair**; Grab / rear shield |
| Drop | **Shield**; Down tilt > shield; Short-hop dair; jump UP B; UP B |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
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
| Neutral / `nair` | 0 | `facing_input:x:0:0:1, drift:opponent:0.6:5, controller_input:a:0:0:0:0:4:screen:screen, wait:21, facing_input:none:0:-1:4` |
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
| Jump / `bair` | 0 | `opponent_input:b:0.36:0:4` |
| Jump / `nair_max_drift` | 0 | `facing_input:x:0:0:1, facing_input:a:-1:0:2, drift:opponent:1:5` |
| Jump / `nair_half_drift` | 0 | `Short Hop, Nair, Drift:Opponent:50%:10` |
| Roll / `down_smash` | 0 | `walk:away_from_opponent:5:0.9, facing_input:a:0:-1:10` |
| Roll / `grab` | 10 | `walk:away_from_opponent:12:0.8, range:fighter_range, wait:1, input:r:0:0:4` |
| Roll / `dash_attack` | 20 | `face:opponent, wait:1, facing_input:none:1:0:4, facing_input:a:0.79:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, grab` |
| Attack / `shield_rear` | 0 | `shield_until_block:70, controller_input:a:0:0.7:0:0:30:screen:screen` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `short_hop_dair` | 0 | `short_hop_dair:1` |
| Drop / `jump_up_b` | 0 | `Short Hop, wait:2, up_special` |
| Drop / `up_b` | 0 | `up_special` |

See [this fighter's detailed execution notes](../KIRBY_YOSHI_LEDGE.md) for configuration advice,
input distinctions and limitations.

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
