//! Normalized result cassettes for replay.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::adapters::{ExplanationResponse, NormalizedLocation};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecordedResult {
    pub candidate: String,
    /// Candidate version the result was recorded against. Replay compares it
    /// with the manifest's pinned identity before scoring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_version: Option<String>,
    /// Candidate source commit recorded with the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_commit: Option<String>,
    pub repository: String,
    /// Commit the normalized result was recorded against. Replay compares it
    /// with the manifest's fixed input before scoring anything.
    pub repository_commit: String,
    pub metric: String,
    pub task: String,
    pub arm: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<NormalizedLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<ExplanationResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Recorded model responses this normalized result depends on. Replay
    /// must find every one of them in the model cassette.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_requests: Vec<ModelDependency>,
}

/// One model response a recorded result depends on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelDependency {
    pub key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub query: String,
}

/// Result cassettes indexed by `(candidate, metric, task, arm)`.
#[derive(Debug, Clone, Default)]
pub struct ResultCassette {
    entries: BTreeMap<(String, String, String, String), RecordedResult>,
}

impl ResultCassette {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read result cassette {}", path.display()))?;
        let mut entries = BTreeMap::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let result: RecordedResult = serde_json::from_str(line).with_context(|| {
                format!(
                    "parse result cassette {} line {}",
                    path.display(),
                    index + 1
                )
            })?;
            let key = (
                result.candidate.clone(),
                result.metric.clone(),
                result.task.clone(),
                result.arm.clone(),
            );
            anyhow::ensure!(
                entries.insert(key, result).is_none(),
                "duplicate result cassette entry in {} line {}",
                path.display(),
                index + 1
            );
        }
        Ok(Self { entries })
    }

    pub fn get(
        &self,
        candidate: &str,
        metric: &str,
        task: &str,
        arm: &str,
    ) -> Option<&RecordedResult> {
        self.entries.get(&(
            candidate.to_string(),
            metric.to_string(),
            task.to_string(),
            arm.to_string(),
        ))
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = &RecordedResult> {
        self.entries.values()
    }
}

/// One recorded model response. The key is the canonical request key; the
/// raw transport metadata is not part of the recording.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelRecord {
    pub key: String,
    pub candidate: String,
    pub task: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub query: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub layer: String,
    /// Canonical OpenAI-compatible request body, without headers, request
    /// IDs, or timestamps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<serde_json::Value>,
    #[serde(default)]
    pub response: serde_json::Value,
}

/// Model cassettes indexed by canonical request key. Replay serves only
/// these responses; it never calls a model endpoint.
#[derive(Debug, Clone, Default)]
pub struct ModelCassette {
    entries: BTreeMap<String, ModelRecord>,
}

impl ModelCassette {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read model cassette {}", path.display()))?;
        let mut entries = BTreeMap::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let record: ModelRecord = serde_json::from_str(line).with_context(|| {
                format!("parse model cassette {} line {}", path.display(), index + 1)
            })?;
            anyhow::ensure!(
                entries.insert(record.key.clone(), record).is_none(),
                "duplicate model cassette key in {} line {}",
                path.display(),
                index + 1
            );
        }
        Ok(Self { entries })
    }

    pub fn get(&self, key: &str) -> Option<&ModelRecord> {
        self.entries.get(key)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ModelRecord> {
        self.entries.values()
    }
}

/// Content fingerprint of a cassette file, in the report's `sha256:<hex>`
/// shape. The recording and the replay of the same temporary cassette
/// produce the same value.
pub fn fingerprint(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read cassette {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!("sha256:{hex}"))
}
