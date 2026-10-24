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
    pub duck_held: bool,
    scope: bool,
    roll_read_blocked: bool,
    resolved: Option<LedgeSettings>,
    resolved_choices: Option<[usize; 5]>,
    view_revision: Option<u64>,
    auto_mash: Option<bool>,
    // Retain a real release through first profile selection/session reset.
    passthrough_buttons: Option<u32>,
    open_requested: bool,
    resume_saved_state: bool,
    #[cfg(feature = "diagnostic-logging")]
    diagnostic_config: Option<kirby_assist_core::diagnostics::modes::Configuration>,
}
impl LedgeMenu {
    pub fn reaction_modes(&self) -> (bool, bool) {
        self.kind
            .and_then(|kind| self.selections.get(&kind))
            .map_or((false, false), |s| {
                (self.scope && s.spot_punish, self.scope && s.grab_dodge)
            })
    }
    pub fn jab_lock_enabled(&self) -> bool {
        self.scope && self.kind == Some(5) && self.selections.get(&5).is_some_and(|s| s.jab_lock)
    }
    pub fn jab_grab_enabled(&self) -> bool {
        self.scope && self.kind == Some(6) && self.selections.contains_key(&6)
    }
    pub fn jab_followup(&self) -> kirby_assist_core::jab_grab::Followup {
        self.selections
            .get(&6)
            .map_or(Default::default(), |s| s.jab_grab)
    }

    pub fn resume_saved_state(&mut self) {
        self.resume_saved_state = true;
    }
    pub fn posture_modes(&self) -> (bool, bool) {
        if !self.scope || self.kind != Some(6) {
            return (false, false);
        }
        self.selections
            .get(&6)
            .map_or((false, false), |s| (s.idle_crouch, s.tilt_turn))
    }
    pub fn finisher_input(
        &self,
        buttons: u32,
    ) -> Option<(kirby_assist_core::button_modes::YMode, bool)> {
        if !self.scope || !self.kind.is_some_and(|kind| matches!(kind, 5 | 6)) {
            return None;
        }
        self.kind
            .and_then(|kind| self.selections.get(&kind))
            .map(|s| (s.y_mode, s.finisher_held(buttons)))
    }
    pub fn request_open(&mut self) {
        self.open_requested = true;
    }
    pub fn roll_read_choice(&self, kind: i32) -> kirby_assist_core::roll_read::Choice {
        if self.scope
            && self.kind == Some(kind)
            && kirby_assist_core::roll_read::fighter_config(kind).is_some()
        {
            self.selections
                .get(&kind)
                .map_or(kirby_assist_core::roll_read::Choice::Off, |s| s.roll_read)
        } else {
            kirby_assist_core::roll_read::Choice::Off
        }
    }
    pub fn observe_passthrough(&mut self, buttons: u32) {
        self.passthrough_buttons = Some(buttons);
        for selection in self.selections.values_mut() {
            selection.observe_passthrough(buttons);
        }
    }
    pub fn auto_mash(&self, startup_default: bool) -> bool {
        self.auto_mash.unwrap_or(startup_default)
    }
    pub fn invalidate(&mut self) {
        self.ledge_held = false;
        self.duck_held = false;
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
        self.resume_saved_state = false;
        self.open_requested = false;
        self.invalidate();
        self.consumed = 0;
        if let Some(selection) = self.kind.and_then(|kind| self.selections.get_mut(&kind)) {
            selection.reset_inputs();
        }
        self.kind = None;
        #[cfg(feature = "diagnostic-logging")]
        {
            self.diagnostic_config = None;
        }
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
        grab_default: bool,
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
        self.roll_read_blocked = buttons
            & (kirby_assist_core::manual_controls::DPAD_SIDES
                | kirby_assist_core::manual_controls::DPAD_UP
                | kirby_assist_core::ledge_trap::DPAD_DOWN
                | kirby_assist_core::ledge_menu::MENU_STICK_CLICK
                | kirby_assist_core::ledge_menu::TWO_FRAME_TRIGGER
                | kirby_assist_core::manual_controls::SHORT_HOP_TRIGGER
                | kirby_assist_core::native_input::MAPPER_Y
                | kirby_assist_core::native_input::MAPPER_MINUS
                | kirby_assist_core::native_input::MAPPER_PLUS)
            != 0;
        let selection = self.selections.entry(kind).or_insert_with(|| {
            let mut s = MenuSelection::new(&profile.menu_options);
            s.grab_dodge = grab_default;
            s
        });
        selection.roll_read_available =
            kirby_assist_core::roll_read::fighter_config(kind).is_some();
        selection.auto_duck_available = matches!(kind, 5 | 6);
        selection.y_fighter_kind = kind;
        if selection.auto_duck_available && !selection.y_mode.supports(kind) {
            selection.y_mode = kirby_assist_core::button_modes::YMode::options_for_kind(kind)[0];
        }
        // Apply after new-profile/reset_inputs, which deliberately disarms L.
        if let Some(buttons) = self.passthrough_buttons.take() {
            selection.observe_passthrough(buttons);
        }
        if self.resume_saved_state && gameplay {
            selection.resume_training_ledge_hold();
            self.resume_saved_state = false;
        }
        self.consumed = selection.input(buttons, &profile.menu_options);
        if self.open_requested {
            self.open_requested = false;
            selection.show();
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
        self.ledge_held = gameplay && selection.ledge_held(buttons);
        self.duck_held = gameplay && selection.auto_duck_held(buttons);
        if self.resolved_choices != Some(selection.choices) {
            self.resolved = Some(selection.settings(&profile.menu_options, base));
            self.resolved_choices = Some(selection.choices);
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            use kirby_assist_core::diagnostics::modes::Configuration;
            let on_off = |on| if on { "ON" } else { "OFF" };
            let mut rows = ["unavailable for this fighter"; 14];
            for (i, list) in profile.menu_options.lists.iter().enumerate() {
                rows[i] = list
                    .options
                    .get(selection.choices[i])
                    .map_or("unknown selection", |choice| choice.name);
            }
            if kirby_assist_core::roll_read::fighter_config(kind).is_some() {
                rows[5] = selection.roll_read.label();
            }
            if matches!(kind, 5 | 6) {
                rows[6] = selection.y_mode.label();
            }
            rows[7] = on_off(*auto_mash);
            if kind == 6 {
                rows[8] = on_off(selection.idle_crouch);
                rows[9] = on_off(selection.tilt_turn);
                rows[10] = selection.jab_grab.label();
            }
            if kind == 5 {
                rows[11] = on_off(selection.jab_lock);
            }
            rows[12] = on_off(selection.spot_punish);
            rows[13] = on_off(selection.grab_dodge);
            let report = Configuration {
                fighter: profile.name,
                rows,
            };
            if self.diagnostic_config != Some(report) {
                self.diagnostic_config = Some(report);
                crate::logging::write_configuration(report);
            }
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
                two_frame: false,
                two_frame_down_tilt: false,
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
