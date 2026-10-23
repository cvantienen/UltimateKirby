# Stick-click and Y shortcuts

Hold **R3 alone** for fast crouch/walk-cancel turbo teabag, with slight creep
in your current facing direction. It alternates every observed native update;
older `turbo_phase_frames` values do not slow this dedicated binding.
Release R3 to stop. **L+R3** keeps its menu-toggle function;
the click must be released after that chord before it can start teabagging.
Busy, airborne and stopped states suspend teabagging.

Press **X** on the ground for a paired **X+ZL** short-hop attempt. Both
physical buttons must be mapped to Jump. The request survives brief readiness
changes and explicitly releases both buttons through native jump squat. Holding
X cannot generate a second jump on takeoff; release and press X again for an
ordinary midair jump. This replaces the former L3 short-hop binding.

Hold **L3 as Kirby** for [down-tilt recovery cover](TWO_FRAME.md). Releasing L3
immediately stops that mode. It has no new L3 assist binding on other fighters.

Tap **Y** for one neutral-special B-reverse attempt: one observed neutral-stick
update, B with neutral stick, then a backward flick on the next observed update,
then physical control returns. It uses B mapped to Special and captures the
opposite of your fighter's facing when Y is pressed. Stick direction does not
choose or change the reverse. The tool supplies neutral B and the opposite
flick automatically; you do not need to move the stick yourself. It does not start 2-frame mode. The game determines
whether that fighter's current special supports reversal; no status, facing or
velocity is forced. Skipped observations, stale data and interruptions cancel
rather than executing a delayed flick. Input delay/jitter can still affect it.

Tech/mash keep priority. The hop and B-reverse shortcuts also yield to slides,
ledge trump and the trip follow-up. Menu directions keep their existing behavior.
The shortcuts reuse the native motion observation and shared controller overlay;
there is no new target scan, geometry scan, networking or runtime file loading.

```text
r3_turbo = true
turbo_phase_frames = 1
```

The legacy oscillator phase setting accepts 1–10 observed native updates;
R3 always overrides it to 1. `turbo_walk_strength` controls the gentle forward
creep, capped to 0.45. The old `l3_turbo` key remains an enable alias for R3,
so old SD-card configuration still loads. Use only one enable key.

The menu filter preserves Y internally for native special-readiness reads while
the final input overlay still suppresses its normal mapped action. This allows
the first Y press to start the shortcut; it does not require an already active
shortcut to open its own readiness queries.

Source changes require a later build and deployment. No build/upload or Rust
test execution was performed for this revision; controller acceptance and
online timing still need device testing.
