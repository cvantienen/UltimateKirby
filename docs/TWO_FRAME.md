# Kirby held-L3 down-tilt recovery cover

**Hold L3 as Kirby** to watch an opponent's low recovery approach. **Release L3**
to immediately stop the policy and release pending inputs. Y remains B-reverse;
L remains ledge autopilot. When both L and L3 are held, L3 selects recovery cover.
L3 has no new assist binding on other fighters. Menu navigation and higher-priority
mash/tech retain their existing behavior.

Stand near the actual ledge and face outward. The policy can turn toward the
ledge before preparing the attack; it does not run to the edge. It declines
positions beyond the configured tilt range, capped at 10 stage units. Geometry
still determines whether Kirby's down-tilt hitbox reaches a particular recovery.

The existing bounded recovery reader supplies position and velocity. The policy
forecasts entry into a small low ledge-approach band, prepares a held down-tilt
stick position (Y=-0.6) one observed update before A, and schedules using Kirby's
**frame-4 down tilt**. Live extra input-delay compensation is zero. The first active frame
is aimed into `[predicted arrival, predicted arrival + 2)`. Native down-tilt
acceptance releases A; a missed forecast is abandoned rather than sent late.

This is a **position/velocity forecast**, not an exact native ledge-snap timer.
It cannot know future recovery steering, acceleration, teleports, capture boxes,
or hurtbox placement. Some approaches do not expose a punishable two-frame
window. Exact contact is not guaranteed online or offline. Test the requested
input, actual attack startup, position and contact on Switch.

The legacy core down-smash recipe remains available to core callers when
`two_frame_down_tilt=false`; the live Kirby L3 binding selects down tilt.
No new roster discovery, game-state writes, animation edits or networking is
added by this binding. Recovery queries run only while the mode is active.

No build/upload or Rust test execution was performed for this source revision.
