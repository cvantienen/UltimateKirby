# Pac-Man

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **55** (`pacman`). This reference follows the compiled source;
it is not a device-calibration result. No fighter file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; Dtilt; **Ftilt**; Ghost f-smash; Fruit (stored) |
| Jump | **Nair**; Uair; Fair; Fruit (stored) |
| Roll | **Lingering ghosts**; Long grab |
| Attack | **Shield > Nair**; Shield > range-checked grab; Shield Up-B |
| Drop | **Shield**; Dtilt > Shield; SH Bite; Low ghosts (spacing) |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Neutral Air | `pacman.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `pacman.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Neutral Air | `pacman.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Neutral Air | `pacman.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Neutral Air | `pacman.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `pacman.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Neutral Air | `pacman.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
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
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

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

Option definitions and input delays come from `fighters/pacman.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 20 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 26 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 28 | `opponent_input:a:0.6:0:1` |
| Neutral / `ghost` | 0 | `opponent_input:a:1:0:15, release:1` |
| Neutral / `fruit_throw` | 0 | `face:opponent, neutral_special:1` |
| Jump / `nair` | 0 | `opponent_input:a+x:0:0:1` |
| Jump / `up_air` | 0 | `opponent_input:a+x:0:1:1` |
| Jump / `forward_air` | 5 | `opponent_input:a+x:1:0:1` |
| Jump / `fruit_throw` | 0 | `face:opponent, neutral_special:1` |
| Roll / `down_smash` | 0 | `opponent_input:a:0:-1:13, release:1` |
| Roll / `grab` | 19 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |
| Drop / `ghosts` | 0 | `face:opponent, down_smash:1` |

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
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


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
