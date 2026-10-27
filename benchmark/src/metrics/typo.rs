//! Typo recall at ranks one and five.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    candidate_unavailable_cell, capability, combine_scores, hit_at, index_unavailable_cell_for,
    label_setup_deviations, recorded_locations, target, unmeasured_cell,
};
use crate::gold::{Gold, GoldTask};
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{NoMatchRate, Score, TypoMetric};

struct RepositoryScores {
    per_candidate: BTreeMap<String, Score>,
    recall_at_5_per_candidate: BTreeMap<String, Score>,
    no_match_rate_per_candidate: BTreeMap<String, NoMatchRate>,
    cells: Vec<Score>,
    recall_at_5_cells: Vec<Score>,
    attempted: u64,
    no_match: u64,
}

pub fn score(manifest: &Manifest, gold: &Gold, cassette: &ResultCassette) -> Result<TypoMetric> {
    let mut per_repository = BTreeMap::new();
    let mut per_candidate_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut recall_at_5_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut no_match_parts: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut repository_cells = Vec::new();
    let mut repository_recall_at_5 = Vec::new();
    let mut no_match = 0u64;
    let mut attempted = 0u64;

    for repository in &manifest.repositories {
        let tasks: Vec<&GoldTask> = gold
            .typo
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
        repository_cells.extend(outcome.cells.iter().cloned());
        repository_recall_at_5.extend(outcome.recall_at_5_cells.iter().cloned());
        for (candidate, cell) in outcome.per_candidate {
            per_candidate_parts
                .entry(candidate)
                .or_default()
                .push(cell);
        }
        for (candidate, cell) in outcome.recall_at_5_per_candidate {
            recall_at_5_parts
                .entry(candidate)
                .or_default()
                .push(cell);
        }
        for (candidate, rate) in outcome.no_match_rate_per_candidate {
            let entry = no_match_parts.entry(candidate).or_insert((0, 0));
            entry.0 += rate.numerator;
            entry.1 += rate.denominator;
        }
        no_match += outcome.no_match;
        attempted += outcome.attempted;
    }

    Ok(TypoMetric {
        target: target(manifest, "typo_recall_at_1"),
        aggregate: combine_scores(&repository_cells),
        recall_at_5: combine_scores(&repository_recall_at_5),
        per_candidate: combine_parts(per_candidate_parts),
        recall_at_5_per_candidate: combine_parts(recall_at_5_parts),
        no_match_rate: NoMatchRate::new(no_match, attempted),
        no_match_rate_per_candidate: no_match_parts
            .into_iter()
            .map(|(candidate, (numerator, denominator))| {
                (candidate, NoMatchRate::new(numerator, denominator))
            })
            .collect(),
        per_repository,
    })
}

fn combine_parts(parts: BTreeMap<String, Vec<Score>>) -> BTreeMap<String, Score> {
    parts
        .into_iter()
        .map(|(candidate, cells)| (candidate, combine_scores(&cells)))
        .collect()
}

fn score_repository(
    manifest: &Manifest,
    tasks: &[&GoldTask],
    repository: &str,
    cassette: &ResultCassette,
) -> Result<RepositoryScores> {
    let task_count = tasks.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut recall_at_5_per_candidate = BTreeMap::new();
    let mut no_match_rate_per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut recall_at_5_cells = Vec::new();
    let mut attempted = 0u64;
    let mut no_match = 0u64;

    for candidate in &manifest.candidates {
        let unavailable = candidate_unavailable_cell(candidate, task_count).or_else(|| {
            index_unavailable_cell_for(manifest, candidate, repository, task_count)
        });
        let unsupported = unavailable
            .is_none()
            .then(|| {
                let capability = capability(candidate, "typo");
                unmeasured_cell(capability, task_count)
            })
            .flatten();

        let (cell, recall_at_5) = if let Some(cell) = unavailable.or(unsupported) {
            (cell.clone(), cell)
        } else {
            let mut candidate_at_1 = 0u64;
            let mut candidate_at_5 = 0u64;
            let mut candidate_no_match = 0u64;
            for task in tasks {
                let results = recorded_locations(
                    cassette,
                    &candidate.id,
                    repository,
                    "typo",
                    &task.id,
                    "default",
                )?;
                attempted += 1;
                if results.is_empty() {
                    candidate_no_match += 1;
                }
                if hit_at(1, &results, &task.expected) {
                    candidate_at_1 += 1;
                }
                if hit_at(5, &results, &task.expected) {
                    candidate_at_5 += 1;
                }
            }
            no_match += candidate_no_match;
            no_match_rate_per_candidate.insert(
                candidate.id.clone(),
                NoMatchRate::new(candidate_no_match, task_count),
            );
            (
                Score::measured(candidate_at_1, task_count, task_count, 0),
                Score::measured(candidate_at_5, task_count, task_count, 0),
            )
        };
        let cell = label_setup_deviations(cell, candidate, &["typo"], repository);
        let recall_at_5 =
            label_setup_deviations(recall_at_5, candidate, &["typo"], repository);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        recall_at_5_per_candidate.insert(candidate.id.clone(), recall_at_5.clone());
        cells.push(cell);
        recall_at_5_cells.push(recall_at_5);
    }

    Ok(RepositoryScores {
        per_candidate,
        recall_at_5_per_candidate,
        no_match_rate_per_candidate,
        cells,
        recall_at_5_cells,
        attempted,
        no_match,
    })
}
