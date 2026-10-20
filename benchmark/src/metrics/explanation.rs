//! Explanation correctness against the fixed rubric and evidence grounding.

use std::collections::BTreeMap;

use anyhow::{Context, Result};

use super::common::{
    candidate_unavailable_cell, capability, combine_scores, index_unavailable_cell_for,
    recorded_locations, target, unmeasured_cell,
};
use crate::adapters::Citation;
use crate::gold::{ExplanationTask, Gold};
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{ExplanationMetric, GroundingScore, Score};

struct RepositoryScores {
    per_candidate: BTreeMap<String, Score>,
    cells: Vec<Score>,
    grounded: u64,
    attempted: u64,
}

pub fn score(
    manifest: &Manifest,
    gold: &Gold,
    cassette: &ResultCassette,
) -> Result<ExplanationMetric> {
    let mut per_repository = BTreeMap::new();
    let mut per_candidate_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut cells = Vec::new();
    let mut grounded_total = 0u64;
    let mut attempted = 0u64;

    for repository in &manifest.repositories {
        let tasks: Vec<&ExplanationTask> = gold
            .explanation
            .iter()
            .filter(|task| task.repository == repository.id)
            .collect();
        if tasks.is_empty() {
            per_repository.insert(
                repository.id.clone(),
                Score::not_applicable("no_gold_tasks"),
            );
            continue;
        }
        let outcome = score_repository(manifest, &tasks, &repository.id, cassette)?;
        per_repository.insert(
            repository.id.clone(),
            combine_scores(&outcome.cells),
        );
        cells.extend(outcome.cells);
        for (candidate, cell) in outcome.per_candidate {
            per_candidate_parts
                .entry(candidate)
                .or_default()
                .push(cell);
        }
        grounded_total += outcome.grounded;
        attempted += outcome.attempted;
    }

    Ok(ExplanationMetric {
        target: target(manifest, "explanation_correctness"),
        aggregate: combine_scores(&cells),
        per_candidate: per_candidate_parts
            .into_iter()
            .map(|(candidate, cells)| (candidate, combine_scores(&cells)))
            .collect(),
        evidence_grounding: GroundingScore {
            value: if attempted == 0 {
                0.0
            } else {
                grounded_total as f64 / attempted as f64
            },
            attempted,
        },
        per_repository,
    })
}

fn score_repository(
    manifest: &Manifest,
    tasks: &[&ExplanationTask],
    repository: &str,
    cassette: &ResultCassette,
) -> Result<RepositoryScores> {
    let task_count = tasks.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut grounded_total = 0u64;

    let mut attempted = 0u64;

    for candidate in &manifest.candidates {
        let unavailable = candidate_unavailable_cell(candidate, task_count).or_else(|| {
            index_unavailable_cell_for(manifest, candidate, repository, task_count)
        });
        let unsupported = unavailable
            .is_none()
            .then(|| {
                let capability = capability(candidate, "explanation");
                unmeasured_cell(capability, task_count)
            })
            .flatten();
        if let Some(cell) = unavailable.or(unsupported) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }

        let mut correct = 0u64;
        let mut grounded = 0u64;
        for task in tasks {
            let locations = recorded_locations(
                cassette,
                &candidate.id,
                repository,
                "explanation",
                &task.id,
                "default",
            )?;
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
        grounded_total += grounded;
        let cell = Score::measured(correct, task_count, task_count, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(RepositoryScores {
        per_candidate,
        cells,
        grounded: grounded_total,
        attempted,
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
