# Fighter settings: complete beginner reference

Fighter settings are **input-policy estimates**, not edits to Smash's character
parameters. They never enlarge hitboxes, shorten startup, change gravity or force
fighter status. A recipe requesting Down+B still asks Smash to accept Down+B.

## Where values come from

1. Shared settings are resolved from `[universal.settings]`, or the legacy
   config fallback when that section is absent; see [configuration](SEQUENCE_FILES.md).
2. The exact native fighter profile supplies grab reference facts. Missing facts
   stay unknown; another fighter is never substituted.
3. `[fighter.settings]` overrides individual policy values. The complete 83-key
   schema below comes from `fighters/fighter_tuning.rs`: `KEYS`, `DEFAULTS`,
   `Tuning::values`, `bounds`, `set` and `visible`.
4. A recipe's explicit ranges, delays and inputs are separate. Editing a tuning
   value does not rewrite an authored `range:20` or `wait:5`.

**Schema fallback is not supplied configuration.** The repository's universal
file currently supplies walking 0.35, acquire X/Y 20/10, follow X/Y 40/48,
stand distance 10, auto-position strength 0.65, manual spacing true,
auto-position false and distance-agnostic true (therefore use-range-limits false).
A fighter override may change these. Your installed file and unsaved menu may
also differ. Per-fighter pages list their actual repository overrides.

## Edit, apply, save and recover

Open Minus in Training, then **Fighter settings**. Up/Down chooses a visible
row; Left/Right changes it using the increment listed below. **X** restores the
configured startup value, not necessarily the Rust fallback. **Y alone** saves;
wait for the result before leaving. **R3** returns. Current numeric policy edits
can be used by live decisions; a row labelled **at startup** records a future
match preference, not a substitute for the current mode switch. Turn that mode
on/off on the ON/OFF page separately. Save is not automatic; external file edits
require a restart. See [menu controls](LEDGE_MENU.md) and [sequence persistence](SEQUENCE_FILES.md).

In the text file use the registered profile name, e.g.:

```ini
[lucina.settings]
grab_dodge_lookahead = 7
counter_default = true
counter_startup = 7
counter_margin = 3
```

This estimates incoming-grab contact seven frames ahead and saves Lucina's
counter preference/startup/margin. It does not change Lucina's native moves.
Edit an existing section/key rather than duplicating it. Keys and booleans are
case-sensitive; boolean values must be exactly `true` or `false`, not 0/1.
Numeric values must be finite and inside the inclusive bounds. Fractional frame
values are accepted: a “frames” label does not imply an integer parser. Only the
move selectors require integers. Menu increments are navigation steps, not file
quantization: a valid file number need not be a multiple of the increment.
Unknown keys, duplicate keys, NaN/infinity and out-of-range values are rejected.
Zero can be an unknown derived grab fallback even though `grab_range=0` is not a
valid override. Omit the key to keep reference-derived behavior.

## Units and prerequisite rules

- **World units** measure root/reference positions, not pixels or the distance
  between extended hands. X is horizontal, Y vertical; depth is a separate axis.
- **Stick fraction** 0.35 means 35% simulated stick strength, not 35% native speed.
  The manual walking curve still reaches 50% at full horizontal stick. Recipe
  movement strengths remain independent.
- **Frames** are policy prediction/hold estimates. Native acceptance, hitstop,
  movement and online input delay can defeat a well-timed request.
- Acquire finds a target; follow keeps it. Range limits OFF ignores those four
  tracking cutoffs. Stand distance is the inward gap from a **known opponent
  ledge**, not a universal stop at every platform edge.
- Automatic stance requires `manual_spacing=false`, `auto_position=true` and
  `use_range_limits=true`; its input never overrides movements authored in a
  committed recipe. A saved auto-position flag alone is insufficient.
- Automatic grabs currently require **your shield**, a vulnerable/grabbable
  front target and an admitted shield-grab recipe. `auto_grab` is a rejected
  retired reaction token. The approach-range key remains parseable/resolvable,
  but cannot enable removed unshielded-grab admission.
- Tilt-facing rows are shown only for Kirby, Yoshi and Lucina. Down defaults true
  for those three; forward/up default false. The supplied Lucina file disables
  all three. The current Tilt face switch must be enabled; side/up C-stick aiming
  requires the game's **Tilt Attack** mapping, which the mod cannot verify.
