//! Replay runner: reads the fixed manifest and replay cassettes and assembles
//! the report.

use std::fmt;
use std::path::Path;

use anyhow::{Context, Result};

use crate::gold::Gold;
use crate::manifest::{Manifest, RunMode};
use crate::metrics;
use crate::record::cassette::{ModelCassette, ResultCassette};
use crate::report::model::{
    CandidateReport, Inputs, LineCountReport, PathDecision, PathDecisionStatus, Report,
    RepositoryReport, RunReport, SCHEMA_VERSION,
};
use crate::report::render;

/// Process exit code for a run that cannot produce a complete report.
pub const EXIT_INPUT_ERROR: i32 = 2;

/// A failed run. The message names the fixed input that stopped the run.
#[derive(Debug, Clone)]
pub struct RunFailure {
    pub exit_code: i32,
    pub message: String,
}

impl fmt::Display for RunFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RunFailure {}

pub type RunResult<T> = std::result::Result<T, RunFailure>;

/// Build the report from fixed inputs and replay cassettes, then write
/// `report.json`/`report.md` into `out_dir`.
///
/// A missing recorded response stops the run before any report is written.
pub fn run_replay(manifest_path: &Path, out_dir: &Path) -> RunResult<Report> {
    run_replay_inner(manifest_path, out_dir).map_err(|error| RunFailure {
        exit_code: EXIT_INPUT_ERROR,
        message: format!("{error:#}"),
    })
}

fn run_replay_inner(manifest_path: &Path, out_dir: &Path) -> Result<Report> {
    let manifest = Manifest::load(manifest_path)?;
    anyhow::ensure!(
        manifest.mode == RunMode::Replay,
        "the fixture manifest must declare replay mode"
    );
    let base = Manifest::base_dir(manifest_path);
    let gold = Gold::load(&manifest, &base)?;
    let cassette = ResultCassette::load(&base.join(&manifest.cassettes.results.path))?;
    let models = ModelCassette::load(&base.join(&manifest.cassettes.model.path))?;
    verify_recorded_model_responses(&cassette, &models)?;
    let metrics = metrics::score(&manifest, &gold, &cassette)?;
    let gaps = metrics::gaps::compute(&manifest, &metrics);

    let command = format!(
        "benchmark run --manifest {} --mode replay --out {}",
        manifest_path.display(),
        out_dir.display()
    );
    let report = Report {
        schema_version: SCHEMA_VERSION,
        run: RunReport {
            id: manifest.run_id.clone(),
            mode: manifest.mode,
            command,
            manifest_fingerprint: manifest.fingerprint()?,
            recording_fingerprint: manifest.cassettes.model.fingerprint.clone(),
            external_requests: 0,
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
            })
            .collect(),
        metrics,
        gaps,
        path_decision: PathDecision {
            status: PathDecisionStatus::Deferred,
            owner: "maintainer--ortsinn".to_string(),
            artifact: "docs/decisions/benchmark-path.md".to_string(),
            rule: "decide after the report; no pre-registered build threshold".to_string(),
        },
    };

    render::write_reports(&report, out_dir)
        .with_context(|| format!("write report into {}", out_dir.display()))?;
    Ok(report)
}

/// Replay serves recorded model responses only. A normalized result that
/// depends on a model response must find it in the model cassette; a missing
/// key fails the run instead of falling back to a live call or a partial
/// report.
fn verify_recorded_model_responses(
    results: &ResultCassette,
    models: &ModelCassette,
) -> Result<()> {
    for result in results.entries() {
        for dependency in &result.model_requests {
            if models.get(&dependency.key).is_some() {
                continue;
            }
            let query = if dependency.query.is_empty() {
                &result.task
            } else {
                &dependency.query
            };
            anyhow::bail!(
                "missing recorded model response {} for candidate {} (repository {}, query {}, layer model)",
                dependency.key,
                result.candidate,
                result.repository,
                query
            );
        }
    }
    Ok(())
}
