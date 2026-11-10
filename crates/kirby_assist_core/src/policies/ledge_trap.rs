//! Shared ledge recognition and routing. The sole option catalog and every
//! fighter-specific input schedule live in startup-loaded sequences.txt.
//! No fighter recipe, move-start delay, hitbox window or retry engine lives here.
use crate::fighter_profiles::presets::PresetSlot;
pub use crate::native_input::ASSIST_BUTTONS;
use crate::native_input::MAPPER_L;
mod attempt {
    //! Read-only native-progress clock; no fighter timing, retries or wall-clock input scheduling.
    use super::{GameplayContext, Opponent};
    type Stamp = (i32, u64, f32, f32);
    #[derive(Default)]
    pub(crate) struct NativeAttempt {
        previous: Option<Stamp>,
        previous_enemy: Option<(u32, Stamp)>,
        enemy_was_fresh: bool,
        age: f32,
        pub expired: bool,
    }
    impl NativeAttempt {
        pub const fn new() -> Self {
            Self {
                previous: None,
                previous_enemy: None,
                enemy_was_fresh: false,
                age: 0.,
                expired: false,
            }
        }

        /// Rebase a completed loop without discarding native actor stamps.
        pub fn consume(&mut self, frames: f32) {
            self.age -= frames;
        }
        pub fn elapsed(&self) -> f32 {
            self.age
        }
        pub fn advance_pause(
            &mut self,
            ctx: GameplayContext,
            enemy: Option<Opponent>,
            timeout: u16,
        ) {
            if ctx.stopped {
                self.enemy_was_fresh = false;
                return;
            }
            if ctx.motion_kind == crate::timing::NO_MOTION_KIND
                || !ctx.motion_frame.is_finite()
                || ctx.motion_frame < 0.0
                || !ctx.motion_rate.is_finite()
                || ctx.motion_rate < 0.0
            {
                self.expired = true;
                return;
            }
            let current = (
                ctx.status,
                ctx.motion_kind,
                ctx.motion_frame,
                ctx.motion_rate,
            );
            let own = progress(self.previous, current);
            let fresh = enemy
                .and_then(|e| {
                    e.motion
                        .map(|m| (e.id, (e.status, m.kind, m.frame, m.rate)))
                })
                .filter(|(_, m)| {
                    m.1 != 0 && m.2.is_finite() && m.2 >= 0.0 && m.3.is_finite() && m.3 > 0.0
                });
            let other = fresh
                .and_then(|(id, m)| {
                    self.previous_enemy
                        .filter(|(before, _)| *before == id)
                        .map(|(_, before)| progress(Some(before), m))
                })
                .unwrap_or(0.0);
            // Prefer our clock; never add the two actors' elapsed times. After a
            // missing opponent sample or hitstop, rebase instead of charging a gap.
            // Shield/edge balancing may hold a pose at a positive reported rate;
            // only these known static states opt into opponent-clock fallback.
            let delta = if own > 0.0 {
                own
            } else if super::needs_opponent_clock(ctx.status, ctx.motion_rate)
                && self.enemy_was_fresh
            {
                other
            } else {
                0.0
            };
            self.previous = Some(current);
            if let Some(stamp) = fresh {
                self.previous_enemy = Some(stamp);
            }
            self.enemy_was_fresh = fresh.is_some();
            if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
                self.expired = true;
                return;
            }
            // Motion frames are f32; tiny division error must not turn an exact
            // one-frame wait into two. This is precision normalization, not delay.
            let nearest = delta.round();
            let delta = if (delta - nearest).abs() < 0.001 {
                nearest
            } else {
                delta
            };
            self.age += delta;
            let nearest = self.age.round();
            if (self.age - nearest).abs() < 0.001 {
                self.age = nearest;
            }
            self.expired |= self.age > f32::from(timeout);
        }
    }
    fn progress(previous: Option<Stamp>, current: Stamp) -> f32 {
        let Some(p) = previous else {
            return 0.0;
        };
        if p.3 == 0.0 && current.3 == 0.0 {
            return 0.0;
        }
        if p.0 != current.0 || p.1 != current.1 {
            return 1.0;
        }
        if current.2 < p.2 {
            return 1.0;
        } // One observed animation wrap.
        if current.2 == p.2 {
            return 0.0;
        }
        // Grounded locomotion adjusts its animation rate as stick strength/speed
        // changes. A step advanced at the NEW rate must not be divided by the old
        // rate: e.g. 0.5 / 0.25 falsely reports two game frames and aborts a walk.
        // Accept the new-rate interpretation only when it describes exactly one
        // update. Duplicate poses and genuinely skipped stable-rate frames retain
        // their existing handling; attack animation clocks are unchanged.
        let advance = current.2 - p.2;
        if (1..=8).contains(&current.0)
            && p.3 > 0.0
            && current.3 > 0.0
            && (p.3 - current.3).abs() > 0.0001
            && (advance - current.3).abs() <= 0.001 * current.3.max(1.0)
        {
            return 1.0;
        }
        let rate = if p.3 > 0.0 {
            p.3
        } else if current.3 > 0.0 {
            current.3
        } else {
            return 0.0;
        };
        (current.2 - p.2) / rate
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn held_guard_and_teeter_poses_use_opponent_frames_without_counting_duplicates_or_hitstop()
        {
            for status in [
                crate::native_states::GUARD_ON,
                crate::native_states::GUARD,
                crate::native_states::status::OTTOTTO,
                crate::native_states::status::OTTOTTO_WAIT,
            ] {
                let mut clock = NativeAttempt::default();
                let ctx = GameplayContext {
                    status,
                    motion_kind: 123,
                    motion_frame: 20.0,
                    motion_rate: 1.0,
                    ..super::super::tests::context()
                };
                let opponent = |frame| {
                    Some(Opponent {
                        motion: Some(crate::gameplay::MotionTiming {
                            kind: 456,
                            frame,
                            rate: 1.0,
                            remaining: None,
                        }),
                        ..super::super::tests::enemy(super::super::CLIFF_CLIMB, false)
                    })
                };
                clock.advance_pause(ctx, opponent(0.0), 60);
                clock.advance_pause(ctx, opponent(1.0), 60);
                assert_eq!(clock.elapsed(), 1.0);
                for _ in 0..20 {
                    clock.advance_pause(ctx, opponent(1.0), 60);
                }
                assert_eq!(clock.elapsed(), 1.0);
                clock.advance_pause(
                    GameplayContext {
                        stopped: true,
                        ..ctx
                    },
                    opponent(5.0),
                    60,
                );
                clock.advance_pause(ctx, opponent(5.0), 60);
                assert_eq!(clock.elapsed(), 1.0);
                clock.advance_pause(ctx, opponent(6.0), 60);
                assert_eq!(clock.elapsed(), 2.0);
                // Returning to a progressing pose must not add both clocks.
                clock.advance_pause(
                    GameplayContext {
                        motion_frame: 21.0,
                        ..ctx
                    },
                    opponent(7.0),
                    60,
                );
                assert_eq!(clock.elapsed(), 3.0);
                assert!(!clock.expired);
            }
            assert!(super::super::needs_opponent_clock(0, 0.0));
            assert!(!super::super::needs_opponent_clock(0, 1.0));
            assert!(!super::super::needs_opponent_clock(
                crate::native_states::status::FALL,
                1.0
            ));
        }
    }
}
pub(crate) use attempt::NativeAttempt;
#[cfg(feature = "diagnostic-logging")]
mod diagnostics {
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
                    (c.status == crate::native_states::status::ATTACK_AIR
                        || custom_status == Some(c.status))
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
}
mod file_timeline;
pub use file_timeline::Timeline as InputTimeline;
mod identity {
    //! Fighter identities only; no option lists, move recipes or schedules.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ResponseImplementation {
        Kirby,
        Luigi,
        CaptainFalcon,
        Mario,
        DonkeyKong,
        Link,
        Samus,
        Yoshi,
        Fox,
        Pikachu,
        Sonic,
        KingKRool,
        Isabelle,
        PacMan,
        LittleMac,
        DarkSamus,
        Ness,
        Jigglypuff,
        Peach,
        Daisy,
        Bowser,
        Sheik,
        Zelda,
        DrMario,
        Pichu,
        Falco,
        Marth,
        Lucina,
        YoungLink,
        Ganondorf,
        Mewtwo,
        Roy,
        Chrom,
        MrGameAndWatch,
        MetaKnight,
        Pit,
        DarkPit,
        ZeroSuitSamus,
        Wario,
        Snake,
        Ike,
        Squirtle,
        Ivysaur,
        Charizard,
        DiddyKong,
        Lucas,
        KingDedede,
        Olimar,
        Lucario,
        Rob,
        ToonLink,
        Wolf,
        Villager,
        MegaMan,
        WiiFitTrainer,
        RosalinaAndLuma,
        Greninja,
        Palutena,
        Robin,
        Shulk,
        BowserJr,
        DuckHunt,
        Ryu,
        Ken,
        Cloud,
        Corrin,
        Bayonetta,
        Inkling,
        Ridley,
        Simon,
        Richter,
        Incineroar,
        MiiBrawler,
        MiiSwordfighter,
        MiiGunner,
        IceClimbers,
        IceClimbersNana,
        Pyra,
        Mythra,
    }
    impl ResponseImplementation {
        pub fn for_kind(kind: i32) -> Option<Self> {
            match kind {
                0 => Some(Self::Mario),
                1 => Some(Self::DonkeyKong),
                2 => Some(Self::Link),
                3 => Some(Self::Samus),
                5 => Some(Self::Yoshi),
                7 => Some(Self::Fox),
                8 => Some(Self::Pikachu),
                6 => Some(Self::Kirby),
                9 => Some(Self::Luigi),
                11 => Some(Self::CaptainFalcon),
                crate::fighter_profiles::sonic::FIGHTER_KIND => Some(Self::Sonic),
                crate::fighter_profiles::krool::FIGHTER_KIND => Some(Self::KingKRool),
                crate::fighter_profiles::isabelle::FIGHTER_KIND => Some(Self::Isabelle),
                crate::fighter_profiles::pacman::FIGHTER_KIND => Some(Self::PacMan),
                crate::fighter_profiles::littlemac::FIGHTER_KIND => Some(Self::LittleMac),
                4 => Some(Self::DarkSamus),
                10 => Some(Self::Ness),
                12 => Some(Self::Jigglypuff),
                13 => Some(Self::Peach),
                14 => Some(Self::Daisy),
                15 => Some(Self::Bowser),
                16 => Some(Self::Sheik),
                17 => Some(Self::Zelda),
                18 => Some(Self::DrMario),
                19 => Some(Self::Pichu),
                20 => Some(Self::Falco),
                21 => Some(Self::Marth),
                22 => Some(Self::Lucina),
                23 => Some(Self::YoungLink),
                24 => Some(Self::Ganondorf),
                25 => Some(Self::Mewtwo),
                26 => Some(Self::Roy),
                27 => Some(Self::Chrom),
                28 => Some(Self::MrGameAndWatch),
                29 => Some(Self::MetaKnight),
                30 => Some(Self::Pit),
                31 => Some(Self::DarkPit),
                32 => Some(Self::ZeroSuitSamus),
                33 => Some(Self::Wario),
                34 => Some(Self::Snake),
                35 => Some(Self::Ike),
                36 => Some(Self::Squirtle),
                37 => Some(Self::Ivysaur),
                38 => Some(Self::Charizard),
                39 => Some(Self::DiddyKong),
                40 => Some(Self::Lucas),
                42 => Some(Self::KingDedede),
                43 => Some(Self::Olimar),
                44 => Some(Self::Lucario),
                45 => Some(Self::Rob),
                46 => Some(Self::ToonLink),
                47 => Some(Self::Wolf),
                48 => Some(Self::Villager),
                49 => Some(Self::MegaMan),
                50 => Some(Self::WiiFitTrainer),
                51 => Some(Self::RosalinaAndLuma),
                53 => Some(Self::Greninja),
                54 => Some(Self::Palutena),
                56 => Some(Self::Robin),
                57 => Some(Self::Shulk),
                58 => Some(Self::BowserJr),
                59 => Some(Self::DuckHunt),
                60 => Some(Self::Ryu),
                61 => Some(Self::Ken),
                62 => Some(Self::Cloud),
                63 => Some(Self::Corrin),
                64 => Some(Self::Bayonetta),
                65 => Some(Self::Inkling),
                66 => Some(Self::Ridley),
                67 => Some(Self::Simon),
                68 => Some(Self::Richter),
                71 => Some(Self::Incineroar),
                72 => Some(Self::MiiBrawler),
                73 => Some(Self::MiiSwordfighter),
                74 => Some(Self::MiiGunner),
                75 => Some(Self::IceClimbers),
                76 => Some(Self::IceClimbersNana),
                90 => Some(Self::Pyra),
                91 => Some(Self::Mythra),
                _ => None,
            }
        }
    }
}
mod visits;
use crate::gameplay::{ControllerOutput, GameplayContext, Opponent};
use crate::settings::AssistSettings;
#[cfg(feature = "diagnostic-logging")]
pub use diagnostics::{
    CalibrationFrame, CalibrationKind, CalibrationMove, NativeRule, RollGrabSnapshot,
};
pub use identity::ResponseImplementation;
pub use visits::LedgeVisits;

