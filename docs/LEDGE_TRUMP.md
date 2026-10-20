# D-pad Down: ledge trump attempt

Install build 1.0.1 or later and tap D-pad Down while grounded and actionable. No
opponent is required. The feature defaults on, including with older configs
that omit `auto_ledge_trump`. Set it to false to restore Down taunt.
This feature uses the existing human Kirby/player-input scope.

On activation only, native floor-support rays search both directions up to 100
world units for the nearest boundary of Kirby's current floor. The nearer edge
wins regardless of facing; equal distances prefer the direction Kirby faces.
There is no opponent or Y-assist acquisition-range requirement. An undetected edge
means no movement, but Down taunt is still consumed. Start on the main flat
stage floor: this finds collision edges, not proof of a grabbable native cliff.
Platforms, slopes, moving terrain and irregular stages need device testing.

The input sequence is full outward stick until native fall, one native
animation step of Down, then full inward stick. It deliberately permits running
off the selected edge. The Down step uses native animation progress rather than
counting repeated mapper polls. This is an input-only attempt: no position,
velocity, status, invincibility or ledge-occupancy writes are made.

Tapping starts the whole sequence; holding Down cannot repeat it. At native
ledge catch/catch-move, ownership ends immediately. Choose the follow-up
yourself. Catching an occupied ledge is not separately confirmed as a successful
trump: the opponent can escape or otherwise prevent the displacement.

The attempt cancels on damage or another unexpected action, running more than
three units past the selected edge while still grounded, a drop more than 16
units below/above the anchor, separation over 12 units after the initial fall,
or 120 changing non-stopped own-state observations. Approach distance is bounded
by the initial distance plus four units.
These observations are not a verified simulation-frame timer. Plus/Minus uses
the existing cancellation path. A failed catch hands back physical control;
there is no automated recovery. Tap again after release for another attempt.

Holding Y activates the ledge assist; L3 only controls turbo teabag. Trump takes priority over Y assist and L3
and side-D-pad turbo while active; mash and tech retain priority. Down taunt is
reserved while the feature is enabled. Disable the setting to restore it.

Diagnostic builds identify the root Cargo package version at startup. `trump input:` logs enablement,
gameplay scope, request freshness, phase and stick output when Down is pressed.
`trump mapped:` logs final mapped buttons and stick, after taunt suppression.
If Down still taunts, check the loaded NRO/version, SD config, selected fighter
(Kirby), selected player input slot and `player_index`. A host build cannot verify the SD install.

The run-off/Down/inward approach follows the manual technique demonstrated in
[How To Ledge Trump Like A Pro](https://www.youtube.com/watch?v=V6B0g3a0KpU).
[Hit Box's ledge-trump notes](https://www.hitboxarcade.com/blogs/smash-box/different-walks-sb-ssbu)
explain that occupied-ledge collision can make a running approach inconsistent.
No universal fastest timing or successful Switch trump has been measured here.
Test both ledges on Final Destination first, starting close to the edge; check
both catch success and immediate manual follow-up. Then vary distance and
opponent. A stalled run-off may need a dash-to-walk refinement after device data.

Policy: `crates/kirby_assist_core/src/ledge_trump.rs`. The live adapter composes
its output through the existing shared stick/button overlay.
