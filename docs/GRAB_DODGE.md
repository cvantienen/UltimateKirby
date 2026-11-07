# Grab dodge and configured punish

**Modes → Grab dodge + punish** is available for every fighter. The supplied
startup config enables it; a fresh match reloads the startup default. It is
independent of Spot punish, Out of shield and Roll rules.

The existing eight-slot fighter roster is checked for ordinary standing, dash
and pivot grab statuses. Detailed reads are limited to those candidates. Their
observed motion, facing, position and velocity plus reference grab timing/reach
drive a relative-motion forecast. Only standing still and shielding are eligible,
and native dodge/shield-entry readiness must allow the input immediately.
Walking/running and other actions are excluded even if a transition term is set.
Current physical movement/attack input is checked too, so the final native idle
frame cannot turn a new run into an automatic spot dodge. Held-shield tilt remains
eligible; full roll input and releasing shield to move stay under manual control.
Down + Shield with neutral horizontal stick requests the dodge; this mode does
not brake movement or reserve controls while waiting for recovery.

The default forecast is six native frames. Continuous linear interval intersection
checks whether grab activity, horizontal reach and vertical proximity overlap at
the same future instant; it cannot step over a contact between half-frame samples. Training
**Fighter settings → Grab dodge forecast frames** saves
`grab_dodge_lookahead = 6` for that fighter; allowed values are 1–8. The maximum
is two fixed axis-interval checks per candidate, with no stage rays or new hooks.
A larger forecast can dodge earlier, but can also dodge too early.
**Grab dodge padding** is your fighter's horizontal body allowance added to the
attacker's reach; **Grab dodge height** controls root-height tolerance. These
remain saved per fighter, including Bowser. A larger body may need more padding;
this is an estimate, not a read of actual hurtboxes or a hitbox edit.

## Current automatic Up-B follow-up

The live mode has its own **Auto react → After dodging enemy grab** entry:

```ini
[bowser.roll.auto_grab_dodged]
start_delay = 0
timeout = 240
reaction = auto_grab_dodged:25:10
actions = facing_input:b:0:1:4, drift:opponent:1:69
```

This was copied exactly from Bowser's prior shared spot-dodge response. Enemy
spot-dodge reactions and your grab-dodge follow-up can now have independent
ranges and actions. **Grab avoidance** still controls this response independently
of the Auto react toggle. Its L3/R3 shortcut opens this entry; the main Auto react
list shows it alongside rolls, dodges and automatic grabs. Edited actions use
the shared sequence player; native readiness, confirmation and ownership remain
mandatory. Generic actions wait for native action recovery, and direct Up-B
retains its specific native Up-B readiness gate.

Only observed native spot-dodge startup from a pending assisted request arms the
follow-up. This confirms dodge startup, not successful grab avoidance. Recovery
owns no input: the mode checks native Up-B readiness and sends the punish in that
same observation, including a legal native dodge cancel. It never forces recovery
to end. The first actionable observation must still have the original grabber
within the saved X/Y range; a miss is discarded rather than queued for later.
Airborne starts, target loss/capture, damage and higher-priority safety ownership
cancel it. Before a punish commits, manual takeover and mode OFF also cancel it.
Committed automatic Bowser Up-B ignores physical buttons and both sticks through
its tracking tail; ledge-trap L still cancels immediately. Mode OFF prevents the
next punish. Once native Up-B is confirmed, its drift can follow Bowser's air spin. A new grab warning cannot interrupt that committed punish.
Native grab/catch/hold/pummel/throw states release the entire plan so throwing
remains manual. If clocks freeze before native Up-B starts, 24 distinct delivered
mapper attempts bound its input; duplicate/wrong-target receipts do not count.
Native confirmation disables that watchdog, preserving the full tracking tail.
Leaving the floor before native startup is confirmed cancels B/Up; a prior
grounded request alone never authorizes a fresh airborne special.

Older `grab_dodged` / `grab_dodged_vulnerable` recipe lists remain readable for
legacy tools. The current live mode uses `auto_grab_dodged`. Without that fighter’s own `auto_grab_dodged` entry, the mode only dodges; it
never borrows an enemy spot-dodge or another fighter’s punish. One delivered
Down+Guard packet is followed by neutral input while native ESCAPE is observed.
Unmapped attempts and missing confirmation both have bounded mapper budgets.

## Defaults and limits

Training **Fighter settings → Grab dodge + punish at startup** saves
`grab_dodge_default = true` or `false` in `[fighter.settings]`. The global fallback
is `auto_spot_dodge` in `[universal.settings]` of `sequences.txt`; the supplied config now sets it true. Saving
a startup default does not change the current match's mode switch.

Held shields may have zero animation rate. Only guard-on/held-shield states
accept that stationary clock; the observed grab animation supplies retry timing
without counting repeated policy polls. Delivery holds one continuous Down +
Shield request; it does not repeatedly flick Down and queue another dodge. The
live adapter observes your native dodge state and releases both inputs at startup.
Hitstop does not advance recipes. The attempt ends after 24 native-progress
frames or 24 distinct delivered mapper attempts; duplicate acknowledgments do
not spend that budget. A temporary
uncancelable state or changed range forecast releases the request without marking
the grab handled. It may be reacquired while still threatening and native readiness
allows it. Native spot-dodge startup consumes the grab once; a delivery timeout
also stops that attempt. Consumed grab identity survives missing samples but
clears when an already-read native status confirms that grab ended, preventing
an old marker from suppressing a later grab. This avoids discarding an unaccepted dodge merely because
shieldstun or a movement prediction briefly interrupted delivery.

Root reach/linear forecasts are approximations, not exact collision prediction.
Command grabs, unknown grab data and situations where native dodge cannot start
remain unsupported. This cannot guarantee avoiding every grab or landing every
punish, especially under online input delay. All actions are simulated controller
input; no physics, statuses, move definitions or hitboxes are changed.

Ordinary-grab follow-up recovery uses the actual opponent's standing/dash/pivot
active window rather than a standing-grab duration shared by all variants. DK's
reference is 8–10 standing, 11–13 dash, 12–14 pivot, with a 19.5-unit standing
reach estimate. Saved reach overrides still apply to the attacker they belong to.
Native readiness and higher-priority ledge responses can still prevent a dodge;
reference forecasts cannot guarantee avoiding every grab.

Incoming-grab defense is sampled before new offense. Existing committed
roll/OOS/ledge/safety owners keep their protection. After native dodge startup,
the defense-only owner releases; the tracked Up-B uses ordinary committed
punish ownership when ready. Active grab frames are inclusive: fractional
progress inside the last listed active frame remains a threat until it ends.

Active ledge mode (held binding or toggle) disables incoming-grab searches and
this follow-up. Busy own states do not start a new dodge or punish; the confirmed
own dodge can still follow its original grabber until native recovery permits
the configured response. Physical Grab also suspends automation before native
catch is visible. Captured actors are excluded using cached capture-link reads.

After Bowser's follow-up actually maps Special+Up, a short passive receipt can
survive hitstop/readiness changes until native Up-B appears. It holds no buttons
or sticks, sends no extra B, and retains punish priority for at most 24 mapper
updates. The configured drift keeps its full duration after that confirmation.
This does not arm a new punish during dodge/shield recovery or alter the single
spot-dodge input. L and the existing safety/target-loss rules clear the receipt.
