//! Live tech requests with mapper acknowledgment. Input-only: native collision
//! rules and lockout still decide whether the requested tech is possible.
use crate::tech_prediction::CATCHED_GANON;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TechContext {
    pub object: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub remaining_stop: u64,
    pub contact_in: Option<f32>,
    pub input_delay: f32,
    pub previous_guard: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechOutput {
    pub press: bool,
    pub owns_guard: bool,
}
#[derive(Default)]
pub struct TechInput {
    last: Option<TechContext>,
    output: TechOutput,
    waiting_for_mapper: bool,
    cooldown: f32,
    cooldown_setting: u16,
}
impl TechInput {
    pub fn last_observation(&self) -> Option<TechContext> {
        self.last
    }
    pub fn cooldown_remaining(&self) -> f32 {
        self.cooldown
    }
    /// Called only after a fresh mapped GUARD edge, never for a policy proposal.
    pub fn mapped_press(&mut self) {
        if self.waiting_for_mapper && self.output.press {
            self.cooldown = f32::from(self.cooldown_setting.max(2));
            self.waiting_for_mapper = false;
            self.output = TechOutput {
                press: false,
                owns_guard: true,
            };
        }
    }
    /// A human shield press during tumble also consumes native input timing.
    /// Track it so automation does not immediately press again into its lockout.
    pub fn mapped_human_guard(&mut self) {
        if self
            .last
            .is_some_and(|ctx| (0x49..=0x4f).contains(&ctx.status) || ctx.status == CATCHED_GANON)
        {
            self.cooldown = f32::from(self.cooldown_setting.max(2));
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
        }
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
    ) -> TechOutput {
        if !enabled {
            *self = Self::default();
            return TechOutput::default();
        }
        if self.last.is_some_and(|last| last.object != ctx.object) {
            *self = Self::default();
        }
        let animation_known = ctx.frame.is_finite()
            && ctx.frame >= 0.0
            && ctx.rate.is_finite()
            && ctx.rate > 0.0
            && ctx.motion != 0;
        let hitstop_clock_known = ctx.stopped && (1..=120).contains(&ctx.remaining_stop);
        if !ctx.input_delay.is_finite() || !animation_known && !hitstop_clock_known {
            // A missing/frozen native clock withholds input, but must not erase
            // a cooldown belonging to an already mapped shield press.
            self.last = Some(ctx);
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
            return self.output;
        }
        if let Some(last) = self.last {
            if !ctx.stopped && !last.stopped {
                let advance = if ctx.status == last.status
                    && ctx.motion == last.motion
                    && ctx.frame >= last.frame
                    && (ctx.rate - last.rate).abs() < 0.001
                {
                    ((ctx.frame - last.frame) / ctx.rate).clamp(0.0, 4.0)
                } else {
                    1.0
                };
                self.cooldown = (self.cooldown - advance).max(0.0);
            }
        }
        let accepted = matches!(ctx.status, 0x67..=0x6b);
        if accepted {
            // An observed successful tech ends our attempt; do not carry a
            // synthetic cooldown through that tech into the next collision.
            self.cooldown = 0.0;
            self.waiting_for_mapper = false;
        }
        self.last = Some(ctx);
        self.cooldown_setting = cooldown;
        // The command-grab slam needs a tighter arrival margin than ordinary
        // tumble. Re-evaluate contact rather than replaying an old prediction.
        let arrival_lead = if ctx.status == CATCHED_GANON {
            4.0
        } else {
            9.0
        };
        let due = ctx.contact_in.is_some_and(|contact| {
            contact.is_finite()
                && contact >= 0.0
                && contact - ctx.input_delay.clamp(0.0, 12.0) <= arrival_lead
        });
        if !due || accepted {
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
            return self.output;
        }
        if self.cooldown > 0.0 {
            self.waiting_for_mapper = false;
            // While the same collision remains imminent, keep physical shield
            // aliases released. Handing them back between samples could create
            // another mapped GUARD edge and restart the input lockout.
            self.output = TechOutput {
                press: false,
                owns_guard: takeover,
            };
            return self.output;
        }
        if self.waiting_for_mapper {
            return self.output;
        }
        if ctx.previous_guard {
            self.output = TechOutput {
                press: false,
                owns_guard: takeover,
            };
        } else {
            self.waiting_for_mapper = true;
            self.output = TechOutput {
                press: true,
                owns_guard: takeover,
            };
        }
        self.output
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(frame: f32) -> TechContext {
        TechContext {
            object: 1,
            status: 0x49,
            motion: 1,
            frame,
            rate: 1.0,
            stopped: false,
            remaining_stop: 0,
            contact_in: Some(3.0),
            input_delay: 0.0,
            previous_guard: false,
        }
    }
    #[test]
    fn unmapped_requests_do_not_spend_cooldown_or_disappear_on_duplicate_polls() {
        let mut input = TechInput::default();
        for _ in 0..30 {
            assert!(input.sample(true, true, 40, ctx(1.0)).press);
            assert_eq!(input.cooldown_remaining(), 0.0);
        }
        assert!(input.sample(true, true, 40, ctx(2.0)).press);
        input.mapped_press();
        assert_eq!(input.cooldown_remaining(), 40.0);
        assert!(!input.sample(true, true, 40, ctx(2.0)).press);
        assert_eq!(input.cooldown_remaining(), 40.0);
    }
    #[test]
    fn held_guard_gets_release_then_press_and_hitstop_does_not_consume_lockout() {
        let mut input = TechInput::default();
        let release = input.sample(
            true,
            true,
            40,
            TechContext {
                previous_guard: true,
                ..ctx(1.0)
            },
        );
        assert!(!release.press && release.owns_guard);
        assert!(input.sample(true, true, 40, ctx(1.0)).press);
        input.mapped_press();
        for remaining in (1..=10).rev() {
            input.sample(
                true,
                true,
                40,
                TechContext {
                    stopped: true,
                    remaining_stop: remaining,
                    ..ctx(1.0)
                },
            );
        }
        assert_eq!(input.cooldown_remaining(), 40.0);
    }
    #[test]
    fn success_clears_synthetic_cooldown_and_changed_prediction_cancels_unmapped_press() {
        let mut input = TechInput::default();
        input.sample(true, true, 40, ctx(1.0));
        input.mapped_press();
        assert!(
            !input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        status: 0x69,
                        motion: 2,
                        frame: 1.0,
                        ..ctx(1.0)
                    }
                )
                .press
        );
        assert!(input.sample(true, true, 40, ctx(2.0)).press);
        assert!(
            !input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        contact_in: None,
                        ..ctx(2.0)
                    }
                )
                .press
        );
        assert_eq!(input.cooldown_remaining(), 0.0);
    }
    #[test]
    fn delay_schedules_earlier_but_unknown_contact_and_disable_never_press() {
        let mut input = TechInput::default();
        let early = TechContext {
            contact_in: Some(14.0),
            ..ctx(1.0)
        };
        assert!(!input.sample(true, true, 40, early).press);
        assert!(
            input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        input_delay: 6.0,
                        ..early
                    }
                )
                .press
        );
        assert!(!input.sample(false, true, 40, early).press);
        assert!(
            !input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        contact_in: None,
                        ..early
                    }
                )
                .press
        );
    }
    #[test]
    fn cooldown_uses_rate_normalized_motion_progress_not_poll_count() {
        let mut input = TechInput::default();
        let first = TechContext {
            rate: 0.5,
            ..ctx(1.0)
        };
        input.sample(true, true, 40, first);
        input.mapped_press();
        for _ in 0..20 {
            input.sample(true, true, 40, first);
        }
        input.sample(
            true,
            true,
            40,
            TechContext {
                frame: 3.0,
                ..first
            },
        );
        assert_eq!(input.cooldown_remaining(), 36.0);
    }
    #[test]
    fn native_hitstop_allows_a_buffered_request_and_additive_mode_preserves_held_guard() {
        let mut input = TechInput::default();
        let stop = TechContext {
            stopped: true,
            motion: 0,
            rate: 0.0,
            remaining_stop: 2,
            ..ctx(1.0)
        };
        assert!(input.sample(true, true, 40, stop).press);
        input.mapped_press();
        assert_eq!(input.cooldown_remaining(), 40.0);
        let mut additive = TechInput::default();
        assert_eq!(
            additive.sample(
                true,
                false,
                40,
                TechContext {
                    previous_guard: true,
                    ..ctx(1.0)
                }
            ),
            TechOutput::default()
        );
    }
    #[test]
    fn shared_overlay_releases_both_triggers_before_a_fresh_tech_edge() {
        use crate::manual_controls::{ManualOverlay, TurboOutput};
        use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR};
        let shoulders = MAPPER_ZL | MAPPER_ZR | MAPPER_L | MAPPER_R;
        let human = ButtonTrace {
            held: MAPPER_ZL | MAPPER_ZR,
            previous: MAPPER_ZL | MAPPER_ZR,
            ..ButtonTrace::default()
        };
        let mut overlay = ManualOverlay::default();
        let mut input = TechInput::default();
        let release = input.sample(
            true,
            true,
            40,
            TechContext {
                previous_guard: true,
                ..ctx(1.0)
            },
        );
        assert!(release.owns_guard && !release.press);
        let mapped =
            overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
        assert_eq!(mapped.buttons.held & shoulders, 0);
        let press = input.sample(true, true, 40, ctx(1.0));
        assert!(press.press);
        let mapped = overlay.apply_controls(
            human,
            [0.0; 2],
            TurboOutput::default(),
            0,
            MAPPER_ZR,
            shoulders,
        );
        assert_eq!(mapped.buttons.held & shoulders, MAPPER_ZR);
        assert_eq!(mapped.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
    }
    #[test]
    fn unavailable_animation_data_and_human_guard_do_not_erase_lockout_tracking() {
        let mut input = TechInput::default();
        input.sample(true, true, 40, ctx(1.0));
        input.mapped_press();
        assert!(
            !input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        motion: 0,
                        rate: 0.0,
                        ..ctx(1.0)
                    }
                )
                .press
        );
        assert_eq!(input.cooldown_remaining(), 40.0);
        input.sample(true, true, 40, ctx(2.0));
        input.mapped_human_guard();
        assert_eq!(input.cooldown_remaining(), 40.0);
        assert!(!input.sample(true, true, 40, ctx(3.0)).press);
    }
}

