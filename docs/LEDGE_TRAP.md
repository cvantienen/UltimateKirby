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
| Normal get-up | Configured tilt, timed without reach checks; automatically turn toward the target if needed. |
| Jump get-up | Prepare a short hop, release X during squat, then neutral-stick A timed to vulnerability after native takeoff. Ignore reach and height. |
| Roll | Bounded supported-ground dash preparation, then directional A in native DASH/RUN. Known vulnerability beyond the active window delays the pulse; attack input does not wait for a range/recovery forecast. |
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
ledge_getup_attack = forward_tilt
ledge_roll_dash_range = 10.0
ledge_follow_range = 40.0
```

Distance-agnostic mode defaults to true even if an old config omits the key.
Setting it false restores the previous reach-gated normal/jump/grab policy.
Legacy tilt, grab, anti-air and acquisition distances remain parsed but do not
gate those responses in the default mode. Roll starts a bounded dash within
`ledge_roll_dash_range` plus `ledge_follow_range` on supported floor. Once Kirby
is observed in DASH/RUN, it requests A without a range or recovery forecast;
known vulnerability more than 34 frames away delays that request.
`ledge_roll_smash_range` remains an alias for
`ledge_roll_dash_range` for old configs.

## Timing and limits

[Kirby's frame data](https://ultimateframedata.com/kirby) gives nair startup 8
and active frames 8–32, and dash attack startup 9 and active frames 9–34.
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
It attempts one actual hop per block; distance and facing do not gate its nair. The recent roll gate loosening was reverted. The earlier tracking/approach and
facing, range and floor checks are restored. Preparation uses full stick; the
attack pulse uses neutral stick plus A after native DASH/RUN is observed. Clearing analog and digital directions on that pulse avoids
combining a fresh horizontal flick with A, which can select forward smash. Neither timing forecasts nor distance-independent input
requests establish guaranteed hits or frame-perfect execution without Switch tests.

Damage, target loss, tracking timeout, release of Y, mash and tech retain their
cancellation/priority rules. Hitstop preserves shield. Exact native attack
status/motion plus a registered hit/capture confirm success; a button request
alone does not. Diagnostics retain their throttled asynchronous output.

These are source changes. Install a newly built NRO and the intended config, then
restart Smash to use them.
