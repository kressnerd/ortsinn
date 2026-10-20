//! Explanation correctness against the fixed rubric and evidence grounding.

use std::collections::BTreeMap;

use anyhow::{Context, Result};

use super::common::{aggregate, capability, recorded_locations, target, unmeasured_cell};
use crate::adapters::Citation;
use crate::gold::{ExplanationTask, Gold};
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{ExplanationMetric, GroundingScore, Score};

pub fn score(
    manifest: &Manifest,
    gold: &Gold,
    cassette: &ResultCassette,
) -> Result<ExplanationMetric> {
    let tasks = gold.explanation.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut correct_total = 0u64;
    let mut grounded_total = 0u64;
    let mut attempted = 0u64;

    for candidate in &manifest.candidates {
        let capability = capability(candidate, "explanation");
        if let Some(cell) = unmeasured_cell(capability, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut correct = 0u64;
        let mut grounded = 0u64;
        for task in &gold.explanation {
            let locations =
                recorded_locations(cassette, &candidate.id, "explanation", &task.id, "default")?;
            anyhow::ensure!(
                locations.is_empty(),
                "explanation responses carry citations, not locations"
            );
            let record = cassette
                .get(&candidate.id, "explanation", &task.id, "default")
                .with_context(|| {
                    format!(
                        "missing explanation for candidate {} task {}",
                        candidate.id, task.id
                    )
                })?;
            let explanation = record.explanation.as_ref().with_context(|| {
                format!(
                    "result cassette entry for candidate {} task {} has no explanation",
                    candidate.id, task.id
                )
            })?;
            attempted += 1;
            if is_grounded(explanation.citations.as_slice(), task) {
                grounded += 1;
            }
            if facts_present(&explanation.text, task) && is_grounded(&explanation.citations, task) {
                correct += 1;
            }
        }
        correct_total += correct;
        grounded_total += grounded;
        let cell = Score::measured(correct, tasks, tasks, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(ExplanationMetric {
        target: target(manifest, "explanation_correctness"),
        aggregate: aggregate(&cells, correct_total, attempted),
        per_candidate,
        evidence_grounding: GroundingScore {
            value: if attempted == 0 {
                0.0
            } else {
                grounded_total as f64 / attempted as f64
            },
            attempted,
        },
    })
}

fn facts_present(text: &str, task: &ExplanationTask) -> bool {
    let text = text.to_lowercase();
    task.required_facts
        .iter()
        .all(|forms| forms.iter().any(|form| text.contains(&form.to_lowercase())))
}

fn is_grounded(citations: &[Citation], task: &ExplanationTask) -> bool {
    citations.iter().any(|citation| {
        citation.path == task.target.path
            && task.acceptable_evidence.iter().any(|range| {
                citation.start_line >= range.start_line && citation.end_line <= range.end_line
            })
    })
}
