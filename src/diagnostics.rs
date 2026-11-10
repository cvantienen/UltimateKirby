//! Diagnostic-only recording, logging and performance instrumentation.
//! Technical: Declares the diagnostic modules compiled for diagnostic builds.
//! Plain English: These modules collect and report optional troubleshooting information.
/// Provides calibration capture and analysis routines.
///
///  This module helps tune behavior using recorded observations.
pub(crate) mod calibration;
/// Provides grab-related diagnostic collection and reporting.
///
///  This module records information useful for investigating grabs.
pub(crate) mod grab_diagnostics;
/// Provides the diagnostic logging backend.
///
///  This module writes optional troubleshooting messages.
pub(crate) mod logging;
/// Provides runtime performance measurements and reports.
///
///  This module shows how much time the plugin spends on its work.
pub(crate) mod performance;
