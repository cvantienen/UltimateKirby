# Shield pressure rules

Shield pressure watches **the opponent's shield and rolls out of shield**. It is
separate from Out of shield, which reacts after an attack hits **your** shield.
Ordinary Roll reactions recognize rolls; they do not by themselves remember that
the opponent was shielding first.

## Create a mode in Training

1. Open **Assist modes → Shield pressure → L3/R3**. This opens a separate priority
   list, not a ledge-getup list. X adds a rule; A/R3 edits the selected rule.
2. Choose **Auto reaction**, your state, enemy side, X distance, height and priority.
   Configure the input sequence and delays just like other assist rules.
3. Press Y in the editor to save. R3/L3 returns to the list. Its manage menu can
   rename, duplicate and delete rules. Enable **Shield pressure** in Assist modes.

The switch starts ON in a fresh menu, but nothing runs until this fighter has
saved Shield pressure rules. The existing text-file validator supports the new
triggers. Use the updated mod to load files containing them.

## Events

| Event token | When it matches |
| --- | --- |
| `shield_pressure_vulnerable` | A held shield or shield-origin roll is currently vulnerable. One rule can prioritize a close front grab for both. |
| `enemy_shield` | Enemy is raising, holding or taking a hit on their shield. Shield release alone does not match. |
| `shield_roll` | Enemy is rolling after an observed shield state. |
| `shield_roll_cross` | That roll has crossed your relative horizontal position. Your own movement can cause the crossing too. |
| `shield_roll_vulnerable` | Shield-origin roll is now vulnerable according to the existing native forecast. |
| `shield_roll_cross_vulnerable` | Both crossing and vulnerability are observed. |
| `shield_roll_away` | Shield-origin roll's observed horizontal velocity points away from your position at its start. No direction is guessed while velocity is zero. |
| `shield_roll_away_vulnerable` | Both rolling away and vulnerability are observed. |

The mode must observe the shield before the roll; enabling it midway through a
roll cannot establish that history. Jump, attack and spot-dodge escapes do not
have shield-origin events in this version. Spot punish still has its own list.

## Example Bowser setup

These are **starting templates**, not verified punish timings. Tune root-distance
bounds and delays in Training. Each rule's priority only compares rules whose
conditions match now; it does not wait for a higher-priority future event.

| Priority | Event / own state | Example range | Response |
| --- | --- | --- | --- |
| 1 | Enemy shielding / grounded | 0–10, in front | Grab |
| 1 | Shield roll crosses / shielding | 0–14, either side | Up-B from shield |
| 2 | Shield roll crosses / grounded | 0–14, either side | Down smash |
| 3 | Shield roll away / grounded | 14–30, in front | Walk closer, then Up-B |

Shielding is also grounded: the priority-1 shielding rule wins over the broader
priority-2 rule when both match. A raw smash while you hold shield needs native
shield-drop recovery; it is not automatically an instant out-of-shield smash.
Choose a legal shield response or explicitly configure the release and wait.

```ini
[bowser.attack.pressure_grab]
name = Pressure close grab
start_delay = 0
reaction = enemy_shield:grounded:0:10:12:front:1
actions = hold:r:4

[bowser.attack.pressure_cross_shield]
name = Pressure cross from shield
start_delay = 0
reaction = shield_roll_cross:shielding:0:14:12:any:1
actions = up_b:4

[bowser.attack.pressure_cross_smash]
name = Pressure cross smash
start_delay = 0
reaction = shield_roll_cross:grounded:0:14:12:any:2
actions = down_smash:4

[bowser.attack.pressure_away]
name = Pressure chase away
start_delay = 0
reaction = shield_roll_away:grounded:14:30:12:front:3
actions = walk_to_opponent:80%:20:8, up_b:4
```

`hold:r:4` uses the existing mapped Grab input for four frames; it assumes your
usual R-to-Grab controller mapping. It does not check actual grab reach. Reduce
its distance bound or add a range check if needed. A roll can be invulnerable
when crossing: allow for your move's startup, or use the vulnerable variant.
Waiting until vulnerability may be too late for a slow move. The editor does not
calculate an optimal response or automatically retime hitboxes.

## Cooperation and cost

- One response runs at a time. Protected ledge visits, safety and existing committed
  actions retain control. Incoming-grab defense retains its higher priority.
- A hit on your shield cancels pressure playback and releases its ownership before
  the Out of shield assist evaluates that hit.
- If enabled Shield pressure has any shield-roll rules, observed shield-origin
  rolls belong to this mode. Ordinary Roll reactions still handle other rolls.
  There is no fallback to generic Roll reactions for a shield-origin roll whose
  pressure conditions never match. Disable Shield pressure to return those rolls
  to the ordinary roll mode.
- There is one response per shield visit and per shield-origin roll. It does not
  repeatedly grab a shield that remains held. Once playback starts, a later roll
  does not replace that committed sequence. Keep close-grab responses short.
- At most eight rules use the shared bounded eight-fighter observation pass.
  No additional game hook, projectile scan, file access or allocation is added to
  per-frame matching. Ground-movement recipes use existing floor-support checks.
  With no configured rules, the mode does no runtime observation work.

Grounded is a condition, not permission to cancel any animation. New responses
wait for the existing native action/cancel gate. Simulated inputs remain subject
to recovery, shieldstun and online input delivery. This mode changes no physics,
hitboxes, move definitions, animation rates or fighter statuses.

The current Bowser file uses the [simple three-rule profile](fighters/bowser.md),
not every optional example above. Its immediate shield-roll Up-B fallback can
start while a close roll is still invulnerable; priority does not wait for the
later grab window. Shield candidates use their native hit status, sharing the
existing follow-up read when already locked. There is no new opponent scan.
