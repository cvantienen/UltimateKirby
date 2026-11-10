//! Constant, input-only ordinary-grab defense. Root-space geometry is an
//! approximation, not proof of collision or a guaranteed dodge online.
use crate::fighter_tuning;
use crate::{gameplay::ControllerOutput, native_input::MAPPER_ZR, timing::motion_elapsed};
// Spot dodge becomes intangible after startup; six native frames gives the
// mapper/online queue margin without firing so early that normal dodge duration
// ends before an ordinary grab's active window.
/// Defines the `LOOKAHEAD` constant used by this module.
///
/// Provides the LOOKAHEAD constant value.
pub const LOOKAHEAD: f32 = 6.0;
/// Defines the `ESCAPE` constant used by this module.
///
/// Provides the ESCAPE constant value.
pub const ESCAPE: i32 = crate::native_states::ESCAPE;
/// Defines the `Context` data structure.
///
/// Holds the data for Context.
#[derive(Clone, Copy)]
pub struct Context {
    /// Stores the `object` value in `Context` as `u32`.
    ///
    ///
    ///  Holds the object value used by Context.
    pub object: u32,
    /// Stores the `status` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the status value used by Context.
    pub status: i32,
    /// Stores the `situation` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the situation value used by Context.
    pub situation: i32,
    /// Stores the `motion` value in `Context` as `u64`.
    ///
    ///
    ///  Holds the motion value used by Context.
    pub motion: u64,
    /// Stores the `frame` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the frame value used by Context.
    pub frame: f32,
    /// Stores the `rate` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the rate value used by Context.
    pub rate: f32,
    /// Stores the `stopped` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the stopped value used by Context.
    pub stopped: bool,
    /// Stores the `can_spot_dodge` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the can spot dodge value used by Context.
    pub can_spot_dodge: bool,
    /// Stores the `position` value in `Context` as `[f32; 2]`.
    ///
    ///
    ///  Stores the position collection for Context.
    pub position: [f32; 2],
    /// Stores the `velocity` value in `Context` as `[f32; 2]`.
    ///
    ///
    ///  Stores the velocity collection for Context.
    pub velocity: [f32; 2],
}
/// Determines whether the `ground_status` condition holds.
///
/// Checks whether ground status.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn ground_status(status: i32) -> bool {
    crate::out_of_shield::stationary_grab_state(status)
}
// Held shields may have a stationary animation. Only these guard states may
// use a zero motion rate; native escape readiness still gates every new request.
/// Determines whether the `stationary_guard` condition holds.
///
/// Checks whether stationary guard.
/// ### Arguments
/// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
fn stationary_guard(c: Context) -> bool {
    matches!(
        c.status,
        crate::native_states::GUARD_ON | crate::native_states::GUARD
    ) && c.rate == 0.0
}
/// Determines whether the `valid_clock_rate` condition holds.
///
/// Checks whether valid clock rate.
/// ### Arguments
/// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
fn valid_clock_rate(c: Context) -> bool {
    c.rate.is_finite() && (c.rate > 0.0 || stationary_guard(c))
}
/// Defines the `Target` data structure.
///
/// Holds the data for Target.
#[derive(Clone, Copy)]
pub struct Target {
    /// Stores the `id` value in `Target` as `u32`.
    ///
    ///
    ///  Holds the id value used by Target.
    pub id: u32,
    /// Stores the `status` value in `Target` as `i32`.
    ///
    ///
    ///  Holds the status value used by Target.
    pub status: i32,
    /// Stores the `motion` value in `Target` as `u64`.
    ///
    ///
    ///  Holds the motion value used by Target.
    pub motion: u64,
    /// Stores the `frame` value in `Target` as `f32`.
    ///
    ///
    ///  Holds the frame value used by Target.
    pub frame: f32,
    /// Stores the `rate` value in `Target` as `f32`.
    ///
    ///
    ///  Holds the rate value used by Target.
    pub rate: f32,
    /// Stores the `position` value in `Target` as `[f32; 2]`.
    ///
    ///
    ///  Stores the position collection for Target.
    pub position: [f32; 2],
    /// Stores the `velocity` value in `Target` as `[f32; 2]`.
    ///
    ///
    ///  Stores the velocity collection for Target.
    pub velocity: [f32; 2],
    /// Stores the `facing` value in `Target` as `f32`.
    ///
    ///
    ///  Holds the facing value used by Target.
    pub facing: f32,
    /// Stores the `first` value in `Target` as `u8`.
    ///
    ///
    ///  Holds the first value used by Target.
    pub first: u8,
    /// Stores the `last` value in `Target` as `u8`.
    ///
    ///
    ///  Holds the last value used by Target.
    pub last: u8,
    /// Stores the `reach` value in `Target` as `f32`.
    ///
    ///
    ///  Holds the reach value used by Target.
    pub reach: f32,
}
/// Defines the `Track` data structure.
///
/// Holds the data for Track.
#[derive(Clone, Copy)]
struct Track {
    /// Stores the `id` value in `Track` as `u32`.
    ///
    ///
    ///  Holds the id value used by Track.
    id: u32,
    /// Stores the `status` value in `Track` as `i32`.
    ///
    ///
    ///  Holds the status value used by Track.
    status: i32,
    /// Stores the `motion` value in `Track` as `u64`.
    ///
    ///
    ///  Holds the motion value used by Track.
    motion: u64,
    /// Stores the `frame` value in `Track` as `f32`.
    ///
    ///
    ///  Holds the frame value used by Track.
    frame: f32,
    /// Stores the `rate` value in `Track` as `f32`.
    ///
    ///
    ///  Holds the rate value used by Track.
    rate: f32,
    /// Stores the `used` value in `Track` as `bool`.
    ///
    ///
    ///  Holds the used value used by Track.
    used: bool,
}

