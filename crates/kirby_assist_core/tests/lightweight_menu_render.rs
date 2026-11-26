//! Exercise the production snapshot builder and draw calls on the host.
//! Only ImGui is replaced with a text/stack recorder; no Switch or mod build.
#![allow(
    dead_code,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unexpected_cfgs
)]
extern crate self as imgui_api;
use kirby_assist_core::{
    fighter_profiles::{FighterProfile, FighterProfiles},
    native_input::*,
    sequence_config::Catalog,
    settings_menu::{MenuRow, MenuSelection, MENU_STICK_CLICK},
};
use std::{
    cell::RefCell,
    ffi::{c_char, c_void, CStr},
};
#[path = "../../../src/live/settings_menu/view.rs"]
mod view;
use view::Snapshot;
#[path = "../../../src/live/settings_menu/lightweight.rs"]
mod lightweight;
#[derive(Default)]
struct Trace {
    text: Vec<String>,
    colors: i32,
    vars: i32,
    fonts: i32,
    wraps: i32,
    begins: usize,
    ends: usize,
    collapse: bool,
    flags: i32,
    width: f32,
    position: [f32; 2],
}
thread_local! {static TRACE:RefCell<Trace>=RefCell::new(Trace::default());}
fn record(f: impl FnOnce(&mut Trace)) {
    TRACE.with(|t| f(&mut t.borrow_mut()));
}
fn color(x: f32, y: f32, z: f32, w: f32) -> bindings::ImVec4_c {
    bindings::ImVec4_c { x, y, z, w }
}
pub mod bindings {
    use super::*;
    #[derive(Clone, Copy)]
    pub struct ImVec2_c {
        pub x: f32,
        pub y: f32,
    }
    #[derive(Clone, Copy)]
    pub struct ImVec4_c {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }
    pub const ImGuiCond_Always: i32 = 1;
    pub const ImGuiStyleVar_WindowPadding: i32 = 0;
    pub const ImGuiCol_Text: i32 = 0;
    pub const ImGuiCol_WindowBg: i32 = 1;
    pub const ImGuiWindowFlags_NoDecoration: i32 = 1;
    pub const ImGuiWindowFlags_NoScrollbar: i32 = 2;
    pub const ImGuiWindowFlags_AlwaysAutoResize: i32 = 4;
    pub const ImGuiWindowFlags_NoSavedSettings: i32 = 8;
    pub const ImGuiWindowFlags_NoInputs: i32 = 16;
    pub const ImGuiWindowFlags_NoFocusOnAppearing: i32 = 32;
    pub const ImGuiWindowFlags_NoBringToFrontOnFocus: i32 = 64;
    pub unsafe fn igSetNextWindowPos(position: ImVec2_c, _: i32, _: ImVec2_c) {
        record(|t| t.position = [position.x, position.y]);
    }
    pub unsafe fn igSetNextWindowSize(size: ImVec2_c, _: i32) {
        record(|t| t.width = size.x);
    }
    pub unsafe fn igSetNextWindowSizeConstraints(
        _: ImVec2_c,
        _: ImVec2_c,
        _: Option<unsafe extern "C" fn()>,
        _: *mut c_void,
    ) {
    }
    pub unsafe fn igPushStyleVar_Vec2(_: i32, _: ImVec2_c) {
        record(|t| t.vars += 1);
    }
    pub unsafe fn igPopStyleVar(n: i32) {
        record(|t| {
            t.vars -= n;
            assert!(t.vars >= 0);
        });
    }
    pub unsafe fn igPushStyleColor_Vec4(_: i32, _: ImVec4_c) {
        record(|t| t.colors += 1);
    }
    pub unsafe fn igPopStyleColor(n: i32) {
        record(|t| {
            t.colors -= n;
            assert!(t.colors >= 0);
        });
    }
    pub unsafe fn igPushFont(_: *mut c_void, _: f32) {
        record(|t| t.fonts += 1);
    }
    pub unsafe fn igPopFont() {
        record(|t| {
            t.fonts -= 1;
            assert!(t.fonts >= 0);
        });
    }
    pub unsafe fn igPushTextWrapPos(_: f32) {
        record(|t| t.wraps += 1);
    }
    pub unsafe fn igPopTextWrapPos() {
        record(|t| {
            t.wraps -= 1;
            assert!(t.wraps >= 0);
        });
    }
    pub unsafe fn igBegin(_: *const c_char, _: *mut bool, flags: i32) -> bool {
        let mut open = false;
        record(|t| {
            t.begins += 1;
            t.flags = flags;
            open = !t.collapse;
        });
        open
    }
    pub unsafe fn igEnd() {
        record(|t| t.ends += 1);
    }
    pub unsafe fn igSameLine(_: f32, _: f32) {}
    pub unsafe fn igTextUnformatted(text: *const c_char, _: *const c_char) {
        let s = CStr::from_ptr(text).to_str().unwrap().to_owned();
        record(|t| t.text.push(s));
    }
}
fn profiles() -> FighterProfiles {
    let source=Catalog::parse("[bowser.neutral.first]\nname=First recipe\nstart_delay=0\nactions=grab:4\n[bowser.neutral.second]\nname=Second recipe\nstart_delay=0\nactions=up_b:4\n").unwrap();
    let mut p = FighterProfiles::default();
    p.add_sequence_choices(&source).unwrap();
    p
}
fn input(m: &mut MenuSelection, p: &FighterProfile, buttons: u32, training: bool) {
    m.input_with_editing(buttons, &p.menu_options, training);
}
fn is_context(snapshot: &Snapshot, text: &str) -> bool {
    matches!(
        text,
        "MATCH / GAMEPLAY ACTIVE"
            | "Hold L + D-pad: select / change"
            | "Release L: gameplay | Minus: hide"
    ) || snapshot
        .row_context
        .iter()
        .any(|line| line.to_str().unwrap() == text)
}
fn draw(snapshot: &Snapshot, collapse: bool) -> Vec<String> {
    TRACE.with(|t| {
        *t.borrow_mut() = Trace {
            collapse,
            ..Default::default()
        }
    });
    unsafe {
        lightweight::draw(snapshot, bindings::ImVec2_c { x: 1280., y: 720. }, 1.);
    }
    TRACE.with(|t| {
        let t = t.borrow();
        assert_eq!((t.colors, t.vars, t.fonts, t.wraps), (0, 0, 0, 0));
        assert_eq!(t.begins, t.ends);
        t.text.clone()
    })
}
#[test]
fn actual_lightweight_renderer_emits_only_option_rows_on_all_three_pages() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    m.begin_match();
    input(&mut m, p, 0, false);
    for page in 0..3 {
        if page != 0 {
            input(&mut m, p, 0, false);
            input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
        }
        assert!(!m.menu_hidden && m.lightweight);
        assert_eq!(m.button_menu, page == 1);
        assert_eq!(m.modes_menu, page == 2);
        assert!(m.quick_menu);
        let snapshot = view::build(p, &m, false, 0).unwrap();
        assert!(snapshot.navigation.is_none());
        let label = snapshot
            .lightweight_label
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(label.starts_with(match page {
            0 => "LEDGE TRAP",
            1 => "BUTTON BINDINGS",
            _ => "ON/OFF SETTINGS",
        }));
        assert!(!label.contains("R3:"));
        assert!(label.contains("L: LEDGE TRAP"));
        assert!(
            snapshot.header.is_empty() && snapshot.body.is_empty() && snapshot.footer.is_empty()
        );
        let text = draw(&snapshot, false);
        assert!(!text.is_empty());
        assert!(text.contains(&label.to_owned()));
        for line in text
            .iter()
            .filter(|s| s.as_str() != ">" && s.as_str() != label && !is_context(&snapshot, s))
        {
            assert!(snapshot
                .values
                .iter()
                .any(|v| v.line.to_str().unwrap() == line));
            assert!(
                !line.contains("Up/Down:")
                    && !line.contains("Minus:")
                    && !line.contains("L+D-pad")
                    && !line.contains("bowser /")
            );
        }
        TRACE.with(|t| {
            let t = t.borrow();
            assert_eq!(t.width, 0.);
            assert_ne!(t.flags & bindings::ImGuiWindowFlags_NoInputs, 0);
        });
        assert!(
            draw(&snapshot, true).is_empty(),
            "collapsed windows still balance every ImGui stack"
        );
    }
}
#[test]
fn bounded_option_window_tracks_cursor_without_rebuilding_context() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, false);
    input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
    let mut snapshot = view::build(p, &m, false, 0).unwrap();
    assert!(snapshot.values.len() > 7);
    let context_ptr = snapshot.row_context[0].as_ptr();
    snapshot.row = snapshot.values.last().unwrap().row;
    let (start, end, focus) = snapshot.row_window();
    assert_eq!(end - start, 7);
    assert_eq!(end, snapshot.values.len());
    assert_eq!(focus, end - 1);
    let text = draw(&snapshot, false);
    assert!(text.contains(&snapshot.row_context[focus].to_str().unwrap().to_owned()));
    assert!(text.contains(&snapshot.values[focus].line.to_str().unwrap().to_owned()));
    assert!(!text.contains(&snapshot.values[0].line.to_str().unwrap().to_owned()));
    assert_eq!(snapshot.row_context[0].as_ptr(), context_ptr);
    assert!(text
        .iter()
        .all(|s| !s.contains("Release L:") && !s.contains("Hold L") && !s.contains("Minus:")));
}

