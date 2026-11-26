# Yoshi

## Configure Yoshi

This page belongs to profile **`yoshi`** (`fighter_kinds::YOSHI`), not a
roster-position index. Use `[yoshi.settings]` for overrides and
`[yoshi.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

Yoshi has down-tilt-facing support but no Kirby idle-crouch or Kirby fair/trip follow-up. Tongue-grab reference data stays exact-kind; do not copy Kirby grab timing. No fixed compiled roll endpoint or adaptive Nair schedule is inherited.

To expand this fighter, add a uniquely named option under **`yoshi`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **5** (`yoshi`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Nair2; Fsmash |
| Jump | **Nair (instant nair, max drift)**; Fw air; Meteor fair (spacing); SHNAIR 1/2 DRIFT; SHNAIR MAX DRIFT; runs SHNAIR |
| Roll | **Tongue grab**; grab shuffle |
| Attack | Shield > nair; **Grab / rear nair** |
| Drop | **Shield**; Down tilt > shield; D AIR; Fw AIR |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
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
| Neutral / `nair_ledge` | 0 | `opponent_input:x:0:0:1, drift_to_opponent:0.4:10:10, wait:8, facing_input:a:0:0:4, wait:10, facing_input:none:0:-1:4` |
| Neutral / `nair_oppenent` | 0 | `facing_input:x:0:0:1, drift:away_from_opponent:0.61:4, facing_input:a:0:0:2, wait:1, drift:opponent:1:10, wait:3, facing_input:none:0:-1:4` |
| Neutral / `nair_v2` | 0 | `facing_input:a:0:0:19` |
| Jump / `nair` | 0 | `facing_input:a+x:0.35:0:4, drift:opponent:1:20` |
| Jump / `up_air` | 0 | `facing_input:x:0:0:1, wait:5, facing_input:a:1:0:4, drift:away_from_opponent:1:6, wait:5, facing_input:none:0:-1:4` |
| Jump / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
| Jump / `nair_fast` | 0 | `short_hop_nair, drift:opponent:0.50:10` |
| Jump / `nair_fas-2` | 0 | `short_hop, nair, drift:opponent:1:10` |
| Jump / `nair_fas-3` | 0 | `drift:ledge:100%:4, short_hop_nair, drift:opponent:1:5` |
| Roll / `grab` | 0 | `drift:away_from_opponent:0.61:10, wait:8, range:30, facing_input:r:0:0:4` |
| Roll / `grab_shuffle` | 0 | `wait:1, face:away, wait:1, face:opponent, walk:opponent:10:0.4, grab:fighter_range` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, short_hop_nair:4` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, facing_input:r:0:0:4` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `menu_added_4` | 0 | `facing_input:x:0:0:1, facing_input:a:0:-1:4, drift:opponent:1:15` |
| Drop / `menu_added_5` | 0 | `facing_input:x:0:0:1, wait:9, facing_input:a:1:0:1, drift:away_from_opponent:1:4, wait:11, facing_input:none:0:-1:4` |

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
