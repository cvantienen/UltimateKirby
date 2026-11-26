# Little Mac

## Configure Little Mac

This page belongs to profile **`littlemac`** (`fighter_kinds::LITTLE_MAC`), not a
roster-position index. Use `[littlemac.settings]` for overrides and
`[littlemac.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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

Qualitative aerial hints say touching/below-near-edge for Little Mac; these are display metadata, not detection ranges. A configured Jump response may stay grounded. KO/charge/resource behavior is not inferred from a move label.

To expand this fighter, add a uniquely named option under **`littlemac`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **52** (`littlemac`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Up tilt; **Down tilt**; Forward tilt; Armored f-smash; Low f-smash (precise); Tiny nair (joke) |
| Jump | **Grounded up tilt**; Tiny up-air (joke); Tiny nair (joke) |
| Roll | **Down smash**; Grab |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield > Up-B** |
| Drop | **Shield**; Down tilt > shield; Low sweep (spacing); F-tilt poke |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `littlemac.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `littlemac.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `littlemac.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `littlemac.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `littlemac.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `littlemac.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 1 start; active 1; 16 total; -- landing |
| Neutral Air | 2 start; active 2; 15 total; 10 landing |
| Forward Air | 10 start; active 10–12; 36 total; 13 landing |
| Back Air | 11 start; active 11–12; 37 total; 16 landing |
| Up Air | 5 start; active 5–8; 41 total; 13 landing |
| Down Air | 7 start; active 7–10; 27 total; 18 landing |
| Forward Tilt | 4/12 start; active 4–5 / 12–13; 37 total; -- landing |
| Up Tilt | 4 start; active 4–10; 29 total; -- landing |
| Down Tilt | 3 start; active 3–4; 25 total; -- landing |
| Dash Attack | 7 start; active 7–9; 33 total; -- landing |
| Forward Smash | 14/14/15 start; active 14–15 / 14–15 / 15; 43/43/43 total; -- landing |
| Up Smash | 10 start; active 10 / 11–14; 47 total; -- landing |
| Down Smash | 10/17 start; active 10–11 / 17–18; 42 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 9–10 |
| Dash grab | frames 13–14 |
| Pivot grab | frames 14–15 |
| Standing reach reference | 12.9 world units |
| Shield-grab startup | 13 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 87 |
| Walk speed | 1.386 |
| Initial dash | 2.365 |
| Run speed | 2.464 |
| Air speed | 1.208 |
| Gravity | 0.09 |
| Fall speed | 1.95 |
| Fast fall | 3.12 |
| Short-hop height | 12.53 |
| Full-hop height | 26 |

## Execution and calibration

Option definitions and input delays come from `fighters/littlemac.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `up_tilt` | 27 | `opponent_input:a:0:0.6:1` |
| Neutral / `down_tilt` | 30 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 29 | `opponent_input:a:0.6:0:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Neutral / `low_forward_smash` | 0 | `opponent_input:a:1:0:23, release:1` |
| Neutral / `nair` | 29 | `opponent_input:a+x:0:0:1` |
| Jump / `up_tilt` | 7 | `opponent_input:a:0:0.6:1` |
| Jump / `up_air` | 5 | `opponent_input:a+x:0:1:1` |
| Jump / `nair` | 9 | `opponent_input:a+x:0:0:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:26, release:1, else_facing, opponent_input:a:0:-1:19, release:1, end_facing` |
| Roll / `grab` | 22 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `sweep` | 0 | `face:opponent, down_smash:1` |
| Drop / `poke` | 0 | `face:opponent, forward_tilt:1` |

See [this fighter's detailed execution notes](../PACMAN_MAC_LEDGE.md) for configuration advice,
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
