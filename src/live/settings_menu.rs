//! Cached per-fighter menu selections and settings. No playback, target
//! acquisition, or combo reads; resolve response metadata only on edits.
//! Technical: Manages cached per-fighter menu state, edit handling, and publication
//! of the selected runtime settings.
//! Plain English: This module keeps the in-game settings menu and current choices
//! up to date.
use super::PROFILES;
use kirby_assist_core::button_bindings::Mode;
use kirby_assist_core::fighter_profiles::{
    presets::{MenuPresets, PresetSlot},
    FighterProfile, FighterProfiles,
};
use kirby_assist_core::{settings::AssistSettings, settings_menu::MenuSelection};
use std::collections::BTreeMap;

#[cfg(feature = "imgui-menu")]
#[path = "settings_menu/menu.rs"]
/// In-game ImGui adapter for observing menu input and publishing views.
mod menu;
#[path = "settings_menu/save.rs"]
/// Serialized background worker and persistence implementation.
mod save;

// Host adapters and worker tests share the same process-wide save globals.
// Serialize simulated mapper polling with worker tests just as STATE does in game.
#[cfg(test)]
/// Serializes host-side tests that share save-worker globals.
pub(super) fn save_test_guard() -> std::sync::MutexGuard<'static, ()> {
    save::WORKER_TEST.lock().unwrap()
}

/// Performs startup recovery of an interrupted saved-file update.
pub(super) fn recover_saved_file() -> Result<(), String> {
    save::recover()
}

/// Updates menu presence from an existing local-fighter callback timestamp.
pub(super) fn observe_local_fighter(now: std::time::Instant) {
    #[cfg(feature = "imgui-menu")]
    menu::observe_local_fighter(now);
    #[cfg(not(feature = "imgui-menu"))]
    let _ = now;
}

/// Requests a transient display of the runtime enabled state.
pub(super) fn show_master_status(enabled: bool) {
    #[cfg(feature = "imgui-menu")]
    menu::show_master_status(enabled);
    #[cfg(not(feature = "imgui-menu"))]
    let _ = enabled;
}

// Loss of a local fighter callback (death/respawn) is not proof the match
// ended. Release retained session data only after the existing master match
// guard unlocks; the renderer handles that one-shot cleanup without new queries.
/// Defers battle-session cleanup until the master match guard is released.
static CLEANUP_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Reads the deferred cleanup request with acquire ordering.
pub(super) fn cleanup_pending() -> bool {
    CLEANUP_PENDING.load(std::sync::atomic::Ordering::Acquire)
}

// Run only from the existing mapper callback, before it acquires STATE.
// Applying a save result can allocate menu drafts; cleanup frees editor data.
// Neither belongs inside the shared ImGui/GPU command-submission callback.
/// Applies completed save results and eligible cleanup outside shared callbacks.
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

/// Settings last resolved for the active fighter. They are rebuilt only when
/// the choices, recipe revision or preview slot that produced them change.
#[derive(Default)]
struct ResolvedSettings {
    /// Last settings value materialized from menu choices.
    settings: Option<AssistSettings>,
    /// Choice array used to produce `settings`.
    choices: Option<[usize; PresetSlot::COUNT]>,
    /// Recipe revision used to produce `settings`.
    recipe_revision: Option<u64>,
    /// Active preview slot used to produce `settings`.
    preview: Option<PresetSlot>,
}
impl ResolvedSettings {
    /// Checks all cache-key components against the current selection.
    fn is_current(&self, selection: &MenuSelection) -> bool {
        self.choices == Some(selection.choices)
            && self.recipe_revision == Some(selection.recipe_revision)
            && self.preview == selection.preview_slot
    }
    /// Recomputes effective settings only when the selection cache key changes.
    fn refresh(
        &mut self,
        selection: &MenuSelection,
        profile: &FighterProfile,
        base: AssistSettings,
    ) {
        if self.is_current(selection) {
            return;
        }
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::health::event(crate::performance::health::Event::Resolve);
        self.settings = Some(selection.settings(&profile.menu_options, profile.apply(base)));
        self.choices = Some(selection.choices);
        self.recipe_revision = Some(selection.recipe_revision);
        self.preview = selection.preview_slot;
    }
}

