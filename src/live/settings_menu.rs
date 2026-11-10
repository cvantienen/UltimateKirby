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
///
///  Handles the on-screen menu when ImGui support is enabled.
mod menu;
#[path = "settings_menu/save.rs"]
/// Serialized background worker and persistence implementation.
///
///  Reads and writes saved settings away from game callbacks.
mod save;

// Host adapters and worker tests share the same process-wide save globals.
// Serialize simulated mapper polling with worker tests just as STATE does in game.
#[cfg(test)]
/// Serializes host-side tests that share save-worker globals.
///
///  Prevents save tests from interfering with each other.
/// ### Returns
/// Returns the resulting `std::sync::MutexGuard<'static, ()>` value.
pub(super) fn save_test_guard() -> std::sync::MutexGuard<'static, ()> {
    save::WORKER_TEST.lock().unwrap()
}

/// Performs startup recovery of an interrupted saved-file update.
///
///  Restores a settings file left halfway through saving.
/// ### Returns
/// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
pub(super) fn recover_saved_file() -> Result<(), String> {
    save::recover()
}

/// Updates menu presence from an existing local-fighter callback timestamp.
///
///  Tells the menu when the local fighter was last seen.
/// ### Arguments
/// - `now` (`std::time::Instant`): Represents the current time; the function uses it as input to the operation described above.
pub(super) fn observe_local_fighter(now: std::time::Instant) {
    #[cfg(feature = "imgui-menu")]
    menu::observe_local_fighter(now);
    #[cfg(not(feature = "imgui-menu"))]
    let _ = now;
}

/// Requests a transient display of the runtime enabled state.
///
///  Shows whether the assist is switched on or off.
/// ### Arguments
/// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
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
///
///  Remembers to clear menu data after the match has safely ended.
static CLEANUP_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Reads the deferred cleanup request with acquire ordering.
///
///  Checks whether session cleanup is still waiting.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(super) fn cleanup_pending() -> bool {
    CLEANUP_PENDING.load(std::sync::atomic::Ordering::Acquire)
}

// Run only from the existing mapper callback, before it acquires STATE.
// Applying a save result can allocate menu drafts; cleanup frees editor data.
// Neither belongs inside the shared ImGui/GPU command-submission callback.
/// Applies completed save results and eligible cleanup outside shared callbacks.
///
///  Finishes save updates and cleanup when the menu can be safely changed.
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
///
///  Reuses the applied settings until the current fighter's choices actually change.
#[derive(Default)]
struct ResolvedSettings {
    /// Last settings value materialized from menu choices.
    ///
    ///  The current applied settings, if already calculated.
    settings: Option<AssistSettings>,
    /// Choice array used to produce `settings`.
    ///
    ///  The menu choices behind the cached settings.
    choices: Option<[usize; PresetSlot::COUNT]>,
    /// Recipe revision used to produce `settings`.
    ///
    ///  The recipe version behind the cached settings.
    recipe_revision: Option<u64>,
    /// Active preview slot used to produce `settings`.
    ///
    ///  Which recipe preview was active for these settings.
    preview: Option<PresetSlot>,
}
impl ResolvedSettings {
    /// Checks all cache-key components against the current selection.
    ///
    ///  Checks whether the saved settings still match the menu.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    fn is_current(&self, selection: &MenuSelection) -> bool {
        self.choices == Some(selection.choices)
            && self.recipe_revision == Some(selection.recipe_revision)
            && self.preview == selection.preview_slot
    }
    /// Recomputes effective settings only when the selection cache key changes.
    ///
    ///  Updates the applied settings only after a relevant menu change.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
    /// - `base` (`AssistSettings`): Represents the base; the function uses it as input to the operation described above.
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
///
///  Keeps prepared reaction rules for the current recipe version.
#[derive(Default)]
struct RuleCache {
    /// Fighter kind and recipe revision used for the cached rule set.
    ///
    ///  Identifies which fighter and recipe version these rules belong to.
    stamp: Option<(i32, u64)>,
    /// Owned resolved rules for the current cache stamp.
    ///
    ///  The prepared automatic rules for that fighter.
    rules: Option<Box<kirby_assist_core::reaction_rules::Resolved>>,
}
impl RuleCache {
    /// Returns rules only when both fighter and revision match the cache stamp.
    ///
    ///  Gets prepared rules only if they belong to the current selection.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `revision` (`u64`): Represents the revision; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn get(
        &self,
        kind: i32,
        revision: u64,
    ) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        (self.stamp == Some((kind, revision))).then_some(())?;
        self.rules.as_deref()
    }
    /// Drops the cache and stamp together.
    ///
    ///  Forgets old prepared rules after a fighter or session change.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    fn clear(&mut self) {
        *self = Self::default();
    }
    /// Resolves reaction recipes on cache misses and applies enabled controls.
    ///
    ///  Prepares automatic rules when their saved recipes change.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// - `options` (`&MenuPresets`): Represents the options; the function uses it as input to the operation described above.
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
///
///  Marks the menu state as changed.
/// ### Arguments
/// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
fn bump_revision(selection: &mut MenuSelection) {
    selection.revision = selection.revision.wrapping_add(1);
}
/// Advances both general and visible-content revisions.
///
///  Marks saved menu text as needing an update.
/// ### Arguments
/// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
fn bump_content(selection: &mut MenuSelection) {
    bump_revision(selection);
    selection.content_revision = selection.content_revision.wrapping_add(1);
}
/// Clears an active preview and advances the revision when one existed.
///
///  Stops a recipe preview and notifies the menu that it changed.
/// ### Arguments
/// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
fn end_preview(selection: &mut MenuSelection) {
    if selection.preview_slot.take().is_some() {
        bump_revision(selection);
    }
}

