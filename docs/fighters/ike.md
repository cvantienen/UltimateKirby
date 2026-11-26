# Ike

## Configure Ike

This page belongs to profile **`ike`** (`fighter_kinds::IKE`), not a
roster-position index. Use `[ike.settings]` for overrides and
`[ike.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

This identity uses the shared file-authored runner, not a dedicated compiled move routine. No standalone Lucina counter, Game & Watch reflector or Kirby-specific follow-up is inherited. Universal held tools have their own eligibility gates; registering a fighter does not bypass them.

To expand this fighter, add a uniquely named option under **`ike`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **35** (`ike`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Down tilt**; Forward tilt; Forward smash |
| Jump | Nair; Forward air; **Grounded up tilt** |
| Roll | **Down smash**; Grab; Down tilt |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield > Uair; Shield > Fair |
| Drop | **Shield**; Down tilt > shield; Hop nair (near); Hop fair (near); Low smash (spacing) |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up Air | `ike.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `ike.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `ike.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `ike.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `ike.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `ike.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 4 start; active 4–5; 23 total; -- landing |
| Neutral Air | 10 start; active 10–14 / 15–22; 59 total; 8 landing |
| Forward Air | 11 start; active 11–15; 51 total; 14 landing |
| Back Air | 7 start; active 7–9; 54 total; 11 landing |
| Up Air | 13 start; active 13–18; 56 total; 9 landing |
| Down Air | 16 start; active 16–17; 54 total; 14 landing |
| Forward Tilt | 12 start; active 12–13; 41 total; -- landing |
| Up Tilt | 11 start; active 11–16 / 17–21; 39 total; -- landing |
| Down Tilt | 7 start; active 7–8; 28 total; -- landing |
| Dash Attack | 15 start; active 15–16 / 17–19; 47 total; -- landing |
| Forward Smash | 31 start; active 31–32 / 33–35; 83 total; -- landing |
| Up Smash | 25 start; active 25–29 / 30–31; 70 total; -- landing |
| Down Smash | 13/32 start; active 13–15 / 32–33 / 34–36; 70 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 10–11 |
| Pivot grab | frames 11–12 |
| Standing reach reference | 11.5 world units |
| Shield-grab startup | 11 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 107 |
| Walk speed | 0.912 |
| Initial dash | 1.815 |
| Run speed | 1.507 |
| Air speed | 1.134 |
| Gravity | 0.092 |
| Fall speed | 1.65 |
| Fast fall | 2.64 |
| Short-hop height | 16 |
| Full-hop height | 29.9 |

## Execution and calibration

Option definitions and input delays come from `fighters/ike.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 12 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 25 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 20 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:21, release:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `forward_air` | 0 | `opponent_input:a+x:1:0:1` |
| Jump / `up_tilt` | 0 | `opponent_input:a:0:0.6:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:4, release:1` |
| Roll / `grab` | 24 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `down_tilt` | 24 | `opponent_input:a:0:-0.6:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `drop_nair` | 0 | `short_hop_nair:4` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
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
