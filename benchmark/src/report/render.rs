//! Renders the report model to JSON and Markdown.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{Report, Score};

pub fn write_reports(report: &Report, out_dir: &Path) -> Result<()> {
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
    out.push_str(&format!("- Command: `{}`\n", report.run.command));
    out.push_str(&format!(
        "- Manifest fingerprint: `{}`\n\n",
        report.run.manifest_fingerprint
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
    }
    out.push('\n');
    render_metrics(&mut out, report);
    out
}

fn render_metrics(out: &mut String, report: &Report) {
    let metrics = &report.metrics;
    out.push_str("## Typo recall\n\n");
    score_line(out, "Typo recall @1", &metrics.typo.aggregate);
    score_line(out, "Typo recall @5", &metrics.typo.recall_at_5);
    out.push('\n');
    out.push_str("## Natural-language retrieval\n\n");
    score_line(out, "NL recall @5", &metrics.natural_language.aggregate);
    out.push_str(&format!(
        "- NL MRR @10: {:.2}\n\n",
        metrics.natural_language.mrr_at_10.value
    ));
    out.push_str("## Explanation correctness\n\n");
    score_line(
        out,
        "Explanation correctness",
        &metrics.explanation.aggregate,
    );
    out.push_str(&format!(
        "- Explanation evidence grounding: {:.2}\n\n",
        metrics.explanation.evidence_grounding.value
    ));
    out.push_str("## Concept recall\n\n");
    score_line(out, "Concept recall @5", &metrics.concept.aggregate);
    score_line(
        out,
        "Concept-equivalence recall @5",
        &metrics.concept_equivalence.aggregate,
    );
    out.push('\n');
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
}

fn status(score: &Score) -> &'static str {
    use super::model::CellStatus;
    match score.status {
        CellStatus::Measured => "measured",
        CellStatus::Unavailable => "unavailable",
        CellStatus::NotApplicable => "not applicable",
    }
}
