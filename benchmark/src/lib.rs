//! Benchmark harness library.
//!
//! The binary and the executable-spec runner share this crate. Product code
//! does not depend on it.

/// Harness version recorded in the report.
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");
