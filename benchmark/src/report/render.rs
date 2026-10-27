//! Renders the report model to JSON and Markdown.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{Report, Score};

pub fn write_reports(report: &Report, out_dir: &Path) -> Result<()> {
    report
        .validate()
        .context("validate report before writing it")?;
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create report directory {}", out_dir.display()))?;
    let json = serde_json::to_string_pretty(report).context("serialize report.json")?;
    fs::write(out_dir.join("report.json"), json)
        .with_context(|| format!("write {}", out_dir.join("report.json").display()))?;
    fs::write(out_dir.join("report.md"), markdown(report))
        .with_context(|| format!("write {}", out_dir.join("report.md").display()))?;
    Ok(())
}

pub fn markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# Benchmark comparison report\n\n");
    out.push_str(&format!("Schema version: {}\n\n", report.schema_version));
    out.push_str("## Run\n\n");
    out.push_str(&format!("- Run: `{}`\n", report.run.id));
    out.push_str(&format!("- Mode: `{:?}`\n", report.run.mode));
    out.push_str(&format!(
        "- External model requests: {}\n\n",
        report.run.external_requests
    ));
    out.push_str("## Reproduction\n\n");
    let reproduction = &report.reproduction;
    out.push_str(&format!(
        "- Command: `. ./env.sh && {}`\n",
        reproduction.command
    ));
    out.push_str(&format!(
        "- Toolchain prefix: `{}`\n",
        reproduction.toolchain_prefix
    ));
    out.push_str(&format!(
        "- Candidate install required: {}\n",
        if reproduction.candidate_install_required {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(&format!(
        "- External model calls: {}\n",
        if reproduction.external_model_calls {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(&format!(
        "- Manifest fingerprint: `{}`\n",
        reproduction.manifest_fingerprint
    ));
    for (metric, fingerprint) in &reproduction.gold_fingerprints {
        out.push_str(&format!("- Gold fingerprint `{metric}`: `{fingerprint}`\n"));
    }
    out.push_str(&format!(
        "- Model cassette fingerprint: `{}`\n",
        reproduction.model_cassette_fingerprint
    ));
    out.push_str(&format!(
        "- Result cassette fingerprint: `{}`\n\n",
        reproduction.result_cassette_fingerprint
    ));
    out.push_str("## Fixed inputs\n\n");
    for repository in &report.inputs.repositories {
        out.push_str(&format!(
            "- `{}` revision `{}` commit `{}` ({} code lines)\n",
            repository.id,
            repository.revision,
            repository.commit,
            repository.loc.language_code_lines
        ));
    }
    out.push('\n');
    out.push_str("### Gold sets\n\n");
    for set in &report.inputs.gold {
        out.push_str(&format!(
            "- `{}` v{}: {} tasks, fingerprint `{}`\n",
            set.metric, set.version, set.tasks, set.fingerprint
        ));
    }
    out.push('\n');
    out.push_str("## Candidates\n\n");
    for candidate in &report.candidates {
        out.push_str(&format!(
            "- `{}` version `{}` commit `{}` (adapter `{}` v{})\n",
            candidate.id,
            candidate.version,
            candidate.commit,
            candidate.adapter,
            candidate.adapter_version
        ));
        out.push_str(&format!("  - Install: `{}`\n", candidate.install));
        if candidate.status == super::model::CandidateStatus::Unavailable {
            out.push_str(&format!(
                "  - Candidate unavailable: {}\n",
                candidate
                    .unavailable_reason
                    .as_deref()
                    .unwrap_or("no reason recorded")
            ));
        }
        out.push_str(&format!(
            "  - Git metadata: `{}`\n",
            candidate.setup.git_metadata
        ));
        for index in &candidate.setup.indexes {
            let deadline = index
                .deadline_seconds
                .map(|deadline| format!(" of a {deadline}s deadline"))
                .unwrap_or_default();
            out.push_str(&format!(
                "  - Index `{}`: {} in {:.2}s{}{}\n",
                index.repository,
                index.status,
                index.seconds,
                deadline,
                index
                    .reason
                    .as_ref()
                    .map(|reason| format!(" ({reason})"))
                    .unwrap_or_default()
            ));
        }
        for (language, tier) in &candidate.setup.language_support {
            out.push_str(&format!("  - Language support `{language}`: {tier}\n"));
        }
        for (tool, version) in &candidate.setup.indexers {
            out.push_str(&format!("  - Indexer `{tool}`: {version}\n"));
        }
        for (tool, version) in &candidate.setup.environment {
            out.push_str(&format!("  - Environment `{tool}`: {version}\n"));
        }
        if let Some(embedding) = &candidate.setup.embedding {
            out.push_str(&format!(
                "  - Embedding provider `{}` at `{}`\n",
                embedding.provider, embedding.base_url
            ));
        }
        if let Some(doctor) = &candidate.setup.doctor {
            out.push_str(&format!("  - Doctor: `{doctor}`\n"));
        }
        for deviation in &candidate.setup.setup_deviations {
            out.push_str(&format!(
                "  - Setup deviation `{}`: {} (capabilities: {}; repositories: {})\n",
                deviation.dependency,
                deviation.reason,
                join_or_all(&deviation.affected_capabilities),
                join_or_all(&deviation.affected_repositories)
            ));
        }
    }
    out.push('\n');
    render_metrics(&mut out, report);
    out
}

fn render_metrics(out: &mut String, report: &Report) {
    let metrics = &report.metrics;
    out.push_str("## Typo recall\n\n");
    score_line(out, "Typo recall @1", &metrics.typo.aggregate);
    if metrics.typo.target.status == crate::manifest::TargetStatus::Proposed {
        out.push_str("- Typo recall @1 target is the proposed target 0.90 (not confirmed)\n");
    }
    score_line(out, "Typo recall @5", &metrics.typo.recall_at_5);
    match metrics.typo.no_match_rate.value {
        Some(value) => out.push_str(&format!(
            "- Typo no-match rate: {value:.2} ({} of {} tasks)\n",
            metrics.typo.no_match_rate.numerator, metrics.typo.no_match_rate.denominator
        )),
        None => out.push_str("- Typo no-match rate: unavailable\n"),
    }
    out.push('\n');
    coverage_line(out, "Typo", &metrics.typo.aggregate.coverage);
    repository_lines(out, "Typo recall @1", &metrics.typo.per_repository);
    out.push('\n');
    out.push_str("## Natural-language retrieval\n\n");
    score_line(out, "NL recall @5", &metrics.natural_language.aggregate);
    out.push_str(&format!(
        "- NL MRR @10: {:.2}\n",
        metrics.natural_language.mrr_at_10.value
    ));
    out.push_str(&format!(
        "- NL query errors: {} (each counted as a miss, not unavailable)\n",
        metrics.natural_language.query_errors
    ));
    for (label, arm) in &metrics.natural_language.proxy_arms {
        match arm.score.value {
            Some(value) => out.push_str(&format!(
                "- NL `{label}` arm (proxy only; excluded from NL recall and MRR): {value:.2} ({} of {})\n",
                arm.score.numerator, arm.score.denominator
            )),
            None => out.push_str(&format!(
                "- NL `{label}` arm (proxy only; excluded from NL recall and MRR): unavailable ({})\n",
                arm.score.reason.as_deref().unwrap_or("no reason")
            )),
        }
    }
    coverage_line(
        out,
        "Natural-language",
        &metrics.natural_language.aggregate.coverage,
    );
    repository_lines(
        out,
        "NL recall @5",
        &metrics.natural_language.per_repository,
    );
    out.push('\n');
    out.push_str("## Explanation correctness\n\n");
    score_line(
        out,
        "Explanation correctness",
        &metrics.explanation.aggregate,
    );
    out.push_str(&format!(
        "- Explanation evidence grounding: {:.2}\n",
        metrics.explanation.evidence_grounding.value
    ));
    coverage_line(
        out,
        "Explanation",
        &metrics.explanation.aggregate.coverage,
    );
    repository_lines(
        out,
        "Explanation correctness",
        &metrics.explanation.per_repository,
    );
    out.push('\n');
    out.push_str("## Concept recall\n\n");
    score_line(out, "Concept recall @5", &metrics.concept.aggregate);
    coverage_line(out, "Concept", &metrics.concept.aggregate.coverage);
    score_line(
        out,
        "Concept no-expansion arm",
        &metrics.concept.arms.no_expansion.score,
    );
    coverage_line(
        out,
        "Concept no-expansion arm",
        &metrics.concept.arms.no_expansion.score.coverage,
    );
    score_line(
        out,
        "Concept-equivalence recall @5",
        &metrics.concept_equivalence.aggregate,
    );
    coverage_line(
        out,
        "Concept-equivalence",
        &metrics.concept_equivalence.aggregate.coverage,
    );
    score_line(
        out,
        "Concept-equivalence no-expansion arm",
        &metrics.concept_equivalence.arms.no_expansion.score,
    );
    coverage_line(
        out,
        "Concept-equivalence no-expansion arm",
        &metrics.concept_equivalence.arms.no_expansion.score.coverage,
    );
    repository_lines(out, "Concept recall @5", &metrics.concept.per_repository);
    repository_lines(
        out,
        "Concept-equivalence recall @5",
        &metrics.concept_equivalence.per_repository,
    );
    out.push('\n');
    out.push_str("## Measured gaps\n\n");
    for (name, gap) in &report.gaps.to_target {
        let label = if gap.target_status == Some(crate::manifest::TargetStatus::Proposed) {
            "to proposed target"
        } else {
            "to target"
        };
        match gap.value {
            Some(value) => {
                out.push_str(&format!("- {name} {label}: {value:+.2}\n"));
            }
            None => out.push_str(&format!(
                "- {name} {label}: {} ({})\n",
                gap_status(gap.status),
                gap.reason.as_deref().unwrap_or("no reason")
            )),
        }
    }
    for (name, gap) in &report.gaps.to_best {
        match &gap.best_candidate {
            Some(best) => out.push_str(&format!(
                "- {name} best measured: {} ({:.2})\n",
                best,
                gap.best_value.unwrap_or_default()
            )),
            None => out.push_str(&format!(
                "- {name} best measured: {} ({})\n",
                gap_status(gap.status),
                gap.reason.as_deref().unwrap_or("no reason")
            )),
        }
    }
    for gap in &report.gaps.capability_gaps {
        out.push_str(&format!(
            "- Capability gap: {} {} ({})\n",
            gap.candidate, gap.capability, gap.reason
        ));
    }
    out.push('\n');
    out.push_str("## Limitations\n\n");
    for limitation in &report.limitations {
        out.push_str(&format!("- {limitation}\n"));
    }
    out.push('\n');
    out.push_str("## Path decision\n\n");
    out.push_str(&format!(
        "- Status: `{:?}`\n- Owner: `{}`\n- Artifact: `{}`\n- Rule: {}\n",
        report.path_decision.status,
        report.path_decision.owner,
        report.path_decision.artifact,
        report.path_decision.rule
    ));
}

/// Default terminal width when `--width` is absent and `COLUMNS` is unset.
pub const DEFAULT_TERMINAL_WIDTH: usize = 80;

/// Terminal width requested by the environment. The CLI's `--width` option
/// is the explicit mechanism; `COLUMNS` is only a fallback for interactive
/// use, so a pipe never supplies a width.
pub fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse().ok())
        .filter(|columns| *columns >= 20)
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

/// Render the concise run summary the CLI prints. Every metric row is a
/// single line regardless of width: the value is right-aligned to the
/// requested width instead of wrapped. Prose is wrapped.
pub fn terminal_summary(report: &Report, width: usize) -> String {
    let width = width.max(20);
    let metrics = &report.metrics;
    let mut out = String::new();
    out.push_str("Benchmark comparison\n");
    out.push_str(&format!(
        "Run: {} ({})\n",
        report.run.id,
        match report.run.mode {
            crate::manifest::RunMode::Replay => "replay",
            crate::manifest::RunMode::Record => "record",
        }
    ));
    out.push_str(&format!(
        "Candidates: {}\n",
        report
            .candidates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    push_wrapped(
        &mut out,
        &format!(
            "Path decision: deferred to {} ({})",
            report.path_decision.owner, report.path_decision.artifact
        ),
        width,
    );
    out.push('\n');

    out.push_str("Typo recall\n");
    metric_row(&mut out, "Typo recall @1", &value_text(&metrics.typo.aggregate), width);
    metric_row(
        &mut out,
        "Typo recall @5",
        &value_text(&metrics.typo.recall_at_5),
        width,
    );
    metric_row(
        &mut out,
        "Typo no-match rate",
        &match metrics.typo.no_match_rate.value {
            Some(value) => format!(
                "{value:.2} ({} of {})",
                metrics.typo.no_match_rate.numerator, metrics.typo.no_match_rate.denominator
            ),
            None => "unavailable".to_string(),
        },
        width,
    );
    metric_row(
        &mut out,
        "Typo coverage",
        &coverage_text(&metrics.typo.aggregate.coverage),
        width,
    );
    out.push('\n');

    out.push_str("Natural-language retrieval\n");
    metric_row(
        &mut out,
        "NL recall @5",
        &value_text(&metrics.natural_language.aggregate),
        width,
    );
    metric_row(
        &mut out,
        "NL MRR @10",
        &format!("{:.2}", metrics.natural_language.mrr_at_10.value),
        width,
    );
    metric_row(
        &mut out,
        "NL query errors",
        &metrics.natural_language.query_errors.to_string(),
        width,
    );
    metric_row(
        &mut out,
        "NL coverage",
        &coverage_text(&metrics.natural_language.aggregate.coverage),
        width,
    );
    out.push('\n');

    out.push_str("Explanation correctness\n");
    metric_row(
        &mut out,
        "Explanation correctness",
        &value_text(&metrics.explanation.aggregate),
        width,
    );
    metric_row(
        &mut out,
        "Explanation grounding",
        &format!("{:.2}", metrics.explanation.evidence_grounding.value),
        width,
    );
    metric_row(
        &mut out,
        "Explanation coverage",
        &coverage_text(&metrics.explanation.aggregate.coverage),
        width,
    );
    out.push('\n');

    out.push_str("Concept recall\n");
    metric_row(
        &mut out,
        "Concept recall @5",
        &value_text(&metrics.concept.aggregate),
        width,
    );
    metric_row(
        &mut out,
        "Concept-equivalence recall @5",
        &value_text(&metrics.concept_equivalence.aggregate),
        width,
    );
    metric_row(
        &mut out,
        "Concept coverage",
        &coverage_text(&metrics.concept.aggregate.coverage),
        width,
    );
    metric_row(
        &mut out,
        "Concept no-expansion coverage",
        &coverage_text(&metrics.concept.arms.no_expansion.score.coverage),
        width,
    );
    out
}

fn value_text(score: &Score) -> String {
    match score.value {
        Some(value) => format!(
            "{value:.2} ({} of {})",
            score.numerator, score.denominator
        ),
        None => format!(
            "{} ({})",
            status(score),
            score.reason.as_deref().unwrap_or("no reason")
        ),
    }
}

fn coverage_text(coverage: &super::model::Coverage) -> String {
    match coverage.value {
        Some(value) => format!(
            "{} of {} ({value:.2})",
            coverage.numerator, coverage.denominator
        ),
        None => format!("{} of {} (no applicable tasks)", coverage.numerator, coverage.denominator),
    }
}

/// One metric row. The value is right-aligned to `width`; it is never
/// wrapped, so a label and its value always stay on the same line.
fn metric_row(out: &mut String, label: &str, value: &str, width: usize) {
    let content = 2 + label.chars().count() + 1 + value.chars().count();
    let gap = width.saturating_sub(content).max(1);
    out.push_str(&format!("  {label}:{}{value}\n", " ".repeat(gap)));
}

fn push_wrapped(out: &mut String, text: &str, width: usize) {
    let mut line = String::new();
    for word in text.split_whitespace() {
        let projected = if line.is_empty() {
            word.chars().count()
        } else {
            line.chars().count() + 1 + word.chars().count()
        };
        if projected > width && !line.is_empty() {
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push_str(&line);
        out.push('\n');
    }
}

fn gap_status(status: super::model::CellStatus) -> &'static str {
    status_label(status)
}

fn coverage_line(out: &mut String, label: &str, coverage: &super::model::Coverage) {
    match coverage.value {
        Some(value) => out.push_str(&format!(
            "- {label} coverage: {} of {} ({value:.2})\n",
            coverage.numerator, coverage.denominator
        )),
        None => out.push_str(&format!(
            "- {label} coverage: {} of {} (no applicable tasks)\n",
            coverage.numerator, coverage.denominator
        )),
    }
}

fn repository_lines(out: &mut String, label: &str, values: &std::collections::BTreeMap<String, Score>) {
    for (repository, score) in values {
        score_line(out, &format!("{label} for `{repository}`"), score);
    }
}

fn score_line(out: &mut String, label: &str, score: &Score) {
    match score.value {
        Some(value) => out.push_str(&format!(
            "- {label}: {value:.2} ({} of {})\n",
            score.numerator, score.denominator
        )),
        None => out.push_str(&format!(
            "- {label}: {} ({})\n",
            status(score),
            score.reason.as_deref().unwrap_or("no reason")
        )),
    }
    for deviation in &score.setup_deviations {
        out.push_str(&format!("  - {deviation}\n"));
    }
}

fn join_or_all(values: &[String]) -> String {
    if values.is_empty() {
        "all".to_string()
    } else {
        values.join(", ")
    }
}

fn status(score: &Score) -> &'static str {
    status_label(score.status)
}

fn status_label(status: super::model::CellStatus) -> &'static str {
    use super::model::CellStatus;
    match status {
        CellStatus::Measured => "measured",
        CellStatus::Unavailable => "unavailable",
        CellStatus::NotApplicable => "not applicable",
    }
}
