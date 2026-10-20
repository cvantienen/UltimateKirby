//! Learn only clean mapped-input -> native action transitions. Animation deltas
//! divided by native motion rate are game frames, never mapper polls or wall time.
//! Frozen, looping, skipped-transition and interrupted samples are censored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Jump,
    Attack(i32),
    AirDodge,
    Grab,
}
impl Action {
    pub fn accepted(self, status: i32, motion: u64) -> bool {
        match self {
            Self::Jump => status == 0xa,
            Self::Attack(0x33) => status == 0x33 && motion == 0xc3a4e2597,
            Self::Attack(wanted) => {
                status == wanted
                    || wanted == 0x32 && status == 0x30
                    || wanted == 0x2f && status == 0x2d
            }
            Self::AirDodge => matches!(status, 0x22..=0x23),
            Self::Grab => matches!(status, 0x37 | 0x39 | 0x3b),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub object: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub ready: bool,
}
#[derive(Default)]
pub struct InputTiming {
    latest: Option<Sample>,
    pending: Option<(Action, Sample, f32, bool)>,
    delays: [f32; 8],
    count: usize,
}
impl InputTiming {
    pub fn delay(&self) -> f32 {
        // Recent upper envelope accommodates modest online jitter, bounded so a
        // bad observation cannot cause an arbitrarily early punish.
        self.delays.iter().copied().fold(0.0, f32::max)
    }
    pub fn mapped(&mut self, action: Action) {
        if let Some((_, _, _, ambiguous)) = self.pending.as_mut() {
            // A later pulse may be the one accepted. Do not attribute it to the
            // original input and teach an inflated transport delay.
            *ambiguous = true;
        } else {
            if let Some(sample) = self.latest.filter(|s| {
                !s.stopped
                    && s.ready
                    && s.motion != 0
                    && s.frame.is_finite()
                    && s.frame >= 0.0
                    && s.rate.is_finite()
                    && s.rate > 0.0
            }) {
                if !action.accepted(sample.status, sample.motion) {
                    self.pending = Some((action, sample, 0.0, false));
                }
            }
        }
    }
    pub fn observe(&mut self, now: Sample) -> Option<f32> {
        self.latest = Some(now);
        let (action, previous, elapsed, ambiguous) = self.pending?;
        if now.object != previous.object
            || !now.frame.is_finite()
            || now.frame < 0.0
            || now.motion == 0
            || !now.rate.is_finite()
            || now.rate <= 0.0
            || elapsed > 12.0
        {
            self.pending = None;
            return None;
        }
        if now.stopped || previous.stopped {
            self.pending = Some((action, now, elapsed, ambiguous));
            return None;
        }
        if action.accepted(now.status, now.motion) {
            self.pending = None;
            // Only the first native action frame identifies its start accurately.
            // Later observations include unknown time across the transition.
            if !ambiguous && now.frame <= now.rate + 0.01 {
                self.delays[self.count % 8] = elapsed.min(12.0);
                self.count = self.count.wrapping_add(1);
                return Some(elapsed);
            }
            return None;
        }
        if !now.ready
            || now.status != previous.status
            || now.motion != previous.motion
            || now.frame < previous.frame
            || (now.rate - previous.rate).abs() > 0.001
        {
            self.pending = None;
            return None;
        }
        let delta = (now.frame - previous.frame) / now.rate;
        if delta > 4.0 {
            self.pending = None;
            return None;
        }
        self.pending = Some((action, now, elapsed + delta, ambiguous));
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample(frame: f32) -> Sample {
        Sample {
            object: 1,
            status: 0,
            motion: 1,
            frame,
            rate: 0.5,
            stopped: false,
            ready: true,
        }
    }
    #[test]
    fn learns_rate_normalized_frames_and_deduplicates_mapper_polls() {
        let mut timing = InputTiming::default();
        timing.observe(sample(4.0));
        timing.mapped(Action::Jump);
        for _ in 0..20 {
            timing.observe(sample(4.0));
        }
        timing.observe(sample(5.5));
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                motion: 2,
                frame: 0.5,
                ..sample(0.0)
            }),
            Some(3.0)
        );
        assert_eq!(timing.delay(), 3.0);
    }
    #[test]
    fn hitstop_does_not_add_delay_and_damage_or_late_acceptance_does_not_train() {
        let mut timing = InputTiming::default();
        timing.observe(sample(4.0));
        timing.mapped(Action::Jump);
        for _ in 0..20 {
            timing.observe(Sample {
                stopped: true,
                ..sample(4.0)
            });
        }
        timing.observe(sample(4.0));
        timing.observe(sample(4.5));
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                motion: 2,
                frame: 0.5,
                ..sample(0.0)
            }),
            Some(1.0)
        );
        timing.observe(sample(4.0));
        timing.mapped(Action::Jump);
        timing.observe(Sample {
            status: 0x47,
            ready: false,
            ..sample(5.0)
        });
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                ..sample(0.0)
            }),
            None
        );
        timing.observe(sample(4.0));
        timing.mapped(Action::Jump);
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                motion: 2,
                ..sample(4.0)
            }),
            None
        );
        assert_eq!(timing.delay(), 1.0);
    }
    #[test]
    fn retry_edges_and_looping_motion_are_not_clean_latency_samples() {
        let mut timing = InputTiming::default();
        timing.observe(sample(4.0));
        timing.mapped(Action::Jump);
        timing.observe(sample(5.0));
        timing.mapped(Action::Jump);
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                motion: 2,
                frame: 0.5,
                ..sample(0.0)
            }),
            None
        );
        assert_eq!(timing.delay(), 0.0);
        timing.observe(sample(9.0));
        timing.mapped(Action::Jump);
        timing.observe(sample(0.0));
        assert_eq!(
            timing.observe(Sample {
                status: 0xa,
                motion: 2,
                frame: 0.5,
                ..sample(0.0)
            }),
            None
        );
        assert_eq!(timing.delay(), 0.0);
    }
    #[test]
    fn forward_smash_startup_confirms_the_requested_smash_without_waiting_for_its_hitbox() {
        assert!(Action::Attack(0x2f).accepted(0x2d, 1));
        assert!(Action::Attack(0x2f).accepted(0x2f, 1));
        assert!(!Action::Attack(0x2f).accepted(0x2a, 1));
        assert!(!Action::Attack(0x2f).accepted(0x30, 1));
    }
}
