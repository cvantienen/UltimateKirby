//! A static saved-configuration selector. Its snapshot has only row labels and
//! saved names: no fighter status, motion or fighter data.
//! Strings are rebuilt only when a selection/row changes. ImGui still draws the
//! cached panel each render frame, as an immediate-mode GUI normally does.
use imgui_api::bindings::*;
use kirby_assist_core::{
    fighter_profiles::FighterProfile,
    native_input::HANDHELD_NPAD_ID,
    session::MenuPresence,
    settings_menu::{MenuRow, MenuSelection},
};
use std::{
    ffi::{CStr, CString},
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

#[path = "view.rs"]
mod view;
#[cfg(feature = "diagnostic-logging")]
use view::SavedChoice;
use view::Snapshot;
#[path = "lightweight.rs"]
mod lightweight;

static VIEW: Mutex<Option<Snapshot>> = Mutex::new(None);
static DEFERRED_CLEAR: AtomicBool = AtomicBool::new(false);
static VIEW_GENERATION: AtomicU64 = AtomicU64::new(0);

static LAST_FOCUS: Mutex<Option<(i32, MenuRow, usize, u8, Option<usize>, u64)>> = Mutex::new(None);
static PRESENCE: Mutex<MenuPresence> = Mutex::new(MenuPresence::new());
static MASTER_TOAST: Mutex<Option<(bool, Instant)>> = Mutex::new(None);
// A hidden/inactive toast needs no mutex on every shared-renderer callback.
static MASTER_TOAST_PENDING: AtomicBool = AtomicBool::new(false);
const MASTER_TOAST_LIFETIME: Duration = Duration::from_millis(1800);
pub(super) fn show_master_status(enabled: bool) {
    if let Ok(mut toast) = MASTER_TOAST.try_lock() {
        *toast = Some((enabled, Instant::now()));
        MASTER_TOAST_PENDING.store(true, Ordering::Release);
    }
}

static MASTER_GESTURE: Mutex<kirby_assist_core::runtime_toggle::RuntimeToggle> =
    Mutex::new(kirby_assist_core::runtime_toggle::RuntimeToggle::new());
static MASTER_CLOCK: Mutex<Option<Instant>> = Mutex::new(None);
static MASTER_PROBE_TICK: AtomicU64 = AtomicU64::new(u64::MAX);

const NPAD_STYLE_FULL_KEY: u32 = 0x1;
const NPAD_STYLE_HANDHELD: u32 = 0x2;
const NPAD_STYLE_JOY_DUAL: u32 = 0x4;
const NPAD_STYLE_JOY_LEFT: u32 = 0x8;
const NPAD_STYLE_JOY_RIGHT: u32 = 0x10;
const NPAD_STYLE_GAMECUBE: u32 = 0x20;

unsafe fn selected_hid_buttons() -> Option<u64> {
    let player = super::super::CONFIG.get()?.player_index;
    let ids = [
        player,
        if player == 0 {
            HANDHELD_NPAD_ID
        } else {
            u32::MAX
        },
    ];
    for id in ids {
        if id == u32::MAX {
            continue;
        }
        let style = nnsdk::nn::hid::GetNpadStyleSet(&id).flags;
        if style == 0 {
            continue;
        }
        if style & NPAD_STYLE_GAMECUBE != 0 {
            let mut state = nnsdk::nn::hid::NpadGcState::default();
            nnsdk::nn::hid::GetNpadGcState(&mut state, &id);
            return Some(state.Buttons);
        }
        let mut state = nnsdk::nn::hid::NpadHandheldState::default();
        if style & NPAD_STYLE_FULL_KEY != 0 {
            nnsdk::nn::hid::GetNpadFullKeyState(&mut state, &id);
        } else if style & NPAD_STYLE_HANDHELD != 0 {
            nnsdk::nn::hid::GetNpadHandheldState(&mut state, &id);
        } else if style & NPAD_STYLE_JOY_DUAL != 0 {
            nnsdk::nn::hid::GetNpadJoyDualState(&mut state, &id);
        } else if style & NPAD_STYLE_JOY_LEFT != 0 {
            nnsdk::nn::hid::GetNpadJoyLeftState(&mut state, &id);
        } else if style & NPAD_STYLE_JOY_RIGHT != 0 {
            nnsdk::nn::hid::GetNpadJoyRightState(&mut state, &id);
        } else {
            continue;
        }
        return Some(state.Buttons);
    }
    None
}

fn disarm_master_gesture() {
    if let Ok(mut gesture) = MASTER_GESTURE.try_lock() {
        gesture.disarm();
    }
}
unsafe fn refresh_match_lock() {
    super::super::refresh_master_match_lock(super::super::observations::battle_menu_ready_go());
}

unsafe fn poll_master_switch() {
    if !super::super::CONFIG
        .get()
        .is_some_and(|config| config.controls.enabled)
    {
        return;
    }
    super::super::MASTER_PULSE_REQUESTED.store(true, Ordering::Relaxed);
    let elapsed = {
        let Ok(mut clock) = MASTER_CLOCK.try_lock() else {
            return;
        };
        clock.get_or_insert_with(Instant::now).elapsed()
    };
    let probe_tick = (elapsed.as_millis() / 250) as u64;
    let periodic_probe = MASTER_PROBE_TICK.swap(probe_tick, Ordering::Relaxed) != probe_tick;
    // During a locked match the master gesture cannot activate. Avoid native
    // HID reads on repeated render callbacks; retain the existing 4Hz unlock
    // probe so leaving battle remains observable without mapper callbacks.
    if super::super::MASTER_IN_MATCH.load(Ordering::Acquire) {
        if periodic_probe {
            refresh_match_lock();
        }
        if super::super::MASTER_IN_MATCH.load(Ordering::Acquire) {
            disarm_master_gesture();
            return;
        }
    }
    let Some(buttons) = selected_hid_buttons() else {
        disarm_master_gesture();
        return;
    };
    if buttons & kirby_assist_core::runtime_toggle::CHORD
        == kirby_assist_core::runtime_toggle::CHORD
    {
        refresh_match_lock();
    }
    let blocked = super::super::MASTER_IN_MATCH.load(Ordering::Acquire);
    let toggled = MASTER_GESTURE
        .try_lock()
        .is_ok_and(|mut gesture| gesture.sample(buttons, blocked, elapsed));
    if toggled {
        super::super::set_runtime_enabled(!super::super::RUNTIME_ENABLED.load(Ordering::Acquire));
    }
}
pub(super) fn observe_local_fighter(now: Instant) {
    if let Ok(mut presence) = PRESENCE.try_lock() {
        presence.observe(now);
    }
}
// Display lifetime follows the battle, independently of expiring attack
// requests, hitstop, death and respawn. Master-switch probes above are separate
// read-only manager checks; menu rendering does not query fighter state.
static BATTLE_ACTIVE: AtomicBool = AtomicBool::new(false);
// Visibility must not depend on acquiring the cached-text mutex. A close
// request can arrive while ImGui is drawing the previous snapshot.
static MENU_VISIBLE: AtomicBool = AtomicBool::new(false);
static LIGHTWEIGHT_REQUESTED: AtomicBool = AtomicBool::new(true);
static REQUESTED_FIGHTER: AtomicI32 = AtomicI32::new(-1);
static REQUESTED_REVISION: AtomicU64 = AtomicU64::new(0);
// The installed host supplies its context through this setup callback. Older
// compatible hosts do not export imgui_get_context_export; importing that
// optional getter would stop Skyline from loading the entire plugin.
static CONTEXT: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());
pub(super) fn set_battle_active(active: bool) {
    let was_active = BATTLE_ACTIVE.swap(active, Ordering::AcqRel);
    #[cfg(feature = "diagnostic-logging")]
    if was_active && !active {
        crate::calibration::clear();
        crate::logging::clear_snapshot();
    }
    #[cfg(not(feature = "diagnostic-logging"))]
    let _ = was_active;
}
pub(super) fn clear() {
    VIEW_GENERATION.fetch_add(1, Ordering::AcqRel);
    MENU_VISIBLE.store(false, Ordering::Release);
    REQUESTED_FIGHTER.store(-1, Ordering::Release);
    set_battle_active(false);
    if let Ok(mut presence) = PRESENCE.try_lock() {
        presence.clear();
    }
    DEFERRED_CLEAR.store(true, Ordering::Release);
    finish_deferred_clear();
}
pub(super) fn finish_deferred_clear() {
    if !DEFERRED_CLEAR.load(Ordering::Acquire) {
        return;
    }
    let Some(_save_gate) = super::save::try_ui_edit() else {
        return;
    };
    if let Ok(mut view) = VIEW.try_lock() {
        // Take first so the shared slot is empty before owned text is freed.
        let old = view.take();
        DEFERRED_CLEAR.store(false, Ordering::Release);
        drop(view);
        drop(old);
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::health::view_bytes(0);
    }
}

