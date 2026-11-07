//! Temporary aggregate counter only; no output, native calls or scheduling.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
static ACTIVE: AtomicBool = AtomicBool::new(false);
static CLOCK_ABORTS: AtomicUsize = AtomicUsize::new(0);
pub fn set_active(on: bool) {
    ACTIVE.store(on, Relaxed);
}
pub fn record_clock_abort() {
    if ACTIVE.load(Relaxed) {
        CLOCK_ABORTS.fetch_add(1, Relaxed);
    }
}
pub fn take_clock_aborts() -> usize {
    CLOCK_ABORTS.swap(0, Relaxed)
}