- Reflect rows are Game & Watch-only; counter rows are Lucina-only for the
  standalone auto-counter path. A universal held Counter/Auto Defense binding
  is a different tool; see [Counter](AUTO_COUNTER.md), [Defense](AUTO_DEFENSE.md).
- Auto Space is a held/bound tool, not enabled by choosing a move selector alone.
  Ground selectors: 0 disabled, 1 jab, 2 forward tilt, 3 up tilt, 4 down tilt,
  5 accepted but disabled. Air selectors: 0 disabled, 1 nair, 2 fair, 3 uair,
  4 dair, 5 bair. It never jumps merely to use an aerial. See [Auto Space](AUTO_SPACING.md).
- DI/SDI preferences are experimental; they are not terrain-aware guarantees.
  [Damage response](TRIP_AND_SDI.md) explains their live gates.

## All 83 accepted keys

`startup` = saved preference; current switch is separate. `current` = policy
value/estimate available to current decisions, subject to the prerequisites above.
`ignored` = accepted for compatibility but does not restore a removed feature.
“Hidden” means absent from Fighter settings, not rejected by the file parser.
The min/max columns are inclusive; the menu increment does not constrain file decimals.

| Key | Menu label / visibility | Schema fallback or derivation | Accepted range; menu increment | Units | Effect |
| --- | --- | --- | --- | --- | --- |
| `walk_strength` | Walking stick %; all fighters | resolved AssistSettings.walk_strength (supplied universal 0.35) | 0.1–0.5; 0.01 | stick fraction | current |
| `grab_range` | Grab reach estimate; all fighters | exact fighter standing grab reach; 0 when unknown (0 cannot be saved) | 1–60; 0.5 | world units | current |
| `horizontal_range` | Ledge acquire X; all fighters | resolved horizontal_range (supplied universal 20) | 1–150; 0.5 | world units | current |
| `vertical_range` | Ledge acquire Y; all fighters | resolved vertical_range (supplied universal 10) | 1–150; 0.5 | world units | current |
| `follow_range` | Ledge follow X; all fighters | resolved follow_range (supplied universal 40) | 1–150; 0.5 | world units | current |
| `follow_vertical_range` | Ledge follow Y; all fighters | resolved follow_vertical_range (supplied universal 48) | 1–150; 0.5 | world units | current |
| `stand_distance` | Ledge stand distance; all fighters | resolved stand_distance (supplied universal 10) | 1–150; 0.5 | world units | current |
| `move_strength` | Auto-position stick %; all fighters | resolved move_strength (supplied universal 0.65) | 0.1–1; 0.01 | stick fraction | current |
| `manual_spacing` | Manual walking cap; all fighters | resolved manual_spacing (supplied universal true) | true / false; toggle | boolean | current |
| `auto_position` | Auto ledge stance; all fighters | resolved auto_position (supplied universal false) | true / false; toggle | boolean | current |
| `use_range_limits` | Use acquire/follow limits; all fighters | inverse distance_agnostic (supplied universal false) | true / false; toggle | boolean | current |
| `grab_dodge_default` | Grab avoidance at startup; all fighters | true | true / false; toggle | boolean | startup |
| `out_of_shield_default` | Automatic grabs at startup; all fighters | true | true / false; toggle | boolean | startup |
| `grab_dodge_lookahead` | Grab dodge forecast frames; all fighters | 6 | 1–8; 0.5 | frames | current |
| `shield_warning_x` | Shield: warning X; all fighters | 24 | 0–100; 0.5 | world units | current |
| `shield_warning_y` | Shield: warning Y; all fighters | 28 | 0–100; 0.5 | world units | current |
| `shield_body_half_width` | Shield: body half-width; all fighters | 6 | 0–40; 0.5 | world units | current |
| `shield_body_height` | Shield: body height; all fighters | 18 | 0–60; 0.5 | world units | current |
| `shield_body_half_depth` | Shield: body half-depth; all fighters | 6 | 0–40; 0.5 | world units | current |
| `shield_forecast` | Shield: forecast frames; all fighters | 5 | 0–10; 0.5 | frames | current |
| `shield_hold` | Shield: release-tail frames; all fighters | 4 | 0–20; 0.5 | frames | current |
| `grab_dodge_padding` | Grab dodge: body padding; all fighters | 2 | 0–30; 0.5 | world units | current |
| `grab_dodge_height` | Grab dodge: height tolerance; all fighters | 8 | 0–60; 0.5 | world units | current |
| `tech_half_width` | Tech probes: half-width; all fighters | 2.5 | 0–30; 0.5 | world units | current |
| `tech_floor_origin` | Tech probes: floor origin Y; all fighters | 1 | 0–40; 0.5 | world units | current |
| `tech_wall_low` | Tech probes: low wall Y; all fighters | 1 | 0–40; 0.5 | world units | current |
| `tech_wall_mid` | Tech probes: middle wall Y; all fighters | 3.5 | 0–40; 0.5 | world units | current |
| `tech_wall_high` | Tech probes: high wall Y; all fighters | 6 | 0–60; 0.5 | world units | current |
| `tech_ceiling_origin` | Tech probes: ceiling origin Y; all fighters | 5 | 0–60; 0.5 | world units | current |
| `tech_floor_padding` | Tech probes: floor extra reach; all fighters | 1.5 | 0–15; 0.5 | world units | current |
| `tech_ceiling_padding` | Tech probes: ceiling extra reach; all fighters | 2 | 0–15; 0.5 | world units | current |
| `punish_down_reach` | Legacy punish: down-smash reach; hidden in Fighter Settings | Kirby 14; Yoshi 13; others 12 | 0–100; 0.5 | world units | ignored |
| `punish_forward_reach` | Legacy punish: forward-smash reach; hidden in Fighter Settings | Kirby 22; others 21 | 0–100; 0.5 | world units | ignored |
| `punish_height` | Legacy punish: height tolerance; hidden in Fighter Settings | 8 | 0–60; 0.5 | world units | ignored |
| `roll_smash_height` | Legacy roll: smash height; hidden in Fighter Settings | 6 | 0–60; 0.5 | world units | ignored |
| `roll_smash_watch` | Legacy roll: smash watch distance; hidden in Fighter Settings | 64 | 0–150; 0.5 | world units | ignored |
| `roll_grab_height` | Legacy roll: grab height; hidden in Fighter Settings | 4 | 0–60; 0.5 | world units | ignored |
| `roll_grab_watch` | Legacy roll: grab watch distance; hidden in Fighter Settings | Kirby 32; others 36 | 0–150; 0.5 | world units | ignored |
| `crouch_range` | Kirby crouch: proximity X; Kirby only | 45 | 0–150; 0.5 | world units | current |
| `crouch_height` | Kirby crouch: proximity Y; Kirby only | 20 | 0–100; 0.5 | world units | current |
| `shield_scan_x` | Shield: scan X cutoff; all fighters | 96 | 1–300; 1 | world units | current |
| `shield_scan_y` | Shield: scan Y cutoff; all fighters | 96 | 1–300; 1 | world units | current |
| `auto_grab_close_range` | Automatic grab: shielded range; all fighters | standing override/reference if standing and dash references are valid; otherwise 0 | 0–150; 0.5 | world units | current |
| `auto_grab_dash_range` | Automatic grab: approach range; all fighters | max(close, standing × dash reach / standing reference reach); otherwise 0 | 0–150; 0.5 | world units | current |
| `reflect_default` | Auto-reflect at startup; Game & Watch only | false | true / false; toggle | boolean | startup |
| `reflect_startup` | Reflect: startup frames; Game & Watch only | 6 | 6–30; 1 | frames | current |
| `reflect_margin` | Reflect: early margin; Game & Watch only | 2 | 0–8; 1 | frames | current |
| `reflect_max_hold` | Reflect: maximum hold frames; Game & Watch only | 45 | 6–120; 1 | frames | current |
| `reflect_half_width` | Reflect: body half-width; Game & Watch only | 5 | 1–30; 0.5 | world units | current |
| `reflect_height` | Reflect: body height; Game & Watch only | 16 | 1–60; 0.5 | world units | current |
| `reflect_half_depth` | Reflect: body half-depth; Game & Watch only | 5 | 1–30; 0.5 | world units | current |
| `tilt_turn_down` | Tilt face: down; Kirby/Yoshi/Lucina only | true for Kirby/Yoshi/Lucina; false otherwise | true / false; toggle | boolean | current |
| `tilt_turn_forward` | Tilt face: forward; Kirby/Yoshi/Lucina only | false | true / false; toggle | boolean | current |
| `tilt_turn_up` | Tilt face: up; Kirby/Yoshi/Lucina only | false | true / false; toggle | boolean | current |
| `counter_default` | Auto-counter at startup; Lucina only | false | true / false; toggle | boolean | startup |
| `counter_startup` | Counter: startup frames; Lucina only | 6 | 1–30; 1 | frames | current |
| `counter_margin` | Counter: early margin; Lucina only | 2 | 0–10; 1 | frames | current |
| `counter_half_width` | Counter: body half-width; Lucina only | 5 | 1–30; 0.5 | world units | current |
| `counter_height` | Counter: body height; Lucina only | 16 | 1–60; 0.5 | world units | current |
| `counter_half_depth` | Counter: body half-depth; Lucina only | 5 | 1–30; 0.5 | world units | current |
| `auto_spacing_minimum_distance` | Auto-spacing: minimum distance; hidden in Fighter Settings | 15 | 0–150; 0.5 | world units | current |
| `auto_spacing_ground_move` | Spacing ground move (0-5); all fighters | 0 | 0–5; 1 | selector | current |
| `auto_spacing_ground_startup` | Spacing ground: startup frames; all fighters | 8 | 1–300; 1 | frames | current |
| `auto_spacing_ground_active_frames` | Spacing ground: active frames; all fighters | 5 | 1–300; 1 | frames | current |
| `auto_spacing_ground_reach` | Spacing ground: reach estimate; all fighters | 20 | 0–150; 0.5 | world units | current |
| `auto_spacing_ground_height` | Spacing ground: height tolerance; all fighters | 9 | 0–100; 0.5 | world units | current |
| `auto_spacing_ground_commitment` | Spacing ground: commitment frames; all fighters | 33 | 1–300; 1 | frames | current |
| `auto_spacing_aerial_move` | Spacing aerial move (0-5); all fighters | 0 | 0–5; 1 | selector | current |
| `auto_spacing_aerial_startup` | Spacing aerial: startup frames; all fighters | 6 | 1–300; 1 | frames | current |
| `auto_spacing_aerial_active_frames` | Spacing aerial: active frames; all fighters | 3 | 1–300; 1 | frames | current |
| `auto_spacing_aerial_reach` | Spacing aerial: reach estimate; all fighters | 20 | 0–150; 0.5 | world units | current |
| `auto_spacing_aerial_height` | Spacing aerial: height tolerance; all fighters | 12 | 0–100; 0.5 | world units | current |
| `auto_spacing_aerial_commitment` | Spacing aerial: commitment frames; all fighters | 30 | 1–300; 1 | frames | current |
| `auto_di_default` | Auto DI (experimental); all fighters | false | true / false; toggle | boolean | startup |
| `auto_sdi_default` | Auto SDI (experimental); all fighters | false | true / false; toggle | boolean | startup |
| `auto_spacing_engagement_range` | Spacing: attack engagement range; hidden in Fighter Settings | 20 | 0–150; 0.5 | world units | current |
| `auto_spacing_ground_speed` | Spacing: ground walk fraction (0-1); all fighters | 0.49 | 0–1; 0.01 | stick fraction | ignored |
| `auto_spacing_maximum_drift` | Spacing: maximum approach drift; all fighters | 1 | 0–1; 0.01 | stick fraction | ignored |
| `auto_spacing_minimum_drift` | Spacing: minimum approach strength; all fighters | 0.1 | 0–1; 0.01 | stick fraction | ignored |
| `auto_spacing_ground_braking` | Spacing: Ground Braking; all fighters | true | true / false; toggle | boolean | ignored |
| `auto_spacing_air_braking` | Spacing: Air Braking; all fighters | true | true / false; toggle | boolean | ignored |
| `auto_spacing_braking_distance` | Spacing: braking activation distance; all fighters | 10 | 0–150; 0.5 | world units | ignored |
| `auto_defense_lead_frames` | Auto Defense lead (frames); all fighters | 4 | 0–30; 1 | frames | current |

