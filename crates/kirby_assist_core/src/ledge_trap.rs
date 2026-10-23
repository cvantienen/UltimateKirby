//! Reactive ledge assistance while the live trigger is held. Native observations, not assumed
//! controller-poll frame counts, gate anti-air and shield/grab transitions.
//! Geometry gates are conservative tunable limits, not an exact hitbox model.
//!
//! Read `sample` first: it handles release/reset and duplicate observations.
//! `evaluate` tracks the target, observes accepted actions, and dispatches the
//! selected response through fighters/ to its input helpers.
//!
//! The children implement methods on the SAME `LedgeTrap`, so moving between
//! files does not create another session or reset its history. In Rust an
//! `impl` block can be split across modules. `pub(super)` lets the parent call
//! a child method without exposing it as part of the public library API.
//! Data types are in types.rs; calculations are in prediction.rs; Kirby's
//! move data comes from fighter_profiles. Existing tests are in tests.rs.
use crate::native_input::{
    MAPPER_A, MAPPER_B, MAPPER_L, MAPPER_R, MAPPER_X, MAPPER_Y, MAPPER_ZL, MAPPER_ZR,
};

// The parent owns visit history and dispatch. Children separate the data,
// calculations and response recipes without creating separate policy state.
mod aerial;
mod attempt;
#[cfg(feature = "diagnostic-logging")]
mod diagnostics;
#[cfg(feature = "diagnostic-logging")]
pub use diagnostics::{
    CalibrationFrame, CalibrationKind, CalibrationMove, NativeRule, RollGrabSnapshot,
};
pub(crate) mod fighters;
mod grounded;
mod prediction;
mod roll;
mod shield;
mod types;

pub use fighters::ResponseImplementation;

// Re-exports preserve existing imports in the plugin, other tools and tests.
use prediction::{
    finite, ground_attack_ready, jump_contact, predicted_offset, roll_stopping_distance,
    AttackWindow,
};
pub use types::{
    frames_until, in_tracking_range, recovery_frames, AirPhysics, GetupAttack, LedgeContext,
    LedgeDropAction, LedgeOutput, LedgeSettings, MotionTiming, Opponent,
};

pub const DPAD_DOWN: u32 = 1 << 2;
pub const LEDGE_TRIGGER: u32 = MAPPER_L; // L always holds ledge autopilot.
pub const CLIFF_CATCH_MOVE: i32 = 0x75;
pub const CLIFF_WAIT: i32 = 0x77;
pub const CLIFF_ATTACK: i32 = 0x78;
pub const CLIFF_CLIMB: i32 = 0x79;
pub const CLIFF_ESCAPE: i32 = 0x7a;
pub const CLIFF_JUMP1: i32 = 0x7b;
pub const CLIFF_JUMP3: i32 = 0x7d;

// Status IDs are names, not an ordered progression through a ledge visit.
// Idle (0) is numerically below CLIFF_WAIT too! A <= check would mistakenly
// treat a finished normal get-up as hanging and never send its punish.
fn is_hanging(status: i32) -> bool {
    (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&status)
}
// Reserve conflicting physical attacks/jumps/shield while a target is tracked.
// Plus/Minus are never owned. Right-stick directions and stick clicks are
// reserved too, so a physical C-stick attack cannot replace the selected option.
pub const LEDGE_BUTTONS: u32 = MAPPER_A
    | MAPPER_B
    | MAPPER_X
    | MAPPER_Y
    | MAPPER_L
    | MAPPER_R
    | MAPPER_ZL
    | MAPPER_ZR
    | (3 << 16)
    | (0xf << 24);
// Native status IDs used below. Naming them keeps the decision rules readable.
const GUARD_ON: i32 = 0x1b;
const GUARD: i32 = 0x1c;
const GUARD_OFF: i32 = 0x1d;
const GUARD_DAMAGE: i32 = 0x1e;
const DOWN_TILT: i32 = 0x2c;
const FORWARD_TILT: i32 = 0x2a;
const DOWN_SMASH: i32 = 0x32;
// Verified FIGHTER_STATUS_KIND_ATTACK_AIR in pinned skyline-smash lua_const.rs.
const ATTACK_AIR: i32 = 0x36;
// Hash40("attack_air_n"); status ATTACK_AIR alone does not identify an aerial.
const NEUTRAL_AIR_MOTION: u64 = 0xc3a4e2597;
const JUMP_SQUAT: i32 = 0xa;
const JUMP_SQUAT_FRAMES: u8 = 3;
// Kirby/Yoshi nair nudges end at this native aerial animation frame.
const NAIR_NUDGE_END_FRAME: u8 = 4;
const GRAB_START: i32 = 0x37;
const DASH_GRAB_START: i32 = 0x39;
const TURN_GRAB_START: i32 = 0x3b;
const GRAB_PULL: i32 = 0x38;
const DASH_GRAB_PULL: i32 = 0x3a;
const GRAB_HOLD: i32 = 0x3c;
const GRAB_PUMMEL: i32 = 0x3d;

