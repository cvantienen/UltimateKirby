//! Ground-only preparation at the nearest edge of Kirby's current floor.
//!
//! The adapter supplies read-only support probes. `nearest_ground_ledge` finds
//! a candidate once; `LedgePosition` remembers it and proposes a damped approach.
//! This prepares a stance. The roll response in ledge_trap/roll.rs has its own
//! committed destination and is responsible for timing the down-smash punish.
use crate::ledge_trap::{LedgeContext, LedgeOutput, LedgeSettings, LEDGE_BUTTONS};
use crate::ledge_trump::floor_edge_with_limit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeSpot {
    pub edge: [f32; 2],
    pub inward: f32,
}

/// Reuse the existing trump's short support probes at the grounded floor height.
/// No vertical stage-body scan, floor-normal FFI, or airborne geometry queries.
pub fn nearest_ground_ledge(
    position: [f32; 2],
    facing: f32,
    mut supported: impl FnMut([f32; 2]) -> bool,
    stand_distance: f32,
) -> Option<LedgeSpot> {
    let edge = floor_edge_with_limit(position, facing, 400, &mut supported)?;
    let inward = (position[0] - edge[0]).signum();
    if inward == 0.0 || !stand_distance.is_finite() || stand_distance < 2.0 {
        return None;
    }
    let destination = [edge[0] + inward * stand_distance, edge[1]];
    supported(destination).then_some(LedgeSpot { edge, inward })
}

#[derive(Default)]
pub struct LedgePosition {
    spot: Option<LedgeSpot>,
    settled: bool,
    last_tick: Option<i64>,
    output: LedgeOutput,
}
impl LedgePosition {
    pub fn spot(&self) -> Option<LedgeSpot> {
        self.spot
    }
    pub fn lock(&mut self, spot: Option<LedgeSpot>) {
        if self.spot.is_none() {
            self.spot = spot
                .filter(|spot| spot.edge.iter().all(|v| v.is_finite()) && spot.inward.abs() == 1.0);
        }
    }
    pub fn action_name(&self) -> &'static str {
        if self.spot.is_none() {
            "seek_ground_ledge"
        } else if self.settled {
            "ready_at_ledge"
        } else {
            "prepare_ground_ledge_position"
        }
    }
    pub fn sample(&mut self, tick: i64, ctx: LedgeContext, settings: LedgeSettings) -> LedgeOutput {
        // Air always yields, even on a repeated mapper tick. Landing starts a
        // fresh floor selection rather than retaining a destination from flight.
        if ctx.situation != 0 {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
        }
        // Eligibility and cancellation must win over a duplicate-tick cache.
        if !settings.enabled
            || !ctx.held
            || !settings.auto_position
            || !ctx.can_act
            || ctx.stopped
            || !ctx.gameplay
            || !ctx
                .position
                .iter()
                .chain(ctx.velocity.iter())
                .all(|v| v.is_finite())
            || !ctx.facing.is_finite()
        {
            self.output = LedgeOutput::default();
            return self.output;
        }
        if self.last_tick == Some(tick) {
            return self.output;
        }
        self.last_tick = Some(tick);
        let mut out = LedgeOutput::default();
        let Some(spot) = self.spot else {
            self.output = out;
            return out;
        };
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        let dx = spot.edge[0] + spot.inward * settings.stand_distance - ctx.position[0];
        let predicted = dx - ctx.velocity[0] * 3.0;
        if self.settled && (dx.abs() > 1.0 || ctx.velocity[0].abs() > 0.2) {
            self.settled = false;
        }
        if !self.settled && dx.abs() <= 0.5 && ctx.velocity[0].abs() <= 0.12 {
            self.settled = true;
        }
        if !self.settled && predicted.abs() > 0.35 && predicted.signum() == dx.signum() {
            let strength = if dx.abs() > 12.0 {
                settings.move_strength
            } else {
                (predicted.abs() * 0.06).clamp(0.2, 0.4)
            };
            if ctx.safe_ground[usize::from(dx > 0.0)] {
                out.stick = Some([dx.signum() * strength, 0.0]);
            }
        } else if self.settled
            && ctx.facing * -spot.inward < 0.0
            && ctx.safe_ground[usize::from(-spot.inward > 0.0)]
        {
            out.stick = Some([-spot.inward * 0.3, 0.0]);
        }
        self.output = out;
        out
    }
}

#[cfg(test)]
mod tests;
