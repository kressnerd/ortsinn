//! Replay and record runners for the fixed benchmark inputs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::gold::Gold;
use crate::manifest::{Candidate, CandidateSetup, GoldMinimumsMode, Manifest, RunMode};
use crate::metrics;
use crate::record::cassette::{self, ModelCassette, ModelRecord, ResultCassette};
use crate::record::keys;
use crate::record::proxy::ModelRecorder;
use crate::report::model::{
    CandidateReport, CandidateStatus, CellStatus, Gaps, GoldSetReport, Inputs, LineCountReport,
    Metrics, PathDecision, PathDecisionStatus, Report, Reproduction, RepositoryReport, RunReport,
    SCHEMA_VERSION,
};
use crate::report::render;

/// Process exit code for a run that cannot produce a complete report.
pub const EXIT_INPUT_ERROR: i32 = 2;

/// Shell prefix that reproduces the run's toolchain. The zlib directory is
/// referenced through the environment variable `env.sh` exports, so the
/// report stays portable across workstations.
pub const TOOLCHAIN_PREFIX: &str =
    "RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu LD_LIBRARY_PATH=\"${ZLIB_LIBRARY_PATH}\"";

/// A failed run. The message names the fixed input that stopped the run.
#[derive(Debug, Clone)]
pub struct RunFailure {
    pub exit_code: i32,
    pub message: String,
}

impl RunFailure {
    /// Turn a run error into the process failure the CLI reports. A replay
    /// miss keeps its typed identity instead of being flattened to a string.
    pub fn from_error(error: anyhow::Error) -> Self {
        if let Some(miss) = error.downcast_ref::<MissingRecordedResponse>() {
            return Self {
                exit_code: EXIT_INPUT_ERROR,
                message: miss.to_string(),
            };
        }
        Self {
            exit_code: EXIT_INPUT_ERROR,
            message: format!("{error:#}"),
        }
    }
}

impl fmt::Display for RunFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RunFailure {}

/// A normalized result depends on a model response the cassette does not
/// hold. Replay stops with exit code 2 and names the missing layer, key,
/// candidate, repository, and query; it never falls back to a live call or a
/// partial report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingRecordedResponse {
    pub key: String,
    pub candidate: String,
    pub repository: String,
    pub task: String,
    pub query: String,
    pub layer: String,
}

impl fmt::Display for MissingRecordedResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "missing recorded {} response {} for candidate {} (repository {}, query {}, task {})",
            self.layer, self.key, self.candidate, self.repository, self.query, self.task
        )
    }
}

impl std::error::Error for MissingRecordedResponse {}

/// A report would claim a measured comparison without the evidence the
/// publication floor requires. The run exits 2 and writes neither report
/// file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementFloorUnmet {
    pub missing_families: Vec<String>,
    pub missing_repositories: Vec<String>,
}

impl fmt::Display for MeasurementFloorUnmet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if !self.missing_families.is_empty() {
            parts.push(format!(
                "no measured candidate cell for families {}",
                self.missing_families.join(", ")
            ));
        }
        if !self.missing_repositories.is_empty() {
            parts.push(format!(
                "no measured cell for repositories {}",
                self.missing_repositories.join(", ")
            ));
        }
        write!(formatter, "publication floor not met: {}", parts.join("; "))
    }
}

impl std::error::Error for MeasurementFloorUnmet {}

pub type RunResult<T> = std::result::Result<T, RunFailure>;

/// Build the report from fixed inputs and replay cassettes, then write
/// `report.json`/`report.md` into `out_dir`.
///
/// A missing recorded response stops the run before any report is written.
pub fn run_replay(manifest_path: &Path, out_dir: &Path) -> RunResult<Report> {
    run_replay_inner(manifest_path, out_dir).map_err(RunFailure::from_error)
}

/// Run the fixture candidates once against the loopback recording boundary,
/// write the model and result cassettes under `out_dir`, then score them the
/// same way replay does.
///
/// The committed fixture cassettes are never overwritten: `out_dir` is a
/// temporary location owned by the caller.
pub fn run_record(manifest_path: &Path, out_dir: &Path, model_endpoint: &str) -> RunResult<Report> {
    run_record_inner(manifest_path, out_dir, model_endpoint).map_err(RunFailure::from_error)
}

