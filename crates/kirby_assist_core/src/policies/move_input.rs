//! Physical-button recipes shared by assists. A must map to Attack and X/ZL
//! to Jump in Smash's controller setup. These are requests, not status writes.
//! Native status/motion confirmation remains the caller's acceptance signal.
use crate::native_input::{MAPPER_A, MAPPER_X};

/// Technical: Defines the `ATTACK` constant used by this module.
///
/// Plain English: Provides the ATTACK constant value.
pub const ATTACK: u32 = MAPPER_A;
/// Technical: Defines the `SHORT_HOP_ATTACK` constant used by this module.
///
/// Plain English: Provides the SHORT HOP ATTACK constant value.
pub const SHORT_HOP_ATTACK: u32 = MAPPER_X | MAPPER_A;
/// Acceptance budget, not attack startup or a timing offset. Buffered online
/// inputs can reach native state after several advancing frames. Keep the
/// sequence owner alive while waiting, without holding Jump or delaying a
/// follow-up once its native prerequisite is observed.
/// Plain English: Acceptance budget, not attack startup or a timing offset. Buffered online inputs can reach native state after several advancing frames. Keep the sequence owner alive while waiting, without holding Jump or delaying a follow-up.
pub const ACCEPTANCE_TIMEOUT: f32 = 24.0;
/// Last-resort bound for automatic input delivery when native animation clocks
/// remain frozen. A mapper receipt is not proof that the move started. This
/// does not advance a recipe or shorten a confirmed move's tracking duration.
/// Plain English: Last-resort bound for automatic input delivery when native animation clocks remain frozen. A mapper receipt is not proof that the move started. This does not advance a recipe or shorten a confirmed move's tracking duration.
#[derive(Default)]
pub(crate) struct DeliveryBudget {
    /// Technical: Stores the `attempts` value in `DeliveryBudget` as `u8`.
    ///
    /// Plain English: Holds the attempts value used by DeliveryBudget.
    attempts: u8,
    /// Technical: Stores the `last_tick` value in `DeliveryBudget` as `Option<i64>`.
    ///
    /// Plain English: Optionally stores the last tick for DeliveryBudget; it may be absent.
    last_tick: Option<i64>,
}
impl DeliveryBudget {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
    pub const fn new() -> Self {
        Self {
            attempts: 0,
            last_tick: None,
        }
    }
    /// Technical: Checks whether the requested input was acknowledged by the mapper.
    ///
    /// Plain English: Checks whether the controller mapping accepted the input.
    pub fn mapped(&mut self, tick: i64) {
        if self.last_tick != Some(tick) {
            self.attempts = self.attempts.saturating_add(1);
            self.last_tick = Some(tick);
        }
    }
    /// Technical: Determines whether the `exhausted` condition holds.
    ///
    /// Plain English: Checks whether exhausted.
    pub fn exhausted(&self) -> bool {
        self.attempts >= 24
    }
}
/// A delivered automatic Up-B may appear in native status after shield hitstop
/// or input delay. Remember receipt only; never emit/retry inputs while waiting.
/// Plain English: A delivered automatic Up-B may appear in native status after shield hitstop or input delay. Remember receipt only; never emit/retry inputs while waiting.
#[derive(Default)]
pub(crate) struct UpBReceipt {
    /// Technical: Stores the `sent` value in `UpBReceipt` as `bool`.
    ///
    /// Plain English: Holds the sent value used by UpBReceipt.
    sent: bool,
    /// Technical: Stores the `ticks` value in `UpBReceipt` as `u8`.
    ///
    /// Plain English: Holds the ticks value used by UpBReceipt.
    ticks: u8,
    /// Technical: Stores the `last_tick` value in `UpBReceipt` as `Option<i64>`.
    ///
    /// Plain English: Optionally stores the last tick for UpBReceipt; it may be absent.
    last_tick: Option<i64>,
}
impl UpBReceipt {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
    pub const fn new() -> Self {
        Self {
            sent: false,
            ticks: 0,
            last_tick: None,
        }
    }
    /// Technical: Checks whether the requested input was acknowledged by the mapper.
    ///
    /// Plain English: Checks whether the controller mapping accepted the input.
    pub fn mapped(&mut self, tick: i64) {
        if !self.sent {
            self.sent = true;
            self.ticks = 1;
            self.last_tick = Some(tick);
        } else {
            self.tick(tick);
        }
    }
    /// Technical: Updates the `tick` state using the supplied inputs.
    ///
    /// Plain English: Updates tick.
    pub fn tick(&mut self, tick: i64) {
        if self.sent && self.last_tick != Some(tick) {
            self.last_tick = Some(tick);
            self.ticks = self.ticks.saturating_add(1);
        }
    }
    /// Technical: Determines whether the `sent` condition holds.
    ///
    /// Plain English: Checks whether sent.
    pub fn sent(&self) -> bool {
        self.sent
    }
    /// Technical: Determines whether the `expired` condition holds.
    ///
    /// Plain English: Checks whether expired.
    pub fn expired(&self) -> bool {
        self.ticks >= 24
    }
}
/// What an automatic direct Up-B tape must do after this sample's checks.
/// Plain English: What an automatic direct Up-B tape must do after this sample's checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpBVerdict {
    /// Technical: Represents the `Continue` case of `UpBVerdict`.
    ///
    /// Plain English: Represents the Continue option in UpBVerdict.
    Continue,
    /// Native startup was just observed: resume the tape after its initial B.
    /// Plain English: Native startup was just observed: resume the tape after its initial B.
    Confirm,
    /// Input-free wait for the delivered B to appear in native status.
    /// Plain English: Input-free wait for the delivered B to appear in native status.
    Wait,
    /// Technical: Represents the `Cancel` case of `UpBVerdict`.
    ///
    /// Plain English: Represents the Cancel option in UpBVerdict.
    Cancel,
}
/// One automatic direct Up-B from delivery to native confirmation. Shared by
/// the Auto-react and configured-defense tapes; the owner keeps the tape, the
/// target and the plan-presence gate.
/// Plain English: One automatic direct Up-B from delivery to native confirmation. Shared by the Auto-react and configured-defense tapes; the owner keeps the tape, the target and the plan-presence gate.
#[derive(Default)]
pub(crate) struct UpBFlow {
    /// Technical: Stores the `armed` value in `UpBFlow` as `bool`.
    ///
    /// Plain English: Holds the armed value used by UpBFlow.
    armed: bool,
    /// Configured-defense flow: may begin from, and tolerate, the actor's own
    /// spot dodge, and confirms native startup even without a mapper receipt.
    /// Auto-react instead delivers its whole critical B pulse until a receipt
    /// is pending.
    /// Plain English: Configured-defense flow: may begin from, and tolerate, the actor's own spot dodge, and confirms native startup even without a mapper receipt. Auto-react instead delivers its whole critical B pulse until a receipt is pending.
    defense: bool,
    /// Technical: Stores the `started_from_dodge` value in `UpBFlow` as `bool`.
    ///
    /// Plain English: Holds the started from dodge value used by UpBFlow.
    started_from_dodge: bool,
    /// Technical: Stores the `seen` value in `UpBFlow` as `bool`.
    ///
    /// Plain English: Holds the seen value used by UpBFlow.
    seen: bool,
    /// Technical: Stores the `waiting` value in `UpBFlow` as `bool`.
    ///
    /// Plain English: Holds the waiting value used by UpBFlow.
    waiting: bool,
    /// Technical: Stores the `epoch` value in `UpBFlow` as `u64`.
    ///
    /// Plain English: Holds the epoch value used by UpBFlow.
    epoch: u64,
    /// Technical: Stores the `delivery` value in `UpBFlow` as `DeliveryBudget`.
    ///
    /// Plain English: Holds the delivery value used by UpBFlow.
    delivery: DeliveryBudget,
    /// Technical: Stores the `receipt` value in `UpBFlow` as `UpBReceipt`.
    ///
    /// Plain English: Holds the receipt value used by UpBFlow.
    receipt: UpBReceipt,
}
impl UpBFlow {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
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
    /// Technical: Updates the `arm` state using the supplied inputs.
    ///
    /// Plain English: Updates arm.
    pub fn arm(&mut self, c: crate::gameplay::GameplayContext, defense: bool) {
        *self = Self {
            armed: true,
            defense,
            started_from_dodge: defense && c.status == crate::native_states::ESCAPE,
            epoch: c.manual_input_epoch,
            ..Self::new()
        };
    }
    /// Technical: Clears the armed state and resets the hold and release timers.
    ///
    /// Plain English: Disarms the toggle and clears its timing history.
    pub fn disarm(&mut self) {
        *self = Self::new();
    }
    /// Technical: Determines whether the `armed` condition holds.
    ///
    /// Plain English: Checks whether armed.
    pub fn armed(&self) -> bool {
        self.armed
    }
    /// Technical: Determines whether the `started` condition holds.
    ///
    /// Plain English: Checks whether started.
    pub fn started(&self) -> bool {
        self.armed && (self.seen || self.receipt.sent())
    }
    /// Technical: Determines whether the `receipt_pending` condition holds.
    ///
    /// Plain English: Checks whether receipt pending.
    pub fn receipt_pending(&self) -> bool {
        self.armed && crate::up_b_receipt::pending(self.seen, self.receipt.sent())
    }
    /// Technical: Determines whether the `awaiting` condition holds.
    ///
    /// Plain English: Checks whether awaiting.
    pub fn awaiting(&self) -> bool {
        self.receipt_pending() && self.waiting
    }
    /// A mapped packet the game rejected still spends the delivery budget.
    /// Plain English: A mapped packet the game rejected still spends the delivery budget.
    pub fn attempted(&mut self, tick: i64) {
        if self.armed && !self.seen {
            self.delivery.mapped(tick);
        }
    }
    /// Technical: Checks whether the requested input was acknowledged by the mapper.
    ///
    /// Plain English: Checks whether the controller mapping accepted the input.
    pub fn mapped(&mut self, tick: i64) {
        if self.armed && !self.seen {
            self.delivery.mapped(tick);
            self.receipt.mapped(tick);
        }
    }
    /// Technical: Updates the `tick` state using the supplied inputs.
    ///
    /// Plain English: Updates tick.
    pub fn tick(&mut self, tick: i64) {
        if self.receipt_pending() {
            self.receipt.tick(tick);
        }
    }
    /// Technical: Determines whether the `manual_handback` condition holds.
    ///
    /// Plain English: Checks whether manual handback.
    pub fn manual_handback(&self, c: crate::gameplay::GameplayContext) -> bool {
        crate::up_b_receipt::manual_handback(self.armed, self.seen, c, self.epoch)
    }
    /// Native confirmation and lifecycle checks before the tape is sampled.
    /// Plain English: Native confirmation and lifecycle checks before the tape is sampled.
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
    /// Plain English: Never drift before native startup. A delivered packet may keep only a bounded, input-free receipt wait; no B is retried.
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
/// Plain English: Less mechanical retry cadence for smash attacks. The caller still sends the first legal smash immediately; this only controls retries when native startup has not appeared. Age advances from native fighter/opponent clocks, so.
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
/// Plain English: Owning only left-stick aim must not erase a physical tilt-stick attack. Full automated sequences still suppress competing physical aerials/tilts.
pub fn blocks_physical_c_stick(owned: u32) -> bool {
    owned & crate::native_input::ASSIST_BUTTONS != 0
}
/// Both planar positions are usable measurements.
/// Plain English: Both planar positions are usable measurements.
pub(crate) fn finite_positions(a: [f32; 2], b: [f32; 2]) -> bool {
    a.iter().chain(b.iter()).all(|v| v.is_finite())
}
/// Technical: Returns the `output` result.
///
/// Plain English: Gets the output result.
pub fn output(stick: [f32; 2], requested: u32) -> crate::gameplay::ControllerOutput {
    crate::gameplay::ControllerOutput {
        owned: crate::native_input::ASSIST_BUTTONS,
        requested,
        stick: Some(stick),
        consume_trigger: false,
        native_short_hop: false,
    }
}
/// Technical: Defines the available `Aerial` cases.
///
/// Plain English: Lists the available Aerial choices.
#[derive(Clone, Copy)]
pub enum Aerial {
    /// Technical: Represents the `Neutral` case of `Aerial`.
    ///
    /// Plain English: Represents the Neutral option in Aerial.
    Neutral,
    /// Technical: Represents the `Forward` case of `Aerial`.
    ///
    /// Plain English: Represents the Forward option in Aerial.
    Forward,
    /// Technical: Represents the `Back` case of `Aerial`.
    ///
    /// Plain English: Represents the Back option in Aerial.
    Back,
    /// Technical: Represents the `Up` case of `Aerial`.
    ///
    /// Plain English: Represents the Up option in Aerial.
    Up,
    /// Technical: Represents the `Down` case of `Aerial`.
    ///
    /// Plain English: Represents the Down option in Aerial.
    Down,
}
/// Technical: Defines the available `Smash` cases.
///
/// Plain English: Lists the available Smash choices.
#[derive(Clone, Copy)]
pub enum Smash {
    /// Technical: Represents the `Forward` case of `Smash`.
    ///
    /// Plain English: Represents the Forward option in Smash.
    Forward,
    /// Technical: Represents the `ForwardUp` case of `Smash`.
    ///
    /// Plain English: Represents the ForwardUp option in Smash.
    ForwardUp,
    /// Technical: Represents the `Up` case of `Smash`.
    ///
    /// Plain English: Represents the Up option in Smash.
    Up,
    /// Technical: Represents the `Down` case of `Smash`.
    ///
    /// Plain English: Represents the Down option in Smash.
    Down,
}
/// Callers provide a neutral release before this flick, and release Attack on
/// native startup. Holding A indefinitely would charge, rather than retry.
/// Plain English: Callers provide a neutral release before this flick, and release Attack on native startup. Holding A indefinitely would charge, rather than retry.
pub fn smash_stick(move_: Smash, facing: f32) -> [f32; 2] {
    match move_ {
        Smash::Forward => [facing.signum(), 0.0],
        Smash::ForwardUp => [facing.signum(), 0.5],
        Smash::Up => [0.0, 1.0],
        Smash::Down => [0.0, -1.0],
    }
}
/// Technical: Returns the `aerial_stick` result.
///
/// Plain English: Gets the aerial stick result.
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
/// Plain English: Directional A is legal in native jump squat or in the air, never in a grounded turnaround. Jump-only launch avoids accidental grounded tilts.
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
/// Technical: Returns an optional `fast_fall` value.
///
/// Plain English: Gets fast fall if it is available.
pub fn fast_fall(velocity_y: f32) -> Option<[f32; 2]> {
    (velocity_y.is_finite() && velocity_y < -0.05).then_some([0.0, -1.0])
}
