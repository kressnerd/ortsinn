//! Replay runner: reads the fixed manifest and result cassettes and assembles
//! the report.

use std::path::Path;

use anyhow::{Context, Result};

use crate::gold::Gold;
use crate::manifest::{Manifest, RunMode};
use crate::metrics;
use crate::record::cassette::ResultCassette;
use crate::report::model::{
    CandidateReport, Inputs, LineCountReport, PathDecision, PathDecisionStatus, Report,
    RepositoryReport, RunReport, SCHEMA_VERSION,
};
use crate::report::render;

/// Build the report from fixed inputs and replay cassettes, then write
/// `report.json`/`report.md` into `out_dir`.
pub fn run_replay(manifest_path: &Path, out_dir: &Path) -> Result<Report> {
    let manifest = Manifest::load(manifest_path)?;
    anyhow::ensure!(
        manifest.mode == RunMode::Replay,
        "the fixture manifest must declare replay mode"
    );
    let base = Manifest::base_dir(manifest_path);
    let gold = Gold::load(&manifest, &base)?;
    let cassette = ResultCassette::load(&base.join(&manifest.cassettes.results.path))?;
    let metrics = metrics::score(&manifest, &gold, &cassette)?;
    let gaps = metrics::gaps::compute(&metrics);

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
