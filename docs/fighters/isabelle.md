# Isabelle

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **70** (`isabelle`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Party popper; Slingshot |
| Jump | **Nair**; Uair; Slingshot |
| Roll | **Dsmash**; Net grab; Slingshot |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Uair; Shield Sling |
| Drop | **Shield**; Dtilt > Shield; Turnip (risky); Slingshot |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 5–25 |
| Down tilt | 9–10 |
| Forward tilt | 8–11 |
| Down smash | 8–9 |
| Down-smash rear window | 12–13 |
| Up Air | 6–31 |
| Forward Air | 10–12 |
| Back Air | 13–15 |
| Down Air | 11–28 |
| Forward Smash | 14–19 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 14 |
| Dash startup | 16 |
| Pivot startup | 17 |
| Shield-grab startup | 18 |
| Active duration | 2 |
| Standing reach reference (world units) | 19.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 88 |
| Walk speed | 1.14 |
| Initial dash | 1.815 |
| Run speed | 1.48 |
| Air speed | 1.02 |
| Gravity | 0.07 |
| Fall speed | 1.3 |
| Fast fall | 2.08 |
| Short-hop height | 15.69 |
| Full-hop height | 32.5 |

[Move source](https://ultimateframedata.com/isabelle) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../KROOL_ISABELLE_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/isabelle.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/isabelle/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/isabelle.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Slingshot coverage

Slingshot is selectable in all five response rows. Facing selects fair or bair
without a turnaround, with no post-emission drift. Neutral/Roll stage a stationary
short-hop shot against protection; Roll does not chase the endpoint. Jump can
use adaptive hop choice. Attack waits for a real shield block and jump readiness;
Drop requests the shot immediately. Projectile travel is not modeled.

Net grab has its own startup, active duration and reach reference. Successful
capture hands back control. Party popper and turnip chase are alternatives with
different startup/spacing; they do not share a guaranteed hit window.