## What each value changes

- **`walk_strength`:** Walking baseline uses the original curve: slightly lower at partial stick, 50% at full horizontal stick. Recipes keep their own strengths.
- **`grab_range`:** Your grab reach estimate for recipes, legacy grab and Yoshi chase helpers. Saved attacker overrides also feed ordinary-grab threat estimates. Never extends hitboxes.
- **`horizontal_range`:** Horizontal acquisition limit; requires acquire/follow limits ON.
- **`vertical_range`:** Vertical acquisition limit; requires acquire/follow limits ON.
- **`follow_range`:** Horizontal limit after acquiring a target; requires acquire/follow limits ON.
- **`follow_vertical_range`:** Vertical limit after acquiring a target; requires acquire/follow limits ON.
- **`stand_distance`:** Hold-L manual walk stop gap from the tracked ledge; also the automatic stance offset. Toggling autopilot ON alone does not limit walking.
- **`move_strength`:** Input strength for eligible automatic stance positioning, not a change to physics.
- **`manual_spacing`:** Enable manual walking limits while L is held. Autopilot ON without L does not cap manual walking.
- **`auto_position`:** Existing automatic stance path; requires manual spacing OFF and range limits ON. Recipes own their movements.
- **`use_range_limits`:** OFF preserves distance-agnostic tracking. ON enables the four acquisition/follow limits.
- **`grab_dodge_default`:** Saved startup ON/OFF for grab avoidance and its Auto react follow-up sequence.
- **`out_of_shield_default`:** Saved startup ON/OFF for automatic grabs and shield backup; configure their ranges/actions under Auto react.
- **`grab_dodge_lookahead`:** Frames ahead to forecast ordinary grab contact (1-8). Default 6; larger forecasts can dodge earlier, not guarantee success.
- **`shield_warning_x`:** Root-to-root horizontal warning distance for approaching verified attacks.
- **`shield_warning_y`:** Root-to-root vertical warning distance; not attack reach.
- **`shield_body_half_width`:** Estimated body radius around your root used against native hitbox geometry.
- **`shield_body_height`:** Estimated standing body height above your root, not a native hurtbox edit.
- **`shield_body_half_depth`:** Estimated body depth used by the shield threat test.
- **`shield_forecast`:** Shared approach, known-startup and active-hitbox prediction horizon.
- **`shield_hold`:** Extra defensive shield hold after a warning ends. Regrabs still clear the old tail.
- **`grab_dodge_padding`:** Your estimated horizontal body allowance added to the actual attacker grab reach.
- **`grab_dodge_height`:** Maximum root-height difference for predicted ordinary grab contact.
- **`tech_half_width`:** Horizontal body offsets for floor, wall and ceiling probes; does not alter native tech eligibility.
- **`tech_floor_origin`:** Floor ray origin above your root.
- **`tech_wall_low`:** Lowest wall probe height above your root.
- **`tech_wall_mid`:** Middle wall probe height above your root.
- **`tech_wall_high`:** Highest wall probe height above your root.
- **`tech_ceiling_origin`:** Upward ray origin above your root.
- **`tech_floor_padding`:** Extra downward distance on floor sweeps, including curved/refined sweeps.
- **`tech_ceiling_padding`:** Extra upward distance on ceiling sweeps.
- **`punish_down_reach`:** Root-based reach for legacy spot-punish and legacy roll down-smash; recipes retain their own ranges.
- **`punish_forward_reach`:** Root-based forward-smash reach for legacy automatic punish selection.
- **`punish_height`:** Vertical root gap used by the legacy spot-dodge punish.
- **`roll_smash_height`:** Height tolerance for the Kirby/Yoshi legacy roll smash reader.
- **`roll_smash_watch`:** Maximum root distance for watching legacy roll-smash opportunities.
- **`roll_grab_height`:** Height tolerance for the Kirby/Yoshi legacy roll grab reader.
- **`roll_grab_watch`:** Maximum root distance for watching legacy roll-grab opportunities.
- **`crouch_range`:** Maximum horizontal root gap for Kirby automatic crouch.
- **`crouch_height`:** Maximum vertical root gap for Kirby automatic crouch.
- **`shield_scan_x`:** Root-distance limit for estimated startup/recovery warnings; active geometry is checked independently.
- **`shield_scan_y`:** Root-height limit for estimated startup/recovery warnings; active geometry is checked independently.
- **`auto_grab_close_range`:** No-dash grab limit while you shield. A vulnerable grabbable opponent in front qualifies; enemy shield is not required.
- **`auto_grab_dash_range`:** Retained approach-range estimate; resolver keeps it at least the close range. Current automatic-grab admission is shield-only, so this cannot restore unshielded approach grabs.
- **`reflect_default`:** Explicit fighter-specific projectile defense. Unsupported fighters never inherit a reflector.
- **`reflect_startup`:** Frames before reflection/absorption begins; cannot reduce Game & Watch below his native six-frame startup.
- **`reflect_margin`:** Start this many frames before predicted contact plus native startup; no waiting action is inserted.
- **`reflect_max_hold`:** Maximum pre-delivery attempt budget. The current reflector releases input after mapper acknowledgement and hands back on native bucket startup; it does not hold Oil Panic for this whole value.
- **`reflect_half_width`:** Estimated body half-width for incoming projectile prediction; does not change native geometry.
- **`reflect_height`:** Estimated body height above the root for incoming projectile prediction.
- **`reflect_half_depth`:** Estimated depth allowance for incoming projectile prediction; off-plane attacks are ignored.
- **`tilt_turn_down`:** Prepare down tilt toward the opponent. Schema fallback is ON for Kirby, Yoshi and Lucina; supplied Lucina overrides it OFF.
- **`tilt_turn_forward`:** Aim physical forward C-stick tilts toward the opponent. Requires Tilt Attack C-stick mapping.
- **`tilt_turn_up`:** Aim physical up C-stick tilts toward the opponent. Requires Tilt Attack C-stick mapping.
- **`counter_default`:** Explicit fighter counter configuration; only verified down-special counters are supported.
- **`counter_startup`:** Frames before the counter becomes active; tune to the fighter's native move startup.
- **`counter_margin`:** Additional frames ahead of predicted melee contact to begin the counter.
- **`counter_half_width`:** Estimated horizontal body half-width for active melee hitbox prediction.
- **`counter_height`:** Estimated body height above root for active melee hitbox prediction.
- **`counter_half_depth`:** Estimated body half-depth for active melee hitbox prediction.
- **`auto_spacing_minimum_distance`:** Minimum root-to-root spacing while the assigned Auto-Spacing & Attack button is held. Universal input assistance, never a toggle.
- **`auto_spacing_ground_move`:** 0 disabled (default for every fighter); 1 jab; 2 forward tilt; 3 up tilt; 4 down tilt; 5 back (invalid, disabled). Ground tilts use directional tilt inputs, never move or status edits.
- **`auto_spacing_ground_startup`:** Estimated frames before the configured ground attack becomes active; does not change native startup.
- **`auto_spacing_ground_active_frames`:** Estimated active duration of the configured ground attack; never changes native hitboxes.
- **`auto_spacing_ground_reach`:** Tunable ground reach estimate, not an exact hitbox or hit guarantee.
- **`auto_spacing_ground_height`:** Tunable ground root-height tolerance, not an exact hitbox or hit guarantee.
- **`auto_spacing_ground_commitment`:** Estimated total ground attack commitment in frames; independent of startup and active duration.
- **`auto_spacing_aerial_move`:** 0 disabled (default for every fighter); 1 neutral air; 2 forward air; 3 up air; 4 down air; 5 back air. Already airborne only; never jumps or consumes manual jump.
- **`auto_spacing_aerial_startup`:** Estimated frames before the configured aerial becomes active; does not change native startup.
- **`auto_spacing_aerial_active_frames`:** Estimated active duration of the configured aerial; never changes native hitboxes.
- **`auto_spacing_aerial_reach`:** Tunable aerial reach estimate, not an exact hitbox or hit guarantee.
- **`auto_spacing_aerial_height`:** Tunable aerial root-height tolerance, not an exact hitbox or hit guarantee.
- **`auto_spacing_aerial_commitment`:** Estimated total aerial commitment in frames; independent of startup and active duration.
- **`auto_di_default`:** Experimental input-only DI with observed blast bounds; not terrain-aware or guaranteed survival. Default OFF.
- **`auto_sdi_default`:** Experimental input-only SDI with observed blast bounds and attacker direction. Default OFF.
- **`auto_spacing_engagement_range`:** Attack only inside this horizontal distance. Ground approach walks toward the target even outside attack range; aerial direction stays manual outside range. Independent of minimum retreat spacing; holding blocks other automatic modes.
- **`auto_spacing_ground_speed`:** Legacy ground-cap value retained for saved configurations. Live manual Auto Space uses the maximum native non-dash stick value; assisted retreat uses full strength and may dash.
- **`auto_spacing_maximum_drift`:** Legacy drift value retained for saved configurations. Auto Space now leaves manual aerial drift unrestricted and uses full drift for retreat.
- **`auto_spacing_minimum_drift`:** Legacy minimum-drift value retained for saved configurations; Auto Space no longer tapers movement.
- **`auto_spacing_ground_braking`:** Legacy ground-braking setting; ignored because Auto Space braking has been removed.
- **`auto_spacing_air_braking`:** Legacy air-braking setting; ignored because Auto Space braking has been removed.
- **`auto_spacing_braking_distance`:** Legacy braking-distance setting; ignored because Auto Space braking has been removed.
- **`auto_defense_lead_frames`:** Frames ahead of predicted contact to begin Auto Defense (0-30, default 4). Input-only timing, not native move startup.