/// Defines `Tracks` as a type alias.
///
/// Holds the Tracks value.
type Tracks = [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY];

/// Defines the `ThreatWindow` data structure.
///
/// Holds the data for ThreatWindow.
#[derive(Clone, Copy)]
struct ThreatWindow {
    /// Stores the `lookahead` value in `ThreatWindow` as `f32`.
    ///
    ///
    ///  Holds the lookahead value used by ThreatWindow.
    lookahead: f32,
    /// Stores the `padding` value in `ThreatWindow` as `f32`.
    ///
    ///
    ///  Holds the padding value used by ThreatWindow.
    padding: f32,
    /// Stores the `height` value in `ThreatWindow` as `f32`.
    ///
    ///
    ///  Holds the height value used by ThreatWindow.
    height: f32,
}

/// Checks whether grab target.
///
/// Checks whether grab target.
/// ### Arguments
/// - `target` (`Target`): Represents the target object or value being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
fn is_grab_target(target: Target) -> bool {
    matches!(
        target.status,
        crate::native_states::CATCH
            | crate::native_states::CATCH_DASH
            | crate::native_states::CATCH_TURN
    ) && target.motion != crate::timing::NO_MOTION_KIND
        && target.frame.is_finite()
        && target.frame >= 0.0
}

/// Returns an optional `track_grabs` value.
///
/// Gets track grabs if it is available.
/// ### Arguments
/// - `old` (`Tracks`): Represents the old; the function uses it as input to the operation described above.
/// - `selected_target` (`Option<u32>`): Represents the selected target; the function uses it as input to the operation described above.
/// - `targets` (`&[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
/// - `context` (`Context`): Represents the current execution context; the function uses it as input to the operation described above.
/// - `request_ready` (`bool`): Represents the request ready; the function uses it as input to the operation described above.
/// - `window` (`ThreatWindow`): Represents the window; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `(Tracks, Option<(u32, f32)>)` value.
fn track_grabs(
    old: Tracks,
    selected_target: Option<u32>,
    targets: &[Option<Target>; 8],
    context: Context,
    request_ready: bool,
    window: ThreatWindow,
) -> (Tracks, Option<(u32, f32)>) {
    let mut tracks = [None; crate::battle_objects::FIGHTER_CAPACITY];
    let mut threat: Option<(u32, f32)> = None;
    for (slot, entry) in targets.iter().enumerate() {
        let Some(target) = *entry else {
            continue;
        };
        if !is_grab_target(target) {
            continue;
        }
        let used = old.iter().flatten().any(|previous| {
            previous.id == target.id
                && previous.status == target.status
                && previous.motion == target.motion
                && previous.used
        });
        tracks[slot] = Some(Track {
            id: target.id,
            status: target.status,
            motion: target.motion,
            frame: target.frame,
            rate: target.rate,
            used,
        });
        if !used && request_ready {
            if let Some(contact) = contact_with_shape(
                context,
                target,
                window.lookahead,
                window.padding,
                window.height,
            ) {
                if threat.is_none_or(|(_, earliest)| contact < earliest) {
                    threat = Some((target.id, contact));
                }
            }
        }
    }

    for previous in old
        .into_iter()
        .flatten()
        .filter(|previous| previous.used || Some(previous.id) == selected_target)
    {
        let still_tracked = tracks.iter().flatten().any(|track| track.id == previous.id);
        let status_changed = targets
            .iter()
            .flatten()
            .any(|target| target.id == previous.id && target.status != previous.status);
        if still_tracked || status_changed {
            continue;
        }
        if let Some(empty) = tracks.iter_mut().find(|track| track.is_none()) {
            *empty = Some(previous);
        }
    }

    (tracks, threat)
}

