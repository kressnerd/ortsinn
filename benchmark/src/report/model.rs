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
    /// Closed, rendered limitation labels: the proposed target, every
    /// unavailable capability reason, setup deviations, labelled proxy arms,
    /// and the explanation rubric's surface-form rule.
    pub limitations: Vec<String>,
    pub path_decision: PathDecision,
    pub reproduction: Reproduction,
}

impl Report {
    /// Validate the closed status and coverage invariants before write. A
    /// production report with an inconsistent coverage count, a measured
    /// status without a value, or a missing fingerprint is never published.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema_version == SCHEMA_VERSION,
            "report schema version {} is not {}",
            self.schema_version,
            SCHEMA_VERSION
        );
        anyhow::ensure!(
            !self.limitations.is_empty(),
            "report has no limitations"
        );
        anyhow::ensure!(
            self.limitations
                .windows(2)
                .all(|pair| pair[0] < pair[1]),
            "report limitations are not sorted and unique"
        );
        anyhow::ensure!(
            !self.reproduction.command.trim().is_empty(),
            "report has no reproduction command"
        );
        for (label, fingerprint) in [
            ("manifest", &self.reproduction.manifest_fingerprint),
            (
                "model cassette",
                &self.reproduction.model_cassette_fingerprint,
            ),
            (
                "result cassette",
                &self.reproduction.result_cassette_fingerprint,
            ),
        ] {
            anyhow::ensure!(
                fingerprint.starts_with("sha256:"),
                "{label} fingerprint is not a sha256 value: {fingerprint}"
            );
        }
        for metric in [
            "typo",
            "natural_language",
            "explanation",
            "concept",
            "concept_equivalence",
        ] {
            anyhow::ensure!(
                self.reproduction.gold_fingerprints.contains_key(metric),
                "report has no {metric} gold fingerprint"
            );
        }
        for (label, score) in self.scores() {
            validate_score(&label, score)?;
        }
        for (name, gap) in &self.gaps.to_target {
            validate_gap(name, gap)?;
        }
        for (name, gap) in &self.gaps.to_best {
            validate_gap(name, gap)?;
        }
        Ok(())
    }

    fn scores(&self) -> Vec<(String, &Score)> {
        let metrics = &self.metrics;
        let mut scores: Vec<(String, &Score)> = Vec::new();
        macro_rules! add {
            ($label:expr, $score:expr $(,)?) => {
                scores.push(($label, $score))
            };
        }
        add!("typo aggregate".into(), &metrics.typo.aggregate);
        add!("typo recall @5".into(), &metrics.typo.recall_at_5);
        for (candidate, cell) in &metrics.typo.per_candidate {
            add!(format!("typo {candidate}"), cell);
        }
        for (repository, cell) in &metrics.typo.per_repository {
            add!(format!("typo repository {repository}"), cell);
        }
        add!(
            "natural-language aggregate".into(),
            &metrics.natural_language.aggregate,
        );
        for (repository, cell) in &metrics.natural_language.per_repository {
            add!(format!("natural-language repository {repository}"), cell);
        }
        for (candidate, cell) in &metrics.natural_language.per_candidate {
            add!(format!("natural-language {candidate}"), cell);
        }
        for (label, arm) in &metrics.natural_language.proxy_arms {
            add!(format!("natural-language proxy {label}"), &arm.score);
            for (candidate, cell) in &arm.per_candidate {
                add!(format!("natural-language proxy {label} {candidate}"), cell);
            }
        }
        add!(
            "explanation aggregate".into(),
            &metrics.explanation.aggregate,
        );
        for (repository, cell) in &metrics.explanation.per_repository {
            add!(format!("explanation repository {repository}"), cell);
        }
        for (candidate, cell) in &metrics.explanation.per_candidate {
            add!(format!("explanation {candidate}"), cell);
        }
        for (name, metric) in [
            ("concept", &metrics.concept),
            ("concept-equivalence", &metrics.concept_equivalence),
        ] {
            add!(format!("{name} aggregate"), &metric.aggregate);
            for (repository, cell) in &metric.per_repository {
                add!(format!("{name} repository {repository}"), cell);
            }
            for (arm_name, arm) in [
                ("default", &metric.arms.default),
                ("no-expansion", &metric.arms.no_expansion),
            ] {
                add!(format!("{name} {arm_name}"), &arm.score);
                for (candidate, cell) in &arm.per_candidate {
                    add!(format!("{name} {arm_name} {candidate}"), cell);
                }
            }
        }
        scores
    }
}