## Worked changes and troubleshooting

**Track farther, do not change attack range:** set acquire X to 25, follow X to
50 and `use_range_limits=true`. You can acquire within 25 and retain within 50;
your grab still needs its own range check. Increasing scan cutoffs may inspect
more candidates; it does not turn a distant attack into a collision.

**Large defender:** increase body half-width or grab padding cautiously, keeping
height/depth realistic. These describe *your* warning envelope. The attacker's
native/reference reach remains separate; your `grab_range` describes your own
grab (and, when you are an observed opponent, your override informs warnings).
Larger envelopes can create false early warnings and waste inputs.

**Counter does nothing:** check Lucina identity, current enable/binding state,
explicit fighter configuration, special readiness, incoming melee evidence and
ownership first. Never lower a startup estimate to pretend the native counter
starts sooner. Supplied Lucina uses startup 7, margin 3, half-width 9, height 17,
depth 6; generic schema fallbacks are 6,2,5,16,5.

**Bucket does nothing:** Game & Watch requires explicit `reflect_default`, native
special readiness, eligible projectile evidence and gauge below 3. Supplied
values are startup 15, margin 8, hold 45, half-width 15, height 16, depth 15;
schema fallbacks are 6,2,45,5,16,5. Predictor horizon is startup + margin + 2,
limited to 40; the estimate never shortens native six-frame startup. Native
bucket confirmation ends input ownership rather than holding B for 45 frames.
See [projectile defense](PROJECTILE_REFLECT.md).

