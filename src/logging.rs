//! The plugin's explicit file/console logger, compiled ONLY with diagnostic-logging.
//! This module does not exist in the silent build. It does not control Skyline,
//! OS/game telemetry, panic handling, or reports from other installed plugins.
use std::io::Write;

const LOG_PATH: &str = "sd:/ultimate/kirby_assist/status.log";

/// Start a fresh diagnostic report at each game launch. The silent build never
/// calls this or creates/truncates this file; existing old logs are left alone.
pub fn start() {
    // `let _ = ...` deliberately ignores an I/O Result. A missing/unwritable
    // SD path should not stop gameplay, but it can mean a diagnostic is lost.
    let _ = std::fs::create_dir_all("sd:/ultimate/kirby_assist");
    let _ = std::fs::write(
        LOG_PATH,
        concat!(
            "Kirby Assist diagnostic build ",
            env!("CARGO_PKG_VERSION"),
            "; on-device validation required\n"
        ),
    );
}

/// Tee explicit plugin messages to standard output and our local SD-card file.
/// Call sites decide whether optional detailed tech messages are enabled.
pub fn write(message: &str) {
    println!("[kirby_assist] {message}");
    // OpenOptions is a builder: append adds to an existing report, create
    // permits its first write. `if let Ok(...)` skips writing on an open error.
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        let _ = writeln!(file, "{message}");
    }
}
