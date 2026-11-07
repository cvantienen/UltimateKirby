//! Small revision-built observation metadata; no per-poll tape copies/scans.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleStart {
    pub distance: f32,
    pub height: f32,
    pub up_special: bool,
}

pub(super) fn needs_ground(sequence: &Sequence) -> bool {
    sequence.actions().iter().any(|action| {
        matches!(
            action,
            Action::Chase { .. }
                | Action::Walk { .. }
                | Action::Run { .. }
                | Action::Approach {
                    airborne: false,
                    ..
                }
        )
    })
}

impl Resolved {
    pub fn automatic_needs_up_special(&self, enabled_events: u8) -> bool {
        self.automatic_up_special & enabled_events != 0
    }
    pub fn simple_starts(&self) -> [Option<SimpleStart>; 3] {
        self.simple_starts
    }

    /// Automatic grabs, their backup and grab-dodge punishment have independent
    /// switches. Their movement checks must not depend on Auto-react's toggle.
    pub fn automatic_ground_watch(&self, enabled_events: u8) -> Option<(f32, f32)> {
        if enabled_events == 0 {
            return None;
        }
        let mut result: Option<(f32, f32)> = None;
        for (index, watch) in self.automatic_ground.iter().enumerate() {
            if enabled_events & (1 << index) == 0 {
                continue;
            }
            if let Some((distance, height)) = watch {
                let previous = result.unwrap_or((0., 0.));
                result = Some((previous.0.max(*distance), previous.1.max(*height)));
            }
        }
        result
    }
}

/// Native Up-B availability remains specific to Up-B recipes. Ordinary custom
/// responses use grounded control readiness, with a lazy shield-readiness query
/// only for an in-range candidate. All native queries happen before STATE locks.
pub fn simple_readiness(
    starts: [Option<SimpleStart>; 3],
    c: LedgeContext,
    targets: &[Option<Target>; 8],
    mut shield_ready: impl FnMut() -> bool,
) -> [bool; 3] {
    let mut result = [false; 3];
    if !c.gameplay
        || !automatic_start_allowed(c.status, c.situation, c.stopped, false)
        || starts.iter().all(Option::is_none)
    {
        return result;
    }
    let mut shield = None;
    for target in targets.iter().flatten() {
        let Some(event) = simple::event(target.status) else {
            continue;
        };
        let Some(start) = starts[event] else {
            continue;
        };
        if result[event]
            || !target.frame.is_finite()
            || target.frame < 0.
            || !target.rate.is_finite()
            || target.rate < 0.
            || !target
                .position
                .iter()
                .chain(c.position.iter())
                .all(|v| v.is_finite())
            || (target.position[0] - c.position[0]).abs() > start.distance
            || (target.position[1] - c.position[1]).abs() > start.height
        {
            continue;
        }
        result[event] = if start.up_special {
            c.can_act
        } else if !crate::out_of_shield::stationary_grab_state(c.status) {
            false
        } else if c.status == 0 || c.can_act {
            true
        } else {
            *shield.get_or_insert_with(&mut shield_ready)
        };
    }
    result
}

/// No engine is created for an unconfigured fighter/event group. Existing
/// engines are borrowed in place instead of taken and rebuilt every poll.
pub fn sample_optional_simple(
    engine: &mut Option<Box<Engine>>,
    allowed: bool,
    rules: Option<&Resolved>,
    settings: LedgeSettings,
    c: LedgeContext,
    targets: [Option<Target>; 8],
    follow: Option<Opponent>,
    readiness: [bool; 3],
) -> LedgeOutput {
    let Some(rules) = rules.filter(|rules| rules.has_simple_events()) else {
        *engine = None;
        return LedgeOutput::default();
    };
    if engine.is_none() && (!allowed || !readiness.into_iter().any(|ready| ready)) {
        return LedgeOutput::default();
    }
    engine
        .get_or_insert_with(Box::<Engine>::default)
        .sample_simple_ready(allowed, rules, settings, c, targets, follow, readiness)
}