**No attack while Auto Space is held:** both selectors default 0 (disabled).
Choose a supported move and realistic startup/active/commitment/reach estimates;
ensure the mode has a binding and valid opponent/state. Ground move 5 is accepted
but intentionally disabled. Increasing commitment holds the policy's commitment
budget, not the move's animation. The six legacy speed/drift/braking fields
76–81 are ignored; changing them cannot restore removed braking/tapering.

## Expand the Rust schema safely

`Tuning([Option<f32>; COUNT])` stores absent/present overrides. `Values([f32; COUNT])`
is a resolved numeric snapshot. `None` means fallback; it is not zero. `usize`
is an array index; all parallel arrays use the same stable index.

For a new input-policy key, append (do not reorder) `KEYS`, `LABELS`, `HINTS`,
its index constant and fallback in `DEFAULTS`, increase `COUNT`, and add bounds.
Classify boolean/frame/menu visibility explicitly. Resolve it into the correct
policy through a typed accessor or `apply`; merely adding a slider does nothing.
Have the configuration/menu owners verify serialization and current/startup
routing. Add focused parser, default, bounds and policy tests in
`tests/unit/fighter_tuning_overrides.rs` and the consuming policy's existing
fixtures. Keep recipes, input simulation and native state separate. No executable
Rust changes or Switch builds are part of this documentation update.
