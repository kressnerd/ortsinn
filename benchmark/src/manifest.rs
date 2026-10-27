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
    /// Inline targets for manifests that do not reference a targets file.
    #[serde(default)]
    pub targets: BTreeMap<String, Target>,
    /// Real target file, relative to the manifest. Mutually exclusive with
    /// inline targets; `--targets` overrides both.
    #[serde(default)]
    pub targets_file: Option<String>,
    /// Published gold minima by repository power. The fixture manifest omits
    /// this and keeps the loader's fixture semantics.
    #[serde(default)]
    pub gold_minimums: Option<GoldMinimums>,
    #[serde(default)]
    pub gold: Vec<GoldSet>,
    pub cassettes: Cassettes,
    #[serde(default)]
    pub index_deadlines: BTreeMap<String, u64>,
}

/// Repository power label. A primary repository carries the full published
/// gold minima; a secondary repository is rendered lower-power and excluded
/// from cross-repository aggregate claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Power {
    Primary,
    Secondary,
}

/// Per-power published gold minima, declared by a real manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoldMinimums {
    pub mode: GoldMinimumsMode,
    pub primary: BTreeMap<String, u64>,
    pub secondary: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GoldMinimumsMode {
    Published,
    Fixture,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub id: String,
    pub url: String,
    pub revision: String,
    pub commit: String,
    pub language: String,
    /// `primary` or `secondary` for a real fixed input. Fixture manifests
    /// leave it unset.
    #[serde(default)]
    pub power: Option<Power>,
    /// Minimum reference-language code lines the input must provide.
    #[serde(default)]
    pub minimum_code_lines: Option<u64>,
    pub loc: LineCount,
    /// Path to the fixed checkout, relative to the manifest directory. Gold
    /// tasks are validated against this checkout before any score runs.
    pub checkout: String,
    /// Present for the generated synthetic monorepo, whose pin is the seed,
    /// generator version, and content fingerprint rather than a git commit.
    #[serde(default)]
    pub synthetic: Option<SyntheticInput>,
}

/// The synthetic input's fixed identity. The monorepo is generated, never
/// committed; this record is the pin a checkout must reproduce.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyntheticInput {
    pub seed: u64,
    pub generator_version: u32,
    /// Recorded fixed-input identifier carried by the generator.
    pub recorded_fingerprint: String,
    /// SHA-256 over the sorted `(relative path, file content digest)` list.
    pub content_fingerprint: String,
    pub code_lines: u64,
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
    /// Where the authored version and commit were observed. Required by the
    /// real manifest; fixture candidates may omit it.
    #[serde(default)]
    pub evidence: Option<CandidateEvidence>,
    #[serde(default)]
    pub capabilities: BTreeMap<String, Capability>,
    #[serde(default)]
    pub setup: CandidateSetup,
    /// Index deadline for this candidate, in seconds; overrides the
    /// manifest-wide public-repository deadline.
    #[serde(default)]
    pub index_deadline_seconds: Option<u64>,
    /// Present when the candidate cannot be started at all; every metric
    /// cell for it is then unavailable with the reason.
    #[serde(default)]
    pub unavailable: Option<Unavailable>,
}

/// Evidence citation for an authored candidate pin. Both values name the
/// source the maintainer observed: the release and the commit.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateEvidence {
    pub version: String,
    pub commit: String,
}

impl Candidate {
    /// The effective per-repository index deadline for this candidate.
    pub fn index_deadline(&self, manifest: &Manifest) -> Option<u64> {
        self.index_deadline_seconds.or_else(|| {
            manifest
                .index_deadlines
                .get("public_repository_seconds")
                .copied()
        })
    }

    /// True when any recorded index exceeded the effective deadline.
    pub fn index_timed_out(&self, manifest: &Manifest) -> bool {
        let Some(deadline) = self.index_deadline(manifest) else {
            return false;
        };
        self.setup
            .indexes
            .iter()
            .any(|index| index.seconds > deadline as f64)
    }

    /// Index records that exceeded the effective deadline. A breach is
    /// derived from the recorded elapsed time, so replay reaches the same
    /// conclusion as the live run.
    pub fn timed_out_indexes<'a>(&'a self, manifest: &Manifest) -> Vec<&'a IndexRecord> {
        let Some(deadline) = self.index_deadline(manifest) else {
            return Vec::new();
        };
        self.setup
            .indexes
            .iter()
            .filter(|index| index.seconds > deadline as f64)
            .collect()
    }
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
    /// Documented setup deviations. Every metric cell a deviation affects
    /// carries its label; the affected cell keeps its measured value.
    #[serde(default)]
    pub setup_deviations: Vec<SetupDeviation>,
}

