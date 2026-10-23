# Luigi and Captain Falcon ledge profiles

Luigi (native kind 9) and Captain Falcon (11) retain their compiled ledge options
and timings. Their combo builders have been removed, along with the other
fighter combos. See [the ledge menu](LEDGE_MENU.md) for held-L autopilot and the other current bindings.

## Ledge timing choices

Move data was checked against original [Luigi frame measurements](https://ultimateframedata.com/luigi)
and [Captain Falcon frame measurements](https://ultimateframedata.com/captain_falcon).
Native kind/status/transition IDs were checked against the pinned skyline-smash
lua_const.rs bindings. Recipe delays are starting values, not experimentally
verified universal optimal timings.

- Luigi: nair 3–31; down tilt 5–8; forward tilt 5–7; first down-smash hit 6–7.
  Shield grab starts at 18, so the faster 6-frame shield-to-nair is the default
  get-up-attack response. Shield grab remains selectable and hands control back
  immediately on capture. Normal-getup nair defaults to lead 8.
- Falcon: first nair hit 7–8; down tilt 11–12; forward tilt 7–10; first down-smash
  hit 19–20. His second nair hit (13–15) and rear down-smash hit (29–30) have gaps;
  they are not counted as continuous coverage. Forward tilt is the normal-getup
  default. Selecting nair clamps early lead to at most 1 frame for its first hit.
- Both have alternative neutral-getup tilts, shield-to-nair/shield-grab choices,
  shield or one down-tilt-then-shield on ledge drop, without Off entries.
  Roll uses the existing fixed common destination and native vulnerability
  schedule. Late recognition/arrival can miss, especially Falcon's slower smash.
  These are practical initial presets, not a proof of optimal match-up coverage.

## Runtime

Only the selected fighter's response executes. Settings and UI labels are
cached on changes; no fighter files load at runtime. Idle movement/ledge reads
are skipped. These changes have not been built or measured on Switch.