/// Automatic reaction rules resolved for one fighter's recipe revision.
#[derive(Default)]
struct RuleCache {
    /// Fighter kind and recipe revision used for the cached rule set.
    stamp: Option<(i32, u64)>,
    /// Owned resolved rules for the current cache stamp.
    rules: Option<Box<kirby_assist_core::reaction_rules::Resolved>>,
}
impl RuleCache {
    /// Returns rules only when both fighter and revision match the cache stamp.
    fn get(
        &self,
        kind: i32,
        revision: u64,
    ) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        (self.stamp == Some((kind, revision))).then_some(())?;
        self.rules.as_deref()
    }
    /// Drops the cache and stamp together.
    fn clear(&mut self) {
        *self = Self::default();
    }
    /// Resolves reaction recipes on cache misses and applies enabled controls.
    fn refresh(&mut self, kind: i32, selection: &mut MenuSelection, options: &MenuPresets) {
        if self.stamp != Some((kind, selection.recipe_revision)) {
            self.stamp = Some((kind, selection.recipe_revision));
            let resolved = if selection.has_reactions {
                selection.resolve_reactions(options)
            } else {
                Ok(Default::default())
            };
            self.rules = match resolved {
                Ok(r) if r.len > 0 => Some(Box::new(r)),
                Ok(_) => None,
                Err(e) => {
                    selection.save_status = Some(e);
                    bump_revision(selection);
                    None
                }
            };
        }
        if let Some(rules) = self.rules.as_mut() {
            rules.set_enabled_controls(selection.reaction_enabled_controls());
        }
    }
}

/// Advances the general selection revision with wrapping arithmetic.
fn bump_revision(selection: &mut MenuSelection) {
    selection.revision = selection.revision.wrapping_add(1);
}
/// Advances both general and visible-content revisions.
fn bump_content(selection: &mut MenuSelection) {
    bump_revision(selection);
    selection.content_revision = selection.content_revision.wrapping_add(1);
}
/// Clears an active preview and advances the revision when one existed.
fn end_preview(selection: &mut MenuSelection) {
    if selection.preview_slot.take().is_some() {
        bump_revision(selection);
    }
}

#[cfg(feature = "diagnostic-logging")]
#[derive(Default)]
/// Caches diagnostic configuration and retained-memory accounting revisions.
struct DiagnosticState {
    /// Last resolved diagnostic configuration.
    config: Option<kirby_assist_core::diagnostics::modes::Configuration>,
    /// Current configuration cache key, including enablement.
    stamp: Option<(i32, u64, bool)>,
    /// Last retained-memory accounting cache key.
    retained_stamp: Option<(i32, u64)>,
}

