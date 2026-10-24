//! Conservative whiff recovery punish. Native observations must first verify
//! the move, final active frame, no active slots/contact/cancel and recovery.
//! Fixed eight-slot history; no target simulation, file I/O or geometry scans.
use crate::{
    button_modes::YMode,
    finisher::{Context as StarterContext, Finisher},
    ledge_trap::{LedgeOutput, LEDGE_BUTTONS},
    native_input::MAPPER_A,
};
pub fn supported(kind: i32) -> bool {
    matches!(kind, 5 | 6)
}
#[derive(Clone, Copy)]
pub struct Context {
    pub starter: StarterContext,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub walk_speed: Option<f32>,
    pub can_tilt: bool,
    pub can_smash: bool,
    pub safe_ground: [bool; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub id: u32,
    pub kind: i32,
    pub motion: u64,
    pub frame: f32,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub remaining: f32,
}
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    motion: u64,
    frame: f32,
    used: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Tilt,
    Smash,
    YoshiBair,
}
impl Action {
    fn startup(self) -> f32 {
        match self {
            Self::Tilt => f32::from(crate::fighter_profiles::kirby::LEDGE_DATA.down_tilt.first),
            Self::Smash => f32::from(crate::fighter_profiles::kirby::WHIFF_SMASH_STARTUP),
            Self::YoshiBair => f32::from(crate::fighter_profiles::yoshi::WHIFF_BAIR_STARTUP_BUDGET),
        }
    }
    fn reach(self) -> f32 {
        match self {
            Self::Tilt => crate::fighter_profiles::kirby::WHIFF_TILT_REACH,
            Self::Smash => crate::fighter_profiles::kirby::WHIFF_SMASH_REACH,
            Self::YoshiBair => crate::fighter_profiles::yoshi::WHIFF_BAIR_REACH,
        }
    }
}
#[derive(Default)]
pub struct WhiffPunisher {
    tracks: [Option<Track>; 8],
    target: Option<u32>,
    action: Option<Action>,
    phase: u8,
    previous: Option<(i32, u64, f32)>,
    previous_rate: f32,
    age: f32,
    output: LedgeOutput,
    yoshi: Finisher,
}
impl WhiffPunisher {
    pub fn active(&self) -> bool {
        self.phase != 0 || self.yoshi.active()
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        let own = c.starter.fighter;
        if !enabled
            || !supported(c.starter.fighter_kind)
            || !own.frame.is_finite()
            || own.frame < 0.0
            || !own.rate.is_finite()
            || own.rate <= 0.0
            || !c.position.iter().all(|x| x.is_finite())
            || !c.velocity_x.is_finite()
            || own.motion == 0
            || !own.facing.is_finite()
            || own.facing.abs() < 0.5
        {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.yoshi.active() {
            let out = self
                .yoshi
                .sample(true, false, YMode::YoshiBairSmash, c.starter);
            // Once a hit confirms, enemy damage/recovery changes are expected.
            if self.yoshi.active()
                && !self.yoshi.confirmed()
                && !targets.iter().flatten().any(|t| Some(t.id) == self.target)
            {
                self.yoshi = Finisher::default();
                return self.end();
            }
            self.output = out;
            return out;
        }
        let old_tracks = self.tracks;
        for (slot, t) in targets.iter().enumerate() {
            self.tracks[slot] = t.filter(|t| valid(*t, c)).map(|t| {
                let used = old_tracks
                    .iter()
                    .flatten()
                    .any(|p| p.id == t.id && p.motion == t.motion && t.frame >= p.frame && p.used);
                Track {
                    id: t.id,
                    motion: t.motion,
                    frame: t.frame,
                    used,
                }
            });
        }
        if self.phase != 0
            && !targets
                .iter()
                .flatten()
                .any(|t| Some(t.id) == self.target && valid(*t, c))
        {
            return self.end();
        }
        if own.stopped {
            return self.output;
        }
        let clock = (own.status, own.motion, own.frame);
        if self.phase != 0 && self.previous == Some(clock) {
            return self.output;
        }
        let elapsed = crate::timing::motion_elapsed(
            self.previous
                .map(|p| (p.0, p.1, p.2, self.previous_rate.max(0.01))),
            (own.status, own.motion, own.frame, own.rate),
            1.0,
        );
        self.previous = Some(clock);
        self.previous_rate = own.rate;
        if !elapsed.is_finite() || !(0.0..=8.0).contains(&elapsed) {
            return self.end();
        }
        if self.phase == 2 {
            let accepted = match self.action {
                Some(Action::Tilt) => own.status == 0x2c,
                Some(Action::Smash) => matches!(own.status, 0x2d..=0x2f),
                _ => false,
            };
            self.age += elapsed;
            if accepted || self.age >= 3.0 || !own.grounded || !c.starter.can_start {
                return self.end();
            }
            return self.output;
        }
        if !own.grounded || !c.starter.can_start {
            return self.end();
        }
        let choice = targets
            .iter()
            .enumerate()
            .filter_map(|(slot, t)| {
                let t = t.filter(|t| valid(*t, c))?;
                if self.tracks[slot].is_some_and(|p| p.used) {
                    return None;
                }
                let dx = t.position[0] + t.velocity_x * t.remaining.min(3.0)
                    - c.position[0]
                    - c.velocity_x * 2.0;
                let actions: &[Action] = if c.starter.fighter_kind == 6 {
                    &[Action::Smash, Action::Tilt]
                } else {
                    &[Action::YoshiBair]
                };
                for &action in actions {
                    let walk = if dx.abs() <= action.reach() {
                        0.0
                    } else {
                        let speed = c
                            .walk_speed
                            .filter(|s| s.is_finite() && (0.01..=5.0).contains(s))?;
                        if !c.safe_ground[usize::from(dx > 0.0)] {
                            return None;
                        }
                        (dx.abs() - action.reach() + 1.0) / (speed * 0.65)
                    };
                    // Includes a movement-release/primer, turn allowance and two
                    // frames of reserve. Unknown transport delay is not invented.
                    let turn = if dx * own.facing < 0.0 { 2.0 } else { 0.0 };
                    if walk + turn + action.startup() + 3.0 < t.remaining {
                        return Some((slot, t, dx, action));
                    }
                }
                None
            })
            .min_by(|(_, a, ax, _), (_, b, bx, _)| {
                (Some(b.id) == self.target)
                    .cmp(&(Some(a.id) == self.target))
                    .then(ax.abs().total_cmp(&bx.abs()))
            });
        let Some((slot, t, dx, action)) = choice else {
            return self.end();
        };
        let same = self.target == Some(t.id) && self.action == Some(action);
        self.target = Some(t.id);
        self.action = Some(action);
        let was_stick = self.output.stick.is_some_and(|s| s[0].abs() > 0.1);
        self.output = controls([0.0, 0.0], 0);
        if dx.abs() > action.reach() {
            self.phase = 3;
            self.output = controls([dx.signum() * 0.65, 0.0], 0);
            self.output.owned |= crate::manual_controls::LEFT_STICK_DIRECTIONS;
            return self.output;
        }
        if action == Action::YoshiBair {
            if !c.starter.can_jump {
                return self.output;
            }
            self.tracks[slot].as_mut().unwrap().used = true;
            self.yoshi = Finisher::default();
            self.output = self.yoshi.sample(
                true,
                true,
                YMode::YoshiBairSmash,
                StarterContext {
                    target_direction: Some(if dx.abs() > 0.5 {
                        dx.signum()
                    } else {
                        own.facing
                    }),
                    ..c.starter
                },
            );
            return self.output;
        }
        if dx * own.facing < -0.5 {
            self.phase = 3;
            self.output = controls([dx.signum() * 0.3, 0.0], 0);
            return self.output;
        }
        let ready = if action == Action::Tilt {
            c.can_tilt
        } else {
            c.can_smash
        };
        if !ready || was_stick || !same || self.phase != 1 {
            self.phase = 1;
            return self.output;
        }
        self.phase = 2;
        self.age = 0.0;
        self.tracks[slot].as_mut().unwrap().used = true;
        self.output = controls(
            if action == Action::Tilt {
                [0.0, -1.0]
            } else {
                [own.facing, 0.0]
            },
            MAPPER_A,
        );
        self.output
    }
    fn end(&mut self) -> LedgeOutput {
        self.phase = 0;
        self.target = None;
        self.action = None;
        self.output = LedgeOutput::default();
        self.output
    }
}
fn valid(t: Target, c: Context) -> bool {
    t.motion != 0
        && t.frame.is_finite()
        && t.frame >= 0.0
        && t.remaining.is_finite()
        && (1.0..=120.0).contains(&t.remaining)
        && t.position.iter().all(|x| x.is_finite())
        && t.velocity_x.is_finite()
        && (t.position[0] - c.position[0]).abs() <= 40.0
        && (t.position[1] - c.position[1]).abs() <= 5.0
        && crate::opponent_data::profile(t.kind)
            .and_then(|p| p.move_for_motion(t.motion))
            .is_some_and(|m| {
                m.whiff_eligible
                    && m.last_active()
                        .is_some_and(|last| t.frame > f32::from(last))
            })
}
fn controls(stick: [f32; 2], requested: u32) -> LedgeOutput {
    LedgeOutput {
        stick: Some(stick),
        requested,
        owned: LEDGE_BUTTONS | 0xf,
        consume_trigger: false,
    }
}