/// One documented setup deviation: a dependency the candidate could not
/// install or a documented mode it could not use.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SetupDeviation {
    pub dependency: String,
    pub reason: String,
    /// Capabilities the deviation affects. An empty list affects every
    /// capability.
    #[serde(default)]
    pub affected_capabilities: Vec<String>,
    /// Repositories the deviation affects. An empty list affects every
    /// repository.
    #[serde(default)]
    pub affected_repositories: Vec<String>,
}

impl SetupDeviation {
    /// The closed label every affected cell carries.
    pub fn label(&self) -> String {
        format!("setup deviation: {}: {}", self.dependency, self.reason)
    }

    /// True when the deviation affects this capability/repository cell.
    pub fn affects(&self, capability: &str, repository: &str) -> bool {
        (self.affected_capabilities.is_empty()
            || self
                .affected_capabilities
                .iter()
                .any(|affected| affected == capability))
            && (self.affected_repositories.is_empty()
                || self
                    .affected_repositories
                    .iter()
                    .any(|affected| affected == repository))
    }
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
    /// The effective deadline applied to this index, in seconds, so a
    /// timeout can be audited from the report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_seconds: Option<u64>,
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
            match &repository.synthetic {
                Some(synthetic) => {
                    anyhow::ensure!(
                        synthetic.code_lines > 0,
                        "synthetic repository {} has no code lines",
                        repository.id
                    );
                    anyhow::ensure!(
                        !synthetic.content_fingerprint.is_empty(),
                        "synthetic repository {} has no content fingerprint",
                        repository.id
                    );
                    anyhow::ensure!(
                        repository.commit.is_empty()
                            || repository.commit.len() == 40,
                        "repository {} commit must be empty or 40 hex characters",
                        repository.id
                    );
                }
                None => anyhow::ensure!(
                    repository.commit.len() == 40,
                    "repository {} commit must be 40 hex characters",
                    repository.id
                ),
            }
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

