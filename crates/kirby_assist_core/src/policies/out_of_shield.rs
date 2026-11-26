//! Confirmed shield-contact responses. Inputs only; never force native states.
use crate::{
    gameplay::{ControllerOutput, Opponent},
    roll_read::Target as ObservedTarget,
};
mod automatic;
mod simple;
pub use automatic::{AutomaticConfig, AutomaticLimits};
mod shield_up_b;

/// Returns the `copied_target` result.
/// Gets the copied target result.
fn copied_target(target: &ObservedTarget) -> Opponent {
    target.opponent(crate::native_states::situation::GROUND, None)
}

/// The locked target: live follow facts first, else the copied snapshot.
pub(super) fn locked_enemy(
    id: Option<u32>,
    follow: Option<Opponent>,
    targets: &[Option<ObservedTarget>; 8],
) -> Option<Opponent> {
    follow.filter(|e| Some(e.id) == id).or_else(|| {
        targets
            .iter()
            .flatten()
            .find(|t| Some(t.id) == id)
            .map(copied_target)
    })
}

/// The horizontally nearest acceptable fighter; snapshot targets precede `follow`.
pub(super) fn nearest_enemy(
    origin: f32,
    targets: &[Option<ObservedTarget>; 8],
    follow: Option<Opponent>,
    mut accept: impl FnMut(Opponent) -> bool,
) -> Option<Opponent> {
    targets
        .iter()
        .flatten()
        .map(copied_target)
        .chain(follow)
        .filter(|e| accept(*e))
        .min_by(|a, b| {
            (a.position[0] - origin)
                .abs()
                .total_cmp(&(b.position[0] - origin).abs())
        })
}

/// Determines whether the `target_within_range` condition holds.
/// Checks whether target within range.
pub(super) fn target_within_range(
    origin: [f32; 2],
    target: [f32; 2],
    range: f32,
    height: f32,
) -> bool {
    range.is_finite()
        && range > 0.
        && height.is_finite()
        && height >= 0.
        && origin
            .iter()
            .chain(target.iter())
            .all(|value| value.is_finite())
        && (target[0] - origin[0]).abs() <= range
        && (target[1] - origin[1]).abs() <= height
}

/// Retained stationary gate for incoming-grab avoidance and legacy reactions.
pub fn stationary_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::WAIT
            | crate::native_states::GUARD_ON
            | crate::native_states::GUARD
    )
}
/// Only shield states may start an automatic shield grab.
pub fn ground_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::GUARD_ON | crate::native_states::GUARD
    )
}
/// Held guard permits direct shield grab using native standing-catch readiness.
pub fn automatic_grab_ready(status: i32, enabled: impl FnMut(i32) -> bool) -> bool {
    automatic_grab_ready_with_shield(status, true, enabled)
}
/// A fresh physical shield press counts before the native status catches up.
pub fn automatic_grab_ready_with_shield(
    status: i32,
    shielded: bool,
    mut enabled: impl FnMut(i32) -> bool,
) -> bool {
    shielded
        && ground_grab_state(status)
        && (status == crate::native_states::GUARD
            || enabled(crate::native_states::transition_terms::CONT_CATCH))
}
/// Shield grabs may punish ordinary hittable opponents without enemy shield.
/// Downed/tripped bodies cannot be ordinary-grab targets; shield Up-B may hit
/// them instead. Current vulnerability/capture links and reach are checked too.
pub fn automatic_grab_target_status_with_shield(status: i32, shielded: bool) -> bool {
    shielded
        && automatic_shield_target_status(status)
        && !crate::native_states::ordinary_grab_immune(status)
}
/// Up-B may hit a vulnerable downed opponent that cannot be grabbed. Keep
/// captures, ledge visits and noncombat states outside the ordinary OOS policy.
pub fn automatic_shield_target_status(status: i32) -> bool {
    crate::reaction_rules::automatic_target_available(status)
}

