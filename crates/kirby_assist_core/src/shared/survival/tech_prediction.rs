//! Pure collision-prediction helpers, without game FFI or memory writes.
//! The native caller supplies status/motion and performs the actual ray checks.
//! Host tests use simple planes to test coverage; they do not simulate Smash's
//! complete collision system or prove that an in-game hit is legally techable.
//!
//! Tune body offsets here, keep native queries in src/live.rs, and add geometry
//! regression tests for each new case instead of embedding stage-specific fixes.
// Ignore only numerical rest, not genuine slow tumble/contact motion.
/// Defines the `APPROACH_EPSILON` constant used by this module.
///
/// Provides the APPROACH EPSILON constant value.
const APPROACH_EPSILON: f32 = 0.0001;
pub use crate::flame_choke::{
    grounded_ganon_contact_in, grounded_ganon_release, CATCHED_AIR_END_GANON, CATCHED_GANON,
};

/// Spend the existing ray budget in the last five damage-hitstop frames rather
/// than waiting until the last two. Global stop/unknown timers remain excluded.
///
///  Spend the existing ray budget in the last five damage-hitstop frames rather than waiting until the last two. Global stop/unknown timers remain excluded.
/// ### Arguments
/// - `remaining` (`u64`): Represents the remaining; the function uses it as input to the operation described above.
/// - `input_delay` (`f32`): Represents the input delay; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn late_damage_stop_prediction(remaining: u64, input_delay: f32) -> bool {
    input_delay.is_finite()
        && remaining > 0
        && remaining as f32
            <= input_delay.clamp(0.0, crate::tech_input::MAX_INPUT_DELAY_FRAMES) + 5.0
}

/// During victim hitstop the reserved new launch is more useful than frozen
/// kinetic motion. Invalid/zero reservations retain the ordinary velocity.
///
///  During victim hitstop the reserved new launch is more useful than frozen kinetic motion. Invalid/zero reservations retain the ordinary velocity.
/// ### Arguments
/// - `current` (`[f32; 2]`): Represents the current; the function uses it as input to the operation described above.
/// - `reserved` (`Option<[f32; 2]>`): Represents the reserved; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[f32; 2]` value.
pub fn prediction_velocity(current: [f32; 2], reserved: Option<[f32; 2]>) -> [f32; 2] {
    reserved
        .filter(|v| v.iter().all(|x| x.is_finite()) && v.iter().any(|x| x.abs() > APPROACH_EPSILON))
        .unwrap_or(current)
}

/// Native acceleration band in which one-frame floor sweeps are trusted.
///
///  Native acceleration band in which one-frame floor sweeps are trusted.
const FLOOR_ACCELERATION: std::ops::RangeInclusive<f32> = -0.3..=-0.005;

/// Vertical offset after `t` frames of constant acceleration.
///
///  Vertical offset after t frames of constant acceleration.
/// ### Arguments
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `acceleration` (`f32`): Represents the acceleration; the function uses it as input to the operation described above.
/// - `t` (`f32`): Represents the t; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `f32` value.
fn arc_height(vy: f32, acceleration: f32, t: f32) -> f32 {
    vy * t + 0.5 * acceleration * t * t
}