include!("publish.rs");

fn color(r: f32, g: f32, b: f32, a: f32) -> ImVec4_c {
    ImVec4_c {
        x: r,
        y: g,
        z: b,
        w: a,
    }
}
unsafe fn card_text(value: &CStr) {
    // Native wrapping advances layout by the same height that gets drawn.
    igPushTextWrapPos(0.0);
    igTextUnformatted(value.as_ptr(), std::ptr::null());
    igPopTextWrapPos();
}
unsafe extern "C" fn setup(context: *mut u64) {
    CONTEXT.store(context, Ordering::Release);
    if !context.is_null() {
        igSetCurrentContext(context.cast());
    }
}
/// Latest master-switch confirmation. Expiry clears the pending flag under the
/// same lock as show_master_status, so it cannot discard a newly published toast.
fn current_master_status(now: Instant) -> Option<bool> {
    if !MASTER_TOAST_PENDING.load(Ordering::Acquire) {
        return None;
    }
    MASTER_TOAST.try_lock().ok().and_then(|mut toast| {
        let active = toast
            .as_ref()
            .copied()
            .filter(|(_, shown)| now.duration_since(*shown) < MASTER_TOAST_LIFETIME);
        if active.is_none() {
            *toast = None;
            MASTER_TOAST_PENDING.store(false, Ordering::Release);
        }
        active.map(|(enabled, _)| enabled)
    })
}
/// `None` while the presence lock is contended: callers keep their current state.
fn presence_fresh(now: Instant) -> Option<bool> {
    PRESENCE.try_lock().ok().map(|presence| presence.fresh(now))
}
/// Display size once the host context is usable; `None` skips this frame.
unsafe fn display_size() -> Option<ImVec2_c> {
    let context = CONTEXT.load(Ordering::Acquire);
    if context.is_null() {
        return None;
    }
    igSetCurrentContext(context.cast());
    let io = igGetIO();
    if io.is_null() {
        return None;
    }
    let display = (*io).DisplaySize;
    (display.x.is_finite() && display.y.is_finite() && display.x > 0.0 && display.y > 0.0)
        .then_some(display)
}
unsafe extern "C" fn draw() {
    let now = Instant::now();
    if BATTLE_ACTIVE.load(Ordering::Acquire) && presence_fresh(now) == Some(false) {
        set_battle_active(false);
    }
    poll_master_switch();
    let active = super::super::feature_activity_allowed();
    #[cfg(feature = "diagnostic-logging")]
    let _draw_time = active
        .then(|| crate::performance::health::Trace::new(crate::performance::health::Lane::Draw));
    #[cfg(feature = "diagnostic-logging")]
    if active {
        crate::performance::health::draw_tick(now);
    }
    // Profile/editor cleanup and save completion run on the existing mapper
    // callback. Rendering never applies edits or destroys the fighter roster.
    let toast = current_master_status(now);
    let battle_active = BATTLE_ACTIVE.load(Ordering::Acquire);
    // There is no dependable character-select-only scene signal in this lightweight UI
    // callback. Show confirmation only after the chord instead of leaving a
    // label across unrelated menus and single-player screens.
    let status = toast.filter(|_| !super::super::MASTER_IN_MATCH.load(Ordering::Acquire) || active);
    let menu_requested = battle_active && MENU_VISIBLE.load(Ordering::Acquire) && active;
    if status.is_none() && !menu_requested {
        return;
    }
    let Some(display) = display_size() else {
        return;
    };
    let scale = (display.y / 720.0).clamp(0.65, 3.0);
    if let Some(enabled) = status {
        draw_master_toast(scale, enabled, display);
    }
    if !menu_requested {
        return;
    }
    // Only recent battle presence can display the menu.
    match presence_fresh(now) {
        Some(true) => {}
        Some(false) => {
            set_battle_active(false);
            return;
        }
        None => return,
    }
    let Ok(view_guard) = VIEW.try_lock() else {
        return;
    };
    let Some(view) = view_guard.as_ref() else {
        return;
    };
    // A busy save may postpone snapshot replacement during a scene switch.
    // Never display a cached Training card in the match-only layout.
    let lightweight = LIGHTWEIGHT_REQUESTED.load(Ordering::Acquire);
    if !view.matches_request(
        lightweight,
        REQUESTED_FIGHTER.load(Ordering::Acquire),
        REQUESTED_REVISION.load(Ordering::Acquire),
        VIEW_GENERATION.load(Ordering::Acquire),
    ) {
        return;
    }
    if lightweight {
        lightweight::draw(view, display, scale);
        return;
    }
    draw_card(view, display, scale, status.is_some());
}