#[test]
fn independent_battle_hud_draws_only_fixed_mode_text_and_balances_collapsed_windows() {
    for collapse in [false, true] {
        TRACE.with(|t| {
            *t.borrow_mut() = Trace {
                collapse,
                ..Default::default()
            }
        });
        unsafe {
            lightweight::draw_battle_hud(
                1,
                c"Spacing",
                (1.0, 0.73, 0.42),
                bindings::ImVec2_c { x: 1280., y: 720. },
                1.,
            );
        }
        TRACE.with(|t| {
            let t = t.borrow();
            assert_eq!(
                t.text,
                if collapse {
                    vec![]
                } else {
                    vec!["Spacing".to_owned()]
                }
            );
            assert_eq!((t.colors, t.vars, t.fonts, t.wraps), (0, 0, 0, 0));
            assert_eq!((t.begins, t.ends), (1, 1));
            assert!((t.position[0] - 1280.0 * 0.38).abs() < 0.01);
            assert!((t.position[1] - 720.0 * 0.92).abs() < 0.01);
            assert_ne!(t.flags & bindings::ImGuiWindowFlags_NoInputs, 0);
        });
    }
}

#[test]
fn training_help_is_present_once_without_losing_controls() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, true);
    input(&mut m, p, MAPPER_MINUS, true);
    m.set_training_category(2);
    m.quick_menu = false;
    m.modes_menu = false;
    m.row = MenuRow::AutoMash;
    let snapshot = view::build(p, &m, false, 0).unwrap();
    assert!(snapshot
        .body
        .iter()
        .any(|line| line.to_str().unwrap().contains("escape inputs")));
    assert!(!snapshot
        .header
        .iter()
        .any(|line| line.to_str().unwrap().starts_with("Help:")));
    assert!(snapshot
        .footer
        .iter()
        .any(|line| line.to_str().unwrap().contains("Minus: close")));
    assert_eq!(
        snapshot.categories.len(),
        MenuSelection::TRAINING_CATEGORIES.len()
    );
}

