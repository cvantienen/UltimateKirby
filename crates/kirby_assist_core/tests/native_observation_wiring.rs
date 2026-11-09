//! Typecheck the complete production observation/frame modules on the host.
//! Execute only copied-ID snapshots; no native FFI calls or installed hooks.
#![allow(dead_code, unexpected_cfgs, unused_imports)]
use kirby_assist_core::{
    battle_objects::FightersInGame, fighter_profiles::FighterProfiles,
    ledge_position::LedgePosition, ledge_trap::LedgeTrap, sequence_config::SequenceLibrary,
};
use std::sync::{atomic::AtomicUsize, Mutex, OnceLock};
#[path = "../../../src/game_ffi.rs"]
mod native_abi;
mod native_params {
    pub const PARAM_GROUND_BRAKE: u64 = 0x0c8cc9db76;
    pub const PARAM_AIR_ACCEL_Y: u64 = 0xba18057d9;
    pub const PARAM_AIR_SPEED_Y_STABLE: u64 = 0x12ec5626fe;
}
thread_local! {
    static LINKS:std::cell::Cell<u32>=const{std::cell::Cell::new(0)};
    static LINK_READS:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};
}
mod game_ffi {
    pub use super::native_abi::*;
    pub unsafe fn is_link(_: *mut Object, slot: i32) -> bool {
        assert!(matches!(slot, 0 | 2));
        super::LINK_READS.with(|n| n.set(n.get() + 1));
        super::LINKS.with(|links| links.get() & (1 << slot) != 0)
    }
}
use game_ffi as game;
#[cfg(feature = "diagnostic-logging")]
#[path = "../../../src/performance.rs"]
mod performance;
struct Menu;
impl Menu {
    fn preview_slot(&self) -> Option<kirby_assist_core::fighter_profiles::presets::PresetSlot> {
        None
    }
}
struct State {
    ledge: Option<LedgeTrap>,
    ledge_position: Option<LedgePosition>,
    ledge_roster: Option<(FightersInGame, i64)>,
    settings_menu: Option<Menu>,
}
static STATE: Mutex<State> = Mutex::new(State {
    ledge: None,
    ledge_position: None,
    ledge_roster: None,
    settings_menu: None,
});
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<SequenceLibrary>> = OnceLock::new();
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
mod attack_sources {
    use super::*;
    pub fn sources() -> [Option<u32>; 32] {
        [None; 32]
    }
}
#[path = "../../../src/live/frame.rs"]
mod frame;
#[path = "../../../src/live/observations.rs"]
mod observations;
#[test]
fn real_frame_cache_keeps_sixteen_actor_roster_copies_scoped_to_one_observation() {
    let roster = std::array::from_fn(|i| Some(i as u32));
    STATE.lock().unwrap().ledge_roster = Some((roster, 1));
    let reads = frame::FrameReads::default();
    assert_eq!(reads.roster(), Some(roster));
    STATE.lock().unwrap().ledge_roster = None;
    assert_eq!(reads.roster(), Some(roster));
    assert_eq!(frame::FrameReads::default().roster(), None);
    assert_eq!(reads.known_statuses(), [None; 16]);
}

#[test]
fn capture_stop_link_is_protected_but_partner_tether_does_not_look_like_a_capture() {
    for (links, captured) in [(0, false), (1, true), (1 << 2, true), (1 << 12, false)] {
        LINKS.with(|v| v.set(links));
        LINK_READS.with(|v| v.set(0));
        let mut header = [0u64, 1];
        let object = header.as_mut_ptr().cast::<game::Object>();
        let reads = frame::FrameReads::default();
        for _ in 0..100 {
            assert_eq!(unsafe { reads.capture_link(object) }, captured);
        }
        assert!(
            LINK_READS.with(|v| v.get()) <= 2,
            "same observation caches both capture-link checks"
        );
    }
}
