# Hold L3 for facing-direction turbo teabag

**Source change only: no NRO has been rebuilt for these controls.**

Hold **L3** to start creeping in Kirby's current facing direction while
alternating crouch and release. Creep input starts on the first eligible update,
including the crouch phase. Release L3 to disengage immediately and return fresh
physical input. Pressing again starts a new cycle. No side-D-pad direction is used.

L3 is exclusively turbo; it does not activate autopilot. Hold **Y** separately
for [ledge assist](LEDGE_TRAP.md). With only Y held, you retain manual capped
walk control. If both are held, eligible automatic punishes override turbo.
Releasing either button disengages only that button's tool.

[Side-D-pad slide attempts](SLIDE_CONTROLS.md) and the [D-pad Down trump](LEDGE_TRUMP.md)
take priority over turbo and ledge assist. Mash and tech retain priority. Native busy
states, damage, air movement and hitstop suspend turbo. The macro never forces a
status change or cancels an attack.

```text
l3_turbo = true
turbo_phase_frames = 2
turbo_walk_strength = 0.35
```

Phase length accepts 1–10 observed native updates per crouch/release phase.
Strength is normalized stick magnitude; the L3 sampler caps it at 0.45. A phase
uses horizontal creep in both halves, with vertical -1 for crouch and 0 for release.
Native movement rules determine the actual speed. Set `l3_turbo=false` to disable
teabag movement without disabling the ledge responses. The old `dpad_turbo` config
key is retained as an alias for the turbo enable setting; it does not restore a
side-D-pad binding. Use only one enable key to avoid order-dependent overrides.

Restart Smash after config changes. The plugin must be rebuilt later to use
these new bindings; existing NRO packages still contain the previous controls.
