# Yoshi

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **5** (`yoshi`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **L+R3**. Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Utilt; Head Fsmash |
| Jump | **Nair**; Uair; Meteor fair (spacing) |
| Roll | **Dsmash**; Tongue grab |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Uair |
| Drop | **Shield**; Dtilt > Shield; Meteor (risky); Dair (risky) |

These are the compact UI names; SH means short hop and Msl means missile.
Labels omit the UI's cached spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 3–25 |
| Down tilt | 8–10 |
| Forward tilt | 5–7 |
| Down smash | 7–8 |
| Down-smash rear window | 22–23 |
| Up Tilt | 8–15 |
| Forward Smash | 14–16 |
| Up Air | 5–6 |
| Forward Air | 16–20 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 14 |
| Dash startup | 16 |
| Pivot startup | 17 |
| Shield-grab startup | 18 |
| Active duration | 8 |
| Standing reach reference (world units) | 26.0 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Execution and calibration

The profile declares adaptive short/full-hop selection for generic Jump coverage;
common Neutral and shield nairs remain short hops. A named handler can override
that policy, including grounded attacks and immediate short-hop recipes.

See [this fighter's detailed execution notes](../KIRBY_YOSHI_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/yoshi.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/yoshi/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/yoshi.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Yoshi-specific timing and movement

Jump nair immediately requests short-hop nair on recognition, bypassing adaptive
hop/height waits. Neutral nair uses a 17-frame early lead before protection ends.
Both nudge toward the opponent with 0.25 stick only below native nair animation
frame 4, then go neutral. Existing momentum can continue after the nudge.
Up-air and fair retain adaptive jump selection.

Roll down-smash stands two units toward the ledge from the configured point.
Tongue grab uses the shared active-window/range check and 0.65 inward walking;
its longer startup and eight active frames are distinct from Kirby's grab.
The approach can extend inward from observed travel and hands control back on
capture. It does not automatically pummel or throw. Offstage fair/dair choices
remain risky chases with their own longer drift, not the brief nair nudge.
