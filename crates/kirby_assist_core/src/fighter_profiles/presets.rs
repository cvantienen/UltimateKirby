//! Compiled response catalog. Every entry binds a label to a real Rust function
//! and a small preparation function. No parsing, file loading or action strings.
use crate::ledge_trap::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetSlot {
    Neutral,
    Jump,
    Roll,
    Attack,
    Drop,
}
impl PresetSlot {
    pub const ALL: [Self; 5] = [
        Self::Neutral,
        Self::Jump,
        Self::Roll,
        Self::Attack,
        Self::Drop,
    ];
}
/// All fighter response functions share copied facts and the existing session.
/// A function can use shared input/timing helpers or implement its own sequence.
pub type ResponseHandler =
    fn(&mut LedgeTrap, LedgeSettings, LedgeContext, Opponent, &mut LedgeOutput);
pub type PrepareResponse = fn(LedgeSettings) -> LedgeSettings;

#[derive(Clone, Copy, Debug)]
pub struct SavedChoice {
    pub id: &'static str,
    pub name: &'static str,
    pub respond: ResponseHandler,
    pub prepare: PrepareResponse,
}
impl SavedChoice {
    pub const fn new(
        id: &'static str,
        name: &'static str,
        respond: ResponseHandler,
        prepare: PrepareResponse,
    ) -> Self {
        Self {
            id,
            name,
            respond,
            prepare,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SavedList {
    pub selected: usize,
    pub options: &'static [SavedChoice],
}
impl SavedList {
    pub const fn new(options: &'static [SavedChoice]) -> Self {
        Self {
            selected: 0,
            options,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MenuPresets {
    pub lists: [SavedList; 5],
}
// Unit fixtures use Kirby; production profiles must explicitly choose a catalog.
#[cfg(test)]
impl Default for MenuPresets {
    fn default() -> Self {
        crate::ledge_trap::fighters::kirby::MENU
    }
}
impl MenuPresets {
    pub fn list(&self, slot: PresetSlot) -> &SavedList {
        &self.lists[slot as usize]
    }
}

#[cfg(test)]
mod tests;