/// Checks whether the observed interaction includes contact.
///
/// Checks whether contact was observed.
/// ### Arguments
/// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
/// - `t` (`Target`): Represents the t; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn contact(c: Context, t: Target) -> Option<f32> {
    contact_with_lookahead(c, t, LOOKAHEAD)
}
/// Returns an optional `contact_with_lookahead` value.
///
/// Gets contact with lookahead if it is available.
/// ### Arguments
/// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
/// - `t` (`Target`): Represents the t; the function uses it as input to the operation described above.
/// - `lookahead` (`f32`): Represents the lookahead; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn contact_with_lookahead(c: Context, t: Target, lookahead: f32) -> Option<f32> {
    contact_with_shape(
        c,
        t,
        lookahead,
        fighter_tuning::DEFAULT_GRAB_DODGE_PADDING,
        fighter_tuning::DEFAULT_GRAB_DODGE_HEIGHT,
    )
}
/// Returns an optional `contact_with_shape` value.
///
/// Gets contact with shape if it is available.
/// ### Arguments
/// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
/// - `t` (`Target`): Represents the t; the function uses it as input to the operation described above.
/// - `lookahead` (`f32`): Represents the lookahead; the function uses it as input to the operation described above.
/// - `padding` (`f32`): Represents the padding; the function uses it as input to the operation described above.
/// - `height` (`f32`): Represents the height; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn contact_with_shape(
    c: Context,
    t: Target,
    lookahead: f32,
    padding: f32,
    height: f32,
) -> Option<f32> {
    if !padding.is_finite()
        || !(0.0..=fighter_tuning::MAX_GRAB_DODGE_PADDING).contains(&padding)
        || !height.is_finite()
        || !(0.0..=fighter_tuning::MAX_GRAB_DODGE_HEIGHT).contains(&height)
    {
        return None;
    }
    if !lookahead.is_finite()
        || !(fighter_tuning::MIN_GRAB_DODGE_LOOKAHEAD_FRAMES
            ..=fighter_tuning::MAX_GRAB_DODGE_LOOKAHEAD_FRAMES)
            .contains(&lookahead)
    {
        return None;
    }
    if !matches!(
        t.status,
        crate::native_states::CATCH
            | crate::native_states::CATCH_DASH
            | crate::native_states::CATCH_TURN
    ) || t.motion == crate::timing::NO_MOTION_KIND
        || !t.frame.is_finite()
        || t.frame < 0.0
        || !t.rate.is_finite()
        || t.rate <= 0.0
        || !t.facing.is_finite()
        || t.facing.abs() < 0.5
        || t.first == 0
        || t.last < t.first
        || t.last > 120
        || !t.reach.is_finite()
        || !(1.0..=60.0).contains(&t.reach)
        || !c
            .position
            .iter()
            .chain(c.velocity.iter())
            .chain(t.position.iter())
            .chain(t.velocity.iter())
            .all(|v| v.is_finite())
    {
        return None;
    }
    let first = ((f32::from(t.first) - t.frame) / t.rate).max(0.0);
    // Chart active frames are inclusive. Frame 7 remains active at 7.5;
    // its exclusive end is 8.0. Do not discard the last active frame early.
    let last = (f32::from(t.last) + 1.0 - t.frame) / t.rate;
    if first > lookahead || last <= 0.0 {
        return None;
    }
    // Intersect the active-time interval with both relative-motion slabs.
    // Sampling every half frame can step over a fast/short-lived contact.
    // Two fixed axis checks replace up to 17 sampled position checks.
    let mut interval = [first, last.min(lookahead)];
    let relative = [
        (c.position[0] - t.position[0]) * t.facing.signum(),
        c.position[1] - t.position[1],
    ];
    let velocity = [
        (c.velocity[0] - t.velocity[0]) * t.facing.signum(),
        c.velocity[1] - t.velocity[1],
    ];
    for (position, speed, low, high) in [
        (relative[0], velocity[0], -padding, t.reach + padding),
        (relative[1], velocity[1], -height, height),
    ] {
        if !position.is_finite() || !speed.is_finite() {
            return None;
        }
        if speed == 0.0 {
            if !(low..=high).contains(&position) {
                return None;
            }
        } else {
            let a = (low - position) / speed;
            let b = (high - position) / speed;
            interval[0] = interval[0].max(a.min(b));
            interval[1] = interval[1].min(a.max(b));
            if interval[0] > interval[1] {
                return None;
            }
        }
    }
    (interval[0] < last).then_some(interval[0])
}

