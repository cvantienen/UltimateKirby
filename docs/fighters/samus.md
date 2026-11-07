# Samus

<!-- BEGIN GENERATED FIGHTER REFERENCE -->

[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)

Native fighter kind **3** (`samus`). This reference follows the canonical fighter data;
it is not a device-calibration result. No fighter-specific file loads during a match.

## Saved ledge responses

The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.
Up/Down selects a row; Left/Right cycles its
choices. Hold **L** to run the selected responses. Defaults are bold.

| Row | Saved choices, in menu order |
| --- | --- |
| Neutral | Nair; **Dtilt**; Ftilt; SH Zair; Homing Msl; Super Msl; Cannon f-smash |
| Jump | **Nair**; SH Zair; Flame fair; Uair; Homing Msl; Super Msl |
| Roll | **Dsmash**; Grab; Move > zair; Homing (stay); Super (stay) |
| Attack | Shield > Nair; Shield > range-checked grab; **Shield Up-B**; Shield > Zair; Shield > Fair; Shield > Uair |
| Drop | **Shield**; Dtilt > Shield; Bomb (now); SH Zair; Homing Msl; Super Msl; Flame Fair (stay) |

These are the compact UI names; SH means short hop and Msl means missile.
Input timelines hand back control after their last step; the game decides acceptance.
Selecting a choice never changes native moves, hitboxes, physics or animation speed.

## Saved assist rules

These rules have separate ON/OFF assists and Training L3/R3 rule lists;
they are not selected ledge/button recipes. Auto react has one sequence per event;
priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.

| Assist trigger | Priority | Saved rule | Section |
| --- | --- | --- | --- |
| `auto_air_dodge` | — | Air dodge > Up B | `samus.roll.auto_air_dodge` |
| `auto_grab` | — | Automatic grab (unshielded) | `samus.roll.auto_grab` |
| `auto_grab_dodged` | — | Dodged grab > Up B | `samus.roll.auto_grab_dodged` |
| `auto_roll` | — | Roll > Up B | `samus.roll.auto_roll` |
| `auto_shield_backup` | — | Shield backup > Up B | `samus.roll.auto_shield_backup` |
| `auto_shield_grab` | — | Automatic grab (shielded) | `samus.roll.auto_shield_grab` |
| `auto_spot_dodge` | — | Spot dodge > Up B | `samus.roll.auto_spot_dodge` |

## Native move reference (not scheduling)

These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit
delays or proof that every phase of a multihit move stays active.

| Move | Stored frames |
| --- | --- |
| Jab 1 | 3 start; active 3–4; 17 total; -- landing |
| Neutral Air | 8/14 start; active 8–11 / 14–15 / 16–22; 45 total; 9 landing |
| Forward Air | 6/12/18/24/30 start; active 6–7 / 12–13 / 18–19 / 24–25 / 30–31; 59 total; 14 landing |
| Back Air | 9 start; active 9–10 / 11–14; 41 total; 14 landing |
| Down Air | 17 start; active 17–18 / 19–21 / 22–23; 48 total; 12 landing |
| Forward Tilt | 8 start; active 8–10; 33 total; -- landing |
| Up Tilt | 15 start; active 15–18; 39 total; -- landing |
| Down Tilt | 6 start; active 6–8; 44 total; -- landing |
| Dash Attack | 8 start; active 8–9 / 10–13 / 14–18; 41 total; -- landing |
| Forward Smash | 10 start; active 10–11; 48 total; -- landing |
| Up Smash | 11/15/19/23/27 start; active 11–12 / 15–16 / 19–20 / 23–24 / 27–28; 56 total; -- landing |
| Down Smash | 9/17 start; active 9–10 / 17–18; 44 total; -- landing |

### Grab data

| Setting | Fighter data |
| --- | --- |
| Standing grab | frames 15–22 |
| Dash grab | frames 17–24 |
| Pivot grab | frames 18–25 |
| Standing reach reference | 42.2 world units |
| Shield-grab startup | 19 |

Reach is an approximate root-based reference, not an animated grab/hurtbox model.
The file decides whether to request an explicit range check. No native timing gate
is inferred from this table; input delivery and the game's buffering rules still matter.

## Baseline fighter stats