pub const DPAD_DOWN: u32 = crate::native_input::MAPPER_DPAD_DOWN;
pub const LEDGE_TRIGGER: u32 = MAPPER_L;
pub const CLIFF_CATCH_MOVE: i32 = crate::native_states::status::CLIFF_CATCH_MOVE;
pub const CLIFF_WAIT: i32 = crate::native_states::status::CLIFF_WAIT;
pub const CLIFF_ATTACK: i32 = crate::native_states::status::CLIFF_ATTACK;
pub const CLIFF_CLIMB: i32 = crate::native_states::status::CLIFF_CLIMB;
pub const CLIFF_ESCAPE: i32 = crate::native_states::status::CLIFF_ESCAPE;
pub const CLIFF_JUMP1: i32 = crate::native_states::status::CLIFF_JUMP1;
pub const CLIFF_JUMP3: i32 = crate::native_states::status::CLIFF_JUMP3;
/// Held shield/edge balancing can freeze a pose at a positive reported rate.
/// Keep a read-only clock sample of the locked opponent in those states too.
pub fn needs_opponent_clock(status: i32, motion_rate: f32) -> bool {
    motion_rate == 0.0
        || matches!(
            status,
            crate::native_states::GUARD_ON
                | crate::native_states::GUARD
                | crate::native_states::status::OTTOTTO
                | crate::native_states::status::OTTOTTO_WAIT
        )
}
/// Exclusive literal-sequence ownership; pause/menu remain available.
pub const SEQUENCE_BUTTONS: u32 = ASSIST_BUTTONS
    | crate::native_input::MAPPER_DPAD_BUTTONS
    | crate::manual_controls::LEFT_STICK_DIRECTIONS;