#[test]
fn training_categories_publish_only_their_options_and_restore_after_tab_focus() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, true);
    input(&mut m, p, MAPPER_MINUS, true);
    for category in 0..5 {
        m.set_training_category(category);
        let snapshot = view::build(p, &m, false, 0).unwrap();
        assert!(snapshot.training_categories_active && !snapshot.training_tab_focus);
        assert_eq!(snapshot.training_category, category);
        assert!(snapshot.lightweight_label.is_none());
        assert!(snapshot
            .navigation
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .contains(if category == 4 {
                "FIGHTER SETTINGS"
            } else {
                MenuSelection::TRAINING_CATEGORIES[category as usize]
            }));
        if category == 4 {
            assert!(snapshot.values.is_empty());
            assert!(!snapshot.body.is_empty());
        } else {
            assert!(!snapshot.values.is_empty());
            assert!(snapshot.values.iter().any(|value| value.row == m.row));
            assert!(snapshot.values.iter().all(|value| {
                m.display_rows().contains(&value.row) && !MenuSelection::page_link(value.row)
            }));
        }
        input(&mut m, p, 0, true);
        input(&mut m, p, kirby_assist_core::manual_controls::DPAD_UP, true);
        let tabs = view::build(p, &m, false, 0).unwrap();
        assert!(tabs.training_tab_focus && tabs.values.is_empty());
        assert!(tabs
            .body
            .iter()
            .any(|line| line.to_str().unwrap().contains("category")));
        input(&mut m, p, 0, true);
        input(&mut m, p, MAPPER_B, true);
        let restored = view::build(p, &m, false, 0).unwrap();
        assert!(!restored.training_tab_focus);
        assert_eq!(restored.values.len(), snapshot.values.len());
        assert_eq!(restored.mode, snapshot.mode);
    }
}

