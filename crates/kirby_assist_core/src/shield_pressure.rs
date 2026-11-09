//! Shield-context reactions, bounded to the existing eight-fighter snapshot.
//! No native queries, guessed move timings, allocation or physics writes.
use crate::{
    defense_response::Response,
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    reaction_rules::{Resolved, Side, Trigger},
    roll_read::Target,
    settings::AssistSettings,
};
/// Represents an observed shield interaction for a specific target.
/// # Fields
/// * `id` - The unique identifier of the target.
/// * `status` - The status of the target.
/// * `motion` - The motion kind of the target.
/// * `frame` - The frame at which the interaction was observed.
/// * `dx` - The relative horizontal displacement from the player.
/// * `from_shield` - Indicates whether the interaction originated from a shield.
/// * `crossed` - Indicates whether the target has crossed the player.
/// * `start_side` - The initial side relative to the player.
/// * `away` - Indicates whether the target is moving away from the player.
/// * `used` - Indicates whether this interaction has been used.
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
/// Represents the shield pressure manager that tracks observed shield interactions and manages responses.
/// # Fields
/// * `object` - The current object being tracked.
/// * `seen` - An array of observed shield interactions.
/// * `response` - The current response being managed.
/// * `claim_rolls` - Indicates whether roll claims are active.
#[derive(Default)]
pub struct Pressure {
    object: Option<u32>,
    seen: [Option<Seen>; 8],
    response: Response,
    claim_rolls: bool,
}
/// Determines whether the given status corresponds to a shield context.
/// # Parameters
/// * `s` - The status to check.
/// # Returns
/// `true` if the status corresponds to a shield context, `false` otherwise.
fn shield_context(s: i32) -> bool {
    (crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE).contains(&s)
}
/// Determines whether the given status corresponds to a roll.
/// # Parameters
/// * `s` - The status to check.
/// # Returns
/// `true` if the status corresponds to a roll, `false` otherwise.
fn roll(s: i32) -> bool {
    matches!(
        s,
        crate::native_states::ESCAPE_F | crate::native_states::ESCAPE_B
    )
}
impl Pressure {
    /// Returns the current target being tracked by the pressure manager.
    pub fn target(&self) -> Option<u32> {
        self.response.target()
    }
    /// Returns whether the pressure manager has committed to a target.
    /// # Returns
    /// `true` if the pressure manager has committed to a target, `false` otherwise.
    pub fn committed(&self) -> bool {
        self.target().is_some()
    }
    /// Returns whether the pressure manager is currently walking.
    /// # Returns
    /// `true` if the pressure manager is walking, `false` otherwise.
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    /// Returns whether the pressure manager requires the ground.
    /// # Returns
    /// `true` if the pressure manager needs the ground, `false` otherwise.
    pub fn needs_ground(&self) -> bool {
        self.response.needs_ground()
    }
    /// Returns whether the pressure manager is holding an attack.
    /// # Returns
    /// `true` if the pressure manager is holding an attack, `false` otherwise.
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    /// Returns the current right stick input, if any.
    /// # Returns
    /// An array representing the right stick input, or `None` if not available.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.response.right_stick()
    }
    /// Cancels any pending response managed by the pressure manager.
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
    }
    /// Returns whether the pressure manager claims a roll for the specified target ID.
    /// # Parameters
    /// * `id` - The ID of the target to check.
    /// # Returns
    /// `true` if the pressure manager claims a roll for the target, `false` otherwise.
    pub fn claims_roll(&self, id: u32) -> bool {
        self.claim_rolls
            && self
                .seen
                .iter()
                .flatten()
                .any(|s| s.id == id && roll(s.status) && s.from_shield)
    }
    // Samples the current gameplay context and updates the pressure manager's state accordingly.
    /// # Parameters
    /// * `enabled` - Indicates whether the pressure manager is enabled.
    /// * `allowed` - Indicates whether the pressure manager is allowed to act.
    /// * `object` - The current object being tracked.
    /// * `rules` - The resolved reaction rules, if any.
    /// * `settings` - The assist settings.
    /// * `c` - The current gameplay context.
    /// * `targets` - An array of potential targets.
    /// * `follow` - The opponent to follow, if any.
    /// # Returns
    /// The controller output based on the pressure manager's state and the current context.
    pub fn sample(
        &mut self,
        enabled: bool,
        allowed: bool,
        object: u32,
        rules: Option<&Resolved>,
        settings: AssistSettings,
        context: GameplayContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
    ) -> ControllerOutput {
        if self.object != Some(object) {
            *self = Self::default();
            self.object = Some(object);
        }
        if !enabled || rules.is_none() || !context.gameplay {
            *self = Self::default();
            return ControllerOutput::default();
        }
        let rules = rules.unwrap();
        self.claim_rolls = Trigger::SHIELD_EVENTS[1..]
            .iter()
            .any(|t| rules.has_defense(*t));
        if !context.stopped {
            let old = self.seen;
            self.seen = std::array::from_fn(|i| {
                targets[i]
                    .filter(|target| {
                        (shield_context(target.status) || roll(target.status))
                            && target.motion != crate::timing::NO_MOTION_KIND
                            && target.frame.is_finite()
                            && target.frame >= 0.
                            && target.position.iter().all(|v| v.is_finite())
                    })
                    .map(|target| {
                        let before = old.iter().flatten().find(|s| s.id == target.id);
                        let same_roll = roll(target.status)
                            && before.is_some_and(|s| {
                                s.status == target.status && s.motion == target.motion && target.frame >= s.frame
                            });
                        let same_shield = shield_context(target.status)
                            && before.is_some_and(|s| shield_context(s.status));
                        let dx = target.position[0] - context.position[0];
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
                            (roll(target.status)
                                && target.velocity_x.is_finite()
                                && target.velocity_x.abs() > 0.001
                                && start_side.abs() > 0.001)
                                .then_some(target.velocity_x * start_side > 0.)
                        });
                        Seen {
                            id: target.id,
                            status: target.status,
                            motion: target.motion,
                            frame: target.frame,
                            start_side,
                            away,
                            dx: if dx.abs() > 0.001 {
                                dx
                            } else {
                                before.map_or(dx, |s| s.dx)
                            },
                            from_shield: roll(target.status)
                                && before.is_some_and(|s| {
                                    shield_context(s.status) || same_roll && s.from_shield
                                }),
                            crossed: roll(target.status)
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
        if !allowed || context.status == crate::native_states::GUARD_DAMAGE {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        // Samples the current gameplay context and updates the pressure manager's state accordingly.
        let mut following = follow.filter(|e| Some(e.id) == self.target());
        // Update the following target based on the current target ID.
        if !self.committed() && !context.stopped && context.can_act {
            'rules: for candidate in rules.rules.iter().flatten() {
                let rule = candidate.sequence.reaction;
                if !rule.trigger.shield_pressure() {
                    continue;
                }
                for (i, target) in targets.iter().enumerate() {
                    let (Some(target), Some(seen)) = (target, self.seen[i]) else {
                        continue;
                    };
                    if seen.used
                        || !rule.state.matches(context)
                        || !context.facing.is_finite()
                        || context.facing.abs() < 0.5
                    {
                        continue;
                    }
                    let dx = target.position[0] - context.position[0];
                    if !dx.is_finite()
                        || dx.abs() < rule.min
                        || dx.abs() > rule.max
                        || (target.position[1] - context.position[1]).abs() > rule.height
                        || matches!(rule.side, Side::Front) && dx * context.facing <= 0.001
                        || matches!(rule.side, Side::Behind) && dx * context.facing >= -0.001
                    {
                        continue;
                    }
                    let matched = match rule.trigger {
                        Trigger::ShieldPressureVulnerable => {
                            (matches!(
                                target.status,
                                crate::native_states::GUARD_ON
                                    | crate::native_states::GUARD
                                    | crate::native_states::GUARD_DAMAGE
                            ) || roll(target.status) && seen.from_shield)
                                && target.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        Trigger::EnemyShield => matches!(
                            target.status,
                            crate::native_states::GUARD_ON
                                | crate::native_states::GUARD
                                | crate::native_states::GUARD_DAMAGE
                        ),
                        Trigger::ShieldRoll => roll(target.status) && seen.from_shield,
                        Trigger::ShieldCross => roll(target.status) && seen.from_shield && seen.crossed,
                        Trigger::ShieldVulnerable => {
                            roll(target.status)
                                && seen.from_shield
                                && target.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        Trigger::ShieldCrossVulnerable => {
                            roll(target.status)
                                && seen.from_shield
                                && seen.crossed
                                && target.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        Trigger::ShieldAway => {
                            roll(target.status) && seen.from_shield && seen.away == Some(true)
                        }
                        Trigger::ShieldAwayVulnerable => {
                            roll(target.status)
                                && seen.from_shield
                                && seen.away == Some(true)
                                && target.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                        }
                        _ => false,
                    };
                    if !matched {
                        continue;
                    }
                    let enemy = Opponent {
                        id: target.id,
                        status: target.status,
                        situation: crate::native_states::situation::GROUND,
                        position: target.position,
                        velocity: [target.velocity_x, 0.],
                        motion: Some(MotionTiming {
                            kind: target.motion,
                            frame: target.frame,
                            rate: target.rate,
                            remaining: target.actionable_in,
                        }),
                        vulnerable: target.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.),
                        vulnerable_in: target.vulnerable_in,
                        ..Default::default()
                    };
                    if self.response.start(rules, rule.trigger, context, enemy) {
                        self.seen[i].as_mut().unwrap().used = true;
                        following = Some(enemy);
                        break 'rules;
                    }
                }
            }
        }
        self.response.sample(true, settings, context, following)
    }
}
