//! Read-only telemetry, compiled only into diagnostic builds. No new native queries.
use super::{LedgeContext, LedgeOutput, LedgeSettings, Opponent, OptionPlan, Pending};
use super::{LedgeTrap, DASH_GRAB_START, GRAB_START, TURN_GRAB_START};
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
    pub fn matches(self, c: LedgeContext) -> bool {
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
            Self::Inhale => c.status == 0x1dc || (0x1e1..=0x1f2).contains(&c.status),
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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CalibrationFrame {
    pub target: u32,
    pub slot: PresetSlot,
    pub expected: CalibrationMove,
    pub context: LedgeContext,
    pub enemy: Option<Opponent>,
    pub output: LedgeOutput,
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

impl LedgeTrap {
    /// One copied observation for every ledge response family. This deliberately
    /// exposes missing recipe data as unknown rather than guessing native frames.
    pub fn calibration_frame(
        &self,
        settings: LedgeSettings,
        context: LedgeContext,
        enemy: Option<Opponent>,
        choice: &'static str,
    ) -> Option<CalibrationFrame> {
        let slot = self.calibration_slot()?;
        let mut expected = self.diagnostic_move.unwrap_or(CalibrationMove::unknown());
        let frames = match self.pending {
            Pending::DownTilt => Some((
                settings.fighter.down_tilt,
                NativeRule::Statuses(&[super::DOWN_TILT]),
            )),
            Pending::ForwardTilt => Some((
                settings.fighter.forward_tilt,
                NativeRule::Statuses(&[super::FORWARD_TILT]),
            )),
            Pending::DownSmash => Some((
                settings.fighter.down_smash_toward(
                    self.roll_intercept
                        .map_or(0.0, |spot| spot.endpoint - context.position[0]),
                    context.facing,
                ),
                NativeRule::Ground {
                    startup: 0x30,
                    attack: super::DOWN_SMASH,
                },
            )),
            Pending::NeutralAir => Some((
                settings.fighter.neutral_air,
                NativeRule::Aerial {
                    motion: super::NEUTRAL_AIR_MOTION,
                    custom_status: None,
                },
            )),
            _ => None,
        };
        if let Some((frames, native)) = frames {
            expected = CalibrationMove {
                kind: CalibrationKind::Attack,
                native,
                first: Some(frames.first),
                last: Some(frames.last),
                full_hop: self.jump_committed.then_some(self.jump_full),
                drift: self.jump_committed.then_some(self.neutral_nair_drift),
                drift_until: (self.neutral_nair_drift != 0.0)
                    .then_some(super::NAIR_NUDGE_END_FRAME),
            };
        } else if self.pending == Pending::Grab
            || matches!(choice, "grab" | "shield_grab" | "shield_rear")
                && self.diagnostic_move.is_none()
        {
            let first = settings.fighter.grab.startup(context.status);
            expected = CalibrationMove {
                kind: CalibrationKind::Grab,
                native: NativeRule::Statuses(&[GRAB_START, DASH_GRAB_START, TURN_GRAB_START]),
                first: Some(first),
                last: Some(
                    first.saturating_add(settings.fighter.grab.active_frames.saturating_sub(1)),
                ),
                ..CalibrationMove::unknown()
            };
        } else if matches!(
            self.pending,
            Pending::DownSpecial | Pending::NeutralSpecial | Pending::UpSpecial | Pending::Bomb
        ) {
            let (first, native) = match self.pending {
                Pending::DownSpecial => (
                    settings.fighter.down_special_startup,
                    NativeRule::Statuses(&[0x1df, 0x1ff, 0x200]),
                ),
                Pending::NeutralSpecial => (
                    Some(crate::fighter_profiles::kirby::INHALE_STARTUP),
                    NativeRule::Inhale,
                ),
                Pending::UpSpecial => (
                    settings.fighter.up_special_startup,
                    NativeRule::Statuses(&[0x1de]),
                ),
                _ => (Some(11), NativeRule::Statuses(&[0x1df, 0x1e6, 0x1e7])),
            };
            expected = CalibrationMove {
                kind: CalibrationKind::Special,
                first,
                native,
                ..CalibrationMove::unknown()
            };
        } else if self.samus_zair.is_some() || matches!(choice, "zair" | "shield_zair") {
            let frames = crate::fighter_profiles::samus::ZAIR_ACTIVE;
            expected = CalibrationMove {
                kind: CalibrationKind::Zair,
                native: NativeRule::Statuses(&[0xf8]),
                first: Some(frames.first),
                last: Some(frames.last),
                full_hop: Some(false),
                ..CalibrationMove::unknown()
            };
        } else if self.drop_dair.is_some() {
            expected = CalibrationMove {
                kind: CalibrationKind::Attack,
                native: NativeRule::Aerial {
                    motion: crate::timing::motion_hash("attack_air_lw"),
                    custom_status: None,
                },
                first: Some(crate::fighter_profiles::kirby::DROP_DAIR_FIRST_ACTIVE as u8),
                full_hop: Some(false),
                ..CalibrationMove::unknown()
            };
        } else if matches!(expected.kind, CalibrationKind::Unknown) {
            if matches!(choice, "nair" | "shield_nair" | "drop_nair") {
                let f = settings.fighter.neutral_air;
                expected = CalibrationMove {
                    kind: CalibrationKind::Attack,
                    native: NativeRule::Aerial {
                        motion: super::NEUTRAL_AIR_MOTION,
                        custom_status: None,
                    },
                    first: Some(f.first),
                    last: Some(f.last),
                    ..CalibrationMove::unknown()
                };
            } else if choice == "shield" {
                expected = CalibrationMove {
                    kind: CalibrationKind::Shield,
                    native: NativeRule::Statuses(&[
                        super::GUARD_ON,
                        super::GUARD,
                        super::GUARD_DAMAGE,
                    ]),
                    ..CalibrationMove::unknown()
                };
            } else {
                let frames = match choice {
                    "down_tilt" | "down_tilt_shield" => Some((
                        settings.fighter.down_tilt,
                        NativeRule::Statuses(&[super::DOWN_TILT]),
                    )),
                    "forward_tilt" | "poke" => Some((
                        settings.fighter.forward_tilt,
                        NativeRule::Statuses(&[super::FORWARD_TILT]),
                    )),
                    "down_smash" => Some((
                        settings.fighter.down_smash_toward(
                            self.roll_intercept
                                .map_or(0.0, |spot| spot.endpoint - context.position[0]),
                            context.facing,
                        ),
                        NativeRule::Ground {
                            startup: 0x30,
                            attack: super::DOWN_SMASH,
                        },
                    )),
                    _ => None,
                };
                if let Some((frames, native)) = frames {
                    expected = CalibrationMove {
                        kind: CalibrationKind::Attack,
                        native,
                        first: Some(frames.first),
                        last: Some(frames.last),
                        ..CalibrationMove::unknown()
                    };
                }
            }
        }
        if expected.kind == CalibrationKind::Unknown {
            if choice == "stone" {
                expected = CalibrationMove {
                    kind: CalibrationKind::Special,
                    first: settings.fighter.down_special_startup,
                    native: NativeRule::Statuses(&[0x1df, 0x1ff, 0x200]),
                    ..CalibrationMove::unknown()
                };
            } else if choice == "inhale" {
                expected = CalibrationMove {
                    kind: CalibrationKind::Special,
                    first: Some(crate::fighter_profiles::kirby::INHALE_STARTUP),
                    native: NativeRule::Inhale,
                    ..CalibrationMove::unknown()
                };
            } else if choice == "bair"
                && settings.implementation == Some(super::ResponseImplementation::Kirby)
            {
                let f = crate::fighter_profiles::kirby::JUMP_BAIR;
                expected = CalibrationMove {
                    kind: CalibrationKind::Attack,
                    native: NativeRule::Aerial {
                        motion: crate::timing::motion_hash("attack_air_b"),
                        custom_status: None,
                    },
                    first: Some(f.first),
                    last: Some(f.last),
                    full_hop: Some(false),
                    ..CalibrationMove::unknown()
                };
            }
        }
        let range = if expected.kind == CalibrationKind::Grab {
            Some(settings.fighter.grab.standing_range)
        } else if matches!(self.pending, Pending::DownTilt | Pending::ForwardTilt)
            || matches!(
                choice,
                "down_tilt" | "down_tilt_shield" | "forward_tilt" | "poke"
            )
        {
            Some(settings.tilt_range)
        } else if slot == PresetSlot::Roll && choice == "down_smash" {
            Some(settings.roll_dash_range)
        } else if expected.kind == CalibrationKind::Attack {
            Some(settings.anti_air_range)
        } else {
            None
        };
        let lead = if slot != PresetSlot::Neutral {
            None
        } else {
            match choice {
                "nair" => Some(
                    f32::from(settings.fighter.normal_hit_lead_frames)
                        + if settings.implementation == Some(super::ResponseImplementation::Kirby) {
                            2.0
                        } else {
                            0.0
                        },
                ),
                "stone" => Some(f32::from(settings.fighter.normal_hit_lead_frames)),
                "inhale" => Some(f32::from(crate::fighter_profiles::kirby::INHALE_HIT_LEAD)),
                "bair" if settings.implementation == Some(super::ResponseImplementation::Kirby) => {
                    Some(f32::from(crate::fighter_profiles::kirby::NEUTRAL_BAIR_LEAD))
                }
                "zair" => Some(f32::from(crate::fighter_profiles::samus::ZAIR_HIT_LEAD)),
                _ => None,
            }
        };
        let mut enemy = enemy.filter(|enemy| Some(enemy.id) == self.target);
        if let Some(enemy) = enemy.as_mut() {
            enemy.velocity = self.observed_velocity;
        }
        Some(CalibrationFrame {
            target: self.target?,
            slot,
            expected,
            context,
            enemy,
            output: self.last_output,
            phase: self.action_name(),
            complete: self.manual_after_response(),
            captured: self.grab_handed_back,
            blocked: matches!(self.plan, OptionPlan::Shield { blocked: true })
                || context.status == super::GUARD_DAMAGE,
            endpoint: self.roll_interception(),
            configured_range: range,
            lead,
            distance_agnostic: settings.distance_agnostic,
            grab_button: settings.grab_button,
        })
    }

    pub fn calibration_slot(&self) -> Option<PresetSlot> {
        match self.plan {
            OptionPlan::NeutralAir => Some(PresetSlot::Jump),
            OptionPlan::Roll => Some(PresetSlot::Roll),
            OptionPlan::Shield { .. } => Some(PresetSlot::Attack),
            OptionPlan::DropShield => Some(PresetSlot::Drop),
            OptionPlan::Watch if self.climb_seen => Some(PresetSlot::Neutral),
            _ => None,
        }
    }
}

#[cfg(test)]
mod calibration_tests {
    use super::*;
    use crate::ledge_trap::tests::{context, enemy};

    #[test]
    fn calibration_covers_every_plan_and_keeps_expected_native_motion_exact() {
        for (plan, climb, slot, choice) in [
            (OptionPlan::Watch, true, PresetSlot::Neutral, "nair"),
            (OptionPlan::NeutralAir, false, PresetSlot::Jump, "nair"),
            (OptionPlan::Roll, false, PresetSlot::Roll, "grab"),
            (
                OptionPlan::Shield { blocked: false },
                false,
                PresetSlot::Attack,
                "shield_nair",
            ),
            (OptionPlan::DropShield, false, PresetSlot::Drop, "shield"),
        ] {
            let trap = LedgeTrap {
                target: Some(2),
                plan,
                climb_seen: climb,
                ..LedgeTrap::default()
            };
            let snapshot = trap
                .calibration_frame(
                    LedgeSettings::default(),
                    context(),
                    Some(enemy(0x7a, false)),
                    choice,
                )
                .unwrap();
            assert_eq!(snapshot.slot, slot);
            if choice == "nair" || choice == "shield_nair" {
                let wrong = LedgeContext {
                    status: super::super::ATTACK_AIR,
                    motion_kind: crate::timing::motion_hash("attack_air_f"),
                    ..context()
                };
                assert!(!snapshot.expected.native.matches(wrong));
                let exact = LedgeContext {
                    motion_kind: super::super::NEUTRAL_AIR_MOTION,
                    ..wrong
                };
                assert!(snapshot.expected.native.matches(exact));
            }
        }
    }
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
    /// Inspect the exact policy inputs and counters, without changing decisions.
    pub fn roll_grab_diagnostic(&self) -> Option<RollGrabSnapshot> {
        if !self.roll_grab_selected {
            return None;
        }
        let (settings, ctx, enemy) = self.last_sample?;
        let first = settings.fighter.grab.startup(ctx.status);
        Some(RollGrabSnapshot {
            target: self.target,
            phase: self.action_name(),
            status: ctx.status,
            motion_frame: ctx.motion_frame,
            position: ctx.position,
            velocity_x: ctx.velocity[0],
            facing: ctx.facing,
            endpoint: self.roll_intercept.map(|spot| spot.endpoint),
            stand_x: self.roll_intercept.map(|spot| spot.stand_x),
            usable_reach: settings.fighter.grab.roll_usable_reach(),
            grab_range: settings.fighter.grab.standing_range,
            can_control: ctx.can_control,
            can_act: ctx.can_act,
            can_grab: ctx.can_grab,
            stopped: ctx.stopped,
            safe_ground: ctx.safe_ground,
            enemy_status: enemy.map(|enemy| enemy.status),
            enemy_frame: enemy.and_then(|enemy| enemy.motion.map(|motion| motion.frame)),
            enemy_position: enemy.map(|enemy| enemy.position),
            vulnerable: enemy.map(|enemy| enemy.vulnerable),
            vulnerable_in: enemy.and_then(|enemy| enemy.vulnerable_in),
            adjusted_opens_in: enemy
                .and_then(|enemy| super::roll::roll_grab_target(enemy).vulnerable_in),
            actionable_in: enemy.and_then(|enemy| {
                enemy
                    .actionable_in
                    .or_else(|| enemy.motion.and_then(|motion| motion.remaining))
            }),
            active: [
                first,
                first.saturating_add(settings.fighter.grab.active_frames.saturating_sub(1)),
            ],
            requested: self.last_output.requested,
            grab_button: settings.grab_button,
            attempts: self.attempts,
            animations: self.grab_animations,
            native_grabbing: matches!(ctx.status, GRAB_START | DASH_GRAB_START | TURN_GRAB_START),
            captured: self.grab_handed_back,
        })
    }
}