#[cfg(feature = "diagnostic-logging")]
#[derive(Default)]
/// Caches diagnostic configuration and retained-memory accounting revisions.
///
///  Avoids repeating unchanged diagnostic work.
struct DiagnosticState {
    /// Last resolved diagnostic configuration.
    ///
    ///  Previously prepared diagnostic settings.
    config: Option<kirby_assist_core::diagnostics::modes::Configuration>,
    /// Current configuration cache key, including enablement.
    ///
    ///  Identifies which diagnostic settings were last checked.
    stamp: Option<(i32, u64, bool)>,
    /// Last retained-memory accounting cache key.
    ///
    ///  Tracks which menu version was last counted for memory.
    retained_stamp: Option<(i32, u64)>,
}

#[derive(Default)]
/// Per-fighter menu selections and derived runtime caches.
///
///  Holds the player's menu choices and settings for the current fighter.
pub(super) struct SettingsMenu {
    /// Selections retained by native fighter kind.
    ///
    ///  Menu choices kept separately for each fighter.
    selections: BTreeMap<i32, MenuSelection>,
    /// Native fighter kind currently selected by the menu.
    ///
    ///  The fighter whose choices are currently active.
    kind: Option<i32>,
    /// Buttons consumed by the menu on the latest input sample.
    ///
    ///  Buttons the menu kept from reaching the game this update.
    pub consumed: u32,
    /// Native button mask selected for forwarding.
    ///
    ///  Buttons that should still reach the game.
    pub native_buttons: u32,
    /// Built-in menu consumption mask from the latest selection input.
    ///
    ///  Buttons handled by the game's own menu controls.
    pub builtin_consumed: u32,
    /// Whether the ledge-assist hold is active.
    ///
    ///  Whether the ledge helper is currently being held.
    pub ledge_held: bool,
    /// Whether the current selection is in gameplay scope.
    ///
    ///  Whether gameplay settings should currently take effect.
    scope: bool,
    /// Cache of effective settings for the current selection.
    ///
    ///  Reuses the applied settings until choices change.
    resolved: ResolvedSettings,
    /// Revision of the last successfully published view.
    ///
    ///  Remembers which menu version is already displayed.
    view_revision: Option<u64>,
    /// Cache for resolved automatic reaction recipes.
    ///
    ///  Reuses prepared automatic rules until recipes change.
    rules: RuleCache,
    /// Effective auto-mash value retained across selection resets.
    ///
    ///  Current auto-mash choice.
    auto_mash: Option<bool>,
    // Retain a real release through first profile selection/session reset.
    /// Physical button/layout snapshot replayed after a fresh selection is initialized.
    ///
    ///  Remembers the real buttons through a menu or fighter switch.
    passthrough_buttons: Option<(u32, kirby_assist_core::native_input::ControlLayout)>,
    /// Defers the request to open the quick menu until selection input.
    ///
    ///  Remembers to open the menu at the next safe update.
    open_requested: bool,
    /// Defers restoration of saved training state until gameplay resumes.
    ///
    ///  Remembers to restore saved menu state when training starts again.
    resume_saved_state: bool,
    #[cfg(feature = "diagnostic-logging")]
    /// Deduplicates diagnostic configuration and retained-memory snapshots.
    ///
    ///  Avoids repeating diagnostic logs and memory calculations.
    diagnostics: DiagnosticState,
}
impl SettingsMenu {
    /// Returns the selection for the currently selected fighter.
    ///
    ///  Gets the current fighter's menu choices.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn current(&self) -> Option<&MenuSelection> {
        self.kind.and_then(|kind| self.selections.get(&kind))
    }
    /// Returns the mutable selection for the currently selected fighter.
    ///
    ///  Gets the current fighter's choices so they can be changed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn current_mut(&mut self) -> Option<&mut MenuSelection> {
        let kind = self.kind?;
        self.selections.get_mut(&kind)
    }
    /// The active selection while its fighter is in gameplay scope.
    ///
    ///  Gets the current fighter's choices only while gameplay is active.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn scoped(&self) -> Option<&MenuSelection> {
        self.current().filter(|_| self.scope)
    }
    /// Scoped selection that is not paused on an editor/tuning/manage page.
    ///
    ///  Gets choices that can currently control gameplay, not editor pages.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn live(&self) -> Option<&MenuSelection> {
        self.scoped()
            .filter(|s| !s.recipe_editing && !s.tuning_page && !s.managing())
    }
    /// Native fighter changes can precede the next mapper/menu poll. Never
    /// expose a previous fighter's automatic rules or ranges in that gap.
    ///
    ///  Hides old fighter settings as soon as the game reports a different fighter.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn for_fighter(&self, kind: i32) -> Option<&Self> {
        (self.kind == Some(kind)).then_some(self)
    }
    #[cfg(feature = "diagnostic-logging")]
    /// Finds a cached display label for a saved menu choice.
    ///
    ///  Looks up the names shown for a saved choice.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
    ///
    ///  Reports which saved recipe is being previewed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn preview_slot(&self) -> Option<PresetSlot> {
        self.current().and_then(|s| s.preview_slot)
    }
    /// Polls the save worker and reconciles a completed result.
    ///
    ///  Checks whether a save finished and updates the menu.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub(super) fn poll_save_result(&mut self) -> bool {
        save::poll().is_some_and(|result| self.apply_save_result(result, PROFILES.get()))
    }
    /// Reconcile a finished save with the selection that requested it. Returns
    /// true when a delete changed the active fighter's saved-file list.
    ///
    ///  Applies a finished save only to the matching current recipe.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `result` (`save::ResultMessage`): Represents the result being processed; the function uses it as input to the operation described above.
    /// - `profiles` (`Option<&FighterProfiles>`): Represents the profiles; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    ///
    ///  Reports whether the assist should be active right now.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `base` (`bool`): Represents the base; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.ledge_held || self.scoped().is_some_and(|s| s.sequence_enabled(false))
    }
    /// Resolves manual native actions for the scoped fighter selection.
    ///
    ///  Gets the buttons requested by manually configured actions.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
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
    ///
    ///  Checks whether an automatic mode is currently claimed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn reserves_automatic_modes(&self) -> bool {
        self.scoped().is_some_and(|s| s.reserves_automatic_modes())
    }
    /// Ends the current selection's preview and updates its revision.
    ///
    ///  Stops the currently previewed recipe.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn finish_preview(&mut self) {
        if let Some(selection) = self.current_mut() {
            end_preview(selection);
        }
    }
    /// Reads ledge-trump hold state only from the scoped selection.
    ///
    ///  Checks whether the ledge-trump control is held.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn ledge_trump_held(&self) -> bool {
        self.scoped().is_some_and(MenuSelection::ledge_trump_held)
    }
    /// Returns the effective Auto Tech mode with startup fallback.
    ///
    ///  Checks whether automatic teching is enabled.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `startup_default` (`bool`): Represents the startup default; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn auto_tech(&self, startup_default: bool) -> bool {
        self.current().map_or(startup_default, |s| {
            s.effective_mode(Mode::AutoTech, s.auto_tech)
        })
    }
    /// Reads grab-dodge enablement from the scoped selection.
    ///
    ///  Checks whether grab dodge assistance is on.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn grab_dodge_enabled(&self) -> bool {
        self.scoped().is_some_and(|s| s.grab_dodge)
    }
    #[cfg(feature = "diagnostic-logging")]
    /// Returns the current Performance diagnostic option.
    ///
    ///  Checks whether performance diagnostics are enabled.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn diagnostic_options(&self) -> bool {
        self.current().is_some_and(|s| s.performance)
    }
    /// Reads out-of-shield assistance from the scoped selection.
    ///
    ///  Checks whether out-of-shield help is on.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn out_of_shield_enabled(&self) -> bool {
        self.scoped().is_some_and(|s| s.out_of_shield)
    }
    /// Defers training ledge-hold restoration until gameplay input.
    ///
    ///  Asks the menu to restore saved training state later.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn resume_saved_state(&mut self) {
        self.resume_saved_state = true;
    }
    /// Returns effective idle-crouch and tilt-turn modes for the current fighter.
    ///
    ///  Gets the two posture helper settings currently in effect.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `(bool, bool)` value.
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
    ///
    ///  Requests that the settings menu open.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn request_open(&mut self) {
        self.open_requested = true;
    }
    /// Returns stored tuning for `kind` only when it is the active fighter.
    ///
    ///  Gets the current fighter's tuning values, or defaults for another.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `kirby_assist_core::fighter_tuning::Tuning` value.
    pub fn fighter_tuning(&self, kind: i32) -> kirby_assist_core::fighter_tuning::Tuning {
        self.for_fighter(kind)
            .and_then(Self::current)
            .map_or(Default::default(), |s| s.tuning)
    }
    /// Builds a fighter-specific reflect configuration for a live selection.
    ///
    ///  Gets the current automatic projectile-reflect settings.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
    ///
    ///  Gets how far ahead the dodge helper checks.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `f32` value.
    pub fn dodge_lookahead(&self) -> f32 {
        self.current()
            .map_or(kirby_assist_core::spot_dodge::LOOKAHEAD, |s| {
                s.tuning.dodge_lookahead()
            })
    }
    /// Returns resolved rules only when they contain enabled menu controls.
    ///
    ///  Gets automatic rules that are active from the menu.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn rule_config(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules().filter(|r| {
            r.has_simple_events()
                && r.enabled_controls & kirby_assist_core::reaction_rules::Control::Menu.bit() != 0
        })
    }
    /// Selects cached rules matching the current scoped recipe revision.
    ///
    ///  Gets the prepared rules for the current recipe version.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn current_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        let revision = self.scoped()?.recipe_revision;
        self.rules.get(self.kind?, revision)
    }
    /// Grab avoidance owns its follow-up toggle independently of Auto react.
    /// Only its own event recipe may punish a confirmed grab dodge.
    ///
    ///  Gets the grab-dodge recipe only when grab dodge is enabled.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn grab_dodge_sequence(&self) -> Option<&kirby_assist_core::sequence_config::Sequence> {
        if !self.live()?.grab_dodge {
            return None;
        }
        self.current_rules()?.simple_sequence(5)
    }
    /// All Auto react recipes share a revision cache, independent of the
    /// individual mode switches that continue to control their admission.
    /// Builds the automatic-action view over the cached resolved recipes.
    ///
    ///  Gets prepared recipes used by automatic follow-up actions.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `kirby_assist_core::out_of_shield::AutomaticConfig<'_>` value.
    pub fn automatic_sequences(&self) -> kirby_assist_core::out_of_shield::AutomaticConfig<'_> {
        let rules = self.current_rules();
        kirby_assist_core::out_of_shield::AutomaticConfig {
            grab: rules.and_then(|r| r.simple_sequence(3)),
            shield_grab: rules.and_then(|r| r.simple_sequence(4)),
            backup: rules.and_then(|r| r.simple_sequence(6)),
        }
    }
    /// Returns configured time/distance limits for the three automatic recipes.
    ///
    ///  Gets how far and how long automatic actions may search.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `[Option<(f32, f32)>; 3]` value.
    pub fn automatic_limits(&self) -> [Option<(f32, f32)>; 3] {
        let r = self.automatic_sequences();
        [r.grab, r.shield_grab, r.backup].map(|s| s.map(|s| (s.reaction.max, s.reaction.height)))
    }
    /// Returns cached resolved rules when at least one mode is enabled.
    ///
    ///  Gets automatic rules currently available to gameplay.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn automatic_rules(&self) -> Option<&kirby_assist_core::reaction_rules::Resolved> {
        self.current_rules()
    }
    /// Records physical passthrough input for the selected fighter's binding state.
    ///
    ///  Remembers buttons that passed through while menu selection changed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `layout` (`kirby_assist_core::native_input::ControlLayout`): Represents the layout; the function uses it as input to the operation described above.
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
    ///
    ///  Gets the current auto-mash choice.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `startup_default` (`bool`): Represents the startup default; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn auto_mash(&self, startup_default: bool) -> bool {
        let base = self.auto_mash.unwrap_or(startup_default);
        self.current()
            .map_or(base, |s| s.effective_mode(Mode::AutoMash, base))
    }
    /// Drops current scope, pending actions, and derived caches without deleting profiles.
    ///
    ///  Pauses menu use and clears temporary active state.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
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
    ///
    ///  Starts using the menu state for a match.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
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
    ///
    ///  Stops using match choices and clears them when safe.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
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
    ///
    ///  Tells the display whether a battle is active.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `active` (`bool`): Represents the active; the function uses it as input to the operation described above.
    pub fn display_battle(&self, active: bool) {
        #[cfg(feature = "imgui-menu")]
        menu::set_battle_active(active);
        #[cfg(not(feature = "imgui-menu"))]
        let _ = active;
    }
    /// True releases old proposals when navigating or changing a tool.
    /// Routes one physical-input sample, refreshes derived caches, and publishes changed views.
    ///
    ///  Applies one frame of menu input and updates the active settings.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `available` (`bool`): Represents the available; the function uses it as input to the operation described above.
    /// - `gameplay` (`bool`): Represents the gameplay; the function uses it as input to the operation described above.
    /// - `bindings_blocked` (`bool`): Represents the bindings blocked; the function uses it as input to the operation described above.
    /// - `safety_jump` (`bool`): Represents the safety jump; the function uses it as input to the operation described above.
    /// - `kind` (`Option<i32>`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `physical` (`u32`): Represents the physical; the function uses it as input to the operation described above.
    /// - `layout` (`kirby_assist_core::native_input::ControlLayout`): Represents the layout; the function uses it as input to the operation described above.
    /// - `base` (`&AssistSettings`): Represents the base; the function uses it as input to the operation described above.
    /// - `mash_default` (`bool`): Represents the mash default; the function uses it as input to the operation described above.
    /// - `grab_default` (`bool`): Represents the grab default; the function uses it as input to the operation described above.
    /// - `tech_default` (`bool`): Represents the tech default; the function uses it as input to the operation described above.
    /// - `training` (`bool`): Represents the training; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    ///
    ///  Switches the menu to a fighter without reusing the old fighter's active data.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
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
    ///
    ///  Gets the settings gameplay should use for this fighter.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`Option<i32>`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `startup` (`&AssistSettings`): Represents the startup; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `AssistSettings` value.
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
///
///  Loads the starting menu choices for a fighter.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `base` (`&AssistSettings`): Represents the base; the function uses it as input to the operation described above.
/// - `grab_default` (`bool`): Represents the grab default; the function uses it as input to the operation described above.
/// - `tech_default` (`bool`): Represents the tech default; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `MenuSelection` value.
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
///
///  Sends one controller update through the menu and reports consumed buttons.
/// ### Arguments
/// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `options` (`&MenuPresets`): Represents the options; the function uses it as input to the operation described above.
/// - `bindings_blocked` (`bool`): Represents the bindings blocked; the function uses it as input to the operation described above.
/// - `safety_jump` (`bool`): Represents the safety jump; the function uses it as input to the operation described above.
/// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
/// - `physical` (`u32`): Represents the physical; the function uses it as input to the operation described above.
/// - `layout` (`kirby_assist_core::native_input::ControlLayout`): Represents the layout; the function uses it as input to the operation described above.
/// - `training` (`bool`): Represents the training; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `u32` value.
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
///
///  Starts a save after the player asks to save menu changes.
/// ### Arguments
/// - `selection` (`&mut MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `options` (`&MenuPresets`): Represents the options; the function uses it as input to the operation described above.
/// - `training` (`bool`): Represents the training; the function uses it as input to the operation described above.
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
///
///  Packages the current recipe choice for saving or deletion.
/// ### Arguments
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
/// - `options` (`&MenuPresets`): Represents the options; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
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
///
///  Keeps diagnostic menu values in the expected report positions.
mod config_rows {
    use super::PresetSlot;
    /// Number of rows in the core diagnostic configuration table.
    ///
    ///  Total number of diagnostic menu rows.
    pub const COUNT: usize = 25;
    /// Diagnostic row index for the Y binding.
    ///
    ///  Position of the Y binding in the report.
    pub const Y_BINDING: usize = 6;
    /// Diagnostic row index for Auto Mash.
    ///
    ///  Position of Auto Mash in the report.
    pub const AUTO_MASH: usize = 7;
    /// Diagnostic row index for Idle Crouch.
    ///
    ///  Position of Idle Crouch in the report.
    pub const IDLE_CROUCH: usize = 8;
    /// Diagnostic row index for Tilt Turn.
    ///
    ///  Position of Tilt Turn in the report.
    pub const TILT_TURN: usize = 9;
    /// Diagnostic row index for Spot Punish.
    ///
    ///  Position of Spot Punish in the report.
    pub const SPOT_PUNISH: usize = 11;
    /// Diagnostic row index for Grab Dodge.
    ///
    ///  Position of Grab Dodge in the report.
    pub const GRAB_DODGE: usize = 12;
    /// Diagnostic row index for Auto Tech.
    ///
    ///  Position of Auto Tech in the report.
    pub const AUTO_TECH: usize = 13;
    /// Diagnostic row index for Performance.
    ///
    ///  Position of Performance in the report.
    pub const PERFORMANCE: usize = 15;
    /// Diagnostic row index for Ledge Mode.
    ///
    ///  Position of Ledge Mode in the report.
    pub const LEDGE_MODE: usize = 22;
    /// Diagnostic row index for Out of Shield.
    ///
    ///  Position of Out of Shield in the report.
    pub const OUT_OF_SHIELD: usize = 23;
    /// Diagnostic row index for Shield Pressure.
    ///
    ///  Position of Shield Pressure in the report.
    pub const SHIELD_PRESSURE: usize = 24;
    /// Rows that show a slot's selected choice, with the text used when it has none.
    ///
    ///  Lists the choice rows and the fallback label for an empty slot.
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
///
///  Prepares the current menu choices for a diagnostic report.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `kirby_assist_core::diagnostics::modes::Configuration` value.
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
    ///
    ///  Writes a new settings snapshot only when something changed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
    /// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
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
    ///
    ///  Updates memory estimates after a fighter or menu revision changes.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `stamp` (`(i32, u64)`): Represents the stamp; the function uses it as input to the operation described above.
    /// - `selections` (`&BTreeMap<i32, MenuSelection>`): Represents the selections; the function uses it as input to the operation described above.
    /// - `rules` (`Option<&kirby_assist_core::reaction_rules::Resolved>`): Represents the rules; the function uses it as input to the operation described above.
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
///
///  Connects the settings menu to the screen.
pub(super) fn install() {
    #[cfg(feature = "imgui-menu")]
    menu::install();
}

#[cfg(test)]
#[path = "../../tests/unit/live/settings_menu/save_completion_tests.rs"]
/// Host regressions for save-result reconciliation and completion behavior.
///
///  Tests that finished saves update the menu correctly.
mod save_completion_tests;
