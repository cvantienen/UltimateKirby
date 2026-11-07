//! Regression for save-time startup publication retaining whole menus forever.
use kirby_assist_core::{
    fighter_profiles::presets::PresetSlot,
    sequence_config::{persist, Catalog},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};

struct CountedSystem;
static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);
static PEAK_BYTES: AtomicIsize = AtomicIsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            let live = LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed)
                + layout.size() as isize;
            PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        System.dealloc(pointer, layout);
    }
}
#[global_allocator]
static ALLOCATOR: CountedSystem = CountedSystem;

#[test]
fn repeated_save_validation_and_renames_do_not_retain_menu_allocations() {
    let mut source = String::new();
    for fighter in ["kirby", "yoshi"] {
        for index in 0..32 {
            source.push_str(&format!(
                "[{fighter}.neutral.p{index}]\nstart_delay=0\nactions=jab\n"
            ));
        }
    }
    let sequence = Catalog::parse(&source).unwrap().entries()[0].sequence;
    let save = || {
        persist::validate_output(&source).unwrap();
        let updated = persist::update_named(
            &source,
            6,
            PresetSlot::Neutral,
            "p0",
            sequence,
            false,
            Some("Saved example"),
        )
        .unwrap();
        persist::validate_output(&updated).unwrap();
    };
    save(); // Warm any unrelated process-wide lazy reference data.
    let before = LIVE_BYTES.load(Ordering::SeqCst);
    for _ in 0..24 {
        save();
    }
    let retained = LIVE_BYTES.load(Ordering::SeqCst) - before;
    assert!(
        retained < 8192,
        "Repeated save validation retained {retained} bytes"
    );
    // A dense roster must not transiently allocate old+new geometric buffers.
    let mut large = String::new();
    for fighter in ["kirby", "yoshi"] {
        for slot in ["neutral", "jump", "roll", "attack", "drop"] {
            for i in 0..32 {
                large.push_str(&format!(
                    "[{fighter}.{slot}.p{i}]\nstart_delay=0\nactions=jab\n"
                ));
            }
        }
    }
    let baseline = LIVE_BYTES.load(Ordering::SeqCst);
    PEAK_BYTES.store(baseline, Ordering::SeqCst);
    let parsed = Catalog::parse(&large).unwrap();
    assert_eq!(parsed.len(), 320);
    let peak = PEAK_BYTES.load(Ordering::SeqCst) - baseline;
    let entry_bytes =
        parsed.len() * std::mem::size_of::<kirby_assist_core::sequence_config::Entry>();
    assert!(
        peak < (entry_bytes * 3 / 2) as isize,
        "Catalog parsing peaked at {peak} bytes for {entry_bytes} entry bytes"
    );
    // Default saves must release their parsed source before validating the new
    // file, rather than briefly retaining two whole fighter catalogs.
    let baseline = LIVE_BYTES.load(Ordering::SeqCst);
    PEAK_BYTES.store(baseline, Ordering::SeqCst);
    let defaulted = persist::set_default(&large, 6, PresetSlot::Neutral, "p0").unwrap();
    let default_peak = PEAK_BYTES.load(Ordering::SeqCst) - baseline;
    assert!(
        default_peak < (entry_bytes * 3 / 2) as isize,
        "Save-as-default peaked at {default_peak} bytes for {entry_bytes} catalog bytes"
    );
    persist::validate_output(&defaulted).unwrap();
    let profiles = kirby_assist_core::fighter_profiles::FighterProfiles::default();
    let allocations = ALLOCATIONS.load(Ordering::SeqCst);
    parsed.validate(&profiles).unwrap();
    let validation_allocations = ALLOCATIONS.load(Ordering::SeqCst) - allocations;
    assert!(validation_allocations < 64,
        "Validation allocated {validation_allocations} times for 320 entries; rows should be counted once");
}
