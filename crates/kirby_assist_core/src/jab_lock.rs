//! Shared knockdown recognition; fighter-specific input-only chase policy.
//! A lock is confirmed by native DOWN_DAMAGE/count, never by a jab press alone.
use crate::{
    ledge_trap::LedgeOutput,
    native_input::{MAPPER_A, MAPPER_R},
    timing::{motion_elapsed, motion_hash},
};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub jab_start: f32,
    pub jab_reach: f32,
    pub smash_start: f32,
    pub smash_reach: f32,
    pub grab_start: f32,
    pub grab_reach: f32,
    pub chase_distance: f32,
}
pub fn config(kind: i32) -> Option<Config> {
    match kind {
        5 => Some(crate::fighter_profiles::yoshi::JAB_LOCK),
        _ => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub id: u32,
    pub status: i32,
    pub frame: f32,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub grounded: bool,
    pub lock_count: i32,
    pub tech_available: bool,
    pub vulnerable_in: Option<f32>,
    pub remaining: Option<f32>,
    /// Native last-attacker entry plus current damage state matches this owner.
    pub hit_by_us: bool,
}
pub fn jab_lockable(t: Target) -> bool {
    matches!(t.status, 0x50..=0x54)
        && t.grounded
        && !t.tech_available
        && (0..2).contains(&t.lock_count)
        && t.vulnerable_in == Some(0.0)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Context {
    pub object: u32,
    pub kind: i32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub grounded: bool,
    pub hit: bool,
    pub facing: f32,
    pub position: [f32; 2],
    pub can_move: bool,
    pub can_jab: bool,
    pub can_smash: bool,
    pub can_grab: bool,
    pub safe_ground: [bool; 2],
}
#[derive(Default)]
pub struct JabLock {
    target: Option<u32>,
    last: Option<Context>,
    output: LedgeOutput,
    request: LedgeOutput,
    age: f32,
    spent_tilt: Option<(u32, u64)>,
    previous_tilt_frame: f32,
    phase: u8, // 0 chase, 1 jab requested, 2 jab accepted, 3 finisher requested.
    lock_count: i32,
    request_age: f32,
    confirmed_lock: bool,
}
impl JabLock {
    pub fn watching(&self) -> bool {
        self.target.is_some()
    }
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.output = LedgeOutput::default();
        self.phase = 0;
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        let Some(cfg) = config(c.kind).filter(|_| enabled) else {
            *self = Self::default();
            return LedgeOutput::default();
        };
        if self.last.is_some_and(|p| p.object != c.object) {
            *self = Self::default();
        }
        if !c.frame.is_finite()
            || !c.rate.is_finite()
            || c.rate <= 0.0
            || !c.position.iter().all(|x| x.is_finite())
            || !c.facing.is_finite()
            || c.facing.abs() < 0.5
        {
            self.cancel_pending();
            return self.output;
        }
        let delta = if c.stopped {
            0.0
        } else {
            motion_elapsed(
                self.last.map(|p| (p.status, p.motion, p.frame, p.rate)),
                (c.status, c.motion, c.frame, c.rate),
                0.0,
            )
        };
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            self.cancel_pending();
            self.last = Some(c);
            return self.output;
        }
        if c.status != 0x2c || c.frame < self.previous_tilt_frame {
            self.spent_tilt = None;
        }
        self.previous_tilt_frame = c.frame;
        self.last = Some(c);
        if self.target.is_none() {
            if c.status == 0x2c
                && c.motion == motion_hash("attack_lw3")
                && c.hit
                && self.spent_tilt != Some((c.object, c.motion))
            {
                // Only arm on a real damaging down tilt with an attributed victim.
                let mut victims = targets
                    .iter()
                    .flatten()
                    .filter(|t| t.hit_by_us && valid(**t, c, cfg));
                // A global HIT flag cannot identify a victim when several
                // enemies are already in attributed damage. Skip ambiguity.
                let victim = victims.next().filter(|_| victims.next().is_none());
                if let Some(t) = victim {
                    self.target = Some(t.id);
                    self.age = 0.0;
                    self.phase = 0;
                    self.confirmed_lock = false;
                    self.lock_count = t.lock_count;
                    self.spent_tilt = Some((c.object, c.motion));
                }
            }
        }
        let Some(id) = self.target else {
            self.output = LedgeOutput::default();
            return self.output;
        };
        self.age += delta;
        let Some(t) = targets
            .into_iter()
            .flatten()
            .find(|t| t.id == id && valid(*t, c, cfg))
        else {
            self.cancel_pending();
            return self.output;
        };
        if self.age > 120.0 || !c.grounded || matches!(c.status,0x47..=0x6b | 0x37..=0x3f) {
            self.cancel_pending();
            return self.output;
        }
        if self.phase == 3 && matches!(c.status,0x30..=0x32 | 0x37..=0x3f) {
            self.cancel_pending();
            return self.output;
        }
        if self.phase == 1 && c.status == 0x27 && c.motion == motion_hash("attack_11") {
            self.phase = 2;
            self.request_age = 0.0;
            self.output = crate::move_input::output([0.0; 2], 0);
        }
        if self.phase == 2 && t.hit_by_us && t.status == 0x5a && t.lock_count > self.lock_count {
            self.confirmed_lock = true;
        }
        if c.stopped {
            return self.output;
        }
        let dx = t.position[0] - c.position[0];
        let direction = if dx.abs() > 0.25 {
            dx.signum()
        } else {
            c.facing.signum()
        };
        let distance = dx.abs();
        let facing = direction * c.facing > 0.0;
        let roll = matches!(t.status, 0x58 | 0x68);
        let grab = roll || t.status == 0x67;
        let getup = matches!(t.status,0x57..=0x59 | 0x67..=0x68);
        let startup = if grab {
            cfg.grab_start
        } else {
            cfg.smash_start
        };
        let window = t
            .vulnerable_in
            .zip(t.remaining)
            .is_some_and(|(opens, ends)| {
                opens.is_finite()
                    && ends.is_finite()
                    && opens >= 0.0
                    && opens <= startup
                    && startup + 1.0 < ends
            });
        let contact_dx = if roll {
            dx + t.velocity_x * startup
        } else {
            dx
        };
        self.output = crate::move_input::output([0.0; 2], 0);
        // Jab 1 must release A throughout recovery: holding it would start jab 2.
        if self.phase == 1 || self.phase == 3 {
            self.request_age += delta;
            if self.request_age > crate::move_input::ACCEPTANCE_TIMEOUT {
                self.cancel_pending();
                return self.output;
            }
            let still_legal = facing
                && if self.phase == 1 {
                    jab_lockable(t)
                        && distance <= cfg.jab_reach
                        && c.can_jab
                        && t.remaining.is_some_and(|end| end > cfg.jab_start + 1.0)
                } else {
                    let requested_grab = self.request.requested & MAPPER_R != 0;
                    requested_grab == grab
                        && window
                        && contact_dx * direction >= 0.0
                        && contact_dx.abs()
                            <= if grab {
                                cfg.grab_reach
                            } else {
                                cfg.smash_reach
                            }
                        && if grab { c.can_grab } else { c.can_smash }
                };
            if !still_legal {
                self.cancel_pending();
            } else if crate::move_input::delivery_pressed(self.request_age) {
                self.output = self.request;
            }
            return self.output;
        }
        let attempt_jab = self.phase == 0
            && jab_lockable(t)
            && t.remaining.is_some_and(|end| end > cfg.jab_start + 1.0);
        let attempt_finish = (self.confirmed_lock || getup) && window;
        if facing && attempt_jab && distance <= cfg.jab_reach && c.can_jab {
            self.phase = 1;
            self.request_age = 0.0;
            self.lock_count = t.lock_count;
            self.output = crate::move_input::output([0.0; 2], MAPPER_A);
            self.request = self.output;
        } else if facing
            && contact_dx * direction >= 0.0
            && attempt_finish
            && contact_dx.abs()
                <= if grab {
                    cfg.grab_reach
                } else {
                    cfg.smash_reach
                }
            && if grab { c.can_grab } else { c.can_smash }
        {
            self.phase = 3;
            self.request_age = 0.0;
            self.output = crate::move_input::output(
                if grab { [0.0; 2] } else { [0.0, -1.0] },
                if grab { MAPPER_R } else { MAPPER_A },
            );
            self.request = self.output;
        } else if c.can_move {
            // Ground-checked walking preserves immediate jab/grab access. Track
            // the observed target, with a bounded drift estimate while tumbling.
            let projected = if roll {
                dx + t.velocity_x.clamp(-3.0, 3.0) * 3.0
            } else {
                dx
            };
            let approach = if projected.abs() > cfg.jab_reach * 0.65 || !facing {
                projected.signum() * 0.65
            } else {
                0.0
            };
            let safe = if approach < 0.0 {
                c.safe_ground[0]
            } else {
                c.safe_ground[1]
            };
            if approach != 0.0 && safe {
                self.output = crate::move_input::output([approach, 0.0], 0);
            }
        }
        self.output
    }
}
fn valid(t: Target, c: Context, cfg: Config) -> bool {
    t.position.iter().all(|v| v.is_finite())
        && t.velocity_x.is_finite()
        && t.frame.is_finite()
        && (t.position[0] - c.position[0]).abs() <= cfg.chase_distance
        && (t.position[1] - c.position[1]).abs() <= 24.0
        && matches!(t.status,0x47..=0x4f | 0x50..=0x54 | 0x57..=0x5b | 0x67..=0x68)
}
