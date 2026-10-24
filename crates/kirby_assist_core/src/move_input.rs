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
/// Three native frames pressed, one neutral frame, then a fresh edge.
/// The owner retains its normal reach/recovery gates and acceptance budget.
pub fn delivery_pressed(age: f32) -> bool {
    age % 4.0 < 3.0
}
/// Owning only left-stick aim must not erase a physical tilt-stick attack.
/// Full automated sequences still suppress competing physical aerials/tilts.
pub fn blocks_physical_c_stick(owned: u32) -> bool {
    owned & crate::ledge_trap::LEDGE_BUTTONS != 0
}
pub fn output(stick: [f32; 2], requested: u32) -> crate::ledge_trap::LedgeOutput {
    crate::ledge_trap::LedgeOutput {
        owned: crate::ledge_trap::LEDGE_BUTTONS,
        requested,
        stick: Some(stick),
        consume_trigger: false,
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
    status == 0xa || matches!(situation, 0 | 2) && matches!(status, 0xb | 0xe | 0xf)
}
pub fn fast_fall(velocity_y: f32) -> Option<[f32; 2]> {
    (velocity_y.is_finite() && velocity_y < -0.05).then_some([0.0, -1.0])
}
