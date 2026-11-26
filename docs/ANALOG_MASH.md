# Analog-stick auto mash

A **clatter** state is a native mashable capture/bury state with a positive
escape timer. Live auto mash requests a full-strength **left-stick cycle**, not
automatic action-button presses and not direct timer edits.

| Distinct eligible update | Stick |
| --- | --- |
| 0 | Right `(1, 0)` |
| 1 | Up `(0, 1)` |
| 2 | Left `(-1, 0)` |
| 3 | Down `(0, -1)` |
| 4 | Repeat Right |

## Activation and settings

Auto mash is independent of ledge holds. Enable its menu row or startup setting;
selected human controller, global enabled and valid session/native observation
still matter. Rust fallback and supplied universal `auto_mash` are **false**.

```ini
[universal.settings]
auto_mash = true
takeover = true
```

Merge these keys into the existing section; do not erase other settings. Rust
fallback takeover is false; the supplied universal file uses true. With takeover
true, competing physical action buttons, stick clicks and C-stick are suppressed
only during active mashing. Without it they pass through, while mash still owns
the left-stick override. Refer to [Controls](CONTROLS.md),
[Fighter settings](FIGHTER_SETTINGS.md) and [Sequence files](SEQUENCE_FILES.md)
for mode admission, mapping and actual startup/saved precedence.

## Clock and handback

Each distinct native-derived update advances one direction, without neutral gaps
or another interval. Duplicate observations reuse output; mapper polls are not a
verified global simulation-frame clock. Hitstop sends no new mash stick and
pauses progression. A new mashable episode, fighter replacement or reversed
input clock restarts Right.

Capture mash has first safety priority, above DI/SDI and ordinary ledge/turbo
stick requests. Losing clatter, disabling the tool, stale requests, controller
reassignment/disconnect, menus and session cancellation remove its override.
Physical controls return immediately on escape rather than receiving a forced
extra neutral frame. The mapper restores original physical controller storage
after its call; generated input is an overlay, not permanent controller mutation.

## Legacy and extension

`alternate_mash`, `mash_buttons_per_frame` and `mash_interval_frames` remain old
compatibility settings, not live cadence controls. `policies/tools.rs::AutoMash`
is a generic button-pulse preset whose minimum interval leaves a release gap.
The live adapter translates clatter to analog mash instead: changing that preset's
button interval cannot speed up the live stick cycle.

For Rust beginners, `RawAutoControls` stores `Option<[f32; 2]>`: `Some` proposes
normalized stick axes, `None` means no override. Output must still pass freshness,
safety arbitration and mapping. Add tests for duplicate/stop/episode reset and
physical handback when changing cadence; do not change clatter timers or native
escape rules. A player wanting a different button tape should use
[Sequence files](SEQUENCE_FILES.md), not assume that tape becomes native mash.

For a controlled Training comparison, repeat the same grab/bury, damage and
positions, and measure gameplay time to escape rather than generated-input count.
Optimal cadence and actual faster escape are not established by this document.