/// One recent, copied observation; no game pointer or poll-based clock.
///
///  One recent, copied observation; no game pointer or poll-based clock.
#[derive(Clone, Copy, Debug)]
pub struct FloorMotionSample {
    /// Stores the `object` value in `FloorMotionSample` as `u32`.
    ///
    ///
///  Holds the object value used by FloorMotionSample.
    pub object: u32,
    /// Stores the `status` value in `FloorMotionSample` as `i32`.
    ///
    ///
///  Holds the status value used by FloorMotionSample.
    pub status: i32,
    /// Stores the `motion` value in `FloorMotionSample` as `u64`.
    ///
    ///
///  Holds the motion value used by FloorMotionSample.
    pub motion: u64,
    /// Stores the `frame` value in `FloorMotionSample` as `f32`.
    ///
    ///
///  Holds the frame value used by FloorMotionSample.
    pub frame: f32,
    /// Stores the `rate` value in `FloorMotionSample` as `f32`.
    ///
    ///
///  Holds the rate value used by FloorMotionSample.
    pub rate: f32,
    /// Stores the `vy` value in `FloorMotionSample` as `f32`.
    ///
    ///
///  Holds the vy value used by FloorMotionSample.
    pub vy: f32,
}
/// Defines the `FloorMotionHistory` data structure.
///
/// Holds the data for FloorMotionHistory.
#[derive(Clone, Copy, Default)]
pub struct FloorMotionHistory {
    /// Stores the `last` value in `FloorMotionHistory` as `Option<FloorMotionSample>`.
    ///
    ///
///  Optionally stores the last for FloorMotionHistory; it may be absent.
    last: Option<FloorMotionSample>,
    /// Stores the `acceleration` value in `FloorMotionHistory` as `f32`.
    ///
    ///
///  Holds the acceleration value used by FloorMotionHistory.
    acceleration: f32,
}
impl FloorMotionHistory {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    pub const fn new() -> Self {
        Self {
            last: None,
            acceleration: 0.0,
        }
    }
    /// Estimate only small, recent downward acceleration in the same native
    /// motion. Discontinuities/new hits/unknown clocks fall back to linear rays.
    ///
    ///  Estimate only small, recent downward acceleration in the same native motion. Discontinuities/new hits/unknown clocks fall back to linear rays.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `now` (`FloorMotionSample`): Represents the current time; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `f32` value.
    pub fn sample(&mut self, now: FloorMotionSample) -> f32 {
        if !crate::timing::clock_known(now.motion, now.frame, now.rate) || !now.vy.is_finite() {
            *self = Self::default();
            return 0.0;
        }
        let last = self.last.replace(now);
        if let Some(last) = last.filter(|last| {
            last.object == now.object
                && last.status == now.status
                && last.motion == now.motion
                && (last.rate - now.rate).abs() < 0.001
        }) {
            let elapsed = (now.frame - last.frame) / now.rate;
            if elapsed == 0.0 && now.vy == last.vy {
                return self.acceleration;
            }
            if (0.5..=2.0).contains(&elapsed) {
                let acceleration = (now.vy - last.vy) / elapsed;
                if FLOOR_ACCELERATION.contains(&acceleration) {
                    self.acceleration = acceleration;
                    return acceleration;
                }
            }
        }
        self.acceleration = 0.0;
        0.0
    }
}

/// The pinned status range covering damage-fly/reflect/fall, not ordinary
/// airborne movement. Broadening it carelessly could cause unrelated dodges.
///
///  The pinned status range covering damage-fly/reflect/fall, not ordinary airborne movement. Broadening it carelessly could cause unrelated dodges.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn is_tumble_status(status: i32) -> bool {
    (crate::native_states::status::DAMAGE_FLY..=crate::native_states::status::DAMAGE_FALL)
        .contains(&status)
}

/// Statuses in which a shield press can be a tech or release request: tumble,
/// wall/ceiling bounce, and the Flame Choke capture/release pair. Release
/// eligibility additionally requires a verified grounded predecessor in the
/// native adapter; aerial releases never receive a contact forecast.
///
///  Statuses in which a shield press can be a tech or release request: tumble, wall/ceiling bounce, and the Flame Choke capture/release pair. Release eligibility additionally requires a verified grounded predecessor in the native.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn is_tech_status(status: i32) -> bool {
    is_tumble_status(status)
        || matches!(
            status,
            crate::native_states::status::STOP_WALL | crate::native_states::status::STOP_CEIL
        )
        || crate::flame_choke::guard_lockout_status(status)
}

/// Withhold automatic guard once native air escape can consume the input.
///
///  Withhold automatic guard once native air escape can consume the input.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `air_escape_enabled` (`bool`): Represents the air escape enabled; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn tech_press_allowed(status: i32, air_escape_enabled: bool) -> bool {
    is_tech_status(status) && !air_escape_enabled
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// These are reasons OUR detector proceeds or stops. Passing Predict is not the
// same as the game's final decision to accept a shield input as a legal tech.
/// Defines the available `TechGate` cases.
///
/// Lists the available TechGate choices.
pub enum TechGate {
    /// Represents the `OutsideTumble` case of `TechGate`.
    ///
    ///
///  Represents the OutsideTumble option in TechGate.
    OutsideTumble,
    /// Represents the `NotAirborne` case of `TechGate`.
    ///
    ///
///  Represents the NotAirborne option in TechGate.
    NotAirborne,
    /// Represents the `PassiveDisabled` case of `TechGate`.
    ///
    ///
///  Represents the PassiveDisabled option in TechGate.
    PassiveDisabled,
    /// Represents the `Hitstop` case of `TechGate`.
    ///
    ///
///  Represents the Hitstop option in TechGate.
    Hitstop,
    /// Represents the `Predict` case of `TechGate`.
    ///
    ///
///  Represents the Predict option in TechGate.
    Predict,
}