/// Keep guard for a direct OOS input, rather than requesting shield release and
/// the attack together. Movement/ordinary attacks keep the tape's neutral drop.
pub(crate) fn direct_shield_input(
    status: i32,
    mut out: ControllerOutput,
    right: [f32; 2],
) -> ControllerOutput {
    use crate::native_input::*;
    if matches!(
        status,
        crate::native_states::GUARD_ON | crate::native_states::GUARD
    ) && out
        .stick
        .is_some_and(|stick| is_direct_shield_cancel(out.requested, stick, right))
    {
        out.requested |= MAPPER_ZR;
    }
    out
}

/// Controller inputs that can cancel shield without a shield-drop handoff.
pub(crate) fn is_direct_shield_cancel(buttons: u32, stick: [f32; 2], right: [f32; 2]) -> bool {
    use crate::native_input::{MAPPER_A, MAPPER_B, MAPPER_R, SEQUENCE_JUMP_BUTTONS};
    buttons & MAPPER_R != 0 && stick[1] >= 0.
        || buttons & SEQUENCE_JUMP_BUTTONS != 0 && stick[1] >= 0.
        || buttons & MAPPER_A != 0 && stick[1] >= 0.95
        || buttons & MAPPER_B != 0 && stick[1] > 0.5
        || right[1] > 0.5
}

/// Runtime Smash data carried by `Target`.
#[derive(Clone, Copy)]
pub struct Target {
    /// battle-object ID.
    pub id: u32,
    /// world-space position.
    pub position: [f32; 2],
    /// world-space velocity.
    pub velocity: [f32; 2],
    /// grounded-state observation.
    pub grounded: bool,
    /// Runtime dangerous value used by Target..
    pub dangerous: bool,
    /// Runtime recovery value used by Target..
    pub recovery: Option<f32>,
}
/// Runtime Smash data carried by `Context`.
#[derive(Clone, Copy)]
pub struct Context {
    /// Runtime object value used by Context..
    pub object: u32,
    /// native fighter-kind ID.
    pub kind: i32,
    /// native fighter status.
    pub status: i32,
    /// native animation/motion ID.
    pub motion: u64,
    /// current native animation frame.
    pub frame: f32,
    /// native animation playback rate.
    pub rate: f32,
    /// Runtime stopped value used by Context..
    pub stopped: bool,
    /// grounded-state observation.
    pub grounded: bool,
    /// world-space position.
    pub position: [f32; 2],
    /// left/right facing sign.
    pub facing: f32,
    /// Runtime shield ready value used by Context..
    pub shield_ready: bool,
    /// Runtime can grab value used by Context..
    pub can_grab: bool,
    /// Runtime can jump value used by Context..
    pub can_jump: bool,
    /// Runtime can up special value used by Context..
    pub can_up_special: bool,
    /// Runtime threatened value used by Context..
    pub threatened: bool,
}
/// Runtime Smash data carried by `OutOfShield`.
#[derive(Default)]
pub struct OutOfShield {
    /// Runtime object value used by OutOfShield..
    object: Option<u32>,
    /// native fighter-kind ID.
    kind: Option<i32>,
    /// selected opponent/projectile target.
    target: Option<u32>,
    /// Runtime guarded value used by OutOfShield..
    guarded: bool,
    /// Runtime response value used by OutOfShield..
    response: crate::defense_response::Response,
    /// Runtime simple value used by OutOfShield..
    simple: simple::Grab,
    /// Runtime shield up b value used by OutOfShield..
    shield_up_b: shield_up_b::ShieldUpB,
    /// Runtime configured response value used by OutOfShield..
    configured_response: bool,
    /// Runtime configured backup value used by OutOfShield..
    configured_backup: bool,
    /// Runtime configured shielded value used by OutOfShield..
    configured_shielded: bool,
    /// Runtime configured grab wait value used by OutOfShield..
    configured_grab_wait: bool,
    /// Runtime configured settling value used by OutOfShield..
    configured_settling: bool,
    guard_hold: f32,
    guard_clock: crate::ledge_trap::NativeAttempt,
}

