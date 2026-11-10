//! Configured spot-dodge punish, using existing bounded recovery observations.
use crate::{
    defense_response::Response,
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    option_read::{Kind, Target},
    reaction_rules::{Resolved, Trigger},
    settings::AssistSettings,
};
/// Represents an observed spot-dodge opportunity for a specific target.
/// # Fields
/// * `id` - The unique identifier of the target.
/// * `status` - The status of the target.
/// * `motion` - The motion kind of the target.
/// * `frame` - The frame at which the opportunity was observed.
/// * `used` - Indicates whether this opportunity has been used.
///
///  Represents an observed spot-dodge opportunity for a specific target. # Fields  id - The unique identifier of the target.  status - The status of the target.  motion - The motion kind of the target.  frame - The frame at which.
#[derive(Clone, Copy)]
struct Seen {
    /// Stores the `id` value in `Seen` as `u32`.
    ///
    ///
///  Holds the id value used by Seen.
    id: u32,
    /// Stores the `status` value in `Seen` as `i32`.
    ///
    ///
///  Holds the status value used by Seen.
    status: i32,
    /// Stores the `motion` value in `Seen` as `u64`.
    ///
    ///
///  Holds the motion value used by Seen.
    motion: u64,
    /// Stores the `frame` value in `Seen` as `f32`.
    ///
    ///
///  Holds the frame value used by Seen.
    frame: f32,
    /// Stores the `used` value in `Seen` as `bool`.
    ///
    ///
///  Holds the used value used by Seen.
    used: bool,
}

/// Represents a punisher that tracks observed spot-dodge opportunities and manages responses.
/// # Fields
/// * `object` - The current object being tracked.
/// * `seen` - An array of observed spot-dodge opportunities.
/// * `response` - The current response being managed.
///
///  Represents a punisher that tracks observed spot-dodge opportunities and manages responses. # Fields  object - The current object being tracked.  seen - An array of observed spot-dodge opportunities.  response - The current.
#[derive(Default)]
pub struct Punisher {
    /// Stores the `object` value in `Punisher` as `Option<u32>`.
    ///
    ///
///  Optionally stores the object for Punisher; it may be absent.
    object: Option<u32>,
    /// Stores the `seen` value in `Punisher` as `[Option<Seen>; 8]`.
    ///
    ///
///  Stores the seen collection for Punisher.
    seen: [Option<Seen>; 8],
    /// Stores the `response` value in `Punisher` as `Response`.
    ///
    ///
///  Holds the response value used by Punisher.
    response: Response,
}
impl Punisher {
    /// Returns the selected target observation.
    ///
    ///
///  Gets the chosen target.
    pub fn target(&self) -> Option<u32> {
        self.response.target()
    }
    /// Checks whether an input owner has committed the action.
    ///
    ///
///  Checks whether the action has been committed.
    pub fn committed(&self) -> bool {
        self.target().is_some()
    }
    /// Determines whether ground.
    ///
    ///
///  Checks whether ground.
    pub fn needs_ground(&self) -> bool {
        self.response.needs_ground()
    }
    /// Determines whether the `walking` condition holds.
    ///
    ///
///  Checks whether walking.
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    /// Determines whether the `attack_hold` condition holds.
    ///
    ///
///  Checks whether attack hold.
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    /// Returns an optional `right_stick` value.
    ///
    ///
///  Gets right stick if it is available.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.response.right_stick()
    }
    /// Cancels the pending.
    ///
    ///
///  Cancels the pending.
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
    }
    /// Samples the current gameplay context and updates the punisher's state accordingly.
    /// # Parameters
    /// * `enabled` - Indicates whether the punisher is enabled.
    /// * `object` - The current object being tracked.
    /// * `rules` - The resolved reaction rules, if any.
    /// * `settings` - The assist settings.
    /// * `c` - The current gameplay context.
    /// * `targets` - An array of potential targets.
    /// * `follow` - The opponent to follow, if any.
    /// # Returns
    /// The controller output based on the punisher's state and the current context.
    ///
///  Samples the current gameplay context and updates the punisher's state accordingly. # Parameters  enabled - Indicates whether the punisher is enabled.  object - The current object being tracked.  rules - The resolved reaction.
    pub fn sample(
        &mut self,
        enabled: bool,
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
        // Preserve the old seen array for comparison when updating the new seen array.
        let old = self.seen;
        // Update the seen array with the current targets, filtering and mapping them to Seen instances.
        self.seen = std::array::from_fn(|i| {
            targets[i]
                .filter(|target| {
                    target.kind == Kind::Spot
                        && target.status == crate::native_states::ESCAPE
                        && target.motion != crate::timing::NO_MOTION_KIND
                        && target.rate.is_finite()
                        && target.rate > 0.
                        && target.frame.is_finite()
                        && target.frame >= 0.
                        && target
                            .position
                            .iter()
                            .chain(target.velocity.iter())
                            .all(|v| v.is_finite())
                })
                .map(|target| Seen {
                    id: target.id,
                    status: target.status,
                    motion: target.motion,
                    frame: target.frame,
                    used: old.iter().flatten().any(|previous| {
                        crate::timing::same_motion_event(
                            (
                                previous.id,
                                previous.status,
                                previous.motion,
                                previous.frame,
                            ),
                            (target.id, target.status, target.motion, target.frame),
                        ) && previous.used
                    }),
                })
        });
        // Early exit if the punisher is not enabled, no rules are available, or gameplay is not active.
        if !enabled || rules.is_none() || !context.gameplay {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        let mut following = follow.filter(|e| Some(e.id) == self.target());
        if !self.committed() && !context.stopped {
            let rules = rules.unwrap();
            let mut best = None::<(u8, usize, Opponent)>;
            for (i, target) in targets.iter().enumerate() {
                let (Some(target), Some(seen)) = (target, self.seen[i]) else {
                    continue;
                };
                if seen.used {
                    continue;
                }
                let enemy = Opponent {
                    id: target.id,
                    status: target.status,
                    situation: crate::native_states::situation::GROUND,
                    position: target.position,
                    velocity: target.velocity,
                    motion: Some(crate::gameplay::MotionTiming {
                        kind: target.motion,
                        frame: target.frame,
                        rate: target.rate,
                        remaining: target.closes.is_finite().then_some(target.closes),
                    }),
                    vulnerable: target.opens.is_finite()
                        && target.opens <= 0.
                        && target.closes.is_finite()
                        && target.closes > 0.,
                    vulnerable_in: target.opens.is_finite().then_some(target.opens),
                    ..Default::default()
                };
                if let Some(sequence) =
                    crate::defense_response::select(rules, Trigger::SpotDodge, context, enemy)
                {
                    let priority = sequence.reaction.priority;
                    if best.is_none_or(|(p, _, _)| priority < p) {
                        best = Some((priority, i, enemy));
                    }
                }
            }
            if let Some((_, index, enemy)) = best {
                if self
                    .response
                    .start(rules, Trigger::SpotDodge, context, enemy)
                {
                    self.seen[index].as_mut().unwrap().used = true;
                    following = Some(enemy);
                }
            }
        }
        self.response.sample(true, settings, context, following)
    }
}