/// Window flags shared by every overlay: never focused, clickable or persisted.
const OVERLAY_FLAGS: i32 = (ImGuiWindowFlags_NoDecoration
    | ImGuiWindowFlags_NoScrollbar
    | ImGuiWindowFlags_AlwaysAutoResize
    | ImGuiWindowFlags_NoSavedSettings
    | ImGuiWindowFlags_NoInputs
    | ImGuiWindowFlags_NoFocusOnAppearing
    | ImGuiWindowFlags_NoBringToFrontOnFocus) as i32;
fn accent() -> ImVec4_c {
    color(0.35, 0.86, 0.78, 1.0)
}
fn row_highlight() -> ImVec4_c {
    color(0.17, 0.54, 0.50, 0.19)
}

unsafe fn draw_card(view: &Snapshot, display: ImVec2_c, scale: f32, status_shown: bool) {
    // A fixed-width card shows only the focused row; never shrink text to fit
    // the entire catalog or horizontally scroll the gameplay overlay.
    let gap = 5.0 * scale;
    let compact = view.quick || view.modes;
    let width = (if compact { 390.0 } else { 560.0 } * scale)
        .min(display.x * if compact { 0.44 } else { 0.49 })
        .min((display.x - 16.0 * scale).max(1.0));
    igPushFont(std::ptr::null_mut(), 15.0 * scale);
    for (slot, value) in [
        (ImGuiCol_WindowBg, color(0.035, 0.047, 0.070, 0.96)),
        (ImGuiCol_Border, color(0.19, 0.25, 0.32, 0.70)),
        (ImGuiCol_Text, color(0.86, 0.90, 0.95, 1.0)),
    ] {
        igPushStyleColor_Vec4(slot as _, value);
    }
    igPushStyleVar_Float(ImGuiStyleVar_WindowBorderSize as _, 1.0 * scale);
    igPushStyleVar_Float(ImGuiStyleVar_WindowRounding as _, 10.0 * scale);
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 12.0 * scale,
            y: 10.0 * scale,
        },
    );
    igPushStyleVar_Vec2(
        ImGuiStyleVar_ItemSpacing as _,
        ImVec2_c {
            x: gap,
            y: 5.0 * scale,
        },
    );
    // Anchor by the window's corner so auto-sized pages grow into the screen.
    let (menu_position, menu_pivot) = if cfg!(feature = "diagnostic-logging") {
        (
            ImVec2_c {
                x: display.x - 8.0 * scale,
                y: 8.0 * scale,
            },
            ImVec2_c { x: 1.0, y: 0.0 },
        )
    } else {
        (
            ImVec2_c {
                x: 8.0 * scale,
                y: 8.0 * scale,
            },
            ImVec2_c { x: 0.0, y: 0.0 },
        )
    };
    igSetNextWindowPos(menu_position, ImGuiCond_Always as _, menu_pivot);
    igSetNextWindowSize(ImVec2_c { x: width, y: 0.0 }, ImGuiCond_Always as _);
    if igBegin(
        c"##saved_options".as_ptr(),
        std::ptr::null_mut(),
        OVERLAY_FLAGS,
    ) {
        draw_card_contents(view, display, scale);
    }
    #[cfg(feature = "diagnostic-logging")]
    let result_space = kirby_assist_core::diagnostics::modes::left_result_space(
        display.y - if status_shown { 24.0 * scale } else { 0.0 },
        scale,
        igGetWindowHeight() + 8.0 * scale,
    );
    #[cfg(not(feature = "diagnostic-logging"))]
    let _ = status_shown;
    igEnd();
    #[cfg(feature = "diagnostic-logging")]
    if let Some((result_y, available)) = result_space {
        draw_result(scale, width, display.x - 8.0 * scale, result_y, available);
    }
    igPopStyleVar(4);
    igPopStyleColor(3);
    igPopFont();
}

