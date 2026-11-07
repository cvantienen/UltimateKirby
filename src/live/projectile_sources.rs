//! Bounded copied projectile identities. A single atomic record keeps the
//! object ID and its native ATTACK generation coherent across hook callbacks.
use kirby_assist_core::projectile_reflect::ProjectileKey;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

const EMPTY: u64 = u64::MAX;
const CAPACITY: usize = 32;
pub(super) struct Sources {
    records: [AtomicU64; CAPACITY],
    next: AtomicUsize,
    generation: AtomicU32,
}
impl Sources {
    pub(super) const fn new() -> Self {
        Self {
            records: [const { AtomicU64::new(EMPTY) }; CAPACITY],
            next: AtomicUsize::new(0),
            generation: AtomicU32::new(0),
        }
    }
    pub(super) fn remember(&self, id: u32) {
        if !matches!(id >> 28, 1 | 4) {
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
    pub(super) fn snapshot(&self) -> [Option<ProjectileKey>; CAPACITY] {
        std::array::from_fn(|index| {
            let record = self.records[index].load(Ordering::Relaxed);
            (record != EMPTY).then_some(ProjectileKey {
                id: record as u32,
                generation: (record >> 32) as u32,
            })
        })
    }
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
    pub(super) fn clear(&self) {
        for slot in &self.records {
            slot.store(EMPTY, Ordering::Relaxed);
        }
        self.next.store(0, Ordering::Relaxed);
        // Preserve generation across a brief route/tracker reset, so a fresh
        // shot cannot collide with an old receipt retained by its input owner.
    }
}
