//! Natural-language recall at five and mean reciprocal rank at ten.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    candidate_unavailable_cell, capability, combine_scores, first_rank, hit_at,
    index_unavailable_cell_for, recorded_locations, target, unmeasured_cell,
};
use crate::gold::{Gold, GoldTask};
use crate::manifest::{CapabilityStatus, Manifest};
use crate::record::cassette::ResultCassette;
use crate::report::model::{Arm, MrrScore, NaturalLanguageMetric, Score};

struct RepositoryScores {
    per_candidate: BTreeMap<String, Score>,
    cells: Vec<Score>,
    attempted: u64,
    reciprocal_sum: f64,
    query_errors: u64,
    query_errors_per_candidate: BTreeMap<String, u64>,
}

pub fn score(
    manifest: &Manifest,
    gold: &Gold,
    cassette: &ResultCassette,
) -> Result<NaturalLanguageMetric> {
    let mut per_repository = BTreeMap::new();
    let mut per_candidate_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut repository_cells = Vec::new();
    let mut query_errors = 0u64;
    let mut query_errors_per_candidate = BTreeMap::new();
    let mut reciprocal_sum = 0.0f64;
    let mut attempted = 0u64;

    for repository in &manifest.repositories {
        let tasks: Vec<&GoldTask> = gold
            .natural_language
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
        repository_cells.extend(outcome.cells);
        for (candidate, cell) in outcome.per_candidate {
            per_candidate_parts
                .entry(candidate)
                .or_default()
                .push(cell);
        }
        query_errors += outcome.query_errors;
        reciprocal_sum += outcome.reciprocal_sum;
        attempted += outcome.attempted;
        for (candidate, count) in outcome.query_errors_per_candidate {
            *query_errors_per_candidate.entry(candidate).or_insert(0) += count;
        }
    }

    let proxies = proxy_arms(manifest, gold, cassette)?;

    Ok(NaturalLanguageMetric {
        target: target(manifest, "natural_language_recall_at_5"),
        aggregate: combine_scores(&repository_cells),
        per_candidate: per_candidate_parts
            .into_iter()
            .map(|(candidate, cells)| (candidate, combine_scores(&cells)))
            .collect(),
        mrr_at_10: MrrScore {
            value: if attempted == 0 {
                0.0
            } else {
                reciprocal_sum / attempted as f64
            },
            attempted,
        },
        query_errors,
        query_errors_per_candidate,
        proxy_arms: proxies,
        per_repository,
    })
}

fn score_repository(
    manifest: &Manifest,
    tasks: &[&GoldTask],
    repository: &str,
    cassette: &ResultCassette,
) -> Result<RepositoryScores> {
    let task_count = tasks.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut attempted = 0u64;
    let mut reciprocal_sum = 0.0f64;
    let mut query_errors = 0u64;
    let mut query_errors_per_candidate = BTreeMap::new();

    for candidate in &manifest.candidates {
        let unavailable = candidate_unavailable_cell(candidate, task_count).or_else(|| {
            index_unavailable_cell_for(manifest, candidate, repository, task_count)
        });
        let unsupported = unavailable
            .is_none()
            .then(|| {
                let capability = capability(candidate, "natural_language");
                unmeasured_cell(capability, task_count)
            })
            .flatten();
        if let Some(cell) = unavailable.or(unsupported) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }

        let mut candidate_hits = 0u64;
        let mut candidate_errors = 0u64;
        for task in tasks {
            // A query error is a miss for that task and is counted separately.
            // It never makes the task unavailable.
            let errored = cassette
                .get(&candidate.id, "natural_language", &task.id, "default")
                .is_some_and(|record| record.error.is_some());
            attempted += 1;
            if errored {
                candidate_errors += 1;
                continue;
            }
            let results = recorded_locations(
                cassette,
                &candidate.id,
                repository,
                "natural_language",
                &task.id,
                "default",
            )?;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
            if let Some(rank) = first_rank(&results, &task.expected) {
                if rank < 10 {
                    reciprocal_sum += 1.0 / (rank as f64 + 1.0);
                }
            }
        }
        query_errors += candidate_errors;
        query_errors_per_candidate.insert(candidate.id.clone(), candidate_errors);
        let cell = Score::measured(candidate_hits, task_count, task_count, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(RepositoryScores {
        per_candidate,
        cells,
        attempted,
        reciprocal_sum,
        query_errors,
        query_errors_per_candidate,
    })
}

/// A regex text search is not natural-language retrieval. When a candidate
/// exposes it, the same questions are measured in a separately labelled arm
/// and kept out of the recall value, coverage, and MRR.
const REGEX_PROXY_ARM: &str = "regex_pattern_search_proxy";

fn proxy_arms(
    manifest: &Manifest,
    gold: &Gold,
    cassette: &ResultCassette,
) -> Result<BTreeMap<String, Arm>> {
    let mut per_candidate_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut cells = Vec::new();

    for repository in &manifest.repositories {
        let tasks: Vec<&GoldTask> = gold
            .natural_language
            .iter()
            .filter(|task| task.repository == repository.id)
            .collect();
        if tasks.is_empty() {
            continue;
        }
        let task_count = tasks.len() as u64;
        for candidate in &manifest.candidates {
            if candidate.unavailable.is_some()
                || index_unavailable_cell_for(manifest, candidate, &repository.id, task_count)
                    .is_some()
                || capability(candidate, REGEX_PROXY_ARM).map(|capability| capability.status)
                    != Some(CapabilityStatus::Measured)
            {
                continue;
            }
            let mut candidate_hits = 0u64;
            for task in &tasks {
                let results = recorded_locations(
                    cassette,
                    &candidate.id,
                    &repository.id,
                    "natural_language",
                    &task.id,
                    REGEX_PROXY_ARM,
                )?;
                if hit_at(5, &results, &task.expected) {
                    candidate_hits += 1;
                }
            }
            let cell = Score::measured(candidate_hits, task_count, task_count, 0);
            per_candidate_parts
                .entry(candidate.id.clone())
                .or_default()
                .push(cell.clone());
            cells.push(cell);
        }
    }

    let mut arms = BTreeMap::new();
    if !cells.is_empty() {
        arms.insert(
            REGEX_PROXY_ARM.to_string(),
            Arm {
                score: combine_scores(&cells),
                per_candidate: per_candidate_parts
                    .into_iter()
                    .map(|(candidate, cells)| (candidate, combine_scores(&cells)))
                    .collect(),
            },
        );
    }
    Ok(arms)
}