unsafe fn draw_card_contents(view: &Snapshot, display: ImVec2_c, scale: f32) {
    igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.58, 0.69, 0.79, 1.0));
    if let Some(navigation) = &view.navigation {
        card_text(navigation);
    }
    igPopStyleColor(1);
    igSeparator();
    draw_value_rows(view, scale);
    for line in &view.header {
        igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.60, 0.84, 0.94, 1.0));
        card_text(line);
        igPopStyleColor(1);
    }
    if !view.body.is_empty() {
        draw_recipe_body(view, display, scale);
    }
    for line in &view.footer {
        igPushStyleColor_Vec4(ImGuiCol_Text as _, footer_tint(line));
        card_text(line);
        igPopStyleColor(1);
    }
}

fn footer_tint(line: &CStr) -> ImVec4_c {
    let bytes = line.to_bytes();
    if bytes.starts_with(b"Save failed") {
        color(1.0, 0.55, 0.45, 1.0)
    } else if bytes.starts_with(b"Saved") {
        accent()
    } else if bytes.starts_with(b"Unsaved") {
        color(0.96, 0.77, 0.42, 1.0)
    } else {
        color(0.72, 0.78, 0.87, 1.0)
    }
}

unsafe fn draw_value_rows(view: &Snapshot, scale: f32) {
    for value in view
        .values
        .iter()
        .filter(|v| view.quick || view.modes || v.row == view.row)
    {
        if MenuSelection::page_link(value.row) {
            igSeparator();
        }
        let selected = value.row == view.row;
        if selected {
            draw_selected_row_backdrop(&value.line, scale);
        }
        let tint = if selected {
            accent()
        } else {
            match value.enabled {
                Some(true) => color(0.50, 0.82, 0.70, 1.0),
                Some(false) => color(0.54, 0.61, 0.69, 1.0),
                None => color(0.86, 0.90, 0.95, 1.0),
            }
        };
        igPushStyleColor_Vec4(ImGuiCol_Text as _, tint);
        card_text(&value.line);
        igPopStyleColor(1);
    }
}