#[derive(Default)]
/// Per-fighter menu selections and derived runtime caches.
pub(super) struct SettingsMenu {
    /// Selections retained by native fighter kind.
    selections: BTreeMap<i32, MenuSelection>,
    /// Native fighter kind currently selected by the menu.
    kind: Option<i32>,
    /// Buttons consumed by the menu on the latest input sample.
    pub consumed: u32,
    /// Native button mask selected for forwarding.
    pub native_buttons: u32,
    /// Built-in menu consumption mask from the latest selection input.
    pub builtin_consumed: u32,
    /// Whether the ledge-assist hold is active.
    pub ledge_held: bool,
    /// Whether the current selection is in gameplay scope.
    scope: bool,
    /// Cache of effective settings for the current selection.
    resolved: ResolvedSettings,
    /// Revision of the last successfully published view.
    view_revision: Option<u64>,
    /// Cache for resolved automatic reaction recipes.
    rules: RuleCache,
    /// Effective auto-mash value retained across selection resets.
    auto_mash: Option<bool>,
    // Retain a real release through first profile selection/session reset.
    /// Physical button/layout snapshot replayed after a fresh selection is initialized.
    passthrough_buttons: Option<(u32, kirby_assist_core::native_input::ControlLayout)>,
    /// Defers the request to open the quick menu until selection input.
    open_requested: bool,
    /// Defers restoration of saved training state until gameplay resumes.
    resume_saved_state: bool,
    #[cfg(feature = "diagnostic-logging")]
    /// Deduplicates diagnostic configuration and retained-memory snapshots.
    diagnostics: DiagnosticState,
}
impl SettingsMenu {
    /// Returns the selection for the currently selected fighter.
    fn current(&self) -> Option<&MenuSelection> {
        self.kind.and_then(|kind| self.selections.get(&kind))
    }
    /// Returns the mutable selection for the currently selected fighter.
    fn current_mut(&mut self) -> Option<&mut MenuSelection> {
        let kind = self.kind?;
        self.selections.get_mut(&kind)
    }
    /// The active selection while its fighter is in gameplay scope.
    fn scoped(&self) -> Option<&MenuSelection> {
        self.current().filter(|_| self.scope)
    }
    /// Scoped selection that is not paused on an editor/tuning/manage page.
    fn live(&self) -> Option<&MenuSelection> {
        self.scoped()
            .filter(|s| !s.recipe_editing && !s.tuning_page && !s.managing())
    }
    /// Native fighter changes can precede the next mapper/menu poll. Never
    /// expose a previous fighter's automatic rules or ranges in that gap.
    pub fn for_fighter(&self, kind: i32) -> Option<&Self> {
        (self.kind == Some(kind)).then_some(self)
    }
    #[cfg(feature = "diagnostic-logging")]
    /// Finds a cached display label for a saved menu choice.
    pub fn selected_choice_labels(&self, kind: i32, slot: PresetSlot) -> Option<(&str, &str)> {
        let selection = self.for_fighter(kind)?.current()?;
        let profile = PROFILES.get()?.profile(kind)?;
        selection.choice_labels(
            &profile.menu_options,
            slot,
            selection.choices[slot as usize],
        )
    }
    /// Returns the active preview slot from the current selection.
    pub fn preview_slot(&self) -> Option<PresetSlot> {
        self.current().and_then(|s| s.preview_slot)
    }
    /// Polls the save worker and reconciles a completed result.
    pub(super) fn poll_save_result(&mut self) -> bool {
        save::poll().is_some_and(|result| self.apply_save_result(result, PROFILES.get()))
    }
    /// Reconcile a finished save with the selection that requested it. Returns
    /// true when a delete changed the active fighter's saved-file list.
    fn apply_save_result(
        &mut self,
        result: save::ResultMessage,
        profiles: Option<&FighterProfiles>,
    ) -> bool {
        #[cfg(feature = "diagnostic-logging")]
        {
            self.diagnostics.retained_stamp = None;
        }
        let result_kind = result.kind;
        let Some(saved) = self.selections.get_mut(&result_kind) else {
            return false;
        };
        if let Some(tuning) = result.tuning {
            saved.save_status = Some(if result.success && saved.tuning != tuning {
                "Saved earlier fighter settings; newer edits are unsaved".into()
            } else {
                result.message
            });
            bump_content(saved);
            return false;
        }
        let profile = profiles.and_then(|p| p.profile(result_kind));
        // A completed save can arrive after a new match/profile reload.
        // Numeric draft indices are not identities: never mutate a new
        // template or different recipe that now occupies the old index.
        let same_recipe = profile
            .and_then(|p| saved.choice(&p.menu_options, result.slot, result.index))
            .is_some_and(|c| result.choice.as_deref() == Some(c.id));
        if result.success && !same_recipe {
            saved.save_status = Some(result.message);
            bump_content(saved);
            return false;
        }
        let file_list_changed = result.success && result.deleted && Some(result_kind) == self.kind;
        if result.success && result.deleted {
            saved.cancel_actions();
        }
        let snapshot = result
            .snapshot
            .as_ref()
            .map(|(sequence, name)| (*sequence, name.as_deref()));
        let snapshot_matches = match snapshot {
            None => saved.recipe_revision == result.revision,
            Some((sequence, name)) => profile.is_some_and(|profile| {
                saved.saved_recipe_matches(
                    &profile.menu_options,
                    result.slot,
                    result.index,
                    sequence,
                    name,
                )
            }),
        };
        if result.success && !result.deleted && snapshot_matches {
            saved.mark_recipe_saved(result.slot, result.index);
        }
        if let Some(profile) = profile.filter(|_| result.success) {
            saved.complete_saved_snapshot(
                &profile.menu_options,
                result.slot,
                result.index,
                result.deleted,
                snapshot.map(|(sequence, _)| sequence),
                snapshot.and_then(|(_, name)| name),
            );
            if saved.lightweight {
                saved.release_training_tools(&profile.menu_options);
            }
            if self.kind == Some(result_kind) {
                self.resolved.recipe_revision = None;
                self.rules.stamp = None;
                self.view_revision = None;
            }
        }
        let current = saved.row.slot() == Some(result.slot)
            && saved.choices[result.slot as usize] == result.index;
        saved.save_status = Some(if !current {
            format!("{:?}: {}", result.slot, result.message)
        } else if !snapshot_matches
            && (result.message.starts_with("Saved") || result.message.starts_with("Already saved"))
        {
            "Saved earlier snapshot; newer edits are unsaved".into()
        } else {
            result.message
        });
        bump_content(saved);
        file_list_changed
    }
    /// Combines the base enablement with current ledge/sequence holds.
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.ledge_held || self.scoped().is_some_and(|s| s.sequence_enabled(false))
    }
    /// Resolves manual native actions for the scoped fighter selection.
    pub fn manual_native_actions(&self) -> u32 {
        let Some((kind, selection)) = self.kind.zip(self.scoped()) else {
            return 0;
        };
        PROFILES
            .get()
            .and_then(|p| p.profile(kind))
            .map_or(0, |profile| {
                selection.manual_native_actions(&profile.menu_options)
            })
    }
    /// Queries whether the scoped selection reserves automatic modes.
    pub fn reserves_automatic_modes(&self) -> bool {
        self.scoped().is_some_and(|s| s.reserves_automatic_modes())
    }
    /// Ends the current selection's preview and updates its revision.
    pub fn finish_preview(&mut self) {
        if let Some(selection) = self.current_mut() {
            end_preview(selection);
        }
    }
    /// Reads ledge-trump hold state only from the scoped selection.
    pub fn ledge_trump_held(&self) -> bool {
        self.scoped().is_some_and(MenuSelection::ledge_trump_held)
    }
    /// Returns the effective Auto Tech mode with startup fallback.
    pub fn auto_tech(&self, startup_default: bool) -> bool {
        self.current().map_or(startup_default, |s| {
            s.effective_mode(Mode::AutoTech, s.auto_tech)
        })
    }
    /// Reads grab-dodge enablement from the scoped selection.
    pub fn grab_dodge_enabled(&self) -> bool {
        self.scoped().is_some_and(|s| s.grab_dodge)
    }
    #[cfg(feature = "diagnostic-logging")]
    /// Returns the current Performance diagnostic option.
    pub fn diagnostic_options(&self) -> bool {
        self.current().is_some_and(|s| s.performance)
    }
    /// Reads out-of-shield assistance from the scoped selection.
    pub fn out_of_shield_enabled(&self) -> bool {
        self.scoped().is_some_and(|s| s.out_of_shield)
    }
    /// Defers training ledge-hold restoration until gameplay input.
    pub fn resume_saved_state(&mut self) {
        self.resume_saved_state = true;
    }
    /// Returns effective idle-crouch and tilt-turn modes for the current fighter.
    pub fn posture_modes(&self) -> (bool, bool) {
        let Some((kind, s)) = self.kind.zip(self.scoped()) else {
            return (false, false);
        };
        (
            kind == kirby_assist_core::fighter_kinds::KIRBY
                && s.effective_mode(Mode::IdleCrouch, s.idle_crouch),
            s.effective_mode(Mode::TiltTurn, s.tilt_turn) && s.tuning.tilt_turn(kind).any(),
        )
    }
    /// Schedules the quick menu to open on the next input update.
    pub fn request_open(&mut self) {
        self.open_requested = true;
    }
    /// Returns stored tuning for `kind` only when it is the active fighter.
    pub fn fighter_tuning(&self, kind: i32) -> kirby_assist_core::fighter_tuning::Tuning {
        self.for_fighter(kind)
            .and_then(Self::current)
            .map_or(Default::default(), |s| s.tuning)
    }
    /// Builds a fighter-specific reflect configuration for a live selection.
    pub fn reflect_config(
        &self,
        kind: i32,
    ) -> Option<kirby_assist_core::projectile_reflect::Config> {
        let selection = self.for_fighter(kind)?.live()?;
        if !selection.auto_reflect {
            return None;
        }
        kirby_assist_core::projectile_reflect::Config::for_fighter(kind, selection.tuning)
    }
    /// Returns configured dodge lookahead or the core default.
    pub fn dodge_lookahead(&self) -> f32 {
        self.current()
            .map_or(kirby_assist_core::spot_dodge::LOOKAHEAD, |s| {
                s.tuning.dodge_lookahead()
            })
    }
    /// Returns resolved rules only when they contain enabled menu controls.
    pub fn rule_config(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules().filter(|r| {
            r.has_simple_events()
                && r.enabled_controls & kirby_assist_core::reaction_rules::Control::Menu.bit() != 0
        })
    }
    /// Selects cached rules matching the current scoped recipe revision.
    fn current_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        let revision = self.scoped()?.recipe_revision;
        self.rules.get(self.kind?, revision)
    }
    /// Grab avoidance owns its follow-up toggle independently of Auto react.
    /// Only its own event recipe may punish a confirmed grab dodge.
    pub fn grab_dodge_sequence(&self) -> Option<&kirby_assist_core::sequence_config::Sequence> {
        if !self.live()?.grab_dodge {
            return None;
        }
        self.current_rules()?.simple_sequence(5)
    }
    /// All Auto react recipes share a revision cache, independent of the
    /// individual mode switches that continue to control their admission.
    /// Builds the automatic-action view over the cached resolved recipes.
    pub fn automatic_sequences(&self) -> kirby_assist_core::out_of_shield::AutomaticConfig<'_> {
        let rules = self.current_rules();
        kirby_assist_core::out_of_shield::AutomaticConfig {
            grab: rules.and_then(|r| r.simple_sequence(3)),
            shield_grab: rules.and_then(|r| r.simple_sequence(4)),
            backup: rules.and_then(|r| r.simple_sequence(6)),
        }
    }
    /// Returns configured time/distance limits for the three automatic recipes.
    pub fn automatic_limits(&self) -> [Option<(f32, f32)>; 3] {
        let r = self.automatic_sequences();
        [r.grab, r.shield_grab, r.backup].map(|s| s.map(|s| (s.reaction.max, s.reaction.height)))
    }
    /// Returns cached resolved rules when at least one mode is enabled.
    pub fn automatic_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules()
    }
    /// Records physical passthrough input for the selected fighter's binding state.
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
    /// Returns retained auto-mash state or the startup default.
    pub fn auto_mash(&self, startup_default: bool) -> bool {
        let base = self.auto_mash.unwrap_or(startup_default);
        self.current()
            .map_or(base, |s| s.effective_mode(Mode::AutoMash, base))
    }
    /// Drops current scope, pending actions, and derived caches without deleting profiles.
    pub fn invalidate(&mut self) {
        self.ledge_held = false;
        self.scope = false;
        if let Some(selection) = self.current_mut() {
            end_preview(selection);
            selection.cancel_actions();
        }
        // Saved metadata remains valid through damage, death and respawn.
    }
    /// Enters match scope and initializes state for the observed session.
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
    /// Ends match scope and conditionally releases retained session data.
    pub fn end_match(&mut self) {
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::clear();
        self.resume_saved_state = false;
        self.open_requested = false;
        self.invalidate();
        self.consumed = 0;
        if let Some(selection) = self.current_mut() {
            selection.reset_inputs();
        }
        self.kind = None;
        self.rules.clear();
        let outside_match = !super::MASTER_IN_MATCH.load(std::sync::atomic::Ordering::Acquire);
        CLEANUP_PENDING.store(!outside_match, std::sync::atomic::Ordering::Release);
        if outside_match {
            self.selections.clear();
            self.auto_mash = None;
            self.passthrough_buttons = None;
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            self.diagnostics = DiagnosticState::default();
            if outside_match {
                crate::performance::health::retained(0, 0, 0, 0);
            }
        }
        self.resolved = ResolvedSettings::default();
        self.view_revision = None;
        #[cfg(feature = "imgui-menu")]
        menu::clear();
    }
    /// Updates ImGui battle visibility without affecting menu state.
    pub fn display_battle(&self, active: bool) {
        #[cfg(feature = "imgui-menu")]
        menu::set_battle_active(active);
        #[cfg(not(feature = "imgui-menu"))]
        let _ = active;
    }
    /// True releases old proposals when navigating or changing a tool.
    /// Routes one physical-input sample, refreshes derived caches, and publishes changed views.
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
        #[cfg(feature = "diagnostic-logging")]
        let _timing =
            crate::performance::health::Trace::new(crate::performance::health::Lane::MenuInput);
        let profile = kind.and_then(|kind| PROFILES.get()?.profile(kind));
        let (Some(kind), Some(profile)) = (kind, profile.filter(|_| available)) else {
            self.invalidate();
            self.consumed = 0;
            return false;
        };
        if self.kind != Some(kind) {
            self.switch_fighter(kind);
        }
        self.selections
            .entry(kind)
            .or_insert_with(|| new_selection(profile, kind, base, grab_default, tech_default));
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
        let options = &profile.menu_options;
        self.consumed = route_input(
            selection,
            options,
            bindings_blocked,
            safety_jump,
            buttons,
            physical,
            layout,
            training,
        );
        self.native_buttons = selection.native_buttons(buttons);
        self.builtin_consumed = selection.builtin_consumed;
        self.rules.refresh(kind, selection, options);
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
        self.resolved.refresh(selection, profile, *base);
        request_saves(selection, kind, options, training);
        #[cfg(feature = "diagnostic-logging")]
        self.diagnostics
            .log_configuration(kind, profile, selection, *auto_mash);
        #[cfg(feature = "imgui-menu")]
        if self.view_revision != Some(selection.revision)
            && menu::publish(profile, selection, *auto_mash)
        {
            self.view_revision = Some(selection.revision);
        }
        #[cfg(feature = "diagnostic-logging")]
        self.diagnostics.record_retained(
            (kind, selection.revision),
            &self.selections,
            self.rules.rules.as_deref(),
        );
        changed
    }
    /// A different native fighter replaces every cached per-fighter view.
    /// Invalidates cross-fighter state and retains only the newly active fighter entry.
    fn switch_fighter(&mut self, kind: i32) {
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::health::event(crate::performance::health::Event::FighterChange);
        #[cfg(feature = "diagnostic-logging")]
        crate::calibration::clear();
        self.invalidate();
        self.selections.retain(|saved_kind, _| *saved_kind == kind);
        self.kind = Some(kind);
        self.rules.clear();
        self.resolved = ResolvedSettings::default();
        self.view_revision = None;
        if let Some(selection) = self.selections.get_mut(&kind) {
            selection.reset_inputs();
        }
    }
    /// Returns scoped resolved settings when available, otherwise applies startup settings.
    pub fn settings(&self, kind: Option<i32>, startup: &AssistSettings) -> AssistSettings {
        if self.scope && self.kind == kind {
            if let Some(resolved) = self.resolved.settings.as_ref() {
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
/// Creates a menu selection initialized from saved recipes and tuning.
fn new_selection(
    profile: &FighterProfile,
    kind: i32,
    base: &AssistSettings,
    grab_default: bool,
    tech_default: bool,
) -> MenuSelection {
    let mut s = MenuSelection::new(&profile.menu_options);
    s.simplified_reactions = true;
    if let Some(saved) = super::SEQUENCES.get() {
        let library = saved.lock().unwrap_or_else(|e| e.into_inner());
        let catalog = library.for_fighter(kind);
        s.load_saved_recipes(&profile.menu_options, &catalog, kind);
        s.tuning = library.tuning.get(&kind).copied().unwrap_or_default();
    }
    s.tuning_values = kirby_assist_core::fighter_tuning::Tuning::values(profile.apply(*base));
    s.tuning_values[11] = if grab_default { 1.0 } else { 0.0 };
    s.grab_dodge = s.tuning.grab_dodge_default(grab_default);
    s.out_of_shield = s.tuning.oos_default();
    s.auto_reflect = s.tuning.0[kirby_assist_core::fighter_tuning::REFLECT_DEFAULT] == Some(1.);
    s.auto_tech = tech_default;
    s
}

/// Feed one frame of physical input to the selection and refresh its native
/// bindings. Returns the buttons the menu consumed.
/// Applies menu input while respecting training-save and binding gates.
fn route_input(
    selection: &mut MenuSelection,
    options: &MenuPresets,
    bindings_blocked: bool,
    safety_jump: bool,
    buttons: u32,
    physical: u32,
    layout: kirby_assist_core::native_input::ControlLayout,
    training: bool,
) -> u32 {
    // Scene permission was copied before the caller acquired STATE. Do
    // not call native game functions from this locked menu adapter.
    selection.block_binding_starts(bindings_blocked);
    selection.preserve_safety_jump(safety_jump);
    selection.set_binding_input(physical, layout);
    let choices_before = (selection.choices, selection.recipe_revision);
    let consumed = if selection.waits_for_training_save(training, save::busy()) {
        selection.wait_for_save(buttons)
    } else {
        selection.input_with_editing(buttons, options, training)
    };
    // capture_binding_input already refreshed these once. Repeat only when
    // this menu input actually changed a choice or its recipe metadata.
    if choices_before != (selection.choices, selection.recipe_revision) {
        selection.refresh_native_bindings(options);
    }
    consumed
}

// Editing requires an available local Training menu, not permission to
// inject gameplay inputs. Respawn/paused observations can temporarily
// clear the actionable fighter while the editor remains available.
/// Converts pending tuning or recipe edits into explicit save-worker requests.
fn request_saves(selection: &mut MenuSelection, kind: i32, options: &MenuPresets, training: bool) {
    if selection.tuning_save_requested {
        let job = save::Job {
            kind,
            slot: PresetSlot::Neutral,
            index: 0,
            revision: selection.recipe_revision,
            choice: String::new(),
            name: None,
            delete: false,
            make_default: false,
            allow_new: false,
            sequence: kirby_assist_core::sequence_config::Sequence::default(),
            tuning: Some(selection.tuning),
        };
        selection.save_status = Some(save::request(job, training));
        bump_revision(selection);
    }
    if !(selection.save_requested || selection.delete_requested) {
        return;
    }
    if let Some(slot) = selection.row.slot() {
        let message = match recipe_job(selection, kind, slot, options) {
            Ok(job) => save::request(job, training),
            Err(message) => message.into(),
        };
        selection.save_status = Some(message);
        bump_revision(selection);
    }
}
/// Captures the selected recipe or delete operation as an immutable save job.
fn recipe_job(
    selection: &MenuSelection,
    kind: i32,
    slot: PresetSlot,
    options: &MenuPresets,
) -> Result<save::Job, &'static str> {
    let index = selection.choices[slot as usize];
    let choice = selection
        .choice(options, slot, index)
        .ok_or("Save failed: no configured recipe")?;
    let snapshot = if selection.delete_requested {
        Some(choice.sequence)
    } else {
        selection.recipe_snapshot(options, slot, index)
    };
    let sequence =
        snapshot.ok_or("Save failed: draft exceeds the action limit; remove an action or delay")?;
    Ok(save::Job {
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
            .or_else(|| choice.sequence.builtin.map(|b| b.label().to_owned())),
        delete: selection.delete_requested,
        make_default: selection.default_requested,
        sequence,
        tuning: None,
    })
}

/// Row positions in core's diagnostic `Configuration` table. Row 10 is a
/// removed slot kept only so later rows stay aligned.
#[cfg(feature = "diagnostic-logging")]
/// Stable indices that align local menu selections with diagnostic configuration rows.
mod config_rows {
    use super::PresetSlot;
    /// Number of rows in the core diagnostic configuration table.
    pub const COUNT: usize = 25;
    /// Diagnostic row index for the Y binding.
    pub const Y_BINDING: usize = 6;
    /// Diagnostic row index for Auto Mash.
    pub const AUTO_MASH: usize = 7;
    /// Diagnostic row index for Idle Crouch.
    pub const IDLE_CROUCH: usize = 8;
    /// Diagnostic row index for Tilt Turn.
    pub const TILT_TURN: usize = 9;
    /// Diagnostic row index for Spot Punish.
    pub const SPOT_PUNISH: usize = 11;
    /// Diagnostic row index for Grab Dodge.
    pub const GRAB_DODGE: usize = 12;
    /// Diagnostic row index for Auto Tech.
    pub const AUTO_TECH: usize = 13;
    /// Diagnostic row index for Performance.
    pub const PERFORMANCE: usize = 15;
    /// Diagnostic row index for Ledge Mode.
    pub const LEDGE_MODE: usize = 22;
    /// Diagnostic row index for Out of Shield.
    pub const OUT_OF_SHIELD: usize = 23;
    /// Diagnostic row index for Shield Pressure.
    pub const SHIELD_PRESSURE: usize = 24;
    /// Rows that show a slot's selected choice, with the text used when it has none.
    pub const CHOICES: [(usize, PresetSlot, &str); 11] = [
        (0, PresetSlot::Neutral, "unknown selection"),
        (1, PresetSlot::Jump, "unknown selection"),
        (2, PresetSlot::Roll, "unknown selection"),
        (3, PresetSlot::Attack, "unknown selection"),
        (4, PresetSlot::Drop, "unknown selection"),
        (16, PresetSlot::X, "Normal button / custom draft"),
        (17, PresetSlot::Y, "Normal button / custom draft"),
        (18, PresetSlot::DpadUp, "Normal / custom / mode"),
        (19, PresetSlot::DpadDown, "Normal / custom / mode"),
        (20, PresetSlot::DpadLeft, "Normal / custom / mode"),
        (21, PresetSlot::DpadRight, "Normal / custom / mode"),
    ];
}

#[cfg(feature = "diagnostic-logging")]
/// Produces the stable row-aligned diagnostic representation for a selection.
fn configuration(
    kind: i32,
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
) -> kirby_assist_core::diagnostics::modes::Configuration {
    use config_rows as row;
    use kirby_assist_core::fighter_kinds::{KIRBY, YOSHI};
    let on_off = |on| if on { "ON" } else { "OFF" };
    let mut rows = ["unavailable for this fighter"; row::COUNT];
    for (index, slot, fallback) in row::CHOICES {
        rows[index] = selection
            .choice_labels(
                &profile.menu_options,
                slot,
                selection.choices[slot as usize],
            )
            .map_or(fallback, |(_, name)| name);
    }
    if matches!(kind, YOSHI | KIRBY) {
        rows[row::Y_BINDING] = "Removed (use Y sequence)";
    }
    if kind == KIRBY {
        rows[row::IDLE_CROUCH] = on_off(selection.idle_crouch);
    }
    rows[row::AUTO_MASH] = on_off(auto_mash);
    rows[row::TILT_TURN] = on_off(selection.tilt_turn);
    rows[row::SPOT_PUNISH] = on_off(selection.spot_punish);
    rows[row::GRAB_DODGE] = on_off(selection.grab_dodge);
    rows[row::AUTO_TECH] = on_off(selection.auto_tech);
    rows[row::PERFORMANCE] = on_off(selection.performance);
    rows[row::LEDGE_MODE] = on_off(selection.ledge_mode());
    rows[row::OUT_OF_SHIELD] =
        on_off(selection.effective_mode(Mode::OutOfShield, selection.out_of_shield));
    rows[row::SHIELD_PRESSURE] = on_off(selection.shield_pressure);
    kirby_assist_core::diagnostics::modes::Configuration::new(profile.name, rows)
        .with_reflect(selection.auto_reflect)
}

#[cfg(feature = "diagnostic-logging")]
impl DiagnosticState {
    /// Log the visible configuration once per change, never per frame.
    fn log_configuration(
        &mut self,
        kind: i32,
        profile: &FighterProfile,
        selection: &MenuSelection,
        auto_mash: bool,
    ) {
        let stamp = (kind, selection.revision, auto_mash);
        if self.stamp == Some(stamp) && self.config.is_some() {
            return;
        }
        let report = configuration(kind, profile, selection, auto_mash);
        if self.config != Some(report) {
            self.config = Some(report);
            crate::logging::write_configuration(report);
        }
        self.stamp = Some(stamp);
    }
    /// Recomputes estimated retained payload only when its cache stamp changes.
    fn record_retained(
        &mut self,
        stamp: (i32, u64),
        selections: &BTreeMap<i32, MenuSelection>,
        rules: Option<&kirby_assist_core::reaction_rules::Resolved>,
    ) {
        if self.retained_stamp == Some(stamp) {
            return;
        }
        let (mut recipes, mut drafts, mut bytes) = (0, 0, 0);
        for s in selections.values() {
            let (r, d, b) = s.diagnostic_retained();
            recipes += r;
            drafts += d;
            bytes += b + std::mem::size_of::<i32>();
        }
        bytes += rules.map_or(0, |r| {
            std::mem::size_of_val(r)
                + r.rules.capacity()
                    * std::mem::size_of::<Option<kirby_assist_core::reaction_rules::Candidate>>()
        });
        crate::performance::health::retained(selections.len(), recipes, drafts, bytes);
        self.retained_stamp = Some(stamp);
    }
}

/// Installs the renderer callbacks when ImGui support is compiled in.
pub(super) fn install() {
    #[cfg(feature = "imgui-menu")]
    menu::install();
}

#[cfg(test)]
#[path = "../../tests/unit/live/settings_menu/save_completion_tests.rs"]
/// Host regressions for save-result reconciliation and completion behavior.
mod save_completion_tests;