#[test]
fn reused_lightweight_state_starts_each_battle_visible_on_ledge_page_with_fresh_content() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, false);
    input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
    assert!(m.button_menu);
    let old = view::build(p, &m, false, 0).unwrap();
    input(&mut m, p, 0, false);
    input(&mut m, p, MAPPER_MINUS, false);
    assert!(m.menu_hidden);
    let content = m.content_revision;
    m.begin_match();
    input(&mut m, p, 0, false);
    assert!(!m.menu_hidden && !m.button_menu && m.quick_menu && m.lightweight);
    assert_eq!(m.row, MenuRow::Neutral);
    assert_ne!(m.content_revision, content);
    assert!(
        !old.reusable(p, &m, false, 0),
        "a reused auxiliary snapshot cannot survive battle entry"
    );
    let snapshot = view::build(p, &m, false, 0).unwrap();
    assert_eq!(snapshot.values.len(), 5);
    assert!(!draw(&snapshot, false).is_empty());
    input(&mut m, p, MAPPER_MINUS, false);
    assert!(m.menu_hidden);
    input(&mut m, p, 0, false);
    assert!(
        m.menu_hidden,
        "the default must not undo an explicit hide during this battle"
    );
    input(&mut m, p, MAPPER_MINUS, false);
    assert!(!m.menu_hidden);
    m.begin_match();
    input(&mut m, p, MAPPER_MINUS, false);
    assert!(
        !m.menu_hidden,
        "held Minus across entry must not synthesize a fresh close"
    );
}
#[test]
fn training_hints_stay_in_training_and_cached_training_cards_cannot_match_lightweight() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, true);
    assert!(m.menu_hidden);
    input(&mut m, p, MAPPER_MINUS, true);
    let training = view::build(p, &m, false, 0).unwrap();
    assert!(training.navigation.is_some());
    assert!(training
        .footer
        .iter()
        .any(|s| s.to_str().unwrap().contains("Minus: close")));
    assert!(m.waits_for_training_save(true, true));
    assert!(
        !m.waits_for_training_save(false, true),
        "save workers may not retain Training's input path outside Training"
    );
    input(&mut m, p, 0, false);
    assert!(m.lightweight && !m.menu_hidden);
    assert!(!training.matches_layout(true));
    assert!(draw(&training, false).is_empty());
    let light = view::build(p, &m, false, 0).unwrap();
    assert!(light.matches_layout(true));
    assert!(!light.matches_layout(false));
    m.begin_match();
    assert!(!m.waits_for_training_save(true, true));
    input(&mut m, p, 0, true);
    assert!(
        !m.lightweight && m.menu_hidden,
        "the next Training battle keeps its original hidden default"
    );
}
#[test]
fn row_navigation_reuses_text_but_page_mode_and_content_changes_invalidate_it() {
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    input(&mut m, p, 0, false);
    let mut snapshot = view::build(p, &m, false, 2).unwrap();
    input(
        &mut m,
        p,
        MAPPER_L | kirby_assist_core::ledge_trap::DPAD_DOWN,
        false,
    );
    let pointer = snapshot.values[0].line.as_ptr();
    assert_eq!(m.row, MenuRow::Jump);
    assert!(snapshot.refresh_cursor(p, &m, false, 2));
    assert_eq!(snapshot.values[0].line.as_ptr(), pointer);
    assert_eq!(snapshot.row, MenuRow::Jump);
    assert!(snapshot.matches_request(true, p.fighter_kind, m.revision, 2));
    assert!(!snapshot.matches_request(true, 22, m.revision, 2));
    assert!(!snapshot.matches_request(false, p.fighter_kind, m.revision, 2));
    assert!(!snapshot.matches_request(true, p.fighter_kind, m.revision + 1, 2));
    assert!(!snapshot.matches_request(true, p.fighter_kind, m.revision, 3));
    assert!(!snapshot.reusable(p, &m, true, 2));
    assert!(!snapshot.reusable(p, &m, false, 3));
    m.button_menu = true;
    assert!(
        !snapshot.reusable(p, &m, false, 2),
        "page identity is checked even if a caller missed a content revision"
    );
    m.button_menu = false;
    input(&mut m, p, 0, false);
    input(
        &mut m,
        p,
        MAPPER_L | kirby_assist_core::manual_controls::DPAD_UP,
        false,
    );
    assert_eq!(m.row, MenuRow::Neutral);
    input(&mut m, p, 0, false);
    input(
        &mut m,
        p,
        MAPPER_L | kirby_assist_core::manual_controls::DPAD_RIGHT,
        false,
    );
    assert!(
        !snapshot.reusable(p, &m, false, 2),
        "changed saved values rebuild cached text"
    );
    let changed = view::build(p, &m, false, 2).unwrap();
    assert!(draw(&changed, false)
        .iter()
        .any(|s| s.contains("Second recipe")));
}

