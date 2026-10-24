//! One input-only Kirby/Yoshi starter per Y press. A fresh hurtbox hit, not a shield
//! or a persistent hit flag, permits the landing finisher. No loops or chasing.
use crate::{
    button_modes::YMode, combat_assist::FairContext, ledge_trap::LedgeOutput,
    native_input::MAPPER_A,
};

#[derive(Clone, Copy)]
pub struct Context {
    pub fighter_kind: i32,
    pub target_direction: Option<f32>,
    pub fighter: FairContext,
    pub velocity_y: f32,
    pub can_jump: bool,
    pub can_start: bool,
}
/// Native floor/physics forecast, acquired only for Kirby's airborne starter.
#[derive(Clone, Copy)]
pub struct AirTiming {
    pub landing_frames: f32,
}
pub fn landing_frames(height: f32, mut velocity: f32, gravity: f32, dive: f32) -> Option<f32> {
    if ![height, velocity, gravity, dive]
        .iter()
        .all(|v| v.is_finite())
        || height <= 0.0
        || !(0.001..=2.0).contains(&gravity)
        || !(0.1..=10.0).contains(&dive)
    {
        return None;
    }
    let mut remaining = height;
    for frame in 1..=180 {
        velocity = if velocity < 0.0 {
            -dive.max(-velocity)
        } else {
            velocity - gravity
        };
        remaining += velocity;
        if remaining <= 0.0001 {
            return Some(frame as f32);
        }
    }
    None
}
/// Locomotion/crouch, including run/turn braking. Attacks, shield stun and
/// damage remain ineligible even when the situation flag still says grounded.
pub fn grounded_start_allowed(status: i32, situation: i32) -> bool {
    situation == 0 && matches!(status, 0..=9 | 0x11..=0x15)
}
pub fn yoshi_early_bair_hit(frame: f32) -> bool {
    use crate::fighter_profiles::yoshi::{BACK_AIR_FIRST, BACK_AIR_MIDDLE};
    [BACK_AIR_FIRST, BACK_AIR_MIDDLE]
        .iter()
        .any(|s| (f32::from(s.first)..f32::from(s.last) + 1.0).contains(&frame))
}
#[derive(Default)]
pub struct Finisher {
    held: bool,
    active: bool,
    mode: YMode,
    previous: Option<FairContext>,
    age: f32,
    jump_seen: bool,
    confirmed: bool,
    aerial_seen: bool,
    jump_requested: bool,
    aerial_request_age: f32,
    fair_apex_ready: bool,
    air_entry: bool,
    down_held: bool,
    facing: f32,
    output: LedgeOutput,
}
impl Finisher {
    pub fn cancel_pending(&mut self) {
        self.end();
    } // Retain held-Y edge history.
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn confirmed(&self) -> bool {
        self.confirmed
    }
    pub fn sample(&mut self, enabled: bool, held: bool, mode: YMode, c: Context) -> LedgeOutput {
        self.sample_with_air(enabled, held, mode, c, None)
    }
    pub fn needs_air_timing(&self) -> bool {
        !self.aerial_seen
    }
    pub fn sample_with_air(
        &mut self,
        enabled: bool,
        held: bool,
        mode: YMode,
        c: Context,
        air: Option<AirTiming>,
    ) -> LedgeOutput {
        let f = c.fighter;
        let pressed = held && !self.held;
        self.held = held;
        if !enabled
            || !mode.supports(c.fighter_kind)
            || !f.frame.is_finite()
            || f.frame < 0.0
            || !f.rate.is_finite()
            || f.rate <= 0.0
            || !c.velocity_y.is_finite()
            || !f.facing.is_finite()
            || f.facing.abs() < 0.5
            || self.active
                && (mode != self.mode || self.previous.is_some_and(|p| p.object != f.object))
        {
            return self.end();
        }
        if !self.active {
            let first = if mode == YMode::FairSmash { 10.0 } else { 18.0 };
            let air_start = c.fighter_kind == 6
                && !f.grounded
                && matches!(f.status, 0xb | 0xc | 0xe | 0xf)
                && air.is_some_and(|a| {
                    a.landing_frames.is_finite() && a.landing_frames >= first + 2.0
                });
            if !pressed
                || !(air_start || f.grounded && c.can_start && grounded_start_allowed(f.status, 0))
                || f.stopped && (air_start || c.fighter_kind != 6)
            {
                return LedgeOutput::default();
            }
            self.active = true;
            self.mode = mode;
            self.age = 0.0;
            self.jump_seen = air_start;
            self.air_entry = air_start;
            self.down_held = false;
            self.confirmed = false;
            self.aerial_seen = false;
            self.jump_requested = false;
            self.aerial_request_age = 0.0;
            self.fair_apex_ready = false;
            self.facing = c
                .target_direction
                .filter(|x| x.is_finite() && x.abs() > 0.5)
                .unwrap_or(f.facing)
                .signum();
            self.previous = Some(f);
            self.output = if air_start {
                self.down_held = c.velocity_y < -0.05;
                controls([0.0, if self.down_held { -1.0 } else { 0.0 }], 0)
            } else if mode == YMode::YoshiBairSmash && f.facing * self.facing > -0.5 {
                controls([-self.facing * 0.3, 0.0], 0)
            } else if c.can_jump || c.fighter_kind == 6 && c.can_start {
                self.jump_requested = true;
                self.jump_input()
            } else {
                controls([0.0, 0.0], 0)
            };
            return self.output;
        }
        let previous = self.previous.unwrap();
        let delta = crate::timing::motion_elapsed(
            Some((
                previous.status,
                previous.motion,
                previous.frame,
                previous.rate,
            )),
            (f.status, f.motion, f.frame, f.rate),
            1.0,
        );
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            return self.end();
        }
        // A hit flag may change in a duplicate native frame; inspect it before
        // the duplicate guard without spending time or creating a button edge.
        let motion = match self.mode {
            YMode::FairSmash => 0xc3495ada5,
            YMode::DairSmash => 0xd40042152,
            YMode::YoshiBairSmash => crate::timing::motion_hash("attack_air_b"),
        };
        let aerial = f.status == 0x36 && f.motion == motion;
        self.aerial_seen |= aerial;
        let hit_window = match self.mode {
            YMode::FairSmash => true, // Any real hit from this exact fair arms Fsmash.
            YMode::DairSmash => [
                (18.0, 20.0),
                (21.0, 23.0),
                (24.0, 26.0),
                (27.0, 29.0),
                (30.0, 32.0),
                (34.0, 35.0),
            ]
            .iter()
            .any(|(first, end)| (*first..*end).contains(&f.frame)),
            YMode::YoshiBairSmash => yoshi_early_bair_hit(f.frame),
        };
        self.confirmed |= aerial && hit_window && f.hit_now;
        // A landing transition may be the first observation reporting contact.
        self.confirmed |= self.mode == YMode::FairSmash
            && self.aerial_seen
            && f.grounded
            && f.status == 0x18
            && f.motion == crate::timing::motion_hash("landing_air_f")
            && f.hit_now;
        self.confirmed |= self.mode == YMode::DairSmash
            && self.aerial_seen
            && f.grounded
            && f.status == 0x18
            && f.motion == crate::timing::motion_hash("landing_air_lw")
            && (0.0..2.0).contains(&f.frame)
            && f.hit_now;
        // Squat is native jump acceptance: release even if stop begins there.
        if f.status == 0xa {
            self.jump_seen = true;
            self.output = controls([0.0; 2], 0);
        }
        if f.stopped {
            return self.output;
        }
        if delta == 0.0 {
            return self.output;
        }
        self.previous = Some(f);
        self.age += delta;
        if self.age > if self.air_entry { 240.0 } else { 90.0 } {
            return self.end();
        }
        let was_attack = self.output.requested & MAPPER_A != 0;
        let was_down = self.output.stick.is_some_and(|s| s[1] < -0.5);
        let was_moving = self.output.stick.is_some_and(|s| s[0].abs() > 0.1);
        self.output = controls([0.0, 0.0], 0);
        if f.status == 0xa {
            self.jump_seen = true;
            // Native squat confirms Jump was accepted, not that an aerial is
            // legal yet. Situation can change before status at takeoff.
            // All directional starters release Jump and A during squat.
            // Selection begins in native air; Fair additionally waits for apex.
            return self.output;
        }
        // Native jump/fall status can appear before the copied situation flag
        // changes at takeoff. Do not treat that overlap as a failed landing.
        let airborne_status = matches!(f.status, 0xb | 0xc | 0xe | 0xf | 0x36);
        if f.grounded && !airborne_status {
            let (start, attack) = match self.mode {
                YMode::FairSmash => (0x2d, 0x2f),
                YMode::YoshiBairSmash => (0x2d, 0x2f),
                YMode::DairSmash => (0x30, 0x32),
            };
            if f.status == attack || f.status == start + 1 {
                return self.end();
            }
            if f.status == start {
                if self.mode == YMode::FairSmash {
                    self.output.stick = Some([self.facing, 0.5]);
                }
                return self.output; // Release A, never charge.
            }
            if self.mode == YMode::FairSmash && self.confirmed {
                if let Some(direction) = c
                    .target_direction
                    .filter(|x| x.is_finite() && x.abs() > 0.5)
                {
                    self.facing = direction.signum();
                }
            }
            if !matches!(f.status, 0..=9 | 0x11..=0x18) {
                return self.end();
            }
            if self.jump_seen {
                if !self.confirmed {
                    return self.end();
                }
                if self.mode == YMode::YoshiBairSmash {
                    if f.facing * self.facing < 0.5 {
                        self.output = controls([self.facing * 0.3, 0.0], 0);
                        return self.output;
                    }
                    if was_moving && !was_attack {
                        return self.output;
                    }
                }
                // Buffer through landing lag, with release gaps for rejected
                // requests, until native smash startup accepts the input.
                if previous.grounded && was_attack {
                    return self.output;
                }
                self.output = controls(
                    match self.mode {
                        YMode::FairSmash => crate::move_input::smash_stick(
                            crate::move_input::Smash::ForwardUp,
                            self.facing,
                        ),
                        YMode::YoshiBairSmash => crate::move_input::smash_stick(
                            crate::move_input::Smash::Forward,
                            self.facing,
                        ),
                        YMode::DairSmash => crate::move_input::smash_stick(
                            crate::move_input::Smash::Down,
                            self.facing,
                        ),
                    },
                    MAPPER_A,
                );
            } else if !self.jump_requested && self.mode != YMode::YoshiBairSmash {
                if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT {
                    return self.end();
                }
                if c.can_jump && c.can_start {
                    self.jump_requested = true;
                    self.output = controls([0.0, 0.0], crate::move_input::SHORT_HOP);
                }
            }
            // One grounded pulse. Keep owning the released input while waiting
            // for a buffered jump to reach native state; never repeat Jump.
            else if self.mode == YMode::YoshiBairSmash && !self.jump_requested {
                if self.age > crate::move_input::ACCEPTANCE_TIMEOUT {
                    return self.end();
                }
                if f.facing * self.facing > -0.5 {
                    self.output = controls([-self.facing * 0.3, 0.0], 0);
                } else if c.can_jump {
                    self.jump_requested = true;
                    self.output = controls([0.0, 0.0], crate::move_input::SHORT_HOP);
                }
            } else if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT {
                return self.end();
            } else if c.fighter_kind == 6 && !self.jump_seen && self.jump_requested {
                // Request now, not after a transition-term observation round trip.
                // Hold the same jump press until native squat, never a second jump.
                self.output = self.jump_input();
            }
            return self.output;
        }
        self.jump_seen = true;
        if !matches!(f.status, 0xb | 0xc | 0xe | 0xf | 0x36) {
            return self.end();
        }
        if f.status == 0x36 {
            if !aerial
                || self.mode == YMode::YoshiBairSmash
                    && f.frame >= f32::from(crate::fighter_profiles::yoshi::BACK_AIR_FINISH.first)
            {
                return self.end();
            }
            if self.mode != YMode::YoshiBairSmash {
                // Wait for descent to create a fresh Down flick, then retain it
                // across hitstop/velocity changes until actual landing.
                // Dair selection may have held Down through a delayed native
                // acceptance. Neutral once before the descending Down flick.
                self.down_held |= c.velocity_y < -0.05 && !was_down;
                if self.down_held {
                    self.output.stick = Some([0.0, -1.0]);
                }
            } else if f.frame >= f32::from(crate::fighter_profiles::yoshi::BACK_AIR_FIRST.first) {
                self.output.stick = crate::move_input::fast_fall(c.velocity_y);
            }
        } else {
            if self.mode != YMode::YoshiBairSmash && self.aerial_seen {
                // A completed fair must not start a second aerial while waiting
                // to land. Preserve the confirmed smash and continue descent.
                self.down_held |= c.velocity_y < -0.05;
                if self.down_held {
                    self.output.stick = Some([0.0, -1.0]);
                }
                return self.output;
            }
            // Keep delivering until native ATTACK_AIR confirms the exact move.
            // Periodic release gaps create a fresh edge if a request was missed;
            // only advancing native frames, never duplicate polls, spend it.
            if self.mode == YMode::YoshiBairSmash && f.facing * self.facing > -0.5 {
                return self.end();
            }
            if self.mode == YMode::FairSmash {
                self.fair_apex_ready |= c.velocity_y <= 0.15;
                if !self.fair_apex_ready {
                    return self.output;
                }
            }
            if self.air_entry && !self.aerial_seen {
                self.down_held |= c.velocity_y < -0.05;
                let first = if self.mode == YMode::FairSmash {
                    10.0
                } else {
                    18.0
                };
                let Some(air) = air.filter(|a| a.landing_frames.is_finite()) else {
                    return self.end();
                };
                if air.landing_frames > first + 4.0 {
                    if self.down_held {
                        self.output.stick = Some([0.0, -1.0]);
                    }
                    return self.output;
                }
            }
            self.request_aerial(was_attack, delta);
        }
        self.output
    }
    fn jump_input(&self) -> LedgeOutput {
        if self.mode == YMode::YoshiBairSmash {
            controls([0.0; 2], crate::move_input::SHORT_HOP)
        } else {
            let mut output = controls([0.0; 2], crate::native_input::MAPPER_X);
            output.stick = None; // Keep physical walking/running direction on takeoff.
            output
        }
    }
    fn request_aerial(&mut self, was_attack: bool, delta: f32) {
        self.aerial_request_age = if was_attack {
            self.aerial_request_age + delta
        } else {
            0.0
        };
        let stick = crate::move_input::aerial_stick(
            if self.mode == YMode::DairSmash {
                crate::move_input::Aerial::Down
            } else if self.mode == YMode::YoshiBairSmash {
                crate::move_input::Aerial::Back
            } else {
                crate::move_input::Aerial::Forward
            },
            if self.mode == YMode::YoshiBairSmash {
                -self.facing
            } else {
                self.facing
            },
        );
        // Three advancing frames held, then one released. No Jump remains held
        // and down is never mixed into the forward/back aerial selection.
        self.output = controls(
            stick,
            if self.aerial_request_age < 3.0 {
                MAPPER_A
            } else {
                0
            },
        );
    }
    fn end(&mut self) -> LedgeOutput {
        self.active = false;
        self.previous = None;
        self.output = LedgeOutput::default();
        self.output
    }
}
fn controls(stick: [f32; 2], requested: u32) -> LedgeOutput {
    let mut output = crate::move_input::output(stick, requested);
    output.owned |= 0xf;
    output
}
