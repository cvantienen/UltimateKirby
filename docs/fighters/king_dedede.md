# King Dedede

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **42** (`king_dedede`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Utilt |
| Jump | **Nair**; Fair; Uair; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Uair; Shield Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair; Near Dair; Low smash (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–29 |
| Down tilt | 6–11 |
| Forward tilt | 12–21 |
| Down smash | 14–22 |
| Up Tilt | 7–13 |
| Forward Smash | 40–44 |
| Forward Air | 13–15 |
| Up Air | 10–10 |
| Down Air | 22–23 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 8 |
| Dash startup | 11 |
| Pivot startup | 12 |
| Shield-grab startup | 12 |
| Active duration | 3 |
| Standing reach reference (world units) | 17.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 127 |
| Walk speed | 1.029 |
| Initial dash | 1.815 |
| Run speed | 1.496 |
| Air speed | 0.735 |
| Gravity | 0.097 |
| Fall speed | 1.95 |
| Fast fall | 3.12 |
| Short-hop height | 16.02 |
| Full-hop height | 32.85 |

[Move source](https://ultimateframedata.com/king_dedede) · [Stats source](https://ultimateframedata.com/stats.php)

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/king_dedede.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/king_dedede/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/king_dedede.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