fn validate_score(label: &str, score: &Score) -> anyhow::Result<()> {
    anyhow::ensure!(
        score.setup_deviations.iter().all(|deviation| !deviation.trim().is_empty()),
        "{label} carries an empty setup-deviation label"
    );
    anyhow::ensure!(
        score
            .setup_deviations
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
        "{label} setup-deviation labels are not sorted and unique"
    );
    let expected_denominator = score.attempted + score.unavailable;
    anyhow::ensure!(
        score.coverage.numerator == score.attempted,
        "{label} coverage numerator {} does not match attempted {}",
        score.coverage.numerator,
        score.attempted
    );
    anyhow::ensure!(
        score.coverage.denominator == expected_denominator,
        "{label} coverage denominator {} does not match attempted {} plus unavailable {}",
        score.coverage.denominator,
        score.attempted,
        score.unavailable
    );
    match score.status {
        CellStatus::Measured => {
            anyhow::ensure!(
                score.value.is_some(),
                "{label} is measured without a value"
            );
            anyhow::ensure!(
                score.denominator > 0,
                "{label} is measured with a zero denominator"
            );
            let value = score.value.unwrap_or_default();
            let expected = score.numerator as f64 / score.denominator as f64;
            anyhow::ensure!(
                (value - expected).abs() < 1e-9,
                "{label} value {value} does not match {} of {}",
                score.numerator,
                score.denominator
            );
        }
        CellStatus::Unavailable | CellStatus::NotApplicable => {
            anyhow::ensure!(
                score.value.is_none(),
                "{label} is not measured but carries a value"
            );
            anyhow::ensure!(
                score.reason.is_some(),
                "{label} is not measured without a reason"
            );
            anyhow::ensure!(
                score.numerator == 0 && score.denominator == 0,
                "{label} is not measured but carries a fraction"
            );
        }
    }
    anyhow::ensure!(
        score.coverage.value.is_none()
            == (score.coverage.denominator == 0),
        "{label} coverage value does not match its denominator"
    );
    if let Some(value) = score.coverage.value {
        let expected = score.coverage.numerator as f64 / score.coverage.denominator as f64;
        anyhow::ensure!(
            (value - expected).abs() < 1e-9,
            "{label} coverage value {value} does not match {} of {}",
            score.coverage.numerator,
            score.coverage.denominator
        );
    }
    Ok(())
}

fn validate_gap(label: &str, gap: &Gap) -> anyhow::Result<()> {
    match gap.status {
        CellStatus::Measured => {
            anyhow::ensure!(gap.value.is_some(), "{label} gap is measured without a value");
        }
        CellStatus::Unavailable | CellStatus::NotApplicable => {
            anyhow::ensure!(
                gap.value.is_none(),
                "{label} gap is not measured but carries a value"
            );
        }
    }
    Ok(())
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
    /// Setup-deviation labels that apply to this cell. An affected cell keeps
    /// its measured value; the label never replaces or zeroes it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub setup_deviations: Vec<String>,
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
            setup_deviations: Vec::new(),
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
            setup_deviations: Vec::new(),
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
            setup_deviations: Vec::new(),
        }
    }

    /// Attach one setup-deviation label. Labels are sorted and unique, and a
    /// measured value is left untouched.
    pub fn add_setup_deviation(&mut self, label: String) {
        if !label.is_empty() {
            self.setup_deviations.push(label);
            self.setup_deviations.sort();
            self.setup_deviations.dedup();
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
    /// Recall @1 for each fixed repository, so a multi-repository run never
    /// collapses into one aggregate.
    pub per_repository: BTreeMap<String, Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub mrr_at_10: MrrScore,
    /// Queries that returned an error. Each counts as a miss for its task and
    /// is reported separately from the recall value.
    pub query_errors: u64,
    pub query_errors_per_candidate: BTreeMap<String, u64>,
    /// Clearly labelled proxy arms measured from the same questions. They are
    /// never folded into the recall value, coverage, or MRR above.
    pub proxy_arms: BTreeMap<String, Arm>,
    /// Recall @5 for each fixed repository.
    pub per_repository: BTreeMap<String, Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub evidence_grounding: GroundingScore,
    /// Correctness for each fixed repository.
    pub per_repository: BTreeMap<String, Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMetric {
    pub target: Target,
    pub aggregate: Score,
    pub per_candidate: BTreeMap<String, Score>,
    pub arms: Arms,
    /// Default-arm recall @5 for each fixed repository.
    pub per_repository: BTreeMap<String, Score>,
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

impl Gap {
    /// A gap without a measured value. It carries a closed reason and no
    /// numeric value, so a zero can never stand in for a missing measurement.
    pub fn unavailable(reason: &str, target_status: Option<TargetStatus>) -> Self {
        Self {
            value: None,
            status: CellStatus::Unavailable,
            reason: Some(reason.to_string()),
            target_status,
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        }
    }

    /// A gap that does not apply because no task was scoreable, for example a
    /// metric whose every candidate is not applicable.
    pub fn not_applicable(reason: &str, target_status: Option<TargetStatus>) -> Self {
        Self {
            value: None,
            status: CellStatus::NotApplicable,
            reason: Some(reason.to_string()),
            target_status,
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        }
    }
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
