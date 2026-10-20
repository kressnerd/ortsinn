//! Report model. `report.json` is the source of truth; `report.md` is
//! rendered from this model.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::manifest::{CandidateSetup, RunMode, Target, TargetStatus};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub run: RunReport,
    pub inputs: Inputs,
    pub candidates: Vec<CandidateReport>,
    pub metrics: Metrics,
    pub gaps: Gaps,
    pub path_decision: PathDecision,
    pub reproduction: Reproduction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub id: String,
    pub mode: RunMode,
    /// External model requests attempted during the run. Replay reads only
    /// local cassettes, so this is zero.
    pub external_requests: u64,
}

/// Everything needed to reproduce the run from the fixed inputs without a
/// candidate install or an external model call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reproduction {
    pub command: String,
    /// Shell prefix that selects the pinned toolchain and zlib path.
    pub toolchain_prefix: String,
    pub manifest_fingerprint: String,
    /// Fingerprint per gold metric; the manifest is the source of truth.
    pub gold_fingerprints: BTreeMap<String, String>,
    pub model_cassette_fingerprint: String,
    pub result_cassette_fingerprint: String,
    pub candidate_install_required: bool,
    pub external_model_calls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inputs {
    pub repositories: Vec<RepositoryReport>,
    pub gold: Vec<GoldSetReport>,
}

/// Identity of one fixed gold set: which metric it scores, its reviewed
/// version, its task count, and its recorded fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldSetReport {
    pub repository: String,
    pub metric: String,
    pub version: u32,
    pub tasks: u64,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryReport {
    pub id: String,
    pub url: String,
    pub revision: String,
    pub commit: String,
    pub language: String,
    pub loc: LineCountReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCountReport {
    pub language_code_lines: u64,
    pub total_code_lines: u64,
    pub tool: String,
    pub command: String,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateReport {
    pub id: String,
    pub version: String,
    pub commit: String,
    pub install: String,
    pub adapter: String,
    pub adapter_version: u32,
    /// Pinned indexers, environment versions, embedding boundary, the
    /// `.git` presence, and the per-repository index outcome.
    pub setup: CandidateSetup,
    pub status: CandidateStatus,
    /// Set exactly when the candidate could not be installed or started.
    pub unavailable_reason: Option<String>,
}

/// Whether the candidate could be driven at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    Available,
    Unavailable,
}

/// Measurement status of one metric cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    Measured,
    Unavailable,
    NotApplicable,
}

/// Metric/task coverage. `not_applicable` cells are excluded from both
/// numerator and denominator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub numerator: u64,
    pub denominator: u64,
    pub value: Option<f64>,
}

impl Coverage {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
            value: (denominator > 0).then(|| numerator as f64 / denominator as f64),
        }
    }
}

/// Fraction of attempted typo tasks for which a candidate returned no result
/// at all. Its denominator always carries the attempted task count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoMatchRate {
    pub value: Option<f64>,
    pub numerator: u64,
    pub denominator: u64,
}

impl NoMatchRate {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            value: (denominator > 0).then(|| numerator as f64 / denominator as f64),
            numerator,
            denominator,
        }
    }
}

/// One measured, unavailable, or not-applicable metric cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub status: CellStatus,
    pub value: Option<f64>,
    pub numerator: u64,
    pub denominator: u64,
    pub attempted: u64,
    pub unavailable: u64,
    pub coverage: Coverage,
    pub reason: Option<String>,
}

impl Score {
    pub fn measured(numerator: u64, denominator: u64, attempted: u64, unavailable: u64) -> Self {
        Self {
            status: CellStatus::Measured,
            value: Some(numerator as f64 / denominator as f64),
            numerator,
            denominator,
            attempted,
            unavailable,
            coverage: Coverage::new(attempted, attempted + unavailable),
            reason: None,
        }
    }

    pub fn unavailable(reason: &str, tasks: u64) -> Self {
        Self {
            status: CellStatus::Unavailable,
            value: None,
            numerator: 0,
            denominator: 0,
            attempted: 0,
            unavailable: tasks,
            coverage: Coverage::new(0, tasks),
            reason: Some(reason.to_string()),
        }
    }

    pub fn not_applicable(reason: &str) -> Self {
        Self {
            status: CellStatus::NotApplicable,
            value: None,
            numerator: 0,
            denominator: 0,
            attempted: 0,
            unavailable: 0,
            coverage: Coverage::new(0, 0),
            reason: Some(reason.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrrScore {
    pub value: f64,
    pub attempted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingScore {
    pub value: f64,
    pub attempted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arm {
    pub score: Score,
    pub per_candidate: BTreeMap<String, Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arms {
    pub default: Arm,
    pub no_expansion: Arm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoMetric {
    pub target: Target,
    pub aggregate: Score,
    pub recall_at_5: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub recall_at_5_per_candidate: BTreeMap<String, Score>,
    pub no_match_rate: NoMatchRate,
    pub no_match_rate_per_candidate: BTreeMap<String, NoMatchRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub mrr_at_10: MrrScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub evidence_grounding: GroundingScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub arms: Arms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub typo: TypoMetric,
    pub natural_language: NaturalLanguageMetric,
    pub explanation: ExplanationMetric,
    pub concept: ConceptMetric,
    pub concept_equivalence: ConceptMetric,
}

/// A measured or unavailable gap. `value` is absent exactly when the gap is
/// unavailable or not applicable; it is never rendered as zero.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    pub value: Option<f64>,
    pub status: CellStatus,
    pub reason: Option<String>,
    pub target_status: Option<TargetStatus>,
    pub best_candidate: Option<String>,
    pub best_value: Option<f64>,
    pub deltas: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gaps {
    pub to_target: BTreeMap<String, Gap>,
    pub to_best: BTreeMap<String, Gap>,
    pub capability_gaps: Vec<CapabilityGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGap {
    pub candidate: String,
    pub capability: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathDecisionStatus {
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathDecision {
    pub status: PathDecisionStatus,
    pub owner: String,
    pub artifact: String,
    pub rule: String,
}