// Follow timeout advances on non-stopped evaluations. Identical observations
// reuse output without spending another attempt. Retries need a release sample,
// not an arbitrary extra timer; animation acceptance and budgets bound them.
const FOLLOW_UPDATES: u16 = 48;
const MAX_ATTEMPTS: u8 = 6;

/// Native readiness term for the grab selected by the current movement state.
pub fn grab_transition_term(status: i32) -> i32 {
    match status {
        0x3 | 0x4 => 0x1e000070, // CONT_CATCH_DASH
        0x7 | 0x8 => 0x1e000071, // CONT_CATCH_TURN
        _ => 0x1e00006f,         // CONT_CATCH
    }
}

/// R is an ordinary grab input; the native locomotion status chooses its catch
/// variant. Brake/turn transitions may expose a different catch term from the
/// one predicted by the previous status. Do not miss a legal input on that edge.
pub fn grab_input_ready(status: i32, mut enabled: impl FnMut(i32) -> bool) -> bool {
    let preferred = grab_transition_term(status);
    enabled(preferred)
        || matches!(status, 0..=9 | 0x11..=0x15)
            && [0x1e00006f, 0x1e000070, 0x1e000071]
                .into_iter()
                .any(|term| term != preferred && enabled(term))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// Plan = which response we are following; Pending below = which action input
// we attempted. Keeping them separate lets us retry a rejected input while
// preserving the opponent option that caused it. These are state, not commands.
enum OptionPlan {
    #[default]
    Watch,
    NeutralAir,
    Roll,
    DropShield,
    Shield {
        blocked: bool,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Pending {
    #[default]
    None,
    DownTilt,
    ForwardTilt,
    DownSmash,
    NeutralAir,
    Grab,
    DownSpecial,
    NeutralSpecial,
    UpSpecial,
    Bomb,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct RollIntercept {
    endpoint: f32,
    stand_x: f32,
    facing: f32,
}

/// Memory for one held-trigger sequence. No native game pointers are stored here.
#[derive(Default)]
pub struct LedgeTrap {
    fighter_move: Option<fighters::one_shot::OneShot>,
    // Visit identity and stance: anchor is the stage ledge, inward points from
    // it onto the stage. finished ends attempts after hit/capture evidence.
    target: Option<u32>,
    anchor: Option<[f32; 2]>,
    inward: f32,
    finished: bool,
    grab_handed_back: bool,
    special_handed_back: bool,
    aerial_handed_back: bool,
    drop_dair: Option<fighters::kirby::DropDair>,
    samus_zair: Option<fighters::samus::Zair>,
    variant_attempt: attempt::NativeAttempt,
    // A directional sample precedes A so the game sees a tilt setup. Preparation
    // is cleared by movement or a plan change; it isn't proof an attack started.
    grounded_tilt_prepared: bool,
    climb_seen: bool,
    plan: OptionPlan,
    pending: Pending,
    last_status: i32,
    outside_updates: u16,
    // Requested pulses and observed animations have separate budgets: a request
    // can be ignored, and an accepted animation can still whiff. Both are bounded.
    attempts: u8,
    grab_buffer_sent: bool,
    grab_ready_seen: bool,
    grab_in_progress: bool,
    grab_delivery: Option<attempt::GrabDelivery>,
    grab_animations: u8,
    roll_phase: &'static str,
    #[cfg(feature = "diagnostic-logging")]
    roll_grab_selected: bool,
    #[cfg(feature = "diagnostic-logging")]
    diagnostic_move: Option<CalibrationMove>,
    roll_intercept: Option<RollIntercept>,
    roll_dash_origin: Option<f32>,
    roll_positioned: bool,
    // Once braking starts, only walking corrections may finish this approach.
    roll_grab_braking: bool,
    roll_turn_attempts: u8,
    movement_preparing: bool,
    movement_attempts: u8,
    jump_committed: bool,
    jump_full: bool,
    jump_air_seen: bool,
    jump_attempts: u8,
    jump_floor_y: Option<f32>,
    neutral_nair_drift: f32,
    tilt_in_progress: bool,
    tilt_animations: u8,
    // Copy snapshots only. Same-animation root travel supplements kinetic speed
    // because ledge animations can move a fighter without matching kinetic speed.
    previous_enemy: Option<Opponent>,
    observed_velocity: [f32; 2],
    /// Once shield is dropped to pursue pushback, never raise it again while
    /// waiting for guard-off/repositioning. Reset on a new option/ledge visit.
    shield_pursuing: bool,
    shield_nair: bool,
    // Empty slots during startup are not an ended attack. Require a witnessed
    // active -> empty transition before offering a whiff punish without a block.
    getup_hitboxes_seen: bool,
    getup_hitboxes_ended: bool,
    last_tick: Option<i64>,
    // Previous proposal is needed for release gaps and stick preparation. The
    // cached sample also includes both fighters' clocks to distinguish a
    // repeated poll from meaningful progress before spending another attempt.
    last_output: LedgeOutput,
    last_sample: Option<(LedgeSettings, LedgeContext, Option<Opponent>)>,
}
impl LedgeTrap {
    /// A successful assisted grab ends this held-trigger session's autopilot.
    pub fn manual_after_grab(&self) -> bool {
        self.grab_handed_back
    }
    /// Captures, specials and completed fighter aerial attempts return manual control.
    pub fn manual_after_response(&self) -> bool {
        self.grab_handed_back || self.special_handed_back || self.aerial_handed_back
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn drop_dair_active(&self) -> bool {
        self.drop_dair.is_some()
    }
    pub fn fighter_aerial_active(&self) -> bool {
        self.drop_dair.is_some()
            || self.samus_zair.is_some()
            || self
                .fighter_move
                .as_ref()
                .is_some_and(|action| action.aerial())
    }
    pub fn samus_zair_active(&self) -> bool {
        self.samus_zair.is_some()
    }
    pub fn attack_cancel_right_stick(&self) -> Option<[f32; 2]> {
        self.fighter_move
            .as_ref()
            .and_then(|action| action.right_stick())
    }
    pub fn mapped_attack_cancel(&mut self, buttons: u32, x: i8) {
        if let Some(action) = self.fighter_move.as_mut() {
            action.mapped_cstick(buttons, x);
        }
    }
    /// Air ownership is restricted to a hop requested by this policy. Damage,
    /// escape, special and unrelated airborne states return physical control.
    pub fn owns_jump_state(&self, status: i32) -> bool {
        self.fighter_move
            .as_ref()
            .is_some_and(|action| action.owns_jump_state(status))
            || self
                .drop_dair
                .as_ref()
                .is_some_and(|dair| dair.owns_jump_state(status))
            || self
                .samus_zair
                .as_ref()
                .is_some_and(|zair| zair.owns_jump_state(status))
            || self.jump_committed
                && matches!(
                    status,
                    JUMP_SQUAT | 0xb | 0xe | 0xf | 0x16..=0x18 | ATTACK_AIR
                )
    }
    /// Seed a stage-derived stance before an opponent is acquired.
    pub fn prepare_stance(&mut self, anchor: [f32; 2], inward: f32) {
        self.anchor = Some(anchor);
        self.inward = inward;
    }
    /// Preparation movement must never override a recognized roll response,
    /// even while native readiness briefly prevents it from owning buttons.
    pub fn owns_roll_navigation(&self) -> bool {
        self.target.is_some() && self.plan == OptionPlan::Roll && !self.finished
    }
    /// Locked roll endpoint, standing position and facing for diagnostics.
    pub fn roll_interception(&self) -> Option<[f32; 3]> {
        self.roll_intercept
            .map(|spot| [spot.endpoint, spot.stand_x, spot.facing])
    }
    /// Read-only diagnostic snapshot of the velocity used for target prediction.
    pub fn observed_target_velocity(&self) -> [f32; 2] {
        self.observed_velocity
    }
    pub fn action_name(&self) -> &'static str {
        if self.samus_zair.is_some() {
            return "samus_short_hop_zair";
        }
        if self.aerial_handed_back {
            return "manual_after_drop_dair";
        }
        if self.drop_dair.is_some() {
            return "short_hop_dair_ledge_drop";
        }
        if self.special_handed_back {
            return "manual_after_stone";
        }
        if self.grab_handed_back {
            return "manual_after_grab";
        }
        if self.finished {
            if let Some([x, _]) = self.last_output.stick {
                if x != 0.0 {
                    return "return_to_ledge";
                }
            }
            return "finished";
        }
        match self.plan {
            OptionPlan::Watch if self.jump_committed => "normal_getup_nair",
            OptionPlan::Watch => "watch_ledge",
            OptionPlan::NeutralAir if self.movement_preparing => "prepare_neutral_air",
            OptionPlan::NeutralAir => "short_hop_neutral_air",
            OptionPlan::Roll if !self.roll_phase.is_empty() => self.roll_phase,
            OptionPlan::Roll => "roll_move_then_down_smash",
            OptionPlan::DropShield => "shield_ledge_drop_aerial",
            OptionPlan::Shield { blocked: false } => "shield_wait_for_block",
            OptionPlan::Shield { blocked: true } if self.shield_nair => {
                "short_hop_nair_after_block"
            }
            OptionPlan::Shield { blocked: true } if self.shield_pursuing => {
                "close_distance_then_grab"
            }
            OptionPlan::Shield { blocked: true } => "shield_grab_after_stun",
        }
    }
    fn abandon_target(&mut self) {
        *self = Self {
            anchor: self.anchor,
            inward: self.inward,
            last_status: self.last_status,
            ..Self::default()
        };
    }
    // Only reset attempt/preparation history when the plan CHANGES. Recreating
    // it every poll would forget release gaps and shield pursuit progress.
    fn set_plan(&mut self, plan: OptionPlan) {
        if self.plan != plan {
            self.plan = plan;
            self.grounded_tilt_prepared = false;
            self.movement_preparing = false;
            self.movement_attempts = 0;
            self.jump_committed = false;
            self.jump_air_seen = false;
            self.jump_attempts = 0;
            self.jump_floor_y = None;
            self.neutral_nair_drift = 0.0;
            self.tilt_in_progress = false;
            self.tilt_animations = 0;
            self.grab_in_progress = false;
            self.grab_delivery = None;
            self.grab_animations = 0;
            self.attempts = 0;
            self.pending = Pending::None;
            self.variant_attempt = attempt::NativeAttempt::default();
            self.drop_dair = None;
            self.samus_zair = None;
            self.roll_phase = "";
            #[cfg(feature = "diagnostic-logging")]
            {
                self.roll_grab_selected = false;
                self.diagnostic_move = None;
            }
            self.roll_intercept = None;
            self.roll_dash_origin = None;
            self.roll_positioned = false;
            self.roll_grab_braking = false;
            self.roll_turn_attempts = 0;
            self.grab_buffer_sent = false;
            self.grab_ready_seen = false;
            self.shield_pursuing = false;
            self.shield_nair = false;
            self.getup_hitboxes_seen = false;
            self.getup_hitboxes_ended = false;
        }
    }
    pub fn requested_attack_status(&self) -> Option<i32> {
        if let Some(action) = self.fighter_move.as_ref() {
            return action.attack_status();
        }
        match self.pending {
            Pending::DownTilt => Some(DOWN_TILT),
            Pending::ForwardTilt => Some(FORWARD_TILT),
            Pending::DownSmash => Some(DOWN_SMASH),
            Pending::NeutralAir => Some(ATTACK_AIR),
            Pending::DownSpecial => Some(0x1df),
            Pending::NeutralSpecial => Some(0x1dc),
            Pending::UpSpecial => Some(0x1de),
            Pending::Bomb => Some(0x1e6), // Samus SPECIAL_GROUND_LW.
            _ => None,
        }
    }
    /// Fresh pulses have a release gap and bounded retries. A pulse is not an
    /// accepted action: confirmation comes from a later native state snapshot.
    fn try_tilt(&mut self, pending: Pending) -> bool {
        if self.last_output.requested & MAPPER_A != 0
            || self.attempts >= MAX_ATTEMPTS
            || self.tilt_animations >= MAX_ATTEMPTS
        {
            return false;
        }
        self.attempts += 1;
        self.pending = pending;
        true
    }
    /// Grab buffering gets one early pulse. Native readiness opens a fresh
    /// request budget, so early pulses cannot use up all actual punish attempts.
    /// A held previous R must be released; there is no arbitrary 3-poll delay.
    fn try_grab(&mut self, settings: LedgeSettings, buffering: bool) -> bool {
        if self.grab_animations >= MAX_ATTEMPTS {
            return false;
        }
        if !buffering && !self.grab_ready_seen {
            self.grab_ready_seen = true;
            self.attempts = 0;
        }
        if buffering && self.grab_buffer_sent {
            return false;
        }
        if self.last_output.requested & settings.grab_button != 0 || self.attempts >= MAX_ATTEMPTS {
            return false;
        }
        if buffering {
            self.grab_buffer_sent = true;
        }
        self.attempts += 1;
        self.pending = Pending::Grab;
        true
    }
    // Public entry point: pass current facts and receive a value to overlay.
    // `&mut self` allows updating this held-trigger session; contexts/settings
    // are Copy snapshots, so the policy cannot mutate the game's real fighter.
    pub fn sample(
        &mut self,
        settings: LedgeSettings,
        tick: i64,
        context: LedgeContext,
        opponent: Option<Opponent>,
    ) -> LedgeOutput {
        if !settings.enabled
            || settings.implementation.is_none()
            || !context.held
            || !context.gameplay
        {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
        }
        if self.manual_after_response() {
            return LedgeOutput::default();
        }
        let sample = (settings, context, opponent);
        let native_clock_known =
            context.motion_kind != 0 || opponent.is_some_and(|enemy| enemy.motion.is_some());
        if (self.last_tick == Some(tick) || native_clock_known) && self.last_sample == Some(sample)
        {
            self.last_tick = Some(tick);
            return self.last_output;
        }
        let effective = if settings.manual_spacing || settings.distance_agnostic {
            LedgeSettings {
                manual_spacing: true,
                auto_position: false,
                ..settings
            }
        } else {
            settings
        };
        self.movement_preparing = false;
        let mut out = self.evaluate(effective, context, opponent);
        if !self.manual_after_response() && (settings.manual_spacing || settings.distance_agnostic)
        {
            let owns_action = out.requested != 0
                || self.tilt_in_progress
                || self.grab_in_progress
                || self.movement_preparing
                || self.drop_dair.is_some()
                || self.samus_zair.is_some()
                || self.fighter_move.is_some()
                || self.jump_committed;
            if !owns_action {
                // Waiting never owns the player's stick. One button-release
                // update follows a pulse, then fresh physical buttons hand back.
                out.stick = None;
                out.owned = if self.last_output.requested != 0 && context.can_control {
                    LEDGE_BUTTONS
                } else {
                    0
                };
            }
        }
        self.last_tick = Some(tick);
        self.last_output = out;
        self.last_sample = Some(sample);
        out
    }
    fn evaluate(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        opponent: Option<Opponent>,
    ) -> LedgeOutput {
        // Changed opponent facts may arrive more often than our native clock
        // advances online. They must not spend the bounded follow-up lifetime.
        let native_progress = !ctx.stopped
            && self.last_sample.as_ref().is_none_or(|(_, before, _)| {
                ctx.motion_kind == 0
                    || before.motion_kind == 0
                    || ctx.status != before.status
                    || ctx.motion_kind != before.motion_kind
                    || ctx.motion_frame != before.motion_frame
            });
        if !ctx
            .position
            .iter()
            .chain(ctx.velocity.iter())
            .all(|v| v.is_finite())
            || !ctx.facing.is_finite()
            || ctx.facing.abs() < 0.5
        {
            self.set_plan(OptionPlan::Watch);
            self.target = None;
            return LedgeOutput::default();
        }
        // Reserve the selected trigger while armed even before a target is found.
        // Actual button/stick takeover is added only after tracking and native
        // eligibility checks below. None stick means leave physical movement.
        let mut out = LedgeOutput {
            consume_trigger: true,
            ..LedgeOutput::default()
        };
        // Confirm before target lookup: a hit can launch the opponent outside
        // the tracking box on this very update. Losing proximity must not lose
        // the confirmation or prevent Kirby returning to his ledge stance.
        self.observe_action(ctx);
        if self.manual_after_response() {
            return LedgeOutput::default();
        }
        if self.continue_roll_grab_request(ctx, &mut out) {
            return out;
        }
        // Finish a short paired B request even if target tracking disappears.
        // Native acceptance above hands back immediately, before any more B.
        if self.continue_special_request(ctx, &mut out) {
            return out;
        }
        // New fighter variants are one ordinary move, with a fixed native
        // acceptance gate and timeout. They never chase or start a follow-up.
        if self.fighter_move.is_some() {
            self.continue_fighter_move(ctx, &mut out);
            return out;
        }
        // The spike attempt is committed once, then follows only our native
        // jump/attack state. Opponent tracking loss cannot turn it into nair.
        if self.drop_dair.is_some() {
            self.continue_drop_dair(ctx, &mut out);
            return out;
        }
        if self.samus_zair.is_some() {
            self.continue_samus_zair(ctx, &mut out);
            return out;
        }
        // All committed hops finish from Kirby's own state. Delayed online
        // snapshots, target loss and changed opponent options cannot drop A.
        if self.jump_committed && !self.finished {
            self.continue_committed_nair(ctx, &mut out);
            return out;
        }
        if opponent.is_some_and(|enemy| {
            !enemy
                .position
                .iter()
                .chain(enemy.velocity.iter())
                .all(|v| v.is_finite())
        }) {
            self.abandon_target();
            return out;
        }
        let Some(mut enemy) = opponent else {
            if self.finished {
                self.complete_punish(settings, ctx, &mut out);
                return out;
            }
            self.abandon_target();
            return out;
        };
        // 1. Acquire/follow one opponent; reset the visit on a new target/regrab.
        let dx = enemy.position[0] - ctx.position[0];
        let dy = enemy.position[1] - ctx.position[1];
        let nearby =
            in_tracking_range(settings, ctx.position, enemy, self.target == Some(enemy.id))
                && dx.is_finite()
                && dy.is_finite()
                && ctx.facing.is_finite();
        let on_ledge = (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&enemy.status);
        let recent_option = (CLIFF_ATTACK..=CLIFF_JUMP3).contains(&enemy.previous_status);
        let recent_drop = settings.drop_defense
            && enemy.situation == 2
            && is_hanging(enemy.previous_status)
            && self.last_status != enemy.status;
        if nearby
            && (on_ledge || recent_option || recent_drop)
            && (self.target != Some(enemy.id)
                || (is_hanging(enemy.status) && !is_hanging(self.last_status))
                || (on_ledge && !(CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&self.last_status)))
        {
            // Reset the entire visit, preserving no pending attack from a
            // different opponent or an earlier regrab.
            *self = Self {
                target: Some(enemy.id),
                anchor: self.anchor,
                inward: self.inward,
                ..Self::default()
            };
        }
        if native_progress {
            self.outside_updates = if on_ledge {
                0
            } else {
                self.outside_updates.saturating_add(1)
            };
        }
        let tracked = nearby
            && self.target == Some(enemy.id)
            && (on_ledge || self.outside_updates <= FOLLOW_UPDATES);
        let dropped = settings.drop_defense
            && enemy.situation == 2
            && !on_ledge
            && !recent_option
            && (is_hanging(self.last_status) || is_hanging(enemy.previous_status));
        self.last_status = enemy.status;
        if !tracked {
            if self.finished {
                self.complete_punish(settings, ctx, &mut out);
                return out;
            }
            self.abandon_target();
            return out;
        }

        // Refresh the ledge anchor while hanging, then retain it when the
        // opponent moves onstage. Following the opponent root as a stance anchor
        // would move the desired standing spot throughout its get-up animation.
        if is_hanging(enemy.status) && ctx.situation == 0 {
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
        self.climb_seen |= enemy.status == CLIFF_CLIMB || enemy.previous_status == CLIFF_CLIMB;
        enemy.velocity = self.target_velocity(enemy, ctx.stopped);

        // 2. Select the opponent-option response. observe_action above already
        // checked whether our previous request actually started or connected.
        self.choose_response(ctx, enemy, dropped);
        if self.finished {
            self.complete_punish(settings, ctx, &mut out);
            return out;
        }
        // Hitstop freezes decisions, not the existing shield hold. Returning an
        // unowned neutral output here used to release ZR on block impact. Only
        // preserve shield in native guard states: damage/air actions stay free.
        if ctx.stopped {
            if (settings.responses.attack_getup && matches!(self.plan, OptionPlan::Shield { .. })
                || matches!(self.plan, OptionPlan::DropShield))
                && !self.shield_pursuing
                && !self.finished
                && ctx.situation == 0
                && matches!(ctx.status, GUARD_ON | GUARD | GUARD_DAMAGE)
            {
                out.owned = LEDGE_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                out.requested = MAPPER_ZR;
            }
            return out;
        }
        if !ctx.can_control {
            // New one-shot variants must not issue a delayed punish after an
            // unrelated damage/air/special state interrupts their request.
            if matches!(
                self.pending,
                Pending::DownSpecial | Pending::NeutralSpecial | Pending::UpSpecial | Pending::Bomb
            ) || self.plan == OptionPlan::DropShield
                && settings.drop_action == LedgeDropAction::DownTiltShield
            {
                self.variant_attempt.expired = true;
            }
            if self.jump_committed && !self.owns_jump_state(ctx.status) {
                self.jump_committed = false;
                self.jump_air_seen = false;
                if self.pending == Pending::NeutralAir {
                    self.pending = Pending::None;
                }
                self.tilt_in_progress = false;
            }
            self.grounded_tilt_prepared = false;
            return out;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if self.grab_in_progress || self.tilt_in_progress {
            return out;
        }
        self.movement_preparing = false;
        // Our profile chooses executable fighter logic; each saved menu choice
        // supplies only that response's already-validated configuration.
        if let Some(implementation) = settings.implementation {
            implementation.respond(self, settings, ctx, enemy, &mut out);
        }
        out
    }

    fn continue_drop_dair(&mut self, ctx: LedgeContext, out: &mut LedgeOutput) {
        let Some(dair) = self.drop_dair.as_mut() else {
            return;
        };
        *out = dair.sample(ctx);
        if dair.finished() {
            self.drop_dair = None;
            self.aerial_handed_back = true;
        }
    }

    fn continue_special_request(&mut self, ctx: LedgeContext, out: &mut LedgeOutput) -> bool {
        if !matches!(
            self.pending,
            Pending::DownSpecial | Pending::NeutralSpecial | Pending::UpSpecial | Pending::Bomb
        ) || ctx.situation != 0
            || (!ctx.can_act
                && !ctx.stopped
                && !(self.pending == Pending::UpSpecial && matches!(ctx.status, GUARD_ON | GUARD)))
            || !(ctx.can_control || ctx.stopped)
            || !self.variant_attempt.holding_special(ctx)
        {
            return false;
        }
        self.movement_preparing = true;
        if !self.variant_attempt.advance(ctx, false, 30) {
            *out = self.last_output;
            return true;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if self.pending == Pending::UpSpecial {
            out.requested = MAPPER_ZR; // Release B between retries, keep shielding.
        }
        if self.variant_attempt.holding_special(ctx) {
            out.requested = MAPPER_B
                | if self.pending == Pending::UpSpecial {
                    MAPPER_ZR
                } else {
                    0
                };
            out.stick = Some(match self.pending {
                Pending::DownSpecial | Pending::Bomb => [0.0, -1.0],
                Pending::UpSpecial => [0.0, 1.0],
                _ => [0.0, 0.0],
            });
        }
        true
    }

    /// Recover stance after a successful attack. Captures have already handed
    /// all controls back and never reach this positioning path.
    fn complete_punish(&self, settings: LedgeSettings, ctx: LedgeContext, out: &mut LedgeOutput) {
        if !ctx.can_control || ctx.stopped || ctx.situation != 0 {
            return;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if ctx.can_act {
            self.position_at_ledge(settings, ctx, out);
        }
    }
    /// Observe acceptance, captures, and whiff recovery before choosing inputs.
    fn observe_action(&mut self, ctx: LedgeContext) {
        // Three separate milestones: requested button -> observed action
        // animation -> actual hit/capture. Only the last marks this visit done.
        // Recovery after a whiff reopens attempts within the animation limit.
        let status = ctx.status;
        // Native Stone/Inhale startup confirms the requested special. Release
        // ownership immediately so the player controls exiting/capture follow-ups.
        self.special_handed_back |= match self.pending {
            Pending::DownSpecial => matches!(status, 0x1df | 0x1ff | 0x200),
            Pending::NeutralSpecial => matches!(status, 0x1dc | 0x1e1..=0x1f2),
            Pending::UpSpecial => status == 0x1de,
            Pending::Bomb => matches!(status, 0x1df | 0x1e6 | 0x1e7),
            _ => false,
        };
        let tilting = match self.pending {
            Pending::DownTilt => status == DOWN_TILT,
            Pending::ForwardTilt => status == FORWARD_TILT,
            Pending::DownSmash => matches!(status, 0x30..=DOWN_SMASH),
            Pending::NeutralAir => status == ATTACK_AIR && ctx.motion_kind == NEUTRAL_AIR_MOTION,
            _ => false,
        };
        // A requested input is only an attempt. Native status confirms which
        // animation started; a grab additionally needs a captured opponent.
        self.finished |= match self.pending {
            Pending::DownSmash => status == DOWN_SMASH && ctx.hit_confirmed,
            Pending::DownTilt | Pending::ForwardTilt | Pending::NeutralAir => {
                tilting && ctx.hit_confirmed
            }
            Pending::Grab => matches!(status, GRAB_PULL | DASH_GRAB_PULL | GRAB_HOLD | GRAB_PUMMEL),
            Pending::None
            | Pending::DownSpecial
            | Pending::NeutralSpecial
            | Pending::UpSpecial
            | Pending::Bomb => false,
        };
        if self.finished && self.pending == Pending::Grab {
            self.grab_handed_back = true;
        }
        if tilting && !self.tilt_in_progress {
            self.tilt_animations += 1;
        }
        if self.tilt_in_progress && !tilting && !self.finished {
            self.attempts = 0;
            self.pending = Pending::None;
        }
        self.tilt_in_progress = tilting;
        let grabbing = self.pending == Pending::Grab
            && matches!(status, GRAB_START | DASH_GRAB_START | TURN_GRAB_START);
        if grabbing {
            self.grab_delivery = None; // Native acceptance ends synthetic R.
        }
        if grabbing && !self.grab_in_progress {
            self.grab_animations += 1;
        }
        if self.grab_in_progress && !grabbing && !self.finished {
            self.attempts = 0;
            self.pending = Pending::None;
            if matches!(self.plan, OptionPlan::Shield { .. }) {
                // A recovered whiff is actionable movement again: retry R
                // directly instead of paying for another shield startup.
                self.shield_pursuing = true;
            }
        }
        self.grab_in_progress = grabbing;
    }

    /// An observed ledge option selects a response; shield needs block evidence.
    fn choose_response(&mut self, ctx: LedgeContext, enemy: Opponent, dropped: bool) {
        // Recover a transition missed between input samples for ALL options,
        // not only jump. Current ledge state always wins over older history.
        let option = if (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&enemy.status) {
            enemy.status
        } else {
            enemy.previous_status
        };
        if !self.finished {
            if option == CLIFF_ATTACK {
                if !matches!(self.plan, OptionPlan::Shield { .. }) {
                    self.set_plan(OptionPlan::Shield { blocked: false });
                }
            } else if option == CLIFF_ESCAPE {
                self.set_plan(OptionPlan::Roll);
            } else if option == CLIFF_CLIMB {
                self.set_plan(OptionPlan::Watch);
            } else if (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&enemy.status)
                || enemy.situation == 2 && (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&option)
            {
                // Remember the option even if Kirby is still recovering. The
                // native previous status also catches a short missed transition.
                self.set_plan(OptionPlan::NeutralAir);
            } else if dropped {
                self.set_plan(OptionPlan::DropShield);
            }
        }
        if matches!(self.plan, OptionPlan::Shield { .. })
            && !ctx.stopped
            && enemy.status == CLIFF_ATTACK
        {
            match enemy.attack_active {
                Some(true) => {
                    self.getup_hitboxes_seen = true;
                    self.getup_hitboxes_ended = false;
                }
                Some(false) if self.getup_hitboxes_seen => self.getup_hitboxes_ended = true,
                _ => {}
            }
        }
        let blocked_now = ctx.status == GUARD_DAMAGE
            && enemy.status == CLIFF_ATTACK
            && ctx.shield_attacker.is_none_or(|id| id == enemy.id);
        let missed_stun_transition = ctx.previous_status == GUARD_DAMAGE
            && matches!(ctx.status, GUARD_ON | GUARD)
            && self.last_output.requested & MAPPER_ZR != 0;
        if matches!(self.plan, OptionPlan::Shield { .. }) && (blocked_now || missed_stun_transition)
        {
            self.plan = OptionPlan::Shield { blocked: true };
        }
    }

    /// Native kinetic speed can omit animation-driven ledge travel. Estimate
    /// world-space speed from root displacement only when the SAME animation
    /// clock advances. Repeated mapper polls reuse the last estimate.
    fn target_velocity(&mut self, enemy: Opponent, stopped: bool) -> [f32; 2] {
        let mut velocity = enemy.velocity.map(finite);
        if !stopped {
            if let (Some(previous), Some(current)) = (self.previous_enemy, enemy.motion) {
                if let Some(old) = previous.motion {
                    if previous.id == enemy.id
                        && previous.status == enemy.status
                        && old.kind == current.kind
                    {
                        let elapsed = (current.frame - old.frame) / current.rate;
                        if current.frame == old.frame {
                            return self.observed_velocity;
                        }
                        if elapsed.is_finite() && (0.25..=8.0).contains(&elapsed) {
                            let observed = [
                                (enemy.position[0] - previous.position[0]) / elapsed,
                                (enemy.position[1] - previous.position[1]) / elapsed,
                            ];
                            if observed
                                .iter()
                                .all(|speed| speed.is_finite() && speed.abs() <= 30.0)
                            {
                                velocity = observed;
                            }
                        }
                    }
                }
            }
            self.previous_enemy = Some(enemy);
            self.observed_velocity = velocity;
        }
        velocity
    }
}

#[cfg(test)]
const NEUTRAL_AIR_FIRST: u8 = crate::fighter_profiles::kirby::LEDGE_DATA.neutral_air.first;
#[cfg(test)]
const NEUTRAL_AIR_LAST: u8 = crate::fighter_profiles::kirby::LEDGE_DATA.neutral_air.last;
#[cfg(test)]
mod tests;
