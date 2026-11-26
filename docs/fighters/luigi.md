# Luigi

## Configure Luigi

This page belongs to profile **`luigi`** (`fighter_kinds::LUIGI`), not a
roster-position index. Use `[luigi.settings]` for overrides and
`[luigi.neutral.my_response]` (or jump/roll/attack/drop) for an authored option.
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
| `walk_strength` | `0.49999985` |

### Support and extension boundary

This identity uses the shared file-authored runner, not a dedicated compiled move routine. No standalone Lucina counter, Game & Watch reflector or Kirby-specific follow-up is inherited. Universal held tools have their own eligibility gates; registering a fighter does not bypass them.

To expand this fighter, add a uniquely named option under **`luigi`** using
existing actions first. Keep all waits, directional inputs, range checks and
resource assumptions explicit. Do not change `fighter_kind` to another fighter
to borrow a move. New primitives require a reviewed shared executor change;
new character-specific prediction needs a verified native adapter, not only
more sliders. [Rust extension contracts](../FIGHTER_CONFIGS.md#extend-the-rust-code)
explain registration, `Option`, validation and focused tests.

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **9** (`luigi`). This reference follows the canonical fighter data;
it is not a device-calibration result. The tables are a generated snapshot of repository assets, not your installed or unsaved menu state. No fighter-specific file loads during a match.

## Saved ledge responses

Training starts with the menu hidden; **Minus** opens it. Outside Training the lightweight selector starts visible.
Up/Down selects a row; Left/Right cycles its
choices. Activate the configured Ledge Assist binding to run selected responses; consult Controls for your layout. Bold choices are supplied file defaults, not Rust defaults.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; F ball Grab; **dash** |
| Jump | Nair; **dash** |
| Roll | Down smash; **Grab** |
| Attack | Shield > Nair; **Shield > range-checked grab** |
| Drop | **Shield**; Down tilt > shield |

These names are shown in the menu exactly as authored in the sequence file.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These are saved configuration records, not proof that a trigger is currently admitted.
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. `auto_grab` is rejected by the current parser; current automatic grabs require your shield. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `luigi.roll.auto_air_dodge` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `luigi.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `luigi.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `luigi.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `luigi.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `luigi.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 2 start; active 2–3; 18 total; -- landing |
| Neutral Air | 3 start; active 3–5 / 6–31; 44 total; 8 landing |
| Forward Air | 7 start; active 7–10; 23 total; 13 landing |
| Back Air | 6 start; active 6–7 / 8–13; 44 total; 10 landing |
| Up Air | 5 start; active 5–7 / 8–11; 26 total; 7 landing |
| Down Air | 10 start; active 10 / 11–14; 28 total; 12 landing |
| Forward Tilt | 5 start; active 5–7; 32 total; -- landing |
| Up Tilt | 5 start; active 5–10; 27 total; -- landing |
| Down Tilt | 5 start; active 5–8; 13 total; -- landing |
| Dash Attack | 4/8/12/16/25 start; active 4 / 8 / 12 / 16 / 25; 47 total; -- landing |
| Forward Smash | 12 start; active 12–13; 41 total; -- landing |
| Up Smash | 9 start; active 9–13; 39 total; -- landing |
| Down Smash | 6/14 start; active 6–7 / 14–15; 37 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 14–20 |
| Dash grab | frames 16–22 |
| Pivot grab | frames 17–24 |
| Standing reach reference | 29.8 world units |
| Shield-grab startup | 18 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 97 |
| Walk speed | 1.134 |
| Initial dash | 1.815 |
| Run speed | 1.65 |
| Air speed | 0.77 |
| Gravity | 0.083 |
| Fall speed | 1.32 |
| Fast fall | 2.112 |
| Short-hop height | 19.98 |
| Full-hop height | 44 |

## Execution and calibration

Option definitions and input delays come from `fighters/luigi.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 0 | `short_hop:1, drift:opponent:0.5:10, opponent_input:a:0:0:1` |
| Neutral / `down_tilt` | 0 | `opponent_input:b:0:0:4, range:fighter_range, wait:30, input:r:0:0:1` |
| Neutral / `forward_tilt` | 20 | `drift:opponent:1:4, opponent_input:a:0.6:0:4` |
| Jump / `nair` | 0 | `short_hop:1, opponent_input:a:0:0:1, drift:opponent:1:10` |
| Jump / `dash` | 0 | `drift:opponent:1:4, opponent_input:a:0.6:0:1` |
| Roll / `down_smash` | 0 | `drift:away_from_opponent:1:4, opponent_input:a:0:-1:20, wait:1` |
| Roll / `grab` | 0 | `drift:away_from_opponent:0.5:4, wait:10, range:fighter_range, press:r:4` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, range:fighter_range, input:r:0:0:4` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |

See [this fighter's detailed execution notes](../FIGHTER_ADDITIONS.md) for configuration advice,
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