/// Defines the `SpotDodge` data structure.
///
/// Holds the data for SpotDodge.
#[derive(Default)]
pub struct SpotDodge {
    /// Stores the `object` value in `SpotDodge` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the object for SpotDodge; it may be absent.
    object: Option<u32>,
    /// Stores the `previous` value in `SpotDodge` as `Option<(i32`.
    ///
    ///
    ///  Optionally stores the previous for SpotDodge; it may be absent.
    previous: Option<(i32, u64, f32, f32)>,
    /// Stores the `tracks` value in `SpotDodge` as `[Option<Track>; crate::battle_objects::FIGHTER_CAPACITY]`.
    ///
    ///
    ///  Stores the tracks collection for SpotDodge.
    tracks: [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY],
    /// Stores the `target` value in `SpotDodge` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the target for SpotDodge; it may be absent.
    target: Option<u32>,
    /// Stores the `age` value in `SpotDodge` as `f32`.
    ///
    ///
    ///  Holds the age value used by SpotDodge.
    age: f32,
    /// Stores the `output` value in `SpotDodge` as `ControllerOutput`.
    ///
    ///
    ///  Holds the output value used by SpotDodge.
    output: ControllerOutput,
    /// Stores the `response` value in `SpotDodge` as `crate::defense_response::Response`.
    ///
    ///
    ///  Holds the response value used by SpotDodge.
    response: crate::defense_response::Response,
    /// Stores the `punish_target` value in `SpotDodge` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the punish target for SpotDodge; it may be absent.
    punish_target: Option<u32>,
    /// Stores the `mapped` value in `SpotDodge` as `u8`.
    ///
    ///
    ///  Holds the mapped value used by SpotDodge.
    mapped: u8,
    /// Stores the `last_mapped` value in `SpotDodge` as `Option<i64>`.
    ///
    ///
    ///  Optionally stores the last mapped for SpotDodge; it may be absent.
    last_mapped: Option<i64>,
    /// Stores the `dodge_delivered` value in `SpotDodge` as `bool`.
    ///
    ///
    ///  Holds the dodge delivered value used by SpotDodge.
    dodge_delivered: bool,
    /// Stores the `punish_clock` value in `SpotDodge` as `crate::ledge_trap::NativeAttempt`.
    ///
    ///
    ///  Holds the punish clock value used by SpotDodge.
    punish_clock: crate::ledge_trap::NativeAttempt,
}
impl SpotDodge {
    /// Updates the `suspend_response` state using the supplied inputs.
    ///
    ///
    ///  Updates suspend response.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn suspend_response(&mut self) {
        self.punish_target = None;
        self.punish_clock = Default::default();
        self.response.suspend();
    }
    /// Missing samples retain consumption; a positively observed end rearms the
    /// next native grab even when its first observed animation frame is later.
    ///
    ///  Missing samples retain consumption; a positively observed end rearms the next native grab even when its first observed animation frame is later.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `known` (`&[Option<(u32, i32)>; N]`): Represents the known; the function uses it as input to the operation described above.
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        for track in &mut self.tracks {
            if track.is_some_and(|t| {
                known
                    .iter()
                    .flatten()
                    .any(|(id, status)| *id == t.id && *status != t.status)
            }) {
                *track = None;
            }
        }
    }
    /// Cancels the response.
    ///
    ///
    ///  Cancels the response.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn cancel_response(&mut self) {
        self.response.cancel();
        self.punish_target = None;
        self.punish_clock = Default::default();
    }
    /// Returns an optional `follow_target` value.
    ///
    ///
    ///  Gets follow target if it is available.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn follow_target(&self) -> Option<u32> {
        self.response
            .target()
            .or(self.punish_target)
            .or(self.target)
    }
    /// Determines whether the `direct_up_b` condition holds.
    ///
    ///
    ///  Checks whether direct up b.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn direct_up_b(&self) -> bool {
        self.response.automatic_up_b()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    ///
    ///
    ///  Checks whether up b delivery started.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn up_b_delivery_started(&self) -> bool {
        self.response.up_b_delivery_started()
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    ///
    ///
    ///  Checks whether up b receipt pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.response.up_b_receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    ///
    ///
    ///  Checks whether awaiting up b.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn awaiting_up_b(&self) -> bool {
        self.response.awaiting_up_b()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates attempted up special.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        self.response.attempted_up_special(tick, target);
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped up special.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        self.response.mapped_up_special(tick, target);
    }
    /// Determines whether the `punish_committed` condition holds.
    ///
    ///
    ///  Checks whether punish committed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn punish_committed(&self) -> bool {
        self.response.target().is_some()
    }
    /// A confirmed dodge and its configured follow-up are one response, even
    /// while native recovery requires an exclusive neutral packet.
    ///
    ///  A confirmed dodge and its configured follow-up are one response, even while native recovery requires an exclusive neutral packet.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn punish_pending(&self) -> bool {
        self.punish_target.is_some() || self.punish_committed()
    }
    /// Determines whether the `grab_delivery_pending` condition holds.
    ///
    ///
    ///  Checks whether grab delivery pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn grab_delivery_pending(&self) -> bool {
        self.response.grab_delivery_pending()
    }
    /// Updates the `mapped_grab` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped grab.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        self.response.mapped_grab(tick, target);
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    ///
    ///
    ///  Updates delivery tick.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    pub fn delivery_tick(&mut self, tick: i64) {
        self.response.delivery_tick(tick);
        if self.dodge_delivered {
            self.dodge_wait_tick(tick);
        }
    }
    /// Determines whether ground.
    ///
    ///
    ///  Checks whether ground.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn needs_ground(&self) -> bool {
        self.response.needs_ground()
    }
    /// Determines whether the `attack_hold` condition holds.
    ///
    ///
    ///  Checks whether attack hold.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    /// Returns an optional `right_stick` value.
    ///
    ///
    ///  Gets right stick if it is available.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.response.right_stick()
    }
    /// Determines whether the `walking` condition holds.
    ///
    ///
    ///  Checks whether walking.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    /// Returns the `punish_configured` result.
    ///
    ///
    ///  Gets the punish configured result.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `can_act` (`bool`): Represents the can act; the function uses it as input to the operation described above.
    /// - `rules` (`Option<&crate::reaction_rules::Resolved>`): Represents the rules; the function uses it as input to the operation described above.
    /// - `settings` (`crate::settings::AssistSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `follow` (`Option<crate::gameplay::Opponent>`): Represents the follow; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn punish_configured(
        &mut self,
        enabled: bool,
        can_act: bool,
        rules: Option<&crate::reaction_rules::Resolved>,
        settings: crate::settings::AssistSettings,
        context: crate::gameplay::GameplayContext,
        follow: Option<crate::gameplay::Opponent>,
    ) -> ControllerOutput {
        if !enabled || rules.is_none() {
            self.response.cancel();
            self.punish_target = None;
            return ControllerOutput::default();
        }
        // The dodge and its configured punish share one native action owner.
        // A new dodge preempts an older punish; do not advance a hidden plan
        // that could later resume with stale attack/hold/C-stick output.
        if self.target.is_some() {
            self.cancel_response();
            return ControllerOutput::default();
        }
        if self.response.target().is_none() && self.punish_target.is_some() {
            if context.situation != crate::native_states::situation::GROUND
                || matches!(
                    context.status,
                    crate::native_states::THROW
                        ..=crate::native_states::status::DAMAGE_FALL
                            | crate::native_states::status::DEAD
                            | crate::native_states::status::REBIRTH
                )
            {
                self.punish_target = None;
            } else if can_act && !context.stopped {
                if let Some(enemy) = follow.filter(|e| Some(e.id) == self.punish_target) {
                    self.response.start(
                        rules.unwrap(),
                        crate::reaction_rules::Trigger::GrabDodged,
                        context,
                        enemy,
                    );
                }
                self.punish_target = None;
            }
        }
        self.response.sample(enabled, settings, context, follow)
    }
    /// One delivered Down+Guard packet, then neutral while awaiting native
    /// ESCAPE. Delivery is not a successful dodge or authorization to punish.
    ///
    ///  One delivered Down+Guard packet, then neutral while awaiting native ESCAPE. Delivery is not a successful dodge or authorization to punish.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_dodge(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) {
            self.dodge_delivered = true;
            self.mark_used(target);
            self.dodge_wait_tick(tick);
            self.output = crate::move_input::output([0.; 2], 0);
            self.output.owned = crate::ledge_trap::SEQUENCE_BUTTONS;
        }
    }
    /// Rejected mapped packets still spend the bounded attempt budget; they
    /// never count as delivery, native dodge success or a punish confirmation.
    ///
    ///  Rejected mapped packets still spend the bounded attempt budget; they never count as delivery, native dodge success or a punish confirmation.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn attempted_dodge(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) {
            self.dodge_wait_tick(tick);
        }
    }
    /// Updates the `dodge_wait_tick` state using the supplied inputs.
    ///
    ///
    ///  Updates dodge wait tick.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    fn dodge_wait_tick(&mut self, tick: i64) {
        if self.target.is_some() && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// A follow-up requires our own observed ESCAPE from a pending warning.
    /// It reuses the saved spot-dodge Up-B recipe; no synthetic dodge success,
    /// forced cancel, extra roster pass or per-frame sequence copy is needed.
    ///
    ///  A follow-up requires our own observed ESCAPE from a pending warning. It reuses the saved spot-dodge Up-B recipe; no synthetic dodge success, forced cancel, extra roster pass or per-frame sequence copy is needed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
    /// - `sequence` (`Option<&crate::sequence_config::Sequence>`): Represents the sequence; the function uses it as input to the operation described above.
    /// - `settings` (`crate::settings::AssistSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `follow` (`Option<crate::gameplay::Opponent>`): Represents the follow; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn punish_automatic(
        &mut self,
        allowed: bool,
        sequence: Option<&crate::sequence_config::Sequence>,
        settings: crate::settings::AssistSettings,
        context: crate::gameplay::GameplayContext,
        follow: Option<crate::gameplay::Opponent>,
    ) -> ControllerOutput {
        if !allowed
            || sequence.is_none()
            || !context.gameplay
            || crate::ledge_trap::sequence_interrupted(context.status)
        {
            self.suspend_response();
            return self.response.sample(false, settings, context, follow);
        }
        if self.target.is_some() {
            return ControllerOutput::default();
        }
        if self.response.target().is_some() {
            return self.response.sample(true, settings, context, follow);
        }
        let Some(id) = self.punish_target else {
            return ControllerOutput::default();
        };
        let Some(enemy) = follow.filter(|e| {
            e.id == id
                && e.position
                    .iter()
                    .chain(context.position.iter())
                    .all(|v| v.is_finite())
                && crate::reaction_rules::automatic_target_available(e.status)
        }) else {
            self.cancel_response();
            return ControllerOutput::default();
        };
        self.punish_clock.advance_pause(context, Some(enemy), 60);
        if context.situation != crate::native_states::situation::GROUND || self.punish_clock.expired
        {
            self.cancel_response();
            return ControllerOutput::default();
        }
        if context.status == ESCAPE && !context.can_act || context.stopped {
            return ControllerOutput::default();
        }
        // This is the first recovery opportunity, not a queued future attack.
        let sequence = sequence.unwrap();
        let r = sequence.reaction;
        self.punish_target = None;
        if !context.can_act
            || (enemy.position[0] - context.position[0]).abs() > r.max
            || (enemy.position[1] - context.position[1]).abs() > r.height
        {
            return ControllerOutput::default();
        }
        if !self.response.start_sequence(*sequence, context, enemy) {
            return ControllerOutput::default();
        }
        self.response.sample(true, settings, context, Some(enemy))
    }
    /// Returns the selected target observation.
    ///
    ///
    ///  Gets the chosen target.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Cancels the pending.
    ///
    ///
    ///  Cancels the pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.mapped = 0;
        self.last_mapped = None;
        self.dodge_delivered = false;
        self.output = ControllerOutput::default();
    }
    /// Marks the used.
    ///
    ///
    ///  Marks the used.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `id` (`u32`): Represents the identifier of an object or entry; the function uses it as input to the operation described above.
    fn mark_used(&mut self, id: u32) {
        for track in self.tracks.iter_mut().flatten() {
            if track.id == id {
                track.used = true;
            }
        }
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> ControllerOutput {
        self.sample_with_lookahead(enabled, c, targets, LOOKAHEAD)
    }
    /// Samples the with lookahead.
    ///
    ///
    ///  Samples the with lookahead.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// - `lookahead` (`f32`): Represents the lookahead; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_with_lookahead(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
        lookahead: f32,
    ) -> ControllerOutput {
        self.sample_with_shape(
            enabled,
            c,
            targets,
            lookahead,
            fighter_tuning::DEFAULT_GRAB_DODGE_PADDING,
            fighter_tuning::DEFAULT_GRAB_DODGE_HEIGHT,
        )
    }
    /// Samples the with shape.
    ///
    ///
    ///  Samples the with shape.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// - `lookahead` (`f32`): Represents the lookahead; the function uses it as input to the operation described above.
    /// - `padding` (`f32`): Represents the padding; the function uses it as input to the operation described above.
    /// - `height` (`f32`): Represents the height; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_with_shape(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
        lookahead: f32,
        padding: f32,
        height: f32,
    ) -> ControllerOutput {
        if self.object != Some(c.object) {
            *self = Self::default();
            self.object = Some(c.object);
        }
        // Frozen native progress must not bypass the real delivery budget.
        // An observed ESCAPE still takes the acceptance path below.
        if c.status != ESCAPE && self.mapped >= 24 {
            if let Some(id) = self.target {
                self.mark_used(id);
            }
            self.cancel_pending();
            return self.output;
        }
        if enabled
            && c.stopped
            && c.situation == crate::native_states::situation::GROUND
            && ground_status(c.status)
            && c.motion != crate::timing::NO_MOTION_KIND
            && c.frame.is_finite()
            && c.frame >= 0.0
            && valid_clock_rate(c)
        {
            return self.output;
        }
        let old = self.tracks;
        // Still observe identity/progress for existing releases, but avoid
        // collision forecasts when this sample cannot request a dodge.
        let request_ready = enabled
            && !c.stopped
            && c.situation == crate::native_states::situation::GROUND
            && ground_status(c.status)
            && c.can_spot_dodge
            && self.punish_target.is_none()
            && !self.response.configured_committed();
        let (tracks, threat) = track_grabs(
            old,
            self.target,
            &targets,
            c,
            request_ready,
            ThreatWindow {
                lookahead,
                padding,
                height,
            },
        );
        self.tracks = tracks;
        if c.status == ESCAPE {
            if let Some(id) = self.target {
                if enabled {
                    self.punish_target = Some(id);
                    self.punish_clock = Default::default();
                }
                self.mark_used(id);
            }
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        if !enabled
            || c.situation != crate::native_states::situation::GROUND
            || c.motion == crate::timing::NO_MOTION_KIND
            || !c.frame.is_finite()
            || c.frame < 0.0
            || !valid_clock_rate(c)
            || !ground_status(c.status)
        {
            // No observed dodge means this grab has not been handled. A
            // temporary uninterruptible state must not suppress a later legal
            // request for the same still-threatening grab.
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        // Internal clock metadata only, not a native animation-rate write.
        // A stationary shield contributes zero progress; the existing target
        // clock below supplies real elapsed time for retries/releases.
        let rate = if stationary_guard(c) { 1.0 } else { c.rate };
        let stamp = (c.status, c.motion, c.frame, rate);
        let mut elapsed = motion_elapsed(self.previous, stamp, 0.0);
        if elapsed == 0.0 {
            if let Some(t) = targets.iter().flatten().find(|t| Some(t.id) == self.target) {
                if let Some(p) = old
                    .iter()
                    .flatten()
                    .find(|p| p.id == t.id && p.status == t.status && p.motion == t.motion)
                {
                    elapsed = motion_elapsed(
                        Some((p.status, p.motion, p.frame, p.rate)),
                        (t.status, t.motion, t.frame, t.rate),
                        0.0,
                    );
                }
            }
        }
        self.previous = Some(stamp);
        if !elapsed.is_finite() || !(0.0..=8.0).contains(&elapsed) {
            self.cancel_pending();
            return self.output;
        }
        if !c.can_spot_dodge {
            // Readiness applies to retries too. Keep this grab eligible when
            // native recovery temporarily prevents the unaccepted request.
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        if let Some(id) = self.target {
            self.age += elapsed;
            if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT
                || self.mapped >= 24
                || !targets.iter().flatten().any(|t| {
                    t.id == id && contact_with_shape(c, *t, lookahead, padding, height).is_some()
                })
            {
                // Only exhaust an attempt after its bounded delivery budget.
                // A changed forecast may become threatening again before the
                // grab ends; native ESCAPE is the actual acceptance signal.
                if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT || self.mapped >= 24 {
                    self.mark_used(id);
                }
                self.cancel_pending();
            } else if self.dodge_delivered {
                self.output = crate::move_input::output([0.; 2], 0);
                self.output.owned = crate::ledge_trap::SEQUENCE_BUTTONS;
            } else {
                // One continuous Down+Guard request. Re-flicking Down every
                // two frames can queue a second dodge behind the first online.
                // Native ESCAPE confirmation releases both sources immediately.
                self.output = crate::move_input::output([0.0, -1.0], MAPPER_ZR);
                self.output.owned = crate::ledge_trap::SEQUENCE_BUTTONS;
            }
            return self.output;
        }
        self.output = ControllerOutput::default();
        if let Some((id, _)) = threat {
            self.target = Some(id);
            self.age = 0.0;
            self.mapped = 0;
            self.last_mapped = None;
            // Down+Guard requests spot dodge, with no horizontal roll input.
            self.output = crate::move_input::output([0.0, -1.0], MAPPER_ZR);
            self.output.owned = crate::ledge_trap::SEQUENCE_BUTTONS;
        }
        self.output
    }
}
pub mod roll_read {
    //! Shared copied opponent facts used by reaction and out-of-shield policies.

    /// Defines the `ESCAPE_F` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE F constant value.
    pub const ESCAPE_F: i32 = crate::native_states::ESCAPE_F;
    /// Defines the `ESCAPE_B` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE B constant value.
    pub const ESCAPE_B: i32 = crate::native_states::ESCAPE_B;

    /// Defines the `Target` data structure.
    ///
    ///
    ///  Holds the data for Target.
    #[derive(Clone, Copy)]
    pub struct Target {
        /// Stores the `id` value in `Target` as `u32`.
        ///
        ///
        ///  Holds the id value used by Target.
        pub id: u32,
        /// Stores the `status` value in `Target` as `i32`.
        ///
        ///
        ///  Holds the status value used by Target.
        pub status: i32,
        /// Copied native motion identity for the configurable rule clock.
        ///
        ///  Copied native motion identity for the configurable rule clock.
        pub motion: u64,
        /// Stores the `frame` value in `Target` as `f32`.
        ///
        ///
        ///  Holds the frame value used by Target.
        pub frame: f32,
        /// Stores the `rate` value in `Target` as `f32`.
        ///
        ///
        ///  Holds the rate value used by Target.
        pub rate: f32,
        /// Stores the `position` value in `Target` as `[f32; 2]`.
        ///
        ///
        ///  Stores the position collection for Target.
        pub position: [f32; 2],
        /// Stores the `velocity_x` value in `Target` as `f32`.
        ///
        ///
        ///  Holds the velocity x value used by Target.
        pub velocity_x: f32,
        /// Stores the `vulnerable_in` value in `Target` as `Option<f32>`.
        ///
        ///
        ///  Optionally stores the vulnerable in for Target; it may be absent.
        pub vulnerable_in: Option<f32>,
        /// Stores the `actionable_in` value in `Target` as `Option<f32>`.
        ///
        ///
        ///  Optionally stores the actionable in for Target; it may be absent.
        pub actionable_in: Option<f32>,
    }
    impl Target {
        /// Determines whether the `vulnerable_now` condition holds.
        ///
        ///
        ///  Checks whether vulnerable now.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn vulnerable_now(&self) -> bool {
            self.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
        }
        /// Copied facts as an `Opponent`. The caller knows the actor's native
        /// situation and which recovery clock, if any, applies.
        ///
        ///  Copied facts as an Opponent. The caller knows the actor's native situation and which recovery clock, if any, applies.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
        /// - `remaining` (`Option<f32>`): Represents the remaining; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `crate::gameplay::Opponent` value.
        pub(crate) fn opponent(
            &self,
            situation: i32,
            remaining: Option<f32>,
        ) -> crate::gameplay::Opponent {
            crate::gameplay::Opponent {
                id: self.id,
                status: self.status,
                situation,
                position: self.position,
                velocity: [self.velocity_x, 0.],
                motion: Some(crate::gameplay::MotionTiming {
                    kind: self.motion,
                    frame: self.frame,
                    rate: self.rate,
                    remaining,
                }),
                vulnerable: self.vulnerable_now(),
                vulnerable_in: self.vulnerable_in,
                ..Default::default()
            }
        }
    }
}
