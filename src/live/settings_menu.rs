//! Cached per-fighter menu selections and settings. No playback, target
//! acquisition, or combo reads; resolve response metadata only on edits.
use super::PROFILES;
use kirby_assist_core::button_bindings::Mode;
use kirby_assist_core::{settings::AssistSettings, settings_menu::MenuSelection};
use std::collections::BTreeMap;

#[cfg(feature = "imgui-menu")]
#[path = "settings_menu/menu.rs"]
mod menu;
#[path = "settings_menu/save.rs"]
mod save;

// Host adapters and worker tests share the same process-wide save globals.
// Serialize simulated mapper polling with worker tests just as STATE does in game.
#[cfg(test)]
pub(super) fn save_test_guard() -> std::sync::MutexGuard<'static, ()> {
    save::WORKER_TEST.lock().unwrap()
}

pub(super) fn recover_saved_file() -> Result<(), String> {
    save::recover()
}

pub(super) fn observe_local_fighter(now: std::time::Instant) {
    #[cfg(feature = "imgui-menu")]
    menu::observe_local_fighter(now);
    #[cfg(not(feature = "imgui-menu"))]
    let _ = now;
}

pub(super) fn show_master_status(enabled: bool) {
    #[cfg(feature = "imgui-menu")]
    menu::show_master_status(enabled);
    #[cfg(not(feature = "imgui-menu"))]
    let _ = enabled;
}

// Loss of a local fighter callback (death/respawn) is not proof the match
// ended. Release retained session data only after the existing master match
// guard unlocks; the renderer handles that one-shot cleanup without new queries.
static CLEANUP_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(super) fn cleanup_pending() -> bool {
    CLEANUP_PENDING.load(std::sync::atomic::Ordering::Acquire)
}

// Run only from the existing mapper callback, before it acquires STATE.
// Applying a save result can allocate menu drafts; cleanup frees editor data.
// Neither belongs inside the shared ImGui/GPU command-submission callback.
pub(super) fn poll_maintenance() {
    use std::sync::atomic::Ordering;
    #[cfg(feature = "imgui-menu")]
    menu::finish_deferred_clear();
    let cleanup = cleanup_pending() && !super::MASTER_IN_MATCH.load(Ordering::Acquire);
    if !cleanup && !save::has_result() {
        return;
    }
    let Ok(mut state) = super::STATE.try_lock() else {
        return;
    };
    let cleanup =
        cleanup && !state.menu_battle_active && !super::MASTER_IN_MATCH.load(Ordering::Acquire);
    if let Some(menu) = state.settings_menu.as_mut() {
        if cleanup {
            menu.end_match();
        }
        if save::has_result() {
            menu.poll_save_result();
        }
    } else {
        // The owner may have been removed after Training ended. Join only an
        // already-finished worker and release its result instead of retaining it.
        drop(save::poll());
        if cleanup {
            CLEANUP_PENDING.store(false, Ordering::Release);
        }
    }
}

