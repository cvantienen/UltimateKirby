# Analog-stick auto mash

Live automatic mash uses a full-strength left-stick cycle. It runs for the selected human fighter while the native clatter timer is positive and the game is not stopped. It generates no automated action-button presses for mash.

| Mashable input update | Left stick |
| --- | --- |
| 0 | Right: `(1.0, 0.0)` |
| 1 | Up: `(0.0, 1.0)` |
| 2 | Left: `(-1.0, 0.0)` |
| 3 | Down: `(0.0, -1.0)` |
| 4 | Right again |

Each distinct update advances one direction, without neutral gaps or an additional interval. Repeated observations of the same update reuse the result. A new mashable episode, fighter replacement, or reversed input clock restarts at Right. Hitstop publishes no automated mash stick and pauses progression. Progress is deduplicated from native observations rather than raw mapper-poll
counts; it is not a verified global simulation-frame clock.

## Settings and control

Use the bundled settings:

```text
auto_mash = true
takeover = true
```

`takeover = true` suppresses physical A/B/X/Y/L/R/ZL/ZR, stick clicks, and C-stick input only during active analog mashing. This keeps physical action inputs from being mixed with the left-stick cycle. Plus retains its cancellation path; Auto-mash is changed from its menu row. With takeover false, physical action buttons/C-stick pass through while the automated left stick still runs.

The mapper uses the current physical controls as soon as the mashable state ends; it does not force a neutral frame after escape. Disabling the tool, stale requests, menu cancellation, disconnect, and controller reassignment also remove its stick override. The native controller memory is restored after every mapper call. Turbo and ledge stick requests cannot override an active mash cycle. Tech retains its existing priority, ZR button, and cooldown.

The old `alternate_mash`, `mash_buttons_per_frame`, and `mash_interval_frames` settings are ignored for SD-config compatibility. There is no button-mash fallback in the live plugin. The generic `AutoMash` preset remains a button-pulse teaching example; live translation emits the stick instead.

## Code walkthrough

`RawAutoControls` in `crates/kirby_assist_core/src/live_input.rs` advances the clatter counter and caches `mash_stick: Option<[f32; 2]>`. `Some([x, y])` proposes a normalized stick position; `None` means no override. Only tech emits HID button requests.

`src/live.rs` publishes the stick alongside the existing request timestamp, so it expires with stale observations. The mapper gives it priority over D-pad stick controls and uses `ManualOverlay` to produce consistent analog values and left-stick direction-bit history. Takeover clears competing physical action/C-stick inputs only while the mash proposal is active. Physical floats, button words, and repeat metadata are saved/restored around the original mapper.

Host tests cover the direction cycle, duplicate-update caching, exit/reentry, disable/reassignment/clock reset, physical button/stick restoration, stick-only overlay without D-pad ownership, legacy configs, and existing tech release/cooldown behavior. Ledge/turbo regressions remain in the suite.

## Device comparison

This is implemented and compiler-checked, but faster escape time has not been measured on a Switch. Controlled [Ultimate input tests](https://hyperts.net/smashsp/rebagacha/) found that simultaneous button presses do not stack and that left-stick movement is effective. That motivates this implementation; it does not establish its optimal cadence on your device/version.

Rebuild and install the new plugin before testing; existing ZIPs were not rebuilt. Compare a repeatable grab or bury at the same damage, characters, and positions across builds. Measure gameplay frames from the start of the mashable state to escape, rather than counting generated inputs. Repeat the test, then check neutral movement, physical input restoration, held D-pad triggers, and tech behavior immediately after escape. The mod leaves native clatter timers and escape rules to Smash.
