//! Legacy grounded Falcon Punch threat candidate; live defense uses geometry.
use super::{DuckEnemy, DuckThreat};

pub const FALCON_PUNCH_MOTION: u64 = 0x915c5de42;
const FIRST: f32 = 53.0;
const LAST: f32 = 57.0;
const START_LEAD: f32 = 16.0;

pub fn candidate(own: [f32; 2], velocity: [f32; 2], enemy: DuckEnemy) -> Option<DuckThreat> {
    if enemy.kind != crate::fighter_kinds::CAPTAIN_FALCON
        || enemy.status != crate::native_states::status::SPECIAL_N
        || enemy.situation != crate::native_states::situation::GROUND
        || enemy.motion != FALCON_PUNCH_MOTION
        || !own
            .iter()
            .chain(velocity.iter())
            .chain(enemy.position.iter())
            .chain(enemy.velocity.iter())
            .chain([&enemy.frame, &enemy.rate, &enemy.facing])
            .all(|v| v.is_finite())
        || enemy.rate <= 0.0
        || enemy.frame < 0.0
        || enemy.frame > LAST
    {
        return None;
    }
    let dx = own[0] - enemy.position[0];
    let lead = (FIRST - enemy.frame) / enemy.rate;
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
