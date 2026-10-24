# Little Mac

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **52** (`littlemac`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Utilt**; Dtilt; Ftilt; Armored f-smash; Low f-smash (precise); Tiny nair (joke) |
| Jump | **Ground Utilt**; Tiny Uair (joke); Tiny nair (joke) |
| Roll | **Dsmash**; Grab |
| Attack | Tiny nair (joke); Grab/rear Nair; **Shield Up-B** |
| Drop | **Shield**; Dtilt > Shield; Low sweep (spacing); F-tilt poke |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 2–2 |
| Down tilt | 3–4 |
| Forward tilt | 4–5 |
| Down smash | 10–11 |
| Down-smash rear window | 17–18 |
| Up Tilt | 4–10 |
| Forward Smash | 14–15 |
| Low Forward Smash | 15–15 |
| Up Air | 5–8 |
| Grounded up-special startup | 3 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 9 |
| Dash startup | 13 |
| Pivot startup | 14 |
| Shield-grab startup | 13 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.9 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 87 |
| Walk speed | 1.386 |
| Initial dash | 2.365 |
| Run speed | 2.464 |
| Air speed | 1.208 |
| Gravity | 0.09 |
| Fall speed | 1.95 |
| Fast fall | 3.12 |
| Short-hop height | 12.53 |
| Full-hop height | 26 |

[Move source](https://ultimateframedata.com/little_mac) · [Stats source](https://ultimateframedata.com/stats.php)

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../PACMAN_MAC_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/littlemac.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/littlemac/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/littlemac.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Grounded coverage

Jump defaults to grounded up tilt, not an adaptive aerial. The tiny aerial
choices are explicit alternatives and should not be read as strong coverage.
Shield-to-Up-B is the default Attack response and waits for real block evidence.
KO Punch, charge/counter management and automatic offstage recovery are absent.