#[test]
fn every_configured_fighter_renders_all_three_lightweight_pages_without_headers_hints_or_stack_errors(
) {
    let catalog = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let mut count = 0;
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(p) = profiles.profile(kind) else {
            continue;
        };
        count += 1;
        let mut m = MenuSelection::new(&p.menu_options);
        m.y_fighter_kind = kind;
        m.simplified_reactions = true;
        m.begin_match();
        input(&mut m, p, 0, false);
        for page in 0..3 {
            if page != 0 {
                input(&mut m, p, 0, false);
                input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
            }
            assert_eq!(m.button_menu, page == 1);
            assert_eq!(m.modes_menu, page == 2);
            let snapshot = view::build(p, &m, false, 0).unwrap();
            assert!(snapshot.navigation.is_none());
            let label = snapshot
                .lightweight_label
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap();
            assert!(label.starts_with(match page {
                0 => "LEDGE TRAP",
                1 => "BUTTON BINDINGS",
                _ => "ON/OFF SETTINGS",
            }));
            assert!(!label.contains("R3:"));
            assert!(label.contains("L: LEDGE TRAP"));
            assert!(
                snapshot.header.is_empty()
                    && snapshot.body.is_empty()
                    && snapshot.footer.is_empty()
            );
            assert!(m.lightweight && !m.menu_hidden && !m.recipe_editing && !m.managing());
            let rendered = draw(&snapshot, false);
            assert!(rendered.contains(&label.to_owned()));
            for line in rendered
                .iter()
                .filter(|s| s.as_str() != ">" && *s != label && !is_context(&snapshot, s))
            {
                assert!(
                    snapshot
                        .values
                        .iter()
                        .any(|v| v.line.to_str().unwrap() == line),
                    "unexpected text for {kind}: {line}"
                );
            }
        }
    }
    assert_eq!(count, 90);
}

