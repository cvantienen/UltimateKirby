//! Common classification of observed motion progress, not an engine frame timer.
//! Callers retain their existing first-sample, transition and timeout policies.
//! No wall-clock delay, learned compensation or extra waiting is introduced.
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