impl OutOfShield {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            object: None,
            kind: None,
            target: None,
            guarded: false,
            response: crate::defense_response::Response::new(),
            simple: simple::Grab::new(),
            shield_up_b: shield_up_b::ShieldUpB::new(),
            configured_response: false,
            configured_backup: false,
            configured_shielded: false,
            configured_grab_wait: false,
            configured_settling: false,
            guard_hold: 0.,
            guard_clock: crate::ledge_trap::NativeAttempt::new(),
        }
    }

    /// Updates the `bind_actor` state using the supplied inputs.
    fn bind_actor(&mut self, c: Context) {
        if self.object != Some(c.object) || self.kind != Some(c.kind) {
            *self = Self::default();
            self.object = Some(c.object);
            self.kind = Some(c.kind);
        }
    }

    /// Returns the selected target observation.
    pub fn target(&self) -> Option<u32> {
        self.simple
            .target()
            .or(self.shield_up_b.target())
            .or(self.target)
            .or(self.response.target())
    }
    /// Cancels the pending.
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
        self.configured_response = false;
        self.configured_backup = false;
        self.configured_shielded = false;
        self.configured_grab_wait = false;
        self.configured_settling = false;
        self.guard_hold = 0.;
        self.guard_clock = crate::ledge_trap::NativeAttempt::new();
        self.simple.cancel();
        self.shield_up_b.cancel();
        self.target = None;
    }
    /// Ledge/manual takeover releases inputs immediately. Delivered R retains
    /// only its bounded receipt so a competing script cannot turn it into Up-B.
    pub fn suspend_automatic(&mut self) {
        self.guard_hold = 0.;
        self.guard_clock = crate::ledge_trap::NativeAttempt::new();
        self.shield_up_b.cancel();
        self.response.suspend();
        if !self.simple.delivery_pending() {
            self.simple.cancel();
        }
        self.configured_response = self.response.grab_delivery_pending();
        self.configured_settling = self.simple.delivery_pending();
        self.target = None;
    }
    /// Acknowledge only an input packet actually passed to the native mapper.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        self.simple.mapped_grab(tick, target);
        if self.configured_response {
            self.response.mapped_grab(tick, target);
        }
    }
    /// Determines whether the `configured_committed` condition holds.
    pub fn configured_committed(&self) -> bool {
        self.configured_response
    }
    /// Determines whether ground.
    pub fn needs_ground(&self) -> bool {
        self.configured_response && self.response.needs_ground()
    }
    /// Determines whether the `attack_hold` condition holds.
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    /// Returns an optional `right_stick` value.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        if self.configured_response {
            self.response.right_stick()
        } else if self.simple.active() || self.shield_up_b.active() {
            Some([0.; 2])
        } else {
            self.response.right_stick()
        }
    }
    /// Determines whether the `walking` condition holds.
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    /// Samples the simple.
    pub fn sample_simple(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        attacker: Option<crate::gameplay::Opponent>,
        range: f32,
        _legacy_far: f32,
    ) -> ControllerOutput {
        let startup = crate::fighter_data::ordinary_grab(c.kind, crate::native_states::CATCH)
            .map(|g| f32::from(g.first));
        self.sample_simple_timed(allowed, c, context, targets, attacker, range, startup)
    }
    /// The live adapter supplies this fighter's resolved standing/shield startup.
    /// No new delay: this only rejects projected misses before pressing Grab.
    pub fn sample_simple_timed(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        attacker: Option<crate::gameplay::Opponent>,
        range: f32,
        startup: Option<f32>,
    ) -> ControllerOutput {
        self.bind_actor(c);
        self.shield_up_b.cancel();
        self.simple.sample(
            allowed,
            c,
            context,
            targets,
            attacker,
            simple::Reach {
                range,
                height: 24.,
                startup,
                shielded: matches!(
                    c.status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                ),
            },
        )
    }
    /// Front vulnerable-target grab: direct neutral-axis R while shielding.
    pub fn sample_automatic(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<crate::gameplay::Opponent>,
        shielded: bool,
        close: f32,
        _legacy_approach: f32,
        startup: Option<f32>,
    ) -> ControllerOutput {
        self.sample_automatic_configured(
            allowed,
            c,
            context,
            targets,
            follow,
            shielded,
            close,
            _legacy_approach,
            startup,
            crate::settings::AssistSettings {
                fighter_kind: c.kind,
                ..Default::default()
            },
            AutomaticConfig::default(),
        )
    }
    /// Determines whether the `shield_up_b_pending` condition holds.
    pub fn shield_up_b_pending(&self) -> bool {
        self.shield_up_b.active() || self.configured_response && self.response.automatic_up_b()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    pub fn up_b_delivery_started(&self) -> bool {
        self.shield_up_b.delivery_started()
            || self.configured_response && self.response.up_b_delivery_started()
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.shield_up_b.receipt_pending()
            || self.configured_response && self.response.up_b_receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    pub fn awaiting_up_b(&self) -> bool {
        self.shield_up_b.awaiting() || self.configured_response && self.response.awaiting_up_b()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.attempted_up_special(tick, target);
        } else {
            self.shield_up_b.attempted(tick, target);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.mapped_up_special(tick, target);
        } else {
            self.shield_up_b.mapped(tick, target);
        }
    }
    /// Determines whether the `grab_delivery_pending` condition holds.
    pub fn grab_delivery_pending(&self) -> bool {
        self.simple.delivery_pending() || self.response.grab_delivery_pending()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    pub fn delivery_tick(&mut self, tick: i64) {
        self.shield_up_b.delivery_tick(tick);
        self.simple.delivery_tick(tick);
        self.response.delivery_tick(tick);
    }
    /// Determines whether the `automatic_grab_pending` condition holds.
    pub fn automatic_grab_pending(&self) -> bool {
        self.simple.active() && !self.configured_response && !self.configured_settling
    }
    /// Samples the configured.
    pub fn sample_configured(
        &mut self,
        enabled: bool,
        rules: Option<&crate::reaction_rules::Resolved>,
        settings: crate::settings::AssistSettings,
        c: Context,
        context: crate::gameplay::GameplayContext,
        target: Option<Target>,
        follow: Option<crate::gameplay::Opponent>,
    ) -> ControllerOutput {
        self.simple.cancel();
        self.shield_up_b.cancel();
        if !enabled || rules.is_none() {
            self.cancel_pending();
            self.guarded = c.status == crate::native_states::GUARD_DAMAGE;
            return ControllerOutput::default();
        }
        self.bind_actor(c);
        if let Some(previous_attacker) = self.response.target() {
            let renewed_hit = c.status == crate::native_states::GUARD_DAMAGE
                && (!self.guarded
                    || c.stopped
                    || !c.shield_ready
                    || target.is_some_and(|t| t.id != previous_attacker));
            if renewed_hit {
                // A new block invalidates the old response's timing/attacker.
                // Re-arm from this contact; never buffer an old Up-B through
                // renewed shieldstun and replay it against a different attacker.
                self.cancel_pending();
                self.guarded = false;
            } else {
                self.guarded = c.status == crate::native_states::GUARD_DAMAGE;
                return self.response.sample(enabled, settings, context, follow);
            }
        }
        let hit = c.status == crate::native_states::GUARD_DAMAGE;
        if hit
            && (!self.guarded || target.is_some_and(|t| self.target.is_some_and(|id| id != t.id)))
        {
            self.target = target.map(|t| t.id);
        }
        self.guarded = hit;
        if !c.grounded
            || !matches!(
                c.status,
                crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
            )
        {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        let Some(t) = target.filter(|t| Some(t.id) == self.target) else {
            self.target = None;
            return ControllerOutput::default();
        };
        if c.stopped || !c.shield_ready {
            return ControllerOutput::default();
        }
        // One response per confirmed contact. No waiting in shield for a later
        // range/recovery window, and no hard-coded fighter move or startup.
        self.target = None;
        if let Some(enemy) = follow.filter(|e| e.id == t.id) {
            self.response.start(
                rules.unwrap(),
                crate::reaction_rules::Trigger::ShieldHit,
                context,
                enemy,
            );
        }
        self.response.sample(enabled, settings, context, follow)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/policy_out_of_shield.rs"]
mod tests;