/// Native incapacitation ends an input tape. Keep ordinary get-up, attack,
/// shield, dodge and special recovery distinct from knockdown/damage.
/// Verified against pinned skyline-smash d2ca90d9, src/lua_const.rs:
/// CAPTURE_PULLED..DAMAGE_FALL, DOWN..DOWN_WAIT_CONTINUE, DOWN_DAMAGE/REFLECT_LR
/// and DAMAGE_FLY_REFLECT_JUMP_BOARD. THROW (0x40) is our attack, THROWN (0x46) is the victim.
pub use crate::native_states::sequence_interrupted;
fn is_hanging(status: i32) -> bool {
    (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&status)
}
/// Any native ledge state: the hang plus every option taken from it.
fn is_ledge_status(status: i32) -> bool {
    is_hanging(status) || is_ledge_option(status)
}
/// A get-up, roll, ledge attack or ledge jump.
fn is_ledge_option(status: i32) -> bool {
    option_slot(status).is_some()
}
/// Both axes of a copied native position/velocity are usable.
pub(crate) fn finite2(p: [f32; 2]) -> bool {
    p[0].is_finite() && p[1].is_finite()
}
fn finite_opponent(e: &Opponent) -> bool {
    finite2(e.position) && finite2(e.velocity)
}
/// Our own native sample is finite with a real left/right facing.
fn usable_context(ctx: GameplayContext) -> bool {
    finite2(ctx.position)
        && finite2(ctx.velocity)
        && ctx.facing.is_finite()
        && ctx.facing.abs() >= 0.5
}
/// The ledge option slot whose input recipe answers this native status.
fn option_slot(status: i32) -> Option<PresetSlot> {
    match status {
        CLIFF_CLIMB => Some(PresetSlot::Neutral),
        CLIFF_ESCAPE => Some(PresetSlot::Roll),
        CLIFF_ATTACK => Some(PresetSlot::Attack),
        CLIFF_JUMP1..=CLIFF_JUMP3 => Some(PresetSlot::Jump),
        _ => None,
    }
}
/// The response slot for an observed opponent: a ledge option it is taking (or
/// just took, as its sticky previous status shows), or a drop from the hang.
fn response_slot(enemy: Opponent, from_hang: bool) -> Option<PresetSlot> {
    option_slot(enemy.status)
        .or_else(|| option_slot(enemy.previous_status))
        .or_else(|| {
            (enemy.situation == crate::native_states::situation::AIR
                && from_hang
                && !is_hanging(enemy.status)
                && !matches!(
                    enemy.status,
                    crate::native_states::THROW
                        ..=crate::native_states::status::DAMAGE_FALL
                            | crate::native_states::status::DEAD
                            | crate::native_states::status::REBIRTH
                ))
            .then_some(PresetSlot::Drop)
        })
}

