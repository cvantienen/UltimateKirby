// Compile the actual standard-library logger on the host to exercise its queue.
// Tests never call start(), create an SD path or launch a background thread.
#[allow(dead_code)]
#[path = "../../../src/logging.rs"]
mod diagnostic_logger;
