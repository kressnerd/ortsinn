//! Concept recall and concept-equivalence recall, with default and
//! no-expansion arms.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    aggregate, capability, capability_reason, hit_at, recorded_locations, target, unmeasured_cell,
};
use crate::gold::GoldTask;
use crate::manifest::{CapabilityStatus, Manifest};
use crate::record::cassette::ResultCassette;
use crate::report::model::{Arm, Arms, ConceptMetric, Score};

pub fn score(
    manifest: &Manifest,
    tasks: &[GoldTask],
    metric_name: &str,
    target_key: &str,
    cassette: &ResultCassette,
) -> Result<ConceptMetric> {
    let task_count = tasks.len() as u64;
    let default = score_arm(
        manifest,
        tasks,
        metric_name,
        "default",
        task_count,
        cassette,
    )?;
    let no_expansion = score_no_expansion(manifest, tasks, metric_name, task_count, cassette)?;
    Ok(ConceptMetric {
        target: target(manifest, target_key),
        aggregate: default.score.clone(),
        per_candidate: default.per_candidate.clone(),
        arms: Arms {
            default,
            no_expansion,
        },
    })
}

fn score_arm(
    manifest: &Manifest,
    tasks: &[GoldTask],
    metric_name: &str,
    arm: &str,
    task_count: u64,
    cassette: &ResultCassette,
) -> Result<Arm> {
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut hits = 0u64;
    let mut attempted = 0u64;

    for candidate in &manifest.candidates {
        let capability = capability(candidate, metric_name);
        if let Some(cell) = unmeasured_cell(capability, task_count) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_hits = 0u64;
        for task in tasks {
            let results = recorded_locations(cassette, &candidate.id, metric_name, &task.id, arm)?;
            attempted += 1;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
        }
        hits += candidate_hits;
        let cell = Score::measured(candidate_hits, task_count, task_count, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(Arm {
        score: aggregate(&cells, hits, attempted),
        per_candidate,
    })
}

fn score_no_expansion(
    manifest: &Manifest,
    tasks: &[GoldTask],
    metric_name: &str,
    task_count: u64,
    cassette: &ResultCassette,
) -> Result<Arm> {
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut hits = 0u64;
    let mut attempted = 0u64;

    for candidate in &manifest.candidates {
        let expansion = capability(candidate, "expansion");
        if expansion.map(|capability| capability.status) != Some(CapabilityStatus::Measured) {
            let cell = Score::not_applicable(capability_reason(expansion));
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let capability = capability(candidate, metric_name);
        if let Some(cell) = unmeasured_cell(capability, task_count) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_hits = 0u64;
        for task in tasks {
            let results = recorded_locations(
                cassette,
                &candidate.id,
                metric_name,
                &task.id,
                "no_expansion",
            )?;
            attempted += 1;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
        }
        hits += candidate_hits;
        let cell = Score::measured(candidate_hits, task_count, task_count, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(Arm {
        score: aggregate(&cells, hits, attempted),
        per_candidate,
    })
}
