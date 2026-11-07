#![cfg(feature = "diagnostic-logging")]
#![allow(unexpected_cfgs)]
mod live {
    pub fn feature_activity_allowed() -> bool {
        true
    }
}
// Exercise the real logger/recorders on the host without opening SD files.
#[allow(dead_code)]
#[path = "../../../src/logging.rs"]
mod diagnostic_logger;
use diagnostic_logger as logging;
#[allow(dead_code)]
#[path = "../../../src/calibration.rs"]
mod calibration;
#[allow(dead_code)]
#[path = "../../../src/grab_diagnostics.rs"]
mod grab_diagnostics;

#[cfg(feature = "diagnostic-logging")]
#[allow(dead_code)]
#[path = "../../../src/performance.rs"]
mod performance;
