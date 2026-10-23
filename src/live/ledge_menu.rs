//! Saved fighter ledge choices. No playback, target acquisition, or combo reads.
//! Resolve response metadata on edits; rebuild UI strings only on revisions.
use super::PROFILES;
use kirby_assist_core::{ledge_menu::MenuSelection, ledge_trap::LedgeSettings};
use std::collections::BTreeMap;

#[cfg(feature = "imgui-menu")]
mod menu;

#[derive(Default)]
pub(super) struct LedgeMenu {
    selections: BTreeMap<i32, MenuSelection>,
    kind: Option<i32>,
    pub consumed: u32,
    pub ledge_held: bool,
    pub two_frame_held: bool,
    scope: bool,
    resolved: Option<LedgeSettings>,
    resolved_choices: Option<[usize; 5]>,
    view_revision: Option<u64>,
    auto_mash: Option<bool>,
}
impl LedgeMenu {
    pub fn auto_mash(&self, startup_default: bool) -> bool {
        self.auto_mash.unwrap_or(startup_default)
    }
    pub fn invalidate(&mut self) {
        self.ledge_held = false;
        self.two_frame_held = false;
        self.scope = false;
        if let Some(selection) = self.kind.and_then(|kind| self.selections.get_mut(&kind)) {
            selection.cancel_actions();
        }
        // Saved metadata remains valid through damage, death and respawn.
    }
    pub fn begin_match(&mut self) {
        self.invalidate();
        self.consumed = 0;
        for selection in self.selections.values_mut() {
            selection.begin_match();
        }
        self.view_revision = None;
    }
    pub fn end_match(&mut self) {
        self.invalidate();
        self.consumed = 0;
        if let Some(selection) = self.kind.and_then(|kind| self.selections.get_mut(&kind)) {
            selection.reset_inputs();
        }
        self.kind = None;
        self.resolved = None;
        self.resolved_choices = None;
        self.view_revision = None;
        #[cfg(feature = "imgui-menu")]
        menu::clear();
    }
    pub fn display_battle(&self, active: bool) {
        #[cfg(feature = "imgui-menu")]
        menu::set_battle_active(active);
        #[cfg(not(feature = "imgui-menu"))]
        let _ = active;
    }
    /// True releases old proposals when navigating or changing a tool.
    pub fn input(
        &mut self,
        available: bool,
        gameplay: bool,
        kind: Option<i32>,
        buttons: u32,
        base: LedgeSettings,
        mash_default: bool,
    ) -> bool {
        let profile = kind.and_then(|kind| PROFILES.get()?.profile(kind));
        let (Some(kind), Some(profile)) = (kind, profile.filter(|_| available)) else {
            self.invalidate();
            self.consumed = 0;
            return false;
        };
        if self.kind != Some(kind) {
            #[cfg(feature = "diagnostic-logging")]
            crate::calibration::clear();
            self.invalidate();
            self.kind = Some(kind);
            self.resolved = None;
            self.resolved_choices = None;
            self.view_revision = None;
            if let Some(selection) = self.selections.get_mut(&kind) {
                selection.reset_inputs();
            }
        }
        self.scope = gameplay;
        let selection = self
            .selections
            .entry(kind)
            .or_insert_with(|| MenuSelection::new(&profile.menu_options));
        let selection_buttons = if kind == kirby_assist_core::fighter_profiles::kirby::FIGHTER_KIND
        {
            buttons
        } else {
            buttons & !kirby_assist_core::ledge_menu::TWO_FRAME_TRIGGER
        };
        self.consumed = selection.input(selection_buttons, &profile.menu_options);
        if kind != kirby_assist_core::fighter_profiles::kirby::FIGHTER_KIND {
            self.consumed &= !kirby_assist_core::ledge_menu::TWO_FRAME_TRIGGER;
        }
        let auto_mash = self.auto_mash.get_or_insert(mash_default);
        if selection.toggle_mash {
            *auto_mash = !*auto_mash;
            self.view_revision = None;
        }
        let changed = selection.cancel_pending;
        if !gameplay {
            selection.cancel_actions();
        }
        self.ledge_held = gameplay && selection.ledge_held(selection_buttons);
        self.two_frame_held = gameplay
            && kind == kirby_assist_core::fighter_profiles::kirby::FIGHTER_KIND
            && selection.two_frame_held(buttons);
        if self.resolved_choices != Some(selection.choices) {
            self.resolved = Some(selection.settings(&profile.menu_options, base));
            self.resolved_choices = Some(selection.choices);
        }
        #[cfg(feature = "imgui-menu")]
        if self.view_revision != Some(selection.revision)
            && menu::publish(profile, selection, *auto_mash)
        {
            self.view_revision = Some(selection.revision);
        }
        changed
    }
    pub fn settings(&self, kind: Option<i32>, base: LedgeSettings) -> LedgeSettings {
        if self.scope && self.kind == kind {
            LedgeSettings {
                two_frame: self.two_frame_held,
                two_frame_down_tilt: self.two_frame_held,
                ..self.resolved.unwrap_or(base)
            }
        } else {
            base
        }
    }
}
pub(super) fn install() {
    #[cfg(feature = "imgui-menu")]
    menu::install();
}
