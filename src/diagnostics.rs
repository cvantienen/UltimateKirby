//! Diagnostic-only recording, logging and performance instrumentation.
//! Technical: Declares the diagnostic modules compiled for diagnostic builds.
//! Plain English: These modules collect and report optional troubleshooting information.
/// Provides calibration capture and analysis routines.
pub(crate) mod calibration;
/// Provides grab-related diagnostic collection and reporting.
pub(crate) mod grab_diagnostics;
/// Provides the diagnostic logging backend.
pub(crate) mod logging;
/// Provides runtime performance measurements and reports.
pub(crate) mod performance;