/// Tinted panel plus accent bar behind the focused row, sized to its wrapped text.
unsafe fn draw_selected_row_backdrop(line: &CStr, scale: f32) {
    let p = igGetCursorScreenPos();
    let available = igGetContentRegionAvail().x.max(1.0);
    let height = igCalcTextSize(line.as_ptr(), std::ptr::null(), false, available).y;
    let draw = igGetWindowDrawList();
    ImDrawList_AddRectFilled(
        draw,
        ImVec2_c {
            x: p.x - 5.0 * scale,
            y: p.y - 2.0 * scale,
        },
        ImVec2_c {
            x: p.x + available + 5.0 * scale,
            y: p.y + height + 2.0 * scale,
        },
        igGetColorU32_Vec4(row_highlight()),
        5.0 * scale,
        0 as _,
    );
    ImDrawList_AddRectFilled(
        draw,
        ImVec2_c {
            x: p.x - 7.0 * scale,
            y: p.y - 1.0 * scale,
        },
        ImVec2_c {
            x: p.x - 5.0 * scale,
            y: p.y + height + 1.0 * scale,
        },
        igGetColorU32_Vec4(accent()),
        1.0 * scale,
        0 as _,
    );
}

/// Numbered editor lines ("01 ...", optionally after the "> " cursor) start a new group.
fn is_recipe_title(line: &CStr) -> bool {
    let b = line.to_bytes();
    let b = b.strip_prefix(b"> ").unwrap_or(b);
    b.len() > 2 && b[0].is_ascii_digit() && b[1].is_ascii_digit()
}

