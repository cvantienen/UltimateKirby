//! Mapper receipt shared by ledge/button tapes, automatic responses and manual
//! Catch. Never turn a delivered Grab into a competing Special/throw packet.
use crate::{
    action_owner::{native_grab_active, Intent, Owner},
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{sequence_interrupted, NativeAttempt},
    native_input::{MAPPER_R, MAPPER_ZR},
};
/// Defines the `GrabHandoff` data structure.
///
/// Holds the data for GrabHandoff.
#[derive(Default)]
pub struct GrabHandoff {
    /// Stores the `pending` value in `GrabHandoff` as `bool`.
    ///
    ///
    ///  Holds the pending value used by GrabHandoff.
    pending: bool,
    /// Stores the `owner` value in `GrabHandoff` as `Option<Owner>`.
    ///
    ///
    ///  Optionally stores the owner for GrabHandoff; it may be absent.
    owner: Option<Owner>,
    /// Stores the `delivery` value in `GrabHandoff` as `crate::move_input::DeliveryBudget`.
    ///
    ///
    ///  Holds the delivery value used by GrabHandoff.
    delivery: crate::move_input::DeliveryBudget,
    /// Stores the `previous_request` value in `GrabHandoff` as `bool`.
    ///
    ///
    ///  Holds the previous request value used by GrabHandoff.
    previous_request: bool,
    /// Stores the `previous_catch` value in `GrabHandoff` as `bool`.
    ///
    ///
    ///  Holds the previous catch value used by GrabHandoff.
    previous_catch: bool,
    /// Stores the `native_clock` value in `GrabHandoff` as `NativeAttempt`.
    ///
    ///
    ///  Holds the native clock value used by GrabHandoff.
    native_clock: NativeAttempt,
    /// Stores the `native_context` value in `GrabHandoff` as `Option<GameplayContext>`.
    ///
    ///
    ///  Optionally stores the native context for GrabHandoff; it may be absent.
    native_context: Option<GameplayContext>,
    /// Stores the `enemy_clock` value in `GrabHandoff` as `NativeAttempt`.
    ///
    ///
    ///  Holds the enemy clock value used by GrabHandoff.
    enemy_clock: NativeAttempt,
    /// Stores the `enemy_id` value in `GrabHandoff` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the enemy id for GrabHandoff; it may be absent.
    enemy_id: Option<u32>,
    /// Stores the `packets` value in `GrabHandoff` as `u8`.
    ///
    ///
    ///  Holds the packets value used by GrabHandoff.
    packets: u8,
    /// Stores the `last_packet` value in `GrabHandoff` as `Option<i64>`.
    ///
    ///
    ///  Optionally stores the last packet for GrabHandoff; it may be absent.
    last_packet: Option<i64>,
}
/// Airborne, caught or otherwise interrupted fighters cannot be waiting on a Grab.
///
///  Airborne, caught or otherwise interrupted fighters cannot be waiting on a Grab.
fn receipt_ends(status: i32, situation: i32) -> bool {
    situation != crate::native_states::situation::GROUND
        || native_grab_active(status)
        || sequence_interrupted(status)
}
impl GrabHandoff {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
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
    /// Checks whether a pending operation is stored.
    ///
    ///
    ///  Checks whether something is waiting to be processed.
    pub fn pending(&self) -> bool {
        self.pending
    }
    /// Admission must retain a queued Grab from another owner. Discarding its
    /// receipt can turn a new ledge Special into a throw when Catch arrives late.
    ///
    ///  Admission must retain a queued Grab from another owner. Discarding its receipt can turn a new ledge Special into a throw when Catch arrives late.
    pub fn reserve_ledge_inputs(&self) -> bool {
        !self.pending || self.owner == Some(Owner::Ledge)
    }
    /// Returns the `grab_packet` result.
    ///
    ///
    ///  Gets the grab packet result.
    pub fn grab_packet(mut out: ControllerOutput) -> ControllerOutput {
        if out.requested & MAPPER_R != 0 {
            out.requested &= MAPPER_R | MAPPER_ZR;
            out.stick = Some([out.stick.map_or(0., |s| s[0]), 0.]);
            out.native_short_hop = false;
        }
        out
    }
    /// Records the time of the latest fighter observation.
    ///
    ///
    ///  Remembers when a fighter was last seen.
    pub fn observe(&mut self, valid: bool, status: i32, situation: i32) {
        if !valid || receipt_ends(status, situation) || self.delivery.exhausted() {
            self.clear_receipt();
            if !valid {
                self.previous_request = false;
                self.previous_catch = false;
            }
        }
    }
    /// Forget the receipt but keep the request/catch edge memory and the last
    /// native sample, so a held button is not mistaken for a new request.
    ///
    ///  Forget the receipt but keep the request/catch edge memory and the last native sample, so a held button is not mistaken for a new request.
    fn clear_receipt(&mut self) {
        *self = Self {
            previous_request: self.previous_request,
            previous_catch: self.previous_catch,
            native_context: self.native_context,
            ..Self::new()
        };
    }
    /// Live observations expire on native progress, not controller polls. A
    /// missing/frozen observation cannot prove a queued Catch has expired.
    /// This receipt emits no input while awaiting acknowledgment.
    ///
    ///  Live observations expire on native progress, not controller polls. A missing/frozen observation cannot prove a queued Catch has expired. This receipt emits no input while awaiting acknowledgment.
    pub fn observe_native(&mut self, valid: bool, context: Option<GameplayContext>) {
        if !valid {
            *self = Self::new();
            return;
        }
        let Some(c) = context else {
            return;
        };
        self.native_context = Some(c);
        if receipt_ends(c.status, c.situation) {
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
    ///
    ///  Held shield can have a static animation even while the game advances. Reuse a current opponent motion already read by the active watch; never poll another fighter or count controller ticks as simulated frames.
    pub fn observe_enemy_progress(&mut self, c: GameplayContext, enemy: Option<Opponent>) {
        if !self.pending {
            return;
        }
        let sample = enemy
            .and_then(|e| e.motion.map(|m| (e, m)))
            .filter(|_| !c.stopped);
        let Some((enemy, motion)) = sample else {
            self.track_enemy(None);
            return;
        };
        self.track_enemy(Some(enemy.id));
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
    /// The opponent clock restarts whenever the watched opponent changes or a
    /// sample is missed.
    ///
    ///  The opponent clock restarts whenever the watched opponent changes or a sample is missed.
    fn track_enemy(&mut self, id: Option<u32>) {
        if id.is_none() || self.enemy_id != id {
            self.enemy_clock = NativeAttempt::new();
            self.enemy_id = id;
        }
    }
    /// Determines whether the `shield_packet` condition holds.
    ///
    ///
    ///  Checks whether shield packet.
    pub fn shield_packet(out: ControllerOutput, right: Option<[f32; 2]>) -> bool {
        out.requested == MAPPER_ZR
            && out.stick.is_none_or(|s| s == [0.; 2])
            && right.is_none_or(|s| s == [0.; 2])
    }
    /// Updates the `restrict` state using the supplied inputs.
    ///
    ///
    ///  Updates restrict.
    pub fn restrict(&self, intents: &mut [Intent]) {
        if self.pending {
            for i in intents {
                if i.owner != Owner::Safety
                    && i.owner != Owner::Defense
                    && Some(i.owner) != self.owner
                {
                    i.present = false;
                }
            }
        }
    }
    /// Same owner's remaining neutral-axis Grab pulse is safe. At its first
    /// other action, abort playback and retain only this input-free receipt.
    ///
    ///  Same owner's remaining neutral-axis Grab pulse is safe. At its first other action, abort playback and retain only this input-free receipt.
    pub fn permits(
        &self,
        owner: Option<Owner>,
        out: ControllerOutput,
        right: Option<[f32; 2]>,
    ) -> bool {
        if !self.pending || owner == Some(Owner::Safety) || Self::shield_packet(out, right) {
            return true;
        }
        let own = owner.is_some() && owner == self.owner;
        let idle = out.requested == 0 && out.stick.is_none_or(|s| s == [0.; 2]);
        let grab_pulse = self.packets < 4
            && out.requested & MAPPER_R != 0
            && out.requested & !(MAPPER_R | MAPPER_ZR) == 0
            && out.stick.is_none_or(|s| s[1] == 0.);
        own && right.is_none_or(|s| s == [0.; 2]) && (idle || grab_pulse)
    }
    /// Called after the original mapper, once per delivered controller tick.
    ///
    ///  Called after the original mapper, once per delivered controller tick.
    pub fn mapped(
        &mut self,
        tick: i64,
        owner: Option<Owner>,
        requested_grab: bool,
        native_catch: bool,
        eligible: bool,
    ) {
        let new_grab_request = requested_grab && !self.previous_request;
        let new_native_catch = native_catch && !self.previous_catch;
        let fresh = new_grab_request || new_native_catch;
        if eligible && !self.pending && fresh {
            self.clear_receipt();
            self.pending = true;
            self.owner = owner;
            if let Some(c) = self.native_context {
                self.native_clock.advance_pause(c, None, u16::MAX);
            }
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
