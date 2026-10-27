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

/// Normalized query text the model key digests. Whitespace is the only
/// normalization: formatting alone cannot change a recorded key.
pub fn normalize_query(query: &str) -> String {
    query.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// SHA-256 digest of the normalized query text, in lowercase hex.
pub fn query_digest(query: &str) -> String {
    let digest = Sha256::digest(normalize_query(query).as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Canonical model-cassette key for one explanation request. The query digest
/// makes a changed gold query a distinct key instead of a reused response.
pub fn explanation_key(candidate: &str, task: &str, query: &str) -> String {
    format!(
        "model:explanation:{candidate}:{task}:sha256:{}",
        query_digest(query)
    )
}

/// Canonical model-cassette key for one embedding request.
pub fn embedding_key(candidate: &str, model: &str, query: &str) -> String {
    format!(
        "model:embedding:{candidate}:{model}:sha256:{}",
        query_digest(query)
    )
}

/// The identifying parts of a model-cassette key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedModelKey<'a> {
    pub layer: &'a str,
    pub candidate: &'a str,
    /// Explanation task id or embedding model name.
    pub ident: &'a str,
    pub digest: Option<&'a str>,
}

/// Split `model:<layer>:<candidate>:<ident>[:sha256:<digest>]`.
pub fn parse_model_key(key: &str) -> Option<ParsedModelKey<'_>> {
    let mut parts = key.split(':');
    if parts.next()? != "model" {
        return None;
    }
    let layer = parts.next()?;
    let candidate = parts.next()?;
    let ident = parts.next()?;
    let rest: Vec<&str> = parts.collect();
    let digest = match rest.as_slice() {
        [] => None,
        ["sha256", digest] if !digest.is_empty() => Some(*digest),
        _ => return None,
    };
    Some(ParsedModelKey {
        layer,
        candidate,
        ident,
        digest,
    })
}

impl ModelRecord {
    /// True when the stored key carries the SHA-256 digest of this record's
    /// own query and names the same candidate and identity.
    pub fn carries_query_digest(&self) -> bool {
        let Some(parsed) = parse_model_key(&self.key) else {
            return false;
        };
        if parsed.candidate != self.candidate {
            return false;
        }
        let digest = query_digest(&self.query);
        if parsed.digest != Some(digest.as_str()) {
            return false;
        }
        match parsed.layer {
            "explanation" => parsed.ident == self.task,
            "embedding" => true,
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold::Gold;
    use crate::manifest::Manifest;
    use crate::record::keys;

    fn fixture_manifest() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e/manifest.yaml")
    }

    #[test]
    fn s18_committed_fixture_declares_and_carries_query_digests() {
        let manifest_path = fixture_manifest();
        let manifest = Manifest::load(&manifest_path).expect("load fixture manifest");
        let base = Manifest::base_dir(&manifest_path);
        let model_path = base.join(&manifest.cassettes.model.path);
        assert_eq!(
            fingerprint(&model_path).expect("fingerprint committed model cassette"),
            manifest.cassettes.model.fingerprint,
            "recomputed model cassette digest"
        );

        let models = ModelCassette::load(&model_path).expect("load committed model cassette");
        assert!(models.entry_count() > 0, "model cassette is empty");
        for record in models.entries() {
            assert!(
                record.carries_query_digest(),
                "model cassette key {} does not carry the digest of its query",
                record.key
            );
        }

        let results = ResultCassette::load(&base.join(&manifest.cassettes.results.path))
            .expect("load committed result cassette");
        let gold = Gold::load(&manifest, &base).expect("load fixture gold");
        for result in results.entries() {
            for dependency in &result.model_requests {
                let resolved = keys::resolve(
                    &gold,
                    &result.repository,
                    &result.metric,
                    &result.candidate,
                    &result.task,
                    &dependency.key,
                )
                .expect("resolve the recorded model dependency");
                assert_eq!(
                    resolved.key, dependency.key,
                    "result cassette key for {}/{} is not the query-digest key",
                    result.candidate, result.task
                );
            }
        }
    }
}
