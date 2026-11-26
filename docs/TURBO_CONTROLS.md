# Turbo t-bag and configurable buttons

**Turbo t-bag** is a current held built-in button role, not a global D-pad macro.
It alternates Down/crouch and a small facing-direction creep input. Holding its
assigned button admits it; releasing stops it. Grounded movement readiness and
higher-priority owners can suppress it, so a physical hold is not a guaranteed
crouch on every game frame. In menus, the same button retains menu semantics
rather than secretly running gameplay automation.

## Configure without confusing a role and a tape

Use [Controls](CONTROLS.md) for the current bindings/layout and Buttons editor.
Fresh role defaults are **L3 Auto Defense**, **D-pad Down Turbo t-bag**, and
**Short hop** on other ordinary slots; an empty supported-fighter R3 list can
fall back to Counter. Saved explicit-default/first-entry selection overrides
fresh defaults. The legacy `normal` token is the built-in labelled **Short hop**,
not a separate plain-input passthrough entry.

A **role** starts an implemented helper; a **saved tape** plays literal inputs.
Both use the shared overlay but differ in admission/cancellation. Moving a
built-in to another slot requires changing the old slot too if you do not want
two triggers. To author plain input, use an explicit literal recipe rather than
assuming the historical Normal button label bypasses assistance.

```ini
[universal.settings]
turbo_walk_strength = 0.35
```

This is the currently supplied universal creep value; Rust fallback is0.45.
It is normalized stick strength, not guaranteed world speed. Merge only the key
into your actual settings section; saving/reloading is documented in
[Sequence files](SEQUENCE_FILES.md). Changing it does not change native crouch
animation rate or introduce a physics speed multiplier.

Legacy `r3_turbo`, `l3_turbo`, `dpad_turbo` and `turbo_phase_frames` remain old
compatibility fields, not selectors/cadence controls for this role. D-pad slots
are ordinary configurable bindings. Ledge trump is a separate one-shot role;
Counter/Spacing/Auto defense have distinct hold/toggle behavior. Do not infer
role assignment from the button name or old hard-coded D-pad descriptions.

## Timing and handback

Each distinct accepted local update advances phase; repeated polls do not
establish a simulation-frame count. Turbo stays below safety, committed recipes,
defense and posture-sensitive control paths. Release, menu/handoff, invalid
controller scope, unavailable ground movement and global/mode suspension remove
its output. The mapper restores raw controller storage after composing the
winning request. A current native crouch does not prove the next generated input
will be accepted during recovery.

## Beginner extension and testing

Players assign the role and tune creep; they need not modify Rust. A literal
saved crouch tape can have authored timing, but does not become the native turbo
helper or inherit its checks. Rust extensions should preserve physical release,
duplicate/stop accounting, native readiness and higher-owner handback. Test with
no enemy first, then while an automatic response interrupts it and after opening
Training/menu handoff. A rapid input cycle is not a guaranteed fastest t-bag or
proof of on-device timing. See [Input recipes](INPUT_RECIPES.md) for literal versus
native-confirmed behavior and [Ledge trump](LEDGE_TRUMP.md) for one-shot semantics.
