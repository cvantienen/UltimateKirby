# Sonic

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **41** (`sonic`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt |
| Jump | **Nair** |
| Roll | **Dsmash**; Grab |
| Attack | **Shield Nair**; Grab/rear Nair |
| Drop | **Shield**; Dtilt > Shield |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 6–38 |
| Down tilt | 6–7 |
| Forward tilt | 8–11 |
| Down smash | 12–13 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 7 |
| Dash startup | 10 |
| Pivot startup | 11 |
| Shield-grab startup | 11 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 86 |
| Walk speed | 1.444 |
| Initial dash | 2.31 |
| Run speed | 3.85 |
| Air speed | 1.208 |
| Gravity | 0.09 |
| Fall speed | 1.65 |
| Fast fall | 2.64 |
| Short-hop height | 16.89 |
| Full-hop height | 35 |

[Move source](https://ultimateframedata.com/sonic) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../SONIC_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/sonic.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/sonic/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/sonic.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Sonic-specific limits

Nair uses the existing paired aerial input path with Sonic's own stored frames.
Roll offers down-smash or grab; capture gives manual control. Grounded Spring
Jump is not used as a damaging shield punish, and Spin Charge has no charge/
release recipe here. Y is reserved for future modes; B-reverse is unbound. Sonic has no integrated
recovery-cover implementation yet.
