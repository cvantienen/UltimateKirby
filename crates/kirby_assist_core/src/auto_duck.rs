//! Geometry, threat aggregation, timing and input policy for automatic ducking.
//! Body bounds are calibration estimates, not native Kirby hurtboxes.
//!
//! Prima's Ultimate Kirby overview explicitly names Falcon Punch as duckable:
//! https://primagames.com/eguides/prima-games-8-eguide/kirby/overview
//! Normal grounded Punch is active 53–57 (Ultimate Frame Data). Reverse, aerial,
//! copied and modified moves are deliberately not inferred from that entry.

use crate::autopilot_guard::{GuardClock, GuardHold, Hitbox};

#[path = "fighters/captain_falcon/punch_duck.rs"]
mod falcon_punch;
pub use falcon_punch::{candidate, FALCON_PUNCH_MOTION};

const MIN_START_LEAD: f32 = 8.0;
const CONFIRM_LEAD: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contact {
    None,
    Duck(f32),
    Shield,
}

pub const CROUCH_TOP: f32 = 4.5;
pub const STAND_TOP: f32 = 12.0;
pub const BODY_HALF_WIDTH: f32 = 4.0;
pub const LOOKAHEAD: usize = 6;

/// Swept capsule contact against estimated standing and crouching bounds.
pub fn contact(position: [f32; 3], velocity: [f32; 2], hit: Hitbox) -> Contact {
    if !position
        .iter()
        .chain(velocity.iter())
        .chain(hit.start.iter())
        .chain(hit.end.iter())
        .chain(hit.velocity.iter())
        .chain([&hit.radius])
        .all(|v| v.is_finite())
        || !(0.0..=100.0).contains(&hit.radius)
        || hit.radius == 0.0
    {
        return Contact::None;
    }
    let lo = std::array::from_fn::<_, 3, _>(|i| hit.start[i].min(hit.end[i]) - hit.radius);
    let hi = std::array::from_fn::<_, 3, _>(|i| hit.start[i].max(hit.end[i]) + hit.radius);
    if hi[2] < position[2] - BODY_HALF_WIDTH || lo[2] > position[2] + BODY_HALF_WIDTH {
        return Contact::None;
    }
    let mut duck_at: Option<f32> = None;
    for frame in 0..=LOOKAHEAD {
        let t = frame as f32;
        let next = (frame + 1).min(LOOKAHEAD) as f32;
        let dx = hit.velocity[0] - velocity[0];
        let dy = hit.velocity[1] - velocity[1];
        let min_x = lo[0] + (dx * t).min(dx * next);
        let max_x = hi[0] + (dx * t).max(dx * next);
        let min_y = lo[1] + (dy * t).min(dy * next);
        let max_y = hi[1] + (dy * t).max(dy * next);
        if max_x < position[0] - BODY_HALF_WIDTH
            || min_x > position[0] + BODY_HALF_WIDTH
            || max_y < position[1]
            || min_y > position[1] + STAND_TOP
        {
            continue;
        }
        if min_y <= position[1] + CROUCH_TOP {
            return Contact::Shield;
        }
        duck_at.get_or_insert(t);
    }
    duck_at.map_or(Contact::None, Contact::Duck)
}

#[derive(Clone, Copy)]
pub struct DuckContext {
    pub clock: GuardClock,
    pub can_crouch: bool,
    pub crouched: bool,
    pub can_shield: bool,
}

/// Automatic crouch/shield ownership, separate from ledge autopilot guarding.
#[derive(Default)]
pub struct DuckDefense {
    hold: DuckHold,
    shield: GuardHold,
    shielding: bool,
}
impl DuckDefense {
    pub fn sample(
        &mut self,
        enabled: bool,
        c: DuckContext,
        threats: DefenseThreat,
    ) -> crate::gameplay::ControllerOutput {
        let ducking = self.hold.sample(
            enabled && c.can_crouch && !self.shielding,
            !c.clock.stopped,
            c.crouched,
            threats,
        );
        self.shielding = self.shield.sample(
            enabled && c.can_shield && !ducking,
            threats.needs_defense(),
            c.clock,
        );
        if !ducking && !self.shielding {
            return crate::gameplay::ControllerOutput::default();
        }
        crate::gameplay::ControllerOutput {
            consume_trigger: false,
            owned: crate::native_input::ASSIST_BUTTONS,
            requested: if self.shielding {
                crate::native_input::MAPPER_ZR
            } else {
                0
            },
            stick: Some([0.0, if ducking { -1.0 } else { 0.0 }]),
            native_short_hop: false,
        }
    }
}

pub fn environment_allows_duck(
    own_scale: f32,
    enemy_scale: f32,
    own_passable: bool,
    enemy_passable: bool,
) -> bool {
    (0.99..=1.01).contains(&own_scale)
        && (0.99..=1.01).contains(&enemy_scale)
        && !own_passable
        && !enemy_passable
}

#[derive(Clone, Copy, Debug)]
pub struct DuckEnemy {
    pub id: u32,
    pub kind: i32,
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub facing: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct DuckThreat {
    id: u32,
    frame: f32,
    lead: f32,
    geometry: bool,
}

/// Live active-hitbox geometry, rather than a move-name allowlist.
pub fn geometry_threat(id: u32, frame: f32, lead: f32) -> DuckThreat {
    DuckThreat {
        id,
        frame,
        lead,
        geometry: true,
    }
}

/// Fixed-size threat summary; any non-duckable warning makes shielding win.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefenseThreat {
    pub shield: bool,
    pub duck: Option<DuckThreat>,
}
impl DefenseThreat {
    pub fn add_duck(&mut self, threat: DuckThreat) {
        if self
            .duck
            .is_some_and(|previous| previous.geometry && threat.geometry)
        {
            if self
                .duck
                .is_some_and(|previous| threat.lead < previous.lead)
            {
                self.duck = Some(threat);
            }
            return;
        }
        if self.duck.is_some() {
            self.shield = true;
        } else {
            self.duck = Some(threat);
        }
    }

    pub fn needs_defense(self) -> bool {
        self.shield || self.duck.is_some()
    }
}

#[derive(Default)]
pub struct DuckHold {
    previous: Option<DuckThreat>,
}
impl DuckHold {
    /// Start early, confirm crouch close to impact, and fall back when too late.
    pub fn sample(
        &mut self,
        eligible: bool,
        can_start: bool,
        crouched: bool,
        threats: DefenseThreat,
    ) -> bool {
        let Some(threat) = threats.duck.filter(|_| eligible && !threats.shield) else {
            self.previous = None;
            return false;
        };
        if threat.geometry {
            if !can_start && self.previous.is_none() {
                return false;
            }
            self.previous = Some(threat);
            return true;
        }
        if self
            .previous
            .is_some_and(|previous| previous.id == threat.id && threat.frame < previous.frame)
        {
            self.previous = None;
            return false;
        }
        let continuing = self
            .previous
            .is_some_and(|previous| previous.id == threat.id && threat.frame >= previous.frame);
        if (!continuing && (!can_start || threat.lead < MIN_START_LEAD))
            || (threat.lead <= CONFIRM_LEAD && !crouched)
        {
            self.previous = None;
            return false;
        }
        self.previous = Some(threat);
        true
    }
}
