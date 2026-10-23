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
    name: CString,
    // Cached with the saved name; drawing never looks up gameplay state.
    number: CString,
}
static VIEW: Mutex<Option<Snapshot>> = Mutex::new(None);
// Display lifetime follows the battle, independently of expiring attack
// requests, hitstop, death and respawn. Drawing makes no native game queries.
static BATTLE_ACTIVE: AtomicBool = AtomicBool::new(false);
// The installed host supplies its context through this setup callback. Older
// compatible hosts do not export imgui_get_context_export; importing that
// optional getter would stop Skyline from loading the entire plugin.
static CONTEXT: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());
pub(super) fn set_battle_active(active: bool) {
    let was_active = BATTLE_ACTIVE.swap(active, Ordering::AcqRel);
    #[cfg(feature = "diagnostic-logging")]
    if was_active && !active {
        crate::calibration::clear();
    }
    #[cfg(not(feature = "diagnostic-logging"))]
    let _ = was_active;
}
pub(super) fn clear() {
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
                    |v| {
                        let name = spacing::menu_name(v.name);
                        let hint = spacing::hint(profile.fighter_kind, slot, v.id);
                        if hint.is_empty() {
                            name.to_owned()
                        } else {
                            format!("{name} [{hint}]")
                        }
                    },
                ),
                index,
                options.len(),
            )
        } else {
            continue;
        };
        let Ok(name) = CString::new(name) else {
            *view = None;
            return false;
        };
        let position = if index < count { index + 1 } else { 0 };
        let Ok(number) = CString::new(format!("{position}/{count}")) else {
            *view = None;
            return false;
        };
        values.push(SavedChoice { name, number });
    }
    *view = Some(Snapshot {
        row: selection.row,
        values,
        // Anchor the panel's bottom-left corner just inside the screen edge.
        position: [0.01, 0.985],
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
    // Keep ImGui's ordinary layout, then thicken the same cached text with a
    // subpixel-offset pass. No extra font atlas, strings or game queries.
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
    if !BATTLE_ACTIVE.load(Ordering::Acquire) {
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
    let width = (360.0 * scale).min(display.x - 8.0);
    if width < 100.0 {
        return;
    }
    let muted = color(0.75, 0.8, 0.88, 1.0);
    let accent = color(1.0, 0.88, 0.35, 1.0);
    igPushFont(std::ptr::null_mut(), 19.0 * scale);
    for (slot, value) in [
        (ImGuiCol_WindowBg, color(0.035, 0.045, 0.075, 0.97)),
        (ImGuiCol_Border, color(0.24, 0.31, 0.43, 0.8)),
        (ImGuiCol_Text, color(0.9, 0.94, 1.0, 1.0)),
    ] {
        igPushStyleColor_Vec4(slot as _, value);
    }
    igPushStyleVar_Float(ImGuiStyleVar_WindowRounding as _, 7.0 * scale);
    igPushStyleVar_Float(ImGuiStyleVar_WindowBorderSize as _, 1.0);
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 8.0 * scale,
            y: 6.0 * scale,
        },
    );
    igPushStyleVar_Vec2(
        ImGuiStyleVar_ItemSpacing as _,
        ImVec2_c {
            x: 6.0 * scale,
            y: 3.0 * scale,
        },
    );
    igPushStyleVar_Vec2(
        ImGuiStyleVar_CellPadding as _,
        ImVec2_c {
            x: 3.0 * scale,
            y: 2.0 * scale,
        },
    );
    igSetNextWindowPos(
        ImVec2_c {
            x: (display.x * view.position[0]).clamp(4.0, (display.x - width - 4.0).max(4.0)),
            y: display.y * view.position[1],
        },
        ImGuiCond_Always as _,
        ImVec2_c { x: 0.0, y: 1.0 },
    );
    igSetNextWindowSize(ImVec2_c { x: width, y: 0.0 }, ImGuiCond_Always as _);
    let flags = ImGuiWindowFlags_NoDecoration
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
        if igBeginTable(
            c"saved_choices".as_ptr(),
            3,
            (ImGuiTableFlags_SizingStretchProp | ImGuiTableFlags_NoSavedSettings) as _,
            ImVec2_c { x: 0.0, y: 0.0 },
            0.0,
        ) {
            igTableSetupColumn(
                c"List".as_ptr(),
                ImGuiTableColumnFlags_WidthFixed as _,
                86.0 * scale,
                0,
            );
            igTableSetupColumn(
                c"Option number".as_ptr(),
                ImGuiTableColumnFlags_WidthFixed as _,
                36.0 * scale,
                0,
            );
            igTableSetupColumn(
                c"Saved option".as_ptr(),
                ImGuiTableColumnFlags_WidthStretch as _,
                1.0,
                0,
            );
            for ((row, (label, marked_label)), value) in MenuRow::ALL
                .iter()
                .zip([
                    (c"Neutral", c"> Neutral"),
                    (c"Jump", c"> Jump"),
                    (c"Roll", c"> Roll"),
                    (c"Attack", c"> Attack"),
                    (c"Drop", c"> Drop"),
                ])
                .zip(&view.values)
            {
                igTableNextRow(0, 0.0);
                igTableNextColumn();
                let selected = *row == view.row;
                if selected {
                    igTableSetBgColor(ImGuiTableBgTarget_RowBg0 as _, 0xff493b20, -1);
                }
                igPushStyleColor_Vec4(ImGuiCol_Text as _, if selected { accent } else { muted });
                text(if selected { marked_label } else { label }, false);
                igPopStyleColor(1);
                igTableNextColumn();
                igPushStyleColor_Vec4(ImGuiCol_Text as _, if selected { accent } else { muted });
                text(&value.number, false);
                igPopStyleColor(1);
                igTableNextColumn();
                if selected {
                    igPushStyleColor_Vec4(ImGuiCol_Text as _, accent);
                }
                igPushTextWrapPos(0.0);
                text(&value.name, true);
                igPopTextWrapPos();
                if selected {
                    igPopStyleColor(1);
                }
            }
            igEndTable();
        }
        igPushStyleColor_Vec4(
            ImGuiCol_Text as _,
            if view.auto_mash { accent } else { muted },
        );
        text(
            if view.auto_mash {
                c"Auto-mash: ON"
            } else {
                c"Auto-mash: OFF"
            },
            false,
        );
        igPopStyleColor(1);
    }
    igEnd();
    #[cfg(feature = "diagnostic-logging")]
    draw_result(view, display, scale, width, flags as _);
    igPopStyleVar(5);
    igPopStyleColor(3);
    igPopFont();
}
#[cfg(feature = "diagnostic-logging")]
unsafe fn draw_result(
    view: &Snapshot,
    display: ImVec2_c,
    scale: f32,
    menu_width: f32,
    flags: ImGuiWindowFlags,
) {
    crate::calibration::with_card(|card| {
        let width = (440.0 * scale).min(display.x - 8.0);
        let menu_x =
            (display.x * view.position[0]).clamp(4.0, (display.x - menu_width - 4.0).max(4.0));
        let x = (menu_x + menu_width + 8.0 * scale).min((display.x - width - 4.0).max(4.0));
        igSetNextWindowPos(
            ImVec2_c {
                x,
                y: display.y * view.position[1],
            },
            ImGuiCond_Always as _,
            ImVec2_c { x: 0.0, y: 1.0 },
        );
        igSetNextWindowSize(ImVec2_c { x: width, y: 0.0 }, ImGuiCond_Always as _);
        igPushFont(std::ptr::null_mut(), 15.5 * scale);
        if igBegin(c"##ledge_result".as_ptr(), std::ptr::null_mut(), flags) {
            igPushTextWrapPos(0.0);
            // Cached completed report only; rendering never inspects gameplay.
            igTextUnformatted(card.as_ptr(), std::ptr::null());
            igPopTextWrapPos();
        }
        igEnd();
        igPopFont();
    });
}
pub(super) fn install() {
    imgui_api::imgui_setup_context(setup);
    imgui_api::imgui_smash_add_on_draw_frame(draw as *const std::ffi::c_void);
}