impl TechGate {
    /// Returns the display label for the value.
    ///
    ///
    ///  Gets the text shown for this value.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting text.
    pub const fn label(self) -> &'static str {
        match self {
            Self::OutsideTumble => "outside_supported_tumble",
            Self::NotAirborne => "not_airborne",
            Self::PassiveDisabled => "native_passive_disabled",
            Self::Hitstop => "hitstop_wait",
            Self::Predict => "predict",
        }
    }
}

/// Classify native gates without claiming that passing them guarantees a tech.
///
///  Classify native gates without claiming that passing them guarantees a tech.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// - `passive_disabled` (`bool`): Represents the passive disabled; the function uses it as input to the operation described above.
/// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `TechGate` value.
pub fn tech_gate(status: i32, situation: i32, passive_disabled: bool, stopped: bool) -> TechGate {
    if !is_tumble_status(status) {
        TechGate::OutsideTumble
    } else if situation != crate::native_states::situation::AIR {
        TechGate::NotAirborne
    } else if passive_disabled {
        TechGate::PassiveDisabled
    } else if stopped {
        TechGate::Hitstop
    } else {
        TechGate::Predict
    }
}

/// A floor collision can change situation before the last techable observation.
/// Admit grounded tumble only when the native floor-tech transition is enabled;
/// ordinary landing/down states and disabled-passive hits remain excluded.
///
///  A floor collision can change situation before the last techable observation. Admit grounded tumble only when the native floor-tech transition is enabled; ordinary landing/down states and disabled-passive hits remain excluded.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// - `passive_disabled` (`bool`): Represents the passive disabled; the function uses it as input to the operation described above.
/// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
/// - `native_floor_tech` (`bool`): Represents the native floor tech; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `TechGate` value.
pub fn tech_gate_with_floor(
    status: i32,
    situation: i32,
    passive_disabled: bool,
    stopped: bool,
    native_floor_tech: bool,
) -> TechGate {
    tech_gate(
        status,
        if situation == crate::native_states::situation::GROUND && native_floor_tech {
            crate::native_states::situation::AIR
        } else {
            situation
        },
        passive_disabled,
        stopped,
    )
}

#[derive(Clone, Copy, Debug)]
// Coordinates and displacement use game world units. [f32; 2] is a fixed
// two-element float array: index 0 is x (right), index 1 is y (up).
/// Defines the `CollisionRay` data structure.
///
/// Holds the data for CollisionRay.
pub struct CollisionRay {
    /// Stores the `origin` value in `CollisionRay` as `[f32; 2]`.
    ///
    ///
///  Stores the origin collection for CollisionRay.
    pub origin: [f32; 2],
    /// Stores the `delta` value in `CollisionRay` as `[f32; 2]`.
    ///
    ///
///  Stores the delta collection for CollisionRay.
    pub delta: [f32; 2],
    /// Only descending floor probes can collide with a one-way platform.
    ///
///  Only descending floor probes can collide with a one-way platform.
    pub passable: bool,
}

