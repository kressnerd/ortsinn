//! Typo recall at ranks one and five.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{aggregate, capability, hit_at, recorded_locations, target, unmeasured_cell};
use crate::gold::Gold;
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{Score, TypoMetric};

pub fn score(manifest: &Manifest, gold: &Gold, cassette: &ResultCassette) -> Result<TypoMetric> {
    let tasks = gold.typo.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut at_1 = 0u64;
    let mut at_5 = 0u64;
    let mut attempted = 0u64;

    for candidate in &manifest.candidates {
        let capability = capability(candidate, "typo");
        if let Some(cell) = unmeasured_cell(capability, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_at_1 = 0u64;
        let mut candidate_at_5 = 0u64;
        for task in &gold.typo {
            let results = recorded_locations(cassette, &candidate.id, "typo", &task.id, "default")?;
            attempted += 1;
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
        let cell = Score::measured(candidate_at_1, tasks, tasks, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    let aggregate_cell = aggregate(&cells, at_1, attempted);
    let recall_at_5 = aggregate(&cells, at_5, attempted);
    Ok(TypoMetric {
        target: target(manifest, "typo_recall_at_1"),
        aggregate: aggregate_cell,
        recall_at_5,
        per_candidate,
    })
}
