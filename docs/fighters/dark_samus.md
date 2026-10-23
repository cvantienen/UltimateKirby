# Dark Samus

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **4** (`dark_samus`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **L+R3**. Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; SH Zair; Homing Msl; Super Msl; Cannon f-smash |
| Jump | **Nair**; SH Zair; Flame fair; Uair; Homing Msl; Super Msl |
| Roll | Dsmash; **Grab**; Move > zair; Homing (stay); Super (stay) |
| Attack | Shield Nair; Grab/rear Nair; **Shield Up-B**; Shield Zair; Shield Fair; Shield Uair |
| Drop | Shield; Dtilt > Shield; **Bomb (now)**; SH Zair; Homing Msl; Super Msl; Flame Fair (stay) |

These are the compact UI names; SH means short hop and Msl means missile.
Labels omit the UI's cached spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 8–11 |
| Down tilt | 6–8 |
| Forward tilt | 8–10 |
| Down smash | 9–10 |
| Down-smash rear window | 17–18 |
| Up Tilt | 15–18 |
| Forward Smash | 10–11 |
| Forward Air | 6–7 |
| Up Air | 5–5 |
| Grounded up-special startup | 4 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 15 |
| Dash startup | 17 |
| Pivot startup | 18 |
| Shield-grab startup | 19 |
| Active duration | 8 |
| Standing reach reference (world units) | 42.2 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../SAMUS_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/dark_samus.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/dark_samus/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/dark_samus.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Shared Samus recipes

This is a separate native fighter profile with its own movement observations.
Its compiled menu aliases Samus's zair, missile, Bomb and shield responses;
choices/defaults therefore match [Samus](samus.md). Zair is selectable in all
five rows. Roll defaults to Grab; Drop defaults to immediate down-B Bomb.
Projectile travel and native acceptance still determine outcomes.
