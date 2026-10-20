# Auto tech input scheduling

`auto_tech = true` applies to the selected human controller in every supported
battle mode. ZR must map to Shield. The implementation supplies controller
inputs; it does not force tech status or override native lockout/untechable rules.

The predictor now reads **ALL kinetic reserves**, including damage knockback.
The previous MAIN-only query could miss the velocity driving a stage collision.
Swept floor, wall and ceiling body probes estimate the first contact time by
bounded ray refinement. `tech_lookahead_frames` remains a base horizon of 1–5;
learned input delay extends the actual search, capped at 20 frames.

For an eligible damage-air/tumble state, the live scheduler compares contact
time with learned input arrival delay. Late damage hitstop can buffer a request
before movement resumes; its remaining stop time is included in the forecast.
Early/global stop, invalid prediction and the native passive-disabled flag do
not generate a tech request. A valid damage-hitstop timer can supply timing even
when the motion clock is frozen or not yet available.

## Ganondorf Side B

Grounded Flame Choke uses victim status `CATCHED_GANON` (`0xbe`), which the
ordinary airborne-tumble detector excluded. The adapter now resolves the actual
capture-linked fighter without depending on Arena player slots or the ledge
roster. It verifies Ganondorf's grounded catch state, then predicts the throw
from the linked captor's native motion frame/rate. The vanilla
[grounded throw script](https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/main/smashline/lua2cpp_ganon/ganon/SpecialS.txt)
applies its throw at motion frame 31. That is a move-specific forecast, not an
assumption that the victim has normal downward knockback.

The linked clock also drives scheduler progress when the victim's capture pose
is frozen. The shield request targets four frames before predicted impact after
learned input delay; ordinary tumble retains its existing nine-frame lead.
Held shields still require release and a fresh mapped press. Invalid/missing
capture links or clocks do not fabricate a throw deadline. This timing assumes
the vanilla move script and still requires on-device verification.

The aerial capture/fall/slam states (`0xbf`–`0xc1`) remain excluded: aerial Flame
Choke has an [untechable knockdown](https://ultimateframedata.com/ganondorf).
Shield inputs cannot convert it into a legal tech.

## Input ownership and confirmation

With `takeover = true`, an already held mapped shield is released before a fresh
ZR press. The release clears all four shoulder-button aliases, so holding ZL/L/R
cannot keep mapped GUARD held while ZR is being released. Other physical buttons
and normal stick direction remain controlled by the player unless another tool
owns them. Additive mode preserves held shields and cannot force a release.

A proposal no longer consumes cooldown. The scheduler waits for the original
mapper to produce a fresh GUARD edge before committing the configured cooldown.
Unmapped requests remain pending only while the current collision is still
predicted. Trajectory/readiness changes cancel them. Fresh human shield presses
in tumble also update this cooldown tracking.

Cooldown advances from rate-normalized native motion progress, excludes hitstop,
and is conservative across animation changes or missing observations. An observed
native tech status ends the attempt and clears the artificial cooldown for the
next collision. While the same collision remains imminent during cooldown,
takeover keeps all physical shield aliases released instead of handing them back
between samples. This prevents a held ZL/ZR/L/R from fabricating another fresh
GUARD edge and disturbing native input timing. None of this changes the game's own tech timers.

The live adapter uses `tech_input.rs` for this acknowledgment-based scheduling;
`RawAutoControls` supplies analog mash independently. Existing pure-Rust legacy
tech examples/tests do not define the live timing path.

With diagnostics and `tech_debug = true`, compare:

- `collision_predicted`: forecast contact, stop time, delay and proposal.
- `tech ZR applied`: fresh raw ZR supplied to the mapper.
- `tech request timing`: current status, motion clock, contact estimate and input
  delay at that press; grounded Flame Choke uses the linked captor's clock.
- `ganon_ground_capture_timing_unavailable`: grounded capture was seen, but a
  verified linked captor/usable pre-impact clock was unavailable.
- `ganon_aerial_slam_untechable`: aerial capture/slam is deliberately excluded.
- `tech mapped: fresh GUARD edge confirmed`: mapped shield input acknowledgment.
- `tech_status_observed`: actual native tech state; a proposal alone is not success.

Collision rays still approximate body geometry and future movement. SDI, changing
knockback, passable/moving/sloped surfaces and online delay jitter can invalidate
a forecast. Untechable impacts and native lockout remain untechable. Host tests
and compiler checks do not establish a zero-miss result on a Switch.

Grounded Flame Choke and shield-handback regressions have been added as source
cases. A host test cannot validate the native capture link or prove a zero-miss
result online. Source edits do not update an installed NRO; a separately
requested build/install and diagnostic console test are still needed.
