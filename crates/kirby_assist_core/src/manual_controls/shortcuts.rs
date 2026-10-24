//! Small physical-button shortcuts. They only propose ordinary controller input.
//! Native animation progress, not mapper poll count, advances each sequence.
use super::SHORT_HOP_TRIGGER;
use crate::ledge_trap::LedgeOutput;
use crate::native_input::{MAPPER_B, MAPPER_X, MAPPER_Y};

#[derive(Clone, Copy, Debug)]
pub struct ShortcutContext {
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub facing: f32,
    pub can_short_hop: bool,
    pub input_delay: f32,
    pub can_neutral_special: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hop,
    HopRelease,
    Neutral,
    Special,
    Reverse,
}

#[derive(Default)]
pub struct Shortcuts {
    previous_buttons: Option<u32>,
    phase: Option<Phase>,
    clock: Option<ShortcutContext>,
    age: f32,
    direction: f32,
    output: LedgeOutput,
    hop_requested: bool,
    hop_quarantined: bool,
}
impl Shortcuts {
    pub fn cancel_pending(&mut self) {
        self.cancel();
    } // Keep physical edge history.
    pub fn active(&self) -> bool {
        self.phase.is_some()
    }
    /// Hide only the grounded X shortcut and its held tail. New airborne X
    /// presses remain physical jumps; L3 is never reserved by this tool.
    pub fn consumed_buttons(&self) -> u32 {
        if self.hop_quarantined || matches!(self.phase, Some(Phase::Hop | Phase::HopRelease)) {
            MAPPER_X
        } else {
            0
        }
    }
    pub fn sample(
        &mut self,
        buttons: u32,
        allowed: bool,
        ctx: Option<ShortcutContext>,
    ) -> LedgeOutput {
        let pressed = buttons & !self.previous_buttons.unwrap_or(0);
        self.previous_buttons = Some(buttons);
        if buttons & MAPPER_X == 0 {
            self.hop_quarantined = false;
        }
        let Some(ctx) = ctx.filter(|c| {
            allowed
                && c.motion != 0
                && c.frame.is_finite()
                && c.frame >= 0.0
                && c.rate.is_finite()
                && c.rate > 0.0
                && c.facing.is_finite()
                && c.facing.abs() >= 0.5
        }) else {
            self.cancel();
            return self.output;
        };
        if ctx.stopped {
            if matches!(self.phase, Some(Phase::Hop | Phase::HopRelease))
                && Self::hop_bufferable(ctx)
            {
                if ctx.status == 0xa {
                    self.phase = Some(Phase::HopRelease);
                    self.output = Self::hop_release();
                }
                return self.output;
            }
            // Like a physical jump, retain a fresh grounded press through
            // native stop. No frame budget is spent while stopped.
            if self.phase.is_none() && pressed & SHORT_HOP_TRIGGER != 0 && Self::hop_bufferable(ctx)
            {
                self.phase = Some(Phase::Hop);
                self.hop_requested = true;
                self.hop_quarantined = true;
                self.clock = Some(ctx);
                self.age = 0.0;
                self.output = LedgeOutput {
                    requested: MAPPER_X,
                    owned: MAPPER_X,
                    ..LedgeOutput::default()
                };
                return self.output;
            }
            self.cancel();
            return self.output;
        }
        if ctx.situation != 0 && matches!(self.phase, Some(Phase::Hop | Phase::HopRelease)) {
            self.cancel();
            return self.output; // A fresh airborne X press remains physical.
        }
        // Ground movement/landing/shield may briefly disable the jump term.
        // Give those inputs a bounded delivery window, not an unbounded queue.
        if self.phase.is_none() {
            self.phase = if pressed & MAPPER_Y != 0 && ctx.can_neutral_special {
                Some(Phase::Neutral)
            } else if pressed & SHORT_HOP_TRIGGER != 0
                && ctx.situation == 0
                && (ctx.can_short_hop || Self::hop_bufferable(ctx))
            {
                self.hop_requested = false;
                self.hop_quarantined = true;
                Some(Phase::Hop)
            } else {
                None
            };
            // Capture opposite facing once. Physical stick input never chooses
            // or retargets the automatic B-reverse direction.
            self.direction = -ctx.facing.signum();
            self.clock = Some(ctx);
            self.age = 0.0;
        } else {
            let previous = self.clock.unwrap();
            let delta = crate::timing::motion_elapsed(
                Some((
                    previous.status,
                    previous.motion,
                    previous.frame,
                    previous.rate,
                )),
                (ctx.status, ctx.motion, ctx.frame, ctx.rate),
                1.0,
            );
            if !delta.is_finite() || !(0.0..=4.0).contains(&delta) {
                self.cancel();
                return self.output;
            }
            if delta == 0.0 {
                return self.output;
            }
            self.clock = Some(ctx);
            self.age += delta;
        }
        self.output = match self.phase {
            Some(Phase::Hop) => {
                if ctx.situation != 0
                    || (!Self::hop_bufferable(ctx) && !ctx.can_short_hop)
                    || self.age >= 8.0
                {
                    self.cancel();
                    self.output = Self::hop_release();
                    return self.output;
                }
                // Emit X for one native update, then release it even if online
                // delay postpones visible jump-squat confirmation. Repeated
                // mapper polls have delta zero and reuse the same press trace.
                if ctx.status == 0xa || self.hop_requested && self.age >= 1.0 {
                    self.phase = Some(Phase::HopRelease);
                    self.age = 0.0;
                    Self::hop_release()
                } else {
                    // A one-update jump-button tap produces a short hop without
                    // injecting shield. ZL could be mapped to shield and roll.
                    self.hop_requested = true;
                    LedgeOutput {
                        requested: MAPPER_X,
                        owned: MAPPER_X,
                        ..LedgeOutput::default()
                    }
                }
            }
            Some(Phase::HopRelease) => {
                // Holding physical X must not turn the hop into a full jump or
                // create an extra aerial jump when the input is handed back.
                if ctx.situation != 0 || ctx.status != 0xa || self.age > 8.0 {
                    self.cancel();
                }
                Self::hop_release()
            }
            Some(Phase::Neutral) => {
                if self.age >= 1.0 {
                    if self.age > 2.0 || !ctx.can_neutral_special {
                        self.cancel();
                        return self.output;
                    }
                    self.phase = Some(Phase::Special);
                    self.age = 0.0;
                    Self::special_input(MAPPER_B, [0.0, 0.0])
                } else {
                    Self::special_input(0, [0.0, 0.0])
                }
            }
            Some(Phase::Special) => {
                if self.age >= 1.0 {
                    // Do not emit a late reverse after a skipped observation.
                    if self.age > 2.0 || !Self::continuation_allowed(ctx.status) {
                        self.cancel();
                        return self.output;
                    }
                    self.phase = Some(Phase::Reverse);
                    self.age = 0.0;
                    Self::special_input(0, [self.direction, 0.0])
                } else {
                    Self::special_input(MAPPER_B, [0.0, 0.0])
                }
            }
            Some(Phase::Reverse) => {
                if self.age >= 1.0 || !Self::continuation_allowed(ctx.status) {
                    self.cancel();
                    return self.output;
                }
                Self::special_input(0, [self.direction, 0.0])
            }
            None => LedgeOutput::default(),
        };
        self.output
    }
    fn hop_bufferable(ctx: ShortcutContext) -> bool {
        // Ground locomotion, squat, landing and guard. Attacks, damage, grabs,
        // air dodges and specials cannot accumulate a delayed jump request.
        ctx.situation == 0 && matches!(ctx.status, 0..=0xa | 0x11..=0x1e)
    }
    fn hop_release() -> LedgeOutput {
        LedgeOutput {
            owned: MAPPER_X,
            ..LedgeOutput::default()
        }
    }
    fn continuation_allowed(status: i32) -> bool {
        // Common movement/attack states or fighter-specific special states.
        // Damage, captures, dodges and scene transitions cannot continue a flick.
        matches!(status, 0..=0x1d | 0x1dc..=0x3ff)
    }
    fn special_input(buttons: u32, stick: [f32; 2]) -> LedgeOutput {
        LedgeOutput {
            stick: Some(stick),
            requested: buttons,
            owned: MAPPER_B,
            ..LedgeOutput::default()
        }
    }
    fn cancel(&mut self) {
        self.phase = None;
        self.clock = None;
        self.age = 0.0;
        self.output = LedgeOutput::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(frame: f32) -> ShortcutContext {
        ShortcutContext {
            status: 0,
            situation: 0,
            motion: 1,
            frame,
            rate: 1.0,
            stopped: false,
            facing: 1.0,
            can_short_hop: true,
            input_delay: 0.0,
            can_neutral_special: true,
        }
    }
    #[test]
    fn first_x_press_is_immediate_and_survives_stop_without_full_hop_or_extra_jump() {
        let mut tool = Shortcuts::default();
        let stopped = ShortcutContext {
            stopped: true,
            ..ctx(0.0)
        };
        let press = tool.sample(MAPPER_X, true, Some(stopped));
        assert_eq!(press.requested, MAPPER_X);
        assert_eq!(press.stick, None);
        for _ in 0..100 {
            assert_eq!(tool.sample(MAPPER_X, true, Some(stopped)), press);
        }
        let squat = ShortcutContext {
            status: 0xa,
            motion: 2,
            ..stopped
        };
        assert_eq!(tool.sample(MAPPER_X, true, Some(squat)).requested, 0);
        assert_eq!(
            tool.sample(
                MAPPER_X,
                true,
                Some(ShortcutContext {
                    status: 0xb,
                    situation: 2,
                    stopped: false,
                    ..squat
                })
            )
            .requested,
            0
        );
        assert_eq!(tool.consumed_buttons(), MAPPER_X);
        tool.sample(
            0,
            true,
            Some(ShortcutContext {
                status: 0xb,
                situation: 2,
                ..ctx(1.0)
            }),
        );
        assert_eq!(
            tool.sample(
                MAPPER_X,
                true,
                Some(ShortcutContext {
                    status: 0xb,
                    situation: 2,
                    ..ctx(2.0)
                })
            )
            .owned,
            0
        );
        assert_eq!(
            tool.consumed_buttons(),
            0,
            "a fresh air jump remains physical"
        );
    }
    #[test]
    fn hop_taps_x_without_shield_and_stops_at_native_squat_without_repeating() {
        let mut tool = Shortcuts::default();
        tool.sample(0, true, Some(ctx(0.0)));
        let hop = tool.sample(SHORT_HOP_TRIGGER, true, Some(ctx(0.0)));
        assert_eq!(hop.requested, MAPPER_X);
        assert_eq!(hop.stick, None);
        assert_eq!(tool.sample(SHORT_HOP_TRIGGER, true, Some(ctx(0.0))), hop);
        let squat = ShortcutContext {
            status: 0xa,
            motion: 2,
            frame: 0.0,
            ..ctx(0.0)
        };
        assert_eq!(
            tool.sample(SHORT_HOP_TRIGGER, true, Some(squat)),
            Shortcuts::hop_release()
        );
        assert_eq!(
            tool.sample(SHORT_HOP_TRIGGER, true, Some(ctx(1.0))),
            Shortcuts::hop_release()
        );
    }
    #[test]
    fn held_x_is_one_short_hop_until_physical_release() {
        let mut tool = Shortcuts::default();
        tool.sample(0, true, Some(ctx(0.0)));
        assert_eq!(
            tool.sample(MAPPER_X, true, Some(ctx(0.0))).requested,
            MAPPER_X
        );
        let squat = ShortcutContext {
            status: 0xa,
            motion: 2,
            frame: 0.0,
            ..ctx(0.0)
        };
        for frame in 0..8 {
            let release = tool.sample(
                MAPPER_X,
                true,
                Some(ShortcutContext {
                    frame: frame as f32,
                    ..squat
                }),
            );
            assert_eq!(release.requested, 0);
            assert_eq!(release.owned, MAPPER_X);
        }
        let air = ShortcutContext {
            status: 0xb,
            situation: 2,
            motion: 3,
            frame: 0.0,
            ..ctx(0.0)
        };
        for frame in 0..20 {
            assert_eq!(
                tool.sample(
                    MAPPER_X,
                    true,
                    Some(ShortcutContext {
                        frame: frame as f32,
                        ..air
                    })
                )
                .requested,
                0
            );
            assert_eq!(tool.consumed_buttons(), MAPPER_X);
        }
        // Landing while X remains held is not a new press.
        assert_eq!(tool.sample(MAPPER_X, true, Some(ctx(0.0))).requested, 0);
        tool.sample(0, true, Some(ctx(1.0)));
        assert_eq!(tool.consumed_buttons(), 0);
        assert_eq!(
            tool.sample(MAPPER_X, true, Some(ctx(1.0))).requested,
            MAPPER_X
        );
    }
    #[test]
    fn online_delayed_jump_status_still_emits_exactly_one_native_x_update() {
        let mut tool = Shortcuts::default();
        tool.sample(0, true, Some(ctx(0.0)));
        let press = tool.sample(MAPPER_X, true, Some(ctx(0.0)));
        assert_eq!(press.requested, MAPPER_X);
        // Extra mapper samples on the same game update reuse the press.
        for _ in 0..12 {
            assert_eq!(tool.sample(MAPPER_X, true, Some(ctx(0.0))), press);
        }
        // The next native update releases X even if delayed status feedback
        // still reports standing instead of jump squat.
        let release = tool.sample(MAPPER_X, true, Some(ctx(1.0)));
        assert_eq!(release.requested, 0);
        assert_eq!(release.owned, MAPPER_X);
        for frame in 2..8 {
            assert_eq!(
                tool.sample(MAPPER_X, true, Some(ctx(frame as f32))).requested,
                0
            );
        }
    }
    #[test]
    fn reverse_primes_neutral_then_b_then_opposite_facing_and_releases() {
        for facing in [-1.0, 1.0] {
            let mut tool = Shortcuts::default();
            let c = ShortcutContext {
                facing,
                situation: 2,
                ..ctx(0.0)
            };
            tool.sample(0, true, Some(c));
            let prime = tool.sample(MAPPER_Y, true, Some(c));
            assert_eq!(prime.stick, Some([0.0, 0.0]));
            assert_eq!(prime.requested, 0);
            // Repeated polls cannot advance B onto the priming input.
            assert_eq!(tool.sample(MAPPER_Y, true, Some(c)), prime);
            let special = tool.sample(0, true, Some(ShortcutContext { frame: 1.0, ..c }));
            assert_eq!(special.requested, MAPPER_B);
            assert_eq!(special.stick, Some([0.0, 0.0]));
            let reverse = tool.sample(
                0,
                true,
                Some(ShortcutContext {
                    status: 0x1dc,
                    motion: 3,
                    frame: 0.0,
                    can_neutral_special: false,
                    ..c
                }),
            );
            assert_eq!(reverse.requested, 0);
            assert_eq!(reverse.stick, Some([-facing, 0.0]));
            assert_eq!(
                tool.sample(
                    0,
                    true,
                    Some(ShortcutContext {
                        status: 0x1dc,
                        motion: 3,
                        frame: 1.0,
                        ..c
                    })
                ),
                LedgeOutput::default()
            );
        }
    }
    #[test]
    fn reverse_direction_stays_opposite_facing_captured_on_y_press() {
        for facing in [-1.0, 1.0] {
            let mut tool = Shortcuts::default();
            let c = ShortcutContext { facing, ..ctx(0.0) };
            tool.sample(0, true, Some(c));
            tool.sample(MAPPER_Y, true, Some(c));
            let b = tool.sample(
                MAPPER_Y,
                true,
                Some(ShortcutContext {
                    facing: -facing,
                    frame: 1.0,
                    ..c
                }),
            );
            assert_eq!(b.requested, MAPPER_B);
            let reverse = tool.sample(
                MAPPER_Y,
                true,
                Some(ShortcutContext {
                    facing: -facing,
                    status: 0x1dc,
                    motion: 3,
                    frame: 0.0,
                    ..c
                }),
            );
            assert_eq!(reverse.stick, Some([-facing, 0.0]));
        }
    }
    #[test]
    fn busy_presses_do_not_queue_and_interruption_cancels_pending_b() {
        let mut tool = Shortcuts::default();
        tool.sample(0, true, Some(ctx(0.0)));
        assert_eq!(
            tool.sample(MAPPER_Y, false, Some(ctx(1.0))),
            LedgeOutput::default()
        );
        assert_eq!(
            tool.sample(MAPPER_Y, true, Some(ctx(2.0))),
            LedgeOutput::default()
        );
        tool.sample(0, true, Some(ctx(3.0)));
        tool.sample(MAPPER_Y, true, Some(ctx(4.0)));
        assert!(tool.active());
        assert_eq!(
            tool.sample(
                MAPPER_Y,
                true,
                Some(ShortcutContext {
                    stopped: true,
                    ..ctx(4.0)
                })
            ),
            LedgeOutput::default()
        );
        assert!(!tool.active());
        assert_eq!(
            tool.sample(MAPPER_Y, true, Some(ctx(5.0))),
            LedgeOutput::default()
        );
    }
    #[test]
    fn missing_or_skipped_native_frames_never_emit_a_late_reverse() {
        let mut tool = Shortcuts::default();
        tool.sample(0, true, Some(ctx(0.0)));
        tool.sample(MAPPER_Y, true, Some(ctx(0.0)));
        assert_eq!(tool.sample(0, true, Some(ctx(4.0))), LedgeOutput::default());
        assert!(!tool.active());
        assert_eq!(tool.sample(MAPPER_Y, true, None), LedgeOutput::default());
    }
}
