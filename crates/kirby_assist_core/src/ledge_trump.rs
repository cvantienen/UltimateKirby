//! One-shot run-off ledge trump. Inputs only; native ledge catch ends ownership.
use crate::ledge_trap::{LedgeContext, LedgeOutput, CLIFF_CATCH_MOVE, CLIFF_WAIT, LEDGE_BUTTONS};

// FIGHTER_STATUS_KIND_FALL in the native lua_const bindings.
const FALL: i32 = 0xe;

/// Find the nearest edge of the current floor using read-only support probes.
/// This detects collision edges, not native grabbability (platforms can differ).
/// Scan only at activation; the attempt retains the selected edge afterward.
pub fn nearest_floor_edge(
    position: [f32; 2],
    facing: f32,
    supported: impl FnMut([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    floor_edge_with_limit(position, facing, 100, supported)
}

/// Shared short floor probes; the trap may search farther than a trump run.
pub fn floor_edge_with_limit(
    position: [f32; 2],
    facing: f32,
    limit: u16,
    mut supported: impl FnMut([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    if !position.iter().all(|v| v.is_finite()) || !facing.is_finite() || !supported(position) {
        return None;
    }
    let preferred = if facing < 0.0 { -1.0 } else { 1.0 };
    let mut nearest: Option<[f32; 2]> = None;
    for direction in [preferred, -preferred] {
        for step in 1..=limit {
            let distance = step as f32;
            let probe = [position[0] + direction * distance, position[1]];
            if supported(probe) {
                continue;
            }
            let mut inside = distance - 1.0;
            let mut outside = distance;
            for _ in 0..6 {
                let midpoint = (inside + outside) * 0.5;
                if supported([position[0] + direction * midpoint, position[1]]) {
                    inside = midpoint;
                } else {
                    outside = midpoint;
                }
            }
            let edge = [
                position[0] + direction * (inside + outside) * 0.5,
                position[1],
            ];
            if nearest
                .is_none_or(|old| (edge[0] - position[0]).abs() < (old[0] - position[0]).abs())
            {
                nearest = Some(edge);
            }
            break;
        }
    }
    nearest
}

#[derive(Clone, Copy)]
enum Phase {
    Approach,
    Down { status: i32, frame: f32 },
    Return,
}

#[derive(Clone, Copy)]
struct Attempt {
    anchor: [f32; 2],
    outward: f32,
    approach_limit: f32,
    phase: Phase,
    updates: u16,
}

#[derive(Default)]
pub struct LedgeTrump {
    held: bool,
    attempt: Option<Attempt>,
    last_tick: Option<i64>,
    last_context: Option<LedgeContext>,
    output: LedgeOutput,
}

impl LedgeTrump {
    /// Keep sampling an in-flight tool after its physical trigger is released.
    pub fn active(&self) -> bool {
        self.attempt.is_some()
    }

    pub fn needs_edge(&self, down: bool) -> bool {
        down && !self.held && self.attempt.is_none()
    }

    pub fn action_name(&self) -> &'static str {
        match self.attempt.map(|attempt| attempt.phase) {
            None => "idle_or_finished",
            Some(Phase::Approach) => "run_to_edge",
            Some(Phase::Down { .. }) => "down_after_runoff",
            Some(Phase::Return) => "inward_to_catch",
        }
    }

    pub fn sample(
        &mut self,
        enabled: bool,
        tick: i64,
        down: bool,
        ctx: LedgeContext,
        edge: Option<[f32; 2]>,
    ) -> LedgeOutput {
        if !enabled || !ctx.gameplay {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
        }
        if self.last_tick == Some(tick) && self.last_context == Some(ctx) && self.held == down {
            return self.output;
        }
        self.last_tick = Some(tick);
        let pressed = down && !self.held;
        self.held = down;
        self.output = LedgeOutput::default();
        if pressed && self.attempt.is_none() && ctx.can_act && !ctx.stopped {
            if let Some(anchor) = edge {
                let dx = anchor[0] - ctx.position[0];
                if anchor.iter().all(|v| v.is_finite())
                    && ctx.position.iter().all(|v| v.is_finite())
                    && (0.0..=100.0).contains(&dx.abs())
                    && dx != 0.0
                    && (anchor[1] - ctx.position[1]).abs() <= 3.0
                    && ctx.situation == 0
                {
                    self.attempt = Some(Attempt {
                        anchor,
                        outward: dx.signum(),
                        approach_limit: dx.abs() + 4.0,
                        phase: Phase::Approach,
                        updates: 0,
                    });
                }
            }
        }
        let Some(mut attempt) = self.attempt else {
            return self.output;
        };
        // No held trigger can re-run the macro after catch, cancellation or timeout.
        if (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&ctx.status) {
            self.attempt = None;
            return self.output;
        }
        let changed = self.last_context != Some(ctx);
        self.last_context = Some(ctx);
        if changed && !ctx.stopped {
            attempt.updates += 1;
        }
        let bounded = ctx.position.iter().all(|v| v.is_finite())
            && ctx.motion_frame.is_finite()
            && ctx.motion_frame >= 0.0
            && (ctx.position[0] - attempt.anchor[0]).abs()
                <= if matches!(attempt.phase, Phase::Approach) {
                    attempt.approach_limit
                } else {
                    12.0
                }
            && (ctx.position[1] - attempt.anchor[1]).abs() <= 16.0
            && attempt.updates <= 120;
        // Native stop makes can_act false even during an otherwise valid run.
        // Freeze that attempt; damage/attacks still cancel, including in stop.
        let supported_state =
            crate::manual_controls::turbo_movement_allowed(ctx.status, ctx.situation, false)
                || ctx.situation == 2 && ctx.status == FALL;
        if !bounded
            || !supported_state
            || !ctx.stopped
                && !(ctx.can_act && ctx.situation == 0 || ctx.situation == 2 && ctx.status == FALL)
        {
            self.attempt = None;
            return self.output;
        }
        if ctx.stopped {
            self.attempt = Some(attempt);
            return self.output;
        }
        let stick = match attempt.phase {
            Phase::Approach if ctx.situation == 0 => {
                // If stage motion invalidated the chosen boundary, do not run
                // farther outward across the stage in search of a different one.
                if (attempt.anchor[0] - ctx.position[0]) * attempt.outward < -3.0 {
                    self.attempt = None;
                    return self.output;
                }
                [attempt.outward, 0.0]
            }
            Phase::Approach => {
                attempt.phase = Phase::Down {
                    status: ctx.status,
                    frame: ctx.motion_frame,
                };
                [0.0, -1.0]
            }
            Phase::Down { status, frame } if ctx.situation == 2 => {
                // Mapper polls are not frames. Hold Down until the fighter's
                // native animation advances, then stop Down so ledge catch is legal.
                if ctx.status == status && ctx.motion_frame <= frame {
                    [0.0, -1.0]
                } else {
                    attempt.phase = Phase::Return;
                    [-attempt.outward, 0.0]
                }
            }
            Phase::Return if ctx.situation == 2 => [-attempt.outward, 0.0],
            _ => {
                self.attempt = None;
                return self.output;
            }
        };
        self.attempt = Some(attempt);
        self.output = LedgeOutput {
            stick: Some(stick),
            owned: LEDGE_BUTTONS,
            ..LedgeOutput::default()
        };
        self.output
    }
}

#[cfg(test)]
mod tests;