fn run_replay_inner(manifest_path: &Path, out_dir: &Path) -> Result<Report> {
    let manifest = Manifest::load(manifest_path)?;
    anyhow::ensure!(
        manifest.mode == RunMode::Replay,
        "the fixture manifest must declare replay mode"
    );
    let base = Manifest::base_dir(manifest_path);
    let gold = Gold::load(&manifest, &base)?;

    let (results_path, results_recorded) =
        resolve_cassette(out_dir, &base, &manifest.cassettes.results.path);
    let (model_path, model_recorded) =
        resolve_cassette(out_dir, &base, &manifest.cassettes.model.path);
    let result_fingerprint = cassette_identity(
        &results_path,
        &manifest.cassettes.results,
        results_recorded,
        "result",
    )?;
    let model_fingerprint = cassette_identity(
        &model_path,
        &manifest.cassettes.model,
        model_recorded,
        "model",
    )?;

    let cassette = ResultCassette::load(&results_path)?;
    let models = ModelCassette::load(&model_path)?;
    verify_fixed_inputs(&manifest, &cassette)?;
    verify_recorded_model_responses(&gold, &cassette, &models)?;
    let metrics = metrics::score(&manifest, &gold, &cassette)?;
    let gaps = metrics::gaps::compute(&manifest, &metrics);
    ensure_publication_floor(&manifest, &metrics)?;

    let command = format!(
        "cargo run -p benchmark -- run --manifest {} --mode replay --out {}",
        manifest_path.display(),
        out_dir.display()
    );
    let reproduction = reproduction(
        &manifest,
        command,
        model_fingerprint,
        result_fingerprint,
        false,
        false,
    )?;
    let report = assemble_report(
        &manifest,
        metrics,
        gaps,
        RunMode::Replay,
        0,
        reproduction,
    );

    render::write_reports(&report, out_dir)
        .with_context(|| format!("write report into {}", out_dir.display()))?;
    Ok(report)
}

fn run_record_inner(manifest_path: &Path, out_dir: &Path, model_endpoint: &str) -> Result<Report> {
    let manifest = Manifest::load(manifest_path)?;
    let base = Manifest::base_dir(manifest_path);
    let gold = Gold::load(&manifest, &base)?;
    let source = ResultCassette::load(&base.join(&manifest.cassettes.results.path))?;
    verify_fixed_inputs(&manifest, &source)?;

    let recorder = ModelRecorder::new(model_endpoint)?;
    let mut models: BTreeMap<String, ModelRecord> = BTreeMap::new();
    for result in source.entries() {
        for dependency in &result.model_requests {
            let request = keys::resolve(
                &gold,
                &result.repository,
                &result.metric,
                &result.candidate,
                &result.task,
                &dependency.key,
            )?;
            anyhow::ensure!(
                request.layer == "explanation",
                "record mode cannot record a {} dependency for {}/{}",
                request.layer,
                result.candidate,
                result.task
            );
            if models.contains_key(&request.key) {
                continue;
            }
            let record = recorder
                .explanation(&result.candidate, &result.task, &request.query)
                .with_context(|| {
                    format!(
                        "record model response {} for candidate {}",
                        request.key, result.candidate
                    )
                })?;
            anyhow::ensure!(
                record.key == request.key,
                "recorded model key {} does not match the gold query key {}",
                record.key,
                request.key
            );
            models.insert(record.key.clone(), record);
        }
    }
    for candidate in &manifest.candidates {
        let Some(embedding) = &candidate.setup.embedding else {
            continue;
        };
        let model = embedding
            .model
            .clone()
            .unwrap_or_else(|| "fixture-embedding".to_string());
        let record = recorder
            .embedding(&candidate.id, &model)
            .with_context(|| format!("record embedding for candidate {}", candidate.id))?;
        models.insert(record.key.clone(), record);
    }

    let results_path = out_dir.join(&manifest.cassettes.results.path);
    let model_path = out_dir.join(&manifest.cassettes.model.path);
    write_jsonl(&results_path, source.entries())?;
    write_jsonl(&model_path, models.values())?;

    let cassette = ResultCassette::load(&results_path)?;
    let recorded_models = ModelCassette::load(&model_path)?;
    verify_recorded_model_responses(&gold, &cassette, &recorded_models)?;
    let metrics = metrics::score(&manifest, &gold, &cassette)?;
    let gaps = metrics::gaps::compute(&manifest, &metrics);
    ensure_publication_floor(&manifest, &metrics)?;

    let command = format!(
        "cargo run -p benchmark -- run --manifest {} --mode record --model-endpoint {} --out {}",
        manifest_path.display(),
        model_endpoint,
        out_dir.display()
    );
    let reproduction = reproduction(
        &manifest,
        command,
        cassette::fingerprint(&model_path)?,
        cassette::fingerprint(&results_path)?,
        false,
        true,
    )?;
    let report = assemble_report(&manifest, metrics, gaps, RunMode::Record, 0, reproduction);

    render::write_reports(&report, out_dir)
        .with_context(|| format!("write report into {}", out_dir.display()))?;
    Ok(report)
}