/// The scrolling recipe-action list. Its height is whatever the footer leaves
/// of the screen budget, and it follows the focused line only when focus moved.
unsafe fn draw_recipe_body(view: &Snapshot, display: ImVec2_c, scale: f32) {
    igSeparator();
    let wrap = (igGetContentRegionAvail().x - 8.0 * scale).max(1.0);
    let measure = |line: &CString| {
        igCalcTextSize(line.as_ptr(), std::ptr::null(), false, wrap).y + 5.0 * scale
    };
    let footer_height = view.footer.iter().map(&measure).sum::<f32>() + 12.0 * scale;
    let content_height = view
        .body
        .iter()
        .map(|line| {
            measure(line)
                + if is_recipe_title(line) {
                    7.0 * scale
                } else {
                    0.0
                }
        })
        .sum::<f32>()
        + 8.0 * scale;
    // Reserve room below the diagnostic editor for its cached logs.
    let limit = display.y
        * if cfg!(feature = "diagnostic-logging") {
            0.72
        } else {
            0.90
        };
    let budget = (limit - igGetCursorPosY() - footer_height - 16.0 * scale).max(32.0 * scale);
    let height = content_height.min(budget);
    let key = (
        view.fighter,
        view.row,
        view.choice,
        view.mode,
        view.focus,
        view.revision,
    );
    let changed = LAST_FOCUS.try_lock().ok().is_some_and(|mut previous| {
        let changed = *previous != Some(key);
        *previous = Some(key);
        changed
    });
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 4.0 * scale,
            y: 4.0 * scale,
        },
    );
    let shown = igBeginChild_Str(
        c"##recipe_actions".as_ptr(),
        ImVec2_c { x: 0.0, y: height },
        0 as _,
        (ImGuiWindowFlags_NoInputs
            | ImGuiWindowFlags_NoScrollbar
            | ImGuiWindowFlags_NoBackground
            | ImGuiWindowFlags_NoSavedSettings) as _,
    );
    if shown {
        if changed && view.focus.is_none() {
            igSetScrollY(0.0);
        }
        for (i, line) in view.body.iter().enumerate() {
            let is_title = is_recipe_title(line);
            if is_title {
                igSeparator();
            }
            let focused = view.focus == Some(i);
            let text_height = igCalcTextSize(line.as_ptr(), std::ptr::null(), false, wrap).y;
            let local_y = igGetCursorPosY();
            let top = igGetScrollY();
            let outside =
                local_y < top + 4.0 * scale || local_y + text_height > top + height - 4.0 * scale;
            if focused {
                let p = igGetCursorScreenPos();
                ImDrawList_AddRectFilled(
                    igGetWindowDrawList(),
                    ImVec2_c {
                        x: p.x - 2.0 * scale,
                        y: p.y - 1.0 * scale,
                    },
                    ImVec2_c {
                        x: p.x + igGetContentRegionAvail().x + 2.0 * scale,
                        y: p.y + text_height + 1.0 * scale,
                    },
                    igGetColorU32_Vec4(row_highlight()),
                    4.0 * scale,
                    0 as _,
                );
            }
            let tint = if focused {
                accent()
            } else if is_title {
                color(0.55, 0.85, 0.95, 1.0)
            } else {
                color(0.92, 0.94, 0.97, 1.0)
            };
            igPushStyleColor_Vec4(ImGuiCol_Text as _, tint);
            card_text(line);
            igPopStyleColor(1);
            if changed && focused && outside {
                igSetScrollHereY(0.5);
            }
        }
    }
    igEndChild();
    igPopStyleVar(1);
    igSeparator();
}

