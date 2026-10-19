# Hold Left/Right for directional turbo crouch

With `auto_ledge_trump = true`, D-pad Down is separately reserved for a
[one-shot ledge trump](LEDGE_TRUMP.md), which takes priority over turbo.

This control is included in both silent and diagnostic builds. With the supplied config:

- Hold D-pad Left: alternate crouch/release and move slightly left during the release phase.
- Hold D-pad Right: the same pattern moving right. These are absolute directions, independent of facing.
- Release the D-pad: restore your current physical left-stick input immediately.
- Hold both directions: suppress side taunts but cancel the macro's stick movement.

The normal side-taunt buttons are consumed during valid selected-player gameplay while this feature is enabled. Standalone D-pad Up/Down and right-stick inputs are unaffected by turbo. Diagonal directions involved in a macro press are consumed through release so they cannot cause another taunt. The [ledge assist](LEDGE_TRAP.md) separately reserves the left-stick click (L3) when enabled; L3 takes priority over turbo movement. The macro temporarily owns the left stick only while it is actually running. In menus or when no fresh valid fighter observation is available, physical controls pass through.

The current fighter scope remains the configured human-controlled Kirby using a Pro Controller. The macro runs only while grounded in idle, walking, running, turning, or crouch states. Airborne movement, attacks, grabs/damage, hitstop, mash, and predicted tech situations suspend the stick macro. Re-entry starts from the crouch phase. No status changes are forced.

## Settings

Edit `SD:/ultimate/kirby_assist/config.txt`, then restart Smash:

```text
dpad_turbo = true
turbo_phase_frames = 1
turbo_walk_strength = 0.25
```

`turbo_phase_frames` is the number of observed selected-controller mapper updates per phase, from 1 through 10. At 1 the input alternates down / horizontal every update. If animation barely changes or the rhythm is unreliable, try 2 or 3. These are input-update counts, not a verified global simulation clock; visible crouch speed still depends on the game/character.

`turbo_walk_strength` is the normalized horizontal stick magnitude in the move phase, from 0.05 through 0.6. The supplied 0.25 is intended as a gentle movement input; hardware testing is needed to tune the resulting travel. Crouch uses x=0, y=-1; move uses x=-strength or +strength, y=0.

`dpad_turbo = false` restores ordinary side-D-pad behavior after restart. Missing config keys use the Rust defaults, which leave this new feature disabled. Replace the config along with the plugin when installing these packages to enable it.

## How to extend it

`manual_controls.rs` contains the pure hold-trigger/phase logic and its tests. `TurboState::sample` reads the original physical D-pad mask before any taunt suppression. It resets on release, direction conflict/change, invalid context, or clock rewind, and repeated calls for one input counter reuse the same phase.

`ManualOverlay` consumes held/previous/pressed/released side-D-pad bits and updates the owned left-stick digital direction history. It composes after the mash/tech overlay, which owns different button bits. The hook also suppresses affected autorepeat keys so held taunt repeat cannot leak through the remapping.

In `src/live.rs`, the verified Controller prefix now exposes left-stick floats at offsets 0x10/0x14. The mapper resolves the selected Kirby on its first call, saves physical axes and repeat flags, supplies the manual overlay to the original mapper, filters reserved taunts in the mapped result, and restores all saved physical values afterwards. D-pad reservation is independent of the short action-request timeout. Turbo supplies down on its first eligible input update with no added startup timer; game animations can still delay a visible crouch. Ledge takeover separately neutralizes the right stick while active. Compile-time layout assertions protect against accidental changes to this declared prefix; executable compatibility still requires device verification.

To add another manual control, define its trigger and owned fields separately, decide its priority/cancellation rules, and test release/hand-back before connecting native writes. Do not read our prior generated output as a new physical trigger.

## Device checks

Host tests and compiler checks cover timing, left/right, conflicts, release, raw/mapped taunt suppression, diagonals, state gating, and physical-stick restoration. The current source fixes have not been tested on your Switch or packaged into new release ZIPs.

Check each direction while stationary, reverse direction, release while physically moving the stick, hold both directions, and try pause/resume, jump, damage, grab, respawn, disconnect, and another player's isolation. Confirm side taunts do not fire while the feature owns them, normal menu navigation remains usable, and right-stick/buttons still work. Use exactly one logging variant at a time.
