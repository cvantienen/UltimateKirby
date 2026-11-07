//! Exercise the production menu cache across real fighter changes on the host.
#![allow(dead_code, unexpected_cfgs)]
use kirby_assist_core::{
    fighter_profiles::FighterProfiles, ledge_trap::LedgeSettings, native_input::ControlLayout,
};
use std::sync::{Mutex, OnceLock};
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<kirby_assist_core::sequence_config::SequenceLibrary>> =
    OnceLock::new();
static SEQUENCES_UPDATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MASTER_IN_MATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct HostState {
    menu_battle_active: bool,
    ledge_menu: Option<live_menu::LedgeMenu>,
}
static STATE: Mutex<HostState> = Mutex::new(HostState {
    menu_battle_active: false,
    ledge_menu: None,
});
fn ledge_settings(_: Option<i32>, base: LedgeSettings) -> LedgeSettings {
    base
}
#[cfg(feature = "diagnostic-logging")]
#[path = "../../../src/performance.rs"]
mod performance;
mod calibration {
    pub fn clear() {}
}
#[cfg(feature = "diagnostic-logging")]
mod logging {
    pub fn write_configuration(_: kirby_assist_core::diagnostics::modes::Configuration) {}
}
#[path = "../../../src/live/ledge_menu.rs"]
mod live_menu;

struct Counted;
thread_local! {
 static COUNTING:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};
 static ALLOCS:std::cell::Cell<usize>=const {std::cell::Cell::new(0)};
 static LIVE:std::cell::Cell<isize>=const {std::cell::Cell::new(0)};
}
unsafe impl std::alloc::GlobalAlloc for Counted {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        let p = std::alloc::GlobalAlloc::alloc(&std::alloc::System, l);
        if !p.is_null() {
            let _ = COUNTING.try_with(|c| {
                if c.get() {
                    ALLOCS.with(|a| a.set(a.get() + 1));
                    LIVE.with(|a| a.set(a.get() + l.size() as isize));
                }
            });
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        let _ = COUNTING.try_with(|c| {
            if c.get() {
                LIVE.with(|a| a.set(a.get() - l.size() as isize));
            }
        });
        std::alloc::GlobalAlloc::dealloc(&std::alloc::System, p, l);
    }
}
#[global_allocator]
static ALLOCATOR: Counted = Counted;
fn prepare_compact_library() {
    use kirby_assist_core::sequence_config::{Catalog, SequenceLibrary};
    SEQUENCES.get_or_init(|| {
        let source=[
            "[bowser.roll.roll]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[bowser.roll.spot]\nstart_delay=0\nreaction=auto_spot_dodge:25:10\nactions=up_b:4\n[bowser.roll.air]\nstart_delay=0\nreaction=auto_air_dodge:35:19\nactions=up_b:4\n[bowser.roll.dodge_grab]\nstart_delay=0\nreaction=auto_grab_dodged:25:10\nactions=up_b:4\n[yoshi.roll.custom_spot]\nstart_delay=0\nreaction=auto_spot_dodge:10:3\nactions=down_smash:1\n[mr_game_and_watch.settings]\nreflect_default=true\n",
            include_str!("../../../deployment/ultimate/kirby_assist/fighters/pyra.txt"),
            include_str!("../../../deployment/ultimate/kirby_assist/fighters/mythra.txt"),
            "\n[pyra.x.saved]\nstart_delay=0\ndefault=true\nactions=input:a:0:0:1\n[mythra.x.saved]\nstart_delay=0\ndefault=true\nactions=input:b:0:0:1\n",
        ].join("\n");
        Mutex::new(SequenceLibrary::from_catalog(Catalog::parse(&source).unwrap()))
    });
    PROFILES.get_or_init(FighterProfiles::default);
}

#[test]
fn fighter_change_never_inherits_bowser_actions_and_grab_dodge_cannot_borrow_another_event() {
    let _save_guard = live_menu::save_test_guard();
    use kirby_assist_core::native_input::{MAPPER_A, MAPPER_B};
    prepare_compact_library();
    let mut menu = live_menu::LedgeMenu::default();
    let registered = (0..0x5e).filter(|kind| PROFILES.get().unwrap().profile(*kind).is_some());
    for kind in [15, 22, 5, 22, 15, 5, 22]
        .into_iter()
        .chain(registered)
        .chain([15, 5, 22])
    {
        for other in [15, 22, 5] {
            if menu.for_fighter(other).is_some() {
                assert_eq!(
                    menu.for_fighter(other).unwrap().automatic_limits(),
                    menu.automatic_limits()
                );
            }
        }
        menu.input(
            true,
            true,
            false,
            false,
            false,
            Some(kind),
            0,
            0,
            ControlLayout::Default,
            &LedgeSettings::default(),
            true,
            true,
            true,
            false,
        );
        assert!(menu.for_fighter(kind).is_some());
        for other in [15, 22, 5].into_iter().filter(|other| *other != kind) {
            assert!(
                menu.for_fighter(other).is_none(),
                "native identity handoff must hide previous rules"
            );
        }
        match kind {
            15 => {
                let r = menu.rule_config().unwrap();
                for event in 0..3 {
                    assert!(matches!(
                        r.simple_sequence(event).unwrap().actions()[0],
                        kirby_assist_core::sequence_config::Action::Input {
                            buttons: MAPPER_B,
                            ..
                        }
                    ));
                }
                assert!(menu.grab_dodge_sequence().is_some());
            }
            5 => {
                let r = menu.rule_config().unwrap();
                assert!(r.simple_sequence(0).is_none() && r.simple_sequence(2).is_none());
                assert!(matches!(
                    r.simple_sequence(1).unwrap().actions()[0],
                    kirby_assist_core::sequence_config::Action::Input {
                        buttons: MAPPER_A,
                        ..
                    }
                ));
                assert!(
                    menu.grab_dodge_sequence().is_none(),
                    "own spot-dodge recipe is not a grab-dodge follow-up"
                );
            }
            _ => {
                assert!(menu.rule_config().is_none());
                assert!(menu.grab_dodge_sequence().is_none());
                assert!(menu.automatic_rules().is_none());
            }
        }
        assert!(
            menu.automatic_sequences().backup.is_none(),
            "no implicit shield backup for kind {kind}"
        );
    }
}

