# Samus ledge responses

These are compiled Samus functions, selected through the existing battle menu.
Hold L to use the saved responses. No fighter files load during the match.

| Row | Available responses | Default |
| --- | --- | --- |
| Neutral | Nair, Down tilt, Forward tilt, Short-hop zair, Homing missile, Super missile, Cannon f-smash | Down tilt |
| Jump | Nair, Short-hop zair, Flame fair, Up air, Homing missile, Super missile | Nair |
| Roll | Down smash, Grab, Move → zair, Homing (stay back), Super (stay back) | Grab |
| Attack | Shield → nair, Grab / rear nair, Shield → Up-B, Shield → zair, Shield → fair, Shield → up air | Shield → Up-B |
| Drop | Shield, Down tilt → shield, Immediate down-B, Short-hop zair, Homing missile, Super missile, Flame fair (stay back) | Immediate down-B |

## Ranged coverage and extra aerials

Missiles are selectable in Neutral, Jump, Roll and Drop. Each starts one grounded
side-B attempt at the first actionable opportunity after recognizing that option,
turning toward the opponent if needed. A neutral primer precedes moderate
horizontal+B for homing, or a full flick+B for Super Missile. The bounded native
input path holds B with its direction through delivery and releases on generic
side-special or the corresponding Samus grounded missile startup. It then hands
control back, including on a whiff. Release/re-hold L to start a new attempt.
Target loss after commitment doesn't drop the paired input. These options do
not approach the roll endpoint, jump after the projectile, or fire a barrage.

Missile startup is **18** for homing and **21** for Super. Those values do not
include flight time. Super Missile initially stalls before accelerating, and
homing trajectories depend on the target. These choices therefore fire early
for area denial rather than inventing a distance/speed calculation and promising
a frame-perfect hit. No weapon scan, travel simulation, homing edits or new
opponent search is added. The label `~ranged / travel` also appears in Roll,
because these two roll choices remain stationary rather than chase.

Use missiles for distant pressure, zair for extended horizontal tether coverage,
and nair for nearby melee coverage. Nair does not gain projectile range. Ground
missiles can pass below jumps or above dropped opponents; stage geometry and
projectile limits still apply. A reactive missile from far away can arrive after
the opponent can shield, evade or reflect it. Existing tracking settings still
apply if distance-agnostic tracking is disabled.

New jump choices use fair's first hit **6–7** or up-air's first hit **5**, with the
existing adaptive hop policy. Shield versions wait for a witnessed block and
shieldstun ending, using the profile's shield hop policy. Later multihits continue
naturally but their inactive gaps aren't counted as continuous coverage. Drop
fair immediately short-hops without post-acceptance forward drift; it doesn't
chase below the ledge. Neutral forward-smash uses **10–11**, a facing correction
and the existing flick primer/release-on-startup path. Existing defaults and
choice indices remain unchanged.

## Zair delivery and timing

[Samus frame data](https://ultimateframedata.com/samus) lists zair active on
animation frames **8–15**, then **16–19**. Those are twelve consecutive animation
frames across two phases with different hitbox geometry; a single point is not
necessarily covered for all twelve.

Neutral, Jump and Roll scheduling aims protection expiry near the middle of
that span: nominally **five frames after the first active frame**, leaving six
later frames. The forecast uses zero extra live input-delay compensation and a four-frame
takeoff/delivery allowance. Actual native acceptance, spacing and height can
shift contact. Unknown protection/recovery timing does not become a guessed hit.

The input sequence is X first, release X during observed jump squat, then R
only after airborne state is observed. Grounded X+R is deliberately avoided.
Airborne R stays held through native delivery until **AIR_LASSO (0xF8)** confirms
zair, with a bounded native-progress timeout. Input-delay learning recognizes
that status rather than expecting a ground grab. No extra air jump is issued.

After acceptance, R releases and neutral drift remains through native frame 19
without fast fall. Landing, damage, unexpected tether/recovery states, expiry
or releasing L ends ownership. One attempt finishes with manual control; it
does not chase a whiff, force recovery or extend the game's active frames.

Shield → zair waits for a witnessed block and shieldstun ending. It requires
the opponent in front; it keeps shielding when facing backward. Drop → zair
starts immediately after a witnessed drop, with a bounded grounded turn if
needed. It is horizontal coverage and can miss an opponent already too low.

Roll → zair uses the existing locked roll endpoint and braking, with a compiled
12-world-unit standoff from that endpoint. That offset is a coverage preset,
not a universal measured hitbox distance. It faces the destination and does not
chase the opponent's moving roll root.

## Roll grab and drop Bomb

Samus standing grab is active on **15–22**; dash grab **17–24**; pivot grab
**18–25**. Roll → Grab approaches the committed roll point, releases movement,
faces the opponent and schedules protection expiry roughly three frames into
the real eight-frame grab window. Native capture immediately returns control
for your pummels and throws.

Drop → down-B requests down+B at the first native actionable opportunity after
recognizing the drop. There is no facing, reach, vulnerability or explosion
prediction gate. It preserves paired down+B through the existing bounded input
delivery path, then hands back on generic down-special or Samus ground/air
Bomb startup (**0x1DF / 0x1E6 / 0x1E7**).

Bomb is generated on frame **11**, its collision begins on frame **33**, and its
explosion timing depends on contact or the fuse. Immediate input cannot make
that delayed attack hit immediately. Weapon lifetime and move properties are
unchanged.

## Verification limits

Frame values were checked against Ultimate Frame Data and native status IDs
against the pinned skyline-smash bindings. Regression source covers catalog
dispatch, neutral/jump/drop/shield zair inputs, target-loss continuation,
hitstop, full active-window ownership, roll destinations, Bomb startup and grab
handoff. No Rust tests, build, upload or Switch match testing ran for this edit.
Additional source fixtures cover distant missile dispatch in both directions,
all four missile rows, neutral priming, target loss, frozen native clocks, and
release/manual handback on missile startup. Formatting and static registration
checks are separate from execution; missile type selection and actual contact
still require device validation.
Spacing, opponent DI and online timing still need device validation; these
responses do not guarantee a punish.
