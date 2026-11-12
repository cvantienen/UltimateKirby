//! One-shot run-off ledge trump. Inputs only; native ledge catch ends ownership.
use crate::{
    gameplay::{ControllerOutput, GameplayContext},
    ledge_trap::{finite2, CLIFF_CATCH_MOVE, CLIFF_WAIT},
    native_input::ASSIST_BUTTONS,
    native_states::{situation, status::FALL},
};

/// Sets the default value for trump edge scan limit.
/// Holds the DEFAULT TRUMP EDGE SCAN LIMIT value.
const DEFAULT_TRUMP_EDGE_SCAN_LIMIT: u16 = 100;

/// Find the nearest edge of the current floor using read-only support probes.
/// This detects collision edges, not native grabbability (platforms can differ).
/// Scan only at activation; the attempt retains the selected edge afterward.
pub fn nearest_floor_edge(
    position: [f32; 2],
    facing: f32,
    supported: impl FnMut([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    floor_edge_with_limit(position, facing, DEFAULT_TRUMP_EDGE_SCAN_LIMIT, supported)
}

/// Shared short floor probes; the trap may search farther than a trump run.
pub fn floor_edge_with_limit(
    position: [f32; 2],
    facing: f32,
    limit: u16,
    mut supported: impl FnMut([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    if !finite2(position) || !facing.is_finite() || !supported(position) {
        return None;
    }
    let preferred = if facing < 0.0 { -1.0 } else { 1.0 };
    let mut nearest: Option<[f32; 2]> = None;
    for direction in [preferred, -preferred] {
        for step in 1..=limit {
            let distance = step as f32;
            // A boundary in this next unit interval cannot beat the edge we
            // already found. Preserve unit-resolution gap detection and facing
            // tie-breaking; only omit probes that cannot change the result.
            if nearest.is_some_and(|old| distance - 1.0 >= (old[0] - position[0]).abs()) {
                break;
            }
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

/// Available `Phase` choices for the Smash assistance menu.
#[derive(Clone, Copy)]
enum Phase {
/// `Approach` option in the `Phase` Smash policy state.
    Approach,
/// `Down` option in the `Phase` Smash policy state.
    Down { status: i32, frame: f32 },
/// `Return` option in the `Phase` Smash policy state.
    Return,
}

/// Runtime Smash data carried by `Attempt`.
#[derive(Clone, Copy)]
struct Attempt {
/// Runtime anchor value used by Attempt..
    anchor: [f32; 2],
/// Runtime outward value used by Attempt..
    outward: f32,
/// Runtime approach limit value used by Attempt..
    approach_limit: f32,
/// Runtime phase value used by Attempt..
    phase: Phase,
/// Runtime updates value used by Attempt..
    updates: u16,
}
impl Attempt {
    /// A usable same-floor edge within trump reach, from solid ground.
    fn start(anchor: [f32; 2], ctx: GameplayContext) -> Option<Self> {
        let dx = anchor[0] - ctx.position[0];
        (finite2(anchor)
            && finite2(ctx.position)
            && dx.abs() <= f32::from(DEFAULT_TRUMP_EDGE_SCAN_LIMIT)
            && dx != 0.0
            && (anchor[1] - ctx.position[1]).abs() <= 3.0
            && ctx.situation == situation::GROUND)
            .then(|| Self {
                anchor,
                outward: dx.signum(),
                approach_limit: dx.abs() + 4.0,
                phase: Phase::Approach,
                updates: 0,
            })
    }
    /// Whether the fighter is still on the macro's path, in a state it can run.
    fn viable(&self, ctx: GameplayContext) -> bool {
        let air_fall = ctx.situation == situation::AIR && ctx.status == FALL;
        let reach = if matches!(self.phase, Phase::Approach) {
            self.approach_limit
        } else {
            12.0
        };
        let bounded = finite2(ctx.position)
            && ctx.motion_frame.is_finite()
            && ctx.motion_frame >= 0.0
            && (ctx.position[0] - self.anchor[0]).abs() <= reach
            && (ctx.position[1] - self.anchor[1]).abs() <= 16.0
            && self.updates <= 120;
        let supported_state =
            crate::manual_controls::turbo_movement_allowed(ctx.status, ctx.situation, false)
                || air_fall;
        // Native stop makes can_act false even during an otherwise valid run.
        // Freeze that attempt; damage/attacks still cancel, including in stop.
        bounded
            && supported_state
            && (ctx.stopped || ctx.can_act && ctx.situation == situation::GROUND || air_fall)
    }
    /// Left-stick request for this sample, advancing the phase; `None` ends the attempt.
    fn drive(&mut self, ctx: GameplayContext) -> Option<[f32; 2]> {
        match self.phase {
            Phase::Approach if ctx.situation == situation::GROUND => {
                let gap = (self.anchor[0] - ctx.position[0]) * self.outward;
                // If stage motion invalidated the chosen boundary, do not run
                // farther outward across the stage in search of a different one.
                if gap < -3.0 {
                    return None;
                }
                // Stop sprinting before the boundary. Excess run momentum
                // used to carry fighters beyond catch range during Down.
                Some([self.outward * if gap <= 8.0 { 0.55 } else { 1.0 }, 0.0])
            }
            Phase::Approach => {
                self.phase = Phase::Down {
                    status: ctx.status,
                    frame: ctx.motion_frame,
                };
                Some([-self.outward, -1.0])
            }
            Phase::Down { status, frame } if ctx.situation == situation::AIR => {
                // Mapper polls are not frames. Hold Down until the fighter's
                // native animation advances, then stop Down so ledge catch is legal.
                if ctx.status == status && ctx.motion_frame <= frame {
                    Some([-self.outward, -1.0])
                } else {
                    self.phase = Phase::Return;
                    Some([-self.outward, 0.0])
                }
            }
            Phase::Return if ctx.situation == situation::AIR => Some([-self.outward, 0.0]),
            _ => None,
        }
    }
}

/// Runtime Smash data carried by `LedgeTrump`.
#[derive(Default)]
pub struct LedgeTrump {
/// Runtime held value used by LedgeTrump..
    held: bool,
/// Runtime attempt value used by LedgeTrump..
    attempt: Option<Attempt>,
/// Runtime last tick value used by LedgeTrump..
    last_tick: Option<i64>,
/// Runtime last context value used by LedgeTrump..
    last_context: Option<GameplayContext>,
/// controller input output.
    output: ControllerOutput,
}

impl LedgeTrump {
    /// Cancels the pending.
    pub fn cancel_pending(&mut self) {
        self.attempt = None;
        self.output = ControllerOutput::default(); // Keep held-Down history.
    }
    /// Keep sampling an in-flight tool after its physical trigger is released.
    pub fn active(&self) -> bool {
        self.attempt.is_some()
    }

    /// Determines whether edge.
    pub fn needs_edge(&self, down: bool) -> bool {
        down && !self.held && self.attempt.is_none()
    }

    /// Returns the `action_name` result.
    pub fn action_name(&self) -> &'static str {
        match self.attempt.map(|attempt| attempt.phase) {
            None => "idle_or_finished",
            Some(Phase::Approach) => "run_to_edge",
            Some(Phase::Down { .. }) => "down_after_runoff",
            Some(Phase::Return) => "inward_to_catch",
        }
    }

    /// Updates the toggle state from the current button sample and match state.
    pub fn sample(
        &mut self,
        enabled: bool,
        tick: i64,
        down: bool,
        ctx: GameplayContext,
        edge: Option<[f32; 2]>,
    ) -> ControllerOutput {
        if !enabled || !ctx.gameplay {
            *self = Self::default();
            return ControllerOutput::default();
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
        self.output = ControllerOutput::default();
        if pressed && self.attempt.is_none() && ctx.can_act && !ctx.stopped {
            self.attempt = edge.and_then(|anchor| Attempt::start(anchor, ctx));
        }
        let Some(mut attempt) = self.attempt else {
            return self.output;
        };
        // No held trigger can re-run the macro after catch, cancellation or timeout.
        if (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&ctx.status) {
            self.attempt = None;
            return self.output;
        }
        let changed = self.last_context.is_none_or(|last| {
            last.status != ctx.status
                || last.motion_kind != ctx.motion_kind
                || last.motion_frame != ctx.motion_frame
        });
        self.last_context = Some(ctx);
        if changed && !ctx.stopped {
            attempt.updates += 1;
        }
        if !attempt.viable(ctx) {
            self.attempt = None;
            return self.output;
        }
        if ctx.stopped {
            self.attempt = Some(attempt);
            return self.output;
        }
        let Some(stick) = attempt.drive(ctx) else {
            self.attempt = None;
            return self.output;
        };
        self.attempt = Some(attempt);
        self.output = ControllerOutput {
            stick: Some(stick),
            owned: ASSIST_BUTTONS
                | if stick[0].abs() < 1.0 {
                    crate::manual_controls::LEFT_STICK_DIRECTIONS
                } else {
                    0
                },
            ..ControllerOutput::default()
        };
        self.output
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ledge_trump.rs"]
mod tests;
