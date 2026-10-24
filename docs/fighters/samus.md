# Samus

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **3** (`samus`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; SH Zair; Homing Msl; Super Msl; Cannon f-smash |
| Jump | **Nair**; SH Zair; Flame fair; Uair; Homing Msl; Super Msl |
| Roll | Dsmash; **Grab**; Move > zair; Homing (stay); Super (stay) |
| Attack | Shield Nair; Grab/rear Nair; **Shield Up-B**; Shield Zair; Shield Fair; Shield Uair |
| Drop | Shield; Dtilt > Shield; **Bomb (now)**; SH Zair; Homing Msl; Super Msl; Flame Fair (stay) |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
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
| Forward Air First | 6–7 |
| Up Air First | 5–5 |
| Forward Smash | 10–11 |
| Zair Active | 8–19 |
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

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 108 |
| Walk speed | 1.115 |
| Initial dash | 1.87 |
| Run speed | 1.654 |
| Air speed | 1.103 |
| Gravity | 0.075 |
| Fall speed | 1.33 |
| Fast fall | 2.128 |
| Short-hop height | 18 |
| Full-hop height | 37 |

[Move source](https://ultimateframedata.com/samus) · [Stats source](https://ultimateframedata.com/stats.php)

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/samus.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/samus/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/samus.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Samus-specific responses

Zair appears in every response row. Neutral/Jump/shield zair stay in place;
Roll zair approaches its coverage point. Zair uses its own neutral-hop then
Grab input sequence, with native zair acceptance, rather than the grounded
roll-grab recipe. Missiles are stationary early pressure: homing startup 18,
Super startup 21, plus native projectile travel. They are not timed guaranteed hits.
Homing starts on the first actionable recognition sample in Neutral/Jump/Roll/
Drop, pairing 0.70 horizontal stick with B toward the opponent, regardless of
current facing. It skips the neutral primer and separate turn wait; the stronger
tilt replaces the shallow 0.45 input that could select Charge Shot. Super Missile
retains its existing turn/neutral-primer/full-flick sequence. Actual missile type
and travel/contact still need Switch validation.

Roll defaults to Grab and Drop to immediate down-B Bomb. Bomb startup confirms
an ordinary input and returns control; generation/collision timing remains native.
Roll Grab first turns toward center stage when roll get-up is recognized and
Samus is facing the ledge. It uses a 0.6-stick inward turn without Grab, then
resumes the existing approach/timing once native facing confirms. Already
inward-facing Samus takes the existing path directly; issued grabs are not
interrupted. The direction preparation also applies to shared Dark Samus recipes.
Dark Samus shares these recipes. See [Samus execution](../SAMUS_LEDGE.md) for
shield gating, projectile limitations and the full zair sequence.
