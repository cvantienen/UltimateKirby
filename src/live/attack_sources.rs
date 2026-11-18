//! Bounded, read-only projectile source tracking for shield and reflection.
//! Technical: Tracks copied weapon/item IDs from attack callbacks for later
//! read-only projectile validation.
//! Plain English: This module remembers which projectiles have attacked without
//! keeping pointers into the game.
use crate::game_ffi as game;
use kirby_assist_core::projectile_reflect::ProjectileKey;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
/// Fixed number of projectile source entries stored by this module.
const CAPACITY: usize = 32;
/// Sentinel used for unused slots in the packed atomic record array.
const EMPTY: u64 = u64::MAX;

/// Fixed-capacity tracker whose packed atomic records pair each object ID
/// with the generation of its observed attack.
struct Sources {
    /// Packed object-ID/generation records; `EMPTY` marks vacant slots.
    records: [AtomicU64; CAPACITY],
    /// Round-robin replacement cursor used when all records are occupied.
    next: AtomicUsize,
    /// Monotonic wrapping counter assigned to observed attack records.
    generation: AtomicU32,
}

impl Sources {
    /// Constructs an empty fixed-capacity tracker with generation zero.
    const fn new() -> Self {
        Self {
            records: [const { AtomicU64::new(EMPTY) }; CAPACITY],
            next: AtomicUsize::new(0),
            generation: AtomicU32::new(0),
        }
    }

    /// Validates weapon/item category, advances the attack generation, and
    /// updates an existing slot or inserts/replaces one.
    fn remember(&self, id: u32) {
        if !matches!(
            id >> 28,
            kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
                | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
        ) {
            return;
        }
        let generation = self
            .generation
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        let record = (u64::from(generation) << 32) | u64::from(id);
        let mut vacant = None;
        for (index, slot) in self.records.iter().enumerate() {
            let old = slot.load(Ordering::Relaxed);
            if old != EMPTY && old as u32 == id {
                // This includes a reused weapon/item and a fresh attack phase
                // on a live article. Keeping only its ID would suppress it.
                slot.store(record, Ordering::Relaxed);
                return;
            }
            if old == EMPTY {
                vacant.get_or_insert(index);
            }
        }
        let index = vacant.unwrap_or_else(|| self.next.fetch_add(1, Ordering::Relaxed) % CAPACITY);
        self.records[index].store(record, Ordering::Relaxed);
    }

    /// Loads each packed atomic record into a copied optional key array.
    fn snapshot(&self) -> [Option<ProjectileKey>; CAPACITY] {
        std::array::from_fn(|index| {
            let record = self.records[index].load(Ordering::Relaxed);
            (record != EMPTY).then_some(ProjectileKey {
                id: record as u32,
                generation: (record >> 32) as u32,
            })
        })
    }

    /// Uses compare-exchange to clear the first matching ID without erasing a
    /// concurrently replaced record.
    fn forget(&self, id: u32) {
        for slot in &self.records {
            let old = slot.load(Ordering::Relaxed);
            if old != EMPTY
                && old as u32 == id
                && slot
                    .compare_exchange(old, EMPTY, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                break;
            }
        }
    }

    /// Marks every slot empty and resets the replacement cursor while
    /// deliberately retaining the generation counter.
    fn clear(&self) {
        for slot in &self.records {
            slot.store(EMPTY, Ordering::Relaxed);
        }
        self.next.store(0, Ordering::Relaxed);
    }
}

/// Maximum number of projectile identities retained by the tracker.
static SOURCES: Sources = Sources::new();
/// Gate indicating whether attack callbacks should record projectile sources.
static TRACKING: AtomicBool = AtomicBool::new(false);

/// Publishes whether the attack callback should record projectile IDs.
pub(super) fn tracking(enabled: bool) {
    TRACKING.store(enabled, Ordering::Relaxed);
}

/// Copied IDs only. Callers revalidate native identity/team before geometry.
/// Returns a copied snapshot of tracked object IDs, omitting generation data.
pub(super) fn sources() -> [Option<u32>; CAPACITY] {
    SOURCES.snapshot().map(|key| key.map(|key| key.id))
}

/// Returns a copied snapshot retaining each attack generation with its ID.
/// from earlier attacks by the same object.
pub(super) fn projectiles() -> [Option<ProjectileKey>; CAPACITY] {
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
    if !TRACKING.load(Ordering::Relaxed) || !super::feature_activity_allowed() {
        return;
    }
    let object = game::lua_object(lua_state);
    if object.is_null() {
        return;
    }
    let id = game::battle_object_id(object);
    SOURCES.remember(id);
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
