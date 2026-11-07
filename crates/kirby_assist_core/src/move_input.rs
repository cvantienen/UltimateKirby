//! Physical-button recipes shared by assists. A must map to Attack and X/ZL
//! to Jump in Smash's controller setup. These are requests, not status writes.
//! Native status/motion confirmation remains the caller's acceptance signal.
use crate::native_input::{MAPPER_A, MAPPER_X, MAPPER_ZL};

pub const SHORT_HOP: u32 = MAPPER_X | MAPPER_ZL;
pub const FULL_HOP: u32 = MAPPER_X;
pub const ATTACK: u32 = MAPPER_A;
pub const SHORT_HOP_ATTACK: u32 = MAPPER_X | MAPPER_A;
pub const SHORT_HOP_PULSE: u32 = MAPPER_X; // Release at native jump squat.
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
    attempts: u8,
    last_tick: Option<i64>,
}
impl DeliveryBudget {
    pub const fn new() -> Self {
        Self {
            attempts: 0,
            last_tick: None,
        }
    }
    pub fn mapped(&mut self, tick: i64) {
        if self.last_tick != Some(tick) {
            self.attempts = self.attempts.saturating_add(1);
            self.last_tick = Some(tick);
        }
    }
    pub fn exhausted(&self) -> bool {
        self.attempts >= 24
    }
}
/// A delivered automatic Up-B may appear in native status after shield hitstop
/// or input delay. Remember receipt only; never emit/retry inputs while waiting.
#[derive(Default)]
pub(crate) struct UpBReceipt {
    sent: bool,
    ticks: u8,
    last_tick: Option<i64>,
}
impl UpBReceipt {
    pub const fn new() -> Self {
        Self {
            sent: false,
            ticks: 0,
            last_tick: None,
        }
    }
    pub fn mapped(&mut self, tick: i64) {
        if !self.sent {
            self.sent = true;
            self.ticks = 1;
            self.last_tick = Some(tick);
        } else {
            self.tick(tick);
        }
    }
    pub fn tick(&mut self, tick: i64) {
        if self.sent && self.last_tick != Some(tick) {
            self.last_tick = Some(tick);
            self.ticks = self.ticks.saturating_add(1);
        }
    }
    pub fn sent(&self) -> bool {
        self.sent
    }
    pub fn expired(&self) -> bool {
        self.ticks >= 24
    }
}
/// Three native frames pressed, one neutral frame, then a fresh edge.
/// The owner retains its normal reach/recovery gates and acceptance budget.
pub fn delivery_pressed(age: f32) -> bool {
    age % 4.0 < 3.0
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
pub fn output(stick: [f32; 2], requested: u32) -> crate::gameplay::ControllerOutput {
    crate::gameplay::ControllerOutput {
        owned: crate::native_input::ASSIST_BUTTONS,
        requested,
        stick: Some(stick),
        consume_trigger: false,
        native_short_hop: false,
    }
}
#[derive(Clone, Copy)]
pub enum Aerial {
    Neutral,
    Forward,
    Back,
    Up,
    Down,
}
#[derive(Clone, Copy)]
pub enum Smash {
    Forward,
    ForwardUp,
    Up,
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
        || matches!(situation, 0 | 2)
            && matches!(
                status,
                crate::native_states::status::JUMP
                    | crate::native_states::status::FALL
                    | crate::native_states::status::FALL_AERIAL
            )
}
pub fn fast_fall(velocity_y: f32) -> Option<[f32; 2]> {
    (velocity_y.is_finite() && velocity_y < -0.05).then_some([0.0, -1.0])
}
