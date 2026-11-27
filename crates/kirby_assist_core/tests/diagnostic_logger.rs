#![allow(unexpected_cfgs)]
// Exercise the real RAM logger and recorders on the host.
#[allow(dead_code)]
#[path = "../../../src/diagnostics/logging.rs"]
mod diagnostic_logger;
use diagnostic_logger as logging;
#[allow(dead_code)]
#[path = "../../../src/diagnostics/calibration.rs"]
mod calibration;
#[allow(dead_code)]
#[path = "../../../src/diagnostics/grab_diagnostics.rs"]
mod grab_diagnostics;

#[allow(dead_code)]
#[path = "../../../src/diagnostics/performance.rs"]
mod performance;
