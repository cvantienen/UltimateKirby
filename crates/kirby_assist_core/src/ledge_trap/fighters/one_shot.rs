//! Bounded single-move variants, used only by fighters that select them.
//! Paired jump+A stays buffered until the exact aerial starts. Smashes release
//! A on native startup, so an accepted request never becomes an automatic charge.
//! Grounded special recipes use B instead and release on listed native statuses.
use super::super::{
    attempt::NativeAttempt, LedgeContext, LedgeOutput, ATTACK_AIR, GUARD, GUARD_DAMAGE, GUARD_ON,
    JUMP_SQUAT, LEDGE_BUTTONS,
};
use crate::manual_controls::RIGHT_STICK_DIRECTIONS;
use crate::native_input::{
    MAPPER_A, MAPPER_B, MAPPER_RIGHT_STICK_LEFT, MAPPER_RIGHT_STICK_RIGHT, MAPPER_X, MAPPER_ZL,
    MAPPER_ZR,
};

#[derive(Clone, Copy)]
pub(super) enum MoveKind {
    Ground {
        startup: i32,
        attack: i32,
    },
    Aerial {
        motion: u64,
    },
    /// Some fighters continue an ordinary aerial in a fighter-specific status.
    /// Exact motion still distinguishes the first attack from chained followups.
    NativeAerial {
        motion: u64,
        status: i32,
    },
    /// One grounded B input, released on any explicitly listed native startup.
    Special {
        statuses: &'static [i32],
    },
}
#[derive(Clone, Copy)]
pub(super) struct Move {
    pub kind: MoveKind,
    pub first: u8,
    pub last: u8,
    /// Horizontal is relative to facing; vertical selects the aerial/attack.
    pub stick: [f32; 2],
    /// Optional post-acceptance outward drift for ledge-drop aerials.
    pub drift: f32,
}
impl Move {
    #[cfg(feature = "diagnostic-logging")]
    pub(in crate::ledge_trap) fn calibration(self) -> super::super::CalibrationMove {
        use super::super::{CalibrationKind, CalibrationMove, NativeRule};
        let native = match self.kind {
            MoveKind::Ground { startup, attack } => NativeRule::Ground { startup, attack },
            MoveKind::Aerial { motion } => NativeRule::Aerial {
                motion,
                custom_status: None,
            },
            MoveKind::NativeAerial { motion, status } => NativeRule::Aerial {
                motion,
                custom_status: Some(status),
            },
            MoveKind::Special { statuses } => NativeRule::Statuses(statuses),
        };
        CalibrationMove {
            kind: if matches!(self.kind, MoveKind::Special { .. }) {
                CalibrationKind::Special
            } else {
                CalibrationKind::Attack
            },
            native,
            first: Some(self.first),
            last: Some(self.last),
            drift: Some(self.drift),
            ..CalibrationMove::unknown()
        }
    }
    pub(super) fn aerial(self) -> bool {
        matches!(
            self.kind,
            MoveKind::Aerial { .. } | MoveKind::NativeAerial { .. }
        )
    }
    fn native_air_status(self, status: i32) -> bool {
        matches!(self.kind, MoveKind::NativeAerial { status: native, .. } if native == status)
    }
}
pub(crate) struct OneShot {
    recipe: Move,
    drift_until: Option<u8>,
    full_hop: bool,
    jump_before_attack: bool,
    turn_before_jump: bool,
    attack_cancel: bool,
    cancel_mapped: bool,
    cancel_right_stick: Option<[f32; 2]>,
    jump_requested: bool,
    facing: f32,
    attempt: NativeAttempt,
    primed: bool,
    jump_seen: bool,
    accepted: bool,
    finished: bool,
    output: LedgeOutput,
}
impl OneShot {
    #[cfg(feature = "diagnostic-logging")]
    pub(in crate::ledge_trap) fn calibration(&self) -> super::super::CalibrationMove {
        let mut result = self.recipe.calibration();
        result.full_hop = self.aerial().then_some(self.full_hop);
        result.drift = Some(self.facing * self.recipe.drift);
        result.drift_until = self.drift_until;
        result
    }
    pub(super) fn new(recipe: Move, facing: f32) -> Self {
        Self {
            recipe,
            drift_until: None,
            full_hop: false,
            jump_before_attack: false,
            turn_before_jump: false,
            attack_cancel: false,
            cancel_mapped: false,
            cancel_right_stick: None,
            jump_requested: false,
            facing: facing.signum(),
            attempt: NativeAttempt::default(),
            primed: false,
            jump_seen: false,
            accepted: false,
            finished: false,
            output: LedgeOutput::default(),
        }
    }
    pub(super) fn with_hop(mut self, full: bool) -> Self {
        self.full_hop = full && self.aerial();
        self
    }
    /// Bound the stick nudge using native animation time, not mapper polls.
    pub(super) fn with_drift_until(mut self, frame: u8) -> Self {
        self.drift_until = Some(frame);
        self
    }
    /// Own the turnaround through native confirmation, even when opponent
    /// tracking disappears or a physical wiggle would otherwise override it.
    /// Launch neutrally, then buffer directional A in native jump squat.
    pub(super) fn with_turn_before_jump(mut self) -> Self {
        self.turn_before_jump = true;
        self.jump_before_attack = true;
        self
    }
    pub(super) fn with_attack_cancel(mut self) -> Self {
        self.attack_cancel = true;
        self
    }
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        (self.attack_cancel && !self.finished)
            .then_some(self.cancel_right_stick.unwrap_or([0.0; 2]))
    }
    /// Called only after normal mapping confirms both attack and C-stick in
    /// the requested direction. An analog-axis write alone is not a flick.
    pub fn mapped_cstick(&mut self, buttons: u32, x: i8) {
        if self.cancel_right_stick.is_some()
            && buttons & 0x81 == 0x81
            && f32::from(x) * self.facing > 0.0
        {
            self.cancel_mapped = true;
        }
    }
    pub fn attack_status(&self) -> Option<i32> {
        match self.recipe.kind {
            MoveKind::Ground { attack, .. } => Some(attack),
            // These variants do not expose one generic native attack status;
            MoveKind::Aerial { .. } | MoveKind::NativeAerial { .. } | MoveKind::Special { .. } => {
                None
            }
        }
    }
    pub fn aerial(&self) -> bool {
        self.recipe.aerial()
    }
    pub fn owns_jump_state(&self, status: i32) -> bool {
        self.aerial()
            && !self.finished
            && (self.recipe.native_air_status(status)
                || self.attack_cancel && !self.jump_seen && matches!(status, 0..=0x8 | 0x27..=0x2c)
                || self.turn_before_jump && !self.jump_seen && matches!(status, 0..=0x8)
                || matches!(
                    status,
                    JUMP_SQUAT | 0xb | 0xe | 0xf | 0x16..=0x18 | ATTACK_AIR
                ))
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn sample(&mut self, c: LedgeContext) -> LedgeOutput {
        if self.finished || !c.can_control && !c.stopped {
            return self.finish();
        }
        let started = match self.recipe.kind {
            MoveKind::Ground { startup, attack } => (startup..=attack).contains(&c.status),
            MoveKind::Aerial { motion } => c.status == ATTACK_AIR && c.motion_kind == motion,
            MoveKind::NativeAerial { motion, status } => {
                (c.status == ATTACK_AIR || c.status == status) && c.motion_kind == motion
            }
            MoveKind::Special { statuses } => statuses.contains(&c.status),
        };
        let fresh = self.attempt.advance(c, started, 90);
        if self.attempt.expired {
            return self.finish();
        }
        if !self.aerial() && started {
            return self.finish();
        }
        let supported = if c.situation == 0 {
            matches!(c.status, 0..=JUMP_SQUAT | 0x11..=0x18 | GUARD_ON | GUARD | GUARD_DAMAGE)
                || self.attack_cancel && matches!(c.status, 0x27..=0x2c) && c.motion_frame <= 2.0
        } else {
            self.aerial()
                && c.situation == 2
                && (matches!(c.status, 0xb | 0xe | 0xf | ATTACK_AIR)
                    || self.recipe.native_air_status(c.status))
        };
        if !supported || self.jump_seen && c.situation == 0 && c.status != JUMP_SQUAT {
            return self.finish();
        }
        self.jump_seen |= c.status == JUMP_SQUAT || c.situation == 2;
        if c.status == ATTACK_AIR || self.recipe.native_air_status(c.status) {
            if !started || c.motion_frame > f32::from(self.recipe.last) {
                return self.finish();
            }
            self.accepted = true;
            self.cancel_right_stick = None;
            let drift = if self
                .drift_until
                .is_none_or(|end| c.motion_frame < f32::from(end))
            {
                self.recipe.drift
            } else {
                0.0
            };
            self.output = controls([self.facing * drift, 0.0], 0);
            return self.output;
        }
        if self.accepted {
            return self.finish();
        }
        if self.attack_cancel && !self.jump_seen && c.situation == 0 {
            if !fresh {
                return self.output;
            }
            if !self.cancel_mapped {
                self.cancel_right_stick = Some([self.facing, 0.0]);
                // Native raw Controller::ButtonBitfield r_right/r_left, as
                // well as analog axes. Send a genuine press through the same
                // edge history as every other assisted button.
                let direction_bit = if self.facing > 0.0 {
                    MAPPER_RIGHT_STICK_RIGHT
                } else {
                    MAPPER_RIGHT_STICK_LEFT
                };
                self.output = controls([0.0; 2], direction_bit);
            } else {
                self.cancel_right_stick = None;
                self.output = controls([-self.facing, 0.0], MAPPER_X | MAPPER_ZL);
            }
            self.output.owned |= RIGHT_STICK_DIRECTIONS;
            return self.output;
        }
        self.cancel_right_stick = None;
        if c.status == GUARD_DAMAGE {
            self.output = controls([0.0; 2], MAPPER_ZR);
            return self.output;
        }
        if self.turn_before_jump && !self.jump_seen && c.situation == 0 {
            if c.facing * self.facing < 0.5 {
                // One grounded turn, not a walking approach: do not veto the
                // turn using a ground probe beyond the ledge. Native jump
                // readiness permits a jump out of turning even if can_act
                // (the locomotion helper's gate) is false during the turn.
                self.output = controls([self.facing * 0.6, 0.0], 0);
                return self.output;
            }
        }
        if self.turn_before_jump && self.jump_seen && c.facing * self.facing < 0.5 {
            if c.situation == 2 {
                return self.finish(); // No A into a forward-facing airborne state.
            }
            self.output = controls([self.facing * 0.6, 0.0], 0);
            return self.output;
        }
        let stick = [self.facing * self.recipe.stick[0], self.recipe.stick[1]];
        if self.aerial() && (c.status == JUMP_SQUAT || c.situation == 2) {
            if self.attack_cancel {
                // The tilt already buffered attack. Do not inject a second A
                // into the cancel; only select its direction after native turn.
                if c.facing * self.facing < 0.5 {
                    return self.finish();
                }
                self.output = controls([-self.facing, 0.0], 0);
                self.output.owned |= RIGHT_STICK_DIRECTIONS;
                return self.output;
            }
            self.output = if self.full_hop && c.status == JUMP_SQUAT {
                controls([0.0; 2], MAPPER_X) // Full hop: never pair A before takeoff.
            } else {
                controls(stick, MAPPER_A)
            };
            return self.output;
        }
        if !fresh {
            return self.output;
        }
        let previous = self.output.requested;
        self.output = controls([0.0; 2], 0);
        if !self.aerial() && !self.primed {
            self.primed = true; // Fresh neutral/release before a smash flick.
            return self.output;
        }
        if self.full_hop {
            if c.can_jump
                && (c.can_act || matches!(c.status, GUARD_ON | GUARD))
                && (self.jump_requested || self.attempt.issue(c, previous, MAPPER_X))
            {
                self.jump_requested = true;
                self.output = controls([0.0; 2], MAPPER_X);
            }
            return self.output;
        }
        let button = if matches!(self.recipe.kind, MoveKind::Special { .. }) {
            MAPPER_B
        } else {
            MAPPER_A
        };
        if self.jump_before_attack && self.aerial() {
            if c.can_jump
                && (c.can_act || self.turn_before_jump)
                && self.attempt.issue(c, previous, MAPPER_X)
            {
                self.output = controls([0.0; 2], MAPPER_X);
            }
            return self.output;
        }
        if (c.can_act || self.aerial() && matches!(c.status, GUARD_ON | GUARD))
            && (!self.aerial() || c.can_jump)
            && (self.attempt.holding_special(c) || self.attempt.issue(c, previous, button))
        {
            self.output = controls(stick, button | if self.aerial() { MAPPER_X } else { 0 });
        }
        self.output
    }
    fn finish(&mut self) -> LedgeOutput {
        self.finished = true;
        self.cancel_right_stick = None;
        self.output = LedgeOutput::default();
        self.output
    }
}
fn controls(stick: [f32; 2], requested: u32) -> LedgeOutput {
    LedgeOutput {
        owned: LEDGE_BUTTONS,
        consume_trigger: true,
        stick: Some(stick),
        requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledge_trap::tests::context;
    use crate::timing::motion_hash;
    fn own(frame: f32) -> LedgeContext {
        LedgeContext {
            motion_kind: motion_hash("wait"),
            motion_frame: frame,
            ..context()
        }
    }
    fn aerial() -> Move {
        Move {
            kind: MoveKind::Aerial {
                motion: motion_hash("attack_air_hi"),
            },
            first: 6,
            last: 31,
            stick: [0.0, 1.0],
            drift: 0.0,
        }
    }
    #[test]
    fn fighter_specific_first_fair_keeps_native_ownership_but_never_chains_a_followup() {
        let recipe = Move {
            kind: MoveKind::NativeAerial {
                motion: motion_hash("attack_air_f"),
                status: 0x1e1,
            },
            first: 7,
            last: 9,
            stick: [1.0, 0.0],
            drift: 0.0,
        };
        let mut action = OneShot::new(recipe, 1.0);
        assert_eq!(action.sample(own(0.0)).requested, MAPPER_X | MAPPER_A);
        assert!(action.owns_jump_state(0x1e1));
        let native = LedgeContext {
            status: 0x1e1,
            situation: 2,
            can_act: false,
            motion_kind: motion_hash("attack_air_f"),
            motion_frame: 1.0,
            ..own(1.0)
        };
        assert_eq!(action.sample(native).requested, 0);
        assert!(!action.finished());
        assert_eq!(action.attack_status(), None, "not a nair delay sample");
        let followup = LedgeContext {
            motion_kind: motion_hash("attack_air_f2"),
            ..native
        };
        assert_eq!(action.sample(followup), LedgeOutput::default());
        assert!(action.finished());
    }
    #[test]
    fn full_hop_holds_jump_without_attack_until_native_takeoff() {
        let mut action = OneShot::new(aerial(), 1.0).with_hop(true);
        assert_eq!(action.sample(own(0.0)).requested, MAPPER_X);
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            stopped: true,
            can_act: false,
            ..own(1.0)
        };
        assert_eq!(action.sample(squat).requested, MAPPER_X);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            ..own(2.0)
        };
        let out = action.sample(air);
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick, Some([0.0, 1.0]));
    }
    #[test]
    fn paired_up_air_buffers_through_frozen_squat_and_delayed_air_acceptance() {
        let mut action = OneShot::new(aerial(), 1.0);
        assert_eq!(action.sample(own(0.0)).requested, MAPPER_X | MAPPER_A);
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            motion_kind: motion_hash("jump_squat"),
            stopped: true,
            can_act: false,
            ..own(1.0)
        };
        let output = action.sample(squat);
        assert_eq!(output.requested, MAPPER_A);
        assert_eq!(output.stick, Some([0.0, 1.0]));
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            can_air_attack: false,
            motion_kind: motion_hash("jump_f"),
            ..own(2.0)
        };
        assert_eq!(action.sample(air).requested, MAPPER_A);
        let accepted = LedgeContext {
            status: ATTACK_AIR,
            motion_kind: motion_hash("attack_air_hi"),
            ..air
        };
        assert_eq!(action.sample(accepted).requested, 0);
        assert!(!action.finished());
        assert_eq!(
            action.sample(LedgeContext {
                motion_frame: 32.0,
                ..accepted
            }),
            LedgeOutput::default()
        );
        assert!(action.finished());
    }
    #[test]
    fn smash_primes_a_flick_and_releases_on_startup_without_charging() {
        let recipe = Move {
            kind: MoveKind::Ground {
                startup: 0x2d,
                attack: 0x2f,
            },
            first: 14,
            last: 19,
            stick: [1.0, 0.0],
            drift: 0.0,
        };
        let mut action = OneShot::new(recipe, -1.0);
        assert_eq!(action.sample(own(0.0)).requested, 0);
        let out = action.sample(own(1.0));
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick, Some([-1.0, 0.0]));
        assert_eq!(
            action.sample(LedgeContext {
                status: 0x2d,
                motion_kind: motion_hash("attack_s4_s"),
                ..own(0.0)
            }),
            LedgeOutput::default()
        );
        assert!(action.finished());
    }
    #[test]
    fn duplicate_observations_do_not_expire_or_repeat_a_jump_and_wrong_aerial_cancels() {
        let mut action = OneShot::new(aerial(), 1.0);
        let out = action.sample(own(0.0));
        for _ in 0..500 {
            assert_eq!(action.sample(own(0.0)), out);
        }
        assert!(!action.finished());
        assert_eq!(
            action.sample(LedgeContext {
                status: ATTACK_AIR,
                situation: 2,
                motion_kind: motion_hash("attack_air_n"),
                ..own(1.0)
            }),
            LedgeOutput::default()
        );
        assert!(action.finished());
    }
}