/// A published report must carry at least one measured candidate cell for
/// each required family and at least one measured cell for every manifest
/// repository. The floor is evaluated before either report file is written,
/// so an all-unavailable run never claims a measured comparison; unavailable
/// candidates remain listed whenever the floor is met.
///
/// A fixture manifest (`gold_minimums.mode: fixture`, or no published
/// minimums) keeps the harness contract tests that deliberately leave a
/// metric unmeasured.
fn ensure_publication_floor(manifest: &Manifest, metrics: &Metrics) -> Result<()> {
    if !publication_floor_required(manifest) {
        return Ok(());
    }
    let unmet = publication_floor(manifest, metrics);
    if unmet.missing_families.is_empty() && unmet.missing_repositories.is_empty() {
        return Ok(());
    }
    Err(unmet.into())
}

fn publication_floor_required(manifest: &Manifest) -> bool {
    manifest
        .gold_minimums
        .as_ref()
        .is_some_and(|minimums| minimums.mode == GoldMinimumsMode::Published)
}

fn publication_floor(manifest: &Manifest, metrics: &Metrics) -> MeasurementFloorUnmet {
    let families = [
        ("typo", &metrics.typo.per_candidate),
        ("natural-language", &metrics.natural_language.per_candidate),
        ("explanation", &metrics.explanation.per_candidate),
        ("concept", &metrics.concept.per_candidate),
    ];
    let missing_families: Vec<String> = families
        .into_iter()
        .filter(|(_, cells)| !cells.values().any(|cell| cell.status == CellStatus::Measured))
        .map(|(name, _)| name.to_string())
        .collect();

    let mut measured_repositories = BTreeSet::new();
    for cells in [
        &metrics.typo.per_repository,
        &metrics.natural_language.per_repository,
        &metrics.explanation.per_repository,
        &metrics.concept.per_repository,
    ] {
        for (repository, cell) in cells {
            if cell.status == CellStatus::Measured {
                measured_repositories.insert(repository.clone());
            }
        }
    }
    let missing_repositories: Vec<String> = manifest
        .repositories
        .iter()
        .map(|repository| repository.id.clone())
        .filter(|repository| !measured_repositories.contains(repository))
        .collect();

    MeasurementFloorUnmet {
        missing_families,
        missing_repositories,
    }
}

/// Prefer cassettes recorded next to the report directory; the committed
/// fixture cassettes are the fallback. The boolean is true when the resolved
/// cassette is a recorded local one whose fingerprint must be computed.
fn resolve_cassette(out_dir: &Path, base: &Path, relative: &str) -> (PathBuf, bool) {
    let local = out_dir.join(relative);
    if local.is_file() {
        (local, true)
    } else {
        (base.join(relative), false)
    }
}

/// Fingerprint a cassette and, for a committed fixed-input cassette, require
/// the manifest's declared digest. A locally recorded cassette was written by
/// this run, so its computed digest becomes the report identity; the manifest
/// still declares the committed cassette's digest.
fn cassette_identity(
    path: &Path,
    declared: &crate::manifest::CassetteRef,
    local: bool,
    layer: &str,
) -> Result<String> {
    if local {
        return cassette::fingerprint(path);
    }
    crate::manifest::ensure_declared_fingerprint(
        path,
        &declared.fingerprint,
        &format!("{layer} cassette"),
    )
}

