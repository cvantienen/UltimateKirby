//! Shared gameplay observations and controller-input requests.
//!
//! These values are copied between automation policies and the live adapter.
//! They contain no game pointers and do not write native fighter state.

/// Maximum number of animation forecast frames considered for timing calculations.
const MAX_ANIMATION_FORECAST_FRAMES: f32 = 120.0;

/// Read-only animation timing; remaining values are measured in game frames.
/// This struct captures the timing information for a single animation,
///  including its current frame, rate of progress, and remaining duration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTiming {
    /// Animation Hash40 identifier, not a status ID or frame count.
    pub kind: u64,
    /// Animation frame, which can be fractional.
    pub frame: f32,
    /// Converts animation progress into game-frame time.
    pub rate: f32,
    /// None means unknown; Some(0.0) means the animation already ended.
    pub remaining: Option<f32>,
}

/// Convert an animation-frame deadline into game-frame time.
///
/// Invalid or past deadlines remain unknown instead of being treated as ready.
pub fn frames_until(frame: f32, deadline: f32, rate: f32) -> Option<f32> {
    if !frame.is_finite()
        || frame < 0.0
        || !deadline.is_finite()
        || !rate.is_finite()
        || rate <= 0.0
    {
        return None;
    }
    let remaining = (deadline - frame) / rate;
    (deadline > 0.0 && (0.0..=MAX_ANIMATION_FORECAST_FRAMES).contains(&remaining))
        .then_some(remaining)
}

/// Validated native air kinetics for short movement forecasts.
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

    #[allow(dead_code)]
    pub(crate) fn displacement(self, speed: f32, frames: f32) -> f32 {
        let mut speed = speed;
        let mut distance = 0.0;
        let mut remaining = frames.clamp(0.0, MAX_ANIMATION_FORECAST_FRAMES);
        while remaining > 0.0 {
            let step = remaining.min(1.0);
            distance += speed * step;
            speed = (speed - self.gravity * step).max(-self.terminal_speed);
            remaining -= step;
        }
        distance
    }
}

/// Return time to the earlier animation end or native cancel deadline.
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

/// Read-only opponent facts copied by the platform adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Opponent {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
    pub situation: i32,
    pub position: [f32; 2],
    /// Aggregate native hit status, not an individual hurtbox model.
    pub vulnerable: bool,
    pub velocity: [f32; 2],
    /// Only ledge policies use this fallback anchor when the fighter hangs.
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

/// Controlled fighter's native state, copied by the platform adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameplayContext {
    /// Whether the controlled fighter is currently holding an input.
    pub held: bool,
    /// Whether the controlled fighter is currently in active gameplay.
    pub gameplay: bool,
    /// Changes only on fresh physical gameplay input, never simulated output.
    pub manual_input_epoch: u64,
    /// Native readiness for this consumer's intended grounded action.
    pub can_act: bool,
    /// The assist may own inputs, including during its guard/grab/tilt animation.
    pub can_control: bool,
    /// Native hitstop: preserve an existing shield, but start no new actions.
    pub stopped: bool,
    /// The controlled fighter's current status kind.
    pub status: i32,
    /// The controlled fighter's current situation kind.
    pub situation: i32,
    /// The controlled fighter's native motion clock distinguishes progress when a target
    /// holds one animation pose; it is also part of duplicate-observation reuse.
    pub motion_frame: f32,
    /// The controlled fighter's current motion frame within its animation.
    pub motion_kind: u64,
    /// The controlled fighter's current motion kind.
    pub motion_rate: f32,
    /// The controlled fighter's current motion rate.
    pub position: [f32; 2],
    /// The controlled fighter's current position in world coordinates.
    pub velocity: [f32; 2],
    /// Native ground deceleration; None avoids assuming a stopping distance.
    pub ground_brake: Option<f32>,
    /// Facing uses its sign (+right/-left); position/velocity remain absolute
    /// world axes. Multiplying dx by facing asks if the target is in front.
    pub facing: f32,
    /// The controlled fighter's previous status kind.
    pub previous_status: i32,
    /// Whether the controlled fighter can currently perform a grab.
    pub can_grab: bool,
    /// Whether the controlled fighter can currently perform a tilt.
    pub can_tilt: bool,
    /// Whether the controlled fighter can currently perform a down tilt.
    pub can_down_tilt: bool,
    /// Whether the controlled fighter can currently perform a down special.
    pub can_down_special: bool,
    /// Whether the controlled fighter can currently perform a dash attack.
    pub can_dash_attack: bool,
    /// Whether the controlled fighter can currently perform a down smash.
    pub can_down_smash: bool,
    /// Explicit additional delay in game frames; live observations supply zero.
    pub input_delay: f32,
    /// Whether the controlled fighter can currently perform a jump.
    pub can_jump: bool,
    /// Whether the controlled fighter can currently perform an air attack.
    pub can_air_attack: bool,
    /// The controlled fighter's current air physics, if applicable.
    pub air_physics: Option<AirPhysics>,
    /// The controlled fighter's short hop speed, if applicable.
    pub short_hop_speed: Option<f32>,
    /// The controlled fighter's full hop speed, if applicable.
    pub full_hop_speed: Option<f32>,
    /// The number of frames the controlled fighter is in shield stun, if applicable.
    pub shield_stun_frames: Option<i32>,
    /// The ID of the attacker causing the shield stun, if applicable.
    pub shield_attacker: Option<u32>,
    /// Native hit infliction during Kirby's current assisted attack.
    pub hit_confirmed: bool,
    /// Ground support along a short swept lookahead, left then right.
    pub safe_ground: [bool; 2],
}

/// Proposed controller inputs; the live adapter arbitrates and maps the request.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ControllerOutput {
    /// Whether to consume the trigger for this update.
    pub consume_trigger: bool,
    /// The desired stick position for this update, if any.
    pub stick: Option<[f32; 2]>,
    /// Buttons to hold for this update (a mask of native controller bits).
    pub requested: u32,
    /// Buttons to replace, including buttons that should currently be released.
    pub owned: u32,
    /// Preserve a committed short-hop decision with this cached output until mapped.
    pub native_short_hop: bool,
}
