//! Concept recall and concept-equivalence recall, with default and
//! no-expansion arms.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    candidate_unavailable_cell, capability, combine_scores, hit_at, index_unavailable_cell_for,
    label_setup_deviations, no_expansion_arm_cell, recorded_locations, target, unmeasured_cell,
};
use crate::gold::GoldTask;
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{Arm, Arms, ConceptMetric, Score};

struct ArmCells {
    arm: Arm,
    cells: Vec<Score>,
}

pub fn score(
    manifest: &Manifest,
    tasks: &[GoldTask],
    metric_name: &str,
    target_key: &str,
    cassette: &ResultCassette,
) -> Result<ConceptMetric> {
    let mut per_repository = BTreeMap::new();
    let mut default_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut no_expansion_parts: BTreeMap<String, Vec<Score>> = BTreeMap::new();
    let mut default_cells = Vec::new();
    let mut no_expansion_cells = Vec::new();

    for repository in &manifest.repositories {
        let repository_tasks: Vec<&GoldTask> = tasks
            .iter()
            .filter(|task| task.repository == repository.id)
            .collect();
        if repository_tasks.is_empty() {
            per_repository.insert(
                repository.id.clone(),
                Score::not_applicable("no_gold_tasks"),
            );
            continue;
        }
        let task_count = repository_tasks.len() as u64;
        let default = score_arm(
            manifest,
            &repository_tasks,
            &repository.id,
            metric_name,
            "default",
            task_count,
            cassette,
        )?;
        let no_expansion = score_no_expansion(
            manifest,
            &repository_tasks,
            &repository.id,
            metric_name,
            task_count,
            cassette,
        )?;
        per_repository.insert(
            repository.id.clone(),
            combine_scores(&default.cells),
        );
        default_cells.extend(default.cells);
        no_expansion_cells.extend(no_expansion.cells);
        for (candidate, cell) in default.arm.per_candidate {
            default_parts.entry(candidate).or_default().push(cell);
        }
        for (candidate, cell) in no_expansion.arm.per_candidate {
            no_expansion_parts
                .entry(candidate)
                .or_default()
                .push(cell);
        }
    }

    let default = Arm {
        score: combine_scores(&default_cells),
        per_candidate: combine_parts(default_parts),
    };
    let no_expansion = Arm {
        score: combine_scores(&no_expansion_cells),
        per_candidate: combine_parts(no_expansion_parts),
    };
    Ok(ConceptMetric {
        target: target(manifest, target_key),
        aggregate: default.score.clone(),
        per_candidate: default.per_candidate.clone(),
        arms: Arms {
            default,
            no_expansion,
        },
        per_repository,
    })
}

fn combine_parts(parts: BTreeMap<String, Vec<Score>>) -> BTreeMap<String, Score> {
    parts
        .into_iter()
        .map(|(candidate, cells)| (candidate, combine_scores(&cells)))
        .collect()
}

fn score_arm(
    manifest: &Manifest,
    tasks: &[&GoldTask],
    repository: &str,
    metric_name: &str,
    arm: &str,
    task_count: u64,
    cassette: &ResultCassette,
) -> Result<ArmCells> {
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();

    for candidate in &manifest.candidates {
        let unavailable = candidate_unavailable_cell(candidate, task_count).or_else(|| {
            index_unavailable_cell_for(manifest, candidate, repository, task_count)
        });
        let unsupported = unavailable
            .is_none()
            .then(|| {
                let capability = capability(candidate, metric_name);
                unmeasured_cell(capability, task_count)
            })
            .flatten();
        if let Some(cell) = unavailable.or(unsupported) {
            let cell = label_setup_deviations(cell, candidate, &[metric_name], repository);
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }

        let mut candidate_hits = 0u64;
        for task in tasks {
            let results = recorded_locations(
                cassette,
                &candidate.id,
                repository,
                metric_name,
                &task.id,
                arm,
            )?;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
        }
        let cell = Score::measured(candidate_hits, task_count, task_count, 0);
        let cell = label_setup_deviations(cell, candidate, &[metric_name], repository);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(ArmCells {
        arm: Arm {
            score: combine_scores(&cells),
            per_candidate,
        },
        cells,
    })
}

fn score_no_expansion(
    manifest: &Manifest,
    tasks: &[&GoldTask],
    repository: &str,
    metric_name: &str,
    task_count: u64,
    cassette: &ResultCassette,
) -> Result<ArmCells> {
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();

    for candidate in &manifest.candidates {
        if let Some(cell) = no_expansion_arm_cell(manifest, candidate, repository, task_count) {
            let cell = label_setup_deviations(
                cell,
                candidate,
                &[metric_name, "expansion"],
                repository,
            );
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let capability = capability(candidate, metric_name);
        if let Some(cell) = unmeasured_cell(capability, task_count) {
            let cell = label_setup_deviations(
                cell,
                candidate,
                &[metric_name, "expansion"],
                repository,
            );
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_hits = 0u64;
        for task in tasks {
            let results = recorded_locations(
                cassette,
                &candidate.id,
                repository,
                metric_name,
                &task.id,
                "no_expansion",
            )?;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
        }
        let cell = Score::measured(candidate_hits, task_count, task_count, 0);
        let cell = label_setup_deviations(
            cell,
            candidate,
            &[metric_name, "expansion"],
            repository,
        );
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(ArmCells {
        arm: Arm {
            score: combine_scores(&cells),
            per_candidate,
        },
        cells,
    })
}
