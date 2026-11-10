//! Shared ledge recognition and routing. The sole option catalog and every
//! fighter-specific input schedule live in startup-loaded sequences.txt.
//! No fighter recipe, move-start delay, hitbox window or retry engine lives here.
use crate::fighter_profiles::presets::PresetSlot;
pub use crate::native_input::ASSIST_BUTTONS;
use crate::native_input::MAPPER_L;
mod attempt {
    //! Read-only native-progress clock; no fighter timing, retries or wall-clock input scheduling.
    use super::{GameplayContext, Opponent};
    /// Technical: Defines `Stamp` as a type alias.
    ///
    /// Plain English: Holds the Stamp value.
    type Stamp = (i32, u64, f32, f32);
    /// Technical: Defines the `NativeAttempt` data structure.
    ///
    /// Plain English: Holds the data for NativeAttempt.
    #[derive(Default)]
    pub(crate) struct NativeAttempt {
        /// Technical: Stores the `previous` value in `NativeAttempt` as `Option<Stamp>`.
        ///
        /// Plain English: Optionally stores the previous for NativeAttempt; it may be absent.
        previous: Option<Stamp>,
        /// Technical: Stores the `previous_enemy` value in `NativeAttempt` as `Option<(u32`.
        ///
        /// Plain English: Optionally stores the previous enemy for NativeAttempt; it may be absent.
        previous_enemy: Option<(u32, Stamp)>,
        /// Technical: Stores the `enemy_was_fresh` value in `NativeAttempt` as `bool`.
        ///
        /// Plain English: Holds the enemy was fresh value used by NativeAttempt.
        enemy_was_fresh: bool,
        /// Technical: Stores the `age` value in `NativeAttempt` as `f32`.
        ///
        /// Plain English: Holds the age value used by NativeAttempt.
        age: f32,
        /// Technical: Stores the `expired` value in `NativeAttempt` as `bool`.
        ///
        /// Plain English: Holds the expired value used by NativeAttempt.
        pub expired: bool,
    }
    impl NativeAttempt {
        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
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
        /// Plain English: Rebase a completed loop without discarding native actor stamps.
        pub fn consume(&mut self, frames: f32) {
            self.age -= frames;
        }
        /// Technical: Returns the `elapsed` result.
        ///
        /// Plain English: Gets the elapsed result.
        pub fn elapsed(&self) -> f32 {
            self.age
        }
        /// Technical: Advances the pause.
        ///
        /// Plain English: Advances the pause.
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
    /// Technical: Returns the `progress` result.
    ///
    /// Plain English: Gets the progress result.
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

        /// Technical: Regression test for `held_guard_and_teeter_poses_use_opponent_frames_without_counting_duplicates_or_hitstop`.
        ///
        /// Plain English: Checks that held guard and teeter poses use opponent frames without counting duplicates or hitstop.
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
    /// Technical: Defines the available `CalibrationKind` cases.
    ///
    /// Plain English: Lists the available CalibrationKind choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CalibrationKind {
        /// Technical: Represents the `Attack` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Attack option in CalibrationKind.
        Attack,
        /// Technical: Represents the `Special` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Special option in CalibrationKind.
        Special,
        /// Technical: Represents the `Grab` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Grab option in CalibrationKind.
        Grab,
        /// Technical: Represents the `Zair` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Zair option in CalibrationKind.
        Zair,
        /// Technical: Represents the `Shield` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Shield option in CalibrationKind.
        Shield,
        /// Technical: Represents the `Unknown` case of `CalibrationKind`.
        ///
        /// Plain English: Represents the Unknown option in CalibrationKind.
        Unknown,
    }

    /// Technical: Defines the available `NativeRule` cases.
    ///
    /// Plain English: Lists the available NativeRule choices.
    #[derive(Clone, Copy, Debug)]
    pub enum NativeRule {
        /// Technical: Represents the `Unknown` case of `NativeRule`.
        ///
        /// Plain English: Represents the Unknown option in NativeRule.
        Unknown,
        /// Technical: Represents the `Ground` case of `NativeRule`.
        ///
        /// Plain English: Represents the Ground option in NativeRule.
        Ground {
            /// Technical: Stores the `startup` value in `NativeRule` as `i32`.
            ///
            /// Plain English: Holds the startup value used by NativeRule.
            startup: i32,
            /// Technical: Stores the `attack` value in `NativeRule` as `i32`.
            ///
            /// Plain English: Holds the attack value used by NativeRule.
            attack: i32,
        },
        /// Technical: Represents the `Aerial` case of `NativeRule`.
        ///
        /// Plain English: Represents the Aerial option in NativeRule.
        Aerial {
            /// Technical: Stores the `motion` value in `NativeRule` as `u64`.
            ///
            /// Plain English: Holds the motion value used by NativeRule.
            motion: u64,
            /// Technical: Stores the `custom_status` value in `NativeRule` as `Option<i32>`.
            ///
            /// Plain English: Optionally stores the custom status for NativeRule; it may be absent.
            custom_status: Option<i32>,
        },
        /// Technical: Represents the `Statuses` case of `NativeRule`.
        ///
        /// Plain English: Represents the Statuses option in NativeRule.
        Statuses(
            /// Technical: Carries the native status IDs that activate the rule in `NativeRule`.
            ///
            /// Plain English: Holds the native status IDs that activate the rule for this `NativeRule` choice.
            &'static [i32],
        ),
        /// Technical: Represents the `Inhale` case of `NativeRule`.
        ///
        /// Plain English: Represents the Inhale option in NativeRule.
        Inhale,
    }
    impl NativeRule {
        /// Technical: Determines whether the `matches` condition holds.
        ///
        /// Plain English: Checks whether matches.
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

