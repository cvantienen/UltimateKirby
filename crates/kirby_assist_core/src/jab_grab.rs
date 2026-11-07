//! A physical Jab 1 followed by a fresh Grab press starts Kirby's follow-up.
//! Native cancel/readiness gates own the earliest recovery -> one gentle
//! forward input -> native-frame grab retries.
//! No distance gate, forced status, recovery edits or hitbox scan.
use crate::{
    gameplay::ControllerOutput,
    manual_controls::{ShortcutContext, LEFT_STICK_DIRECTIONS},
    native_input::{MAPPER_A, MAPPER_L, MAPPER_R},
    timing::{motion_elapsed, motion_hash},
};
pub const TRIGGER: u32 = MAPPER_R;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Followup {
    #[default]
    Grab,
    DownSmash,
}
impl Followup {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grab => "Jab -> Grab",
            Self::DownSmash => "Jab -> D smash",
        }
    }
    pub fn cycle(self) -> Self {
        match self {
            Self::Grab => Self::DownSmash,
            Self::DownSmash => Self::Grab,
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct Readiness {
    pub can_grab: bool,
    pub can_walk: bool,
    /// Actual native cancel permission, not a catch transition term alone.
    pub cancel_jab: bool,
}
#[derive(Default)]
pub struct JabGrab {
    previous_buttons: Option<u32>,
    previous: Option<(i32, u64, f32, f32)>,
    // 0 idle, 3 jab recovery, 4 follow-up delivery,
    // 5 release the physical Grab trigger (no auto-pummel).
    phase: u8,
    followup: Followup,
    latched: bool,
    age: f32,
    request_age: f32,
    walk_request_at: Option<f32>,
    output: ControllerOutput,
}
impl JabGrab {
    pub fn active(&self) -> bool {
        matches!(self.phase, 1..=4)
    }
    pub fn cancel_pending(&mut self) {
        self.phase = if self.phase != 0 && self.previous_buttons.is_some_and(|b| b & TRIGGER != 0) {
            5
        } else {
            0
        };
        self.previous = None;
        self.output = ControllerOutput::default();
    }
    fn tail(&mut self, buttons: u32) -> ControllerOutput {
        self.phase = if buttons & TRIGGER != 0 { 5 } else { 0 };
        self.output = ControllerOutput {
            owned: buttons & TRIGGER,
            ..ControllerOutput::default()
        };
        self.output
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        buttons: u32,
        stick: [f32; 2],
        context: Option<ShortcutContext>,
        can_grab: bool,
    ) -> ControllerOutput {
        self.sample_with_readiness(
            enabled,
            buttons,
            stick,
            context,
            Readiness {
                can_grab,
                ..Readiness::default()
            },
        )
    }
    pub fn sample_with_followup(
        &mut self,
        enabled: bool,
        buttons: u32,
        stick: [f32; 2],
        context: Option<ShortcutContext>,
        ready: Readiness,
        followup: Followup,
    ) -> ControllerOutput {
        if self.phase == 0 {
            self.followup = followup;
        }
        self.sample_with_readiness(enabled, buttons, stick, context, ready)
    }
    fn followup_output(&self) -> ControllerOutput {
        match self.followup {
            Followup::Grab => crate::move_input::output([0.0; 2], MAPPER_R),
            Followup::DownSmash => crate::move_input::output([0.0, -1.0], MAPPER_A),
        }
    }
    pub fn sample_with_readiness(
        &mut self,
        enabled: bool,
        buttons: u32,
        stick: [f32; 2],
        context: Option<ShortcutContext>,
        ready: Readiness,
    ) -> ControllerOutput {
        let pressed = buttons & !self.previous_buttons.unwrap_or(0);
        self.previous_buttons = Some(buttons);
        if buttons & TRIGGER == 0 {
            self.latched = false;
        }
        let Some(c) = context else {
            self.cancel_pending();
            return ControllerOutput::default();
        };
        if !enabled {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        if self.phase == 5 {
            return self.tail(buttons);
        }
        let valid = stick.iter().all(|v| v.is_finite())
            && stick[0].abs() <= 0.2
            && stick[1].abs() <= 0.2
            && c.situation == 0
            && c.facing.is_finite()
            && c.facing.abs() >= 0.5
            && c.motion != 0
            && c.frame.is_finite()
            && c.frame >= 0.0
            && c.rate.is_finite()
            && c.rate > 0.0
            && buttons & !(TRIGGER | MAPPER_L | LEFT_STICK_DIRECTIONS) == 0;
        if !valid {
            return if self.active() {
                self.tail(buttons)
            } else {
                ControllerOutput::default()
            };
        }
        if self.phase == 0 {
            let jab = c.status == crate::native_states::status::ATTACK
                && c.motion == motion_hash("attack_11");
            // The player supplies Jab 1 normally, releases A, then presses
            // their mapped Grab button. Consume that Grab edge during jab so
            // it cannot become a late jab continuation or premature grab.
            let grab_after_jab = jab && buttons & MAPPER_A == 0 && pressed & TRIGGER != 0;
            if !grab_after_jab || self.latched {
                return ControllerOutput::default();
            }
            self.latched = true;
            self.phase = 3;
            self.age = 0.0;
            self.request_age = 0.0;
            self.walk_request_at = None;
            self.previous = Some((c.status, c.motion, c.frame, c.rate));
        }
        let stamp = (c.status, c.motion, c.frame, c.rate);
        let delta = if c.stopped {
            0.0
        } else {
            motion_elapsed(self.previous, stamp, 0.0)
        };
        self.previous = Some(stamp);
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            return self.tail(buttons);
        }
        self.age += delta;
        if self.age > 90.0 && self.phase != 4 {
            return self.tail(buttons);
        }
        if matches!(
            c.status,
            crate::native_states::status::CATCH..=crate::native_states::status::CATCH_JUMP
        ) || self.phase == 4
            && self.followup == Followup::DownSmash
            && matches!(
                c.status,
                crate::native_states::status::ATTACK_LW4_START
                    ..=crate::native_states::status::ATTACK_LW4
            )
        {
            return self.tail(buttons);
        }
        let jab = c.status == crate::native_states::status::ATTACK
            && c.motion == motion_hash("attack_11");
        if !(matches!(c.status,crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV)
            || jab
            || self.phase == 4
                && matches!(
                    c.status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_OFF
                ))
        {
            return self.tail(buttons);
        }
        if self.phase == 2 && jab {
            self.phase = 3;
            self.request_age = 0.0;
            // Release A even in hitstop, before jab 2 can buffer.
            self.output = crate::move_input::output([0.0; 2], 0);
        }
        if c.stopped {
            return self.output;
        }
        self.output = crate::move_input::output([0.0; 2], 0);
        if self.phase == 1 {
            self.request_age += delta;
            if self.request_age > crate::move_input::ACCEPTANCE_TIMEOUT {
                return self.tail(buttons);
            }
            // Neutralize crouch first, rather than turning A into a down tilt.
            if matches!(
                c.status,
                crate::native_states::status::WAIT..=crate::native_states::status::WALK_BRAKE
            ) {
                self.phase = 2;
                self.request_age = 0.0;
                self.output = crate::move_input::output([0.0; 2], MAPPER_A);
            }
        } else if self.phase == 2 {
            self.request_age += delta;
            if self.request_age > crate::move_input::ACCEPTANCE_TIMEOUT {
                return self.tail(buttons);
            }
            // Retry rejected jab input with release gaps. Exact attack_11
            // acceptance above releases A before jab 2 can buffer.
            if crate::move_input::delivery_pressed(self.request_age) {
                self.output = crate::move_input::output([0.0; 2], MAPPER_A);
            }
        } else if self.phase == 3 {
            // Start immediately when native jab recovery/cancel permits movement.
            // One gentle forward input only; duplicates retain it until the
            // next native frame, never advancing from controller poll counts.
            if let Some(at) = self.walk_request_at {
                if self.age - at >= 1.0 {
                    // A one-frame movement input can be accepted without a
                    // separately observed WALK state. Start ordinary grab/
                    // smash retries now; the native buffer decides acceptance.
                    self.phase = 4;
                    self.request_age = 0.0;
                    self.output = self.followup_output();
                } else {
                    self.output = crate::move_input::output([c.facing.signum() * 0.3, 0.0], 0);
                }
            } else if (!jab || ready.cancel_jab)
                && (ready.can_walk
                    || !jab
                        && matches!(
                            c.status,
                            crate::native_states::status::WAIT
                                ..=crate::native_states::status::TURN_RUN_BRAKE
                        ))
            {
                self.walk_request_at = Some(self.age);
                self.output = crate::move_input::output([c.facing.signum() * 0.3, 0.0], 0);
            }
        } else if self.phase == 4 {
            // Fastest fresh edges: one native frame pressed, one released.
            // No range/readiness veto or delivery timeout after the walk was
            // registered. Native grab/capture, interruption or cancellation ends it.
            self.request_age = (self.request_age + delta) % 2.0;
            if self.request_age < 1.0 {
                self.output = self.followup_output();
            }
        }
        self.output
    }
}