/// Collision-probe dimensions in local fighter units.
///
///  Collision-probe dimensions in local fighter units.
#[derive(Clone, Copy, Debug)]
pub struct Body {
    /// Stores the `half_width` value in `Body` as `f32`.
    ///
    ///
///  Holds the half width value used by Body.
    pub half_width: f32,
    /// Stores the `floor_origin` value in `Body` as `f32`.
    ///
    ///
///  Holds the floor origin value used by Body.
    pub floor_origin: f32,
    /// Stores the `wall` value in `Body` as `[f32; 3]`.
    ///
    ///
///  Stores the wall collection for Body.
    pub wall: [f32; 3],
    /// Stores the `ceiling_origin` value in `Body` as `f32`.
    ///
    ///
///  Holds the ceiling origin value used by Body.
    pub ceiling_origin: f32,
    /// Stores the `floor_padding` value in `Body` as `f32`.
    ///
    ///
///  Holds the floor padding value used by Body.
    pub floor_padding: f32,
    /// Stores the `ceiling_padding` value in `Body` as `f32`.
    ///
    ///
///  Holds the ceiling padding value used by Body.
    pub ceiling_padding: f32,
}
impl Default for Body {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    fn default() -> Self {
        Self {
            half_width: 2.5,
            floor_origin: 1.,
            wall: [1., 3.5, 6.],
            ceiling_origin: 5.,
            floor_padding: 1.5,
            ceiling_padding: 2.,
        }
    }
}
impl Body {
    /// Forecast dimensions are local fighter units. Native contacts remain
    /// authoritative; do not use hurtbox/model height as an invented ECB size.
    ///
    ///  Forecast dimensions are local fighter units. Native contacts remain authoritative; do not use hurtbox/model height as an invented ECB size.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `scale` (`f32`): Represents the scale; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    pub fn scaled(self, scale: f32) -> Self {
        let scale = if scale.is_finite() && (0.1..=10.).contains(&scale) {
            scale
        } else {
            1.
        };
        Self {
            half_width: self.half_width * scale,
            floor_origin: self.floor_origin * scale,
            wall: self.wall.map(|v| v * scale),
            ceiling_origin: self.ceiling_origin * scale,
            floor_padding: self.floor_padding * scale,
            ceiling_padding: self.ceiling_padding * scale,
        }
    }
    /// Finite, non-negative and bounded: rejects corrupted per-fighter tuning.
    ///
    ///  Finite, non-negative and bounded: rejects corrupted per-fighter tuning.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    fn plausible(&self) -> bool {
        [
            self.half_width,
            self.floor_origin,
            self.ceiling_origin,
            self.floor_padding,
            self.ceiling_padding,
            self.wall[0],
            self.wall[1],
            self.wall[2],
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0. && *v <= 100.)
    }
}
/// Three probes per approaching surface; unused directions stay empty.
///
///  Three probes per approaching surface; unused directions stay empty.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[Option<CollisionRay>; 9]` value.
pub fn tech_probes(x: f32, y: f32, vx: f32, vy: f32, horizon: f32) -> [Option<CollisionRay>; 9] {
    tech_probes_with_body(x, y, vx, vy, horizon, Body::default())
}
/// Returns an optional `tech_probes_with_body` value.
///
/// Gets tech probes with body if it is available.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[Option<CollisionRay>; 9]` value.
pub fn tech_probes_with_body(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    body: Body,
) -> [Option<CollisionRay>; 9] {
    // Reserve three floor, three wall, and three ceiling slots. Option::None
    // means that direction is not approaching; callers use flatten() to skip it.
    let mut probes = [None; 9];
    if ![x, y, vx, vy, horizon, vx * horizon, vy * horizon]
        .iter()
        .all(|v| v.is_finite())
        || !(1.0..=20.0).contains(&horizon)
    {
        return probes;
    }
    // Default side offsets and heights are generic body estimates; callers may
    // supply per-fighter tuning. These values
    // are not queried collision bounds; slopes/platform motion remain heuristic.
    // velocity * horizon sweeps ahead rather than checking only the current point.
    // enumerate() pairs each body offset with a slot index. Positive y is up:
    // negative vy approaches a floor, positive vy approaches a ceiling. Faster
    // motion makes a longer sweep, rather than waiting until Kirby touches it.
    for (i, offset) in [-body.half_width, 0.0, body.half_width]
        .into_iter()
        .enumerate()
    {
        let surfaces = [
            (vy < -APPROACH_EPSILON, [x + offset, y + body.floor_origin], true),
            (vx.abs() > APPROACH_EPSILON, [x, y + body.wall[i]], false),
            (vy > APPROACH_EPSILON, [x + offset, y + body.ceiling_origin], false),
        ];
        for (surface, (approaching, origin, passable)) in surfaces.into_iter().enumerate() {
            if approaching {
                probes[surface * 3 + i] = Some(CollisionRay {
                    origin,
                    delta: probe_delta(surface, vx, vy, horizon, body),
                    passable,
                });
            }
        }
    }
    probes
}

/// Displacement of a floor (0), wall (1) or ceiling (2) probe over `horizon`
/// frames, padded by the body so thin surfaces are caught before contact.
///
///  Displacement of a floor (0), wall (1) or ceiling (2) probe over horizon frames, padded by the body so thin surfaces are caught before contact.
/// ### Arguments
/// - `surface` (`usize`): Represents the surface; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[f32; 2]` value.
fn probe_delta(surface: usize, vx: f32, vy: f32, horizon: f32, body: Body) -> [f32; 2] {
    match surface {
        0 => [vx * horizon, vy * horizon - body.floor_padding],
        1 => [vx * horizon + vx.signum() * body.half_width, vy * horizon],
        _ => [vx * horizon, vy * horizon + body.ceiling_padding],
    }
}

