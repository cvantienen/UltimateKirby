//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::*;
/// Startup from Ultimate Frame Data; root-distance estimates need Switch tuning.
pub const JAB_LOCK: crate::jab_lock::Config = crate::jab_lock::Config {
    jab_start: 3.0,
    jab_reach: 7.0,
    smash_start: 7.0,
    smash_reach: 12.0,
    grab_start: 14.0,
    grab_reach: 23.0,
    chase_distance: 60.0,
};
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "yoshi",
        fighter_kind: 5,
        // Up-air/fair keep adaptive hops. Jump-nair has its own immediate
        // short-hop sequence in ledge_trap/fighters/yoshi.rs.
        ledge: LEDGE_DATA,

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
