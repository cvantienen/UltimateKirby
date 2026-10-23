//! Sonic's saved ledge choices reuse the bounded, native-confirmed input engine.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::enable_roll;
use crate::fighter_profiles::presets::{MenuPresets, SavedChoice, SavedList};

// Neutral nair and shield-to-nair defaults. No grounded Spring Jump attack.
const BASE: MenuPresets = super::standard::menu(0, 0, false);
static ROLL_OPTIONS: [SavedChoice; 2] = [
    BASE.lists[2].options[0],
    SavedChoice::new("grab", "Grab", respond_to_roll_grab, enable_roll),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        BASE.lists[0],
        BASE.lists[1],
        SavedList::new(&ROLL_OPTIONS),
        BASE.lists[3],
        BASE.lists[4],
    ],
};

fn respond_to_roll_grab(
    trap: &mut LedgeTrap,
    settings: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    // Uses the locked roll destination; native capture ends input ownership.
    trap.respond_to_roll_grab(settings, ctx, enemy, out);
}