    /// Technical: Defines the `CalibrationMove` data structure.
    ///
    /// Plain English: Holds the data for CalibrationMove.
    #[derive(Clone, Copy, Debug)]
    pub struct CalibrationMove {
        /// Technical: Stores the `kind` value in `CalibrationMove` as `CalibrationKind`.
        ///
        /// Plain English: Holds the kind value used by CalibrationMove.
        pub kind: CalibrationKind,
        /// Technical: Stores the `native` value in `CalibrationMove` as `NativeRule`.
        ///
        /// Plain English: Holds the native value used by CalibrationMove.
        pub native: NativeRule,
        /// Technical: Stores the `first` value in `CalibrationMove` as `Option<u8>`.
        ///
        /// Plain English: Optionally stores the first for CalibrationMove; it may be absent.
        pub first: Option<u8>,
        /// Technical: Stores the `last` value in `CalibrationMove` as `Option<u8>`.
        ///
        /// Plain English: Optionally stores the last for CalibrationMove; it may be absent.
        pub last: Option<u8>,
        /// Technical: Stores the `full_hop` value in `CalibrationMove` as `Option<bool>`.
        ///
        /// Plain English: Optionally stores the full hop for CalibrationMove; it may be absent.
        pub full_hop: Option<bool>,
        /// Technical: Stores the `drift` value in `CalibrationMove` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the drift for CalibrationMove; it may be absent.
        pub drift: Option<f32>,
        /// Technical: Stores the `drift_until` value in `CalibrationMove` as `Option<u8>`.
        ///
        /// Plain English: Optionally stores the drift until for CalibrationMove; it may be absent.
        pub drift_until: Option<u8>,
        /// Technical: Stores the `fast_fall_from` value in `CalibrationMove` as `Option<u8>`.
        ///
        /// Plain English: Optionally stores the fast fall from for CalibrationMove; it may be absent.
        pub fast_fall_from: Option<u8>,
    }
    impl CalibrationMove {
        /// Technical: Returns the `unknown` result.
        ///
        /// Plain English: Gets the unknown result.
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

    /// Technical: Defines the `CalibrationFrame` data structure.
    ///
    /// Plain English: Holds the data for CalibrationFrame.
    #[derive(Clone, Copy, Debug)]
    pub struct CalibrationFrame {
        /// Technical: Stores the `target` value in `CalibrationFrame` as `u32`.
        ///
        /// Plain English: Holds the target value used by CalibrationFrame.
        pub target: u32,
        /// Technical: Stores the `slot` value in `CalibrationFrame` as `PresetSlot`.
        ///
        /// Plain English: Holds the slot value used by CalibrationFrame.
        pub slot: PresetSlot,
        /// Technical: Stores the `expected` value in `CalibrationFrame` as `CalibrationMove`.
        ///
        /// Plain English: Holds the expected value used by CalibrationFrame.
        pub expected: CalibrationMove,
        /// Diagnostic-only native activity evidence; never a gameplay gate.
        /// Plain English: Diagnostic-only native activity evidence; never a gameplay gate.
        pub native_attack_active: Option<bool>,
        /// Technical: Stores the `timeline_end` value in `CalibrationFrame` as `Option<&'static str>`.
        ///
        /// Plain English: Optionally stores the timeline end for CalibrationFrame; it may be absent.
        pub timeline_end: Option<&'static str>,
        /// Technical: Stores the `context` value in `CalibrationFrame` as `GameplayContext`.
        ///
        /// Plain English: Holds the context value used by CalibrationFrame.
        pub context: GameplayContext,
        /// Technical: Stores the `enemy` value in `CalibrationFrame` as `Option<Opponent>`.
        ///
        /// Plain English: Optionally stores the enemy for CalibrationFrame; it may be absent.
        pub enemy: Option<Opponent>,
        /// Technical: Stores the `output` value in `CalibrationFrame` as `ControllerOutput`.
        ///
        /// Plain English: Holds the output value used by CalibrationFrame.
        pub output: ControllerOutput,
        /// Technical: Stores the `phase` value in `CalibrationFrame` as `&'static str`.
        ///
        /// Plain English: Holds the phase value used by CalibrationFrame.
        pub phase: &'static str,
        /// Technical: Stores the `complete` value in `CalibrationFrame` as `bool`.
        ///
        /// Plain English: Holds the complete value used by CalibrationFrame.
        pub complete: bool,
        /// Technical: Stores the `captured` value in `CalibrationFrame` as `bool`.
        ///
        /// Plain English: Holds the captured value used by CalibrationFrame.
        pub captured: bool,
        /// Technical: Stores the `blocked` value in `CalibrationFrame` as `bool`.
        ///
        /// Plain English: Holds the blocked value used by CalibrationFrame.
        pub blocked: bool,
        /// Technical: Stores the `endpoint` value in `CalibrationFrame` as `Option<[f32; 3]>`.
        ///
        /// Plain English: Optionally stores the endpoint for CalibrationFrame; it may be absent.
        pub endpoint: Option<[f32; 3]>,
        /// Technical: Stores the `configured_range` value in `CalibrationFrame` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the configured range for CalibrationFrame; it may be absent.
        pub configured_range: Option<f32>,
        /// Technical: Stores the `lead` value in `CalibrationFrame` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the lead for CalibrationFrame; it may be absent.
        pub lead: Option<f32>,
        /// Technical: Stores the `distance_agnostic` value in `CalibrationFrame` as `bool`.
        ///
        /// Plain English: Holds the distance agnostic value used by CalibrationFrame.
        pub distance_agnostic: bool,
        /// Technical: Stores the `grab_button` value in `CalibrationFrame` as `u32`.
        ///
        /// Plain English: Holds the grab button value used by CalibrationFrame.
        pub grab_button: u32,
    }

