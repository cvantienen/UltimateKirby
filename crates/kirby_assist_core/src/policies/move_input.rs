//! Physical-button recipes shared by assists. A must map to Attack and X/ZL
//! to Jump in Smash's controller setup. These are requests, not status writes.
//! Native status/motion confirmation remains the caller's acceptance signal.
use crate::native_input::{MAPPER_A, MAPPER_X};

/// `ATTACK` is a configuration value used by the Smash runtime.
pub const ATTACK: u32 = MAPPER_A;
/// `SHORT_HOP_ATTACK` is a configuration value used by the Smash runtime.
pub const SHORT_HOP_ATTACK: u32 = MAPPER_X | MAPPER_A;
/// Acceptance budget, not attack startup or a timing offset. Buffered online
/// inputs can reach native state after several advancing frames. Keep the
/// sequence owner alive while waiting, without holding Jump or delaying a
/// follow-up once its native prerequisite is observed.
pub const ACCEPTANCE_TIMEOUT: f32 = 24.0;
/// Last-resort bound for automatic input delivery when native animation clocks
/// remain frozen. A mapper receipt is not proof that the move started. This
/// does not advance a recipe or shorten a confirmed move's tracking duration.
#[derive(Default)]
pub(crate) struct DeliveryBudget {
/// Runtime attempts value used by DeliveryBudget..
    attempts: u8,
/// Runtime last tick value used by DeliveryBudget..
    last_tick: Option<i64>,
}
impl DeliveryBudget {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            attempts: 0,
            last_tick: None,
        }
    }
    /// Checks whether the requested input was acknowledged by the mapper.
    pub fn mapped(&mut self, tick: i64) {
        if self.last_tick != Some(tick) {
            self.attempts = self.attempts.saturating_add(1);
            self.last_tick = Some(tick);
        }
    }
    /// Determines whether the `exhausted` condition holds.
    pub fn exhausted(&self) -> bool {
        self.attempts >= 24
    }
}
/// A delivered automatic Up-B may appear in native status after shield hitstop
/// or input delay. Remember receipt only; never emit/retry inputs while waiting.
#[derive(Default)]
pub(crate) struct UpBReceipt {
/// Runtime sent value used by UpBReceipt..
    sent: bool,
/// Runtime ticks value used by UpBReceipt..
    ticks: u8,
/// Runtime last tick value used by UpBReceipt..
    last_tick: Option<i64>,
}
impl UpBReceipt {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            sent: false,
            ticks: 0,
            last_tick: None,
        }
    }
    /// Checks whether the requested input was acknowledged by the mapper.
    pub fn mapped(&mut self, tick: i64) {
        if !self.sent {
            self.sent = true;
            self.ticks = 1;
            self.last_tick = Some(tick);
        } else {
            self.tick(tick);
        }
    }
    /// Updates the `tick` state using the supplied inputs.
    pub fn tick(&mut self, tick: i64) {
        if self.sent && self.last_tick != Some(tick) {
            self.last_tick = Some(tick);
            self.ticks = self.ticks.saturating_add(1);
        }
    }
    /// Determines whether the `sent` condition holds.
    pub fn sent(&self) -> bool {
        self.sent
    }
    /// Determines whether the `expired` condition holds.
    pub fn expired(&self) -> bool {
        self.ticks >= 24
    }
}
/// What an automatic direct Up-B tape must do after this sample's checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpBVerdict {
/// `Continue` option in the `UpBVerdict` Smash policy state.
    Continue,
    /// Native startup was just observed: resume the tape after its initial B.
    Confirm,
    /// Input-free wait for the delivered B to appear in native status.
    Wait,
