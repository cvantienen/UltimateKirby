# Lucas

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **40** (`lucas`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; Utilt; Stick reflector |
| Jump | **Nair**; Fair; Uair; Ground Utilt |
| Roll | Dsmash; **Grab**; Dtilt |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Uair; Shield Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 7–7 |
| Down tilt | 3–4 |
| Forward tilt | 7–9 |
| Down smash | 20–22 |
| Up Tilt | 4–4 |
| Forward Smash | 14–15 |
| Forward Air | 9–12 |
| Up Air | 7–10 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 12 |
| Dash startup | 14 |
| Pivot startup | 15 |
| Shield-grab startup | 16 |
| Active duration | 6 |
| Standing reach reference (world units) | 22.9 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 94 |
| Walk speed | 0.893 |
| Initial dash | 1.815 |
| Run speed | 1.65 |
| Air speed | 1.155 |
| Gravity | 0.09 |
| Fall speed | 1.37 |
| Fast fall | 2.192 |
| Short-hop height | 13 |
| Full-hop height | 29.41 |

[Move source](https://ultimateframedata.com/lucas) · [Stats source](https://ultimateframedata.com/stats.php)

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/lucas.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/lucas/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/lucas.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
