# Hold Y: timed ledge responses with manual spacing

Hold **Y** to engage; release it to disengage. Y is consumed, and the automatic
jump uses **X**. L3 remains turbo teabag. Use A=Attack, X=Jump, R=Grab, ZR=Shield,
and Tap Jump off.

These controls apply to every human-controlled fighter on the selected player
input slot in all battle modes. Move timings and ranges still use Kirby defaults.

The default `ledge_distance_agnostic=true` policy leaves normal/jump/get-up-attack
spacing to you. Opponent acquisition and those responses ignore horizontal and
vertical range gates. Distance still ranks eligible ledge targets; it cannot
prevent acquiring the only eligible target. The assist selects a response from
the opponent's observed ledge option and schedules inputs using vulnerability,
recovery and native action readiness.

| Opponent option | Response |
| --- | --- |
| Hang | Keep manual capped walking. |
| Normal get-up | Short-hop nair, targeting the first active hitbox three frames before known invincibility ends. Neutral stick avoids facing-dependent aerials. |
| Jump get-up | Prepare a short hop, release X during squat, then neutral-stick A as soon as aerial input is legal after the committed takeoff. Ignore reach and height. |
| Roll | Predict a short-horizon roll endpoint and approach on supported ground; release horizontal movement, then pulse down-stick+A for down smash during the predicted vulnerability window. |
| Get-up attack | Hold shield; after a confirmed block, buffer grab regardless of distance or target protection metadata. If facing away, jump directly out of shield and request neutral air at the first observed aerial-ready update; do not turn or grab. No whiff-only grab in this mode. |
| Generic ledge drop | Hold shield as fallback. Recognized ledge options take priority. |
| Confirmed capture | Optional outward throw when an anchor is available. |

Manual horizontal input stays capped at `ledge_walk_strength` while waiting.
Only roll approaches automatically; normal-getup turning is a bounded facing
correction. No automatic return to a standing spot occurs. Actual punish recipes
briefly own their necessary inputs, then restore player spacing. L3 movement
remains independent, with automatic punish inputs taking priority.

## Configuration

```text
ledge_trap = true
ledge_manual_spacing = true
ledge_distance_agnostic = true
ledge_walk_strength = 0.35
ledge_drop_defense = true
ledge_getup_attack = neutral_air
ledge_roll_dash_range = 10.0
ledge_follow_range = 40.0
```

Distance-agnostic mode defaults to true even if an old config omits the key.
Setting it false restores the previous reach-gated normal/jump/grab policy.
Legacy tilt, grab, anti-air and acquisition distances remain parsed but do not
gate those responses in the default mode. Roll approaches a predicted endpoint
within `ledge_roll_dash_range` plus `ledge_follow_range` on supported floor.
`ledge_roll_dash_range` is the approach stopping distance; its old
`ledge_roll_smash_range` alias still works. Once the punish is due, timing takes
priority over reaching that exact distance. Native down-smash readiness is
required so an uncancellable dash cannot turn the attack into dash attack.

## Timing and limits

[Kirby's frame data](https://ultimateframedata.com/kirby) gives nair startup 8
and active frames 8–32, and down smash startup 7 with front/back active phases
through frame 19.
Nair is one lingering hit with weakening phases. A proposed contact must overlap
known vulnerability and precede any known action deadline. Grounded recovery uses
the earlier valid cancel/animation-end frame. Missing protection timing waits
for observed vulnerability; it does not assume protection ended.

Native transitions still control when Kirby can actually jump, attack or grab.
Normal/jump timing ignores collision geometry by request: an attack can come out
from across the stage and miss. The front-facing block-triggered grab ignores reach. The rear-side response
preserves shield through shieldstun, uses the native jump-cancel route directly
from guard, releases X during jump squat, then requests neutral-stick A as soon as
the owned hop is aerial-attack-ready. This avoids turning and guard-off recovery.
It attempts one actual hop per block; distance and facing do not gate its nair.
A committed ledge-jump intercept also completes its nair once aerial-ready,
even if the opponent's recovery estimate has expired. It does not chase a target
that has already landed.

Input timing starts only after the original mapper produces the expected action
button. Clean native action starts measure extra delay using motion-frame
advancement divided by native motion rate. Mapper polls, wall time and hitstop
are not counted. Interrupted actions, motion loops, rate changes, repeated input
pulses and missed first-action frames are discarded rather than guessed. The
recent eight clean samples supply a bounded upper-envelope compensation shared
by these input recipes. Unknown delay starts at zero and resets with the gameplay
session. Diagnostics report clean samples and the applied compensation.

Every forecast is recomputed against the current opponent protection/recovery
window with that delay included. This is a sampled animation-based estimator,
not access to the game's online input queue or a global simulation clock. It
cannot learn from frozen/ambiguous animations, undo delay that already occurred,
or establish guaranteed hits. Switch testing is still required.

Damage, target loss, tracking timeout, release of Y, mash and tech retain their
cancellation/priority rules. Hitstop preserves shield. Exact native attack
status/motion plus a registered hit/capture confirm success; a button request
alone does not. Diagnostics retain their throttled asynchronous output.

These are source changes. Install a newly built NRO and the intended config, then
restart Smash to use them.

Normal get-up now defaults to `neutral_air` (`nair` is also accepted). The
bundled config selects it; an existing config explicitly selecting a legacy tilt
continues to select that tilt until updated. Hop scheduling includes three jump
squat frames, eight nair startup frames and the learned delay for each of the
two input requests. It launches when that total is within three frames of known
protection expiry, then releases X and requests neutral A once airborne-ready.
Late recognition/action readiness can reduce that lead. Unknown protection waits
for observed vulnerability. The active hitbox still needs correct player spacing.
