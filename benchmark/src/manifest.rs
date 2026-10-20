//! The manifest is the single fixed-input record for a benchmark run.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Recording mode. Replay is the default and the only mode this change needs
/// for the executable specs; record mode is added by its own scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    Replay,
    Record,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub run_id: String,
    pub mode: RunMode,
    pub repositories: Vec<Repository>,
    pub candidates: Vec<Candidate>,
    pub targets: BTreeMap<String, Target>,
    pub gold: Vec<GoldSet>,
    pub cassettes: Cassettes,
    #[serde(default)]
    pub index_deadlines: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub id: String,
    pub url: String,
    pub revision: String,
    pub commit: String,
    pub language: String,
    pub loc: LineCount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineCount {
    pub language_code_lines: u64,
    pub total_code_lines: u64,
    pub tool: String,
    pub command: String,
    #[serde(default)]
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Candidate {
    pub id: String,
    pub version: String,
    pub commit: String,
    pub install: String,
    pub adapter: String,
    pub adapter_version: u32,
    #[serde(default)]
    pub capabilities: BTreeMap<String, Capability>,
    #[serde(default)]
    pub setup: CandidateSetup,
    /// Present when the candidate cannot be started at all; every metric
    /// cell for it is then unavailable with the reason.
    #[serde(default)]
    pub unavailable: Option<Unavailable>,
}

/// Recorded candidate setup normalization. Each field is optional because the
/// candidates expose different setup surfaces.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CandidateSetup {
    /// Runtime environment versions (for example the Serena Python version).
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    /// Pinned indexer versions (for example CKB's SCIP indexers).
    #[serde(default)]
    pub indexers: BTreeMap<String, String>,
    /// Documented doctor/help output recorded before indexing.
    #[serde(default)]
    pub doctor: Option<String>,
    /// Language support tiers (CKB records Rust as Tier 3).
    #[serde(default)]
    pub language_support: BTreeMap<String, String>,
    /// Embedding boundary configuration (open-codebase-index).
    #[serde(default)]
    pub embedding: Option<EmbeddingConfig>,
    /// `present` when the candidate copy keeps its `.git` directory.
    #[serde(default = "unknown_git_metadata")]
    pub git_metadata: String,
    /// Per-repository index outcome and duration.
    #[serde(default)]
    pub indexes: Vec<IndexRecord>,
}

fn unknown_git_metadata() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub base_url: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexRecord {
    pub repository: String,
    pub status: String,
    pub seconds: f64,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Candidate-level unavailability. A candidate that cannot be installed or
/// started is recorded and the run continues with the others.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Unavailable {
    pub reason: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Measured,
    Unavailable,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capability {
    pub status: CapabilityStatus,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Target {
    pub value: Option<f64>,
    #[serde(default = "unstated")]
    pub status: TargetStatus,
    pub source: String,
}

fn unstated() -> TargetStatus {
    TargetStatus::Unstated
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetStatus {
    Confirmed,
    Proposed,
    Unstated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoldSet {
    pub repository: String,
    pub metric: String,
    pub version: u32,
    pub tasks: u64,
    pub fingerprint: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cassettes {
    pub results: CassetteRef,
    pub model: CassetteRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CassetteRef {
    pub path: String,
    pub fingerprint: String,
}

impl Manifest {
    /// Load and validate a manifest from disk. The manifest path's parent is
    /// the base for every relative fixture path.
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read manifest {}", path.display()))?;
        let manifest: Self = serde_yaml_ng::from_str(&text)
            .with_context(|| format!("parse manifest {}", path.display()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.schema_version == 1,
            "unsupported manifest schema version {}",
            self.schema_version
        );
        anyhow::ensure!(
            !self.repositories.is_empty(),
            "manifest has no repositories"
        );
        anyhow::ensure!(!self.candidates.is_empty(), "manifest has no candidates");
        for repository in &self.repositories {
            anyhow::ensure!(
                repository.commit.len() == 40,
                "repository {} commit must be 40 hex characters",
                repository.id
            );
        }
        for candidate in &self.candidates {
            anyhow::ensure!(
                candidate.commit.len() == 40,
                "candidate {} commit must be 40 hex characters",
                candidate.id
            );
        }
        Ok(())
    }

    pub fn repository(&self, id: &str) -> Option<&Repository> {
        self.repositories
            .iter()
            .find(|repository| repository.id == id)
    }

    pub fn candidate(&self, id: &str) -> Option<&Candidate> {
        self.candidates.iter().find(|candidate| candidate.id == id)
    }

    pub fn gold_set(&self, metric: &str) -> Option<&GoldSet> {
        self.gold.iter().find(|gold| gold.metric == metric)
    }

    /// Content-addressed identity of the fixed inputs. The JSON encoding is
    /// canonical for this model (maps are sorted), so the digest is stable.
    pub fn fingerprint(&self) -> Result<String> {
        let bytes = serde_json::to_vec(self).context("serialize manifest for fingerprint")?;
        let digest = Sha256::digest(bytes);
        Ok(format!("sha256:{}", hex(&digest)))
    }

    pub fn base_dir(path: &Path) -> PathBuf {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }
}

pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}