#[cfg(test)]
mod ganon_and_release_regressions {
    use super::*;
    use crate::tech_prediction::grounded_ganon_contact_in;

    fn capture(frame: f32, delay: f32) -> TechContext {
        TechContext {
            object: 1,
            status: CATCHED_GANON,
            // This motion clock comes from the linked captor; the victim can
            // remain in one frozen capture pose throughout the sequence.
            motion: 7,
            frame,
            rate: 1.0,
            stopped: false,
            remaining_stop: 0,
            contact_in: grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e2, frame, 1.0),
            input_delay: delay,
            previous_guard: false,
        }
    }

    #[test]
    fn grounded_ganon_waits_for_tight_arrival_window_and_compensates_online_delay() {
        for delay in [0.0, 3.0, 6.0] {
            let mut input = TechInput::default();
            let due_frame = 31.0 - delay - 4.0;
            assert!(
                !input
                    .sample(true, true, 40, capture(due_frame - 1.0, delay))
                    .press
            );
            assert_eq!(input.cooldown_remaining(), 0.0);
            for _ in 0..20 {
                assert!(
                    input
                        .sample(true, true, 40, capture(due_frame, delay))
                        .press
                );
            }
            input.mapped_press();
            assert_eq!(input.cooldown_remaining(), 40.0);
            let release = input.sample(true, true, 40, capture(due_frame, delay));
            assert!(!release.press);
            assert!(release.owns_guard);
        }
    }

    #[test]
    fn cooldown_keeps_held_shield_aliases_released_until_the_collision_ends() {
        use crate::manual_controls::{ManualOverlay, TurboOutput};
        use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR};
        let shoulders = MAPPER_L | MAPPER_R | MAPPER_ZL | MAPPER_ZR;
        let human = ButtonTrace {
            held: MAPPER_ZL | MAPPER_ZR,
            previous: MAPPER_ZL | MAPPER_ZR,
            ..ButtonTrace::default()
        };
        let mut input = TechInput::default();
        let mut overlay = ManualOverlay::default();
        let context = capture(27.0, 0.0);
        input.sample(
            true,
            true,
            40,
            TechContext {
                previous_guard: true,
                ..context
            },
        );
        overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
        assert!(input.sample(true, true, 40, context).press);
        overlay.apply_controls(
            human,
            [0.0; 2],
            TurboOutput::default(),
            0,
            MAPPER_ZR,
            shoulders,
        );
        input.mapped_press();
        for frame in 28..=31 {
            let out = input.sample(true, true, 40, capture(frame as f32, 0.0));
            assert!(out.owns_guard && !out.press);
            let trace =
                overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
            assert_eq!(trace.buttons.held & shoulders, 0);
            assert_eq!(
                trace.buttons.pressed & shoulders,
                0,
                "held physical shields must not fabricate another GUARD edge"
            );
        }
        let ended = input.sample(
            true,
            true,
            40,
            TechContext {
                status: 0x67,
                contact_in: None,
                ..context
            },
        );
        assert_eq!(ended, TechOutput::default());
        assert_eq!(input.cooldown_remaining(), 0.0);
    }

    #[test]
    fn ordinary_tumble_retains_its_existing_arrival_window_and_invalid_capture_never_presses() {
        let mut input = TechInput::default();
        let ordinary = TechContext {
            status: 0x49,
            contact_in: Some(8.0),
            ..capture(20.0, 0.0)
        };
        assert!(input.sample(true, true, 40, ordinary).press);
        let mut input = TechInput::default();
        assert!(
            !input
                .sample(
                    true,
                    true,
                    40,
                    TechContext {
                        contact_in: None,
                        ..capture(27.0, 0.0)
                    }
                )
                .press
        );
        assert!(!input.sample(false, true, 40, capture(27.0, 0.0)).press);
    }
}
