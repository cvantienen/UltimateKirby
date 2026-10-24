# Pac-Man

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **55** (`pacman`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

Open the menu with **R3**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | **Nair**; Dtilt; Ftilt; Ghost f-smash; Fruit (stored) |
| Jump | **Nair**; Uair; Fair; Fruit (stored) |
| Roll | Lingering ghosts; **Long grab** |
| Attack | **Shield Nair**; Grab/rear Nair; Shield Tramp |
| Drop | **Shield**; Dtilt > Shield; SH Bite; Low ghosts (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Compact move labels match the menu without spacing hints. Capture returns control for manual
pummels/throws. Selecting a choice does not add a combo or change native hitboxes.

## Compiled move windows

These are stored inclusive animation-frame spans. They are not total input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Neutral air | 3–19 |
| Down tilt | 7–9 |
| Forward tilt | 5–7 |
| Down smash | 15–28 |
| Forward Smash | 16–29 |
| Up Air | 9–16 |
| Forward Air | 5–8 |
| Grounded up-special startup | 4 |

### Grab data

| Setting | Compiled value |
| --- | --- |
| Standing startup | 12 |
| Dash startup | 14 |
| Pivot startup | 15 |
| Shield-grab startup | 16 |
| Active duration | 22 |
| Standing reach reference (world units) | 28.2 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
Shared roll grabs test predicted range/facing, height and vulnerability on the
same active frame. Native catch readiness and input delivery still matter.

## Baseline reference stats

Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not
instance modifiers or a replacement for existing ledge tuning.

| Stat | Reference |
| --- | --- |
| Weight | 95 |
| Walk speed | 1.097 |
| Initial dash | 1.87 |
| Run speed | 1.672 |
| Air speed | 1.092 |
| Gravity | 0.072 |
| Fall speed | 1.35 |
| Fast fall | 2.16 |
| Short-hop height | 16.48 |
| Full-hop height | 34.1 |

[Move source](https://ultimateframedata.com/pac_man) · [Stats source](https://ultimateframedata.com/stats.php)

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

- Registration: `crates/kirby_assist_core/src/fighter_profiles/pacman.rs`.
- Move data: `crates/kirby_assist_core/src/fighter_profiles/pacman/timing.rs`.
- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/pacman.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
These are compiled choices; source changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

## Immediate stored-fruit response

Neutral and Jump each have an additional **Fruit (stored)** choice. Prepare the
trampoline/spacing yourself and charge-cancel the desired fruit beforehand.
The response does not charge/select fruit, inspect its identity, set up Pac-Jump
or read/edit trampoline color. Existing choices/defaults remain unchanged.

On recognizing the selected get-up, grounded/actionable Pac-Man requests neutral
B immediately, with no protection-expiry, projectile-arrival or range wait. If
facing away, a 0.6-stick turn comes first; neutral B follows native facing
confirmation. B releases on observed neutral-special entry, leaving one attempt
per L hold. Release/re-hold L to rearm. The player must supply stored fruit:
without it this can enter ordinary Bonus Fruit charging. There is no automatic
fruit selection or recovery from a missing-fruit setup.

Tapping Special releases a stored fruit; holding it resumes cycling, according
to the [Prima Ultimate guide](https://primagames.com/eguides/prima-games-8-eguide/pac-man/brawler-arsenal).
The stored startup reference is 12+7=19 frames from
[Ultimate Frame Data](https://ultimateframedata.com/pac_man), not projectile
arrival time. This is an immediate throw input, not an immediate hitbox.

The proposed fruit-double-hit-to-red trampoline interaction has not been
verified. These responses neither guarantee that conversion nor an automatic KO;
test each fruit/spacing/get-up combination in Training before relying on it.

## Lingering ghosts and long grab

Roll defaults to the long grab; its 22 active frames participate in protection
and reach checks. Lingering ghosts use the bounded down-smash planning window
inside native activity rather than extending a hitbox. Shield-to-trampoline is
an alternative that returns control on native startup. Fruit inventory, hydrant
setups and automatic throw/combo follow-ups are not implemented.
