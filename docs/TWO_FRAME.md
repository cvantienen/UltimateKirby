# Kirby recovery cover inside ledge autopilot

**Hold L as Kirby** for the normal saved ledge responses and in-range low-recovery
coverage with down tilt. **Release L** to stop. There is no separate recovery
toggle, menu row or standalone R3 binding. Other fighters retain their existing
ledge options until explicitly configured. Diagnostic Training compatibility uses R3 only for its menu; normal L
autopilot includes this Kirby coverage.

Normal get-up responses keep their settings and priority. Recovery cover starts
only while no ledge punish, roll navigation, capture handback or other higher
priority tool owns the action. Once its tilt primer/request is committed, the
shared input path protects that short delivery from shield/preparation movement
and onstage Roll read. It never combines a recovery tilt with another punish.

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
`two_frame_down_tilt=false`; `two_frame::autopilot_settings` explicitly registers
Kirby with down tilt. It does not register another fighter by fallback.
No new roster discovery, game-state writes, animation edits or networking is
added by this binding. Recovery queries use the existing cached roster while
Kirby holds L near a locked ledge; positions outside 1–10 units inside that ledge
skip the recovery scan. Standard ledge observations remain unchanged for others.

Host scheduling regressions cover tilt timing, range, facing, release, duplicate
polls and missed forecasts. Actual recovery contact still needs Switch testing.