    /// Technical: Defines the `RollGrabSnapshot` data structure.
    ///
    /// Plain English: Holds the data for RollGrabSnapshot.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct RollGrabSnapshot {
        /// Technical: Stores the `target` value in `RollGrabSnapshot` as `Option<u32>`.
        ///
        /// Plain English: Optionally stores the target for RollGrabSnapshot; it may be absent.
        pub target: Option<u32>,
        /// Technical: Stores the `phase` value in `RollGrabSnapshot` as `&'static str`.
        ///
        /// Plain English: Holds the phase value used by RollGrabSnapshot.
        pub phase: &'static str,
        /// Technical: Stores the `status` value in `RollGrabSnapshot` as `i32`.
        ///
        /// Plain English: Holds the status value used by RollGrabSnapshot.
        pub status: i32,
        /// Technical: Stores the `motion_frame` value in `RollGrabSnapshot` as `f32`.
        ///
        /// Plain English: Holds the motion frame value used by RollGrabSnapshot.
        pub motion_frame: f32,
        /// Technical: Stores the `position` value in `RollGrabSnapshot` as `[f32; 2]`.
        ///
        /// Plain English: Stores the position collection for RollGrabSnapshot.
        pub position: [f32; 2],
        /// Technical: Stores the `velocity_x` value in `RollGrabSnapshot` as `f32`.
        ///
        /// Plain English: Holds the velocity x value used by RollGrabSnapshot.
        pub velocity_x: f32,
        /// Technical: Stores the `facing` value in `RollGrabSnapshot` as `f32`.
        ///
        /// Plain English: Holds the facing value used by RollGrabSnapshot.
        pub facing: f32,
        /// Technical: Stores the `endpoint` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the endpoint for RollGrabSnapshot; it may be absent.
        pub endpoint: Option<f32>,
        /// Technical: Stores the `stand_x` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the stand x for RollGrabSnapshot; it may be absent.
        pub stand_x: Option<f32>,
        /// Technical: Stores the `usable_reach` value in `RollGrabSnapshot` as `f32`.
        ///
        /// Plain English: Holds the usable reach value used by RollGrabSnapshot.
        pub usable_reach: f32,
        /// Technical: Stores the `grab_range` value in `RollGrabSnapshot` as `f32`.
        ///
        /// Plain English: Holds the grab range value used by RollGrabSnapshot.
        pub grab_range: f32,
        /// Technical: Stores the `can_control` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the can control value used by RollGrabSnapshot.
        pub can_control: bool,
        /// Technical: Stores the `can_act` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the can act value used by RollGrabSnapshot.
        pub can_act: bool,
        /// Technical: Stores the `can_grab` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the can grab value used by RollGrabSnapshot.
        pub can_grab: bool,
        /// Technical: Stores the `stopped` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the stopped value used by RollGrabSnapshot.
        pub stopped: bool,
        /// Technical: Stores the `safe_ground` value in `RollGrabSnapshot` as `[bool; 2]`.
        ///
        /// Plain English: Stores the safe ground collection for RollGrabSnapshot.
        pub safe_ground: [bool; 2],
        /// Technical: Stores the `enemy_status` value in `RollGrabSnapshot` as `Option<i32>`.
        ///
        /// Plain English: Optionally stores the enemy status for RollGrabSnapshot; it may be absent.
        pub enemy_status: Option<i32>,
        /// Technical: Stores the `enemy_frame` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the enemy frame for RollGrabSnapshot; it may be absent.
        pub enemy_frame: Option<f32>,
        /// Technical: Stores the `enemy_position` value in `RollGrabSnapshot` as `Option<[f32; 2]>`.
        ///
        /// Plain English: Optionally stores the enemy position for RollGrabSnapshot; it may be absent.
        pub enemy_position: Option<[f32; 2]>,
        /// Technical: Stores the `vulnerable` value in `RollGrabSnapshot` as `Option<bool>`.
        ///
        /// Plain English: Optionally stores the vulnerable for RollGrabSnapshot; it may be absent.
        pub vulnerable: Option<bool>,
        /// Technical: Stores the `vulnerable_in` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the vulnerable in for RollGrabSnapshot; it may be absent.
        pub vulnerable_in: Option<f32>,
        /// Technical: Stores the `adjusted_opens_in` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the adjusted opens in for RollGrabSnapshot; it may be absent.
        pub adjusted_opens_in: Option<f32>,
        /// Technical: Stores the `actionable_in` value in `RollGrabSnapshot` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the actionable in for RollGrabSnapshot; it may be absent.
        pub actionable_in: Option<f32>,
        /// Technical: Stores the `active` value in `RollGrabSnapshot` as `[u8; 2]`.
        ///
        /// Plain English: Stores the active collection for RollGrabSnapshot.
        pub active: [u8; 2],
        /// Technical: Stores the `requested` value in `RollGrabSnapshot` as `u32`.
        ///
        /// Plain English: Holds the requested value used by RollGrabSnapshot.
        pub requested: u32,
        /// Technical: Stores the `grab_button` value in `RollGrabSnapshot` as `u32`.
        ///
        /// Plain English: Holds the grab button value used by RollGrabSnapshot.
        pub grab_button: u32,
        /// Technical: Stores the `attempts` value in `RollGrabSnapshot` as `u8`.
        ///
        /// Plain English: Holds the attempts value used by RollGrabSnapshot.
        pub attempts: u8,
        /// Technical: Stores the `animations` value in `RollGrabSnapshot` as `u8`.
        ///
        /// Plain English: Holds the animations value used by RollGrabSnapshot.
        pub animations: u8,
        /// Technical: Stores the `native_grabbing` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the native grabbing value used by RollGrabSnapshot.
        pub native_grabbing: bool,
        /// Technical: Stores the `captured` value in `RollGrabSnapshot` as `bool`.
        ///
        /// Plain English: Holds the captured value used by RollGrabSnapshot.
        pub captured: bool,
    }

