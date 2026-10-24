# Bayonetta

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **64** (`bayonetta`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt 1; Utilt; Wicked fist |
| Jump | **Nair**; Fair 1; Uair; Ground Utilt |
| Roll | Dsmash; **Grab**; Dtilt |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Uair; Shield Fair 1 |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair 1 |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 9–25 |
| Down tilt | 7–8 |
| Forward tilt | 12–12 |
| Down smash | 22–25 |
| Up Tilt | 7–15 |
| Forward Smash | 17–21 |
| Forward Air | 7–9 |
| Up Air | 9–18 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 7 |
| Dash startup | 10 |
| Pivot startup | 11 |
| Shield-grab startup | 11 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.5 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 81 |
| Walk speed | 0.945 |
| Initial dash | 1.936 |
| Run speed | 1.76 |
| Air speed | 1.019 |
| Gravity | 0.12 |
| Fall speed | 1.77 |
| Fast fall | 2.832 |
| Short-hop height | 21.35 |
| Full-hop height | 39 |

[Move source](https://ultimateframedata.com/bayonetta) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/bayonetta.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/bayonetta/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/bayonetta.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
