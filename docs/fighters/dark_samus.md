# Dark Samus

## Configure Dark Samus

This page belongs to profile **`dark_samus`** (`fighter_kinds::DARK_SAMUS`), not a
roster-position index. Use `[dark_samus.settings]` for overrides and
`[dark_samus.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

Dark Samus has its own kind/profile/catalog. It does not borrow Samus selections or automatically alias her scripts. Copying a reviewed tether/missile recipe is an explicit configuration action; keep this form's data and saved settings independent.

To expand this fighter, add a uniquely named option under **`dark_samus`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **4** (`dark_samus`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Down tilt**; Forward tilt; Short-hop zair; Homing missile; Super missile; Cannon f-smash |
| Jump | **Nair**; Short-hop zair; Flame fair; Up air; Homing missile; Super missile |
| Roll | **Down smash**; Grab; Move > zair; Homing (stay back); Super (stay back) |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield > Up-B**; Shield > Zair; Shield > Fair; Shield > Uair |
| Drop | **Shield**; Down tilt > shield; Immediate down-B; Short-hop zair; Homing missile; Super missile; Flame fair (stay back) |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `dark_samus.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `dark_samus.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `dark_samus.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `dark_samus.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `dark_samus.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `dark_samus.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3–4; 17 total; -- landing |
| Neutral Air | 8/14 start; active 8–11 / 14–15 / 16–22; 45 total; 9 landing |
| Forward Air | 6/12/18/24/30 start; active 6–7 / 12–13 / 18–19 / 24–25 / 30–31; 59 total; 14 landing |
| Back Air | 9 start; active 9–10 / 11–14; 41 total; 14 landing |
| Down Air | 17 start; active 17–18 / 19–21 / 22–23; 48 total; 12 landing |
| Forward Tilt | 8 start; active 8–10; 33 total; -- landing |
| Up Tilt | 15 start; active 15–18; 39 total; -- landing |
| Down Tilt | 6 start; active 6–8; 44 total; -- landing |
| Dash Attack | 8 start; active 8–9 / 10–13 / 14–18; 41 total; -- landing |
| Forward Smash | 10 start; active 10–11; 48 total; -- landing |
| Up Smash | 11/15/19/23/27 start; active 11–12 / 15–16 / 19–20 / 23–24 / 27–28; 56 total; -- landing |
| Down Smash | 9/17 start; active 9–10 / 17–18; 44 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 15–22 |
| Dash grab | frames 17–24 |
| Pivot grab | frames 18–25 |
| Standing reach reference | 42.2 world units |
| Shield-grab startup | 19 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 108 |
| Walk speed | 1.115 |
| Initial dash | 1.87 |
| Run speed | 1.654 |
| Air speed | 1.103 |
| Gravity | 0.075 |
| Fall speed | 1.33 |
| Fast fall | 2.128 |
| Short-hop height | 18 |
| Full-hop height | 37 |

## Execution and calibration

Option definitions and input delays come from `fighters/dark_samus.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 13 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 27 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 25 | `opponent_input:a:0.6:0:1` |
| Neutral / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Neutral / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Neutral / `super_missile` | 0 | `face:opponent, side_special:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:26, release:1` |
| Jump / `nair` | 2 | `opponent_input:a+x:0:0:1` |
| Jump / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Jump / `forward_air` | 5 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 6 | `opponent_input:a+x:0:1:1` |
| Jump / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Jump / `super_missile` | 0 | `face:opponent, side_special:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:25, release:1, else_facing, opponent_input:a:0:-1:17, release:1, end_facing` |
| Roll / `grab` | 16 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Roll / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Roll / `super_missile` | 0 | `face:opponent, side_special:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Attack / `shield_zair` | 0 | `shield_until_block:90, check:enemy_in_front:0, input:x:0:0:1, wait:2, check:self_airborne:0, input:r:0:0:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `bomb` | 0 | `face:opponent, down_special:1` |
| Drop / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Drop / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Drop / `super_missile` | 0 | `face:opponent, side_special:1` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |

See [this fighter's detailed execution notes](../SAMUS_LEDGE.md) for configuration advice,
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
