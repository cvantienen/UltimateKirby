//! Kirby tilt-trip recognition and buffered forward-smash inputs.
use super::FOLLOWUP_BUTTONS;
use crate::gameplay::ControllerOutput;
use crate::native_input::MAPPER_A;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripTarget {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
}
fn trip_tilt(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::ATTACK_S3 | crate::native_states::status::ATTACK_LW3
    )
}
fn tripping(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::SLIP..=crate::native_states::status::SLIP_WAIT
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripContext {
    pub object: u32,
    pub kirby: bool,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub grounded: bool,
    pub facing: f32,
    pub stopped: bool,
    pub targets: [Option<TripTarget>; 8],
}
#[derive(Default)]
pub struct TripFollowup {
    last: Option<TripContext>,
    output: ControllerOutput,
    queued: bool,
    spent: bool,
    facing: f32,
}
impl TripFollowup {
    pub fn cancel_pending(&mut self) {
        self.queued = false;
        self.spent = true;
        self.output = ControllerOutput::default();
    }
    pub fn watching(&self) -> bool {
        self.queued
    }
    pub fn sample(&mut self, enabled: bool, ctx: TripContext) -> ControllerOutput {
        // Invalid/changed identity resets the tool. An identical observation
        // reuses output, so repeated mapper polls do not manufacture new pulses.
        if !enabled || !ctx.kirby || !ctx.frame.is_finite() || ctx.frame < 0.0 {
            *self = Self::default();
            return ControllerOutput::default();
        }
        if self.last.is_some_and(|last| last.object != ctx.object) {
            *self = Self::default();
        }
        if self.last == Some(ctx) {
            return self.output;
        }
        let native_progress = !ctx.stopped
            && self.last.is_none_or(|last| {
                ctx.status != last.status || ctx.motion != last.motion || ctx.frame != last.frame
            });
        let new_tilt = trip_tilt(ctx.status)
            && self.last.is_none_or(|last| {
                last.status != ctx.status || ctx.frame < last.frame || ctx.motion != last.motion
            });
        if new_tilt && !self.queued {
            self.spent = false;
        }
        // Recognize current trips and grounded re-hit damage whose immediate
        // previous state was a trip. No new-trip transition or hit flag required.
        let recognized = !self.queued
            && !self.spent
            && trip_tilt(ctx.status)
            && ctx.targets.iter().flatten().any(|enemy| {
                tripping(enemy.status)
                    || tripping(enemy.previous_status)
                        && matches!(
                            enemy.status,
                            crate::native_states::status::DAMAGE
                                | crate::native_states::status::DOWN_DAMAGE
                        )
            });
        if recognized {
            self.queued = true;
            self.facing = if ctx.facing.is_finite() && ctx.facing < 0.0 {
                -1.0
            } else {
                1.0
            };
        }
        let mut out = ControllerOutput::default();
        let was_queued = self.queued;
        if self.queued {
            let accepted = matches!(
                ctx.status,
                crate::native_states::status::ATTACK_S4_START
                    ..=crate::native_states::status::ATTACK_S4
            );
            let supported = ctx.grounded
                && matches!(ctx.status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV | crate::native_states::status::ATTACK_S3 | crate::native_states::status::ATTACK_LW3);
            if accepted || !supported {
                self.queued = false;
                self.spent = true;
            } else if !recognized && !native_progress {
                // Changed target facts are evidence, not another input frame.
                // Preserve the current press/release through duplicate polls.
                self.last = Some(ctx);
                return self.output;
            } else {
                // Own every competing gameplay button and both sticks from
                // recognition onward, including tilt lag and release gaps.
                out.owned = FOLLOWUP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                // First release the physical tilt/A. Then keep sending fresh
                // directional+A flicks into the game's normal buffer until smash
                // startup is observed. No readiness gate or six-pulse cutoff.
                if !recognized && !ctx.stopped && self.output.requested & MAPPER_A == 0 {
                    out.requested = MAPPER_A;
                    out.stick = Some(crate::move_input::smash_stick(
                        crate::move_input::Smash::Forward,
                        self.facing,
                    ));
                }
            }
        }
        // Release immediately at smash startup so the macro does not charge it.
        if out.owned == 0 && was_queued {
            out.owned = FOLLOWUP_BUTTONS;
            out.stick = Some([0.0, 0.0]);
        }
        self.last = Some(ctx);
        self.output = out;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(status: i32, target_status: i32) -> TripContext {
        TripContext {
            object: 1,
            kirby: true,
            status,
            motion: 1,
            frame: 1.0,
            grounded: true,
            facing: 1.0,
            stopped: false,
            targets: [
                Some(TripTarget {
                    id: 2,
                    status: target_status,
                    previous_status: crate::native_states::status::WAIT,
                }),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }
    }
    #[test]
    fn trip_immediately_owns_controls_and_buffers_smash_without_readiness_or_target_rechecks() {
        let mut state = TripFollowup::default();
        let trip = TripContext {
            stopped: true,
            ..ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            )
        };
        let prepare = state.sample(true, trip);
        assert_eq!(prepare.requested, 0);
        assert_eq!(prepare.owned, FOLLOWUP_BUTTONS);
        assert_eq!(prepare.stick, Some([0.0, 0.0]));
        assert!(state.watching());
        assert_eq!(state.sample(true, trip), prepare);
        // A latched trip is not cancelled by missing targets or their recovery.
        // There is no can_smash input: requests start while tilt lag remains.
        for frame in 2..=31 {
            let out = state.sample(
                true,
                TripContext {
                    frame: frame as f32,
                    stopped: false,
                    targets: [None; 8],
                    ..trip
                },
            );
            assert_eq!(out.owned, FOLLOWUP_BUTTONS);
            assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
            assert_eq!(
                out.stick,
                Some(if frame % 2 == 0 {
                    [1.0, 0.0]
                } else {
                    [0.0, 0.0]
                })
            );
        }
        // A transition to idle still receives a forward-smash request at once.
        let ready = TripContext {
            status: crate::native_states::status::WAIT,
            frame: 0.0,
            targets: [None; 8],
            ..trip
        };
        let ready = TripContext {
            stopped: false,
            ..ready
        };
        assert_eq!(state.sample(true, ready).requested, MAPPER_A);
        let accepted = TripContext {
            status: crate::native_states::status::ATTACK_S4_START,
            frame: 1.0,
            ..ready
        };
        let released = state.sample(true, accepted);
        assert_eq!(released.requested, 0);
        assert_eq!(released.stick, Some([0.0, 0.0]));
        assert!(!state.watching());
        assert_eq!(
            state
                .sample(
                    true,
                    TripContext {
                        frame: 2.0,
                        ..accepted
                    }
                )
                .owned,
            0
        );
    }
    #[test]
    fn either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms() {
        for status in [
            crate::native_states::status::ATTACK_S3,
            crate::native_states::status::ATTACK_LW3,
        ] {
            for facing in [-1.0, 1.0] {
                for trip_status in [
                    crate::native_states::status::SLIP,
                    crate::native_states::status::SLIP_DAMAGE,
                    crate::native_states::status::SLIP_WAIT,
                ] {
                    let mut state = TripFollowup::default();
                    let trip = TripContext {
                        facing,
                        ..ctx(status, trip_status)
                    };
                    assert_eq!(state.sample(true, trip).requested, 0);
                    let attack = state.sample(true, TripContext { frame: 2.0, ..trip });
                    assert_eq!(attack.requested, MAPPER_A);
                    assert_eq!(attack.stick, Some([facing, 0.]));
                    state.sample(
                        true,
                        TripContext {
                            status: crate::native_states::status::ATTACK_S4_START,
                            ..trip
                        },
                    );
                    state.sample(
                        true,
                        TripContext {
                            status: crate::native_states::status::WAIT,
                            ..trip
                        },
                    );
                    state.sample(true, trip);
                    assert!(state.watching());
                    assert_eq!(
                        state
                            .sample(true, TripContext { frame: 2.0, ..trip })
                            .requested,
                        MAPPER_A
                    );
                }
            }
        }
    }
    #[test]
    fn trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps() {
        use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
        use crate::native_input::ButtonTrace;
        let mut state = TripFollowup::default();
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held: FOLLOWUP_BUTTONS | LEFT_STICK_DIRECTIONS,
            ..ButtonTrace::default()
        };
        for frame in 1..=5 {
            let out = state.sample(
                true,
                TripContext {
                    frame: frame as f32,
                    ..ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                },
            );
            let mapped = overlay.apply_controls(
                human,
                [-1.0, -1.0],
                TurboOutput { stick: out.stick },
                0,
                out.requested,
                out.owned,
            );
            assert_eq!(mapped.buttons.held & FOLLOWUP_BUTTONS, out.requested);
            assert_eq!(mapped.stick, out.stick.unwrap());
            assert_eq!(mapped.buttons.pressed & MAPPER_A, out.requested);
        }
        let off = state.sample(
            false,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        let restored = overlay.apply_controls(
            human,
            [-1.0, -1.0],
            TurboOutput { stick: off.stick },
            0,
            off.requested,
            off.owned,
        );
        assert_eq!(
            restored.buttons.held & FOLLOWUP_BUTTONS,
            human.held & FOLLOWUP_BUTTONS
        );
        assert_eq!(restored.stick, [-1.0, -1.0]);
    }
    #[test]
    fn no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel() {
        for sample in [
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::WAIT,
            ),
            ctx(
                crate::native_states::status::WAIT,
                crate::native_states::status::SLIP,
            ),
            TripContext {
                kirby: false,
                ..ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                )
            },
        ] {
            assert_eq!(
                TripFollowup::default().sample(true, sample),
                ControllerOutput::default()
            );
        }
        for terminal in [
            TripContext {
                grounded: false,
                ..ctx(
                    crate::native_states::status::JUMP,
                    crate::native_states::status::SLIP,
                )
            },
            ctx(
                crate::native_states::status::DAMAGE,
                crate::native_states::status::SLIP,
            ),
        ] {
            let mut state = TripFollowup::default();
            state.sample(
                true,
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                ),
            );
            assert_eq!(state.sample(true, terminal).requested, 0);
            assert!(!state.watching());
            assert_eq!(
                state
                    .sample(
                        true,
                        TripContext {
                            frame: 2.0,
                            ..terminal
                        }
                    )
                    .owned,
                0
            );
        }
        let mut state = TripFollowup::default();
        state.sample(
            true,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        assert_eq!(
            state
                .sample(
                    false,
                    ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                )
                .owned,
            0
        );
        assert!(!state.watching());
        state.sample(
            true,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        assert_eq!(
            state
                .sample(
                    true,
                    TripContext {
                        object: 3,
                        ..ctx(
                            crate::native_states::status::WAIT,
                            crate::native_states::status::WAIT,
                        )
                    }
                )
                .owned,
            0
        );
        assert!(!state.watching());
    }
}
