//! Diagnostic-only recording, logging and performance instrumentation.
//! Technical: Declares the diagnostic modules compiled for diagnostic builds.
//! Plain English: These modules collect and report optional troubleshooting information.
/// Technical: Provides calibration capture and analysis routines.
/// Plain English: This module helps tune behavior using recorded observations.
pub(crate) mod calibration;
/// Technical: Provides grab-related diagnostic collection and reporting.
/// Plain English: This module records information useful for investigating grabs.
pub(crate) mod grab_diagnostics;
/// Technical: Provides the diagnostic logging backend.
/// Plain English: This module writes optional troubleshooting messages.
pub(crate) mod logging;
/// Technical: Provides runtime performance measurements and reports.
/// Plain English: This module shows how much time the plugin spends on its work.
pub(crate) mod performance;
