//! Shared game-frame forecasts. These calculate timing, not collision certainty.
//! Callers supply measured deadlines and geometry; unknown protection never
//! silently becomes vulnerability. Animation end is only a recovery estimate.

pub const NO_MOTION_KIND: u64 = 0;
pub const MOTION_ATTACK_AIR_F: u64 = 0xc3495ada5;
const HASH40_CRC32_POLYNOMIAL: u32 = 0xedb88320;

/// Hash40 motion identifier: CRC32 in the low word, byte length in the high word.
pub const fn motion_hash(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut crc = !0u32;
    let mut index = 0;
    while index < bytes.len() {
        crc ^= bytes[index] as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = (crc >> 1) ^ (HASH40_CRC32_POLYNOMIAL & (0u32.wrapping_sub(crc & 1)));
            bit += 1;
        }
        index += 1;
    }
    ((bytes.len() as u64) << 32) | (!crc) as u64
}

#[inline]
pub(crate) fn same_motion_event(
    previous: (u32, i32, u64, f32),
    current: (u32, i32, u64, f32),
) -> bool {
    previous.0 == current.0
        && previous.1 == current.1
        && previous.2 == current.2
        && current.3 >= previous.3
}

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
    /// Constructs a new `MotionStamp` with the given status, motion, frame, and rate.
    /// # Parameters
    /// * `status` - The status code of the motion.
    /// * `motion` - The identifier of the motion.
    /// * `frame` - The current frame of the motion.
    /// * `rate` - The playback rate of the motion.
    /// # Returns
    /// A `MotionStamp` initialized with the provided values.
    pub fn new(status: i32, motion: u64, frame: f32, rate: f32) -> Self {
        Self {
            status,
            motion,
            frame,
            rate,
        }
    }

    /// Checks if the motion stamp represents a valid and positive playback state.
    /// # Returns
    /// `true` if the frame and rate are finite and the rate is positive, `false` otherwise.
    pub fn valid(self) -> bool {
        playback_valid(self.frame, self.rate)
    }
}

fn playback_valid(frame: f32, rate: f32) -> bool {
    frame.is_finite() && frame >= 0.0 && rate.is_finite() && rate > 0.0
}

/// A usable native animation clock: a known motion with a finite, non-negative
/// frame and a finite, positive rate.
pub fn clock_known(motion: u64, frame: f32, rate: f32) -> bool {
    motion != NO_MOTION_KIND && playback_valid(frame, rate)
}

/// Represents the progress of a motion relative to its previous state.
/// This enum helps track whether a motion has just started, repeated, advanced, transitioned, rewound, or become invalid.
/// # Variants
/// * `Initial` - The motion has just started.
/// * `Duplicate` - The motion has not advanced since the last observation.
/// * `Advanced(f32)` - The motion has progressed by the given number of frames.
/// * `Transition` - The motion has changed to a different status or motion.
/// * `Rewind` - The motion has moved backward in time.
/// * `Invalid` - The motion is in an invalid state.
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

/// Computes the progress of a motion given its previous and current stamps.
/// # Parameters
/// * `previous` - The previous motion stamp, if available.
/// * `current` - The current motion stamp.
/// # Returns
/// A `MotionProgress` value representing how the motion has progressed.
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
/// `None` means the counter went backwards.
pub fn simulation_elapsed(previous: Option<u64>, current: u64) -> Option<u64> {
    previous.map_or(Some(0), |old| current.checked_sub(old))
}

/// Inclusive range of attack frames, counted from 1, whose hitbox is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveWindow {
    pub first: u16,
    pub last: u16,
}

/// How long a target stays protected and when it can act again, in game frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoveryWindow {
    /// Some(0) means vulnerable now; None means unknown.
    pub vulnerable_in: Option<f32>,
    /// Exclusive deadline: at this time the target may choose a defense.
    pub actionable_in: Option<f32>,
}
/// Represents a predicted contact with the target.
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

/// Tracks the progression of animation samples and assigns a tick count to each unique state.
/// # Fields
/// * `previous` - The previous observed animation sample, if any.
/// * `tick` - The current tick count representing the number of unique states observed.
#[derive(Default)]
pub struct AnimationClock {
    previous: Option<AnimationSample>,
    tick: i64,
}

impl AnimationClock {
    /// Constructs a new `AnimationClock` with no previous sample and a tick count of zero.
    pub const fn new() -> Self {
        Self {
            previous: None,
            tick: 0,
        }
    }
    /// Observes a new animation sample and updates the tick count if the state has changed.
    /// # Parameters
    /// * `sample` - The current animation sample to observe.
    /// * `stopped` - Indicates whether the animation has stopped.
    /// # Returns
    /// An `Option<i64>` representing the current tick count if the state has changed, or `None` if the observation is ignored.
    pub fn observe(&mut self, sample: AnimationSample, stopped: bool) -> Option<i64> {
        if stopped || !clock_known(sample.kind, sample.frame, 1.0) || !sample.auxiliary.is_finite() {
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
    fn motion_event_continuation_does_not_accept_rewinds_or_nan() {
        let previous = (7, 2, 3, 5.0);
        assert!(same_motion_event(previous, (7, 2, 3, 5.0)));
        assert!(same_motion_event(previous, (7, 2, 3, 6.0)));
        assert!(!same_motion_event(previous, (7, 2, 3, 4.0)));
        assert!(!same_motion_event(previous, (7, 2, 3, f32::NAN)));
        assert!(!same_motion_event((7, 2, 3, f32::NAN), previous));
        assert!(!same_motion_event(previous, (8, 2, 3, 6.0)));
        assert!(!same_motion_event(previous, (7, 4, 3, 6.0)));
        assert!(!same_motion_event(previous, (7, 2, 4, 6.0)));
    }

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