/// `Cancel` option in the `UpBVerdict` Smash policy state.
    Cancel,
}
/// One automatic direct Up-B from delivery to native confirmation. Shared by
/// the Auto-react and configured-defense tapes; the owner keeps the tape, the
/// target and the plan-presence gate.
#[derive(Default)]
pub(crate) struct UpBFlow {
/// Runtime armed value used by UpBFlow..
    armed: bool,
    /// Configured-defense flow: may begin from, and tolerate, the actor's own
    /// spot dodge, and confirms native startup even without a mapper receipt.
    /// Auto-react instead delivers its whole critical B pulse until a receipt
    /// is pending.
    defense: bool,
/// Runtime started from dodge value used by UpBFlow..
    started_from_dodge: bool,
/// Runtime seen value used by UpBFlow..
    seen: bool,
/// Runtime waiting value used by UpBFlow..
    waiting: bool,
/// Runtime epoch value used by UpBFlow..
    epoch: u64,
/// Runtime delivery value used by UpBFlow..
    delivery: DeliveryBudget,
/// Runtime receipt value used by UpBFlow..
    receipt: UpBReceipt,
}
impl UpBFlow {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            armed: false,
            defense: false,
            started_from_dodge: false,
            seen: false,
            waiting: false,
            epoch: 0,
            delivery: DeliveryBudget::new(),
            receipt: UpBReceipt::new(),
        }
    }
    /// Updates the `arm` state using the supplied inputs.
    pub fn arm(&mut self, c: crate::gameplay::GameplayContext, defense: bool) {
        *self = Self {
            armed: true,
            defense,
            started_from_dodge: defense && c.status == crate::native_states::ESCAPE,
            epoch: c.manual_input_epoch,
            ..Self::new()
        };
    }
    /// Clears the armed state and resets the hold and release timers.
    pub fn disarm(&mut self) {
        *self = Self::new();
    }
    /// Determines whether the `armed` condition holds.
    pub fn armed(&self) -> bool {
        self.armed
    }
    /// Determines whether the `started` condition holds.
    pub fn started(&self) -> bool {
        self.armed && (self.seen || self.receipt.sent())
    }
    /// Determines whether the `receipt_pending` condition holds.
    pub fn receipt_pending(&self) -> bool {
        self.armed && crate::up_b_receipt::pending(self.seen, self.receipt.sent())
    }
    /// Determines whether the `awaiting` condition holds.
    pub fn awaiting(&self) -> bool {
        self.receipt_pending() && self.waiting
    }
    /// A mapped packet the game rejected still spends the delivery budget.
    pub fn attempted(&mut self, tick: i64) {
        if self.armed && !self.seen {
            self.delivery.mapped(tick);
        }
    }
    /// Checks whether the requested input was acknowledged by the mapper.
    pub fn mapped(&mut self, tick: i64) {
        if self.armed && !self.seen {
            self.delivery.mapped(tick);
            self.receipt.mapped(tick);
        }
    }
    /// Updates the `tick` state using the supplied inputs.
    pub fn tick(&mut self, tick: i64) {
        if self.receipt_pending() {
            self.receipt.tick(tick);
        }
    }
    /// Determines whether the `manual_handback` condition holds.
    pub fn manual_handback(&self, c: crate::gameplay::GameplayContext) -> bool {
        crate::up_b_receipt::manual_handback(self.armed, self.seen, c, self.epoch)
    }
    /// Native confirmation and lifecycle checks before the tape is sampled.
    pub fn advance(&mut self, c: crate::gameplay::GameplayContext, kind: i32) -> UpBVerdict {
        use crate::native_states::{status, ESCAPE, ESCAPE_AIR_SLIDE, ESCAPE_F, GUARD_DAMAGE};
        let native = crate::reaction_rules::automatic_up_b_status(kind, c.status, c.previous_status);
        let confirm = match crate::up_b_receipt::evaluate(
            self.seen,
            native,
            self.waiting,
            self.started_from_dodge,
            c,
            &self.receipt,
            &self.delivery,
        ) {
            crate::up_b_receipt::ReceiptAction::Confirm => self.defense || self.receipt.sent(),
            crate::up_b_receipt::ReceiptAction::Cancel => return UpBVerdict::Cancel,
            crate::up_b_receipt::ReceiptAction::Wait => {
                self.waiting = true;
                return UpBVerdict::Wait;
            }
            crate::up_b_receipt::ReceiptAction::Continue => false,
        };
        self.seen |= native;
        // A native roll/jump/wrong special is not successful Up-B delivery.
        // Confirmed Up-B drift also ends once ordinary control resumes.
        let first_escape = if self.defense { ESCAPE_F } else { ESCAPE };
        let wrong = matches!(
            c.status,
            status::JUMP..=status::FALL_AERIAL
                | crate::native_states::CATCH..=crate::native_states::THROW
                | status::SPECIAL_N
                | status::SPECIAL_S
                | status::SPECIAL_LW
        ) || (first_escape..=ESCAPE_AIR_SLIDE).contains(&c.status)
            || self.defense && c.status == ESCAPE && (!c.can_act || self.seen);
        let ended = self.seen
            && (!native
                || matches!(c.status,status::WAIT..=status::TURN_RUN_BRAKE|status::FALL_SPECIAL|status::LANDING..=GUARD_DAMAGE));
        let cannot_start = !self.seen
            && (c.situation != crate::native_states::situation::GROUND
                || c.stopped
                || !c.can_act
                || !(crate::reaction_rules::automatic_start_status(c.status)
                    || self.defense && c.status == ESCAPE)
                || self.delivery.exhausted());
        if wrong || ended || cannot_start {
            UpBVerdict::Cancel
        } else if confirm {
            UpBVerdict::Confirm
        } else {
            UpBVerdict::Continue
        }
    }
    /// Never drift before native startup. A delivered packet may keep only a
    /// bounded, input-free receipt wait; no B is retried.
    pub fn after_sample(&mut self, manual_drifting: bool, requested: u32) -> UpBVerdict {
        if !self.armed
            || !(manual_drifting || !self.seen && requested & crate::native_input::MAPPER_B == 0)
        {
            return UpBVerdict::Continue;
        }
        if crate::up_b_receipt::pending(self.seen, self.receipt.sent()) {
            self.waiting = true;
            UpBVerdict::Wait
        } else {
            UpBVerdict::Cancel
        }
    }
}
/// Less mechanical retry cadence for smash attacks. The caller still sends the
/// first legal smash immediately; this only controls retries when native startup
/// has not appeared. Age advances from native fighter/opponent clocks, so extra
/// controller polls online cannot speed up or reshape the rhythm.
pub fn smash_retry_pressed(age: f32, target: u32) -> bool {
    if !age.is_finite() || age < 2.0 {
        return false;
    }
    // Two slightly different bounded rhythms avoid every target receiving the
    // same metronomic pattern. Both include clean release frames for fresh A.
    let frame = age.floor() as u32;
    if target & 1 == 0 {
        matches!(frame % 10, 2 | 3 | 5 | 6 | 7)
    } else {
        matches!(frame % 11, 2 | 3 | 4 | 6 | 7 | 9)
    }
}
/// Owning only left-stick aim must not erase a physical tilt-stick attack.
/// Full automated sequences still suppress competing physical aerials/tilts.
pub fn blocks_physical_c_stick(owned: u32) -> bool {
    owned & crate::native_input::ASSIST_BUTTONS != 0
}
/// Both planar positions are usable measurements.
pub(crate) fn finite_positions(a: [f32; 2], b: [f32; 2]) -> bool {
    a.iter().chain(b.iter()).all(|v| v.is_finite())
}
/// Returns the `output` result.
/// Gets the output result.
pub fn output(stick: [f32; 2], requested: u32) -> crate::gameplay::ControllerOutput {
    crate::gameplay::ControllerOutput {
        owned: crate::native_input::ASSIST_BUTTONS,
        requested,
        stick: Some(stick),
        consume_trigger: false,
        native_short_hop: false,
    }
}
/// Available `Aerial` choices for the Smash assistance menu.
#[derive(Clone, Copy)]
pub enum Aerial {
/// `Neutral` option in the `Aerial` Smash policy state.
    Neutral,
/// `Forward` option in the `Aerial` Smash policy state.
    Forward,
/// `Back` option in the `Aerial` Smash policy state.
    Back,
/// `Up` option in the `Aerial` Smash policy state.
    Up,
/// `Down` option in the `Aerial` Smash policy state.
    Down,
}
/// Available `Smash` choices for the Smash assistance menu.
#[derive(Clone, Copy)]
pub enum Smash {
/// `Forward` option in the `Smash` Smash policy state.
    Forward,
/// `ForwardUp` option in the `Smash` Smash policy state.
    ForwardUp,
/// `Up` option in the `Smash` Smash policy state.
    Up,
/// `Down` option in the `Smash` Smash policy state.
    Down,
}
/// Callers provide a neutral release before this flick, and release Attack on
/// native startup. Holding A indefinitely would charge, rather than retry.
pub fn smash_stick(move_: Smash, facing: f32) -> [f32; 2] {
    match move_ {
        Smash::Forward => [facing.signum(), 0.0],
        Smash::ForwardUp => [facing.signum(), 0.5],
        Smash::Up => [0.0, 1.0],
        Smash::Down => [0.0, -1.0],
    }
}
/// Returns the `aerial_stick` result.
/// Gets the aerial stick result.
pub fn aerial_stick(move_: Aerial, facing: f32) -> [f32; 2] {
    let facing = if facing < 0.0 { -1.0 } else { 1.0 };
    match move_ {
        Aerial::Neutral => [0.0, 0.0],
        Aerial::Forward => [facing, 0.0],
        Aerial::Back => [-facing, 0.0],
        Aerial::Up => [0.0, 1.0],
        Aerial::Down => [0.0, -1.0],
    }
}
/// Directional A is legal in native jump squat or in the air, never in a
/// grounded turnaround. Jump-only launch avoids accidental grounded tilts.
pub fn aerial_ready(status: i32, situation: i32) -> bool {
    status == crate::native_states::status::JUMP_SQUAT
        || matches!(
            situation,
            crate::native_states::situation::GROUND | crate::native_states::situation::AIR
        ) && matches!(
            status,
            crate::native_states::status::JUMP
                | crate::native_states::status::FALL
                | crate::native_states::status::FALL_AERIAL
        )
}
/// Returns an optional `fast_fall` value.
/// Gets fast fall if it is available.
pub fn fast_fall(velocity_y: f32) -> Option<[f32; 2]> {
    (velocity_y.is_finite() && velocity_y < -0.05).then_some([0.0, -1.0])
}
