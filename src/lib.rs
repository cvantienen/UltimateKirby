//! Start here: this is the Switch plugin's entry point, not a desktop program.
//! Skyline calls `main` when it loads this plugin inside Smash. `live::install`
//! loads shared settings and optional fighter sequence overrides, validates the
//! compiled fighter catalog, checks the executable signature, and registers callbacks.
//!
//! Reading order: `live.rs` for integration, core `live_input/config.rs` for
//! settings, `ledge_trap.rs` for response dispatch, and `native_input.rs` for
//! raw button conversion. docs/CODE_MAP.md connects those pieces step by step.
// Beginner map: this package connects to Smash; crates/kirby_assist_core holds
// ordinary Rust calculations. Start with that crate's examples/walkthrough.rs
// if pointers/hooks are new to you, then return here to follow the real path.
// `mod name;` includes name.rs in this crate; it does not start a thread or tool.
mod game_ffi;
mod live;
// The diagnostic logger is absent from silent builds, not just disabled
// by a setting. The separate opt-in silent-probe is bounded aggregate SD output.
// Config cannot enable either feature without rebuilding.
#[cfg(feature = "diagnostic-logging")]
mod calibration;
#[cfg(feature = "diagnostic-logging")]
mod grab_diagnostics;
#[cfg(feature = "diagnostic-logging")]
mod logging;
#[cfg(feature = "diagnostic-logging")]
mod performance;

#[cfg(all(feature = "silent-probe", feature = "diagnostic-logging"))]
compile_error!("silent-probe is a separate temporary build, not diagnostic logging");
#[cfg(feature = "silent-probe")]
mod one_match_probe;

// This attribute generates the native entry point/module metadata Skyline needs.
// Adding a tool does not require another main function; extend the core policy
// and connect its observation/output in `live.rs` instead.
#[skyline::main(name = "kirby_assist")]
pub fn main() {
    live::install();
}
