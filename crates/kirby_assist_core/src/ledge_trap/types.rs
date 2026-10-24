//! Data passed into and out of the ledge policy; no game pointers live here.
//!
//! `LedgeSettings` is startup tuning. `LedgeContext` and `Opponent` are copied
//! observations for this update. `LedgeOutput` is an input request, not evidence
//! that Smash accepted the action. Optional timing is unknown when it is `None`.
//! These types stay available through `ledge_trap`, which re-exports them.
use super::is_hanging;
use crate::fighter_profiles::{LedgeFighterData, LedgeResponses};
use crate::native_input::MAPPER_R;

/// Normal-getup punish; short-hop neutral air is the default. Legacy forward tilt
/// is angled down for low targets;
/// choosing it does not change the neutral-air jump response, roll down smash, or shield grabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GetupAttack {
    DownTilt,
    ForwardTilt,
    #[default]
    NeutralAir,
    DownSpecial,
    NeutralSpecial,
}

/// Executable ledge-drop recipes; each can have its own saved tuning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LedgeDropAction {
    #[default]
    Shield,
    DownTiltShield,
    ShortHopDair,
    Bomb,
}
/// Optional numeric overrides. The selected hardcoded function supplies defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResponseAdjustments {
    pub nair_hit_lead: Option<u8>,
    pub stone_hit_lead: Option<u8>,
    pub stone_startup: Option<u8>,
}

/// Read-only animation timing; remaining values are converted to game frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTiming {
    // kind is an animation Hash40 identifier, not a status ID or frame count.
    // frame can be fractional; rate converts animation progress to game time.
    // remaining=None means unknown, whereas Some(0.0) means already at the end.
    pub kind: u64,
    pub frame: f32,
    pub rate: f32,
    pub remaining: Option<f32>,
}

/// Animation rates can differ by character/motion. Invalid metadata stays
/// unknown instead of becoming a guessed invincibility or action deadline.
pub fn frames_until(frame: f32, deadline: f32, rate: f32) -> Option<f32> {
    if !frame.is_finite()
        || frame < 0.0
        || !deadline.is_finite()
        || !rate.is_finite()
        || rate <= 0.0
    {
        return None;
    }
    // Example: animation frame 10 -> deadline 16 at rate 2 gives 3 game
    // frames. Reject past/invalid deadlines instead of treating them as "now".
    let remaining = (deadline - frame) / rate;
    (deadline > 0.0 && (0.0..=120.0).contains(&remaining)).then_some(remaining)
}

/// Validated native air kinetics, used only for the short airborne forecast.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AirPhysics {
    pub gravity: f32,
    pub terminal_speed: f32,
}
impl AirPhysics {
    pub fn new(gravity: f32, terminal_speed: f32) -> Option<Self> {
        (gravity.is_finite()
            && (0.001..=2.0).contains(&gravity)
            && terminal_speed.is_finite()
            && (0.1..=10.0).contains(&terminal_speed))
        .then_some(Self {
            gravity,
            terminal_speed,
        })
    }

    pub(super) fn displacement(self, speed: f32, frames: f32) -> f32 {
        // Integrate native gravity with terminal fall speed, including fractional
        // frames. No acceleration is fabricated when native parameters are absent.
        let mut speed = speed;
        let mut distance = 0.0;
        let mut remaining = frames.clamp(0.0, 120.0);
        while remaining > 0.0 {
            let step = remaining.min(1.0);
            distance += speed * step;
            speed = (speed - self.gravity * step).max(-self.terminal_speed);
            remaining -= step;
        }
        distance
    }
}

/// Earlier native cancel/end deadline; a past cancel is already actionable.
pub fn recovery_frames(frame: f32, end: f32, cancel: f32, rate: f32) -> Option<f32> {
    let end = frames_until(frame, end, rate)?;
    if cancel.is_finite() && cancel > 0.0 {
        if cancel >= frame + end * rate {
            return Some(end);
        }
        if cancel <= frame {
            return Some(0.0);
        }
        Some(frames_until(frame, cancel, rate)?.min(end))
    } else {
        Some(end)
    }
}

