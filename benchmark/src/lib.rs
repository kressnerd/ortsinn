//! Benchmark harness library.
//!
//! The binary and the executable-spec runner share this crate. Product code
//! does not depend on it.

pub mod adapters;
pub mod cli;
pub mod gold;
pub mod manifest;
pub mod metrics;
pub mod record;
pub mod report;
pub mod runner;

/// Harness version recorded in the report.
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");
