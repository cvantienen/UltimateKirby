//! Shield-context reactions, bounded to the existing eight-fighter snapshot.
//! No native queries, guessed move timings, allocation or physics writes.
use crate::{
    defense_response::Response,
    ledge_trap::{LedgeContext, LedgeOutput, LedgeSettings, MotionTiming, Opponent},
    reaction_rules::{Resolved, Side, Trigger},
    roll_read::Target,
};
#[derive(Clone, Copy)]
struct Seen {
    id: u32,
    status: i32,
    motion: u64,
    frame: f32,
    dx: f32,
    from_shield: bool,
    crossed: bool,
    start_side: f32,
    away: Option<bool>,
    used: bool,
}
#[derive(Default)]
pub struct Pressure {
    object: Option<u32>,
    seen: [Option<Seen>; 8],
    response: Response,
    claim_rolls: bool,
}
fn shield_context(s: i32) -> bool {
    (0x1b..=0x1e).contains(&s)
}
fn roll(s: i32) -> bool {
    matches!(s, 0x20 | 0x21)
}
impl Pressure {
    pub fn target(&self) -> Option<u32> {
        self.response.target()
    }
    pub fn committed(&self) -> bool {
        self.target().is_some()
    }
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    pub fn needs_ground(&self) -> bool {
        self.response.needs_ground()
    }
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.response.right_stick()
    }
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
    }
    pub fn claims_roll(&self, id: u32) -> bool {
        self.claim_rolls
            && self
                .seen
                .iter()
                .flatten()
                .any(|s| s.id == id && roll(s.status) && s.from_shield)
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        allowed: bool,
        object: u32,
        rules: Option<&Resolved>,
        settings: LedgeSettings,
        c: LedgeContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
    ) -> LedgeOutput {
        if self.object != Some(object) {
            *self = Self::default();
            self.object = Some(object);
        }
        if !enabled || rules.is_none() || !c.gameplay {
            *self = Self::default();
            return LedgeOutput::default();
        }
        let rules = rules.unwrap();
        self.claim_rolls = Trigger::SHIELD_EVENTS[1..]
            .iter()
            .any(|t| rules.has_defense(*t));
        if !c.stopped {
            let old = self.seen;
            self.seen = std::array::from_fn(|i| {
                targets[i]
                    .filter(|t| {
                        (shield_context(t.status) || roll(t.status))
                            && t.motion != 0
                            && t.frame.is_finite()
                            && t.frame >= 0.
                            && t.position.iter().all(|v| v.is_finite())
                    })
                    .map(|t| {
                        let before = old.iter().flatten().find(|s| s.id == t.id);
                        let same_roll = roll(t.status)
                            && before.is_some_and(|s| {
                                s.status == t.status && s.motion == t.motion && t.frame >= s.frame
                            });
                        let same_shield = shield_context(t.status)
                            && before.is_some_and(|s| shield_context(s.status));
                        let dx = t.position[0] - c.position[0];
                        let start_side = if same_roll {
                            before.map_or(0., |s| s.start_side)
                        } else {
                            before.map_or(dx, |s| s.dx)
                        };
                        let away = if same_roll {
                            before.and_then(|s| s.away)
                        } else {
                            None
                        }
                        .or_else(|| {
                            (roll(t.status)
                                && t.velocity_x.is_finite()
                                && t.velocity_x.abs() > 0.001
                                && start_side.abs() > 0.001)
                                .then_some(t.velocity_x * start_side > 0.)
                        });
                        Seen {
                            id: t.id,
                            status: t.status,
                            motion: t.motion,
                            frame: t.frame,
                            start_side,
                            away,
                            dx: if dx.abs() > 0.001 {
                                dx
                            } else {
                                before.map_or(dx, |s| s.dx)
                            },
                            from_shield: roll(t.status)
                                && before.is_some_and(|s| {
                                    shield_context(s.status) || same_roll && s.from_shield
                                }),
                            crossed: roll(t.status)
                                && before.is_some_and(|s| {
                                    same_roll && s.crossed
                                        || (same_roll || shield_context(s.status)) && s.dx * dx < 0.
                                }),
                            used: (same_roll || same_shield) && before.is_some_and(|s| s.used),
                        }
                    })
            });
        }
        // A fresh hit on our shield belongs to OOS, not an old pressure tape.
        if !allowed || c.status == 0x1e {
            self.cancel_pending();
            return LedgeOutput::default();
        }
        let mut following = follow.filter(|e| Some(e.id) == self.target());
        if !self.committed() && !c.stopped && c.can_act {
            'rules: for candidate in rules.rules.iter().flatten() {
                let r = candidate.sequence.reaction;
                if !r.trigger.shield_pressure() {
                    continue;
                }
                for (i, t) in targets.iter().enumerate() {
                    let (Some(t), Some(seen)) = (t, self.seen[i]) else {
                        continue;
                    };
                    if seen.used
                        || !r.state.matches(c)
                        || !c.facing.is_finite()
                        || c.facing.abs() < 0.5
                    {
                        continue;
                    }
                    let dx = t.position[0] - c.position[0];
                    if !dx.is_finite()
                        || dx.abs() < r.min
                        || dx.abs() > r.max
                        || (t.position[1] - c.position[1]).abs() > r.height
                        || matches!(r.side, Side::Front) && dx * c.facing <= 0.001
                        || matches!(r.side, Side::Behind) && dx * c.facing >= -0.001
                    {
                        continue;
                    }
                    let matched = match r.trigger {
                        Trigger::ShieldPressureVulnerable=>(matches!(t.status,0x1b|0x1c|0x1e)||roll(t.status)&&seen.from_shield)
                            &&t.vulnerable_in.is_some_and(|v|v.is_finite()&&v<=0.),
                        Trigger::EnemyShield => matches!(t.status, 0x1b | 0x1c | 0x1e),
                        Trigger::ShieldRoll => roll(t.status) && seen.from_shield,
                        Trigger::ShieldCross => roll(t.status) && seen.from_shield && seen.crossed,
                        Trigger::ShieldVulnerable => {
                            roll(t.status)
                                && seen.from_shield
                                && t.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        Trigger::ShieldCrossVulnerable => {
                            roll(t.status)
                                && seen.from_shield
                                && seen.crossed
                                && t.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        Trigger::ShieldAway => {
                            roll(t.status) && seen.from_shield && seen.away == Some(true)
                        }
                        Trigger::ShieldAwayVulnerable => {
                            roll(t.status)
                                && seen.from_shield
                                && seen.away == Some(true)
                                && t.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        _ => false,
                    };
                    if !matched {
                        continue;
                    }
                    let enemy = Opponent {
                        id: t.id,
                        status: t.status,
                        situation: 0,
                        position: t.position,
                        velocity: [t.velocity_x, 0.],
                        motion: Some(MotionTiming {
                            kind: t.motion,
                            frame: t.frame,
                            rate: t.rate,
                            remaining: t.actionable_in,
                        }),
                        vulnerable: t.vulnerable_in.is_some_and(|v|v.is_finite()&&v<=0.),
                        vulnerable_in: t.vulnerable_in,
                        ..Default::default()
                    };
                    if self.response.start(rules, r.trigger, c, enemy) {
                        self.seen[i].as_mut().unwrap().used = true;
                        following = Some(enemy);
                        break 'rules;
                    }
                }
            }
        }
        self.response.sample(true, settings, c, following)
    }
}
