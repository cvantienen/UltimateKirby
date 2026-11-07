//! Confirmed shield-contact responses. Inputs only; never force native states.
use crate::gameplay::ControllerOutput;
mod automatic;
mod simple;
pub use automatic::{AutomaticConfig, AutomaticLimits};
mod shield_up_b;

/// Retained stationary gate for incoming-grab avoidance and legacy reactions.
pub fn stationary_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::WAIT
            | crate::native_states::GUARD_ON
            | crate::native_states::GUARD
    )
}
/// Ground locomotion, crouch and ordinary landing may grab as soon as native
/// transitions permit it. Attack recovery, shield drop/stun and air stay out.
pub fn ground_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT | crate::native_states::GUARD_ON | crate::native_states::GUARD
    )
}
/// Existing idle assistance retains its one delivered approach pulse. Already
/// moving/landing actors send Grab immediately, without first easing the stick.
pub fn automatic_grab_needs_approach(status: i32, velocity_x: f32) -> bool {
    status == crate::native_states::status::WAIT
        && velocity_x.is_finite()
        && velocity_x.abs() <= 0.01
}
/// Held guard permits direct shield grab. Ground movement can use standing or
/// dash-catch readiness; no pivot input or native status is forced.
pub fn automatic_grab_ready(status: i32, enabled: impl FnMut(i32) -> bool) -> bool {
    automatic_grab_ready_with_shield(
        status,
        matches!(
            status,
            crate::native_states::GUARD_ON | crate::native_states::GUARD
        ),
        enabled,
    )
}
/// A fresh physical shield press counts before the native status catches up;
/// its direct grab must never borrow dash-catch readiness from locomotion.
pub fn automatic_grab_ready_with_shield(
    status: i32,
    shielded: bool,
    mut enabled: impl FnMut(i32) -> bool,
) -> bool {
    ground_grab_state(status)
        && (status == crate::native_states::GUARD
            || enabled(crate::native_states::transition_terms::CONT_CATCH)
            || !shielded
                && !matches!(
                    status,
                    crate::native_states::GUARD_ON | crate::native_states::GUARD
                )
                && enabled(crate::native_states::transition_terms::CONT_CATCH_DASH))
}
/// Unshielded automatic grabs require observed enemy shield startup/hold/damage.
/// Native vulnerability, capture links, front side and range are separate gates.
pub fn automatic_grab_target_status(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::GUARD_ON
            | crate::native_states::GUARD
            | crate::native_states::GUARD_DAMAGE
    )
}
/// Shield grabs may punish ordinary hittable opponents without enemy shield.
/// Downed/tripped bodies cannot be ordinary-grab targets; shield Up-B may hit
/// them instead. Current vulnerability/capture links and reach are checked too.
pub fn automatic_grab_target_status_with_shield(status: i32, shielded: bool) -> bool {
    if shielded {
        automatic_shield_target_status(status)
            && !crate::native_states::ordinary_grab_immune(status)
    } else {
        automatic_grab_target_status(status)
    }
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
    ) && out.stick.is_some_and(|s| {
        out.requested & MAPPER_R != 0 && s[1] >= 0.
            || out.requested & SEQUENCE_JUMP_BUTTONS != 0 && s[1] >= 0.
            || out.requested & MAPPER_A != 0 && s[1] >= 0.95
            || out.requested & MAPPER_B != 0 && s[1] > 0.5
            || right[1] > 0.5
    }) {
        out.requested |= MAPPER_ZR;
    }
    out
}

