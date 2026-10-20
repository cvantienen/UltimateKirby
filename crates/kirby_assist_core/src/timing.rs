//! Shared game-frame forecasts. These calculate timing, not collision certainty.
//! Callers supply measured deadlines and geometry; unknown protection never
//! silently becomes vulnerability. Animation end is only a recovery estimate.

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
    pub fn observe(&mut self, sample: AnimationSample, stopped: bool) -> Option<i64> {
        if stopped
            || sample.kind == 0
            || !sample.frame.is_finite()
            || sample.frame < 0.0
            || !sample.auxiliary.is_finite()
        {
            return None;
        }
        if self.previous != Some(sample) {
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
            status: 0,
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
            status: 1,
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