// The full ray already passed all eligibility/finite checks. Refinement only
// changes its horizon; rebuilding all nine body probes for one slot is wasteful.
/// Returns the `refined_probe` result.
///
/// Gets the refined probe result.
/// ### Arguments
/// - `ray` (`CollisionRay`): Represents the ray; the function uses it as input to the operation described above.
/// - `index` (`usize`): Represents the zero-based position of an item; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `CollisionRay` value.
fn refined_probe(
    ray: CollisionRay,
    index: usize,
    vx: f32,
    vy: f32,
    horizon: f32,
    body: Body,
) -> CollisionRay {
    CollisionRay {
        delta: probe_delta(index / 3, vx, vy, horizon, body),
        ..ray
    }
}

/// Estimate first contact using bounded ray refinement. The native query only
/// reports a hit, so this brackets time rather than assuming the full lookahead
/// is the collision time. At most five refinement queries per hit ray.
///
///  Estimate first contact using bounded ray refinement. The native query only reports a hit, so this brackets time rather than assuming the full lookahead is the collision time. At most five refinement queries per hit ray.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `collides` (`impl FnMut(CollisionRay) -> bool`): Represents the collides; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn first_contact_in(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    collides: impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    first_contact_with_floor_acceleration(x, y, vx, vy, horizon, 0.0, collides)
}

/// Use at most five one-frame floor sweeps when recent native motion shows
/// downward acceleration. Rising segments before an apex are excluded.
/// Wall/ceiling rays and fallback geometry are unchanged.
///
///  Use at most five one-frame floor sweeps when recent native motion shows downward acceleration. Rising segments before an apex are excluded. Wall/ceiling rays and fallback geometry are unchanged.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `acceleration` (`f32`): Represents the acceleration; the function uses it as input to the operation described above.
/// - `collides` (`impl FnMut(CollisionRay) -> bool`): Represents the collides; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn first_contact_with_floor_acceleration(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    acceleration: f32,
    collides: impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    first_contact_with_body(
        x,
        y,
        vx,
        vy,
        horizon,
        acceleration,
        Body::default(),
        collides,
    )
}
/// Determines whether the `first_contact_with_body` condition holds.
///
/// Checks whether first contact with body.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `acceleration` (`f32`): Represents the acceleration; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// - `collides` (`impl FnMut(CollisionRay) -> bool`): Represents the collides; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn first_contact_with_body(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    acceleration: f32,
    body: Body,
    mut collides: impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    let full = tech_probes_with_body(x, y, vx, vy, horizon, body);
    let mut earliest: Option<f32> = None;
    let curved_floor = [x, y, vx, vy, horizon, vx * horizon, vy * horizon]
        .iter()
        .all(|v| v.is_finite())
        && (1.0..=5.0).contains(&horizon)
        && vy + acceleration * horizon < -APPROACH_EPSILON
        && FLOOR_ACCELERATION.contains(&acceleration);
    if curved_floor {
        earliest = curved_floor_contact(x, y, vx, vy, horizon, acceleration, body, &mut collides);
        if earliest.is_some_and(|end| end <= 1.0) {
            return earliest;
        }
    }
    for (index, ray) in full.into_iter().enumerate() {
        if curved_floor && index < 3 {
            continue;
        }
        let Some(ray) = ray else {
            continue;
        };
        // A long diagonal with fixed body padding can miss a narrow surface
        // that the one-frame ray hits. It must not veto imminent contact.
        // Reuse the existing refinement query first; an immediate hit avoids
        // the full sweep and all later probes. Hit/refinement budgets stay fixed.
        if collides(refined_probe(ray, index, vx, vy, 1.0, body)) {
            return Some(1.0);
        }
        if horizon <= 1.0 || !collides(ray) {
            continue;
        }
        let (mut low, mut high) = (1.0, horizon);
        for _ in 0..4 {
            if high - low <= 0.25 {
                break;
            }
            let mid = (low + high) * 0.5;
            let ray = refined_probe(ray, index, vx, vy, mid, body);
            if collides(ray) {
                high = mid;
            } else {
                low = mid;
            }
        }
        earliest = Some(earliest.map_or(high, |old| old.min(high)));
        // One frame is the predictor's minimum. No later body ray can improve
        // that result, so avoid the remaining collision queries.
        if high <= 1.0 {
            return earliest;
        }
    }
    earliest
}

