//! Fox's compiled ledge options. Change defaults here or
//! add a fighter-specific handler without changing the shared input engine.
use crate::fighter_profiles::presets::MenuPresets;
pub(crate) const MENU: MenuPresets = super::standard::menu(0, 0, false);