// Compile the native publisher itself, including the logging-only branches.
mod save {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    };
    pub static BUSY: AtomicBool = AtomicBool::new(false);
    static GATE: Mutex<()> = Mutex::new(());
    pub fn busy() -> bool {
        BUSY.load(Ordering::Acquire)
    }
    pub fn try_ui_edit() -> Option<MutexGuard<'static, ()>> {
        if BUSY.load(Ordering::Acquire) {
            None
        } else {
            GATE.try_lock().ok()
        }
    }
}
#[cfg(feature = "diagnostic-logging")]
mod performance {
    pub mod health {
        pub enum Event {
            MenuPublish,
            MenuBuild,
            MenuRetry,
        }
        pub enum Lane {
            MenuBuild,
        }
        pub struct Trace;
        impl Trace {
            pub fn new(_: Lane) -> Self {
                Self
            }
        }
        pub fn event(_: Event) {}
        pub fn view_bytes(_: usize) {}
    }
}
mod publisher {
    #[cfg(feature = "diagnostic-logging")]
    use super::view::SavedChoice;
    use super::*;
    #[cfg(feature = "diagnostic-logging")]
    use std::ffi::CString;
    use std::sync::{
        atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering},
        Mutex,
    };
    static TEST: Mutex<()> = Mutex::new(());
    static BATTLE_HUD: view::BattleHud = view::BattleHud::new();
    static VIEW: Mutex<Option<Snapshot>> = Mutex::new(None);
    static VIEW_GENERATION: AtomicU64 = AtomicU64::new(0);
    static DEFERRED_CLEAR: AtomicBool = AtomicBool::new(false);
    static MENU_VISIBLE: AtomicBool = AtomicBool::new(false);
    static LIGHTWEIGHT_REQUESTED: AtomicBool = AtomicBool::new(true);
    static REQUESTED_FIGHTER: AtomicI32 = AtomicI32::new(-1);
    static REQUESTED_REVISION: AtomicU64 = AtomicU64::new(0);
    fn finish_deferred_clear() {
        let Some(_gate) = super::save::try_ui_edit() else {
            return;
        };
        if let Ok(mut view) = VIEW.try_lock() {
            view.take();
            DEFERRED_CLEAR.store(false, Ordering::Release);
        }
    }
    include!("../../../src/live/settings_menu/publish.rs");
    #[test]
    fn hidden_mode_publication_does_not_wait_for_cached_menu_or_depend_on_l_hold() {
        let _test = TEST.lock().unwrap();
        use kirby_assist_core::{button_bindings::Builtin, fighter_profiles::presets::PresetSlot};
        let profiles = profiles();
        let p = profiles.profile(15).unwrap();
        let mut m = MenuSelection::new(&p.menu_options);
        m.menu_hidden = true;
        m.choices[PresetSlot::DpadUp as usize] =
            p.menu_options.list(PresetSlot::DpadUp).options.len()
                + Builtin::LModeSwap.visible_template_offset().unwrap();
        m.input(0, &p.menu_options);
        let lock = VIEW.lock().unwrap();
        publish_hud(&m, 2);
        let (_, first, _) = BATTLE_HUD.current().unwrap();
        assert_eq!(first.to_str().unwrap(), "Ledge");
        m.input(kirby_assist_core::manual_controls::DPAD_UP, &p.menu_options);
        assert!(m.l_spacing_selected());
        publish_hud(&m, 2);
        let (player, label, _) = BATTLE_HUD.current().unwrap();
        assert_eq!(player, 2);
        assert_eq!(label.to_str().unwrap(), "Spacing");
        m.input(MAPPER_L, &p.menu_options);
        publish_hud(&m, 2);
        assert_eq!(BATTLE_HUD.current().unwrap().1.as_ptr(), label.as_ptr());
        m.input(0, &p.menu_options);
        publish_hud(&m, 2);
        assert_eq!(BATTLE_HUD.current().unwrap().1.as_ptr(), label.as_ptr());
        BATTLE_HUD.clear();
        assert!(BATTLE_HUD.current().is_none());
        let next = MenuSelection::new(&p.menu_options);
        publish_hud(&next, 0);
        assert_eq!(BATTLE_HUD.current().unwrap().0, 0);
        assert_eq!(BATTLE_HUD.current().unwrap().1.to_str().unwrap(), "Ledge");
        BATTLE_HUD.clear();
        drop(lock);
    }
    #[test]
    fn actual_publisher_builds_refreshes_and_retries_without_missing_scope_or_blocking() {
        let _test = TEST.lock().unwrap();
        let profiles = profiles();
        let p = profiles.profile(15).unwrap();
        let mut m = MenuSelection::new(&p.menu_options);
        input(&mut m, p, 0, false);
        assert!(publish(p, &m, false));
        let pointer = VIEW.lock().unwrap().as_ref().unwrap().values[0]
            .line
            .as_ptr();
        super::save::BUSY.store(true, Ordering::Release);
        input(
            &mut m,
            p,
            MAPPER_L | kirby_assist_core::ledge_trap::DPAD_DOWN,
            false,
        );
        assert!(
            publish(p, &m, false),
            "cursor refresh must work without replacing strings while save is busy"
        );
        {
            let view = VIEW.lock().unwrap();
            let v = view.as_ref().unwrap();
            assert_eq!(v.values[0].line.as_ptr(), pointer);
            assert_eq!(v.row, m.row);
        }
        input(&mut m, p, 0, false);
        input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
        assert!(
            !publish(p, &m, false),
            "content replacement must respect the save gate"
        );
        {
            let view = VIEW.lock().unwrap();
            assert!(!view
                .as_ref()
                .unwrap()
                .matches_request(true, p.fighter_kind, m.revision, 0));
        }
        super::save::BUSY.store(false, Ordering::Release);
        assert!(publish(p, &m, false));
        {
            let locked = VIEW.lock().unwrap();
            assert!(
                !publish(p, &m, false),
                "a held snapshot lock must not block the input thread"
            );
            drop(locked);
        }
        assert!(publish(p, &m, false));
        input(&mut m, p, 0, false);
        input(&mut m, p, MAPPER_MINUS, false);
        assert!(publish(p, &m, false));
        assert!(!MENU_VISIBLE.load(Ordering::Acquire));
        assert!(VIEW.lock().unwrap().is_none());

        input(&mut m, p, 0, true);
        input(&mut m, p, MAPPER_MINUS, true);
        assert!(!m.lightweight && !m.menu_hidden);
        assert!(publish(p, &m, false));
        super::save::BUSY.store(true, Ordering::Release);
        m.save_status = Some("Saving now; keep Smash open until complete".into());
        m.revision += 1;
        assert!(!publish(p, &m, false));
        {
            let locked = VIEW.lock().unwrap();
            let v = locked.as_ref().unwrap();
            assert!(
                v.matches_request(
                    false,
                    REQUESTED_FIGHTER.load(Ordering::Acquire),
                    REQUESTED_REVISION.load(Ordering::Acquire),
                    VIEW_GENERATION.load(Ordering::Acquire),
                ),
                "an active Training save must retain its last published card"
            );
        }
        input(&mut m, p, 0, false);
        assert!(!publish(p, &m, false));
        {
            let locked = VIEW.lock().unwrap();
            assert!(
                !locked.as_ref().unwrap().matches_request(
                    true,
                    p.fighter_kind,
                    REQUESTED_REVISION.load(Ordering::Acquire),
                    0,
                ),
                "save-time retention must not leak Training cards into matches"
            );
        }
        super::save::BUSY.store(false, Ordering::Release);
        assert!(publish(p, &m, false));
    }
}