    impl LedgeTrap {
        /// Technical: Returns an optional `calibration_slot` value.
        ///
        /// Plain English: Gets calibration slot if it is available.
        pub fn calibration_slot(&self) -> Option<PresetSlot> {
            self.slot
        }
        /// Technical: Returns an optional `calibration_frame` value.
        ///
        /// Plain English: Gets calibration frame if it is available.
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
        /// Technical: Returns an optional `roll_grab_diagnostic` value.
        ///
        /// Plain English: Gets roll grab diagnostic if it is available.
        pub fn roll_grab_diagnostic(&self) -> Option<RollGrabSnapshot> {
            None
        }
    }
}
mod file_timeline;
pub use file_timeline::Timeline as InputTimeline;
mod identity {
    //! Fighter identities only; no option lists, move recipes or schedules.
    /// Technical: Defines the available `ResponseImplementation` cases.
    ///
    /// Plain English: Lists the available ResponseImplementation choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ResponseImplementation {
        /// Technical: Represents the `Kirby` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Kirby option in ResponseImplementation.
        Kirby,
        /// Technical: Represents the `Luigi` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Luigi option in ResponseImplementation.
        Luigi,
        /// Technical: Represents the `CaptainFalcon` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the CaptainFalcon option in ResponseImplementation.
        CaptainFalcon,
        /// Technical: Represents the `Mario` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Mario option in ResponseImplementation.
        Mario,
        /// Technical: Represents the `DonkeyKong` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DonkeyKong option in ResponseImplementation.
        DonkeyKong,
        /// Technical: Represents the `Link` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Link option in ResponseImplementation.
        Link,
        /// Technical: Represents the `Samus` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Samus option in ResponseImplementation.
        Samus,
        /// Technical: Represents the `Yoshi` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Yoshi option in ResponseImplementation.
        Yoshi,
        /// Technical: Represents the `Fox` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Fox option in ResponseImplementation.
        Fox,
        /// Technical: Represents the `Pikachu` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Pikachu option in ResponseImplementation.
        Pikachu,
        /// Technical: Represents the `Sonic` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Sonic option in ResponseImplementation.
        Sonic,
        /// Technical: Represents the `KingKRool` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the KingKRool option in ResponseImplementation.
        KingKRool,
        /// Technical: Represents the `Isabelle` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Isabelle option in ResponseImplementation.
        Isabelle,
        /// Technical: Represents the `PacMan` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the PacMan option in ResponseImplementation.
        PacMan,
        /// Technical: Represents the `LittleMac` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the LittleMac option in ResponseImplementation.
        LittleMac,
        /// Technical: Represents the `DarkSamus` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DarkSamus option in ResponseImplementation.
        DarkSamus,
        /// Technical: Represents the `Ness` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ness option in ResponseImplementation.
        Ness,
        /// Technical: Represents the `Jigglypuff` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Jigglypuff option in ResponseImplementation.
        Jigglypuff,
        /// Technical: Represents the `Peach` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Peach option in ResponseImplementation.
        Peach,
        /// Technical: Represents the `Daisy` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Daisy option in ResponseImplementation.
        Daisy,
        /// Technical: Represents the `Bowser` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Bowser option in ResponseImplementation.
        Bowser,
        /// Technical: Represents the `Sheik` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Sheik option in ResponseImplementation.
        Sheik,
        /// Technical: Represents the `Zelda` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Zelda option in ResponseImplementation.
        Zelda,
        /// Technical: Represents the `DrMario` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DrMario option in ResponseImplementation.
        DrMario,
        /// Technical: Represents the `Pichu` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Pichu option in ResponseImplementation.
        Pichu,
        /// Technical: Represents the `Falco` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Falco option in ResponseImplementation.
        Falco,
        /// Technical: Represents the `Marth` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Marth option in ResponseImplementation.
        Marth,
        /// Technical: Represents the `Lucina` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Lucina option in ResponseImplementation.
        Lucina,
        /// Technical: Represents the `YoungLink` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the YoungLink option in ResponseImplementation.
        YoungLink,
        /// Technical: Represents the `Ganondorf` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ganondorf option in ResponseImplementation.
        Ganondorf,
        /// Technical: Represents the `Mewtwo` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Mewtwo option in ResponseImplementation.
        Mewtwo,
        /// Technical: Represents the `Roy` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Roy option in ResponseImplementation.
        Roy,
        /// Technical: Represents the `Chrom` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Chrom option in ResponseImplementation.
        Chrom,
        /// Technical: Represents the `MrGameAndWatch` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MrGameAndWatch option in ResponseImplementation.
        MrGameAndWatch,
        /// Technical: Represents the `MetaKnight` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MetaKnight option in ResponseImplementation.
        MetaKnight,
        /// Technical: Represents the `Pit` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Pit option in ResponseImplementation.
        Pit,
        /// Technical: Represents the `DarkPit` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DarkPit option in ResponseImplementation.
        DarkPit,
        /// Technical: Represents the `ZeroSuitSamus` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the ZeroSuitSamus option in ResponseImplementation.
        ZeroSuitSamus,
        /// Technical: Represents the `Wario` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Wario option in ResponseImplementation.
        Wario,
        /// Technical: Represents the `Snake` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Snake option in ResponseImplementation.
        Snake,
        /// Technical: Represents the `Ike` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ike option in ResponseImplementation.
        Ike,
        /// Technical: Represents the `Squirtle` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Squirtle option in ResponseImplementation.
        Squirtle,
        /// Technical: Represents the `Ivysaur` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ivysaur option in ResponseImplementation.
        Ivysaur,
        /// Technical: Represents the `Charizard` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Charizard option in ResponseImplementation.
        Charizard,
        /// Technical: Represents the `DiddyKong` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DiddyKong option in ResponseImplementation.
        DiddyKong,
        /// Technical: Represents the `Lucas` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Lucas option in ResponseImplementation.
        Lucas,
        /// Technical: Represents the `KingDedede` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the KingDedede option in ResponseImplementation.
        KingDedede,
        /// Technical: Represents the `Olimar` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Olimar option in ResponseImplementation.
        Olimar,
        /// Technical: Represents the `Lucario` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Lucario option in ResponseImplementation.
        Lucario,
        /// Technical: Represents the `Rob` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Rob option in ResponseImplementation.
        Rob,
        /// Technical: Represents the `ToonLink` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the ToonLink option in ResponseImplementation.
        ToonLink,
        /// Technical: Represents the `Wolf` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Wolf option in ResponseImplementation.
        Wolf,
        /// Technical: Represents the `Villager` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Villager option in ResponseImplementation.
        Villager,
        /// Technical: Represents the `MegaMan` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MegaMan option in ResponseImplementation.
        MegaMan,
        /// Technical: Represents the `WiiFitTrainer` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the WiiFitTrainer option in ResponseImplementation.
        WiiFitTrainer,
        /// Technical: Represents the `RosalinaAndLuma` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the RosalinaAndLuma option in ResponseImplementation.
        RosalinaAndLuma,
        /// Technical: Represents the `Greninja` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Greninja option in ResponseImplementation.
        Greninja,
        /// Technical: Represents the `Palutena` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Palutena option in ResponseImplementation.
        Palutena,
        /// Technical: Represents the `Robin` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Robin option in ResponseImplementation.
        Robin,
        /// Technical: Represents the `Shulk` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Shulk option in ResponseImplementation.
        Shulk,
        /// Technical: Represents the `BowserJr` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the BowserJr option in ResponseImplementation.
        BowserJr,
        /// Technical: Represents the `DuckHunt` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the DuckHunt option in ResponseImplementation.
        DuckHunt,
        /// Technical: Represents the `Ryu` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ryu option in ResponseImplementation.
        Ryu,
        /// Technical: Represents the `Ken` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ken option in ResponseImplementation.
        Ken,
        /// Technical: Represents the `Cloud` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Cloud option in ResponseImplementation.
        Cloud,
        /// Technical: Represents the `Corrin` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Corrin option in ResponseImplementation.
        Corrin,
        /// Technical: Represents the `Bayonetta` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Bayonetta option in ResponseImplementation.
        Bayonetta,
        /// Technical: Represents the `Inkling` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Inkling option in ResponseImplementation.
        Inkling,
        /// Technical: Represents the `Ridley` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Ridley option in ResponseImplementation.
        Ridley,
        /// Technical: Represents the `Simon` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Simon option in ResponseImplementation.
        Simon,
        /// Technical: Represents the `Richter` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Richter option in ResponseImplementation.
        Richter,
        /// Technical: Represents the `Incineroar` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Incineroar option in ResponseImplementation.
        Incineroar,
        /// Technical: Represents the `MiiBrawler` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MiiBrawler option in ResponseImplementation.
        MiiBrawler,
        /// Technical: Represents the `MiiSwordfighter` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MiiSwordfighter option in ResponseImplementation.
        MiiSwordfighter,
        /// Technical: Represents the `MiiGunner` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the MiiGunner option in ResponseImplementation.
        MiiGunner,
        /// Technical: Represents the `IceClimbers` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the IceClimbers option in ResponseImplementation.
        IceClimbers,
        /// Technical: Represents the `IceClimbersNana` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the IceClimbersNana option in ResponseImplementation.
        IceClimbersNana,
        /// Technical: Represents the `Pyra` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Pyra option in ResponseImplementation.
        Pyra,
        /// Technical: Represents the `Mythra` case of `ResponseImplementation`.
        ///
        /// Plain English: Represents the Mythra option in ResponseImplementation.
        Mythra,
    }
    impl ResponseImplementation {
        /// Technical: Returns an optional `for_kind` value.
        ///
        /// Plain English: Gets for kind if it is available.
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

/// Technical: Defines the `DPAD_DOWN` constant used by this module.
///
/// Plain English: Provides the DPAD DOWN constant value.
pub const DPAD_DOWN: u32 = crate::native_input::MAPPER_DPAD_DOWN;
/// Technical: Defines the `LEDGE_TRIGGER` constant used by this module.
///
/// Plain English: Provides the LEDGE TRIGGER constant value.
pub const LEDGE_TRIGGER: u32 = MAPPER_L;
/// Technical: Defines the `CLIFF_CATCH_MOVE` constant used by this module.
///
/// Plain English: Provides the CLIFF CATCH MOVE constant value.
pub const CLIFF_CATCH_MOVE: i32 = crate::native_states::status::CLIFF_CATCH_MOVE;
/// Technical: Defines the `CLIFF_WAIT` constant used by this module.
///
/// Plain English: Provides the CLIFF WAIT constant value.
pub const CLIFF_WAIT: i32 = crate::native_states::status::CLIFF_WAIT;
/// Technical: Defines the `CLIFF_ATTACK` constant used by this module.
///
/// Plain English: Provides the CLIFF ATTACK constant value.
pub const CLIFF_ATTACK: i32 = crate::native_states::status::CLIFF_ATTACK;
/// Technical: Defines the `CLIFF_CLIMB` constant used by this module.
///
/// Plain English: Provides the CLIFF CLIMB constant value.
pub const CLIFF_CLIMB: i32 = crate::native_states::status::CLIFF_CLIMB;
/// Technical: Defines the `CLIFF_ESCAPE` constant used by this module.
///
/// Plain English: Provides the CLIFF ESCAPE constant value.
pub const CLIFF_ESCAPE: i32 = crate::native_states::status::CLIFF_ESCAPE;
/// Technical: Defines the `CLIFF_JUMP1` constant used by this module.
///
/// Plain English: Provides the CLIFF JUMP1 constant value.
pub const CLIFF_JUMP1: i32 = crate::native_states::status::CLIFF_JUMP1;
/// Technical: Defines the `CLIFF_JUMP3` constant used by this module.
///
/// Plain English: Provides the CLIFF JUMP3 constant value.
pub const CLIFF_JUMP3: i32 = crate::native_states::status::CLIFF_JUMP3;
/// Held shield/edge balancing can freeze a pose at a positive reported rate.
/// Keep a read-only clock sample of the locked opponent in those states too.
/// Plain English: Held shield/edge balancing can freeze a pose at a positive reported rate. Keep a read-only clock sample of the locked opponent in those states too.
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
/// Plain English: Exclusive literal-sequence ownership; pause/menu remain available.
pub const SEQUENCE_BUTTONS: u32 = ASSIST_BUTTONS
    | crate::native_input::MAPPER_DPAD_BUTTONS
    | crate::manual_controls::LEFT_STICK_DIRECTIONS;
/// Native incapacitation ends an input tape. Keep ordinary get-up, attack,
/// shield, dodge and special recovery distinct from knockdown/damage.
/// Verified against pinned skyline-smash d2ca90d9, src/lua_const.rs:
/// CAPTURE_PULLED..DAMAGE_FALL, DOWN..DOWN_WAIT_CONTINUE, DOWN_DAMAGE/REFLECT_LR
/// and DAMAGE_FLY_REFLECT_JUMP_BOARD. THROW (0x40) is our attack, THROWN (0x46) is the victim.
pub use crate::native_states::sequence_interrupted;
/// Technical: Checks whether hanging.
///
/// Plain English: Checks whether hanging.
fn is_hanging(status: i32) -> bool {
    (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&status)
}
/// Any native ledge state: the hang plus every option taken from it.
/// Plain English: Any native ledge state: the hang plus every option taken from it.
fn is_ledge_status(status: i32) -> bool {
    is_hanging(status) || is_ledge_option(status)
}
/// A get-up, roll, ledge attack or ledge jump.
/// Plain English: A get-up, roll, ledge attack or ledge jump.
fn is_ledge_option(status: i32) -> bool {
    option_slot(status).is_some()
}
/// Both axes of a copied native position/velocity are usable.
/// Plain English: Both axes of a copied native position/velocity are usable.
pub(crate) fn finite2(p: [f32; 2]) -> bool {
    p[0].is_finite() && p[1].is_finite()
}
/// Technical: Determines whether the `finite_opponent` condition holds.
///
/// Plain English: Checks whether finite opponent.
fn finite_opponent(e: &Opponent) -> bool {
    finite2(e.position) && finite2(e.velocity)
}
/// Our own native sample is finite with a real left/right facing.
/// Plain English: Our own native sample is finite with a real left/right facing.
fn usable_context(ctx: GameplayContext) -> bool {
    finite2(ctx.position)
        && finite2(ctx.velocity)
        && ctx.facing.is_finite()
        && ctx.facing.abs() >= 0.5
}
/// The ledge option slot whose input recipe answers this native status.
/// Plain English: The ledge option slot whose input recipe answers this native status.
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
/// Plain English: The response slot for an observed opponent: a ledge option it is taking (or just took, as its sticky previous status shows), or a drop from the hang.
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
/// Plain English: Acquisition can use the actual cliff anchor instead of a hanging fighter's much lower root. Tracked opponents get a separate, bounded follow-up box.
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

/// Technical: Defines the `LedgeTrap` data structure.
///
/// Plain English: Holds the data for LedgeTrap.
#[derive(Default)]
pub struct LedgeTrap {
    /// Technical: Stores the `target` value in `LedgeTrap` as `Option<u32>`.
    ///
    /// Plain English: Optionally stores the target for LedgeTrap; it may be absent.
    target: Option<u32>,
    /// Technical: Stores the `preview` value in `LedgeTrap` as `Option<PresetSlot>`.
    ///
    /// Plain English: Optionally stores the preview for LedgeTrap; it may be absent.
    preview: Option<PresetSlot>,
    /// Technical: Stores the `slot` value in `LedgeTrap` as `Option<PresetSlot>`.
    ///
    /// Plain English: Optionally stores the slot for LedgeTrap; it may be absent.
    slot: Option<PresetSlot>,
    /// Technical: Stores the `anchor` value in `LedgeTrap` as `Option<[f32; 2]>`.
    ///
    /// Plain English: Optionally stores the anchor for LedgeTrap; it may be absent.
    anchor: Option<[f32; 2]>,
    /// Technical: Stores the `inward` value in `LedgeTrap` as `f32`.
    ///
    /// Plain English: Holds the inward value used by LedgeTrap.
    inward: f32,
    /// Technical: Stores the `last_status` value in `LedgeTrap` as `i32`.
    ///
    /// Plain English: Holds the last status value used by LedgeTrap.
    last_status: i32,
    /// Technical: Stores the `observed_velocity` value in `LedgeTrap` as `[f32; 2]`.
    ///
    /// Plain English: Stores the observed velocity collection for LedgeTrap.
    observed_velocity: [f32; 2],
    /// Technical: Stores the `file_timeline` value in `LedgeTrap` as `Option<file_timeline::Timeline>`.
    ///
    /// Plain English: Optionally stores the file timeline for LedgeTrap; it may be absent.
    file_timeline: Option<file_timeline::Timeline>,
    #[cfg(feature = "diagnostic-logging")]
    /// Technical: Stores the `last_timeline_end` value in `LedgeTrap` as `Option<&'static str>`.
    ///
    /// Plain English: Optionally stores the last timeline end for LedgeTrap; it may be absent.
    last_timeline_end: Option<&'static str>,
    /// Technical: Stores the `complete` value in `LedgeTrap` as `bool`.
    ///
    /// Plain English: Holds the complete value used by LedgeTrap.
    complete: bool,
    /// Technical: Stores the `visit_consumed` value in `LedgeTrap` as `bool`.
    ///
    /// Plain English: Holds the visit consumed value used by LedgeTrap.
    visit_consumed: bool,
    /// Technical: Stores the `captured` value in `LedgeTrap` as `bool`.
    ///
    /// Plain English: Holds the captured value used by LedgeTrap.
    captured: bool,
    /// Technical: Stores the `last_output` value in `LedgeTrap` as `ControllerOutput`.
    ///
    /// Plain English: Holds the last output value used by LedgeTrap.
    last_output: ControllerOutput,
    /// Technical: Stores the `last_tick` value in `LedgeTrap` as `Option<i64>`.
    ///
    /// Plain English: Optionally stores the last tick for LedgeTrap; it may be absent.
    last_tick: Option<i64>,
}
impl LedgeTrap {
    /// Arm a one-shot input preview; never synthesize opponent status or game state.
    /// Plain English: Arm a one-shot input preview; never synthesize opponent status or game state.
    pub fn request_preview(&mut self, slot: PresetSlot) {
        *self = Self {
            preview: Some(slot),
            ..Self::default()
        };
    }
    /// Technical: Determines whether the `preview_active` condition holds.
    ///
    /// Plain English: Checks whether preview active.
    pub fn preview_active(&self) -> bool {
        self.preview.is_some() && !self.complete
    }
    /// Technical: Determines whether the `manual_after_response` condition holds.
    ///
    /// Plain English: Checks whether manual after response.
    pub fn manual_after_response(&self) -> bool {
        self.complete
    }
    /// Technical: Determines whether the `file_timeline_active` condition holds.
    ///
    /// Plain English: Checks whether file timeline active.
    pub fn file_timeline_active(&self) -> bool {
        self.file_timeline.is_some()
    }
    /// Technical: Determines whether the `file_needs_ground` condition holds.
    ///
    /// Plain English: Checks whether file needs ground.
    pub fn file_needs_ground(&self) -> bool {
        self.file_timeline
            .as_ref()
            .is_some_and(|t| t.needs_ground())
    }
    /// Technical: Determines whether the `file_inputs_only` condition holds.
    ///
    /// Plain English: Checks whether file inputs only.
    pub fn file_inputs_only(&self) -> bool {
        self.file_timeline.as_ref().is_some_and(|t| t.inputs_only())
    }
    /// Technical: Returns the selected target observation.
    ///
    /// Plain English: Gets the chosen target.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Technical: Determines whether the `engaged` condition holds.
    ///
    /// Plain English: Checks whether engaged.
    fn engaged(&self) -> bool {
        self.preview.is_some() || self.target.is_some() || self.file_timeline.is_some()
    }
    /// Technical: Determines whether response observation.
    ///
    /// Plain English: Checks whether response observation.
    pub fn needs_response_observation(&self) -> bool {
        !self.complete && self.engaged()
    }
    /// Technical: Returns an optional `observation_target` value.
    ///
    /// Plain English: Gets observation target if it is available.
    pub fn observation_target(&self) -> Option<u32> {
        self.target.filter(|_| self.needs_response_observation())
    }
    /// Technical: Determines whether the `reserves_response` condition holds.
    ///
    /// Plain English: Checks whether reserves response.
    pub fn reserves_response(&self, enemy: Option<Opponent>) -> bool {
        !self.visit_consumed
            && !self.complete
            && (self.engaged()
                || enemy
                    .is_some_and(|e| is_ledge_status(e.status) || is_hanging(e.previous_status)))
    }
    /// Technical: Determines whether the `committed_input_pending` condition holds.
    ///
    /// Plain English: Checks whether committed input pending.
    pub fn committed_input_pending(&self) -> bool {
        self.file_timeline_active()
    }
    /// The selected walking step owns this analog input, not a movement/physics override.
    /// Plain English: The selected walking step owns this analog input, not a movement/physics override.
    pub fn requested_walk_stick(&self) -> Option<[f32; 2]> {
        self.file_timeline
            .as_ref()
            .filter(|t| t.walking())
            .and(self.last_output.stick)
    }
    /// Technical: Determines whether the `requests_attack_hold` condition holds.
    ///
    /// Plain English: Checks whether requests attack hold.
    pub fn requests_attack_hold(&self) -> bool {
        self.file_timeline.as_ref().is_some_and(|t| t.attack_hold())
    }
    /// Technical: Returns an optional `requested_right_stick` value.
    ///
    /// Plain English: Gets requested right stick if it is available.
    pub fn requested_right_stick(&self) -> Option<[f32; 2]> {
        self.file_timeline.as_ref().map(|t| t.right_stick())
    }
    /// Technical: Determines whether the `owns_jump_state` condition holds.
    ///
    /// Plain English: Checks whether owns jump state.
    pub fn owns_jump_state(&self, status: i32) -> bool {
        self.file_timeline_active() && !sequence_interrupted(status)
    }
    /// Technical: Updates the `prepare_stance` state using the supplied inputs.
    ///
    /// Plain English: Updates prepare stance.
    pub fn prepare_stance(&mut self, anchor: [f32; 2], inward: f32) {
        if finite2(anchor) && inward.is_finite() {
            self.anchor = Some(anchor);
            self.inward = inward.signum();
        }
    }
    /// Technical: Determines whether the `owns_roll_navigation` condition holds.
    ///
    /// Plain English: Checks whether owns roll navigation.
    pub fn owns_roll_navigation(&self) -> bool {
        self.file_timeline_active() && self.slot == Some(PresetSlot::Roll)
    }
    /// Technical: Returns the `observed_target_velocity` result.
    ///
    /// Plain English: Gets the observed target velocity result.
    pub fn observed_target_velocity(&self) -> [f32; 2] {
        self.observed_velocity
    }
    /// Technical: Returns the `action_name` result.
    ///
    /// Plain English: Gets the action name result.
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
    /// Technical: Updates the toggle state from the current button sample and match state.
    ///
    /// Plain English: Checks the current buttons and updates whether the toggle should activate.
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
    /// Plain English: Editor preview: answer the one observed opponent with the chosen slot, whatever its native status.
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
    /// Technical: Starts the timeline.
    ///
    /// Plain English: Starts the timeline.
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
    /// Plain English: Complete this visit without a response (or after an unusable observation).
    fn end_visit(&mut self) -> ControllerOutput {
        self.file_timeline = None;
        self.complete = true;
        self.last_output = ControllerOutput::default();
        self.last_output
    }
    /// Technical: Returns the `continue_timeline` result.
    ///
    /// Plain English: Gets the continue timeline result.
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
