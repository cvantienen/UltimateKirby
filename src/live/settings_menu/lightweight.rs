//! Compact match overlay for page context and selectable option rows.
//! Technical: Provides the bounded renderer for in-match lightweight pages.
//! Plain English: This module draws the smaller menu shown during a match.
use super::{color, Snapshot};
use imgui_api::bindings::*;

/// No tables, measured rows or per-frame string formatting/allocation.
pub(super) unsafe fn draw(view: &Snapshot, display: ImVec2_c, scale: f32) {
    if !view.lightweight {
        return;
    }
    let diagnostic = cfg!(feature = "diagnostic-logging");
    igSetNextWindowPos(
        ImVec2_c {
            x: if diagnostic {
                display.x - 8.0 * scale
            } else {
                8.0 * scale
            },
            y: 8.0 * scale,
        },
        ImGuiCond_Always as _,
        ImVec2_c {
            x: if diagnostic { 1.0 } else { 0.0 },
            y: 0.0,
        },
    );
    // Natural text width, bounded for unusually long saved names. A fixed wrap
    // limit lets ImGui fit short text without per-frame text measurements.
    let max_width = display.x * 0.42;
    igSetNextWindowSize(ImVec2_c { x: 0.0, y: 0.0 }, ImGuiCond_Always as _);
    igSetNextWindowSizeConstraints(
        ImVec2_c { x: 0.0, y: 0.0 },
        ImVec2_c {
            x: max_width,
            y: display.y - 16.0 * scale,
        },
        None,
        std::ptr::null_mut(),
    );
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 8.0 * scale,
            y: 6.0 * scale,
        },
    );
    igPushFont(std::ptr::null_mut(), 15.0 * scale);
    igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.92, 0.94, 0.96, 1.0));
    igPushStyleColor_Vec4(ImGuiCol_WindowBg as _, color(0.025, 0.035, 0.05, 0.80));
    let flags = ImGuiWindowFlags_NoDecoration
        | ImGuiWindowFlags_NoScrollbar
        | ImGuiWindowFlags_AlwaysAutoResize
        | ImGuiWindowFlags_NoSavedSettings
        | ImGuiWindowFlags_NoInputs
        | ImGuiWindowFlags_NoFocusOnAppearing
        | ImGuiWindowFlags_NoBringToFrontOnFocus;
    if igBegin(
        c"##match_selector".as_ptr(),
        std::ptr::null_mut(),
        flags as _,
    ) {
        igPushTextWrapPos(max_width - 16.0 * scale);
        let (r, g, b) = view.accent_rgb();
        igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.72, 0.78, 0.85, 1.0));
        igTextUnformatted(c"MATCH / GAMEPLAY ACTIVE".as_ptr(), std::ptr::null());
        igPopStyleColor(1);
        if let Some(label) = &view.lightweight_label {
            igPushStyleColor_Vec4(ImGuiCol_Text as _, color(r, g, b, 1.0));
            igTextUnformatted(label.as_ptr(), std::ptr::null());
            igPopStyleColor(1);
        }
        let (start, end, focus) = view.row_window();
        if let Some(context) = view.row_context.get(focus) {
            igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.72, 0.78, 0.85, 1.0));
            igTextUnformatted(context.as_ptr(), std::ptr::null());
            igPopStyleColor(1);
        }
        for value in &view.values[start..end] {
            if value.row == view.row {
                igPushStyleColor_Vec4(ImGuiCol_Text as _, color(r, g, b, 1.0));
                igTextUnformatted(c">".as_ptr(), std::ptr::null());
                igSameLine(0.0, 3.0 * scale);
                igTextUnformatted(value.line.as_ptr(), std::ptr::null());
                igPopStyleColor(1);
            } else {
                igTextUnformatted(value.line.as_ptr(), std::ptr::null());
            }
        }
        igPushStyleColor_Vec4(ImGuiCol_Text as _, color(0.72, 0.78, 0.85, 1.0));
        igTextUnformatted(
            c"Hold L + D-pad: select / change".as_ptr(),
            std::ptr::null(),
        );
        igTextUnformatted(
            c"Release L: gameplay | Minus: hide".as_ptr(),
            std::ptr::null(),
        );
        igPopStyleColor(1);
        igPopTextWrapPos();
    }
    igEnd();
    igPopStyleColor(2);
    igPopStyleVar(1);
    igPopFont();
}
