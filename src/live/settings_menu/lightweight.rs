//! Compact match overlay for page context and selectable option rows.
//! Technical: Provides the bounded renderer for in-match lightweight pages.
//! Plain English: This module draws the smaller menu shown during a match.
use super::{color, Snapshot};
use imgui_api::bindings::*;
use kirby_assist_core::settings_menu::MenuSelection;
use std::ffi::CStr;

// Shared by the full card and compact renderer; keep status separate from focus.
pub(super) mod palette {
    use super::{color, ImVec4_c};
    pub(crate) fn surface(alpha: f32) -> ImVec4_c {
        color(0.035, 0.045, 0.070, alpha)
    }
    pub(crate) fn border() -> ImVec4_c {
        color(0.26, 0.31, 0.42, 0.85)
    }
    pub(crate) fn text() -> ImVec4_c {
        color(0.90, 0.93, 0.98, 1.0)
    }
    pub(crate) fn muted() -> ImVec4_c {
        color(0.66, 0.72, 0.82, 1.0)
    }
    pub(crate) fn focus() -> ImVec4_c {
        color(1.0, 0.91, 0.65, 1.0)
    }
    pub(crate) fn highlight() -> ImVec4_c {
        color(0.33, 0.28, 0.15, 0.65)
    }
    pub(crate) fn value() -> ImVec4_c {
        color(0.98, 0.79, 0.51, 1.0)
    }
    pub(crate) fn action() -> ImVec4_c {
        color(0.80, 0.70, 1.0, 1.0)
    }
    pub(crate) fn success() -> ImVec4_c {
        color(0.53, 0.89, 0.68, 1.0)
    }
    pub(crate) fn warning() -> ImVec4_c {
        color(1.0, 0.74, 0.40, 1.0)
    }
    pub(crate) fn error() -> ImVec4_c {
        color(1.0, 0.57, 0.59, 1.0)
    }
}

pub(crate) fn value_tint(enabled: Option<bool>, action: bool) -> ImVec4_c {
    match enabled {
        Some(true) => palette::success(),
        Some(false) => palette::muted(),
        None if action => palette::action(),
        None => palette::value(),
    }
}

/// Draw label and cached value separately so focus never hides ON/OFF state.
pub(super) unsafe fn draw_option_line(
    line: &CStr,
    selected: bool,
    enabled: Option<bool>,
    action: bool,
) {
    let label_tint = if selected {
        palette::focus()
    } else {
        palette::text()
    };
    igPushStyleColor_Vec4(ImGuiCol_Text as _, label_tint);
    if let Some(split) = line.to_bytes().iter().position(|byte| *byte == b':') {
        let value = line.as_ptr().add(split + 1);
        igTextUnformatted(line.as_ptr(), value);
        igPopStyleColor(1);
        igSameLine(0.0, 0.0);
        igPushStyleColor_Vec4(ImGuiCol_Text as _, value_tint(enabled, action));
        igTextUnformatted(value, std::ptr::null());
    } else {
        igTextUnformatted(line.as_ptr(), std::ptr::null());
    }
    igPopStyleColor(1);
}

/// Best-effort normalized placement by configured controller slot, not a native HUD anchor.
/// The game may reorder damage panels in teams or layouts with more than four players.
pub(super) unsafe fn draw_battle_hud(
    player: u32,
    label: &'static std::ffi::CStr,
    rgb: (f32, f32, f32),
    display: ImVec2_c,
    scale: f32,
) {
    igSetNextWindowPos(
        ImVec2_c {
            x: display.x * (0.14 + 0.24 * player.min(3) as f32),
            y: display.y * 0.92,
        },
        ImGuiCond_Always as _,
        ImVec2_c { x: 0.0, y: 1.0 },
    );
    igSetNextWindowSize(ImVec2_c { x: 0.0, y: 0.0 }, ImGuiCond_Always as _);
    igPushStyleVar_Vec2(
        ImGuiStyleVar_WindowPadding as _,
        ImVec2_c {
            x: 5.0 * scale,
            y: 3.0 * scale,
        },
    );
    igPushFont(std::ptr::null_mut(), 13.0 * scale);
    igPushStyleColor_Vec4(ImGuiCol_Text as _, color(rgb.0, rgb.1, rgb.2, 1.0));
    igPushStyleColor_Vec4(ImGuiCol_WindowBg as _, palette::surface(0.90));
    let flags = ImGuiWindowFlags_NoDecoration
        | ImGuiWindowFlags_NoScrollbar
        | ImGuiWindowFlags_AlwaysAutoResize
        | ImGuiWindowFlags_NoSavedSettings
        | ImGuiWindowFlags_NoInputs
        | ImGuiWindowFlags_NoFocusOnAppearing
        | ImGuiWindowFlags_NoBringToFrontOnFocus;
    if igBegin(
        c"##battle_l_mode".as_ptr(),
        std::ptr::null_mut(),
        flags as _,
    ) {
        igTextUnformatted(label.as_ptr(), std::ptr::null());
    }
    igEnd();
    igPopStyleColor(2);
    igPopStyleVar(1);
    igPopFont();
}

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
    igPushStyleColor_Vec4(ImGuiCol_Text as _, palette::text());
    igPushStyleColor_Vec4(ImGuiCol_WindowBg as _, palette::surface(0.94));
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
        igPushStyleColor_Vec4(ImGuiCol_Text as _, palette::muted());
        igTextUnformatted(c"MATCH / GAMEPLAY ACTIVE".as_ptr(), std::ptr::null());
        igPopStyleColor(1);
        if let Some(label) = &view.lightweight_label {
            igPushStyleColor_Vec4(ImGuiCol_Text as _, color(r, g, b, 1.0));
            igTextUnformatted(label.as_ptr(), std::ptr::null());
            igPopStyleColor(1);
        }
        let (start, end, focus) = view.row_window();
        if let Some(context) = view.row_context.get(focus) {
            igPushStyleColor_Vec4(ImGuiCol_Text as _, palette::muted());
            igTextUnformatted(context.as_ptr(), std::ptr::null());
            igPopStyleColor(1);
        }
        for value in &view.values[start..end] {
            let selected = value.row == view.row;
            if selected {
                igPushStyleColor_Vec4(ImGuiCol_Text as _, palette::focus());
                igTextUnformatted(c">".as_ptr(), std::ptr::null());
                igSameLine(0.0, 3.0 * scale);
                igPopStyleColor(1);
            }
            draw_option_line(
                &value.line,
                selected,
                value.enabled,
                MenuSelection::page_link(value.row),
            );
        }
        igPopTextWrapPos();
    }
    igEnd();
    igPopStyleColor(2);
    igPopStyleVar(1);
    igPopFont();
}
