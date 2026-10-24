#![cfg(feature = "diagnostic-logging")]
#![allow(unexpected_cfgs)]
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
