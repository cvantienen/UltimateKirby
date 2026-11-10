//! Small revision-built observation metadata; no per-poll tape copies/scans.
use super::*;

/// Defines the `SimpleStart` data structure.
///
/// Holds the data for SimpleStart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleStart {
    /// Stores the `distance` value in `SimpleStart` as `f32`.
    ///
    ///
///  Holds the distance value used by SimpleStart.
    pub distance: f32,
    /// Stores the `height` value in `SimpleStart` as `f32`.
    ///
    ///
///  Holds the height value used by SimpleStart.
    pub height: f32,
    /// Stores the `up_special` value in `SimpleStart` as `bool`.
    ///
    ///
///  Holds the up special value used by SimpleStart.
    pub up_special: bool,
}

/// Determines whether ground.
///
/// Checks whether ground.
/// ### Arguments
/// - `sequence` (`&Sequence`): Represents the sequence; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    /// Determines whether the `automatic_needs_up_special` condition holds.
    ///
    ///
    ///  Checks whether automatic needs up special.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled_events` (`u8`): Represents the enabled events; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn automatic_needs_up_special(&self, enabled_events: u8) -> bool {
        self.automatic_up_special & enabled_events != 0
    }
    /// Returns an optional `simple_starts` value.
    ///
    ///
    ///  Gets simple starts if it is available.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `[Option<SimpleStart>; 3]` value.
    pub fn simple_starts(&self) -> [Option<SimpleStart>; 3] {
        self.simple_starts
    }

    /// Automatic grabs, their backup and grab-dodge punishment have independent
    /// switches. Their movement checks must not depend on Auto-react's toggle.
    ///
    ///  Automatic grabs, their backup and grab-dodge punishment have independent switches. Their movement checks must not depend on Auto-react's toggle.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled_events` (`u8`): Represents the enabled events; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
///
///  Native Up-B availability remains specific to Up-B recipes. Ordinary custom responses use grounded control readiness, with a lazy shield-readiness query only for an in-range candidate. All native queries happen before STATE locks.
/// ### Arguments
/// - `starts` (`[Option<SimpleStart>; 3]`): Represents the starts; the function uses it as input to the operation described above.
/// - `c` (`GameplayContext`): Represents the c; the function uses it as input to the operation described above.
/// - `targets` (`&[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
/// - `shield_ready` (`impl FnMut() -> bool`): Represents the shield ready; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[bool; 3]` value.
pub fn simple_readiness(
    starts: [Option<SimpleStart>; 3],
    c: GameplayContext,
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
        } else if c.status == crate::native_states::status::WAIT || c.can_act {
            true
        } else {
            *shield.get_or_insert_with(&mut shield_ready)
        };
    }
    result
}

/// No engine is created for an unconfigured fighter/event group. Existing
/// engines are borrowed in place instead of taken and rebuilt every poll.
///
///  No engine is created for an unconfigured fighter/event group. Existing engines are borrowed in place instead of taken and rebuilt every poll.
/// ### Arguments
/// - `engine` (`&mut Option<Box<Engine>>`): Represents the engine; the function uses it as input to the operation described above.
/// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
/// - `rules` (`Option<&Resolved>`): Represents the rules; the function uses it as input to the operation described above.
/// - `settings` (`AssistSettings`): Represents the settings; the function uses it as input to the operation described above.
/// - `c` (`GameplayContext`): Represents the c; the function uses it as input to the operation described above.
/// - `targets` (`[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
/// - `follow` (`Option<Opponent>`): Represents the follow; the function uses it as input to the operation described above.
/// - `readiness` (`[bool; 3]`): Represents the readiness; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `ControllerOutput` value.
pub fn sample_optional_simple(
    engine: &mut Option<Box<Engine>>,
    allowed: bool,
    rules: Option<&Resolved>,
    settings: AssistSettings,
    c: GameplayContext,
    targets: [Option<Target>; 8],
    follow: Option<Opponent>,
    readiness: [bool; 3],
) -> ControllerOutput {
    let Some(rules) = rules.filter(|rules| rules.has_simple_events()) else {
        *engine = None;
        return ControllerOutput::default();
    };
    if engine.is_none() && (!allowed || !readiness.into_iter().any(|ready| ready)) {
        return ControllerOutput::default();
    }
    engine
        .get_or_insert_with(Box::<Engine>::default)
        .sample_simple_ready(allowed, rules, settings, c, targets, follow, readiness)
}