/// The first one-frame floor sweep along the accelerated arc that hits, as its
/// end time. Rising segments before an apex are excluded.
///
///  The first one-frame floor sweep along the accelerated arc that hits, as its end time. Rising segments before an apex are excluded.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `acceleration` (`f32`): Represents the acceleration; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// - `collides` (`&mut impl FnMut(CollisionRay) -> bool`): Represents the collides; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn curved_floor_contact(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    acceleration: f32,
    body: Body,
    collides: &mut impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    let apex = if vy > 0.0 { -vy / acceleration } else { 0.0 };
    for step in 1..=5 {
        let end = (step as f32).min(horizon);
        let before = (step - 1) as f32;
        if before >= end {
            break;
        }
        let start = before.max(apex);
        if start >= end {
            continue;
        }
        for offset in [-body.half_width, 0.0, body.half_width] {
            let ray = CollisionRay {
                origin: [
                    x + vx * start + offset,
                    y + arc_height(vy, acceleration, start) + body.floor_origin,
                ],
                delta: [
                    vx * (end - start),
                    arc_height(vy, acceleration, end)
                        - arc_height(vy, acceleration, start)
                        - body.floor_padding,
                ],
                passable: true,
            };
            if collides(ray) {
                return Some(end);
            }
        }
    }
    None
}

/// Native adapter's contact forecast: sweep each simulation frame separately.
/// A long padded diagonal is not a monotonic collision predicate and can miss
/// narrow platforms between its endpoints. Frame segments preserve the body
/// offsets along the entire path. Native passive terms select eligible surfaces
/// in floor/wall/ceiling order; a disabled surface cannot trigger a shield edge.
///
///  Native adapter's contact forecast: sweep each simulation frame separately. A long padded diagonal is not a monotonic collision predicate and can miss narrow platforms between its endpoints. Frame segments preserve the body.
/// ### Arguments
/// - `x` (`f32`): Represents the horizontal coordinate; the function uses it as input to the operation described above.
/// - `y` (`f32`): Represents the vertical coordinate; the function uses it as input to the operation described above.
/// - `vx` (`f32`): Represents the vx; the function uses it as input to the operation described above.
/// - `vy` (`f32`): Represents the vy; the function uses it as input to the operation described above.
/// - `horizon` (`f32`): Represents the horizon; the function uses it as input to the operation described above.
/// - `acceleration` (`f32`): Represents the acceleration; the function uses it as input to the operation described above.
/// - `body` (`Body`): Represents the body; the function uses it as input to the operation described above.
/// - `surfaces` (`[bool; 3]`): Represents the surfaces; the function uses it as input to the operation described above.
/// - `collides` (`impl FnMut(CollisionRay) -> bool`): Represents the collides; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn first_contact_swept(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    acceleration: f32,
    body: Body,
    surfaces: [bool; 3],
    mut collides: impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    if ![x, y, vx, vy, horizon].iter().all(|v| v.is_finite())
        || !(1.0..=20.0).contains(&horizon)
        || !body.plausible()
    {
        return None;
    }
    let acceleration = if FLOOR_ACCELERATION.contains(&acceleration) {
        acceleration
    } else {
        0.
    };
    let height = |t: f32| arc_height(vy, acceleration, t);
    for step in 0..horizon.ceil() as u32 {
        let start = step as f32;
        let end = (start + 1.).min(horizon);
        let duration = end - start;
        let dx = vx * duration;
        let dy = height(end) - height(start);
        let rays = tech_probes_with_body(x + vx * start, y + height(start), dx, dy, 1., body);
        let mut earliest: Option<f32> = None;
        for (index, ray) in rays.into_iter().enumerate() {
            if !surfaces[index / 3] {
                continue;
            }
            let Some(ray) = ray else { continue };
            if !collides(ray) {
                continue;
            }
            let (mut low, mut high) = (0., 1.);
            for _ in 0..3 {
                let mid = (low + high) * 0.5;
                if collides(refined_probe(ray, index, dx, dy, mid, body)) {
                    high = mid;
                } else {
                    low = mid;
                }
            }
            let contact = (start + high * duration).max(1.);
            earliest = Some(earliest.map_or(contact, |old| old.min(contact)));
        }
        if earliest.is_some() {
            return earliest;
        }
    }
    None
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
#[path = "../../../tests/unit/tech_prediction.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/unit/tech_prediction_grounded_ganon.rs"]
mod grounded_ganon_tests;