    /// Validate the authored candidate pins and their evidence citations.
    /// Fixture candidates omit evidence, so this check is separate from
    /// `load`/`validate` and is run for the real manifest.
    pub fn validate_candidate_pins(&self) -> Result<()> {
        for candidate in &self.candidates {
            anyhow::ensure!(
                !candidate.version.is_empty(),
                "candidate {} has no version",
                candidate.id
            );
            anyhow::ensure!(
                is_commit(&candidate.commit),
                "candidate {} commit {} is not a 40-character commit",
                candidate.id,
                candidate.commit
            );
            anyhow::ensure!(
                !candidate.adapter.is_empty(),
                "candidate {} has no adapter",
                candidate.id
            );
            let evidence = candidate.evidence.as_ref().with_context(|| {
                format!("candidate {} has no evidence record", candidate.id)
            })?;
            anyhow::ensure!(
                !evidence.version.is_empty() && !evidence.commit.is_empty(),
                "candidate {} evidence is incomplete",
                candidate.id
            );
            anyhow::ensure!(
                evidence
                    .version
                    .contains(candidate.version.trim_start_matches('v')),
                "candidate {} version evidence does not name {}: {}",
                candidate.id,
                candidate.version,
                evidence.version
            );
            anyhow::ensure!(
                evidence.commit.contains(&candidate.commit),
                "candidate {} commit evidence does not name {}: {}",
                candidate.id,
                candidate.commit,
                evidence.commit
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

/// A 40-character hexadecimal git commit, as authored in the manifest.
pub fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// Content fingerprint of a fixed input file, in the report's `sha256:<hex>`
/// shape. Gold files and committed cassettes declare this value in the
/// manifest, and the loader compares it before scoring.
pub fn file_fingerprint(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(format!("sha256:{}", hex(&digest)))
}

/// Require a committed fixed input to match its declared digest. The label
/// names the input (for example `gold set fixture-repo typo` or
/// `result cassette`); the error carries the path, the declared value, and
/// the actual value.
pub fn ensure_declared_fingerprint(path: &Path, declared: &str, label: &str) -> Result<String> {
    let actual = file_fingerprint(path)?;
    anyhow::ensure!(
        actual == declared,
        "{label} fingerprint mismatch for {}: expected {}, actual {}",
        path.display(),
        declared,
        actual
    );
    Ok(actual)
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

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = "573b71c3d5cfaba3a83d602f7534605047a01363";

    fn candidate(
        id: &str,
        version: &str,
        commit: &str,
        evidence: Option<CandidateEvidence>,
    ) -> Candidate {
        Candidate {
            id: id.to_string(),
            version: version.to_string(),
            commit: commit.to_string(),
            install: "work/tools/candidate".to_string(),
            adapter: "ocbi".to_string(),
            adapter_version: 1,
            evidence,
            capabilities: BTreeMap::new(),
            setup: CandidateSetup::default(),
            index_deadline_seconds: None,
            unavailable: None,
        }
    }

    fn manifest(candidates: Vec<Candidate>) -> Manifest {
        Manifest {
            schema_version: 1,
            run_id: "test".to_string(),
            mode: RunMode::Replay,
            repositories: Vec::new(),
            candidates,
            targets: BTreeMap::new(),
            targets_file: None,
            gold_minimums: None,
            gold: Vec::new(),
            cassettes: Cassettes {
                results: CassetteRef {
                    path: "results.jsonl".to_string(),
                    fingerprint: String::new(),
                },
                model: CassetteRef {
                    path: "model.jsonl".to_string(),
                    fingerprint: String::new(),
                },
            },
            index_deadlines: BTreeMap::new(),
        }
    }

    fn evidence(version: &str, commit: &str) -> Option<CandidateEvidence> {
        Some(CandidateEvidence {
            version: version.to_string(),
            commit: commit.to_string(),
        })
    }

    /// S5: a setup deviation labels exactly the affected capability and
    /// repository cells; an empty scope affects every cell.
    #[test]
    fn s5_setup_deviation_labels_only_the_affected_scope() {
        let deviation = SetupDeviation {
            dependency: "scip-python".to_string(),
            reason: "documented SCIP indexer unavailable; fallback parser used".to_string(),
            affected_capabilities: vec!["natural_language".to_string()],
            affected_repositories: vec!["fixture-repo".to_string()],
        };
        assert!(deviation.affects("natural_language", "fixture-repo"));
        assert!(!deviation.affects("concept", "fixture-repo"));
        assert!(!deviation.affects("natural_language", "other-repo"));
        assert!(deviation.label().contains("scip-python"));
        assert!(deviation.label().contains("fallback parser"));

        let unscoped = SetupDeviation {
            dependency: "language-server".to_string(),
            reason: "documented language server unavailable".to_string(),
            affected_capabilities: Vec::new(),
            affected_repositories: Vec::new(),
        };
        assert!(unscoped.affects("typo", "any-repository"));
    }

    /// S2: candidate pins require a version, a 40-hex commit, an adapter, and
    /// evidence naming both. A mismatched pin is rejected.
    #[test]
    fn s2_candidate_pins_require_evidence_and_40_hex_commits() {
        let good = manifest(vec![candidate(
            "ocbi",
            "v0.37.0",
            COMMIT,
            evidence(
                "https://github.com/Helweg/open-codebase-index/releases/tag/v0.37.0",
                "https://github.com/Helweg/open-codebase-index/commit/573b71c3d5cfaba3a83d602f7534605047a01363",
            ),
        )]);
        good.validate_candidate_pins().expect("valid candidate pin");

        let short = manifest(vec![candidate("ocbi", "v0.37.0", "573b71c3", evidence("v0.37.0", COMMIT))]);
        let error = short.validate_candidate_pins().expect_err("short commit must fail");
        assert!(format!("{error:#}").contains("40-character commit"), "{error:#}");

        let no_evidence = manifest(vec![candidate("ocbi", "v0.37.0", COMMIT, None)]);
        let error = no_evidence
            .validate_candidate_pins()
            .expect_err("missing evidence must fail");
        assert!(format!("{error:#}").contains("no evidence record"), "{error:#}");

        let version_mismatch = manifest(vec![candidate(
            "ocbi",
            "v0.37.0",
            COMMIT,
            evidence("releases/tag/v0.38.2", COMMIT),
        )]);
        let error = version_mismatch
            .validate_candidate_pins()
            .expect_err("mismatched evidence version must fail");
        assert!(format!("{error:#}").contains("does not name v0.37.0"), "{error:#}");

        let commit_mismatch = manifest(vec![candidate(
            "ocbi",
            "v0.37.0",
            COMMIT,
            evidence("v0.37.0", "5ff375db862c17dbf40168fa3924696baf1839a6"),
        )]);
        let error = commit_mismatch
            .validate_candidate_pins()
            .expect_err("mismatched evidence commit must fail");
        assert!(format!("{error:#}").contains("does not name"), "{error:#}");

        let pin_mismatch = manifest(vec![candidate(
            "ocbi",
            "v0.37.0",
            "5ff375db862c17dbf40168fa3924696baf1839a6",
            evidence("v0.37.0", COMMIT),
        )]);
        let error = pin_mismatch
            .validate_candidate_pins()
            .expect_err("candidate commit/evidence mismatch must fail");
        assert!(format!("{error:#}").contains("does not name"), "{error:#}");
    }
}
