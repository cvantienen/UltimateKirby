//! One input-only Kirby/Yoshi starter per Y press. A fresh hurtbox hit, not a shield
//! or a persistent hit flag, permits the landing finisher. No loops or chasing.
use crate::{
    button_modes::YMode, combat_assist::FairContext, gameplay::ControllerOutput,
    native_input::MAPPER_A,
};

/// Short-hop descent budget; native progress excludes hitstop and repeated polls.
pub const FAST_FALL_FRAMES: f32 = 16.0;

#[derive(Clone, Copy)]
pub struct Context {
    pub fighter_kind: i32,
    pub target_direction: Option<f32>,
    pub fighter: FairContext,
    pub velocity_y: f32,
    pub can_jump: bool,
    pub can_start: bool,
}
/// Locomotion/crouch, including run/turn braking. Attacks, shield stun and
/// damage remain ineligible even when the situation flag still says grounded.
pub fn grounded_start_allowed(status: i32, situation: i32) -> bool {
    situation == 0
        && matches!(status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV)
}
pub fn yoshi_early_bair_hit(frame: f32) -> bool {
    let Some(move_data) = crate::fighter_data::named_move(5, "Back Air") else {
        return false;
    };
    let Some(final_hit) = move_data.active.iter().map(|span| span.last).max() else {
        return false;
    };
    move_data.active.iter().any(|span| {
        span.last < final_hit
            && (f32::from(span.first)..f32::from(span.last) + 1.0).contains(&frame)
    })
}
fn yoshi_bair_final_hit_start() -> Option<f32> {
    let move_data = crate::fighter_data::named_move(5, "Back Air")?;
    let final_hit = move_data.active.iter().max_by_key(|span| span.last)?;
    Some(f32::from(final_hit.first))
}
fn kirby_dair_hit(frame: f32) -> bool {
    crate::fighter_data::named_move(6, "Down Air").is_some_and(|move_data| {
        move_data
            .active
            .iter()
            .any(|span| (f32::from(span.first)..f32::from(span.last) + 1.0).contains(&frame))
    })
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
    down_held: bool,
    down_age: f32,
    facing: f32,
    output: ControllerOutput,
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
    /// The mapper must mark every delivered takeoff request as JumpMini, even
    /// while online startup feedback is delayed.
    pub fn requests_short_hop(&self) -> bool {
        self.active && !self.jump_seen && self.output.requested & crate::native_input::MAPPER_X != 0
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        held: bool,
        mode: YMode,
        c: Context,
    ) -> ControllerOutput {
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
        let frame_data_available = match mode {
            YMode::DairSmash => crate::fighter_data::named_move(6, "Down Air")
                .is_some_and(|move_data| !move_data.active.is_empty()),
            YMode::YoshiBairSmash => crate::fighter_data::named_move(5, "Back Air")
                .is_some_and(|move_data| !move_data.active.is_empty()),
            YMode::FairSmash => true,
        };
        if !frame_data_available {
            return self.end();
        }
        if !self.active {
            if !pressed
                || !f.grounded
                || !c.can_start
                || !grounded_start_allowed(f.status, crate::native_states::status::WAIT)
                || f.stopped && c.fighter_kind != crate::fighter_kinds::KIRBY
            {
                return ControllerOutput::default();
            }
            self.active = true;
            self.mode = mode;
            self.age = 0.0;
            self.jump_seen = false;
            self.down_held = false;
            self.down_age = 0.0;
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
            self.output = if c.can_jump || matches!(c.fighter_kind, 5 | 6) && c.can_start {
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
        let aerial = f.status == crate::native_states::status::ATTACK_AIR && f.motion == motion;
        self.aerial_seen |= aerial;
        let hit_window = match self.mode {
            YMode::FairSmash => true, // Any real hit from this exact fair arms Fsmash.
            YMode::DairSmash => kirby_dair_hit(f.frame),
            YMode::YoshiBairSmash => yoshi_early_bair_hit(f.frame),
        };
        self.confirmed |= aerial && hit_window && f.hit_now;
        // A landing transition may be the first observation reporting contact.
        self.confirmed |= self.mode == YMode::FairSmash
            && self.aerial_seen
            && f.grounded
            && f.status == crate::native_states::status::LANDING_ATTACK_AIR
            && f.motion == crate::timing::motion_hash("landing_air_f")
            && f.hit_now;
        self.confirmed |= self.mode == YMode::DairSmash
            && self.aerial_seen
            && f.grounded
            && f.status == crate::native_states::status::LANDING_ATTACK_AIR
            && f.motion == crate::timing::motion_hash("landing_air_lw")
            && (0.0..2.0).contains(&f.frame)
            && f.hit_now;
        // Squat is native jump acceptance: release even if stop begins there.
        if f.status == crate::native_states::status::JUMP_SQUAT {
            self.jump_seen = true;
            self.output = if self.mode == YMode::YoshiBairSmash {
                controls([-self.facing, 0.0], 0)
            } else {
                controls([0.0; 2], 0)
            };
        }
        if f.stopped {
            return self.output;
        }
        if delta == 0.0 {
            return self.output;
        }
        if self.down_held && !f.grounded {
            self.down_age += delta;
            if self.down_age >= FAST_FALL_FRAMES {
                return self.end();
            }
        }
        self.previous = Some(f);
        self.age += delta;
        if self.age > 90.0 {
            return self.end();
        }
        let was_attack = self.output.requested & MAPPER_A != 0;
        let was_down = self.output.stick.is_some_and(|s| s[1] < -0.5);
        let was_moving = self.output.stick.is_some_and(|s| s[0].abs() > 0.1);
        self.output = controls([0.0, 0.0], 0);
        if f.status == crate::native_states::status::JUMP_SQUAT {
            self.jump_seen = true;
            // Native squat confirms Jump was accepted, not that an aerial is
            // legal yet. Situation can change before status at takeoff.
            // All directional starters release Jump and A during squat.
            // Selection begins in native air; Fair additionally waits for apex.
            if self.mode == YMode::YoshiBairSmash {
                self.output.stick = Some([-self.facing, 0.0]);
            }
            return self.output;
        }
        // Native jump/fall status can appear before the copied situation flag
        // changes at takeoff. Do not treat that overlap as a failed landing.
        let airborne_status = matches!(
            f.status,
            crate::native_states::status::JUMP
                | crate::native_states::status::JUMP_AERIAL
                | crate::native_states::status::FALL
                | crate::native_states::status::FALL_AERIAL
                | crate::native_states::status::ATTACK_AIR
        );
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
            if !matches!(f.status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_ATTACK_AIR)
            {
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
                if c.can_jump || c.can_start {
                    self.jump_requested = true;
                    self.output = self.jump_input();
                }
            } else if self.mode == YMode::YoshiBairSmash && !self.jump_seen && self.jump_requested {
                // Preserve the full turnaround direction from a sprint until
                // native jump squat accepts the one-shot short-hop request.
                self.output = controls([-self.facing, 0.0], 0);
            } else if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT {
                return self.end();
            } else if c.fighter_kind == crate::fighter_kinds::KIRBY
                && !self.jump_seen
                && self.jump_requested
            {
                // Request now, not after a transition-term observation round trip.
                // Hold the same jump press until native squat, never a second jump.
                self.output = self.jump_input();
            }
            return self.output;
        }
        self.jump_seen = true;
        if !matches!(
            f.status,
            crate::native_states::status::JUMP
                | crate::native_states::status::JUMP_AERIAL
                | crate::native_states::status::FALL
                | crate::native_states::status::FALL_AERIAL
                | crate::native_states::status::ATTACK_AIR
        ) {
            return self.end();
        }
        if f.status == crate::native_states::status::ATTACK_AIR {
            if !aerial
                || self.mode == YMode::YoshiBairSmash
                    && yoshi_bair_final_hit_start().is_none_or(|first| f.frame >= first)
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
            } else {
                // Queue Down at the apex and retain it for the rest of bair.
                // This fast-falls at the first legal native frame instead of
                // waiting until bair's first hitbox at frame 11.
                self.down_held |= c.velocity_y <= 0.15;
                if self.down_held {
                    self.output.stick = Some([0.0, -1.0]);
                }
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
            self.request_aerial(was_attack, delta);
        }
        self.output
    }
    fn jump_input(&self) -> ControllerOutput {
        if self.mode == YMode::YoshiBairSmash {
            // Reverse-aerial-rush launch: jump and full reverse together. Hold
            // the direction through squat, then attack/drift back toward the
            // target to produce an instant turnaround bair even from full run.
            controls([-self.facing, 0.0], crate::move_input::SHORT_HOP)
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
    fn end(&mut self) -> ControllerOutput {
        self.active = false;
        self.previous = None;
        self.output = ControllerOutput::default();
        self.output
    }
}
fn controls(stick: [f32; 2], requested: u32) -> ControllerOutput {
    let mut output = crate::move_input::output(stick, requested);
    output.owned |= 0xf;
    output
}
