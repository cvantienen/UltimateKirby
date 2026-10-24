//! Optional onstage roll-through punish, separate from ledge get-up responses.
//! Fixed-size tracking; down smash may make a bounded, ground-checked approach.
mod smash;
use crate::{
    ledge_trap::LedgeOutput,
    native_input::{MAPPER_A, MAPPER_R},
};

pub const ESCAPE_F: i32 = 0x20;
pub const ESCAPE_B: i32 = 0x21;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Choice {
    #[default]
    Off,
    DownSmash,
    Grab,
}
impl Choice {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::DownSmash => "D/F smash",
            Self::Grab => "Grab",
        }
    }
    pub fn next(self) -> Self {
        self.cycle(true)
    }
    pub fn cycle(self, forward: bool) -> Self {
        const ALL: [Choice; 3] = [Choice::Off, Choice::DownSmash, Choice::Grab];
        ALL[(self as usize + if forward { 1 } else { 2 }) % ALL.len()]
    }
}
#[derive(Clone, Copy)]
pub struct FighterConfig {
    pub first: u8,
    pub last: u8,
    /// Separate rear sweep for sequential smashes; None means the main window
    /// covers both sides (Kirby) or this config is a grab.
    pub back: Option<(u8, u8)>,
    /// Approximate horizontal root reach, not a live grab/hurtbox measurement.
    pub reach: f32,
    pub height: f32,
    pub watch_distance: f32,
}
pub fn fighter_config(kind: i32) -> Option<FighterConfig> {
    match kind {
        6 => Some(crate::fighter_profiles::kirby::ROLL_READ_DOWN_SMASH),
        5 => Some(crate::fighter_profiles::yoshi::ROLL_READ_DOWN_SMASH),
        _ => None,
    }
}
pub fn config_for(kind: i32, choice: Choice) -> Option<FighterConfig> {
    match choice {
        Choice::Off => None,
        Choice::DownSmash => fighter_config(kind),
        Choice::Grab => match kind {
            6 => Some(crate::fighter_profiles::kirby::ROLL_READ_GRAB),
            5 => Some(crate::fighter_profiles::yoshi::ROLL_READ_GRAB),
            _ => None,
        },
    }
}
#[derive(Clone, Copy)]
pub struct Context {
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub can_act: bool,
    pub can_smash: bool,
    pub can_grab: bool,
    pub facing: f32,
    pub position: [f32; 2],
    /// Native movement observations, only needed for down-smash approaches.
    pub movement: Option<Movement>,
}
#[derive(Clone, Copy)]
pub struct Movement {
    pub walk_speed: f32,
    pub velocity_x: f32,
    pub brake: f32,
    pub safe_ground: [bool; 2],
}
pub const SMASH_WALK_STICK: f32 = 0.65;
#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub status: i32,
    pub frame: f32,
    pub rate: f32,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub vulnerable_in: Option<f32>,
    pub actionable_in: Option<f32>,
}
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    status: i32,
    frame: f32,
    rate: f32,
    dx: f32,
    used: bool,
    crossing: bool,
}
#[derive(Default)]
pub struct RollRead {
    tracks: [Option<Track>; 8],
    // 1 = primer/turn preparation, 2 = issued A or R,
    // 3 = walking approach, 4 = shield release. Native startup releases.
    phase: u8,
    target: Option<u32>,
    previous: Option<(i32, u64, f32)>,
    output: LedgeOutput,
    choice: Choice,
    request_age: f32,
    previous_rate: f32,
}
impl RollRead {
    pub fn committed(&self) -> bool {
        matches!(self.phase, 1 | 2 | 4) && self.output.owned != 0
    }
    /// Cancel a losing plan without making its roll eligible again.
    pub fn cancel_pending(&mut self) {
        if let Some(target) = self.target {
            for track in self.tracks.iter_mut().flatten() {
                if track.id == target {
                    track.used = true;
                }
            }
        }
        self.phase = 0;
        self.target = None;
        self.output = LedgeOutput::default();
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        config: FighterConfig,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        self.sample_choice(
            if enabled {
                Choice::DownSmash
            } else {
                Choice::Off
            },
            config,
            c,
            targets,
        )
    }
    pub fn sample_choice(
        &mut self,
        choice: Choice,
        config: FighterConfig,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        if self.choice != choice {
            *self = Self {
                choice,
                ..Self::default()
            };
        }
        if choice == Choice::Off
            || c.situation != 0
            || !c.frame.is_finite()
            || c.frame < 0.0
            || !c.rate.is_finite()
            || c.rate <= 0.0
            || c.motion == 0
            || !c.facing.is_finite()
            || c.facing.abs() < 0.5
            || !c.position.iter().all(|v| v.is_finite())
            || config.first == 0
            || config.last < config.first
            || config.last > 120
            || config
                .back
                .is_some_and(|(first, last)| first == 0 || last < first || last > 120)
            || !config.reach.is_finite()
            || config.reach <= 0.0
            || !config.height.is_finite()
            || config.height < 0.0
            || !config.watch_distance.is_finite()
            || config.watch_distance <= 0.0
        {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if choice == Choice::DownSmash {
            return self.sample_smash(config, c, targets);
        }
        if self.phase != 0 {
            return self.deliver_grab(config, c, targets);
        }
        self.output = LedgeOutput::default();
        // Track each roster slot separately; a different roll/animation rewind
        // rearms that enemy, never another press during the same roll.
        for (slot, target) in targets.into_iter().enumerate() {
            let Some(t) = target else {
                self.tracks[slot] = None;
                continue;
            };
            if !matches!(t.status, ESCAPE_F | ESCAPE_B)
                || !t.frame.is_finite()
                || t.frame < 0.0
                || !t.position.iter().all(|v| v.is_finite())
                || !t.velocity_x.is_finite()
            {
                self.tracks[slot] = None;
                continue;
            }
            let dx = t.position[0] - c.position[0];
            let old = self.tracks[slot]
                .filter(|p| p.id == t.id && p.status == t.status && t.frame >= p.frame);
            let approaching = dx * t.velocity_x < -0.05;
            let crossed = old.is_some_and(|p| p.crossing && p.dx * dx <= 0.0);
            let crossing = approaching || crossed || old.is_some_and(|p| p.crossing);
            let used = old.is_some_and(|p| p.used);
            self.tracks[slot] = Some(Track {
                id: t.id,
                status: t.status,
                frame: t.frame,
                rate: t.rate,
                dx,
                used,
                crossing,
            });
            if used
                || !crossing
                || c.stopped
                || !c.can_act
                || !(if choice == Choice::Grab {
                    c.can_grab
                } else {
                    c.can_smash
                })
                || dx.abs() > config.watch_distance
                || (t.position[1] - c.position[1]).abs() > config.height
            {
                continue;
            }
            let Some(opens) = t
                .vulnerable_in
                .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 120.0)
            else {
                continue;
            };
            let midpoint = f32::from(config.first + (config.last - config.first) / 2);
            if opens > midpoint + 1.0
                || t.actionable_in
                    .is_none_or(|end| !end.is_finite() || end <= f32::from(config.first) + 1.0)
            {
                continue;
            }
            // Short bounded extrapolation: rolling velocity is not constant
            // across the entire animation. Never run after a projected root.
            let projected = dx + t.velocity_x * opens.min(6.0);
            if projected.abs() > config.reach {
                continue;
            }
            self.tracks[slot].as_mut().unwrap().used = true;
            self.phase = 1;
            self.request_age = 0.0;
            self.target = Some(t.id);
            self.previous = Some((c.status, c.motion, c.frame));
            self.previous_rate = c.rate;
            self.output = controls([0.0, 0.0], 0);
            if choice == Choice::Grab {
                let desired = grab_direction(
                    dx + t.velocity_x * f32::from(config.first).min(6.0),
                    c.facing,
                );
                if !c.facing.is_finite() || c.facing.abs() < 0.5 {
                    self.phase = 0;
                    self.output = LedgeOutput::default();
                } else if c.facing * desired < 0.0 {
                    self.output = controls([desired * 0.3, 0.0], 0);
                }
            }
            break;
        }
        self.output
    }

