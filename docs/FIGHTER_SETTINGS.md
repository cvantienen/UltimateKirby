# Fighter settings in simple terms

These fighter-setting keys include seven retired keys
and two legacy automatic-grab range keys retained for file compatibility. Some entries appear only for
Kirby or Yoshi, or affect older helpers rather than your custom rule sequences.
The text keys below belong under `[bowser.settings]`, `[kirby.settings]`, etc.

Open **Minus → Fighter settings** in Training. Up/Down selects a value;
Left/Right changes it; **X** restores its configured startup value; **Y alone**
saves; **R3** returns. Wait for the save result. See the
[menu controls](LEDGE_MENU.md#fighter-settings-in-training).

Distances are **game world units**, not frames. X means horizontal distance;
Y means vertical distance. Most range checks compare the fighters' reference
positions near their bodies, not the outer edges of their hands or hitboxes.
Stick percentages mean simulated left-stick strength, not a change to walk speed.
All settings affect input decisions only: they never change physics, hitboxes,
grab length, move speed, or native invincibility.

## Tilt facing

The **Tilt face** mode must be ON. These saved options independently select
which grounded moves it may aim toward the nearest opponent.

| Menu name | Text key | Default and input |
| --- | --- | --- |
| Tilt face: down | `tilt_turn_down` | ON for Kirby/Yoshi, OFF for others. Prepares held Down or a physical down C-stick tilt. |
| Tilt face: forward | `tilt_turn_forward` | OFF. Aims a physical side C-stick tilt. |
| Tilt face: up | `tilt_turn_up` | OFF. Aims a physical up C-stick tilt. |

Forward and up require a C-stick configured as **Tilt Attack** in the game;
the assist cannot verify that mapping. With Smash Attack mapping, enabling them
can redirect smashes. They leave left-stick+A attacks alone. See
[input recipes](INPUT_RECIPES.md#grounded-posture-menu-rows) for priorities
and limitations.

## Walking and getting into ledge position

| Menu name | Text key | Simple explanation / changing it |
| --- | --- | --- |
| Walking stick % | `walk_strength` | Baseline for slow walking while holding L. Higher allows stronger partial-stick input. The existing curve still uses 50% at full horizontal stick; this is not an exact speed setting. |
| Grab reach estimate | `grab_range` | How far the assist thinks **this fighter's** grab can reach. Higher lets range-checked grabs attempt from farther away; too high means more misses. Used by `grab:fighter_range` and supported grab helpers. If this fighter is the opponent, its saved override also affects incoming-grab estimates. It does not change another fighter's grab reach. |
| Ledge acquire X | `horizontal_range` | How far left/right you can be from the ledge opponent when tracking **starts**. Higher starts tracking from farther away. Requires range limits ON. |
| Ledge acquire Y | `vertical_range` | How far above/below the ledge opponent you can be when tracking **starts**. Higher accepts larger height differences. Requires range limits ON. |
| Ledge follow X | `follow_range` | How far left/right an **already tracked** opponent can move before tracking is lost. Higher keeps the target longer. Requires range limits ON. |
| Ledge follow Y | `follow_vertical_range` | How far above/below an **already tracked** opponent can move before tracking is lost. Higher keeps tracking through larger height differences. Requires range limits ON. |
| Ledge stand distance | `stand_distance` | Stops held-L walking closer to the opponent's known ledge. Walking away is allowed even inside this gap. Higher stops farther back. The remembered inward direction updates if the opponent changes ledges. Also used by automatic stance positioning. |
| Auto-position stick % | `move_strength` | How strongly the assist pushes the stick when automatically moving into its ledge stance. Higher moves more forcefully. Your recipe's Walk/Run/Drift strengths remain separate. |
| Manual walking cap | `manual_spacing` | ON enables the held-L walking limits. OFF disables that manual spacing path. |
| Auto ledge stance | `auto_position` | Enables the existing automatic stance-positioning path. It also requires Manual walking cap OFF and range limits ON. It does not replace movements written into recipes. |
| Use acquire/follow limits | `use_range_limits` | ON uses the four acquire/follow distances above. OFF ignores those distance limits for tracking. |

**Acquire = find the target. Follow = keep the target.** For example, acquire X
20 and follow X 40 means “start tracking within 20 horizontal units; once found,
keep tracking until the gap exceeds 40.” These are distances to the opponent,
not your minimum safe gap from the ledge. **Ledge stand distance** controls that gap.

The walking stop uses the opponent's tracked/recently grabbed ledge, not every
platform edge. Without a known ledge, it can limit walking strength but has no
ledge position to stop against. The Ledge trap mode toggle alone does not impose
manual walking limits; holding L does. [More about L](LEDGE_MENU.md#manual-walking-stop-distance).

## Grab dodge and out-of-shield startup

| Menu name | Text key | Simple explanation / changing it |
| --- | --- | --- |
| Grab dodge + punish at startup | `grab_dodge_default` | Whether this mode starts ON for this fighter in a fresh match. Does not immediately change the current match's switch. |
| Out of shield at startup | `out_of_shield_default` | Whether OOS starts ON for this fighter in a fresh match. Current live mode requires enemy shield for unshielded grabs; your shield permits ordinary vulnerable front grabs and close shield Up-B backup; legacy OOS rules remain readable. |
| Grab dodge forecast frames | `grab_dodge_lookahead` | How many frames ahead to look for grab contact, from 1–8. Higher can dodge earlier; lower waits until nearer contact. Too early can also be wrong. |
| Grab dodge: body padding | `grab_dodge_padding` | Extra allowance for **your body** around the attacker's grab reach. Higher treats more near-miss positions as dangerous and can help with large fighters. It does not enlarge the real grab. |
| Grab dodge: height tolerance | `grab_dodge_height` | Maximum height difference between your position and the grabber for a warning. Higher covers larger height differences but can react to grabs that would miss vertically. |

For Bowser being grabbed by DK, Bowser's **body padding** and **forecast frames**
affect the defense. Bowser's **Grab reach estimate** describes Bowser's own grab;
DK's grab warning uses DK's reference reach or DK's saved override.

The current grab-dodge follow-up has its own **Auto react → After dodging enemy
grab** sequence, independently of the Auto react toggle. Unshielded automatic
grabs require visible enemy shield; increasing their range does not bypass that
condition. Your shield-grab recipe does not require enemy shield.

## Incoming-attack shield prediction

These tune the existing defensive shield prediction. They do not choose the
move used after a confirmed shield hit; that belongs to the OOS rule's actions.

| Menu name | Text key | Simple explanation / changing it |
| --- | --- | --- |
| Shield: warning X | `shield_warning_x` | Horizontal warning distance for verified approaching attacks. Higher considers attacks from farther left/right. It is not the attack's actual reach. |
| Shield: warning Y | `shield_warning_y` | Vertical warning distance. Higher considers attacks from farther above/below. |
| Shield: body half-width | `shield_body_half_width` | Estimated distance from your body's center to either side. Higher predicts more horizontal hits against your body. |
| Shield: body height | `shield_body_height` | Estimated body height above your reference position. Higher includes attacks passing higher above your feet. |
| Shield: body half-depth | `shield_body_half_depth` | Estimated half-size in the screen's depth direction. Higher includes more depth overlap in hitbox checks. This is different from left/right X. |
| Shield: forecast frames | `shield_forecast` | How far ahead to predict an incoming hit. Higher can shield earlier, with more prediction uncertainty. |
| Shield: release-tail frames | `shield_hold` | Extra shield-hold time after a warning disappears. Higher is more cautious but can keep you shielding longer. Ledge regrab handling can clear the old tail. |
| Shield: scan X cutoff | `shield_scan_x` | Coarse horizontal cutoff before inspecting attack hitboxes. Higher lets farther candidates reach those checks and may increase work. Movement can expand the cutoff. |
| Shield: scan Y cutoff | `shield_scan_y` | Coarse vertical cutoff before inspecting attack hitboxes. Higher includes more above/below candidates and may increase work. Movement can expand the cutoff. |

## Auto-tech collision probes

A **probe** is an invisible prediction line used to look for a stage collision.
It does not change the character's collision shape. These are placement estimates,
not “better tech” sliders: larger values can also predict a false/early collision.
Native tech eligibility, lockout and air-dodge safety checks still apply.

| Menu name | Text key | Simple explanation / changing it |
| --- | --- | --- |
| Tech probes: half-width | `tech_half_width` | How far left/right the outer body probes sit from your position. Higher checks farther to either side. |
| Tech probes: floor origin Y | `tech_floor_origin` | Height above your position where floor checks begin. Higher starts the floor prediction line higher. It is not a delay. |
| Tech probes: low wall Y | `tech_wall_low` | Height of the lowest sideways wall-check line. Higher moves that line upward. |
| Tech probes: middle wall Y | `tech_wall_mid` | Height of the middle wall-check line. Higher moves that line upward. |
| Tech probes: high wall Y | `tech_wall_high` | Height of the highest wall-check line. Higher moves that line upward. |
| Tech probes: ceiling origin Y | `tech_ceiling_origin` | Height where upward ceiling checks begin. Higher starts nearer overhead surfaces. |
| Tech probes: floor extra reach | `tech_floor_padding` | Extra downward distance added to the predicted floor sweep. Higher can notice a nearby floor earlier, but may warn before real contact. |
| Tech probes: ceiling extra reach | `tech_ceiling_padding` | Extra upward distance added to ceiling prediction. Higher can notice a ceiling earlier, but may warn before real contact. |

## Retired fixed-punish settings

These seven keys are still accepted in old files, but are **hidden from Fighter
Settings and have no effect on live automation**. The fixed helper that used
them has been removed from live execution. Use your reaction rule's range, Side,
height, priority and actions instead.

| Former menu name | Text key | Meaning now |
| --- | --- | --- |
| Legacy punish: down-smash reach | `punish_down_reach` | Retired; does not change custom down-smash rules. |
| Legacy punish: forward-smash reach | `punish_forward_reach` | Retired; does not change custom forward-smash rules. |
| Legacy punish: height tolerance | `punish_height` | Retired; use each rule's height. |
| Legacy roll: smash height | `roll_smash_height` | Retired; use each rule's height. |
| Legacy roll: smash watch distance | `roll_smash_watch` | Retired; use each rule's X range. |
| Legacy roll: grab height | `roll_grab_height` | Retired; use each rule's height. |
| Legacy roll: grab watch distance | `roll_grab_watch` | Retired; use each rule's X range. |

## Fighter-specific helpers

| Menu name | Text key | Simple explanation / changing it |
| --- | --- | --- |
| Kirby crouch: proximity X | `crouch_range` | How close horizontally an opponent must be for automatic crouching. Higher begins crouching with opponents farther away. |
| Kirby crouch: proximity Y | `crouch_height` | Allowed vertical gap for automatic crouching. Higher also considers opponents farther above/below. |

## Saved values versus defaults

An omitted text key uses the current default. A saved key overrides that value
for its fighter. Some defaults use fighter reference facts; body geometry and
many distance settings remain estimates because the reference data lacks them.
Changing a fighter setting does not rewrite distances or waits explicitly written
in a recipe. Turning a mode ON still requires a matching configured rule to run
that mode's custom response.

```ini
[bowser.settings]
stand_distance = 15
use_range_limits = true
horizontal_range = 25
vertical_range = 20
follow_range = 50
follow_vertical_range = 45
grab_dodge_lookahead = 7
grab_dodge_padding = 10
```

This example stops held-L walking 15 units from the known ledge, starts tracking
within the configured acquire gaps, keeps tracking within the larger follow gaps,
and gives Bowser's grab defense a seven-frame forecast with ten units of body
allowance. It does not change Bowser's physical body or DK's grab hitboxes.

Unshielded automatic grabs require an enemy **shielding, vulnerable and in front**.
While you hold shield, enemy shield is not required; the opponent must instead be
**vulnerable, grabbable and in front** within the shield-grab limits. Their
limits now belong with their actions in the Auto react sequence list:

```ini
[bowser.roll.auto_grab]
reaction = auto_grab:35:24
actions = input:r:0:0:4

[bowser.roll.auto_shield_grab]
reaction = auto_shield_grab:25:10
actions = input:r:0:0:4

[bowser.roll.auto_shield_backup]
reaction = auto_shield_backup:25:15
actions = up_b:4, drift:opponent:1:69
```

Add the normal `start_delay = 0` / timeout fields when creating a full section.
The first number is maximum horizontal separation and the second is height.
Movement, shield safety and native dash confirmation remain unchanged. Backup
has independent limits while retaining grab-first selection. Each fighter's
current effective ranges were preserved during migration.

Legacy keys `auto_grab_close_range` and `auto_grab_dash_range` remain accepted
under `[fighter.settings]` when no corresponding recipe exists. Explicit recipe
limits win; duplicate tuning sliders are hidden so changing an ignored legacy
value cannot appear to change an explicit sequence. Startup mode switches,
grab-warning forecast/body padding and other analysis settings remain here.
See [all Auto react entries](REACTION_RULES.md#current-simple-auto-react-mode).
