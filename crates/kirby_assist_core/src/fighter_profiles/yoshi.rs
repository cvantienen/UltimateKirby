//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "yoshi",
        fighter_kind: crate::fighter_kinds::YOSHI,
        // Up-air/fair keep adaptive hops. Jump-nair has its own immediate
        // short-hop sequence in ledge_trap/fighters/yoshi.rs.
        grab: super::FighterGrabData::from_kind(5),

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
