# Project backlog

### Character-specific changes needed

- [ ] Fix and device-test Kirby's forward-facing attack-cancel bair route; it can
  still turn into forward tilt instead of bair.
- [ ] Continue fighter-by-fighter roll-grab calibration for approach distance,
  facing, grab range and the vulnerable-frame target.
- [ ] Finish on-device calibration of Yoshi's Neutral short-hop nair and bair
  choices, including hit timing, fast fall, drift and handback.
- [ ] Validate each fighter's saved ledge choices on hardware and record any
  timing, spacing or move-delivery adjustments on that fighter's page.
- [ ] Add or tune fighter-specific response choices as testing exposes gaps.

### System-level changes

- [ ] Investigate the doubles connection error.
- [ ] Confirm Minus-menu and assist-list Training testing with stock Training Modpack v7.1.1 in
  Training Mode on the diagnostic Switch.
- [ ] Measure silent and diagnostic callback cost on the Switch and keep logging,
  result-card rendering and observations within the frame-time budget.
- [ ] Compare offline and Arena timing with controlled diagnostic captures; keep
  network delay findings separate from local input-delivery bugs.
- [ ] Revisit live combo playback later; it is currently removed.

## Task notes

Record expected behavior and validation for each task. Keep portable/host checks
separate from actual Switch testing. Gameplay timing, save-crash resolution,
Training-to-online GPU stability and heat still require on-device confirmation;
no documentation or passing host test establishes those outcomes.
