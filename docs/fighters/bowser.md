# Bowser

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **15** (`bowser`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Utilt; Dropkick |
| Jump | **Nair**; Fair; Uair; Ground Utilt |
| Roll | **Dsmash**; Grab; Dtilt |
| Attack | Shield Nair; Grab/rear Nair; **Shield Up-B**; Shield Uair; Shield Fair |
| Drop | **Shield**; Dtilt > Shield; Near Nair; Near Fair; Low smash (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 8–29 |
| Down tilt | 10–12 |
| Forward tilt | 10–14 |
| Down smash | 12–14 |
| Down-smash rear window | 28–30 |
| Up Tilt | 11–16 |
| Forward Smash | 22–27 |
| Forward Air | 11–14 |
| Up Air | 9–13 |
| Grounded up-special startup | 6 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 8 |
| Dash startup | 11 |
| Pivot startup | 12 |
| Shield-grab startup | 12 |
| Active duration | 3 |
| Standing reach reference (world units) | 20.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 135 |
| Walk speed | 0.901 |
| Initial dash | 2.255 |
| Run speed | 1.971 |
| Air speed | 1.155 |
| Gravity | 0.125 |
| Fall speed | 1.77 |
| Fast fall | 2.832 |
| Short-hop height | 15.7 |
| Full-hop height | 33 |

[Move source](https://ultimateframedata.com/bowser) · [Stats source](https://ultimateframedata.com/stats.php)

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/bowser.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/bowser/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/bowser.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->
