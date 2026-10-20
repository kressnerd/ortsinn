//! Typo recall at ranks one and five.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    aggregate, candidate_unavailable_cell, capability, hit_at, index_unavailable_cell,
    recorded_locations, target, unmeasured_cell,
};
use crate::gold::Gold;
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{NoMatchRate, Score, TypoMetric};

pub fn score(manifest: &Manifest, gold: &Gold, cassette: &ResultCassette) -> Result<TypoMetric> {
    let tasks = gold.typo.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut recall_at_5_per_candidate = BTreeMap::new();
    let mut no_match_rate_per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut at_1 = 0u64;
    let mut at_5 = 0u64;
    let mut attempted = 0u64;
    let mut no_match = 0u64;

    for candidate in &manifest.candidates {
        if let Some(cell) = candidate_unavailable_cell(candidate, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            recall_at_5_per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        if let Some(cell) = index_unavailable_cell(manifest, candidate, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            recall_at_5_per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let capability = capability(candidate, "typo");
        if let Some(cell) = unmeasured_cell(capability, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            recall_at_5_per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_at_1 = 0u64;
        let mut candidate_at_5 = 0u64;
        let mut candidate_no_match = 0u64;
        for task in &gold.typo {
            let results = recorded_locations(cassette, &candidate.id, "typo", &task.id, "default")?;
            attempted += 1;
            if results.is_empty() {
                candidate_no_match += 1;
            }
            let expected = &task.expected;
            if hit_at(1, &results, expected) {
                candidate_at_1 += 1;
            }
            if hit_at(5, &results, expected) {
                candidate_at_5 += 1;
            }
        }
        at_1 += candidate_at_1;
        at_5 += candidate_at_5;
        no_match += candidate_no_match;
        no_match_rate_per_candidate.insert(
            candidate.id.clone(),
            NoMatchRate::new(candidate_no_match, tasks),
        );
        let cell = Score::measured(candidate_at_1, tasks, tasks, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        recall_at_5_per_candidate.insert(
            candidate.id.clone(),
            Score::measured(candidate_at_5, tasks, tasks, 0),
        );
        cells.push(cell);
    }

    let aggregate_cell = aggregate(&cells, at_1, attempted);
    let recall_at_5 = aggregate(&cells, at_5, attempted);
    Ok(TypoMetric {
        target: target(manifest, "typo_recall_at_1"),
        aggregate: aggregate_cell,
        recall_at_5,
        per_candidate,
        recall_at_5_per_candidate,
        no_match_rate: NoMatchRate::new(no_match, attempted),
        no_match_rate_per_candidate,
    })
}