fn write_jsonl<T: serde::Serialize>(path: &Path, entries: impl Iterator<Item = T>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create cassette directory {}", parent.display()))?;
    }
    let mut text = String::new();
    for entry in entries {
        text.push_str(&serde_json::to_string(&entry).context("serialize cassette entry")?);
        text.push('\n');
    }
    fs::write(path, text).with_context(|| format!("write cassette {}", path.display()))
}

fn reproduction(
    manifest: &Manifest,
    command: String,
    model_cassette_fingerprint: String,
    result_cassette_fingerprint: String,
    candidate_install_required: bool,
    external_model_calls: bool,
) -> Result<Reproduction> {
    Ok(Reproduction {
        command,
        toolchain_prefix: TOOLCHAIN_PREFIX.to_string(),
        manifest_fingerprint: manifest.fingerprint()?,
        gold_fingerprints: manifest
            .gold
            .iter()
            .map(|set| (set.metric.clone(), set.fingerprint.clone()))
            .collect(),
        model_cassette_fingerprint,
        result_cassette_fingerprint,
        candidate_install_required,
        external_model_calls,
    })
}

fn assemble_report(
    manifest: &Manifest,
    metrics: Metrics,
    gaps: Gaps,
    mode: RunMode,
    external_requests: u64,
    reproduction: Reproduction,
) -> Report {
    let limitations = report_limitations(manifest, &metrics);
    Report {
        schema_version: SCHEMA_VERSION,
        run: RunReport {
            id: manifest.run_id.clone(),
            mode,
            external_requests,
        },
        inputs: Inputs {
            repositories: manifest
                .repositories
                .iter()
                .map(|repository| RepositoryReport {
                    id: repository.id.clone(),
                    url: repository.url.clone(),
                    revision: repository.revision.clone(),
                    commit: repository.commit.clone(),
                    language: repository.language.clone(),
                    loc: LineCountReport {
                        language_code_lines: repository.loc.language_code_lines,
                        total_code_lines: repository.loc.total_code_lines,
                        tool: repository.loc.tool.clone(),
                        command: repository.loc.command.clone(),
                        exclusions: repository.loc.exclusions.clone(),
                    },
                })
                .collect(),
            gold: manifest
                .gold
                .iter()
                .map(|set| GoldSetReport {
                    repository: set.repository.clone(),
                    metric: set.metric.clone(),
                    version: set.version,
                    tasks: set.tasks,
                    fingerprint: set.fingerprint.clone(),
                })
                .collect(),
        },
        candidates: manifest
            .candidates
            .iter()
            .map(|candidate| CandidateReport {
                id: candidate.id.clone(),
                version: candidate.version.clone(),
                commit: candidate.commit.clone(),
                install: candidate.install.clone(),
                adapter: candidate.adapter.clone(),
                adapter_version: candidate.adapter_version,
                setup: recorded_setup(manifest, candidate),
                status: if candidate.unavailable.is_some() {
                    CandidateStatus::Unavailable
                } else {
                    CandidateStatus::Available
                },
                unavailable_reason: candidate
                    .unavailable
                    .as_ref()
                    .map(|unavailable| unavailable.reason.clone()),
            })
            .collect(),
        metrics,
        gaps,
        limitations,
        path_decision: PathDecision {
            status: PathDecisionStatus::Deferred,
            owner: "maintainer--ortsinn".to_string(),
            artifact: "docs/decisions/benchmark-path.md".to_string(),
            rule: "decide after the report; no pre-registered build threshold".to_string(),
        },
        reproduction,
    }
}

