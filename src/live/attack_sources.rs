//! Skyline hook layer over the portable projectile source tracker.
//! Technical: Records copied weapon/item IDs from native attack callbacks into
//! `kirby_assist_core::projectile_reflect::Sources` for later read-only
//! projectile validation.
//! Plain English: This module remembers which projectiles have attacked without
//! keeping pointers into the game.
use crate::game_ffi as game;
use kirby_assist_core::projectile_reflect::{ProjectileKey, Sources, SOURCE_CAPACITY};
use std::sync::atomic::{AtomicBool, Ordering};

/// Projectile identities observed since the last clear.
static SOURCES: Sources = Sources::new();
/// Gate indicating whether attack callbacks should record projectile sources.
static TRACKING: AtomicBool = AtomicBool::new(false);

/// Publishes whether the attack callback should record projectile IDs.
pub(super) fn tracking(enabled: bool) {
    TRACKING.store(enabled, Ordering::Relaxed);
}

/// Copied IDs only. Callers revalidate native identity/team before geometry.
/// Returns a copied snapshot of tracked object IDs, omitting generation data.
pub(super) fn sources() -> [Option<u32>; SOURCE_CAPACITY] {
    SOURCES.snapshot().map(|key| key.map(|key| key.id))
}

/// Returns copied IDs with software ATTACK observation generations. This is
/// not an allocation token: unobserved reuse and ownership transfer remain unknown.
pub(super) fn projectiles() -> [Option<ProjectileKey>; SOURCE_CAPACITY] {
    SOURCES.snapshot()
}

/// Removes the matching projectile identity from the bounded tracker.
pub(super) fn forget(id: u32) {
    SOURCES.forget(id);
}

#[skyline::hook(replace = game::attack_command)]
/// Chains the native attack command, then records eligible active weapon/item IDs.
unsafe fn remember_attack(lua_state: u64) {
    original!()(lua_state);
    if crate::logging::active() && game::is_training_mode() && crate::diagnostics::telemetry::ready() {
        super::telemetry::remember_attack(game::lua_object(lua_state));
    }
    if !TRACKING.load(Ordering::Relaxed) || !super::feature_activity_allowed() {
        return;
    }
    let object = game::lua_object(lua_state);
    if object.is_null() {
        return;
    }
    SOURCES.remember(game::battle_object_id(object));
}

/// Installs the hook that observes native attack-command callbacks.
pub(super) unsafe fn install() {
    skyline::install_hook!(remember_attack);
}

/// Disables tracking and clears all retained projectile identities.
pub(super) fn clear() {
    tracking(false);
    SOURCES.clear();
}