#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub grounded: bool,
    pub dangerous: bool,
    pub recovery: Option<f32>,
}
#[derive(Clone, Copy)]
pub struct Context {
    pub object: u32,
    pub kind: i32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub grounded: bool,
    pub position: [f32; 2],
    pub facing: f32,
    pub shield_ready: bool,
    pub can_grab: bool,
    pub can_jump: bool,
    pub can_up_special: bool,
    pub threatened: bool,
}
#[derive(Default)]
pub struct OutOfShield {
    object: Option<u32>,
    kind: Option<i32>,
    target: Option<u32>,
    guarded: bool,
    response: crate::defense_response::Response,
    simple: simple::Grab,
    shield_up_b: shield_up_b::ShieldUpB,
    configured_response: bool,
    configured_backup: bool,
    configured_shielded: bool,
    configured_grab_wait: bool,
    configured_settling: bool,
}
impl OutOfShield {
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
        }
    }
    fn bind_actor(&mut self, c: Context) {
        if self.object != Some(c.object) || self.kind != Some(c.kind) {
            *self = Self::default();
            self.object = Some(c.object);
            self.kind = Some(c.kind);
        }
    }
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        self.simple.observe_statuses(known);
    }
    pub fn target(&self) -> Option<u32> {
        self.simple
            .target()
            .or(self.shield_up_b.target())
            .or(self.target)
            .or(self.response.target())
    }
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
        self.configured_response = false;
        self.configured_backup = false;
        self.configured_shielded = false;
        self.configured_grab_wait = false;
        self.configured_settling = false;
        self.simple.cancel();
        self.shield_up_b.cancel();
        self.target = None;
    }
    /// Ledge/manual takeover releases inputs immediately. Delivered R retains
    /// only its bounded receipt so a competing script cannot turn it into Up-B.
    pub fn suspend_automatic(&mut self) {
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
    pub fn configured_committed(&self) -> bool {
        self.configured_response
    }
    pub fn needs_ground(&self) -> bool {
        self.configured_response && self.response.needs_ground()
    }
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        if self.configured_response {
            self.response.right_stick()
        } else if self.simple.active() || self.shield_up_b.active() {
            Some([0.; 2])
        } else {
            self.response.right_stick()
        }
    }
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    pub fn sample_simple(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        attacker: Option<crate::gameplay::Opponent>,
        close: f32,
        _legacy_far: f32,
    ) -> ControllerOutput {
        let startup = crate::fighter_data::ordinary_grab(c.kind, crate::native_states::CATCH)
            .map(|g| f32::from(g.first));
        self.sample_simple_timed(allowed, c, context, targets, attacker, close, startup)
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
            range,
            24.,
            startup,
            range,
            false,
            matches!(
                c.status,
                crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
            ),
        )
    }
    /// Front vulnerable-target grab (enemy shield required only when unshielded):
    /// direct neutral-axis R in shield, immediate
    /// R during ground movement/landing, or one idle approach pulse then R.
    pub fn sample_automatic(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<crate::gameplay::Opponent>,
        shielded: bool,
        close: f32,
        approach: f32,
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
            approach,
            startup,
            crate::settings::AssistSettings {
                fighter_kind: c.kind,
                ..Default::default()
            },
            AutomaticConfig::default(),
        )
    }
    /// Neutralize the stick during unshielded Grab delivery while preserving
    /// manual turning-away cancellation. Shield grabs already use neutral axes.
    pub fn with_grab_movement(
        &mut self,
        mut output: ControllerOutput,
        physical: [f32; 2],
    ) -> ControllerOutput {
        if self.simple.turning_away(physical[0]) {
            self.suspend_automatic();
            return ControllerOutput::default();
        }
        if self.simple.unshielded_grab_started()
            && output.requested & crate::native_input::MAPPER_R != 0
        {
            output.stick = Some([0.0, 0.0]);
        }
        output
    }
    pub fn unshielded_grab_started(&self) -> bool {
        self.simple.unshielded_grab_started()
    }
    pub fn shield_up_b_pending(&self) -> bool {
        self.shield_up_b.active() || self.configured_response && self.response.automatic_up_b()
    }
    pub fn up_b_delivery_started(&self) -> bool {
        self.shield_up_b.delivery_started()
            || self.configured_response && self.response.up_b_delivery_started()
    }
    pub fn up_b_receipt_pending(&self) -> bool {
        self.shield_up_b.receipt_pending()
            || self.configured_response && self.response.up_b_receipt_pending()
    }
    pub fn awaiting_up_b(&self) -> bool {
        self.shield_up_b.awaiting() || self.configured_response && self.response.awaiting_up_b()
    }
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.attempted_up_special(tick, target);
        } else {
            self.shield_up_b.attempted(tick, target);
        }
    }
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.mapped_up_special(tick, target);
        } else {
            self.shield_up_b.mapped(tick, target);
        }
    }
    pub fn grab_delivery_pending(&self) -> bool {
        self.simple.delivery_pending() || self.response.grab_delivery_pending()
    }
    pub fn delivery_tick(&mut self, tick: i64) {
        self.shield_up_b.delivery_tick(tick);
        self.simple.delivery_tick(tick);
        self.response.delivery_tick(tick);
    }
    pub fn automatic_grab_pending(&self) -> bool {
        self.simple.active() && !self.configured_response && !self.configured_settling
    }
    pub fn approach_pending(&self) -> bool {
        self.simple.approach_pending()
    }
    pub fn approach_active(&self) -> bool {
        self.simple.approach_active()
    }
    pub fn mapped_approach(&mut self, tick: i64, target: u32) {
        self.simple.mapped_approach(tick, target);
    }
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
