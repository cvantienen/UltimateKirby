# Kirby

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **6** (`kirby`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **L+R3**. Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Stone; Inhale; Bair |
| Jump | **Nair**; Bair |
| Roll | **Dsmash**; Grab |
| Attack | **Grab/rear Nair**; Grab/rear Guard |
| Drop | **Shield**; Dtilt > Shield; SH Dair |

These are the compact UI names; SH means short hop and Msl means missile.
Labels omit the UI's cached spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 8–32 |
| Down tilt | 4–6 |
| Forward tilt | 5–8 |
| Down smash | 7–19 |
| Jump Bair | 6–12 |
| Grounded down-special startup | 11 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 6 |
| Dash startup | 9 |
| Pivot startup | 10 |
| Shield-grab startup | 10 |
| Active duration | 2 |
| Standing reach reference (world units) | 12.5 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Execution and calibration

The profile declares short hops for common nair responses.

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/kirby.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/kirby/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/kirby.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Kirby-specific timing and inputs

Neutral and Jump nair use a quarter-stick nudge toward the opponent only while
native nair animation frame is below 4. It starts after nair acceptance; the
stick then returns to neutral, although momentum can continue. Neutral nair adds
two frames to the configured lead (8 becomes 10 in the supplied config). Jump
nair's timing gate permits two frames earlier; immediate launch cues stay immediate.

Jump bair uses 0.35 drift. Neutral bair short-hops with zero drift and a
three-frame early lead. Forward-facing bair uses backward C-stick plus the
acknowledged jump cancel; back-facing uses the direct route. Use Tilt Attack
C-stick and both X/ZL mapped to Jump.

Roll Grab uses 0.65 inward walking with a bounded inward extension from observed
roll travel. Capture returns full control. L3 recovery cover, tilt-trip follow-up
and fair-first-hit landing follow-up are separate tools; see [controls](../CONTROLS.md).
These current movement edits still need device testing.
