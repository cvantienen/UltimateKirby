//! Shared, bounded recovery punish for spot dodges, avoided grabs and far rolls.
//! Reach values are calibration estimates; native recovery and readiness gate inputs.
use crate::{
    ledge_trap::LedgeOutput,
    native_input::MAPPER_A,
    timing::{motion_elapsed, ActiveWindow, RecoveryWindow},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Spot,
    Grab,
    Roll,
}
#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub kind: Kind,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub opens: f32,
    pub closes: f32,
}
#[derive(Clone, Copy)]
pub struct Context {
    pub object: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub grounded: bool,
    pub can_act: bool,
    pub down_ready: bool,
    pub forward_ready: bool,
    pub facing: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}
#[derive(Clone, Copy)]
pub struct Config {
    pub down: [u8; 2],
    pub forward: [u8; 2],
    pub down_reach: f32,
    pub forward_reach: f32,
}
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    status: i32,
    motion: u64,
    frame: f32,
    rate: f32,
    used: bool,
}
#[derive(Default)]
pub struct Punisher {
    object: Option<u32>,
    tracks: [Option<Track>; 8],
    previous: Option<(i32, u64, f32, f32)>,
    target: Option<u32>,
    phase: u8,
    forward: bool,
    age: f32,
    output: LedgeOutput,
}
impl Punisher {
    pub fn committed(&self) -> bool {
        self.target.is_some() && self.output.owned != 0
    }
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.phase = 0;
        self.output = LedgeOutput::default();
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        cfg: Config,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        if self.object != Some(c.object) {
            *self = Self::default();
            self.object = Some(c.object);
        }
        let old = self.tracks;
        for (i, t) in targets.iter().enumerate() {
            self.tracks[i] = t.map(|t| Track {
                id: t.id,
                status: t.status,
                motion: t.motion,
                frame: t.frame,
                rate: t.rate,
                used: old.iter().flatten().any(|p| {
                    p.id == t.id
                        && p.status == t.status
                        && p.motion == t.motion
                        && t.frame >= p.frame
                        && p.used
                }),
            });
        }
        if !enabled
            || !c.grounded
            || !c.frame.is_finite()
            || c.frame < 0.0
            || c.motion == 0
            || !c.rate.is_finite()
            || c.rate <= 0.0
            || !c.facing.is_finite()
            || c.facing.abs() < 0.5
        {
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        if matches!(c.status, 0x2d..=0x32) {
            self.cancel_pending();
            return self.output;
        }
        if c.stopped {
            return self.output;
        }
        let clock = (c.status, c.motion, c.frame, c.rate);
        let mut delta = motion_elapsed(self.previous, clock, 0.0);
        self.previous = Some(clock);
        if delta == 0.0 {
            if let Some(t) = targets.iter().flatten().find(|t| Some(t.id) == self.target) {
                if let Some(p) = old
                    .iter()
                    .flatten()
                    .find(|p| p.id == t.id && p.motion == t.motion)
                {
                    delta = motion_elapsed(
                        Some((p.status, p.motion, p.frame, p.rate)),
                        (t.status, t.motion, t.frame, t.rate),
                        0.0,
                    );
                }
            }
        }
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            self.cancel_pending();
            return self.output;
        }
        if self.target.is_some_and(|id| {
            !targets
                .iter()
                .flatten()
                .any(|t| t.id == id && contact(c, cfg, *t, self.forward, 0.0))
        }) {
            self.cancel_pending();
            return self.output;
        }
        if self.phase != 0 && delta == 0.0 {
            return self.output;
        }
        if self.phase == 2 {
            self.age += delta;
            if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT
                || !targets
                    .iter()
                    .flatten()
                    .any(|t| Some(t.id) == self.target && contact(c, cfg, *t, self.forward, 0.0))
            {
                self.cancel_pending();
            } else {
                self.output = if crate::move_input::delivery_pressed(self.age) {
                    crate::move_input::output(
                        if self.forward {
                            [c.facing.signum(), 0.0]
                        } else {
                            [0.0, -1.0]
                        },
                        MAPPER_A,
                    )
                } else {
                    crate::move_input::output([0.0; 2], 0)
                };
            }
            return self.output;
        }
        let shield = match c.status {
            0x1b..=0x1c => 11.0,
            0x1d => (11.0 - c.frame).max(0.0),
            _ => 0.0,
        };
        if !c.can_act
            && shield == 0.0
            && !(matches!(c.status, 0x1b..=0x1d) && (c.down_ready || c.forward_ready))
        {
            self.cancel_pending();
            return self.output;
        }
        let setup = shield + if self.phase == 1 { 0.0 } else { 1.0 };
        let mut selected = None;
        for (i, t) in targets.iter().enumerate() {
            let Some(t) = t else { continue };
            if self.tracks[i].unwrap().used || self.target.is_some_and(|id| id != t.id) {
                continue;
            }
            let at = t.opens.clamp(0.0, 120.0);
            let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
            let forward = x.abs() > cfg.down_reach;
            if t.kind == Kind::Roll && !forward {
                continue;
            } // close rolls retain the existing down-smash reader
            if contact(c, cfg, *t, forward, setup) {
                selected = Some((i, *t, forward, x));
                break;
            }
        }
        let Some((i, t, forward, x)) = selected else {
            self.cancel_pending();
            return self.output;
        };
        self.target = Some(t.id);
        self.forward = forward;
        self.output = crate::move_input::output([0.0; 2], 0);
        if forward && x * c.facing < 0.5 {
            if shield == 0.0 && c.can_act {
                self.output.stick = Some([x.signum() * 0.3, 0.0]);
            }
            self.phase = 0;
            return self.output;
        }
        if shield > 0.0 {
            self.phase = 1;
            return self.output;
        }
        if self.phase != 1 {
            self.phase = 1;
            return self.output;
        }
        if !(if forward {
            c.forward_ready
        } else {
            c.down_ready
        }) {
            return self.output;
        }
        self.phase = 2;
        self.age = 0.0;
        self.tracks[i].as_mut().unwrap().used = true;
        self.output = crate::move_input::output(
            if forward {
                [c.facing.signum(), 0.0]
            } else {
                [0.0, -1.0]
            },
            MAPPER_A,
        );
        self.output
    }
}
fn contact(c: Context, cfg: Config, t: Target, forward: bool, setup: f32) -> bool {
    if !t.frame.is_finite()
        || !t.rate.is_finite()
        || t.rate <= 0.0
        || !t.opens.is_finite()
        || t.opens < 0.0
        || !t.closes.is_finite()
        || t.closes <= t.opens
        || t.motion == 0
        || !c
            .position
            .iter()
            .chain(c.velocity.iter())
            .chain(t.position.iter())
            .chain(t.velocity.iter())
            .all(|v| v.is_finite())
    {
        return false;
    }
    let frames = if forward { cfg.forward } else { cfg.down };
    let reach = if forward {
        cfg.forward_reach
    } else {
        cfg.down_reach
    };
    ActiveWindow {
        first: frames[0] as u16,
        last: frames[1] as u16,
    }
    .first_contact(
        setup,
        RecoveryWindow {
            vulnerable_in: Some(t.opens),
            actionable_in: Some(t.closes),
        },
        |at| {
            let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
            let y = t.position[1] - c.position[1] + (t.velocity[1] - c.velocity[1]) * at;
            x.abs() <= reach && y.abs() <= 8.0
        },
    )
    .is_some()
}
