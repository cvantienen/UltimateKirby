//! Mapper receipt shared by ledge/button tapes, automatic responses and manual
//! Catch. Never turn a delivered Grab into a competing Special/throw packet.
use crate::{
    action_owner::{native_grab_active, Intent, Owner},
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{sequence_interrupted, NativeAttempt},
    native_input::{MAPPER_R, MAPPER_ZR, SEQUENCE_JUMP_BUTTONS},
};
#[derive(Default)]
pub struct GrabHandoff {
    pending: bool,
    owner: Option<Owner>,
    delivery: crate::move_input::DeliveryBudget,
    previous_request: bool,
    previous_catch: bool,
    native_clock: NativeAttempt,
    native_context: Option<GameplayContext>,
    enemy_clock: NativeAttempt,
    enemy_id: Option<u32>,
    packets: u8,
    last_packet: Option<i64>,
}
impl GrabHandoff {
    pub const fn new() -> Self {
        Self {
            pending: false,
            owner: None,
            delivery: crate::move_input::DeliveryBudget::new(),
            previous_request: false,
            previous_catch: false,
            native_clock: NativeAttempt::new(),
            native_context: None,
            enemy_clock: NativeAttempt::new(),
            enemy_id: None,
            packets: 0,
            last_packet: None,
        }
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    /// Admission must retain a queued Grab from another owner. Discarding its
    /// receipt can turn a new ledge Special into a throw when Catch arrives late.
    pub fn reserve_ledge_inputs(&self) -> bool {
        !self.pending || self.owner == Some(Owner::Ledge)
    }
    pub fn grab_packet(mut out: ControllerOutput) -> ControllerOutput {
        if out.requested & MAPPER_R != 0 {
            out.requested &= MAPPER_R | MAPPER_ZR;
            out.stick = Some([out.stick.map_or(0., |s| s[0]), 0.]);
            out.native_short_hop = false;
        }
        out
    }
    pub fn observe(&mut self, valid: bool, status: i32, situation: i32) {
        if !valid
            || situation != 0
            || native_grab_active(status)
            || sequence_interrupted(status)
            || self.delivery.exhausted()
        {
            self.clear_receipt();
            if !valid {
                self.previous_request = false;
                self.previous_catch = false;
            }
        }
    }
    fn clear_receipt(&mut self) {
        self.pending = false;
        self.owner = None;
        self.delivery = Default::default();
        self.native_clock = NativeAttempt::new();
        self.enemy_clock = NativeAttempt::new();
        self.enemy_id = None;
        self.packets = 0;
        self.last_packet = None;
    }
    /// Live observations expire on native progress, not controller polls. A
    /// missing/frozen observation cannot prove a queued Catch has expired.
    /// This receipt emits no input while awaiting acknowledgment.
    pub fn observe_native(&mut self, valid: bool, context: Option<GameplayContext>) {
        if !valid {
            *self = Self::new();
            return;
        }
        let Some(c) = context else {
            return;
        };
        self.native_context = Some(c);
        if c.situation != 0 || native_grab_active(c.status) || sequence_interrupted(c.status) {
            self.clear_receipt();
        } else if self.pending {
            self.native_clock.advance_pause(c, None, u16::MAX);
            if self.native_clock.elapsed() >= crate::move_input::ACCEPTANCE_TIMEOUT {
                self.clear_receipt();
            }
        }
    }
    /// Held shield can have a static animation even while the game advances.
    /// Reuse a current opponent motion already read by the active watch; never
    /// poll another fighter or count controller ticks as simulated frames.
    pub fn observe_enemy_progress(&mut self, c: GameplayContext, enemy: Option<Opponent>) {
        if !self.pending {
            return;
        }
        let sample = enemy
            .and_then(|e| e.motion.map(|m| (e, m)))
            .filter(|_| !c.stopped);
        let Some((enemy, motion)) = sample else {
            self.enemy_clock = NativeAttempt::new();
            self.enemy_id = None;
            return;
        };
        if self.enemy_id != Some(enemy.id) {
            self.enemy_clock = NativeAttempt::new();
            self.enemy_id = Some(enemy.id);
        }
        self.enemy_clock.advance_pause(
            GameplayContext {
                status: enemy.status,
                motion_kind: motion.kind,
                motion_frame: motion.frame,
                motion_rate: motion.rate,
                ..c
            },
            None,
            u16::MAX,
        );
        if self.enemy_clock.elapsed() >= crate::move_input::ACCEPTANCE_TIMEOUT {
            self.clear_receipt();
        }
    }
    pub fn manual_jump_packet(out: ControllerOutput, right: Option<[f32; 2]>) -> bool {
        out.requested & SEQUENCE_JUMP_BUTTONS != 0
            && out.requested & !SEQUENCE_JUMP_BUTTONS == 0
            && out.stick.is_none_or(|s| s[1] == 0.)
            && right.is_none_or(|s| s == [0.; 2])
    }
    pub fn shield_packet(out: ControllerOutput, right: Option<[f32; 2]>) -> bool {
        out.requested == MAPPER_ZR
            && out.stick.is_none_or(|s| s == [0.; 2])
            && right.is_none_or(|s| s == [0.; 2])
    }
    pub fn restrict(&self, intents: &mut [Intent]) {
        self.restrict_with_manual_jump(intents, false);
    }
    pub fn restrict_with_manual_jump(&self, intents: &mut [Intent], manual_jump: bool) {
        if self.pending {
            for i in intents {
                if i.owner != Owner::Safety
                    && i.owner != Owner::Defense
                    && Some(i.owner) != self.owner
                    && !(manual_jump && i.owner == Owner::Shortcut)
                {
                    i.present = false;
                }
            }
        }
    }
    /// Same owner's remaining neutral-axis Grab pulse is safe. At its first
    /// other action, abort playback and retain only this input-free receipt.
    pub fn permits(
        &self,
        owner: Option<Owner>,
        out: ControllerOutput,
        right: Option<[f32; 2]>,
    ) -> bool {
        !self.pending
            || owner == Some(Owner::Safety)
            || owner.is_some()
                && owner == self.owner
                && out.requested == 0
                && out.stick.is_none_or(|s| s == [0.; 2])
                && right.is_none_or(|s| s == [0.; 2])
            || Self::shield_packet(out, right)
            || owner == Some(Owner::Shortcut) && Self::manual_jump_packet(out, right)
            || owner.is_some()
                && owner == self.owner
                && self.packets < 4
                && out.requested & MAPPER_R != 0
                && out.requested & !(MAPPER_R | MAPPER_ZR) == 0
                && out.stick.is_none_or(|s| s[1] == 0.)
                && right.is_none_or(|s| s == [0.; 2])
    }
    /// Called after the original mapper, once per delivered controller tick.
    pub fn mapped(
        &mut self,
        tick: i64,
        owner: Option<Owner>,
        requested_grab: bool,
        native_catch: bool,
        eligible: bool,
    ) {
        let fresh =
            requested_grab && !self.previous_request || native_catch && !self.previous_catch;
        if eligible && !self.pending && fresh {
            self.pending = true;
            self.owner = owner;
            self.delivery = Default::default();
            self.native_clock = NativeAttempt::new();
            self.enemy_clock = NativeAttempt::new();
            self.enemy_id = None;
            if let Some(c) = self.native_context {
                self.native_clock.advance_pause(c, None, u16::MAX);
            }
            self.packets = 0;
            self.last_packet = None;
        }
        self.previous_request = requested_grab;
        self.previous_catch = native_catch;
        if self.pending {
            self.delivery.mapped(tick);
            if (requested_grab || native_catch) && self.last_packet != Some(tick) {
                self.packets = self.packets.saturating_add(1);
                self.last_packet = Some(tick);
            }
        }
    }
}
