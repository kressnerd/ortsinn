//! Fixture adapter. Compiled only for tests or the `fixture` feature, so the
//! release binary cannot emit fixture measurements.

use std::collections::BTreeMap;

use anyhow::{Context, Result};

use super::{ExplanationResponse, NormalizedLocation, dedup_locations};

/// A normalized fixture response keyed by task and arm.
#[derive(Debug, Clone)]
pub struct FixtureResponse {
    pub results: Vec<NormalizedLocation>,
    pub explanation: Option<ExplanationResponse>,
}

/// Serves the committed fixture cassette for one candidate. This is the
/// adapter used by the fixture scenarios; it never starts a real candidate.
#[derive(Debug, Clone)]
pub struct FixtureAdapter {
    candidate: String,
    responses: BTreeMap<(String, String), FixtureResponse>,
}

impl FixtureAdapter {
    pub fn new(candidate: impl Into<String>) -> Self {
        Self {
            candidate: candidate.into(),
            responses: BTreeMap::new(),
        }
    }

    pub fn candidate(&self) -> &str {
        &self.candidate
    }

    pub fn insert(
        &mut self,
        metric: impl Into<String>,
        task: impl Into<String>,
        response: FixtureResponse,
    ) {
        self.responses
            .insert((metric.into(), task.into()), response);
    }

    pub fn query(&self, metric: &str, task: &str) -> Result<Vec<NormalizedLocation>> {
        let key = (metric.to_string(), task.to_string());
        let response = self
            .responses
            .get(&key)
            .with_context(|| format!("no fixture response for {metric}/{task}"))?;
        Ok(dedup_locations(&response.results))
    }
}
