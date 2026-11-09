//! Host-only filesystem tests for the explicit save worker; never touches SD.
// Mirror the adapter's two parent modules so production save publication
// resolves its existing globals; these stubs never access a Switch.
static SEQUENCES: std::sync::OnceLock<
    std::sync::Mutex<kirby_assist_core::sequence_config::SequenceLibrary>,
> = std::sync::OnceLock::new();
static SEQUENCES_UPDATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[path = "."]
mod live {
    #[allow(dead_code)]
    #[path = "../../../src/live/settings_menu/save.rs"]
    mod menu_save;
}

#[cfg(feature = "diagnostic-logging")]
#[allow(dead_code)]
#[path = "../../../src/diagnostics/performance.rs"]
mod performance;

// Exercise the exact cache-copy implementation without loading ImGui/native UI.
#[allow(dead_code)]
#[path = "../../../src/live/settings_menu/view.rs"]
mod menu_view;
