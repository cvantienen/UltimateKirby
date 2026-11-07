# Mythra

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **91** (`mythra`). This reference follows the canonical fighter data;
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
| Jab 1 | 2 start; active 2–4; 20 total; -- landing |
| Forward Air | 8 start; active 8–11; 37 total; 10 landing |
| Back Air | 10 start; active 10–11 / 12; 33 total; 10 landing |
| Up Air | 9 start; active 9–13; 30 total; 8 landing |
| Down Air | 13 start; active 13–15 / 16; 40 total; 11 landing |
| Forward Tilt | 8 start; active 8–9; 25 total; -- landing |
| Up Tilt | 7 start; active 7–12; 26 total; -- landing |
| Down Tilt | 5 start; active 5–7; 20 total; -- landing |
| Dash Attack | 9 start; active 9–10; 39 total; -- landing |
| Forward Smash | 14 start; active 14–17; 49 total; -- landing |
| Down Smash | 8/14 start; active 8–10 / 14–16; 44 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 6–7 |
| Dash grab | frames 11–12 |
| Pivot grab | frames 12–13 |
| Standing reach reference | 12.0 world units |
| Shield-grab startup | 10 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
| --- | --- |
| Weight | 92 |
| Walk speed | 1.42 |
| Initial dash | 2.45 |
| Run speed | 2.41 |
| Air speed | 1.22 |
| Gravity | 0.137 |
| Fall speed | 1.87 |
| Fast fall | 2.992 |
| Short-hop height | Unknown |
| Full-hop height | Unknown |

## Execution and calibration

Option definitions and input delays come from `fighters/mythra.txt`, assembled into the Switch’s `sequences.txt`. Each option
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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/mythra.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Editable menu and form swaps

Mythra has a separate fighter file and five editable ledge starters. Each
uses `wait:0` until configured in Training; no automatic attack or reflector
is supplied. Unconfigured automatic punish events remain inactive.

Swapping forms uses the existing selected-fighter handoff to clear the outgoing
menu and policies and load the incoming form's saved choices. Save edits before
swapping. Each form's edits are stored independently.

These menus require both the updated plugin and the updated active sequences
file. A normal upload only stages sequences; replace the active file with Smash
closed when installing this configuration.
