//! Natural-language recall at five and mean reciprocal rank at ten.

use std::collections::BTreeMap;

use anyhow::Result;

use super::common::{
    aggregate, candidate_unavailable_cell, capability, first_rank, hit_at, index_unavailable_cell,
    recorded_locations, target, unmeasured_cell,
};
use crate::gold::Gold;
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::{MrrScore, NaturalLanguageMetric, Score};

pub fn score(
    manifest: &Manifest,
    gold: &Gold,
    cassette: &ResultCassette,
) -> Result<NaturalLanguageMetric> {
    let tasks = gold.natural_language.len() as u64;
    let mut per_candidate = BTreeMap::new();
    let mut cells = Vec::new();
    let mut hits = 0u64;
    let mut attempted = 0u64;
    let mut reciprocal_sum = 0.0f64;

    for candidate in &manifest.candidates {
        if let Some(cell) = candidate_unavailable_cell(candidate, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        if let Some(cell) = index_unavailable_cell(manifest, candidate, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let capability = capability(candidate, "natural_language");
        if let Some(cell) = unmeasured_cell(capability, tasks) {
            per_candidate.insert(candidate.id.clone(), cell.clone());
            cells.push(cell);
            continue;
        }
        let mut candidate_hits = 0u64;
        for task in &gold.natural_language {
            let results = recorded_locations(
                cassette,
                &candidate.id,
                "natural_language",
                &task.id,
                "default",
            )?;
            attempted += 1;
            if hit_at(5, &results, &task.expected) {
                candidate_hits += 1;
            }
            if let Some(rank) = first_rank(&results, &task.expected) {
                if rank < 10 {
                    reciprocal_sum += 1.0 / (rank as f64 + 1.0);
                }
            }
        }
        hits += candidate_hits;
        let cell = Score::measured(candidate_hits, tasks, tasks, 0);
        per_candidate.insert(candidate.id.clone(), cell.clone());
        cells.push(cell);
    }

    Ok(NaturalLanguageMetric {
        target: target(manifest, "natural_language_recall_at_5"),
        aggregate: aggregate(&cells, hits, attempted),
        per_candidate,
        mrr_at_10: MrrScore {
            value: if attempted == 0 {
                0.0
            } else {
                reciprocal_sum / attempted as f64
            },
            attempted,
        },
    })
}
