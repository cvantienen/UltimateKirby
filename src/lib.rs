//! Start here: this is the Switch plugin's entry point, not a desktop program.
//! Skyline calls `main` when it loads this plugin inside Smash. `live::install`
//! loads settings, checks the executable signature, and registers callbacks.
//!
//! Reading order: `live.rs` for integration, `live_input.rs` for tool timing,
//! `native_input.rs` for raw button conversion, then the core's logical rules.
// Beginner map: this package connects to Smash; crates/kirby_assist_core holds
// ordinary Rust calculations. Start with that crate's examples/walkthrough.rs
// if pointers/hooks are new to you, then return here to follow the real path.
// `mod name;` includes name.rs in this crate; it does not start a thread or tool.
mod game_ffi;
mod live;
// The entire logger module is absent from the silent binary, not just disabled
// by a setting. Config cannot turn it back on without rebuilding this feature.
#[cfg(feature = "diagnostic-logging")]
mod logging;

// This attribute generates the native entry point/module metadata Skyline needs.
// Adding a tool does not require another main function; extend the core policy
// and connect its observation/output in `live.rs` instead.
#[skyline::main(name = "kirby_assist")]
pub fn main() {
    live::install();
}