/// Closed limitation labels derived from the fixed inputs and the measured
/// report. They state what the measurement does not cover rather than hiding
/// it in prose.
fn report_limitations(manifest: &Manifest, metrics: &Metrics) -> Vec<String> {
    use crate::manifest::{CapabilityStatus, TargetStatus};
    use std::collections::BTreeSet;

    let mut limitations = BTreeSet::new();
    for (key, target) in &manifest.targets {
        if target.status == TargetStatus::Proposed {
            let value = target
                .value
                .map(|value| format!("{value}"))
                .unwrap_or_else(|| "unstated".to_string());
            limitations.insert(format!(
                "target {key} = {value} is proposed, not a confirmed acceptance criterion"
            ));
        }
    }
    for candidate in &manifest.candidates {
        for (name, capability) in &candidate.capabilities {
            if capability.status == CapabilityStatus::Unavailable {
                limitations.insert(format!(
                    "capability {} {name} unavailable: {}",
                    candidate.id,
                    capability
                        .reason
                        .as_deref()
                        .unwrap_or("no reason recorded")
                ));
            }
        }
        for deviation in &candidate.setup.setup_deviations {
            limitations.insert(format!(
                "candidate {} {}",
                candidate.id,
                deviation.label()
            ));
        }
        for (language, tier) in &candidate.setup.language_support {
            if tier.contains('3') {
                limitations.insert(format!(
                    "candidate {} language support {language}: {tier}",
                    candidate.id
                ));
            }
        }
    }
    for label in metrics.natural_language.proxy_arms.keys() {
        limitations.insert(format!(
            "arm {label} is a labelled proxy excluded from natural-language recall and mean reciprocal rank"
        ));
    }
    limitations.insert(
        "explanation rubric credits only the listed surface forms; a paraphrase outside them is not credited"
            .to_string(),
    );
    limitations.into_iter().collect()
}

/// Report the setup with deadline breaches applied: an index that exceeded
/// its deadline is recorded as `timed_out` with the reason and keeps its
/// measured elapsed time.
fn recorded_setup(manifest: &Manifest, candidate: &Candidate) -> CandidateSetup {
    let mut setup = candidate.setup.clone();
    let Some(deadline) = candidate.index_deadline(manifest) else {
        return setup;
    };
    for index in &mut setup.indexes {
        index.deadline_seconds = Some(deadline);
        if index.seconds > deadline as f64 {
            index.status = "timed_out".to_string();
            index.reason = Some("index_timeout".to_string());
        }
    }
    setup
}

/// Every recorded result carries the commit it was recorded against. A
/// manifest that names a different commit would score measurements against
/// the wrong fixed input, so the run stops before writing any report.
fn verify_fixed_inputs(manifest: &Manifest, results: &ResultCassette) -> Result<()> {
    for result in results.entries() {
        let candidate = manifest.candidate(&result.candidate).with_context(|| {
            format!(
                "fixed-input mismatch: the result cassette names candidate {}, which the manifest does not fix",
                result.candidate
            )
        })?;
        let version = result.candidate_version.as_deref().with_context(|| {
            format!(
                "fixed-input mismatch: the result cassette for candidate {} has no recorded candidate version",
                result.candidate
            )
        })?;
        if version != candidate.version {
            anyhow::bail!(
                "candidate identity mismatch for {}: expected version {}, actual version {}",
                candidate.id,
                candidate.version,
                version
            );
        }
        let commit = result.candidate_commit.as_deref().with_context(|| {
            format!(
                "fixed-input mismatch: the result cassette for candidate {} has no recorded candidate commit",
                result.candidate
            )
        })?;
        if commit != candidate.commit {
            anyhow::bail!(
                "candidate identity mismatch for {}: expected commit {}, actual commit {}",
                candidate.id,
                candidate.commit,
                commit
            );
        }
        let repository = manifest.repository(&result.repository).with_context(|| {
            format!(
                "fixed-input mismatch: the result cassette names repository {}, which the manifest does not fix",
                result.repository
            )
        })?;
        if result.repository_commit != repository.commit {
            anyhow::bail!(
                "fixed-input mismatch for repository {}: expected commit {}, actual commit {}",
                repository.id,
                result.repository_commit,
                repository.commit
            );
        }
    }
    Ok(())
}

