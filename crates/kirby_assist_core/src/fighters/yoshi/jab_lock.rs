//! Yoshi-specific defaults for the shared jab-lock policy.

use super::Config;

pub(super) fn config() -> Option<Config> {
    let kind = crate::fighter_kinds::YOSHI;
    let active_start = |name| {
        crate::fighter_data::named_move(kind, name)?
            .active
            .iter()
            .map(|span| span.first)
            .min()
            .map(f32::from)
    };
    let fighter = crate::fighter_data::fighter(kind)?;
    let grab = crate::fighter_data::ordinary_grab(kind, crate::native_states::CATCH)?;
    Some(Config {
        jab_start: active_start("Jab 1")?,
        jab_reach: 7.0,
        smash_start: active_start("Down Smash")?,
        smash_reach: 12.0,
        grab_start: f32::from(grab.first),
        grab_reach: fighter.numeric_stat("grabs.grabrange"),
        chase_distance: 60.0,
    })
}
