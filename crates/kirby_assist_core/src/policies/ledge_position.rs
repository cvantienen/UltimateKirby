//! Ground-only preparation at the nearest edge of Kirby's current floor.
//!
//! The adapter supplies read-only support probes. `nearest_ground_ledge` finds
//! a candidate once; `LedgePosition` remembers it and proposes a damped approach.
//! This prepares a stance for the selected response. Ordered response inputs
//! are executed separately by `ledge_trap/file_timeline.rs`.
use crate::ledge_trap::finite2;
use crate::ledge_trump::floor_edge_with_limit;
use crate::{
    gameplay::{ControllerOutput, GameplayContext},
    native_input::ASSIST_BUTTONS,
    settings::AssistSettings,
};

const MAX_GROUNDED_LEDGE_SCAN_DISTANCE: u16 = 400;

fn eligible_for_positioning(settings: AssistSettings, ctx: GameplayContext) -> bool {
    settings.enabled
        && ctx.held
        && settings.auto_position
        && ctx.can_act
        && !ctx.stopped
        && ctx.gameplay
        && finite2(ctx.position)
        && finite2(ctx.velocity)
        && ctx.facing.is_finite()
}

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
    // Validate before the scan: every probe is a native ray check.
    if !stand_distance.is_finite() || stand_distance < 2.0 {
        return None;
    }
    let edge = floor_edge_with_limit(
        position,
        facing,
        MAX_GROUNDED_LEDGE_SCAN_DISTANCE,
        &mut supported,
    )?;
    let inward = (position[0] - edge[0]).signum();
    let destination = [edge[0] + inward * stand_distance, edge[1]];
    supported(destination).then_some(LedgeSpot { edge, inward })
}

#[derive(Default)]
pub struct LedgePosition {
    spot: Option<LedgeSpot>,
    settled: bool,
    last_tick: Option<i64>,
    output: ControllerOutput,
}
impl LedgePosition {
    pub fn spot(&self) -> Option<LedgeSpot> {
        self.spot
    }
    pub fn lock(&mut self, spot: Option<LedgeSpot>) {
        if self.spot.is_none() {
            self.spot = spot.filter(|spot| finite2(spot.edge) && spot.inward.abs() == 1.0);
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
    pub fn sample(
        &mut self,
        tick: i64,
        ctx: GameplayContext,
        settings: AssistSettings,
    ) -> ControllerOutput {
        // Air always yields, even on a repeated mapper tick. Landing starts a
        // fresh floor selection rather than retaining a destination from flight.
        if ctx.situation != crate::native_states::situation::GROUND {
            *self = Self::default();
            return ControllerOutput::default();
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
        }
        // Eligibility and cancellation must win over a duplicate-tick cache.
        if !eligible_for_positioning(settings, ctx) {
            self.output = ControllerOutput::default();
            return self.output;
        }
        if self.last_tick == Some(tick) {
            return self.output;
        }
        self.last_tick = Some(tick);
        let mut out = ControllerOutput::default();
        let Some(spot) = self.spot else {
            self.output = out;
            return out;
        };
        out.owned = ASSIST_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        let dx = spot.edge[0] + spot.inward * settings.stand_distance - ctx.position[0];
        let predicted = dx - ctx.velocity[0] * 3.0;
        let stance_drifted = dx.abs() > 1.0 || ctx.velocity[0].abs() > 0.2;
        let at_stance = dx.abs() <= 0.5 && ctx.velocity[0].abs() <= 0.12;
        // Settling needs the tight stance; leaving it needs a clear drift.
        self.settled = if self.settled {
            !stance_drifted
        } else {
            at_stance
        };
        let should_approach =
            !self.settled && predicted.abs() > 0.35 && predicted.signum() == dx.signum();
        let should_face_edge = self.settled
            && ctx.facing * -spot.inward < 0.0
            && ctx.safe_ground[usize::from(-spot.inward > 0.0)];
        if should_approach {
            let strength = if dx.abs() > 12.0 {
                settings.move_strength
            } else {
                (predicted.abs() * 0.06).clamp(0.2, 0.4)
            };
            if ctx.safe_ground[usize::from(dx > 0.0)] {
                out.stick = Some([dx.signum() * strength, 0.0]);
            }
        } else if should_face_edge {
            out.stick = Some([-spot.inward * 0.3, 0.0]);
        }
        self.output = out;
        out
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ledge_position.rs"]
mod tests;
