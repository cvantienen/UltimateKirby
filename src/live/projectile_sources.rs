//! Bounded copied projectile identities. A single atomic record keeps the
//! object ID and its native ATTACK generation coherent across hook callbacks.
//! Technical: Stores projectile identity/generation pairs in a fixed-size atomic
//! ring so concurrent callback snapshots remain self-contained.
//! Plain English: This module remembers recent projectiles without allocating or
//! keeping game pointers.
use kirby_assist_core::projectile_reflect::ProjectileKey;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// Represents an empty record for projectile sources.
/// Sentinel used for unused slots in the packed atomic record array.
const EMPTY: u64 = u64::MAX;
/// The maximum number of projectile sources that can be tracked.
/// Number of fixed atomic slots in [`Sources`].
const CAPACITY: usize = 32;

/// Tracks projectile sources with bounded capacity and atomic coherence.
/// Fixed-capacity tracker whose packed atomic records pair each object ID
/// with the generation of its observed attack.
/// pointers into the game.
pub(super) struct Sources {
    /// Packed object-ID/generation records; `EMPTY` marks vacant slots.
    records: [AtomicU64; CAPACITY],
    /// Round-robin replacement cursor used when all records are occupied.
    next: AtomicUsize,
    /// Monotonic wrapping counter assigned to observed attack records.
    generation: AtomicU32,
}
impl Sources {
    /// Creates a new instance of `Sources` with empty records and initial generation.
    /// Constructs an empty fixed-capacity tracker with generation zero.
    pub(super) const fn new() -> Self {
        Self {
            records: [const { AtomicU64::new(EMPTY) }; CAPACITY],
            next: AtomicUsize::new(0),
            generation: AtomicU32::new(0),
        }
    }
    /// Remembers a projectile source by its ID, updating the generation if necessary.
    /// Validates weapon/item category, advances the attack generation,
    /// and updates an existing slot or inserts/replaces one.
    /// is reused for a later attack.
    pub(super) fn remember(&self, id: u32) {
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
    /// Returns a snapshot of the current projectile sources as an array of `Option<ProjectileKey>`.
    /// Loads each packed atomic record into a copied optional key array.
    pub(super) fn snapshot(&self) -> [Option<ProjectileKey>; CAPACITY] {
        std::array::from_fn(|index| {
            let record = self.records[index].load(Ordering::Relaxed);
            (record != EMPTY).then_some(ProjectileKey {
                id: record as u32,
                generation: (record >> 32) as u32,
            })
        })
    }
    /// Forgets a projectile source by its ID, marking its slot as empty.
    /// Uses compare-exchange to clear the first matching ID without
    /// erasing a concurrently replaced record.
    pub(super) fn forget(&self, id: u32) {
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
    /// Clears all projectile sources, resetting the next index but preserving the generation.
    /// Marks every slot empty and resets the replacement cursor while
    /// deliberately retaining the generation counter.
    /// from old receipts.
    pub(super) fn clear(&self) {
        for slot in &self.records {
            slot.store(EMPTY, Ordering::Relaxed);
        }
        self.next.store(0, Ordering::Relaxed);
        // Preserve generation across a brief route/tracker reset, so a fresh
        // shot cannot collide with an old receipt retained by its input owner.
    }
}