#[test]
fn damage_toggles_are_independent_persisted_and_invalidate_cached_rows() {
    use kirby_assist_core::{
        fighter_tuning::{AUTO_DI_DEFAULT, AUTO_SDI_DEFAULT},
        sequence_config::persist,
    };
    let profiles = profiles();
    let p = profiles.profile(15).unwrap();
    let mut m = MenuSelection::new(&p.menu_options);
    m.begin_match();
    input(&mut m, p, 0, false);
    input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
    input(&mut m, p, 0, false);
    input(&mut m, p, MAPPER_L | MENU_STICK_CLICK, false);
    assert!(m.modes_menu);
    assert_eq!(m.damage_sticks(), [false; 2]);
    let before = view::build(p, &m, false, 0).unwrap();
    for row in [MenuRow::AutoDi, MenuRow::AutoSdi] {
        assert_eq!(
            before.values.iter().find(|v| v.row == row).unwrap().enabled,
            Some(false)
        );
    }
    m.row = MenuRow::AutoDi;
    input(&mut m, p, 0, false);
    input(
        &mut m,
        p,
        MAPPER_L | kirby_assist_core::manual_controls::DPAD_RIGHT,
        false,
    );
    assert_eq!(m.damage_sticks(), [true, false]);
    assert!(!before.reusable(p, &m, false, 0));
    let after = view::build(p, &m, false, 0).unwrap();
    assert_eq!(
        after
            .values
            .iter()
            .find(|v| v.row == MenuRow::AutoDi)
            .unwrap()
            .enabled,
        Some(true)
    );
    m.row = MenuRow::AutoSdi;
    input(&mut m, p, 0, false);
    input(
        &mut m,
        p,
        MAPPER_L | kirby_assist_core::manual_controls::DPAD_RIGHT,
        false,
    );
    assert_eq!(m.damage_sticks(), [true, true]);
    m.tuning.0[AUTO_DI_DEFAULT] = Some(0.);
    m.tuning_values[AUTO_DI_DEFAULT] = 1.;
    assert_eq!(
        m.damage_sticks(),
        [false, true],
        "saved OFF overrides startup ON"
    );
    let source = "[universal.settings]\nauto_di=true\n[bowser.neutral.keep]\nstart_delay=0\nactions=grab:4\n";
    let saved = persist::update_tuning(source, 15, m.tuning).unwrap();
    assert!(saved.contains("[universal.settings]\nauto_di=true"));
    assert!(saved.contains("[bowser.neutral.keep]\nstart_delay=0\nactions=grab:4"));
    let parsed = Catalog::parse(&saved).unwrap();
    let tuning = parsed.tuning.get(&15).unwrap();
    assert_eq!(tuning.0[AUTO_DI_DEFAULT], Some(0.));
    assert_eq!(tuning.0[AUTO_SDI_DEFAULT], Some(1.));
    assert!(Catalog::parse("[bowser.settings]\nauto_di_default=1\n").is_err());
}
