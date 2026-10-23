//! Per-fighter sequence decisions, separate from native attack frame data.
//! Adaptive hop selection uses instance physics rather than a character-name
//! guess. Reach remains the explicitly configured forecast envelope, not a
//! claimed measurement of each move's collision volume.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hop {
    #[default]
    Short,
    Full,
    /// Prefer a short hop that covers the target; otherwise try full hop.
    Adaptive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AerialExecution {
    pub neutral: Hop,
    pub jump: Hop,
    pub shield: Hop,
}
impl AerialExecution {
    pub const SHORT: Self = Self {
        neutral: Hop::Short,
        jump: Hop::Short,
        shield: Hop::Short,
    };
    pub const ADAPTIVE_JUMP: Self = Self {
        jump: Hop::Adaptive,
        ..Self::SHORT
    };
}
impl Default for AerialExecution {
    fn default() -> Self {
        Self::SHORT
    }
}
