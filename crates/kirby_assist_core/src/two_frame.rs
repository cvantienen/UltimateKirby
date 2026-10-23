//! Predictive recovery cover using ordinary down+A inputs. This is a forecast,
//! not a native ledge-snap timer: recovery steering, hurtboxes and capture boxes
//! vary. Never treat the forecast as proof that every recovery is two-frameable.
use crate::ledge_position::LedgeSpot;
use crate::ledge_trap::{LedgeContext, LedgeOutput, LedgeSettings, LEDGE_BUTTONS};
use crate::native_input::MAPPER_A;

/// A small copied observation; the Switch adapter retains no opponent pointers.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryTarget {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
    pub situation: i32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub stopped: bool,
    pub vulnerable: bool,
}

// Compiled projection tuning, in world units. This band estimates a low rising
// approach near the ledge; it is NOT a universal character's real capture box.
// Keep it separate from attack startup, which comes from the fighter profile.
const OUTSIDE_REACH: f32 = 6.0;
const ROOT_BELOW_EDGE: f32 = 6.0;
const MAX_FORECAST: f32 = 32.0;
const MAX_STAND_DISTANCE: f32 = 16.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Waiting,
    Release,
    Press,
    Done,
}

/// Fixed-size, one-request-per-approach controller. Animation time, rather than
/// controller polls or wall-clock time, advances release/hold/expiry durations.
#[derive(Default)]
pub struct TwoFrame {
    target: Option<u32>,
    phase: Phase,
    previous: Option<(i32, u64, f32, f32)>,
    age: f32,
    phase_at: f32,
    missing_at: Option<f32>,
    output: LedgeOutput,
}
impl TwoFrame {
    pub fn target(&self) -> Option<u32> {
        if self.phase == Phase::Waiting {
            None
        } else {
            self.target
        }
    }
    pub fn action_name(&self) -> &'static str {
        match self.phase {
            Phase::Waiting => "two_frame_watch_recovery",
            Phase::Release => "two_frame_prepare_input",
            Phase::Press => "two_frame_attack_request",
            Phase::Done => "two_frame_handoff",
        }
    }
    pub fn sample(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        spot: Option<LedgeSpot>,
        enemy: Option<RecoveryTarget>,
    ) -> LedgeOutput {
        if !settings.enabled || !settings.two_frame || !ctx.gameplay || !ctx.held {
            *self = Self::default();
            return LedgeOutput::default();
        }
        // Higher-priority interrupts/airborne movement always hand back input.
        if ctx.situation != 0 || (!ctx.can_control && !ctx.stopped) {
            self.phase = Phase::Done;
            self.output = LedgeOutput::default();
            return self.output;
        }
        if ctx.stopped {
            return self.output;
        }
        let Some(delta) = self.advance(ctx) else {
            self.phase = Phase::Done;
            self.output = LedgeOutput::default();
            return self.output;
        };
        if delta == 0.0 {
            return self.output;
        }
        self.output = LedgeOutput::default();
        // A received smash-start/charge/attack releases A immediately. Holding A
        // through this state would charge and destroy the scheduled hit timing.
        if self.phase == Phase::Press {
            let accepted = if settings.two_frame_down_tilt {
                ctx.status == 0x2c
            } else {
                matches!(ctx.status, 0x30..=0x32)
            };
            if accepted || self.age - self.phase_at >= ctx.input_delay.clamp(0.0, 12.0).ceil() + 2.0
            {
                self.phase = Phase::Done;
                self.output = neutral();
            } else {
                self.output = attack_input(settings);
            }
            return self.output;
        }
        let Some(enemy) = enemy else {
            self.abandon_preparation();
            let since = *self.missing_at.get_or_insert(self.age);
            if self.age - since >= 30.0 {
                self.target = None;
                self.phase = Phase::Waiting;
            }
            return self.output;
        };
        self.missing_at = None;
        if self.target != Some(enemy.id) {
            self.target = Some(enemy.id);
            self.phase = Phase::Waiting;
        }
        let Some(spot) = spot.filter(valid_spot) else {
            self.abandon_preparation();
            return self.output;
        };
        if !valid_target(enemy) {
            self.phase = Phase::Done;
            return self.output;
        }
        let outside = (spot.edge[0] - enemy.position[0]) * spot.inward;
        let height = enemy.position[1] - spot.edge[1];
        // Actual ledge/ground observation or leaving the near-ledge approach
        // rearms for a NEW recovery, never for another poll of this approach.
        if enemy.situation != 2
            || (0x75..=0x7d).contains(&enemy.status)
            || outside > 30.0
            || height < -40.0
            || outside < -8.0
        {
            self.phase = Phase::Waiting;
            return self.output;
        }
        if self.phase == Phase::Done || enemy.stopped || !enemy.vulnerable {
            self.abandon_preparation();
            return self.output;
        }
        let stand = (ctx.position[0] - spot.edge[0]) * spot.inward;
        if !ctx.can_act
            || !(if settings.two_frame_down_tilt {
                ctx.can_down_tilt
            } else {
                ctx.can_down_smash
            })
            || !ctx.input_delay.is_finite()
            || !(0.0..=12.0).contains(&ctx.input_delay)
            || !ctx
                .position
                .iter()
                .chain(ctx.velocity.iter())
                .all(|v| v.is_finite())
            || !(1.0..=if settings.two_frame_down_tilt {
                settings.tilt_range.min(10.0)
            } else {
                MAX_STAND_DISTANCE
            })
                .contains(&stand)
            || (ctx.position[1] - spot.edge[1]).abs() > 4.0
            || ctx.velocity[0].abs() > 0.5
            || !ctx.safe_ground[usize::from(spot.inward > 0.0)]
        {
            self.abandon_preparation();
            return self.output;
        }
        if settings.two_frame_down_tilt && (spot.edge[0] - ctx.position[0]) * ctx.facing < -0.5 {
            // Dtilt is directional. Turn before scheduling, never combine a
            // turnaround flick with the timed down-tilt request.
            self.phase = Phase::Waiting;
            self.output = LedgeOutput::default();
            if ctx.safe_ground[usize::from(-spot.inward > 0.0)] {
                self.output = neutral();
                self.output.stick = Some([-spot.inward * 0.3, 0.0]);
            }
            return self.output;
        }
        let frames = if settings.two_frame_down_tilt {
            settings.fighter.down_tilt
        } else {
            settings
                .fighter
                .down_smash_toward(spot.edge[0] - ctx.position[0], ctx.facing)
        };
        let first = frames.first;
        if first == 0 || first > frames.last {
            self.abandon_preparation();
            return self.output;
        }
        let Some(arrival) = approach_in(spot, enemy) else {
            // Abandon stale preparation if the opponent steers away.
            self.phase = Phase::Waiting;
            return self.output;
        };
        let lead = f32::from(first) + ctx.input_delay;
        if self.phase == Phase::Release && self.age - self.phase_at > 4.0 {
            self.phase = Phase::Waiting;
        }
        // Release the physical attack/direction before the request so down+A
        // produces a fresh smash flick, even if the player was holding them.
        if self.phase == Phase::Waiting && arrival <= lead + 1.0 && arrival > lead - 2.0 {
            self.phase = Phase::Release;
            self.phase_at = self.age;
            self.output = prepare_input(settings);
        } else if self.phase == Phase::Release && self.age > self.phase_at {
            // Aim first active contact inside [predicted arrival, arrival+2).
            // A missed window is abandoned instead of sending a late attack.
            if arrival <= lead && lead < arrival + 2.0 {
                self.phase = Phase::Press;
                self.phase_at = self.age;
                self.output = attack_input(settings);
            } else if lead >= arrival + 2.0 {
                self.phase = Phase::Done;
            } else {
                self.output = prepare_input(settings);
            }
        }
        self.output
    }
    fn abandon_preparation(&mut self) {
        if self.phase == Phase::Release {
            self.phase = Phase::Waiting;
        }
    }
    fn advance(&mut self, ctx: LedgeContext) -> Option<f32> {
        if ctx.motion_kind == 0
            || !ctx.motion_frame.is_finite()
            || ctx.motion_frame < 0.0
            || !ctx.motion_rate.is_finite()
            || ctx.motion_rate <= 0.0
        {
            return None;
        }
        let delta = match self.previous {
            None => 1.0,
            Some((status, motion, frame, rate))
                if status == ctx.status && motion == ctx.motion_kind =>
            {
                if ctx.motion_frame < frame && self.phase == Phase::Waiting {
                    // An idle animation may loop while watching. Pending
                    // requests instead expire on a rewind/unknown time gap.
                    1.0
                } else {
                    (ctx.motion_frame - frame) / rate
                }
            }
            Some(_) => 1.0,
        };
        self.previous = Some((
            ctx.status,
            ctx.motion_kind,
            ctx.motion_frame,
            ctx.motion_rate,
        ));
        if !delta.is_finite() || !(0.0..=4.0).contains(&delta) {
            return None;
        }
        self.age += delta;
        Some(delta)
    }
}
fn neutral() -> LedgeOutput {
    LedgeOutput {
        consume_trigger: true,
        stick: Some([0.0, 0.0]),
        owned: LEDGE_BUTTONS,
        requested: 0,
    }
}
fn press() -> LedgeOutput {
    LedgeOutput {
        stick: Some([0.0, -1.0]),
        requested: MAPPER_A,
        ..neutral()
    }
}
fn prepare_input(settings: LedgeSettings) -> LedgeOutput {
    let mut out = neutral();
    if settings.two_frame_down_tilt {
        out.stick = Some([0.0, -0.6]);
    }
    out
}
fn attack_input(settings: LedgeSettings) -> LedgeOutput {
    if settings.two_frame_down_tilt {
        LedgeOutput {
            requested: MAPPER_A,
            ..prepare_input(settings)
        }
    } else {
        press()
    }
}
fn valid_spot(spot: &LedgeSpot) -> bool {
    spot.edge.iter().all(|v| v.is_finite()) && spot.inward.abs() == 1.0
}
fn valid_target(enemy: RecoveryTarget) -> bool {
    enemy
        .position
        .iter()
        .chain(enemy.velocity.iter())
        .all(|v| v.is_finite())
}
/// Continuous intersection with a small low ledge-approach band. Current native
/// velocity is extrapolated only over a bounded horizon; no future steering or
/// special-move acceleration is invented. Falling approaches are not assumed
/// to expose the same two-frame window as a low rising recovery.
fn approach_in(spot: LedgeSpot, enemy: RecoveryTarget) -> Option<f32> {
    if !valid_spot(&spot)
        || !valid_target(enemy)
        || enemy.velocity[1] < -0.1
        || (0x75..=0x7d).contains(&enemy.previous_status)
    {
        return None;
    }
    let p = [
        (spot.edge[0] - enemy.position[0]) * spot.inward,
        enemy.position[1] - spot.edge[1],
    ];
    let v = [-enemy.velocity[0] * spot.inward, enemy.velocity[1]];
    let mut enter: f32 = 0.0;
    let mut exit = MAX_FORECAST;
    for (axis, (lo, hi)) in [(-1.0, OUTSIDE_REACH), (-ROOT_BELOW_EDGE, 2.0)]
        .into_iter()
        .enumerate()
    {
        if v[axis].abs() < 0.01 {
            if !(lo..=hi).contains(&p[axis]) {
                return None;
            }
        } else {
            let a = (lo - p[axis]) / v[axis];
            let b = (hi - p[axis]) / v[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    // Already inside, moving away, or crossing too quickly is not a forecast
    // with enough observed lead to launch a new grounded attack reliably.
    (enter > 0.0 && enter <= exit && exit - enter >= 1.0 && enter <= MAX_FORECAST).then_some(enter)
}

#[cfg(test)]
mod tests;