#[derive(Default)]
pub(super) struct SettingsMenu {
    selections: BTreeMap<i32, MenuSelection>,
    kind: Option<i32>,
    pub consumed: u32,
    pub native_buttons: u32,
    pub builtin_consumed: u32,
    pub ledge_held: bool,
    pub duck_held: bool,
    scope: bool,
    resolved: Option<AssistSettings>,
    resolved_choices:
        Option<[usize; kirby_assist_core::fighter_profiles::presets::PresetSlot::COUNT]>,
    resolved_recipe_revision: Option<u64>,
    resolved_preview: Option<kirby_assist_core::fighter_profiles::presets::PresetSlot>,
    view_revision: Option<u64>,
    rule_cache: Option<Box<kirby_assist_core::reaction_rules::Resolved>>,
    rule_stamp: Option<(i32, u64)>,
    auto_mash: Option<bool>,
    // Retain a real release through first profile selection/session reset.
    passthrough_buttons: Option<(u32, kirby_assist_core::native_input::ControlLayout)>,
    open_requested: bool,
    resume_saved_state: bool,
    #[cfg(feature = "diagnostic-logging")]
    diagnostic_config: Option<kirby_assist_core::diagnostics::modes::Configuration>,
    #[cfg(feature = "diagnostic-logging")]
    diagnostic_stamp: Option<(i32, u64, bool)>,
    #[cfg(feature = "diagnostic-logging")]
    retained_stamp: Option<(i32, u64)>,
}
impl SettingsMenu {
    /// Native fighter changes can precede the next mapper/menu poll. Never
    /// expose a previous fighter's automatic rules or ranges in that gap.
    pub fn for_fighter(&self, kind: i32) -> Option<&Self> {
        (self.kind == Some(kind)).then_some(self)
    }
    #[cfg(feature = "diagnostic-logging")]
    pub fn selected_choice_labels(
        &self,
        kind: i32,
        slot: kirby_assist_core::fighter_profiles::presets::PresetSlot,
    ) -> Option<(&str, &str)> {
        self.for_fighter(kind)?;
        let profile = PROFILES.get()?.profile(kind)?;
        let selection = self.selections.get(&kind)?;
        selection.choice_labels(
            &profile.menu_options,
            slot,
            selection.choices[slot as usize],
        )
    }
    pub fn preview_slot(&self) -> Option<kirby_assist_core::fighter_profiles::presets::PresetSlot> {
        self.kind
            .and_then(|k| self.selections.get(&k))
            .and_then(|s| s.preview_slot)
    }
    pub(super) fn poll_save_result(&mut self) -> bool {
        save::poll().is_some_and(|result| self.apply_save_result(result, PROFILES.get()))
    }
    fn apply_save_result(
        &mut self,
        result: save::ResultMessage,
        profiles: Option<&kirby_assist_core::fighter_profiles::FighterProfiles>,
    ) -> bool {
        let mut file_list_changed = false;
        {
            #[cfg(feature = "diagnostic-logging")]
            {
                self.retained_stamp = None;
            }
            let result_kind = result.kind;
            if let Some(saved) = self.selections.get_mut(&result_kind) {
                if let Some(tuning) = result.tuning {
                    saved.save_status = Some(if result.success && saved.tuning != tuning {
                        "Saved earlier fighter settings; newer edits are unsaved".into()
                    } else {
                        result.message
                    });
                    saved.revision = saved.revision.wrapping_add(1);
                    saved.content_revision = saved.content_revision.wrapping_add(1);
                    return false;
                }
                // A completed save can arrive after a new match/profile reload.
                // Numeric draft indices are not identities: never mutate a new
                // template or different recipe that now occupies the old index.
                let same_recipe = profiles
                    .and_then(|p| p.profile(result_kind))
                    .and_then(|p| saved.choice(&p.menu_options, result.slot, result.index))
                    .is_some_and(|c| result.choice.as_deref() == Some(c.id));
                if result.success && !same_recipe {
                    saved.save_status = Some(result.message);
                    saved.revision = saved.revision.wrapping_add(1);
                    saved.content_revision = saved.content_revision.wrapping_add(1);
                    return false;
                }
                if result.success && result.deleted {
                    saved.cancel_actions();
                    file_list_changed = Some(result_kind) == self.kind;
                }
                let snapshot_matches = result.snapshot.as_ref().map_or(
                    saved.recipe_revision == result.revision,
                    |(sequence, name)| {
                        profiles
                            .and_then(|p| p.profile(result_kind))
                            .is_some_and(|profile| {
                                saved.saved_recipe_matches(
                                    &profile.menu_options,
                                    result.slot,
                                    result.index,
                                    *sequence,
                                    name.as_deref(),
                                )
                            })
                    },
                );
                if result.success && !result.deleted && snapshot_matches {
                    saved.mark_recipe_saved(result.slot, result.index);
                }
                if result.success {
                    if let Some(profile) = profiles.and_then(|p| p.profile(result_kind)) {
                        saved.complete_saved_snapshot(
                            &profile.menu_options,
                            result.slot,
                            result.index,
                            result.deleted,
                            result.snapshot.as_ref().map(|(sequence, _)| *sequence),
                            result
                                .snapshot
                                .as_ref()
                                .and_then(|(_, name)| name.as_deref()),
                        );
                        if saved.lightweight {
                            saved.release_training_tools(&profile.menu_options);
                        }
                        if self.kind == Some(result_kind) {
                            self.resolved_recipe_revision = None;
                            self.rule_stamp = None;
                            self.view_revision = None;
                        }
                    }
                }
                let current = saved.row.slot() == Some(result.slot)
                    && saved.choices[result.slot as usize] == result.index;
                saved.save_status = Some(if !current {
                    format!("{:?}: {}", result.slot, result.message)
                } else if !snapshot_matches
                    && (result.message.starts_with("Saved")
                        || result.message.starts_with("Already saved"))
                {
                    "Saved earlier snapshot; newer edits are unsaved".into()
                } else {
                    result.message
                });
                saved.revision = saved.revision.wrapping_add(1);
                saved.content_revision = saved.content_revision.wrapping_add(1);
            }
        }
        file_list_changed
    }
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.ledge_held
            || self.scope
                && self
                    .kind
                    .and_then(|k| self.selections.get(&k))
                    .is_some_and(|s| s.sequence_enabled(false))
    }
    pub fn manual_native_actions(&self) -> u32 {
        let Some(kind) = self.kind.filter(|_| self.scope) else {
            return 0;
        };
        let Some(profile) = PROFILES.get().and_then(|p| p.profile(kind)) else {
            return 0;
        };
        self.selections
            .get(&kind)
            .map_or(0, |s| s.manual_native_actions(&profile.menu_options))
    }
    pub fn reserves_automatic_modes(&self) -> bool {
        self.scope
            && self
                .kind
                .and_then(|k| self.selections.get(&k))
                .is_some_and(|s| s.reserves_automatic_modes())
    }
    pub fn finish_preview(&mut self) {
        if let Some(s) = self.kind.and_then(|k| self.selections.get_mut(&k)) {
            if s.preview_slot.take().is_some() {
                s.revision = s.revision.wrapping_add(1);
            }
        }
    }
    pub fn ledge_trump_held(&self) -> bool {
        self.scope
            && self
                .kind
                .and_then(|k| self.selections.get(&k))
                .is_some_and(MenuSelection::ledge_trump_held)
    }
    pub fn auto_tech(&self, startup_default: bool) -> bool {
        self.kind
            .and_then(|kind| self.selections.get(&kind))
            .map_or(startup_default, |s| {
                s.effective_mode(Mode::AutoTech, s.auto_tech)
            })
    }
    pub fn grab_dodge_enabled(&self) -> bool {
        self.scope
            && self
                .kind
                .and_then(|k| self.selections.get(&k))
                .is_some_and(|s| s.grab_dodge)
    }
    #[cfg(feature = "diagnostic-logging")]
    pub fn diagnostic_options(&self) -> bool {
        self.kind
            .and_then(|kind| self.selections.get(&kind))
            .is_some_and(|s| s.performance)
    }
    pub fn out_of_shield_enabled(&self) -> bool {
        self.scope
            && self
                .kind
                .and_then(|k| self.selections.get(&k))
                .is_some_and(|s| s.out_of_shield)
    }
    pub fn jab_lock_enabled(&self) -> bool {
        self.scope
            && self.kind == Some(5)
            && self
                .selections
                .get(&5)
                .is_some_and(|s| s.effective_mode(Mode::JabLock, s.jab_lock))
    }

    pub fn resume_saved_state(&mut self) {
        self.resume_saved_state = true;
    }
    pub fn posture_modes(&self) -> (bool, bool) {
        if !self.scope {
            return (false, false);
        }
        self.kind
            .and_then(|k| self.selections.get(&k))
            .map_or((false, false), |s| {
                (
                    self.kind == Some(6) && s.effective_mode(Mode::IdleCrouch, s.idle_crouch),
                    s.effective_mode(Mode::TiltTurn, s.tilt_turn)
                        && self.kind.is_some_and(|kind| s.tuning.tilt_turn(kind).any()),
                )
            })
    }
    pub fn request_open(&mut self) {
        self.open_requested = true;
    }
    pub fn fighter_tuning(&self, kind: i32) -> kirby_assist_core::fighter_tuning::Tuning {
        self.selections
            .get(&kind)
            .filter(|_| self.kind == Some(kind))
            .map_or(Default::default(), |s| s.tuning)
    }
    pub fn reflect_config(
        &self,
        kind: i32,
    ) -> Option<kirby_assist_core::projectile_reflect::Config> {
        self.for_fighter(kind)?;
        let selection = self.selections.get(&kind)?;
        if !self.scope
            || !selection.auto_reflect
            || selection.recipe_editing
            || selection.tuning_page
            || selection.managing()
        {
            return None;
        }
        kirby_assist_core::projectile_reflect::Config::for_fighter(kind, selection.tuning)
    }
    pub fn dodge_lookahead(&self) -> f32 {
        self.kind
            .and_then(|k| self.selections.get(&k))
            .map_or(kirby_assist_core::spot_dodge::LOOKAHEAD, |s| {
                s.tuning.dodge_lookahead()
            })
    }
    pub fn rule_config(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules().filter(|r| {
            r.has_simple_events()
                && r.enabled_controls & kirby_assist_core::reaction_rules::Control::Menu.bit() != 0
        })
    }
    fn current_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        let kind = self.kind?;
        let revision = self.selections.get(&kind)?.recipe_revision;
        (self.scope && self.rule_stamp == Some((kind, revision))).then_some(())?;
        self.rule_cache.as_deref()
    }
    /// Grab avoidance owns its follow-up toggle independently of Auto react.
    /// Only its own event recipe may punish a confirmed grab dodge.
    pub fn grab_dodge_sequence(&self) -> Option<&kirby_assist_core::sequence_config::Sequence> {
        let selection = self.kind.and_then(|kind| self.selections.get(&kind))?;
        if !self.scope
            || !selection.grab_dodge
            || selection.recipe_editing
            || selection.tuning_page
            || selection.managing()
        {
            return None;
        }
        self.current_rules()?.simple_sequence(5)
    }
    /// All Auto react recipes share a revision cache, independent of the
    /// individual mode switches that continue to control their admission.
    pub fn automatic_sequences(&self) -> kirby_assist_core::out_of_shield::AutomaticConfig<'_> {
        let rules = self.current_rules();
        kirby_assist_core::out_of_shield::AutomaticConfig {
            grab: rules.and_then(|r| r.simple_sequence(3)),
            shield_grab: rules.and_then(|r| r.simple_sequence(4)),
            backup: rules.and_then(|r| r.simple_sequence(6)),
        }
    }
    pub fn automatic_limits(&self) -> [Option<(f32, f32)>; 3] {
        let r = self.automatic_sequences();
        [r.grab, r.shield_grab, r.backup].map(|s| s.map(|s| (s.reaction.max, s.reaction.height)))
    }
    pub fn automatic_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules()
    }
    pub fn observe_passthrough(
        &mut self,
        buttons: u32,
        layout: kirby_assist_core::native_input::ControlLayout,
    ) {
        self.passthrough_buttons = Some((buttons, layout));
        for selection in self.selections.values_mut() {
            selection.observe_physical_passthrough(buttons, layout);
        }
    }
    pub fn auto_mash(&self, startup_default: bool) -> bool {
        let base = self.auto_mash.unwrap_or(startup_default);
        self.kind
            .and_then(|k| self.selections.get(&k))
            .map_or(base, |s| s.effective_mode(Mode::AutoMash, base))
    }
    pub fn invalidate(&mut self) {
        self.ledge_held = false;
        self.duck_held = false;
        self.scope = false;
        if let Some(selection) = self.kind.and_then(|kind| self.selections.get_mut(&kind)) {
            if selection.preview_slot.take().is_some() {
                selection.revision = selection.revision.wrapping_add(1);
            }
            selection.cancel_actions();
        }
        // Saved metadata remains valid through damage, death and respawn.
    }
    pub fn begin_match(&mut self) {
        // A positively observed new battle supersedes deferred cleanup for the
        // previous battle. Never let the render callback discard its new menu.
        if CLEANUP_PENDING.swap(false, std::sync::atomic::Ordering::AcqRel) {
            // End-of-match cleanup may have been deferred while the native match
            // lock remained set. Never reuse those drafts/selections in a new match.
            self.selections.clear();
            self.kind = None;
            self.auto_mash = None;
            self.passthrough_buttons = None;
        }
        self.invalidate();
        self.consumed = 0;
        for selection in self.selections.values_mut() {
            selection.begin_match();
        }
        self.view_revision = None;
    }
    pub fn end_match(&mut self) {
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::clear();
        self.resume_saved_state = false;
        self.open_requested = false;
        self.invalidate();
        self.consumed = 0;
        if let Some(selection) = self.kind.and_then(|kind| self.selections.get_mut(&kind)) {
            selection.reset_inputs();
        }
        self.kind = None;
        self.rule_cache = None;
        self.rule_stamp = None;
        let outside_match = !super::MASTER_IN_MATCH.load(std::sync::atomic::Ordering::Acquire);
        CLEANUP_PENDING.store(!outside_match, std::sync::atomic::Ordering::Release);
        if outside_match {
            self.selections.clear();
            self.auto_mash = None;
            self.passthrough_buttons = None;
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            self.diagnostic_config = None;
            self.diagnostic_stamp = None;
            self.retained_stamp = None;
            if outside_match {
                crate::performance::health::retained(0, 0, 0, 0);
            }
        }
        self.resolved = None;
        self.resolved_choices = None;
        self.resolved_recipe_revision = None;
        self.resolved_preview = None;
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
        bindings_blocked: bool,
        safety_jump: bool,
        kind: Option<i32>,
        buttons: u32,
        physical: u32,
        layout: kirby_assist_core::native_input::ControlLayout,
        base: &AssistSettings,
        mash_default: bool,
        grab_default: bool,
        tech_default: bool,
        training: bool,
    ) -> bool {
        self.input_inner(
            available,
            gameplay,
            bindings_blocked,
            safety_jump,
            kind,
            buttons,
            physical,
            layout,
            base,
            mash_default,
            grab_default,
            tech_default,
            training,
        )
    }
    fn input_inner(
        &mut self,
        available: bool,
        gameplay: bool,
        bindings_blocked: bool,
        safety_jump: bool,
        kind: Option<i32>,
        buttons: u32,
        physical: u32,
        layout: kirby_assist_core::native_input::ControlLayout,
        base: &AssistSettings,
        mash_default: bool,
        grab_default: bool,
        tech_default: bool,
        training: bool,
    ) -> bool {
        #[cfg(feature = "diagnostic-logging")]
        let _timing =
            crate::performance::health::Trace::new(crate::performance::health::Lane::MenuInput);
        let profile = kind.and_then(|kind| PROFILES.get()?.profile(kind));
        let (Some(kind), Some(profile)) = (kind, profile.filter(|_| available)) else {
            self.invalidate();
            self.consumed = 0;
            return false;
        };
        let fighter_changed = self.kind != Some(kind);
        if fighter_changed {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::FighterChange);
            #[cfg(feature = "diagnostic-logging")]
            crate::calibration::clear();
            self.invalidate();
            self.selections.retain(|saved_kind, _| *saved_kind == kind);
            self.kind = Some(kind);
            self.rule_cache = None;
            self.rule_stamp = None;
            self.resolved = None;
            self.resolved_choices = None;
            self.resolved_recipe_revision = None;
            self.view_revision = None;
            if let Some(selection) = self.selections.get_mut(&kind) {
                selection.reset_inputs();
            }
        }
        self.selections.entry(kind).or_insert_with(|| {
            let mut s = MenuSelection::new(&profile.menu_options);
            s.simplified_reactions = true;
            if let Some(saved) = super::SEQUENCES.get() {
                let library = saved.lock().unwrap_or_else(|e| e.into_inner());
                let catalog = library.for_fighter(kind);
                s.load_saved_recipes(&profile.menu_options, &catalog, kind);
                s.tuning = library.tuning.get(&kind).copied().unwrap_or_default();
            }
            s.tuning_values =
                kirby_assist_core::fighter_tuning::Tuning::values(profile.apply(*base));
            s.tuning_values[11] = if grab_default { 1.0 } else { 0.0 };
            s.grab_dodge = s.tuning.grab_dodge_default(grab_default);
            s.out_of_shield = s.tuning.oos_default();
            s.auto_reflect =
                s.tuning.0[kirby_assist_core::fighter_tuning::REFLECT_DEFAULT] == Some(1.);
            s.auto_tech = tech_default;
            s
        });
        let file_list_changed = self.poll_save_result();
        self.scope = gameplay;
        let selection = self.selections.get_mut(&kind).unwrap();
        selection.y_fighter_kind = kind;
        // Apply after new-profile/reset_inputs, which deliberately disarms L.
        if let Some((buttons, layout)) = self.passthrough_buttons.take() {
            selection.observe_physical_passthrough(buttons, layout);
        }
        if self.resume_saved_state && gameplay {
            selection.resume_training_ledge_hold();
            self.resume_saved_state = false;
        }
        // Scene permission was copied before the caller acquired STATE. Do
        // not call native game functions from this locked menu adapter.
        selection.block_binding_starts(bindings_blocked);
        selection.preserve_safety_jump(safety_jump);
        selection.set_binding_input(physical, layout);
        let binding_choices_before = (selection.choices, selection.recipe_revision);
        self.consumed = if selection.waits_for_training_save(training, save::busy()) {
            selection.wait_for_save(buttons)
        } else {
            selection.input_with_editing(buttons, &profile.menu_options, training)
        };
        // capture_binding_input already refreshed these once. Repeat only when
        // this menu input actually changed a choice or its recipe metadata.
        if binding_choices_before != (selection.choices, selection.recipe_revision) {
            selection.refresh_native_bindings(&profile.menu_options);
        }
        self.native_buttons = selection.native_buttons(buttons);
        self.builtin_consumed = selection.builtin_consumed;
        if self.rule_stamp != Some((kind, selection.recipe_revision)) {
            self.rule_stamp = Some((kind, selection.recipe_revision));
            self.rule_cache = match if selection.has_reactions {
                selection.resolve_reactions(&profile.menu_options)
            } else {
                Ok(Default::default())
            } {
                Ok(r) if r.len > 0 => Some(Box::new(r)),
                Ok(_) => None,
                Err(e) => {
                    selection.save_status = Some(e);
                    selection.revision = selection.revision.wrapping_add(1);
                    None
                }
            };
        }
        if let Some(rules) = self.rule_cache.as_mut() {
            rules.set_enabled_controls(selection.reaction_enabled_controls());
        }
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::set_test_latency(selection.test_latency);
        if self.open_requested {
            self.open_requested = false;
            selection.show_quick();
        }
        let auto_mash = self.auto_mash.get_or_insert(mash_default);
        if selection.toggle_mash {
            *auto_mash = !*auto_mash;
            self.view_revision = None;
        }
        let changed = selection.cancel_pending || file_list_changed;
        if !gameplay {
            selection.preview_slot = None;
            selection.cancel_actions();
        }
        self.ledge_held = gameplay
            && (selection.preview_slot.is_some()
                || selection.ledge_held(buttons)
                || selection.autopilot_held(buttons));
        self.duck_held = gameplay && selection.auto_duck_held(buttons);
        if self.resolved_choices != Some(selection.choices)
            || self.resolved_recipe_revision != Some(selection.recipe_revision)
            || self.resolved_preview != selection.preview_slot
        {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::Resolve);
            let resolved = selection.settings(&profile.menu_options, profile.apply(*base));
            self.resolved = Some(resolved);
            self.resolved_choices = Some(selection.choices);
            self.resolved_recipe_revision = Some(selection.recipe_revision);
            self.resolved_preview = selection.preview_slot;
        }
        // Editing requires an available local Training menu, not permission to
        // inject gameplay inputs. Respawn/paused observations can temporarily
        // clear the actionable fighter while the editor remains available.
        if selection.tuning_save_requested {
            selection.save_status = Some(save::request(
                save::Job {
                    kind,
                    slot: kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral,
                    index: 0,
                    revision: selection.recipe_revision,
                    choice: String::new(),
                    name: None,
                    delete: false,
                    make_default: false,
                    allow_new: false,
                    sequence: kirby_assist_core::sequence_config::Sequence::default(),
                    tuning: Some(selection.tuning),
                },
                training,
            ));
            selection.revision = selection.revision.wrapping_add(1);
        }
        if selection.save_requested || selection.delete_requested {
            if let Some(slot) = selection.row.slot() {
                let index = selection.choices[slot as usize];
                let message = if let Some(choice) =
                    selection.choice(&profile.menu_options, slot, index)
                {
                    let snapshot = if selection.delete_requested {
                        Some(choice.sequence)
                    } else {
                        selection.recipe_snapshot(&profile.menu_options, slot, index)
                    };
                    if let Some(sequence) = snapshot {
                        save::request(
                            save::Job {
                                kind,
                                slot,
                                index,
                                revision: selection.recipe_revision,
                                choice: choice.id.to_owned(),
                                allow_new: choice.allow_new,
                                name: selection
                                    .name_override(slot, index)
                                    .map(str::to_owned)
                                    .or_else(|| choice.sequence.mode.map(|m| m.label().to_owned()))
                                    .or_else(|| {
                                        choice.sequence.builtin.map(|b| b.label().to_owned())
                                    }),
                                delete: selection.delete_requested,
                                make_default: selection.default_requested,
                                sequence,
                                tuning: None,
                            },
                            training,
                        )
                    } else {
                        "Save failed: draft exceeds the action limit; remove an action or delay"
                            .into()
                    }
                } else {
                    "Save failed: no configured recipe".into()
                };
                selection.save_status = Some(message);
                selection.revision = selection.revision.wrapping_add(1);
            }
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            let stamp = (kind, selection.revision, *auto_mash);
            if self.diagnostic_stamp != Some(stamp) || self.diagnostic_config.is_none() {
                use kirby_assist_core::diagnostics::modes::Configuration;
                let on_off = |on| if on { "ON" } else { "OFF" };
                let mut rows = ["unavailable for this fighter"; 25];
                for (i, slot) in kirby_assist_core::fighter_profiles::presets::PresetSlot::ALL
                    .into_iter()
                    .take(5)
                    .enumerate()
                {
                    rows[i] = selection
                        .choice_labels(&profile.menu_options, slot, selection.choices[i])
                        .map_or("unknown selection", |(_, name)| name);
                }
                if matches!(
                    kind,
                    kirby_assist_core::fighter_kinds::YOSHI
                        | kirby_assist_core::fighter_kinds::KIRBY
                ) {
                    rows[6] = "Removed (use Y sequence)";
                }
                rows[7] = on_off(*auto_mash);
                rows[9] = on_off(selection.tilt_turn);
                if kind == kirby_assist_core::fighter_kinds::KIRBY {
                    rows[8] = on_off(selection.idle_crouch);
                }
                if kind == kirby_assist_core::fighter_kinds::YOSHI {
                    rows[10] = on_off(selection.jab_lock);
                }
                rows[11] = on_off(selection.spot_punish);
                rows[12] = on_off(selection.grab_dodge);
                rows[13] = on_off(selection.auto_tech);
                rows[15] = on_off(selection.performance);
                rows[23] =
                    on_off(selection.effective_mode(Mode::OutOfShield, selection.out_of_shield));
                rows[24] = on_off(selection.shield_pressure);
                for (row, slot) in [
                    (
                        16,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::X,
                    ),
                    (
                        17,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::Y,
                    ),
                ] {
                    rows[row] = selection
                        .choice_labels(
                            &profile.menu_options,
                            slot,
                            selection.choices[slot as usize],
                        )
                        .map_or("Normal button / custom draft", |(_, name)| name);
                }
                for (row, slot) in [
                    (
                        18,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadUp,
                    ),
                    (
                        19,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadDown,
                    ),
                    (
                        20,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadLeft,
                    ),
                    (
                        21,
                        kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadRight,
                    ),
                ] {
                    rows[row] = selection
                        .choice_labels(
                            &profile.menu_options,
                            slot,
                            selection.choices[slot as usize],
                        )
                        .map_or("Normal / custom / mode", |(_, name)| name);
                }
                rows[22] = on_off(selection.ledge_mode());
                let report =
                    Configuration::new(profile.name, rows).with_reflect(selection.auto_reflect);
                if self.diagnostic_config != Some(report) {
                    self.diagnostic_config = Some(report);
                    crate::logging::write_configuration(report);
                }
                self.diagnostic_stamp = Some(stamp);
            }
        }
        #[cfg(feature = "imgui-menu")]
        if self.view_revision != Some(selection.revision)
            && menu::publish(profile, selection, *auto_mash)
        {
            self.view_revision = Some(selection.revision);
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            let stamp = (kind, selection.revision);
            if self.retained_stamp != Some(stamp) {
                let (mut recipes, mut drafts, mut bytes) = (0, 0, 0);
                for s in self.selections.values() {
                    let (r, d, b) = s.diagnostic_retained();
                    recipes += r;
                    drafts += d;
                    bytes += b + std::mem::size_of::<i32>();
                }
                bytes += self.rule_cache.as_ref().map_or(0, |r| {
                    std::mem::size_of_val(&**r)
                        + r.rules.capacity()
                            * std::mem::size_of::<
                                Option<kirby_assist_core::reaction_rules::Candidate>,
                            >()
                });
                crate::performance::health::retained(self.selections.len(), recipes, drafts, bytes);
                self.retained_stamp = Some(stamp);
            }
        }
        changed
    }
    pub fn settings(&self, kind: Option<i32>, startup: &AssistSettings) -> AssistSettings {
        if self.scope && self.kind == kind {
            if let Some(resolved) = self.resolved.as_ref() {
                return AssistSettings {
                    enabled: self.sequence_enabled(startup.enabled),
                    ..*resolved
                };
            }
            let base = super::settings_for_fighter(kind, *startup);
            return AssistSettings {
                enabled: self.sequence_enabled(base.enabled),
                ..base
            };
        }
        super::settings_for_fighter(kind, *startup)
    }
}
pub(super) fn install() {
    #[cfg(feature = "imgui-menu")]
    menu::install();
}

#[cfg(test)]
#[path = "../../tests/unit/live/settings_menu/save_completion_tests.rs"]
mod save_completion_tests;