From the canonical [Fighter Data catalog](../FIGHTER_DATA.md); baseline facts,
not instance modifiers or a replacement for live observations.

| Stat | Fighter data |
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

## Execution and calibration

Option definitions and input delays come from `fighters/samus.txt`, assembled into the Switch’s `sequences.txt`. Each option
has an explicit `start_delay`; waits between actions are literal native frames.
No compiled fighter recipe, adaptive-hop policy or native move window schedules them.

| Situation / ID | Start delay | Ordered inputs |
| --- | --- | --- |
| Neutral / `nair` | 13 | `opponent_input:a+x:0:0:1` |
| Neutral / `down_tilt` | 27 | `opponent_input:a:0:-0.6:1` |
| Neutral / `forward_tilt` | 25 | `opponent_input:a:0.6:0:1` |
| Neutral / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Neutral / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Neutral / `super_missile` | 0 | `face:opponent, side_special:1` |
| Neutral / `forward_smash` | 0 | `opponent_input:a:1:0:26, release:1` |
| Jump / `nair` | 2 | `opponent_input:a+x:0:0:1` |
| Jump / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Jump / `forward_air` | 5 | `opponent_input:a+x:1:0:1` |
| Jump / `up_air` | 6 | `opponent_input:a+x:0:1:1` |
| Jump / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Jump / `super_missile` | 0 | `face:opponent, side_special:1` |
| Roll / `down_smash` | 0 | `if_facing_opponent, opponent_input:a:0:-1:25, release:1, else_facing, opponent_input:a:0:-1:17, release:1, end_facing` |
| Roll / `grab` | 16 | `check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Roll / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Roll / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Roll / `super_missile` | 0 | `face:opponent, side_special:1` |
| Attack / `shield_nair` | 0 | `shield_until_block:90, opponent_input:a+x:0:0:1` |
| Attack / `shield_grab` | 0 | `shield_until_block:90, check:enemy_in_front:0, range:fighter_range_instant, input:r:0:0:1` |
| Attack / `shield_up_b` | 0 | `shield_until_block:90, input:b:0:1:1` |
| Attack / `shield_zair` | 0 | `shield_until_block:90, check:enemy_in_front:0, input:x:0:0:1, wait:2, check:self_airborne:0, input:r:0:0:1` |
| Attack / `shield_fair` | 0 | `shield_until_block:90, opponent_input:a+x:1:0:1` |
| Attack / `shield_up_air` | 0 | `shield_until_block:90, opponent_input:a+x:0:1:1` |
| Drop / `shield` | 0 | `shield:255` |
| Drop / `down_tilt_shield` | 0 | `down_tilt:1, wait:30, shield:255` |
| Drop / `bomb` | 0 | `face:opponent, down_special:1` |
| Drop / `zair` | 0 | `short_hop:1, wait:2, grab` |
| Drop / `homing_missile` | 0 | `face:opponent, side_special:1:0.5` |
| Drop / `super_missile` | 0 | `face:opponent, side_special:1` |
| Drop / `forward_air` | 0 | `face:opponent, short_hop_fair:1` |

See [this fighter's detailed execution notes](../SAMUS_LEDGE.md) for its specific sequences,
timing overrides, drift and exceptions.

Spacing, resource requirements, opponent steering, stage geometry and online
input arrival still affect contact. There is no learned-delay compensation.
Use the [diagnostic result card](../CALIBRATION_CARD.md),
[execution/calibration](../FIGHTER_EXECUTION.md) and the
[roster/data catalog notes](../NON_DLC_ROSTER.md)
before treating an option as reliable in a particular setup.

## Source and edits

Paths are relative to the repository root:

- Registration: `crates/kirby_assist_core/src/fighter_profiles/samus.rs`.
- Stats and move frames: `data/Ultimate_stats.json`.
- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.
- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.

[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.
Recipes are saved file configuration. External file edits need a Smash restart;
Rust/reference-data changes need a separately authorized build/install.
Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.

<!-- END GENERATED FIGHTER REFERENCE -->

> Historical implementation notes: the old conditional fighter engines described below have been retired. Current options and exact input delays are the file-authored schedules above and in `sequences.txt`; native reference facts do not add scheduling.


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
