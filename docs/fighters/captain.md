# Captain Falcon

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **11** (`captain`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Ftilt**; Nair; Dtilt |
| Jump | **Nair** |
| Roll | **Dsmash** |
| Attack | **Grab/rear Nair**; Shield Nair |
| Drop | **Shield**; Dtilt > Shield |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–8 |
| Down tilt | 11–12 |
| Forward tilt | 7–10 |
| Down smash | 19–20 |
| Down-smash rear window | 29–30 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 6 |
| Dash startup | 9 |
| Pivot startup | 10 |
| Shield-grab startup | 10 |
| Active duration | 2 |
| Standing reach reference (world units) | 13.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 68 |
| Walk speed | 0.735 |
| Initial dash | 1.65 |
| Run speed | 1.271 |
| Air speed | 1.332 |
| Gravity | 0.053 |
| Fall speed | 0.98 |
| Fast fall | 1.568 |
| Short-hop height | 11.26 |
| Full-hop height | 19.79 |

[Move source](https://ultimateframedata.com/jigglypuff) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../FIGHTER_ADDITIONS.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/captain.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/captain/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/captain.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
