//! Read-only native-progress clock; no fighter timing, retries or wall-clock input scheduling.
use super::{LedgeContext, Opponent};
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
    pub fn advance_pause(&mut self, ctx: LedgeContext, enemy: Option<Opponent>, timeout: u16) {
        if ctx.stopped {
            self.enemy_was_fresh = false;
            return;
        }
        if ctx.motion_kind == 0
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
        } else if super::needs_opponent_clock(ctx.status, ctx.motion_rate) && self.enemy_was_fresh {
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
    fn held_guard_and_teeter_poses_use_opponent_frames_without_counting_duplicates_or_hitstop() {
        for status in [0x1b, 0x1c, 0x7e, 0x7f] {
            let mut clock = NativeAttempt::default();
            let ctx = LedgeContext {
                status,
                motion_kind: 123,
                motion_frame: 20.0,
                motion_rate: 1.0,
                ..super::super::tests::context()
            };
            let opponent = |frame| {
                Some(Opponent {
                    motion: Some(super::super::MotionTiming {
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
                LedgeContext {
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
                LedgeContext {
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
        assert!(!super::super::needs_opponent_clock(0xe, 1.0));
    }
}
