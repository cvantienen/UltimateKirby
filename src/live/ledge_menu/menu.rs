//! A static saved-configuration selector. Its snapshot has only row labels and
//! saved names: no fighter status, motion or opponent data.
//! Strings are rebuilt only when a selection/row changes. ImGui still draws the
//! cached panel each render frame, as an immediate-mode GUI normally does.
use imgui_api::bindings::*;
use kirby_assist_core::{
    fighter_profiles::{spacing, FighterProfile},
    ledge_menu::{MenuRow, MenuSelection},
};
use std::{
    ffi::{CStr, CString},
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Mutex,
    },
};

struct Snapshot {
    row: MenuRow,
    values: Vec<SavedChoice>,
    position: [f32; 2],
    auto_mash: bool,
}
struct SavedChoice {
    row: MenuRow,
    // Complete single-column lines are cached on edits, never formatted in draw.
    line: CString,
    selected_line: CString,
}
static VIEW: Mutex<Option<Snapshot>> = Mutex::new(None);
// Display lifetime follows the battle, independently of expiring attack
// requests, hitstop, death and respawn. Drawing makes no native game queries.
static BATTLE_ACTIVE: AtomicBool = AtomicBool::new(false);
// Visibility must not depend on acquiring the cached-text mutex. A close
// request can arrive while ImGui is drawing the previous snapshot.
static MENU_VISIBLE: AtomicBool = AtomicBool::new(false);
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
    MENU_VISIBLE.store(false, Ordering::Release);
    set_battle_active(false);
    if let Ok(mut view) = VIEW.try_lock() {
        *view = None;
    }
}
pub(super) fn publish(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
) -> bool {
    MENU_VISIBLE.store(!selection.menu_hidden, Ordering::Release);
    let Ok(mut view) = VIEW.try_lock() else {
        return false;
    };
    if selection.menu_hidden {
        *view = None;
        return true;
    }
    let mut values = Vec::with_capacity(MenuRow::ALL.len());
    for row in MenuRow::ALL {
        let (name, index, count) = if let Some(slot) = row.slot() {
            let options = profile.menu_options.list(slot).options;
            let index = selection.choices[slot as usize];
            (
                options.get(index).map_or_else(
                    || "Unavailable".to_owned(),
                    |v| spacing::menu_name(v.name).to_owned(),
                ),
                index,
                options.len(),
            )
        } else if row == MenuRow::RollRead && selection.roll_read_available {
            (
                selection.roll_read.label().to_owned(),
                selection.roll_read as usize,
                3,
            )
        } else if row == MenuRow::YMode && selection.auto_duck_available {
            let options =
                kirby_assist_core::button_modes::YMode::options_for_kind(selection.y_fighter_kind);
            (
                selection.y_mode.label().to_owned(),
                options
                    .iter()
                    .position(|v| *v == selection.y_mode)
                    .unwrap_or(0),
                options.len(),
            )
        } else if row == MenuRow::IdleCrouch && selection.y_fighter_kind == 6 {
            (
                if selection.idle_crouch { "ON" } else { "OFF" }.to_owned(),
                usize::from(selection.idle_crouch),
                2,
            )
        } else if row == MenuRow::TiltTurn && selection.y_fighter_kind == 6 {
            (
                if selection.tilt_turn { "ON" } else { "OFF" }.to_owned(),
                usize::from(selection.tilt_turn),
                2,
            )
        } else if row == MenuRow::JabGrab && selection.y_fighter_kind == 6 {
            (
                selection.jab_grab.label().to_owned(),
                usize::from(selection.jab_grab == kirby_assist_core::jab_grab::Followup::DownSmash),
                2,
            )
        } else if row == MenuRow::JabLock && selection.y_fighter_kind == 5 {
            (
                if selection.jab_lock { "ON" } else { "OFF" }.to_owned(),
                usize::from(selection.jab_lock),
                2,
            )
        } else if matches!(row, MenuRow::SpotPunish | MenuRow::GrabDodge) {
            let enabled = if row == MenuRow::SpotPunish {
                selection.spot_punish
            } else {
                selection.grab_dodge
            };
            (
                if enabled { "ON" } else { "OFF" }.to_owned(),
                usize::from(enabled),
                2,
            )
        } else if row == MenuRow::AutoMash {
            (
                if auto_mash { "ON" } else { "OFF" }.to_owned(),
                usize::from(auto_mash),
                2,
            )
        } else {
            continue;
        };
        let position = if index < count { index + 1 } else { 0 };
        let body = if matches!(
            row,
            MenuRow::AutoMash
                | MenuRow::SpotPunish
                | MenuRow::GrabDodge
                | MenuRow::IdleCrouch
                | MenuRow::TiltTurn
                | MenuRow::JabGrab
                | MenuRow::JabLock
        ) {
            format!("{}: {name}", row.label())
        } else {
            format!("{}: {name}  {position}/{count}", row.label())
        };
        let (Ok(line), Ok(selected_line)) = (
            CString::new(format!("  {body}")),
            CString::new(format!("> {body}")),
        ) else {
            *view = None;
            return false;
        };
        values.push(SavedChoice {
            row,
            line,
            selected_line,
        });
    }
    *view = Some(Snapshot {
        row: selection.row,
        values,
        // Anchor the panel's top-left corner just inside the screen edge.
        position: [0.005, 0.008],
        auto_mash,
    });
    true
}
fn color(r: f32, g: f32, b: f32, a: f32) -> ImVec4_c {
    ImVec4_c {
        x: r,
        y: g,
        z: b,
        w: a,
    }
}
unsafe fn text(value: &CStr, wrap: bool) {
    let position = igGetCursorScreenPos();
    let wrap_width = if wrap {
        igGetContentRegionAvail().x.max(1.0)
    } else {
        0.0
    };
    let font_size = igGetFontSize();
    // Small opposing dark shadows preserve contrast without a panel backdrop.
    // Reuse the same cached string/font; no icons, textures or font atlas work.
    for direction in [-1.0, 1.0] {
        ImDrawList_AddText_FontPtr(
            igGetWindowDrawList(),
            igGetFont(),
            font_size,
            ImVec2_c {
                x: position.x + direction * font_size * 0.065,
                y: position.y + direction * font_size * 0.065,
            },
            0xf5000000,
            value.as_ptr(),
            std::ptr::null(),
            wrap_width,
            std::ptr::null(),
        );
    }
    igTextUnformatted(value.as_ptr(), std::ptr::null());
    ImDrawList_AddText_FontPtr(
        igGetWindowDrawList(),
        igGetFont(),
        font_size,
        ImVec2_c {
            x: position.x + font_size * 0.03,
            y: position.y,
        },
        igGetColorU32_Col(ImGuiCol_Text as _, 1.0),
        value.as_ptr(),
        std::ptr::null(),
        wrap_width,
        std::ptr::null(),
    );
}
unsafe extern "C" fn setup(context: *mut u64) {
    CONTEXT.store(context, Ordering::Release);
    if !context.is_null() {
        igSetCurrentContext(context.cast());
    }
}
unsafe extern "C" fn draw() {
    if !BATTLE_ACTIVE.load(Ordering::Acquire) || !MENU_VISIBLE.load(Ordering::Acquire) {
        return;
    }
    let Ok(view) = VIEW.try_lock() else {
        return;
    };
    let Some(view) = view.as_ref() else {
        return;
    };
    let context = CONTEXT.load(Ordering::Acquire);
    if context.is_null() {
        return;
    }
    igSetCurrentContext(context.cast());
    let io = igGetIO();
    if io.is_null() {
        return;
    }
    let display = (*io).DisplaySize;
    if !display.x.is_finite() || !display.y.is_finite() || display.x <= 0.0 || display.y <= 0.0 {
        return;
    }
    let scale = (display.y / 720.0).clamp(0.65, 3.0);
    // Measure the cached complete lines. No word wrapping, per-frame string
    // formatting or hardcoded width that clips a fighter's longer move names.
    let diagnostic = cfg!(feature = "diagnostic-logging");
    let menu_font_size = if diagnostic { 14.0 } else { 19.0 } * scale;
    igPushFont(std::ptr::null_mut(), menu_font_size);
    let width = if diagnostic {
        (280.0 * scale).min(display.x * 0.25)
    } else {
        (view
            .values
            .iter()
            .map(|v| igCalcTextSize(v.selected_line.as_ptr(), std::ptr::null(), false, -1.0).x)
            .fold(0.0_f32, f32::max)
            + 8.0 * scale)
            .min(display.x - 8.0)
    };
    igPopFont();
    if width < 100.0 {
        return;
    }
    let accent = color(1.0, 0.88, 0.35, 1.0);
    let normal = color(0.94, 0.97, 1.0, 1.0);
    let modes = color(0.82, 0.78, 1.0, 1.0);
    igPushFont(std::ptr::null_mut(), menu_font_size);
    for (slot, value) in [
        (ImGuiCol_WindowBg, color(0.0, 0.0, 0.0, 0.0)),
        (ImGuiCol_Border, color(0.0, 0.0, 0.0, 0.0)),
        (ImGuiCol_Text, color(0.9, 0.94, 1.0, 1.0)),
    ] {
        igPushStyleColor_Vec4(slot as _, value);
    }
    igPushStyleVar_Float(ImGuiStyleVar_WindowBorderSize as _, 0.0);
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 3.0 * scale,
            y: 3.0 * scale,
        },
    );
    igPushStyleVar_Vec2(
        ImGuiStyleVar_ItemSpacing as _,
        ImVec2_c {
            x: 0.0,
            y: if diagnostic { 2.0 } else { 4.0 } * scale,
        },
    );
    igSetNextWindowPos(
        ImVec2_c {
            x: if diagnostic {
                4.0 * scale
            } else {
                (display.x * view.position[0]).clamp(4.0, (display.x - width - 4.0).max(4.0))
            },
            y: if diagnostic {
                8.0 * scale
            } else {
                display.y * view.position[1]
            },
        },
        ImGuiCond_Always as _,
        ImVec2_c { x: 0.0, y: 0.0 },
    );
    igSetNextWindowSize(ImVec2_c { x: width, y: 0.0 }, ImGuiCond_Always as _);
    let flags = ImGuiWindowFlags_NoDecoration
        | ImGuiWindowFlags_NoBackground
        | ImGuiWindowFlags_AlwaysAutoResize
        | ImGuiWindowFlags_NoSavedSettings
        | ImGuiWindowFlags_NoInputs
        | ImGuiWindowFlags_NoFocusOnAppearing
        | ImGuiWindowFlags_NoBringToFrontOnFocus;
    if igBegin(
        c"##saved_options".as_ptr(),
        std::ptr::null_mut(),
        flags as _,
    ) {
        for value in &view.values {
            let selected = value.row == view.row;
            let tint = if selected {
                accent
            } else if value.row == MenuRow::AutoMash {
                if view.auto_mash {
                    color(0.45, 1.0, 0.6, 1.0)
                } else {
                    color(1.0, 0.48, 0.42, 1.0)
                }
            } else if matches!(value.row, MenuRow::RollRead | MenuRow::YMode) {
                modes
            } else {
                normal
            };
            igPushStyleColor_Vec4(ImGuiCol_Text as _, tint);
            if diagnostic {
                igPushTextWrapPos(0.0);
            }
            text(
                if selected {
                    &value.selected_line
                } else {
                    &value.line
                },
                diagnostic,
            );
            if diagnostic {
                igPopTextWrapPos();
            }
            igPopStyleColor(1);
        }
    }
    #[cfg(feature = "diagnostic-logging")]
    let result_space = kirby_assist_core::diagnostics::modes::left_result_space(
        display.y,
        scale,
        igGetWindowHeight(),
    );
    igEnd();
    #[cfg(feature = "diagnostic-logging")]
    if let Some((result_y, available)) = result_space {
        draw_result(scale, width, result_y, available, flags as _);
    }
    igPopStyleVar(3);
    igPopStyleColor(3);
    igPopFont();
}
#[cfg(feature = "diagnostic-logging")]
unsafe fn draw_result(scale: f32, width: f32, y: f32, available: f32, flags: ImGuiWindowFlags) {
    {
        igSetNextWindowPos(
            ImVec2_c { x: 4.0 * scale, y },
            ImGuiCond_Always as _,
            ImVec2_c { x: 0.0, y: 0.0 },
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
        igPushFont(std::ptr::null_mut(), 12.5 * scale);
        igPushStyleColor_Vec4(ImGuiCol_WindowBg as _, color(0.035, 0.045, 0.075, 0.97));
        if igBegin(
            c"##ledge_result".as_ptr(),
            std::ptr::null_mut(),
            flags & !(ImGuiWindowFlags_NoBackground as ImGuiWindowFlags),
        ) {
            igPushTextWrapPos(0.0);
            // Both cards are cached by the worker. Drawing performs no native
            // reads, formatting, file I/O or blocking lock acquisition.
            if !crate::logging::with_snapshot(|card| {
                igTextUnformatted(card.as_ptr(), std::ptr::null())
            }) {
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
}
pub(super) fn install() {
    imgui_api::imgui_setup_context(setup);
    imgui_api::imgui_smash_add_on_draw_frame(draw as *const std::ffi::c_void);
}