/// Acquisition can use the actual cliff anchor instead of a hanging fighter's
/// much lower root. Tracked opponents get a separate, bounded follow-up box.
pub fn in_tracking_range(
    settings: AssistSettings,
    position: [f32; 2],
    enemy: Opponent,
    tracked: bool,
) -> bool {
    if settings.distance_agnostic {
        return finite2(position) && finite2(enemy.position);
    }
    let reference = if !tracked && is_hanging(enemy.status) {
        enemy.cliff_position.unwrap_or(enemy.position)
    } else {
        enemy.position
    };
    let dx = reference[0] - position[0];
    let dy = reference[1] - position[1];
    let limits = if tracked {
        [settings.follow_range, settings.follow_vertical_range]
    } else {
        [settings.horizontal_range, settings.vertical_range]
    };
    dx.is_finite() && dy.is_finite() && dx.abs() <= limits[0] && dy.abs() <= limits[1]
}

#[derive(Default)]
pub struct LedgeTrap {
    target: Option<u32>,
    preview: Option<PresetSlot>,
    slot: Option<PresetSlot>,
    anchor: Option<[f32; 2]>,
    inward: f32,
    last_status: i32,
    observed_velocity: [f32; 2],
    file_timeline: Option<file_timeline::Timeline>,
    #[cfg(feature = "diagnostic-logging")]
    last_timeline_end: Option<&'static str>,
    complete: bool,
    visit_consumed: bool,
    captured: bool,
    last_output: ControllerOutput,
    last_tick: Option<i64>,
}
impl LedgeTrap {
    /// Arm a one-shot input preview; never synthesize opponent status or game state.
    pub fn request_preview(&mut self, slot: PresetSlot) {
        *self = Self {
            preview: Some(slot),
            ..Self::default()
        };
    }
    pub fn preview_active(&self) -> bool {
        self.preview.is_some() && !self.complete
    }
    pub fn manual_after_response(&self) -> bool {
        self.complete
    }
    pub fn file_timeline_active(&self) -> bool {
        self.file_timeline.is_some()
    }
    pub fn file_needs_ground(&self) -> bool {
        self.file_timeline
            .as_ref()
            .is_some_and(|t| t.needs_ground())
    }
    pub fn file_inputs_only(&self) -> bool {
        self.file_timeline.as_ref().is_some_and(|t| t.inputs_only())
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    fn engaged(&self) -> bool {
        self.preview.is_some() || self.target.is_some() || self.file_timeline.is_some()
    }
    pub fn needs_response_observation(&self) -> bool {
        !self.complete && self.engaged()
    }
    pub fn observation_target(&self) -> Option<u32> {
        self.target.filter(|_| self.needs_response_observation())
    }
    pub fn reserves_response(&self, enemy: Option<Opponent>) -> bool {
        !self.visit_consumed
            && !self.complete
            && (self.engaged()
                || enemy
                    .is_some_and(|e| is_ledge_status(e.status) || is_hanging(e.previous_status)))
    }
    pub fn committed_input_pending(&self) -> bool {
        self.file_timeline_active()
    }
    /// The selected walking step owns this analog input, not a movement/physics override.
    pub fn requested_walk_stick(&self) -> Option<[f32; 2]> {
        self.file_timeline
            .as_ref()
            .filter(|t| t.walking())
            .and(self.last_output.stick)
    }
    pub fn requests_attack_hold(&self) -> bool {
        self.file_timeline.as_ref().is_some_and(|t| t.attack_hold())
    }
    pub fn requested_right_stick(&self) -> Option<[f32; 2]> {
        self.file_timeline.as_ref().map(|t| t.right_stick())
    }
    pub fn owns_jump_state(&self, status: i32) -> bool {
        self.file_timeline_active() && !sequence_interrupted(status)
    }
    pub fn prepare_stance(&mut self, anchor: [f32; 2], inward: f32) {
        if finite2(anchor) && inward.is_finite() {
            self.anchor = Some(anchor);
            self.inward = inward.signum();
        }
    }
    pub fn owns_roll_navigation(&self) -> bool {
        self.file_timeline_active() && self.slot == Some(PresetSlot::Roll)
    }
    pub fn observed_target_velocity(&self) -> [f32; 2] {
        self.observed_velocity
    }
    pub fn action_name(&self) -> &'static str {
        if self.visit_consumed {
            "await_new_ledge_visit"
        } else if self.complete {
            "file_complete"
        } else if self.file_timeline.is_some() {
            "file_timeline"
        } else if self.target.is_some() {
            "watch_ledge"
        } else {
            "unarmed"
        }
    }
    pub fn sample(
        &mut self,
        settings: AssistSettings,
        tick: i64,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
    ) -> ControllerOutput {
        if !settings.enabled
            || !ctx.held
            || !ctx.gameplay
            || self.last_tick.is_some_and(|last| tick < last)
        {
            *self = Self::default();
            return ControllerOutput::default();
        }
        self.last_tick = Some(tick);
        if self.complete {
            return ControllerOutput::default();
        }
        if !usable_context(ctx) {
            return self.end_visit();
        }
        if self.file_timeline.is_some() {
            return self.continue_timeline(settings, ctx, enemy);
        }
        if let Some(slot) = self.preview {
            return self.sample_preview(settings, ctx, enemy, slot);
        }
        let Some(enemy) = enemy.filter(finite_opponent) else {
            self.target = None;
            return ControllerOutput::default();
        };
        if !in_tracking_range(settings, ctx.position, enemy, self.target == Some(enemy.id)) {
            self.target = None;
            return ControllerOutput::default();
        }
        let hang = is_hanging(enemy.status);
        let from_hang = self.target == Some(enemy.id) && is_hanging(self.last_status)
            || is_hanging(enemy.previous_status);
        let slot = response_slot(enemy, from_hang);
        if hang || slot.is_some() {
            self.target = Some(enemy.id);
            self.observed_velocity = enemy.velocity;
            if let Some(anchor) = enemy.cliff_position.filter(|p| finite2(*p)) {
                self.anchor = Some(anchor);
                if (ctx.position[0] - anchor[0]).abs() > 0.5 {
                    self.inward = (ctx.position[0] - anchor[0]).signum();
                }
            }
        }
        self.last_status = enemy.status;
        let Some(slot) = slot else {
            return ControllerOutput::default();
        };
        self.slot = Some(slot);
        self.start_timeline(settings, ctx, slot, enemy)
    }
    /// Editor preview: answer the one observed opponent with the chosen slot,
    /// whatever its native status.
    fn sample_preview(
        &mut self,
        settings: AssistSettings,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
        slot: PresetSlot,
    ) -> ControllerOutput {
        let Some(enemy) = enemy.filter(finite_opponent) else {
            return self.end_visit();
        };
        self.target = Some(enemy.id);
        self.slot = Some(slot);
        self.anchor = enemy.cliff_position.filter(|p| finite2(*p));
        if let Some(anchor) = self.anchor {
            self.inward = (ctx.position[0] - anchor[0]).signum();
        }
        self.start_timeline(settings, ctx, slot, enemy)
    }
    fn start_timeline(
        &mut self,
        settings: AssistSettings,
        ctx: GameplayContext,
        slot: PresetSlot,
        enemy: Opponent,
    ) -> ControllerOutput {
        // No file recipe means no response. There is no compiled fighter fallback.
        // An explicitly empty row is a completed no-response choice; it must
        // not reserve other input helpers for the rest of this visit.
        let Some(sequence) = settings.sequence_overrides[slot as usize] else {
            return self.end_visit();
        };
        self.file_timeline = Some(file_timeline::Timeline::new(sequence, enemy));
        self.continue_timeline(settings, ctx, Some(enemy))
    }
    /// Complete this visit without a response (or after an unusable observation).
    fn end_visit(&mut self) -> ControllerOutput {
        self.file_timeline = None;
        self.complete = true;
        self.last_output = ControllerOutput::default();
        self.last_output
    }
    fn continue_timeline(
        &mut self,
        settings: AssistSettings,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
    ) -> ControllerOutput {
        let timeline = self.file_timeline.as_mut().expect("active timeline");
        let out = timeline.sample(settings, ctx, enemy, self.anchor, self.inward);
        if timeline.finished() {
            #[cfg(feature = "diagnostic-logging")]
            {
                self.last_timeline_end = timeline.end_reason;
            }
            self.file_timeline = None;
            self.complete = true;
            self.captured = matches!(
                ctx.status,
                crate::native_states::CATCH_PULL
                    | crate::native_states::CATCH_DASH_PULL
                    | crate::native_states::status::CATCH_WAIT
                    | crate::native_states::status::CATCH_ATTACK
            );
        }
        self.last_output = out;
        out
    }
}
#[cfg(test)]
#[path = "../../tests/unit/ledge_trap.rs"]
mod tests;
