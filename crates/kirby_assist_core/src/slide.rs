//! D-pad short-hop -> earliest-airborne downward-angled dodge attempt. Inputs only.
//! Tap for one attempt; hold to repeat once native grounded jump becomes ready.
use crate::ledge_trap::{LedgeOutput, LEDGE_BUTTONS};
use crate::manual_controls::{DPAD_LEFT, DPAD_RIGHT, DPAD_SIDES};
use crate::native_input::{MAPPER_X, MAPPER_ZR};

// Requested raw controller values; on-device slide distance is not established.
pub const SLIDE_STICK_X: f32 = 0.9588;
pub const SLIDE_STICK_Y: f32 = -0.2839;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlideContext {
    pub gameplay: bool,
    pub stopped: bool,
    pub status: i32,
    pub situation: i32,
    pub motion_frame: f32,
    pub can_jump: bool,
    pub can_air_dodge: bool,
    pub safe_ground: [bool; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Phase {
    #[default]
    Idle,
    Queued,
    Takeoff,
    Dodge,
    Landing,
}

#[derive(Default)]
pub struct SlideState {
    phase: Phase,
    direction: i8,
    previous_direction: i8,
    started: i64,
    last_sample: Option<(i64, u32, SlideContext)>,
    last_output: LedgeOutput,
}
impl SlideState {
    pub fn sample(
        &mut self,
        enabled: bool,
        tick: i64,
        physical: u32,
        ctx: SlideContext,
    ) -> LedgeOutput {
        if !enabled || !ctx.gameplay || !ctx.motion_frame.is_finite() {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_sample.is_some_and(|last| tick < last.0) {
            *self = Self::default();
        }
        let key = (tick, physical, ctx);
        if self.last_sample == Some(key) {
            return self.last_output;
        }
        let direction = match physical & DPAD_SIDES {
            DPAD_LEFT => -1,
            DPAD_RIGHT => 1,
            _ => 0,
        };
        let pressed = direction != 0 && direction != self.previous_direction;
        self.previous_direction = direction;
        // Damage/captures/ledge states cancel. A grounded attack/recovery can
        // queue a tap, but receives no override until native jump readiness.
        let supported_status =
            matches!(ctx.status, 0..=0x46) || ctx.situation == 0 && ctx.status >= 0x1dc;
        let can_queue = ctx.situation == 0 || matches!(ctx.status, 0x22..=0x23);

        if pressed && can_queue && supported_status && physical & DPAD_SIDES != DPAD_SIDES {
            self.phase = Phase::Queued;
            self.direction = direction;
            self.started = tick;
        }
        let mut out = LedgeOutput::default();
        if physical & DPAD_SIDES == DPAD_SIDES || !supported_status {
            self.phase = Phase::Idle;
        } else if !ctx.stopped {
            if self.phase == Phase::Queued && self.jump_ready(ctx, self.direction) {
                self.phase = Phase::Takeoff;
                self.started = tick;
                out = owned();
                out.requested = MAPPER_X;
            } else {
                match self.phase {
                    Phase::Idle => {}
                    Phase::Queued => {
                        // A released tap is retained for up to 60 advancing native
                        // observations. Waiting never owns movement or buttons.
                        if direction != 0 && direction == self.direction {
                            self.started = tick; // Held requests survive long native recovery.
                        } else if tick.saturating_sub(self.started) > 60 {
                            self.phase = Phase::Idle;
                        }
                    }
                    Phase::Takeoff => {
                        if tick.saturating_sub(self.started) > 8 {
                            self.phase = Phase::Idle;
                        } else if ctx.situation == 2 {
                            // Native jump frame, not mapper poll count or a
                            // fabricated height-to-frame conversion. A missed
                            // early window aborts rather than dodging high up.
                            if ctx.status == 0xb && (0.0..=1.0).contains(&ctx.motion_frame) {
                                out = owned();
                                if ctx.can_air_dodge {
                                    out.stick = Some([
                                        f32::from(self.direction) * SLIDE_STICK_X,
                                        SLIDE_STICK_Y,
                                    ]);
                                    out.requested = MAPPER_ZR;
                                    self.phase = Phase::Dodge;
                                    self.started = tick;
                                }
                            } else {
                                self.phase = Phase::Idle;
                            }
                        } else {
                            out = owned(); // Release X during jump squat.
                        }
                    }
                    Phase::Dodge => {
                        if matches!(ctx.status, 0x22..=0x23)
                            || ctx.situation == 0 && ctx.status != 0xa
                        {
                            self.phase = Phase::Landing;
                            out = owned(); // Release ZR, retain slide momentum.
                        } else if tick.saturating_sub(self.started) > 8 {
                            self.phase = Phase::Idle;
                        } else {
                            out = owned();
                            // Release ZR but preserve the angle until native
                            // acceptance; a buffered dodge must not see neutral.
                            out.stick =
                                Some([f32::from(self.direction) * SLIDE_STICK_X, SLIDE_STICK_Y]);
                        }
                    }
                    Phase::Landing => {
                        out = owned();
                        if tick.saturating_sub(self.started) > 120 {
                            self.phase = Phase::Idle;
                            out = LedgeOutput::default();
                        }
                    }
                }
            }
        }
        if self.phase == Phase::Landing && !ctx.stopped && ctx.situation == 0 && ctx.can_jump {
            self.phase = Phase::Idle;
            out = LedgeOutput::default();
            if self.jump_ready(ctx, direction) {
                self.direction = direction;
                self.phase = Phase::Takeoff;
                self.started = tick;
                out = owned();
                out.requested = MAPPER_X;
            }
        }
        self.last_sample = Some(key);
        self.last_output = out;
        out
    }

    fn jump_ready(&self, ctx: SlideContext, direction: i8) -> bool {
        direction != 0
            && ctx.situation == 0
            && ctx.can_jump
            && ctx.safe_ground[usize::from(direction > 0)]
    }
}
fn owned() -> LedgeOutput {
    LedgeOutput {
        stick: Some([0.0, 0.0]),
        owned: LEDGE_BUTTONS,
        ..LedgeOutput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ground() -> SlideContext {
        SlideContext {
            gameplay: true,
            stopped: false,
            status: 0,
            situation: 0,
            motion_frame: 0.0,
            can_jump: true,
            can_air_dodge: false,
            safe_ground: [true; 2],
        }
    }
    fn air(frame: f32) -> SlideContext {
        SlideContext {
            status: 0xb,
            situation: 2,
            motion_frame: frame,
            can_jump: false,
            can_air_dodge: true,
            ..ground()
        }
    }
    #[test]
    fn tap_either_direction_short_hops_dodges_and_returns_control_after_landing() {
        for (button, direction) in [(DPAD_LEFT, -1.0), (DPAD_RIGHT, 1.0)] {
            let mut state = SlideState::default();
            let jump = state.sample(true, 0, button, ground());
            assert_eq!(jump.requested, MAPPER_X);
            assert_eq!(state.sample(true, 0, button, ground()), jump);
            let squat = SlideContext {
                status: 0xa,
                motion_frame: 1.0,
                can_jump: false,
                ..ground()
            };
            assert_eq!(state.sample(true, 1, 0, squat).requested, 0);
            let dodge = state.sample(true, 2, 0, air(1.0));
            assert_eq!(dodge.requested, MAPPER_ZR);
            assert_eq!(
                dodge.stick,
                Some([direction * SLIDE_STICK_X, SLIDE_STICK_Y])
            );
            assert_eq!(state.sample(true, 2, 0, air(1.0)), dodge);
            let escape = SlideContext {
                status: 0x23,
                motion_frame: 2.0,
                ..air(1.0)
            };
            assert_eq!(state.sample(true, 3, 0, escape).requested, 0);
            let lag = SlideContext {
                status: 0x19,
                can_jump: false,
                ..ground()
            };
            assert_eq!(state.sample(true, 4, 0, lag).requested, 0);
            assert_eq!(state.sample(true, 5, 0, ground()), LedgeOutput::default());
        }
    }
    #[test]
    fn held_direction_repeats_only_when_landing_recovery_allows_jump() {
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_RIGHT, ground());
        state.sample(true, 1, DPAD_RIGHT, air(1.0));
        state.sample(
            true,
            2,
            DPAD_RIGHT,
            SlideContext {
                status: 0x23,
                ..air(2.0)
            },
        );
        for tick in 3..8 {
            assert_eq!(
                state
                    .sample(
                        true,
                        tick,
                        DPAD_RIGHT,
                        SlideContext {
                            status: 0x19,
                            can_jump: false,
                            motion_frame: tick as f32,
                            ..ground()
                        }
                    )
                    .requested,
                0
            );
        }
        assert_eq!(
            state.sample(true, 8, DPAD_RIGHT, ground()).requested,
            MAPPER_X
        );
    }
    #[test]
    fn missed_or_unavailable_first_air_frame_never_becomes_a_late_dodge() {
        for ctx in [
            air(2.0),
            SlideContext {
                can_air_dodge: false,
                ..air(1.0)
            },
        ] {
            let mut state = SlideState::default();
            state.sample(true, 0, DPAD_RIGHT, ground());
            assert_eq!(state.sample(true, 1, DPAD_RIGHT, ctx).requested, 0);
            assert_eq!(state.sample(true, 2, DPAD_RIGHT, air(3.0)).requested, 0);
        }
    }
    #[test]
    fn unsafe_busy_conflicting_or_disabled_starts_do_not_jump() {
        for ctx in [
            SlideContext {
                safe_ground: [false; 2],
                ..ground()
            },
            SlideContext {
                can_jump: false,
                ..ground()
            },
            SlideContext {
                status: 0x47,
                ..ground()
            },
            air(1.0),
        ] {
            assert_eq!(
                SlideState::default().sample(true, 0, DPAD_RIGHT, ctx),
                LedgeOutput::default()
            );
        }
        assert_eq!(
            SlideState::default().sample(true, 0, DPAD_SIDES, ground()),
            LedgeOutput::default()
        );
        assert_eq!(
            SlideState::default().sample(false, 0, DPAD_RIGHT, ground()),
            LedgeOutput::default()
        );
    }
    #[test]
    fn damage_scope_loss_and_clock_rewind_cancel_but_stop_freezes_without_dodging() {
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_RIGHT, ground());
        assert_eq!(
            state.sample(
                true,
                1,
                DPAD_RIGHT,
                SlideContext {
                    stopped: true,
                    ..air(1.0)
                }
            ),
            LedgeOutput::default()
        );
        assert_eq!(
            state.sample(true, 2, DPAD_RIGHT, air(1.0)).requested,
            MAPPER_ZR
        );
        assert_eq!(
            state.sample(
                true,
                3,
                DPAD_RIGHT,
                SlideContext {
                    status: 0x47,
                    ..air(2.0)
                }
            ),
            LedgeOutput::default()
        );
        assert_eq!(
            state.sample(true, 4, DPAD_RIGHT, air(1.0)),
            LedgeOutput::default()
        );
        assert_eq!(
            state.sample(
                true,
                5,
                0,
                SlideContext {
                    gameplay: false,
                    ..ground()
                }
            ),
            LedgeOutput::default()
        );
        assert_eq!(
            state.sample(true, 6, DPAD_RIGHT, ground()).requested,
            MAPPER_X
        );
        assert_eq!(state.sample(true, 0, 0, ground()), LedgeOutput::default());
    }
    #[test]
    fn directional_air_dodge_landing_can_loop_on_the_first_jump_ready_observation() {
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_LEFT, ground());
        state.sample(true, 1, DPAD_LEFT, air(1.0));
        // Skip intermediate dodge/landing samples, as can happen across hooks.
        assert_eq!(
            state.sample(true, 2, DPAD_LEFT, ground()).requested,
            MAPPER_X
        );
    }

    #[test]
    fn slide_recipe_has_real_jump_and_shield_edges_with_the_shared_overlay() {
        use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
        use crate::native_input::ButtonTrace;
        let mut state = SlideState::default();
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace::default();
        let mut apply = |out: LedgeOutput| {
            overlay.apply_controls(
                human,
                [-1.0, 0.0],
                TurboOutput {
                    consume_dpad: false,
                    stick: out.stick,
                },
                DPAD_SIDES,
                out.requested,
                out.owned,
            )
        };
        let jump = apply(state.sample(true, 0, DPAD_RIGHT, ground()));
        assert_eq!(jump.buttons.pressed & MAPPER_X, MAPPER_X);
        let release = apply(state.sample(
            true,
            1,
            DPAD_RIGHT,
            SlideContext {
                status: 0xa,
                can_jump: false,
                motion_frame: 1.0,
                ..ground()
            },
        ));
        assert_eq!(release.buttons.released & MAPPER_X, MAPPER_X);
        let dodge = apply(state.sample(true, 2, DPAD_RIGHT, air(1.0)));
        assert_eq!(dodge.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
        assert_eq!(dodge.stick, [SLIDE_STICK_X, SLIDE_STICK_Y]);
        assert_eq!(dodge.buttons.held & LEFT_STICK_DIRECTIONS, 1 << 21);
        let release = apply(state.sample(
            true,
            3,
            0,
            SlideContext {
                status: 0x23,
                ..air(2.0)
            },
        ));
        assert_eq!(release.buttons.released & MAPPER_ZR, MAPPER_ZR);
    }
    #[test]
    fn downward_angle_is_preserved_until_native_dodge_acceptance_with_zr_released() {
        for (button, direction) in [(DPAD_RIGHT, 1.0), (DPAD_LEFT, -1.0)] {
            let mut state = SlideState::default();
            state.sample(true, 0, button, ground());
            let request = state.sample(true, 1, button, air(1.0));
            let angle = [direction * 0.9588, -0.2839];
            assert_eq!(request.stick, Some(angle));
            assert_eq!(request.requested, MAPPER_ZR);
            let awaiting = state.sample(true, 2, button, air(2.0));
            assert_eq!(awaiting.stick, Some(angle));
            assert_eq!(awaiting.requested, 0);
            let accepted = state.sample(
                true,
                3,
                button,
                SlideContext {
                    status: 0x23,
                    ..air(1.0)
                },
            );
            assert_eq!(accepted.stick, Some([0.0, 0.0]));
            assert_eq!(accepted.requested, 0);
        }
    }
    #[test]
    fn moving_and_cancel_ready_grounded_actions_start_without_waiting_for_idle() {
        for status in [1, 2, 3, 4, 5, 6, 7, 8, 9, 0x23, 0x29, 0x2a] {
            let ctx = SlideContext { status, ..ground() };
            assert_eq!(
                SlideState::default()
                    .sample(true, 0, DPAD_RIGHT, ctx)
                    .requested,
                MAPPER_X
            );
        }
    }

    #[test]
    fn a_tapped_slide_queues_through_recovery_without_owning_inputs_then_starts_when_legal() {
        let mut state = SlideState::default();
        let busy = SlideContext {
            status: 0x29,
            can_jump: false,
            ..ground()
        };
        assert_eq!(
            state.sample(true, 0, DPAD_RIGHT, busy),
            LedgeOutput::default()
        );
        assert_eq!(state.sample(true, 1, 0, busy), LedgeOutput::default());
        let ready = SlideContext {
            can_jump: true,
            motion_frame: 20.0,
            ..busy
        };
        assert_eq!(state.sample(true, 2, 0, ready).requested, MAPPER_X);
        assert_eq!(state.sample(true, 3, 0, air(1.0)).requested, MAPPER_ZR);
    }

    #[test]
    fn queued_slide_uses_the_correct_side_floor_probe_and_remembers_a_hitstop_tap() {
        let mut state = SlideState::default();
        let stopped = SlideContext {
            stopped: true,
            ..ground()
        };
        assert_eq!(
            state.sample(true, 0, DPAD_RIGHT, stopped),
            LedgeOutput::default()
        );
        assert_eq!(state.sample(true, 1, 0, ground()).requested, MAPPER_X);
        for (button, safe) in [(DPAD_LEFT, [true, false]), (DPAD_RIGHT, [false, true])] {
            assert_eq!(
                SlideState::default()
                    .sample(
                        true,
                        0,
                        button,
                        SlideContext {
                            safe_ground: safe,
                            ..ground()
                        }
                    )
                    .requested,
                MAPPER_X
            );
        }
    }

    #[test]
    fn queued_slide_expires_or_cancels_on_damage_and_disable() {
        let busy = SlideContext {
            status: 0x29,
            can_jump: false,
            ..ground()
        };
        for cancel in [
            SlideContext {
                status: 0x47,
                ..busy
            },
            SlideContext {
                gameplay: false,
                ..busy
            },
        ] {
            let mut state = SlideState::default();
            state.sample(true, 0, DPAD_RIGHT, busy);
            state.sample(true, 1, 0, cancel);
            assert_eq!(state.sample(true, 2, 0, ground()), LedgeOutput::default());
        }
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_RIGHT, busy);
        state.sample(true, 61, 0, busy);
        assert_eq!(state.sample(true, 62, 0, ground()), LedgeOutput::default());
    }
    #[test]
    fn tap_during_air_dodge_slide_is_retained_until_grounded_jump_readiness() {
        let mut state = SlideState::default();
        let sliding = SlideContext {
            status: 0x23,
            situation: 2,
            can_jump: false,
            ..ground()
        };
        assert_eq!(
            state.sample(true, 0, DPAD_LEFT, sliding),
            LedgeOutput::default()
        );
        assert_eq!(state.sample(true, 1, 0, sliding), LedgeOutput::default());
        let grounded = SlideContext {
            situation: 0,
            can_jump: true,
            ..sliding
        };
        assert_eq!(state.sample(true, 2, 0, grounded).requested, MAPPER_X);
    }

    #[test]
    fn holding_slide_through_long_recovery_starts_at_first_legal_window() {
        let mut state = SlideState::default();
        let busy = SlideContext {
            status: 0x29,
            can_jump: false,
            ..ground()
        };
        state.sample(true, 0, DPAD_RIGHT, busy);
        for tick in 1..90 {
            assert_eq!(
                state.sample(true, tick, DPAD_RIGHT, busy),
                LedgeOutput::default()
            );
        }
        assert_eq!(
            state.sample(true, 90, DPAD_RIGHT, ground()).requested,
            MAPPER_X
        );
    }
}
