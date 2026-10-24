//! Held-Y Kirby duck/shield ownership. Live observations use duck_geometry.
//! The Falcon-only candidate below is retained for legacy policy regressions,
//! not as a restriction on the current live reader.
//!
//! Prima's Ultimate Kirby overview explicitly names Falcon Punch as duckable:
//! https://primagames.com/eguides/prima-games-8-eguide/kirby/overview
//! Normal grounded Punch is active 53–57 (Ultimate Frame Data). Reverse, aerial,
//! copied and modified moves are deliberately not inferred from this entry.

pub const FALCON_PUNCH_MOTION: u64 = 0x915c5de42; // Hash40("special_n")
const FIRST: f32 = 53.0;
const LAST: f32 = 57.0;
const START_LEAD: f32 = 16.0;
const MIN_START_LEAD: f32 = 8.0;
const CONFIRM_LEAD: f32 = 3.0;

#[derive(Clone, Copy)]
pub struct DuckContext {
    pub clock: super::GuardClock,
    pub can_crouch: bool,
    pub crouched: bool,
    pub can_shield: bool,
}

/// Independent held-Y defense. L's shield safeguard uses separate state.
#[derive(Default)]
pub struct DuckDefense {
    hold: DuckHold,
    shield: super::GuardHold,
    shielding: bool,
}
impl DuckDefense {
    pub fn sample(
        &mut self,
        enabled: bool,
        c: DuckContext,
        threats: DefenseThreat,
    ) -> crate::ledge_trap::LedgeOutput {
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
            return crate::ledge_trap::LedgeOutput::default();
        }
        crate::ledge_trap::LedgeOutput {
            consume_trigger: false,
            owned: crate::ledge_trap::LEDGE_BUTTONS,
            requested: if self.shielding {
                crate::native_input::MAPPER_ZR
            } else {
                0
            },
            stick: Some([0.0, if ducking { -1.0 } else { 0.0 }]),
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

/// Cheap spatial/motion filter, called inside the existing opponent pass.
/// Live callers additionally require normal size and solid, non-passable floor.
pub fn candidate(own: [f32; 2], velocity: [f32; 2], enemy: DuckEnemy) -> Option<DuckThreat> {
    if enemy.kind != 0xb // FIGHTER_KIND_CAPTAIN
        || enemy.status != 0x1dc // FIGHTER_STATUS_KIND_SPECIAL_N
        || enemy.situation != 0
        || enemy.motion != FALCON_PUNCH_MOTION
        || !own.iter().chain(velocity.iter()).chain(enemy.position.iter())
            .chain(enemy.velocity.iter()).chain([&enemy.frame, &enemy.rate, &enemy.facing])
            .all(|v| v.is_finite())
        || enemy.rate <= 0.0
        || enemy.frame < 0.0
        || enemy.frame > LAST
    {
        return None;
    }
    let dx = own[0] - enemy.position[0];
    let lead = (FIRST - enemy.frame) / enemy.rate;
    // Do not infer clearance on slopes, while approaching, at body overlap or
    // while the attacker faces away. These are deliberately narrow estimates.
    if !(6.0..=24.0).contains(&dx.abs())
        || (own[1] - enemy.position[1]).abs() > 0.25
        || !(0.99..=1.01).contains(&enemy.facing.abs())
        || dx * enemy.facing <= 0.0
        || velocity[0].abs() > 0.1
        || enemy.velocity[0].abs() > 0.1
        || velocity[1].abs() > 0.05
        || enemy.velocity[1].abs() > 0.05
        || !lead.is_finite()
        || lead > START_LEAD
    {
        return None;
    }
    Some(DuckThreat {
        id: enemy.id,
        frame: enemy.frame,
        lead,
        geometry: false,
    })
}

/// One fixed-size result for the existing roster pass. Any other warning (even
/// a second duckable attacker) wins over ducking; roster order cannot hide it.
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
    /// Start at least eight native game frames early. Require actual SQUAT_WAIT
    /// three frames before impact; rejected/late crouches fall back to shield.
    /// Repeated polls/hitstop hold the same proposal, without consuming a timer.
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
            // Attempt down immediately for every detected high hitbox; a late
            // first observation may still lose to crouch startup/input delay.
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
