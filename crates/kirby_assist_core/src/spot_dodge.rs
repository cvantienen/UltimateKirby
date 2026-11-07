//! Constant, input-only ordinary-grab defense. Root-space geometry is an
//! approximation, not proof of collision or a guaranteed dodge online.
use crate::fighter_tuning;
use crate::{gameplay::ControllerOutput, native_input::MAPPER_ZR, timing::motion_elapsed};
// Spot dodge becomes intangible after startup; six native frames gives the
// mapper/online queue margin without firing so early that normal dodge duration
// ends before an ordinary grab's active window.
pub const LOOKAHEAD: f32 = 6.0;
pub const ESCAPE: i32 = crate::native_states::ESCAPE;
#[derive(Clone, Copy)]
pub struct Context {
    pub object: u32,
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub can_spot_dodge: bool,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}
pub fn ground_status(status: i32) -> bool {
    crate::out_of_shield::stationary_grab_state(status)
}
// Held shields may have a stationary animation. Only these guard states may
// use a zero motion rate; native escape readiness still gates every new request.
fn stationary_guard(c: Context) -> bool {
    matches!(
        c.status,
        crate::native_states::GUARD_ON | crate::native_states::GUARD
    ) && c.rate == 0.0
}
fn valid_clock_rate(c: Context) -> bool {
    c.rate.is_finite() && (c.rate > 0.0 || stationary_guard(c))
}
#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub facing: f32,
    pub first: u8,
    pub last: u8,
    pub reach: f32,
}
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    status: i32,
    motion: u64,
    frame: f32,
    rate: f32,
    used: bool,
}

pub fn contact(c: Context, t: Target) -> Option<f32> {
    contact_with_lookahead(c, t, LOOKAHEAD)
}
pub fn contact_with_lookahead(c: Context, t: Target, lookahead: f32) -> Option<f32> {
    contact_with_shape(
        c,
        t,
        lookahead,
        fighter_tuning::DEFAULT_GRAB_DODGE_PADDING,
        fighter_tuning::DEFAULT_GRAB_DODGE_HEIGHT,
    )
}
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

#[derive(Default)]
pub struct SpotDodge {
    object: Option<u32>,
    previous: Option<(i32, u64, f32, f32)>,
    tracks: [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY],
    target: Option<u32>,
    age: f32,
    output: ControllerOutput,
    response: crate::defense_response::Response,
    punish_target: Option<u32>,
    mapped: u8,
    last_mapped: Option<i64>,
    dodge_delivered: bool,
    punish_clock: crate::ledge_trap::NativeAttempt,
}
impl SpotDodge {
    pub fn suspend_response(&mut self) {
        self.punish_target = None;
        self.punish_clock = Default::default();
        self.response.suspend();
    }
    /// Missing samples retain consumption; a positively observed end rearms the
    /// next native grab even when its first observed animation frame is later.
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
    pub fn cancel_response(&mut self) {
        self.response.cancel();
        self.punish_target = None;
        self.punish_clock = Default::default();
    }
    pub fn follow_target(&self) -> Option<u32> {
        self.response
            .target()
            .or(self.punish_target)
            .or(self.target)
    }
    pub fn direct_up_b(&self) -> bool {
        self.response.automatic_up_b()
    }
    pub fn up_b_delivery_started(&self) -> bool {
        self.response.up_b_delivery_started()
    }
    pub fn up_b_receipt_pending(&self) -> bool {
        self.response.up_b_receipt_pending()
    }
    pub fn awaiting_up_b(&self) -> bool {
        self.response.awaiting_up_b()
    }
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        self.response.attempted_up_special(tick, target);
    }
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        self.response.mapped_up_special(tick, target);
    }
    pub fn punish_committed(&self) -> bool {
        self.response.target().is_some()
    }
    /// A confirmed dodge and its configured follow-up are one response, even
    /// while native recovery requires an exclusive neutral packet.
    pub fn punish_pending(&self) -> bool {
        self.punish_target.is_some() || self.punish_committed()
    }
    pub fn grab_delivery_pending(&self) -> bool {
        self.response.grab_delivery_pending()
    }
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        self.response.mapped_grab(tick, target);
    }
    pub fn delivery_tick(&mut self, tick: i64) {
        self.response.delivery_tick(tick);
        if self.dodge_delivered {
            self.dodge_wait_tick(tick);
        }
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
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    pub fn punish_configured(
        &mut self,
        enabled: bool,
        can_act: bool,
        rules: Option<&crate::reaction_rules::Resolved>,
        settings: crate::ledge_trap::LedgeSettings,
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
    pub fn attempted_dodge(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) {
            self.dodge_wait_tick(tick);
        }
    }
    fn dodge_wait_tick(&mut self, tick: i64) {
        if self.target.is_some() && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// A follow-up requires our own observed ESCAPE from a pending warning.
    /// It reuses the saved spot-dodge Up-B recipe; no synthetic dodge success,
    /// forced cancel, extra roster pass or per-frame sequence copy is needed.
    pub fn punish_automatic(
        &mut self,
        allowed: bool,
        sequence: Option<&crate::sequence_config::Sequence>,
        settings: crate::ledge_trap::LedgeSettings,
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
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.mapped = 0;
        self.last_mapped = None;
        self.dodge_delivered = false;
        self.output = ControllerOutput::default();
    }
    fn mark_used(&mut self, id: u32) {
        for track in self.tracks.iter_mut().flatten() {
            if track.id == id {
                track.used = true;
            }
        }
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> ControllerOutput {
        self.sample_with_lookahead(enabled, c, targets, LOOKAHEAD)
    }
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
        let mut threat: Option<(u32, f32)> = None;
        self.tracks = [None; crate::battle_objects::FIGHTER_CAPACITY];
        for (slot, entry) in targets.iter().enumerate() {
            let Some(t) = entry else {
                continue;
            };
            if !matches!(
                t.status,
                crate::native_states::CATCH
                    | crate::native_states::CATCH_DASH
                    | crate::native_states::CATCH_TURN
            ) || t.motion == crate::timing::NO_MOTION_KIND
                || !t.frame.is_finite()
                || t.frame < 0.0
            {
                continue;
            }
            let used = old
                .iter()
                .flatten()
                .any(|p| p.id == t.id && p.status == t.status && p.motion == t.motion && p.used);
            self.tracks[slot] = Some(Track {
                id: t.id,
                status: t.status,
                motion: t.motion,
                frame: t.frame,
                rate: t.rate,
                used,
            });
            if !used && request_ready {
                if let Some(at) = contact_with_shape(c, *t, lookahead, padding, height) {
                    if threat.is_none_or(|(_, best)| at < best) {
                        threat = Some((t.id, at));
                    }
                }
            }
        }
        for previous in old
            .into_iter()
            .flatten()
            .filter(|p| p.used || Some(p.id) == self.target)
        {
            if self.tracks.iter().flatten().any(|t| t.id == previous.id)
                || targets
                    .iter()
                    .flatten()
                    .any(|t| t.id == previous.id && t.status != previous.status)
            {
                continue;
            }
            if let Some(empty) = self.tracks.iter_mut().find(|t| t.is_none()) {
                *empty = Some(previous);
            }
        }
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
