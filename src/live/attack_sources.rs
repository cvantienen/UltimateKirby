//! Bounded, read-only projectile source tracking for shield and reflection.
use crate::game_ffi as game;
use std::sync::atomic::{AtomicBool, Ordering};
#[path = "projectile_sources.rs"]
mod projectile_sources;

const CAPACITY: usize = 32;
static SOURCES: projectile_sources::Sources = projectile_sources::Sources::new();
static TRACKING: AtomicBool = AtomicBool::new(false);

pub(super) fn tracking(enabled: bool) {
    TRACKING.store(enabled, Ordering::Relaxed);
}

/// Copied IDs only. Callers revalidate native identity/team before geometry.
pub(super) fn sources() -> [Option<u32>; CAPACITY] {
    SOURCES.snapshot().map(|key| key.map(|key| key.id))
}

pub(super) fn projectiles(
) -> [Option<kirby_assist_core::projectile_reflect::ProjectileKey>; CAPACITY] {
    SOURCES.snapshot()
}

pub(super) fn forget(id: u32) {
    SOURCES.forget(id);
}

#[skyline::hook(replace = game::attack_command)]
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

pub(super) unsafe fn install() {
    skyline::install_hook!(remember_attack);
}

pub(super) fn clear() {
    tracking(false);
    SOURCES.clear();
}
