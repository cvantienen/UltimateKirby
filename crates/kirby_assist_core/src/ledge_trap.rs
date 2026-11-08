//! Shared ledge recognition and routing. The sole option catalog and every
//! fighter-specific input schedule live in startup-loaded sequences.txt.
//! No fighter recipe, move-start delay, hitbox window or retry engine lives here.
use crate::fighter_profiles::presets::PresetSlot;
pub use crate::native_input::ASSIST_BUTTONS;
use crate::native_input::MAPPER_L;
mod attempt;
pub(crate) use attempt::NativeAttempt;
#[cfg(feature = "diagnostic-logging")]
mod diagnostics;
mod file_timeline;
pub use file_timeline::Timeline as InputTimeline;
mod identity;
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
pub fn sequence_interrupted(status: i32) -> bool {
    crate::native_states::sequence_interrupted(status)
}
fn is_hanging(status: i32) -> bool {
    (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&status)
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
        return position
            .iter()
            .chain(enemy.position.iter())
            .all(|value| value.is_finite());
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

// Shared grab readiness is still used by independent helpers, not by raw
// sequence inputs. These native term IDs are not move-start timing values.
pub fn grab_transition_term(status: i32) -> i32 {
    match status {
        crate::native_states::status::DASH | crate::native_states::status::RUN => {
            crate::native_states::transition_terms::CONT_CATCH_DASH
        }
        crate::native_states::status::TURN_DASH | crate::native_states::status::TURN_RUN => {
            crate::native_states::transition_terms::CONT_CATCH_TURN
        }
        _ => crate::native_states::transition_terms::CONT_CATCH,
    }
}
pub fn grab_input_ready(status: i32, mut enabled: impl FnMut(i32) -> bool) -> bool {
    let preferred = grab_transition_term(status);
    enabled(preferred)
        || matches!(status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV)
            && [
                crate::native_states::transition_terms::CONT_CATCH,
                crate::native_states::transition_terms::CONT_CATCH_DASH,
                crate::native_states::transition_terms::CONT_CATCH_TURN,
            ]
            .into_iter()
            .any(|term| term != preferred && enabled(term))
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
    pub fn manual_after_grab(&self) -> bool {
        self.complete && self.captured
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
    pub fn needs_response_observation(&self) -> bool {
        !self.complete
            && (self.preview.is_some() || self.target.is_some() || self.file_timeline.is_some())
    }
    pub fn observation_target(&self) -> Option<u32> {
        self.target.filter(|_| self.needs_response_observation())
    }
    pub fn reserves_response(&self, enemy: Option<Opponent>) -> bool {
        !self.visit_consumed
            && !self.complete
            && (self.needs_response_observation()
                || enemy.is_some_and(|e| {
                    (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&e.status)
                        || is_hanging(e.previous_status)
                }))
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
        if anchor.iter().all(|v| v.is_finite()) && inward.is_finite() {
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
        if !ctx
            .position
            .iter()
            .chain(ctx.velocity.iter())
            .all(|v| v.is_finite())
            || !ctx.facing.is_finite()
            || ctx.facing.abs() < 0.5
        {
            self.file_timeline = None;
            self.complete = true;
            self.last_output = ControllerOutput::default();
            return self.last_output;
        }
        if self.file_timeline.is_some() {
            return self.continue_timeline(settings, ctx, enemy);
        }
        if let Some(slot) = self.preview {
            let Some(enemy) = enemy.filter(|e| {
                e.position
                    .iter()
                    .chain(e.velocity.iter())
                    .all(|v| v.is_finite())
            }) else {
                self.complete = true;
                return ControllerOutput::default();
            };
            self.target = Some(enemy.id);
            self.slot = Some(slot);
            self.anchor = enemy
                .cliff_position
                .filter(|p| p.iter().all(|v| v.is_finite()));
            if let Some(anchor) = self.anchor {
                self.inward = (ctx.position[0] - anchor[0]).signum();
            }
            let Some(sequence) = settings.sequence_overrides[slot as usize] else {
                self.complete = true;
                return ControllerOutput::default();
            };
            self.file_timeline = Some(file_timeline::Timeline::new(sequence, enemy));
            return self.continue_timeline(settings, ctx, Some(enemy));
        }
        let Some(enemy) = enemy.filter(|e| {
            e.position
                .iter()
                .chain(e.velocity.iter())
                .all(|v| v.is_finite())
        }) else {
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
        let slot = match enemy.status {
            CLIFF_CLIMB => Some(PresetSlot::Neutral),
            CLIFF_ESCAPE => Some(PresetSlot::Roll),
            CLIFF_ATTACK => Some(PresetSlot::Attack),
            CLIFF_JUMP1..=CLIFF_JUMP3 => Some(PresetSlot::Jump),
            _ if enemy.previous_status == CLIFF_CLIMB => Some(PresetSlot::Neutral),
            _ if enemy.previous_status == CLIFF_ESCAPE => Some(PresetSlot::Roll),
            _ if enemy.previous_status == CLIFF_ATTACK => Some(PresetSlot::Attack),
            _ if (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&enemy.previous_status) => {
                Some(PresetSlot::Jump)
            }
            _ if enemy.situation == crate::native_states::situation::AIR
                && from_hang
                && !hang
                && !matches!(
                    enemy.status,
                    crate::native_states::THROW
                        ..=crate::native_states::status::DAMAGE_FALL
                            | crate::native_states::status::DEAD
                            | crate::native_states::status::REBIRTH
                ) =>
            {
                Some(PresetSlot::Drop)
            }
            _ => None,
        };
        if hang || slot.is_some() {
            self.target = Some(enemy.id);
            self.observed_velocity = enemy.velocity;
            if let Some(anchor) = enemy
                .cliff_position
                .filter(|p| p.iter().all(|v| v.is_finite()))
            {
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
        // No file recipe means no response. There is no compiled fighter fallback.
        let Some(sequence) = settings.sequence_overrides[slot as usize] else {
            // An explicitly empty row is a completed no-response choice; it
            // must not reserve other input helpers for the rest of this visit.
            self.complete = true;
            self.last_output = ControllerOutput::default();
            return self.last_output;
        };
        self.file_timeline = Some(file_timeline::Timeline::new(sequence, enemy));
        self.continue_timeline(settings, ctx, Some(enemy))
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
#[path = "../tests/unit/ledge_trap/tests.rs"]
mod tests;