/// Validated config values. Defaults and response timing are shared by both builds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeSettings {
    /// Select the separate recovery-intercept policy instead of get-up recipes.
    pub two_frame: bool,
    /// Kirby's held-L3 recovery cover uses down tilt; legacy core callers may
    /// retain the down-smash recipe when this is false.
    pub two_frame_down_tilt: bool,
    /// Resolved executable handler; a profile without one cannot run ledge recipes.
    pub implementation: Option<super::ResponseImplementation>,
    /// Indices into the fighter's compiled function lists. None preserves direct
    /// core/legacy callers that supply settings without using the battle menu.
    pub response_choices: [Option<usize>; 5],
    /// Numeric-only startup adjustments; these cannot register new actions.
    pub response_adjustments: ResponseAdjustments,
    /// Resolved move timings and response switches for our selected fighter.
    pub fighter: LedgeFighterData,
    pub aerial_execution: crate::fighter_profiles::execution::AerialExecution,
    pub responses: LedgeResponses,
    pub enabled: bool,
    /// Enable the selected response to a witnessed generic ledge release.
    pub drop_defense: bool,
    pub drop_action: LedgeDropAction,
    /// Native-time budget to request/confirm the drop punish before shielding.
    pub drop_request_timeout_frames: u16,
    pub getup_attack: GetupAttack,
    // Ranges/heights/stance distances are WORLD units, not stick percentages.
    // Increasing reach only makes the policy willing to attack from farther
    // away; it does not enlarge Kirby's real hitbox and can create more whiffs.
    pub anti_air_range: f32,
    pub anti_air_height: f32,
    /// Broad target acquisition limits in world units.
    pub horizontal_range: f32,
    pub vertical_range: f32,
    /// Separate attack limits prevent treating acquisition range as hitbox reach.
    pub tilt_range: f32,
    pub grab_range: f32,
    // Follow limits keep the acquired opponent through its get-up travel;
    // acquisition limits above decide whether to acquire it in the first place.
    pub follow_range: f32,
    pub follow_vertical_range: f32,
    /// Preserve player movement between punish inputs. The roll response still
    /// navigates to its committed coverage point regardless of this setting.
    pub manual_spacing: bool,
    /// Player owns spacing; normal/jump/grab timing does not test reach.
    pub distance_agnostic: bool,
    /// Baseline horizontal walk strength: normal cap is -0.02, near-full
    /// horizontal cap is +0.03, both bounded to 0.1..0.45.
    pub walk_strength: f32,
    /// Legacy reach setting retained for config compatibility; roll navigation
    /// now uses the common roll_distance point instead of a reach forecast.
    pub roll_dash_range: f32,
    /// Plan contact within this many opening active frames of down smash (1..13).
    pub roll_hit_window_frames: u8,
    pub auto_position: bool,
    /// Heuristic stage-side stance offset, not a universal optimal location.
    pub stand_distance: f32,
    /// Common roll coverage distance inward from the ledge; auto uses 25 units.
    pub roll_distance: Option<f32>,
    // Unlike distances, move_strength is normalized stick magnitude (0..1).
    // grab_buffer_frames permits one early input near native shieldstun end;
    // it does not shorten shieldstun or the native grab animation.
    pub move_strength: f32,
    pub grab_buffer_frames: u16,
    /// Legacy parsed setting. Captures always return control to the player.
    pub throw_outward: bool,
    /// Native mask of the button mapped to Grab; R for this user's controls.
    pub grab_button: u32,
}
impl Default for LedgeSettings {
    fn default() -> Self {
        Self {
            two_frame: false,
            two_frame_down_tilt: false,
            implementation: Some(super::ResponseImplementation::Kirby),
            response_choices: [None; 5],
            response_adjustments: ResponseAdjustments::default(),
            fighter: crate::fighter_profiles::kirby::LEDGE_DATA,
            aerial_execution: crate::fighter_profiles::execution::AerialExecution::SHORT,
            responses: LedgeResponses::default(),
            enabled: false,
            drop_defense: true,
            drop_action: LedgeDropAction::Shield,
            drop_request_timeout_frames: 30,
            getup_attack: GetupAttack::NeutralAir,
            anti_air_range: 12.0,
            anti_air_height: 18.0,
            horizontal_range: 12.0,
            vertical_range: 10.0,
            tilt_range: 10.0,
            grab_range: 8.0,
            follow_range: 40.0,
            follow_vertical_range: 48.0,
            manual_spacing: true,
            distance_agnostic: true,
            walk_strength: 0.35,
            roll_dash_range: 10.0,
            roll_hit_window_frames: 5,
            auto_position: true,
            stand_distance: 8.0,
            roll_distance: None,
            move_strength: 1.0,
            grab_buffer_frames: 3,
            throw_outward: false,
            grab_button: MAPPER_R,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
// All fields are copied observations. Missing optional metadata is represented
// explicitly so tests and native callers cannot silently invent a deadline.
pub struct Opponent {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
    pub situation: i32,
    pub position: [f32; 2],
    /// Aggregate native hit status, not an individual hurtbox model.
    pub vulnerable: bool,
    pub velocity: [f32; 2],
    pub cliff_position: Option<[f32; 2]>,
    /// Sanitized native motion metadata; None means prediction is unavailable.
    pub vulnerable_in: Option<f32>,
    pub motion: Option<MotionTiming>,
    /// Present for get-up attacks; None preserves conservative block-only behavior.
    pub attack_active: Option<bool>,
    pub air_physics: Option<AirPhysics>,
    /// Native cancel deadline, separate from intermediate jump animation ends.
    pub actionable_in: Option<f32>,
}
/// Acquisition can use the actual cliff anchor instead of a hanging fighter's
/// much lower root. Tracked opponents get a separate, bounded follow-up box.
pub fn in_tracking_range(
    settings: LedgeSettings,
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

/// Controlled fighter's native state, copied by the Switch adapter before evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeContext {
    pub held: bool,
    pub gameplay: bool,
    /// Kirby can act from normal grounded movement right now.
    pub can_act: bool,
    /// The assist may own inputs, including during its guard/grab/tilt animation.
    pub can_control: bool,
    /// Native hitstop: preserve an existing shield, but start no new actions.
    pub stopped: bool,
    pub status: i32,
    pub situation: i32,
    /// Kirby's native motion clock distinguishes real progress when a target
    /// holds one animation pose; it is also part of duplicate-observation reuse.
    pub motion_frame: f32,
    pub motion_kind: u64,
    pub motion_rate: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    /// Native ground deceleration; None avoids assuming a stopping distance.
    pub ground_brake: Option<f32>,
    // Facing uses its sign (+right/-left); position/velocity remain absolute
    // world axes. Multiplying dx by facing asks if the target is in front.
    pub facing: f32,
    pub previous_status: i32,
    pub can_grab: bool,
    pub can_tilt: bool,
    pub can_down_tilt: bool,
    pub can_down_special: bool,
    pub can_dash_attack: bool,
    pub can_down_smash: bool,
    /// Explicit additional delay in game frames; live observations supply zero.
    pub input_delay: f32,
    pub can_jump: bool,
    pub can_air_attack: bool,
    pub air_physics: Option<AirPhysics>,
    pub short_hop_speed: Option<f32>,
    pub full_hop_speed: Option<f32>,
    pub shield_stun_frames: Option<i32>,
    pub shield_attacker: Option<u32>,
    /// Native hit infliction during Kirby's current assisted attack.
    pub hit_confirmed: bool,
    /// Ground support along a short swept lookahead, left then right.
    pub safe_ground: [bool; 2],
}
/// Proposed inputs, applied by ManualOverlay rather than written into game state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LedgeOutput {
    pub consume_trigger: bool,
    pub stick: Option<[f32; 2]>,
    /// Buttons to hold for this update (a mask of native controller bits).
    pub requested: u32,
    /// Buttons to replace, including buttons that should currently be released.
    pub owned: u32,
}
