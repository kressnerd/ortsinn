//! Metric scorers. All values use a micro-average over measured tasks;
//! unavailable tasks are excluded from the value denominator and reported as
//! coverage.

pub mod common;
pub mod concept;
pub mod explanation;
pub mod gaps;
pub mod natural_language;
pub mod typo;

use anyhow::Result;

use crate::gold::Gold;
use crate::manifest::Manifest;
use crate::record::cassette::ResultCassette;
use crate::report::model::Metrics;

pub fn score(manifest: &Manifest, gold: &Gold, cassette: &ResultCassette) -> Result<Metrics> {
    Ok(Metrics {
        typo: typo::score(manifest, gold, cassette)?,
        natural_language: natural_language::score(manifest, gold, cassette)?,
        explanation: explanation::score(manifest, gold, cassette)?,
        concept: concept::score(
            manifest,
            &gold.concept,
            "concept",
            "concept_recall_at_5",
            cassette,
        )?,
        concept_equivalence: concept::score(
            manifest,
            &gold.concept_equivalence,
            "concept_equivalence",
            "concept_equivalence_recall_at_5",
            cassette,
        )?,
    })
}
