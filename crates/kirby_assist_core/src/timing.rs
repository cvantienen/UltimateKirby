//! Shared game-frame forecasts. These calculate timing, not collision certainty.
//! Callers supply measured deadlines and geometry; unknown protection never
//! silently becomes vulnerability. Animation end is only a recovery estimate.

pub const NO_MOTION_KIND: u64 = 0;
pub const MOTION_ATTACK_AIR_F: u64 = 0xc3495ada5;
pub const MOTION_ATTACK_AIR_LW: u64 = 0xd40042152;
const HASH40_CRC32_POLYNOMIAL: u32 = 0xedb88320;

/// Common classification of observed motion progress, not an engine frame timer.
/// Callers retain their existing first-sample, transition and timeout policies.
/// No wall-clock delay, learned compensation or extra waiting is introduced.
#[derive(Clone, Copy, Debug)]
pub struct MotionStamp {
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
}

impl MotionStamp {
    pub fn new(status: i32, motion: u64, frame: f32, rate: f32) -> Self {
        Self {
            status,
            motion,
            frame,
            rate,
        }
    }

    pub fn valid(self) -> bool {
        self.frame.is_finite() && self.frame >= 0.0 && self.rate.is_finite() && self.rate > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionProgress {
    Initial,
    Duplicate,
    Advanced(f32),
    Transition,
    Rewind,
    Invalid,
}

impl MotionProgress {
    /// Keep each caller's startup/transition policy explicit. Rewinds and invalid
    /// clocks require a caller decision; they never silently count as progress.
    pub fn elapsed(self, initial: f32, transition: f32) -> Option<f32> {
        match self {
            Self::Initial => Some(initial),
            Self::Duplicate => Some(0.0),
            Self::Advanced(n) => Some(n),
            Self::Transition => Some(transition),
            Self::Rewind | Self::Invalid => None,
        }
    }
}

pub fn motion_progress(previous: Option<MotionStamp>, current: MotionStamp) -> MotionProgress {
    if !current.valid() {
        return MotionProgress::Invalid;
    }
    let Some(previous) = previous else {
        return MotionProgress::Initial;
    };
    if !previous.valid() {
        return MotionProgress::Invalid;
    }
    if previous.status != current.status || previous.motion != current.motion {
        return MotionProgress::Transition;
    }
    let frames = (current.frame - previous.frame) / previous.rate;
    if !frames.is_finite() {
        MotionProgress::Invalid
    } else if frames < 0.0 {
        MotionProgress::Rewind
    } else if frames == 0.0 {
        MotionProgress::Duplicate
    } else {
        MotionProgress::Advanced(frames)
    }
}

/// Adapter for existing sequence histories. NaN makes an invalid/rewound clock
/// follow the caller's existing bounded-clock cancellation path.
pub fn motion_elapsed(
    previous: Option<(i32, u64, f32, f32)>,
    current: (i32, u64, f32, f32),
    initial: f32,
) -> f32 {
    let stamp = |(status, motion, frame, rate)| MotionStamp::new(status, motion, frame, rate);
    motion_progress(previous.map(stamp), stamp(current))
        .elapsed(initial, 1.0)
        .unwrap_or(f32::NAN)
}

/// For adapters that supply actual simulation-frame numbers, not animation
/// samples. Never mix this counter with mapper ticks or elapsed milliseconds.
pub fn simulation_elapsed(previous: Option<u64>, current: u64) -> Option<u64> {
    previous.map_or(Some(0), |old| current.checked_sub(old))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveWindow {
    pub first: u16,
    pub last: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoveryWindow {
    /// Some(0) means vulnerable now; None means unknown.
    pub vulnerable_in: Option<f32>,
    /// Exclusive deadline: at this time the target may choose a defense.
    pub actionable_in: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PredictedContact {
    pub in_frames: f32,
    /// None means that an inescapable recovery window was not established.
    pub recovery_margin: Option<f32>,
}

impl ActiveWindow {
    /// Earliest modeled contact after input delay, before recovery expires.
    /// `in_reach` must predict geometry at the SAME time as vulnerability.
    pub fn first_contact(
        self,
        input_delay: f32,
        recovery: RecoveryWindow,
        mut in_reach: impl FnMut(f32) -> bool,
    ) -> Option<PredictedContact> {
        let vulnerable = recovery.vulnerable_in?;
        if self.first == 0
            || self.first > self.last
            || self.last > 120
            || !input_delay.is_finite()
            || !(0.0..=120.0).contains(&input_delay)
            || !vulnerable.is_finite()
            || !(0.0..=120.0).contains(&vulnerable)
            || recovery
                .actionable_in
                .is_some_and(|end| !end.is_finite() || end <= 0.0)
        {
            return None;
        }
        (self.first..=self.last).find_map(|frame| {
            let lead = f32::from(frame) + input_delay;
            let margin = recovery.actionable_in.map(|end| end - lead);
            (lead >= vulnerable && margin.is_none_or(|left| left > 0.0) && in_reach(lead))
                .then_some(PredictedContact {
                    in_frames: lead,
                    recovery_margin: margin,
                })
        })
    }
}

/// Deduplicate native animation progress for input schedulers. This is NOT a
/// global simulation clock: missing/frozen animation data cannot prove a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationSample {
    pub object: u32,
    pub status: i32,
    pub kind: u64,
    pub frame: f32,
    /// A native advancing timer for frozen poses, e.g. remaining clatter time.
    pub auxiliary: f32,
}

#[derive(Default)]
pub struct AnimationClock {
    previous: Option<AnimationSample>,
    tick: i64,
}

impl AnimationClock {
    pub const fn new() -> Self {
        Self {
            previous: None,
            tick: 0,
        }
    }
    pub fn observe(&mut self, sample: AnimationSample, stopped: bool) -> Option<i64> {
        if stopped
            || sample.kind == NO_MOTION_KIND
            || !sample.frame.is_finite()
            || sample.frame < 0.0
            || !sample.auxiliary.is_finite()
        {
            return None;
        }
        let stamp = |s: AnimationSample| MotionStamp::new(s.status, s.kind, s.frame, 1.0);
        let progress = motion_progress(self.previous.map(stamp), stamp(sample));
        let changed = progress != MotionProgress::Duplicate
            || self
                .previous
                .is_none_or(|old| old.object != sample.object || old.auxiliary != sample.auxiliary);
        if changed {
            self.tick = self.tick.wrapping_add(1);
            self.previous = Some(sample);
        }
        Some(self.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_uses_same_frame_for_geometry_protection_delay_and_recovery() {
        let active = ActiveWindow { first: 6, last: 12 };
        let recovery = RecoveryWindow {
            vulnerable_in: Some(9.0),
            actionable_in: Some(11.0),
        };
        assert_eq!(
            active.first_contact(1.0, recovery, |f| f >= 10.0),
            Some(PredictedContact {
                in_frames: 10.0,
                recovery_margin: Some(1.0)
            })
        );
        assert_eq!(active.first_contact(1.0, recovery, |f| f >= 11.0), None);
        assert_eq!(
            active.first_contact(
                0.0,
                RecoveryWindow {
                    vulnerable_in: None,
                    ..recovery
                },
                |_| true
            ),
            None
        );
        for invalid in [f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(active.first_contact(invalid, recovery, |_| true), None);
            assert_eq!(
                active.first_contact(
                    0.0,
                    RecoveryWindow {
                        vulnerable_in: Some(invalid),
                        ..recovery
                    },
                    |_| true
                ),
                None
            );
        }
    }

    #[test]
    fn clock_reuses_polls_freezes_in_stop_and_accepts_loops_and_identity_changes() {
        let mut clock = AnimationClock::default();
        let sample = AnimationSample {
            object: 1,
            status: crate::native_states::status::WAIT,
            kind: 1,
            frame: 5.0,
            auxiliary: 0.0,
        };
        let first = clock.observe(sample, false);
        for _ in 0..20 {
            assert_eq!(clock.observe(sample, false), first);
        }
        assert_eq!(clock.observe(sample, true), None);
        assert_eq!(clock.observe(sample, false), first);
        assert_ne!(
            clock.observe(
                AnimationSample {
                    frame: 0.0,
                    ..sample
                },
                false
            ),
            first
        );
        let loop_tick = clock.observe(sample, false);
        assert_ne!(
            clock.observe(
                AnimationSample {
                    object: 2,
                    ..sample
                },
                false
            ),
            loop_tick
        );
        assert_eq!(
            clock.observe(
                AnimationSample {
                    frame: f32::NAN,
                    ..sample
                },
                false
            ),
            None
        );
    }
    #[test]
    fn native_timer_advances_mash_in_a_frozen_pose_without_counting_mapper_polls() {
        let mut clock = AnimationClock::default();
        let sample = AnimationSample {
            object: 1,
            status: crate::native_states::status::WALK,
            kind: 1,
            frame: 0.0,
            auxiliary: 30.0,
        };
        let first = clock.observe(sample, false);
        assert_eq!(clock.observe(sample, false), first);
        assert_ne!(
            clock.observe(
                AnimationSample {
                    auxiliary: 29.0,
                    ..sample
                },
                false
            ),
            first
        );
        assert_eq!(
            clock.observe(
                AnimationSample {
                    auxiliary: f32::NAN,
                    ..sample
                },
                false
            ),
            None
        );
    }
}

/// Hash40 motion identifier, used by ledge-policy regression fixtures.
pub fn motion_hash(name: &str) -> u64 {
    let mut crc = !0u32;
    for byte in name.bytes() {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (HASH40_CRC32_POLYNOMIAL & (0u32.wrapping_sub(crc & 1)));
        }
    }
    ((name.len() as u64) << 32) | u64::from(!crc)
}
