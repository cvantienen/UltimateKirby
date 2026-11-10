//! Bounded, read-only projectile source tracking for shield and reflection.
//! Technical: Tracks copied weapon/item IDs from attack callbacks for later
//! read-only projectile validation.
//! Plain English: This module remembers which projectiles have attacked without
//! keeping pointers into the game.
use crate::game_ffi as game;
use std::sync::atomic::{AtomicBool, Ordering};
#[path = "projectile_sources.rs"]
/// Technical: Implements the fixed-size atomic store used by attack-source tracking.
/// Plain English: Holds the no-allocation list of remembered projectile attacks.
mod projectile_sources;

/// Technical: Fixed number of projectile source entries stored by this module.
/// Plain English: The shared tracker holds up to 32 sources.
const CAPACITY: usize = 32;
/// Technical: Maximum number of projectile identities retained by the tracker.
/// Plain English: At most 32 recent projectile sources are remembered.
static SOURCES: projectile_sources::Sources = projectile_sources::Sources::new();
/// Technical: Gate indicating whether attack callbacks should record projectile sources.
/// Plain English: This switch turns projectile tracking on only when needed.
static TRACKING: AtomicBool = AtomicBool::new(false);

/// Technical: Publishes whether the attack callback should record projectile IDs.
/// Plain English: Turns projectile remembering on or off.
pub(super) fn tracking(enabled: bool) {
    TRACKING.store(enabled, Ordering::Relaxed);
}

/// Copied IDs only. Callers revalidate native identity/team before geometry.
/// Technical: Returns a copied snapshot of tracked object IDs, omitting generation data.
/// Plain English: Gives callers projectile IDs they can check again before use.
pub(super) fn sources() -> [Option<u32>; CAPACITY] {
    SOURCES.snapshot().map(|key| key.map(|key| key.id))
}

/// Technical: Returns a copied snapshot retaining each attack generation with its ID.
/// Plain English: Gives reflection logic projectile IDs that distinguish new attacks
/// from earlier attacks by the same object.
pub(super) fn projectiles(
) -> [Option<kirby_assist_core::projectile_reflect::ProjectileKey>; CAPACITY] {
    SOURCES.snapshot()
}

/// Technical: Removes the matching projectile identity from the bounded tracker.
/// Plain English: Stops treating the given object as a remembered projectile.
pub(super) fn forget(id: u32) {
    SOURCES.forget(id);
}

#[skyline::hook(replace = game::attack_command)]
/// Technical: Chains the native attack command, then records eligible active weapon/item IDs.
/// Plain English: After an attack starts, this remembers its projectile if tracking is enabled.
unsafe fn remember_attack(lua_state: u64) {
    original!()(lua_state);
    if !TRACKING.load(Ordering::Relaxed) || !super::feature_activity_allowed() {
        return;
    }
    let object = game::lua_object(lua_state);
    if object.is_null() {
        return;
    }
    let id = game::battle_object_id(object);
    if !matches!(
        id >> 28,
        kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
            | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
    ) {
        return;
    }
    SOURCES.remember(id);
}

/// Technical: Installs the hook that observes native attack-command callbacks.
/// Plain English: Connects projectile tracking to the game's attack events.
pub(super) unsafe fn install() {
    skyline::install_hook!(remember_attack);
}

/// Technical: Disables tracking and clears all retained projectile identities.
/// Plain English: Forgets every projectile and stops recording new ones.
pub(super) fn clear() {
    tracking(false);
    SOURCES.clear();
}
