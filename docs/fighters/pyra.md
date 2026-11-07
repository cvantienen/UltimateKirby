# Pyra

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **90** (`pyra`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Custom (edit in Training)** |
| Jump | **Custom (edit in Training)** |
| Roll | **Custom (edit in Training)** |
| Attack | **Custom (edit in Training)** |
| Drop | **Custom (edit in Training)** |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| — | — | No saved assist rules in the supplied master | — |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3–5; 21 total; -- landing |
| Neutral Air | 11 start; active 11–16 / 17–22; 56 total; 15 landing |
| Forward Air | 11 start; active 11–14; 48 total; 14 landing |
| Back Air | 16 start; active 16–17 / 18; 49 total; 14 landing |
| Up Air | 13 start; active 13–17; 57 total; 8 landing |
| Down Air | 17 start; active 17 / 18 / 19–20; 65 total; 14 landing |
| Forward Tilt | 12 start; active 12–13; 37 total; -- landing |
| Up Tilt | 11 start; active 11–16; 39 total; -- landing |
| Down Tilt | 9 start; active 9–11; 26 total; -- landing |
| Dash Attack | 17 start; active 17–18; 52 total; -- landing |
| Forward Smash | 20 start; active 20–23; 67 total; -- landing |
| Up Smash | 15/17 start; active 15–16 / 17–23 / 24–33; 66 total; -- landing |
| Down Smash | 12/18 start; active 12–14 / 18–20; 47 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 7–8 |
| Dash grab | frames 12–13 |
| Pivot grab | frames 13–14 |
| Standing reach reference | 12.0 world units |
| Shield-grab startup | 11 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 98 |
| Walk speed | 0.92 |
| Initial dash | 1.69 |
| Run speed | 1.38 |
| Air speed | 1.1 |
| Gravity | 0.078 |
| Fall speed | 1.62 |
| Fast fall | 2.592 |
| Short-hop height | Unknown |
| Full-hop height | Unknown |

## Execution and calibration

Option definitions and input delays come from `fighters/pyra.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `custom` | 0 | `wait:0` |
| Jump / `custom` | 0 | `wait:0` |
| Roll / `custom` | 0 | `wait:0` |
| Attack / `custom` | 0 | `wait:0` |
| Drop / `custom` | 0 | `wait:0` |

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Editable menu and form swaps

Pyra has a separate fighter file and five editable ledge starters. Each
uses `wait:0` until configured in Training; no automatic attack or reflector
is supplied. Unconfigured automatic punish events remain inactive.

Swapping forms uses the existing selected-fighter handoff to clear the outgoing
menu and policies and load the incoming form's saved choices. Save edits before
swapping. Each form's edits are stored independently.

These menus require both the updated plugin and the updated active sequences
file. A normal upload only stages sequences; replace the active file with Smash
closed when installing this configuration.