    /// Standing roll-grab, never a chase. Prime/release R, turn toward the
    /// short projected contact point if needed, then retry only a bounded input
    /// delivery window. Native grab/capture acceptance hands control back.
    fn deliver_grab(
        &mut self,
        config: FighterConfig,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        if matches!(c.status, 0x37..=0x40) || !c.facing.is_finite() || c.facing.abs() < 0.5 {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        }
        if c.stopped {
            return self.output;
        }
        if !c.can_act {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        }
        let clock = (c.status, c.motion, c.frame);
        let mut elapsed = crate::timing::motion_elapsed(
            self.previous.map(|p| (p.0, p.1, p.2, self.previous_rate)),
            (c.status, c.motion, c.frame, c.rate),
            1.0,
        );
        if elapsed == 0.0 {
            if let Some(t) = targets.iter().flatten().find(|t| Some(t.id) == self.target) {
                if let Some(p) = self
                    .tracks
                    .iter()
                    .flatten()
                    .find(|p| p.id == t.id && p.status == t.status)
                {
                    elapsed = crate::timing::motion_elapsed(
                        Some((p.status, 1, p.frame, p.rate)),
                        (t.status, 1, t.frame, t.rate),
                        0.0,
                    );
                }
            }
        }
        for t in targets.iter().flatten() {
            if let Some(p) = self.tracks.iter_mut().flatten().find(|p| p.id == t.id) {
                p.frame = t.frame;
                p.rate = t.rate;
            }
        }
        if !(0.0..=8.0).contains(&elapsed) {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        }
        self.previous = Some(clock);
        self.previous_rate = c.rate;

        self.request_age += elapsed;
        if self.request_age
            > if self.phase == 2 {
                crate::move_input::ACCEPTANCE_TIMEOUT
            } else {
                6.0
            }
        {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        }
        let target = targets.into_iter().flatten().find(|t| {
            Some(t.id) == self.target
                && matches!(t.status, ESCAPE_F | ESCAPE_B)
                && t.position.iter().all(|v| v.is_finite())
                && t.velocity_x.is_finite()
                && t.frame.is_finite()
                && t.frame >= 0.0
                && t.vulnerable_in
                    .is_some_and(|v| v.is_finite() && v >= 0.0 && v <= 120.0)
                && t.actionable_in
                    .is_some_and(|v| v.is_finite() && v > f32::from(config.first))
                && (t.position[1] - c.position[1]).abs() <= config.height
        });
        let Some(t) = target else {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        };
        let dx = t.position[0] - c.position[0];
        let projected = dx + t.velocity_x * f32::from(config.first).min(6.0);
        if dx.abs() > config.watch_distance || projected.abs() > config.reach {
            self.phase = 0;
            self.output = LedgeOutput::default();
            return self.output;
        }
        // Revalidate live reach/window even when neither native clock advances.
        if elapsed == 0.0 {
            return self.output;
        }
        if self.phase == 2 {
            if !c.can_grab || projected * c.facing < -0.5 {
                self.cancel_pending();
            } else {
                self.output = controls(
                    [0.0; 2],
                    if crate::move_input::delivery_pressed(self.request_age) {
                        MAPPER_R
                    } else {
                        0
                    },
                );
            }
            return self.output;
        }
        let desired = grab_direction(projected, c.facing);
        if c.facing * desired < 0.0 {
            self.output = controls([desired * 0.3, 0.0], 0);
        } else if self.output.stick.is_some_and(|s| s[0] != 0.0)
            || !c.can_grab
            || t.vulnerable_in.unwrap() > f32::from(config.first)
        {
            self.output = controls([0.0, 0.0], 0);
        } else {
            self.phase = 2;
            self.request_age = 0.0;
            self.output = controls([0.0, 0.0], MAPPER_R);
        }
        self.output
    }
}
fn grab_direction(projected: f32, facing: f32) -> f32 {
    if projected.abs() <= 0.5 {
        facing.signum()
    } else {
        projected.signum()
    }
}
fn controls(stick: [f32; 2], requested: u32) -> LedgeOutput {
    crate::move_input::output(stick, requested)
}