#[test]
fn pyra_mythra_swaps_reload_only_the_active_forms_menu_and_saved_inputs() {
    use kirby_assist_core::{
        fighter_profiles::presets::PresetSlot,
        native_input::{MAPPER_A, MAPPER_B},
        sequence_config::Action,
    };
    let _save_guard = live_menu::save_test_guard();
    prepare_compact_library();
    let mut menu = live_menu::LedgeMenu::default();
    for kind in [90, 91, 90, 15, 91, 90, 91] {
        let base = PROFILES
            .get()
            .unwrap()
            .profile(kind)
            .unwrap()
            .apply(LedgeSettings::default());
        menu.input(
            true,
            true,
            false,
            false,
            false,
            Some(kind),
            0,
            0,
            ControlLayout::Default,
            &base,
            true,
            true,
            true,
            false,
        );
        assert!(menu.for_fighter(kind).is_some());
        for old in [90, 91, 15].into_iter().filter(|old| *old != kind) {
            assert!(menu.for_fighter(old).is_none());
        }
        if matches!(kind, 90 | 91) {
            let current = menu.settings(Some(kind), &base);
            for slot in PresetSlot::ALL.into_iter().take(5) {
                let sequence =
                    current.sequence_overrides[slot as usize].expect("visible editable ledge slot");
                assert_eq!(sequence.actions(), &[Action::Wait(0)]);
            }
            let saved = current.sequence_overrides[PresetSlot::X as usize].unwrap();
            assert!(
                matches!(saved.actions()[0],Action::Input{buttons,..}if buttons==if kind==90{MAPPER_A}else{MAPPER_B})
            );
            assert!(menu.automatic_rules().is_none());
            assert!(menu.reflect_config(kind).is_none());
        }
    }
    menu.end_match();
    assert!(menu.for_fighter(90).is_none() && menu.for_fighter(91).is_none());
}

#[test]
fn production_reflector_configuration_tracks_only_the_current_fighter_and_live_scope() {
    let _save_guard = live_menu::save_test_guard();
    prepare_compact_library();
    let mut menu = live_menu::LedgeMenu::default();
    for (kind, gameplay) in [
        (28, true),
        (22, true),
        (28, true),
        (28, false),
        (15, true),
        (28, true),
    ] {
        menu.input(
            true,
            gameplay,
            false,
            false,
            false,
            Some(kind),
            0,
            0,
            ControlLayout::Default,
            &LedgeSettings::default(),
            true,
            true,
            true,
            false,
        );
        assert_eq!(menu.reflect_config(kind).is_some(), kind == 28 && gameplay);
        if kind != 28 {
            assert!(menu.reflect_config(28).is_none());
        }
    }
    menu.end_match();
    assert!(menu.reflect_config(28).is_none());
}

#[test]
fn compact_runtime_idle_has_no_allocations_and_fighter_switches_do_not_retain_old_menus() {
    let _guard = live_menu::save_test_guard();
    prepare_compact_library();
    let mut menu = live_menu::LedgeMenu::default();
    let sample = |m: &mut live_menu::LedgeMenu, kind| {
        m.input(
            true,
            true,
            false,
            false,
            false,
            Some(kind),
            0,
            0,
            ControlLayout::Default,
            &LedgeSettings::default(),
            true,
            true,
            true,
            false,
        );
    };
    for _ in 0..3 {
        sample(&mut menu, 15);
        sample(&mut menu, 22);
        sample(&mut menu, 5);
        sample(&mut menu, 15);
    }
    ALLOCS.with(|a| a.set(0));
    LIVE.with(|a| a.set(0));
    COUNTING.with(|c| c.set(true));
    for _ in 0..10000 {
        sample(&mut menu, 15);
    }
    COUNTING.with(|c| c.set(false));
    let allocs = ALLOCS.with(|a| a.get());
    assert_eq!(
        allocs, 0,
        "idle mapper/menu polling allocated {allocs} times"
    );
    LIVE.with(|a| a.set(0));
    COUNTING.with(|c| c.set(true));
    for _ in 0..100 {
        for kind in [22, 5, 22, 15] {
            sample(&mut menu, kind);
        }
    }
    COUNTING.with(|c| c.set(false));
    let retained = LIVE.with(|a| a.get());
    assert!(
        retained < 8192,
        "fighter changes retained {retained} extra bytes"
    );
    assert!(menu.for_fighter(15).is_some());
    assert!(menu.for_fighter(22).is_none());
    menu.end_match();
    assert!(menu.for_fighter(15).is_none());
    assert!(menu.automatic_rules().is_none());
}
