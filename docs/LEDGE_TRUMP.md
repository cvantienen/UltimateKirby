# Assignable ledge trump mode

Available in both builds, initially OFF. Enable **Trump** on the Assist modes
ON/OFF page. There is no default D-pad Down trigger and no generic Hold/Toggle
mode assignment on Button configs; old such binding entries are ignored.

Trump movement pauses while a menu is visible. Native grounded readiness and
the selected human fighter/controller scope are required. It uses existing
read-only floor geometry and simulated movement inputs; it does not change
positions, velocities or ledge physics.

On activation, native floor-support rays search both directions up to 100
world units for the nearest boundary of the controlled fighter's current floor.
Failed searches retry at most once per 30 mapper ticks while held; releasing
the trigger or changing modes clears that search state. The nearer edge
wins regardless of facing; equal distances prefer the controlled fighter’s facing.
There is no opponent acquisition-range requirement. An undetected edge
means no movement, and D-pad Down is still reserved. Start on the main flat
stage floor: this finds collision edges, not proof of a grabbable native cliff.
Platforms, slopes, moving terrain and irregular stages need device testing.

The input sequence is full outward stick until native fall, one native
animation step of Down, then full inward stick. It deliberately permits running
off the selected edge. The Down step uses native animation progress rather than
counting repeated mapper polls. This is an input-only attempt: no position,
velocity, status, invincibility or ledge-occupancy writes are made.

Tapping starts the whole sequence; holding D-pad Down cannot repeat it. At native
ledge catch/catch-move, ownership ends immediately. Choose the follow-up
yourself. Catching an occupied ledge is not separately confirmed as a successful
trump: the opponent can escape or otherwise prevent the displacement.

The attempt cancels on damage or another unexpected action, running more than
three units past the selected edge while still grounded, a drop more than 16
units below/above the anchor, separation over 12 units after the initial fall,
or 120 changing non-stopped own-state observations. Approach distance is bounded
by the initial distance plus four units.
These observations are not a verified simulation-frame timer. Plus uses
the existing cancellation path. A failed catch hands back physical control;
there is no automated recovery. Tap again after release for another attempt.

Holding L activates ledge autopilot, including Kirby recovery cover. Minus toggles the menu. Trump takes priority over ledge autopilot, shortcuts and
side-D-pad slides while active; mash and tech retain priority. D-pad Down is
reserved while the feature is enabled. Disable the setting to restore it.

Diagnostic builds identify the root Cargo package version at startup. `trump input:` logs enablement,
gameplay scope, request freshness, phase and stick output when D-pad Down is pressed.
`trump mapped:` logs final mapped buttons and stick, after taunt suppression.
If the enabled Trump mode does not start, check the loaded NRO/version, SD config, selected human fighter, selected player input slot and `player_index`. A host build cannot verify the SD install.

The run-off/Down/inward approach follows the manual technique demonstrated in
[How To Ledge Trump Like A Pro](https://www.youtube.com/watch?v=V6B0g3a0KpU).
[Hit Box's ledge-trump notes](https://www.hitboxarcade.com/blogs/smash-box/different-walks-sb-ssbu)
explain that occupied-ledge collision can make a running approach inconsistent.
No universal fastest timing or successful Switch trump has been measured here.
Test both ledges on Final Destination first, starting close to the edge; check
both catch success and immediate manual follow-up. Then vary distance and
opponent. A stalled run-off may need a dash-to-walk refinement after device data.

The approach now changes from full run input to 0.55 walking input within eight
units of the selected edge, suppressing dash aliases there. On the first native
airborne observation it sends inward+Down, then releases Down after native
progress and keeps inward input. This reduces outward carry during the former
Down-only phase. It cannot make an ungrabbable platform edge catchable, erase
native ledge lockouts, or guarantee a trump; verify both sides on Switch.

Policy: `crates/kirby_assist_core/src/ledge_trump.rs`. The live adapter composes
its output through the existing shared stick/button overlay.
