# Button-mapped ledge trump

Assign **Built-in: Ledge trump** to a button in Button configs. No button is
assigned by default, so D-pad Down has no special behavior. The assigned button
can be tapped: the input-only sequence continues after release. Release and
press again after changing the assignment. A held button cannot start the
sequence again after it finishes.

The routine requires native grounded readiness and the selected human
fighter/controller scope. It uses existing read-only floor geometry and
simulated controller input; it does not change positions, velocities or ledge
physics.

On activation, native floor-support rays search both directions up to 100
world units for the nearest boundary of the controlled fighter's current floor.
A failed search does not repeat while the button remains held; release and
press again to retry.
The nearer edge wins regardless of facing; equal distances
prefer the controlled fighter's facing. There is no opponent acquisition-range
requirement. An undetected edge means no movement. Start on the main flat stage
floor: this finds collision edges, not proof of a grabbable native cliff.
Platforms, slopes, moving terrain and irregular stages need device testing.

The input sequence is outward stick until native fall, one native animation
step of Down, then inward stick. It deliberately permits running off the
selected edge. The Down step uses native animation progress rather than
counting repeated mapper polls. This is an input-only attempt: no position,
velocity, status, invincibility or ledge-occupancy writes are made.

At native ledge catch/catch-move, ownership ends immediately. Choose the
follow-up yourself. Catching an occupied ledge is not separately confirmed as
a successful trump: the opponent can escape or otherwise prevent displacement.

The attempt cancels on damage or another unexpected action, running more than
three units past the selected edge while still grounded, a drop more than 16
units from the anchor, separation over 12 units after the initial fall, or 120
changing non-stopped own-state observations. Approach distance is bounded by
the initial distance plus four units. These observations are not a verified
simulation-frame timer. Plus uses the existing cancellation path. A failed
catch hands back physical control; press the assigned button again for another
attempt.

Holding L activates ledge autopilot, including Kirby recovery cover. Minus
toggles the menu. An active ledge-trump attempt takes priority over ledge
autopilot and side-D-pad slides; mash and tech retain priority. Only the
assigned button is reserved for its binding; D-pad Down is not reserved by
default.

Diagnostic builds identify the root Cargo package version at startup. The
`trump input:` message reports scope, request freshness, phase and stick output
when the assigned button is pressed. The `trump mapped:` message reports final
mapped buttons and stick after taunt suppression. If the assigned action does
not start, check the loaded NRO/version, selected human fighter, selected
player input slot and `player_index`. A host build cannot verify the SD install.

The run-off/inward approach follows the manual technique demonstrated in
[How To Ledge Trump Like A Pro](https://www.youtube.com/watch?v=V6B0g3a0KpU).
[Hit Box's ledge-trump notes](https://www.hitboxarcade.com/blogs/smash-box/different-walks-sb-ssbu)
explain that occupied-ledge collision can make a running approach inconsistent.
No universal fastest timing or successful Switch trump has been measured here.
Test both ledges on Final Destination first, starting close to the edge; check
both catch success and immediate manual follow-up. Then vary distance and
opponent.

The approach changes from full run input to 0.55 walking input within eight
units of the selected edge, suppressing dash aliases there. On the first native
airborne observation it sends inward+Down, then releases Down after native
progress and keeps inward input. This reduces outward carry during the former
Down-only phase. It cannot make an ungrabbable platform edge catchable, erase
native ledge lockouts, or guarantee a trump; verify both sides on Switch.

Policy: `crates/kirby_assist_core/src/policies/ledge_trump.rs`. The live adapter composes
its output through the existing shared stick/button overlay.
