//! Normalized candidate models shared by the cassette and report layers.
//!
//! The real candidate adapters are delivered by the `benchmark-run` change.
//! This change proves the contract with recorded normalized results.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// A normalized candidate location. A candidate result without a line range
/// is a file-level resolution, never a promoted symbol match.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NormalizedLocation {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub symbol: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
}

/// One candidate explanation with citations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationResponse {
    pub text: String,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Citation {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Deduplicate by `(path, start_line, end_line, symbol)` without reordering.
pub fn dedup_locations(locations: &[NormalizedLocation]) -> Vec<NormalizedLocation> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for location in locations {
        let key = (
            location.path.clone(),
            location.start_line,
            location.end_line,
            location.symbol.clone(),
        );
        if seen.insert(key) {
            out.push(location.clone());
        }
    }
    out
}
