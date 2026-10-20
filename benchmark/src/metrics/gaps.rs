//! Measured gaps: to-best for every metric and to-target where a target is
//! stated. A gap without a measured value is unavailable, never zero.

use std::collections::BTreeMap;

use crate::manifest::{CapabilityStatus, Manifest, Target, TargetStatus};
use crate::report::model::{CapabilityGap, CellStatus, Gap, Gaps, Metrics, Score};

pub fn compute(manifest: &Manifest, metrics: &Metrics) -> Gaps {
    let mut to_target = BTreeMap::new();
    let mut to_best = BTreeMap::new();
    let mut capability_gaps = Vec::new();

    for candidate in &manifest.candidates {
        if let Some(unavailable) = &candidate.unavailable {
            capability_gaps.push(CapabilityGap {
                candidate: candidate.id.clone(),
                capability: "candidate".to_string(),
                reason: unavailable.reason.clone(),
            });
        }
        for (capability, status) in &candidate.capabilities {
            if status.status == CapabilityStatus::Unavailable {
                capability_gaps.push(CapabilityGap {
                    candidate: candidate.id.clone(),
                    capability: capability.clone(),
                    reason: status
                        .reason
                        .clone()
                        .unwrap_or_else(|| "capability_not_declared".to_string()),
                });
            }
        }
    }

    let mut target_gap = |key: &str, target: &Target, measured: Option<f64>| {
        to_target.insert(key.to_string(), gap_to_target(target, measured));
    };
    target_gap(
        "typo_recall_at_1",
        &metrics.typo.target,
        metrics.typo.aggregate.value,
    );
    target_gap(
        "natural_language_recall_at_5",
        &metrics.natural_language.target,
        metrics.natural_language.aggregate.value,
    );
    target_gap(
        "explanation_correctness",
        &metrics.explanation.target,
        metrics.explanation.aggregate.value,
    );
    target_gap(
        "concept_recall_at_5",
        &metrics.concept.target,
        metrics.concept.aggregate.value,
    );
    target_gap(
        "concept_equivalence_recall_at_5",
        &metrics.concept_equivalence.target,
        metrics.concept_equivalence.aggregate.value,
    );

    to_best.insert(
        "typo_recall_at_1".to_string(),
        gap_to_best(&metrics.typo.per_candidate),
    );
    to_best.insert(
        "typo_recall_at_5".to_string(),
        gap_to_best(&metrics.typo.recall_at_5_per_candidate),
    );
    to_best.insert(
        "natural_language_recall_at_5".to_string(),
        gap_to_best(&metrics.natural_language.per_candidate),
    );
    to_best.insert(
        "explanation_correctness".to_string(),
        gap_to_best(&metrics.explanation.per_candidate),
    );
    to_best.insert(
        "concept_recall_at_5".to_string(),
        gap_to_best(&metrics.concept.per_candidate),
    );
    to_best.insert(
        "concept_equivalence_recall_at_5".to_string(),
        gap_to_best(&metrics.concept_equivalence.per_candidate),
    );

    Gaps {
        to_target,
        to_best,
        capability_gaps,
    }
}

fn gap_to_target(target: &Target, measured: Option<f64>) -> Gap {
    if target.status == TargetStatus::Unstated || target.value.is_none() {
        return Gap {
            value: None,
            status: CellStatus::NotApplicable,
            reason: Some("target_unstated".to_string()),
            target_status: Some(TargetStatus::Unstated),
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        };
    }
    let Some(target_value) = target.value else {
        unreachable!("target value checked above");
    };
    match measured {
        Some(measured) => Gap {
            value: Some(measured - target_value),
            status: CellStatus::Measured,
            reason: None,
            target_status: Some(target.status),
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        },
        None => Gap {
            value: None,
            status: CellStatus::Unavailable,
            reason: Some("no_measured_candidate".to_string()),
            target_status: Some(target.status),
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        },
    }
}

fn gap_to_best(cells: &BTreeMap<String, Score>) -> Gap {
    let measured: Vec<(&String, f64)> = cells
        .iter()
        .filter_map(|(candidate, cell)| cell.value.map(|value| (candidate, value)))
        .collect();
    let Some((best_candidate, best_value)) = measured
        .iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(candidate, value)| ((*candidate).clone(), *value))
    else {
        return Gap {
            value: None,
            status: CellStatus::Unavailable,
            reason: Some("no_measured_candidate".to_string()),
            target_status: None,
            best_candidate: None,
            best_value: None,
            deltas: BTreeMap::new(),
        };
    };
    let mut deltas = BTreeMap::new();
    for (candidate, value) in measured {
        deltas.insert(candidate.clone(), value - best_value);
    }
    Gap {
        value: Some(0.0),
        status: CellStatus::Measured,
        reason: None,
        target_status: None,
        best_candidate: Some(best_candidate),
        best_value: Some(best_value),
        deltas,
    }
}