unsafe fn draw_master_toast(scale: f32, enabled: bool, display: ImVec2_c) {
    igSetNextWindowPos(
        ImVec2_c {
            x: display.x - 8.0 * scale,
            y: display.y - 6.0 * scale,
        },
        ImGuiCond_Always as _,
        ImVec2_c { x: 1.0, y: 1.0 },
    );
    igPushFont(std::ptr::null_mut(), 11.0 * scale);
    igPushStyleColor_Vec4(
        ImGuiCol_Text as _,
        if enabled {
            color(0.45, 1.0, 0.58, 0.82)
        } else {
            color(1.0, 0.48, 0.42, 0.82)
        },
    );
    if igBegin(
        c"##kirby_assist_master".as_ptr(),
        std::ptr::null_mut(),
        OVERLAY_FLAGS | ImGuiWindowFlags_NoBackground as i32,
    ) {
        igTextUnformatted(
            if enabled {
                c"Kirby Assist: ON".as_ptr()
            } else {
                c"Kirby Assist: OFF".as_ptr()
            },
            std::ptr::null(),
        );
    }
    igEnd();
    igPopStyleColor(1);
    igPopFont();
}
#[cfg(feature = "diagnostic-logging")]
unsafe fn draw_result(scale: f32, width: f32, right: f32, y: f32, available: f32) {
    igSetNextWindowPos(
        ImVec2_c { x: right, y },
        ImGuiCond_Always as _,
        ImVec2_c { x: 1.0, y: 0.0 },
    );
    igSetNextWindowSize(ImVec2_c { x: width, y: 0.0 }, ImGuiCond_Always as _);
    igSetNextWindowSizeConstraints(
        ImVec2_c { x: width, y: 0.0 },
        ImVec2_c {
            x: width,
            y: available,
        },
        None,
        std::ptr::null_mut(),
    );
    igPushFont(std::ptr::null_mut(), 13.5 * scale);
    igPushStyleColor_Vec4(ImGuiCol_WindowBg as _, color(0.035, 0.045, 0.075, 0.97));
    if igBegin(
        c"##ledge_result".as_ptr(),
        std::ptr::null_mut(),
        OVERLAY_FLAGS,
    ) {
        igPushTextWrapPos(0.0);
        crate::performance::with_card(|card| {
            igTextUnformatted(card.as_ptr(), std::ptr::null());
            igSeparator();
        });
        // Both cards are cached by the worker. Drawing performs no native
        // reads, formatting, file I/O or blocking lock acquisition.
        if !crate::logging::with_snapshot(|card| igTextUnformatted(card.as_ptr(), std::ptr::null()))
        {
            igTextUnformatted(
                c"LIVE SNAPSHOT: waiting for an assisted action. Full log: status.log".as_ptr(),
                std::ptr::null(),
            );
        }
        crate::calibration::with_card(|card| {
            igSeparator();
            igTextUnformatted(c"LAST COMPLETED LEDGE ATTEMPT".as_ptr(), std::ptr::null());
            igTextUnformatted(card.as_ptr(), std::ptr::null());
        });
        igPopTextWrapPos();
    }
    igEnd();
    igPopStyleColor(1);
    igPopFont();
}
pub(super) fn install() {
    imgui_api::imgui_setup_context(setup);
    imgui_api::imgui_smash_add_on_draw_frame(draw as *const std::ffi::c_void);
}
