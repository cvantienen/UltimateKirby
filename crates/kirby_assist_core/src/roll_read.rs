//! Shared copied opponent facts used by reaction and out-of-shield policies.

pub const ESCAPE_F: i32 = crate::native_states::ESCAPE_F;
pub const ESCAPE_B: i32 = crate::native_states::ESCAPE_B;

#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub status: i32,
    /// Copied native motion identity for the configurable rule clock.
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub position: [f32; 2],
    pub velocity_x: f32,
    pub vulnerable_in: Option<f32>,
    pub actionable_in: Option<f32>,
}