/// Replay serves recorded model responses only. Every result's dependency is
/// resolved against the current gold query, so a changed query is a missing
/// response instead of a reused answer; the run stops before any report.
fn verify_recorded_model_responses(
    gold: &Gold,
    results: &ResultCassette,
    models: &ModelCassette,
) -> Result<()> {
    for record in models.entries() {
        anyhow::ensure!(
            record.carries_query_digest(),
            "model cassette key {} does not carry the SHA-256 digest of its query",
            record.key
        );
    }
    for result in results.entries() {
        for dependency in &result.model_requests {
            let request = keys::resolve(
                gold,
                &result.repository,
                &result.metric,
                &result.candidate,
                &result.task,
                &dependency.key,
            )?;
            if models.get(&request.key).is_some() && dependency.key == request.key {
                continue;
            }
            return Err(MissingRecordedResponse {
                key: request.key,
                candidate: result.candidate.clone(),
                repository: result.repository.clone(),
                task: result.task.clone(),
                query: request.query,
                layer: request.layer.to_string(),
            }
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::model::Score;
    use std::fs;

    fn floor_manifest_dir(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("benchmark-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create floor fixture directory");
        fs::write(root.join("results.jsonl"), "").expect("write empty result cassette");
        fs::write(
            root.join("manifest.yaml"),
            r#"schema_version: 1
run_id: floor-test
mode: replay
repositories:
  - id: repo-a
    url: "test://repo-a"
    revision: r1
    commit: "1111111111111111111111111111111111111111"
    language: python
    checkout: .
    loc: {language_code_lines: 1, total_code_lines: 1, tool: test, command: test, exclusions: []}
candidates:
  - id: candidate-a
    version: v1
    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    install: test
    adapter: fixture
    adapter_version: 1
gold_minimums:
  mode: published
  primary: {typo: 0, natural_language: 0, explanation: 0, concept: 0, concept_equivalence: 0}
  secondary: {typo: 0, natural_language: 0, explanation: 0, concept: 0, concept_equivalence: 0}
cassettes:
  results: {path: results.jsonl, fingerprint: "sha256:00"}
  model: {path: model.jsonl, fingerprint: "sha256:00"}
"#,
        )
        .expect("write floor manifest");
        root
    }

    fn empty_metrics(manifest: &Manifest, cassette: &ResultCassette) -> Metrics {
        let gold = Gold {
            typo: Vec::new(),
            natural_language: Vec::new(),
            explanation: Vec::new(),
            concept: Vec::new(),
            concept_equivalence: Vec::new(),
            sets: Vec::new(),
        };
        metrics::score(manifest, &gold, cassette).expect("score an empty run")
    }

    fn measure_every_family(metrics: &mut Metrics) {
        for (candidate, repository) in [
            (&mut metrics.typo.per_candidate, &mut metrics.typo.per_repository),
            (
                &mut metrics.natural_language.per_candidate,
                &mut metrics.natural_language.per_repository,
            ),
            (
                &mut metrics.explanation.per_candidate,
                &mut metrics.explanation.per_repository,
            ),
            (
                &mut metrics.concept.per_candidate,
                &mut metrics.concept.per_repository,
            ),
        ] {
            candidate.insert("candidate-a".to_string(), Score::measured(1, 1, 1, 0));
            repository.insert("repo-a".to_string(), Score::measured(1, 1, 1, 0));
        }
    }

    #[test]
    fn s16_publication_floor_requires_a_measured_cell_per_family_and_repository() {
        let root = floor_manifest_dir("publication-floor");
        let manifest = Manifest::load(&root.join("manifest.yaml")).expect("load floor manifest");
        let cassette =
            ResultCassette::load(&root.join("results.jsonl")).expect("load empty cassette");
        let mut metrics = empty_metrics(&manifest, &cassette);

        let unmet = publication_floor(&manifest, &metrics);
        assert_eq!(
            unmet.missing_families,
            vec![
                "typo".to_string(),
                "natural-language".to_string(),
                "explanation".to_string(),
                "concept".to_string(),
            ]
        );
        assert_eq!(unmet.missing_repositories, vec!["repo-a".to_string()]);
        assert!(unmet.to_string().contains("publication floor not met"));
        assert!(ensure_publication_floor(&manifest, &metrics).is_err());

        let mut repository_only = empty_metrics(&manifest, &cassette);
        repository_only
            .typo
            .per_repository
            .insert("repo-a".to_string(), Score::measured(1, 1, 1, 0));
        let unmet = publication_floor(&manifest, &repository_only);
        assert_eq!(unmet.missing_families.len(), 4);
        assert!(unmet.missing_repositories.is_empty());

        measure_every_family(&mut metrics);
        assert!(ensure_publication_floor(&manifest, &metrics).is_ok());

        let mut fixture = Manifest::load(&root.join("manifest.yaml")).expect("load fixture mode");
        fixture.gold_minimums = None;
        let unmeasured = empty_metrics(&fixture, &cassette);
        assert!(
            ensure_publication_floor(&fixture, &unmeasured).is_ok(),
            "a fixture manifest keeps its unmeasured metric"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
