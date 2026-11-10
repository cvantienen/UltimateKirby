//! Start here: this is the Switch plugin's entry point, not a desktop program.
//! Skyline calls `main` when it loads this plugin inside Smash. `live::install`
//! loads shared settings and optional fighter sequence overrides, validates the
//! compiled fighter catalog, checks the executable signature, and registers callbacks.
//!
//! Reading order: `live.rs` for integration, core `live_input.rs` for
//! settings, `ledge_trap.rs` for response dispatch, and `native_input.rs` for
//! raw button conversion. docs/CODE_MAP.md connects those pieces step by step.
// Beginner map: this package connects to Smash; crates/kirby_assist_core holds
// ordinary Rust calculations. Start with that crate's examples/walkthrough.rs
// if pointers/hooks are new to you, then return here to follow the real path.
// `mod name;` includes name.rs in this crate; it does not start a thread or tool.
//! Technical: Declares the plugin modules and exposes Skyline's load entry point.
//! Plain English: This file wires the plugin together and starts it when Smash loads it.
/// Technical: Declares the narrow native ABI boundary used by the live integration.
/// Plain English: Provides read-only ways to inspect Smash's native state.
mod game_ffi;
/// Technical: Declares the Switch runtime integration and hook orchestration.
/// Plain English: Connects fighter observations to controller assistance.
mod live;
// The diagnostic logger is absent from silent builds, not just disabled
// by a setting. Config cannot enable it without rebuilding.
#[cfg(feature = "diagnostic-logging")]
/// Technical: Includes optional diagnostic storage and reporting in diagnostic builds.
/// Plain English: Enables troubleshooting support only when compiled with the feature.
mod diagnostics;
#[cfg(feature = "diagnostic-logging")]
use diagnostics::{calibration, grab_diagnostics, logging, performance};

// This attribute generates the native entry point/module metadata Skyline needs.
// Adding a tool does not require another main function; extend the core policy
// and connect its observation/output in `live.rs` instead.
#[skyline::main(name = "kirby_assist")]
/// Technical: Initializes the live plugin integration when Skyline loads the module.
/// Plain English: Skyline calls this once to start Kirby Assist.
pub fn main() {
    live::install();
}
