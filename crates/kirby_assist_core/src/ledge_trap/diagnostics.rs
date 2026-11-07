//! Read-only file-timeline telemetry. No inferred fighter scheduling constants.
use super::{AssistSettings, ControllerOutput, GameplayContext, LedgeTrap, Opponent};
use crate::fighter_profiles::presets::PresetSlot;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalibrationKind {
    Attack,
    Special,
    Grab,
    Zair,
    Shield,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub enum NativeRule {
    Unknown,
    Ground {
        startup: i32,
        attack: i32,
    },
    Aerial {
        motion: u64,
        custom_status: Option<i32>,
    },
    Statuses(&'static [i32]),
    Inhale,
}
impl NativeRule {
    pub fn matches(self, c: GameplayContext) -> bool {
        match self {
            Self::Unknown => false,
            Self::Ground { startup, attack } => (startup..=attack).contains(&c.status),
            Self::Aerial {
                motion,
                custom_status,
            } => {
                (c.status == super::ATTACK_AIR || custom_status == Some(c.status))
                    && c.motion_kind == motion
            }
            Self::Statuses(statuses) => statuses.contains(&c.status),
            Self::Inhale => {
                c.status == crate::native_states::status::SPECIAL_N
                    || (crate::native_states::fighter_status::KIRBY_SPECIAL_N_LOOP
                        ..=crate::native_states::fighter_status::KIRBY_SPECIAL_N_SPIT)
                        .contains(&c.status)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CalibrationMove {
    pub kind: CalibrationKind,
    pub native: NativeRule,
    pub first: Option<u8>,
    pub last: Option<u8>,
    pub full_hop: Option<bool>,
    pub drift: Option<f32>,
    pub drift_until: Option<u8>,
    pub fast_fall_from: Option<u8>,
}
impl CalibrationMove {
    pub const fn unknown() -> Self {
        Self {
            kind: CalibrationKind::Unknown,
            native: NativeRule::Unknown,
            first: None,
            last: None,
            full_hop: None,
            drift: None,
            drift_until: None,
            fast_fall_from: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CalibrationFrame {
    pub target: u32,
    pub slot: PresetSlot,
    pub expected: CalibrationMove,
    /// Diagnostic-only native activity evidence; never a gameplay gate.
    pub native_attack_active: Option<bool>,
    pub timeline_end: Option<&'static str>,
    pub context: GameplayContext,
    pub enemy: Option<Opponent>,
    pub output: ControllerOutput,
    pub phase: &'static str,
    pub complete: bool,
    pub captured: bool,
    pub blocked: bool,
    pub endpoint: Option<[f32; 3]>,
    pub configured_range: Option<f32>,
    pub lead: Option<f32>,
    pub distance_agnostic: bool,
    pub grab_button: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RollGrabSnapshot {
    pub target: Option<u32>,
    pub phase: &'static str,
    pub status: i32,
    pub motion_frame: f32,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub facing: f32,
    pub endpoint: Option<f32>,
    pub stand_x: Option<f32>,
    pub usable_reach: f32,
    pub grab_range: f32,
    pub can_control: bool,
    pub can_act: bool,
    pub can_grab: bool,
    pub stopped: bool,
    pub safe_ground: [bool; 2],
    pub enemy_status: Option<i32>,
    pub enemy_frame: Option<f32>,
    pub enemy_position: Option<[f32; 2]>,
    pub vulnerable: Option<bool>,
    pub vulnerable_in: Option<f32>,
    pub adjusted_opens_in: Option<f32>,
    pub actionable_in: Option<f32>,
    pub active: [u8; 2],
    pub requested: u32,
    pub grab_button: u32,
    pub attempts: u8,
    pub animations: u8,
    pub native_grabbing: bool,
    pub captured: bool,
}

impl LedgeTrap {
    pub fn calibration_slot(&self) -> Option<PresetSlot> {
        self.slot
    }
    pub fn calibration_frame(
        &self,
        settings: AssistSettings,
        context: GameplayContext,
        enemy: Option<Opponent>,
        _: &str,
    ) -> Option<CalibrationFrame> {
        Some(CalibrationFrame {
            target: self.target?,
            slot: self.slot?,
            expected: CalibrationMove::unknown(),
            native_attack_active: None,
            timeline_end: self.last_timeline_end,
            context,
            enemy,
            output: self.last_output,
            phase: self.action_name(),
            complete: self.complete,
            captured: self.captured,
            blocked: context.status == crate::native_states::GUARD_DAMAGE,
            endpoint: None,
            configured_range: None,
            lead: None,
            distance_agnostic: settings.distance_agnostic,
            grab_button: settings.grab_button,
        })
    }
    pub fn roll_grab_diagnostic(&self) -> Option<RollGrabSnapshot> {
        None
    }
}
