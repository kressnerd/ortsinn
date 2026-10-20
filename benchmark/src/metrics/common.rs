//! Shared scoring helpers.

use anyhow::{Result, bail};

use crate::adapters::NormalizedLocation;
use crate::gold::matches;
use crate::manifest::{Candidate, Capability, CapabilityStatus, Manifest, Target, TargetStatus};
use crate::record::cassette::ResultCassette;
use crate::report::model::{CellStatus, Score};

pub fn target(manifest: &Manifest, key: &str) -> Target {
    manifest
        .targets
        .get(key)
        .cloned()
        .unwrap_or_else(|| Target {
            value: None,
            status: TargetStatus::Unstated,
            source: "unstated in confirmed records".to_string(),
        })
}

pub fn capability<'a>(candidate: &'a Candidate, name: &str) -> Option<&'a Capability> {
    candidate.capabilities.get(name)
}

pub fn capability_reason(capability: Option<&Capability>) -> &str {
    capability
        .and_then(|capability| capability.reason.as_deref())
        .unwrap_or("capability_not_declared")
}

/// A candidate that could not be installed or started is unavailable for
/// every metric cell with its reason; it is never scored as zero.
pub fn candidate_unavailable_cell(candidate: &Candidate, tasks: u64) -> Option<Score> {
    candidate
        .unavailable
        .as_ref()
        .map(|unavailable| Score::unavailable(&unavailable.reason, tasks))
}

/// A candidate whose index exceeded its deadline is unavailable for every
/// metric on that repository, with the reason and never as zero.
pub fn index_unavailable_cell(
    manifest: &Manifest,
    candidate: &Candidate,
    tasks: u64,
) -> Option<Score> {
    if candidate.unavailable.is_some() {
        return None;
    }
    candidate
        .index_timed_out(manifest)
        .then(|| Score::unavailable("index_timeout", tasks))
}

/// Index availability is per repository: a deadline breach on one fixed
/// repository leaves the candidate's measured cells on another untouched.
pub fn index_unavailable_cell_for(
    manifest: &Manifest,
    candidate: &Candidate,
    repository: &str,
    tasks: u64,
) -> Option<Score> {
    if candidate.unavailable.is_some() {
        return None;
    }
    let deadline = candidate.index_deadline(manifest)?;
    candidate
        .setup
        .indexes
        .iter()
        .any(|index| {
            index.repository == repository && index.seconds > deadline as f64
        })
        .then(|| Score::unavailable("index_timeout", tasks))
}

/// The no-expansion arm is available only when the candidate started, its
/// index stayed inside the deadline, and the manifest declares a measured
/// expansion switch. Precedence is deliberate: an unavailable candidate or a
/// timed-out index makes the arm unavailable, never not-applicable, and a
/// missing switch is `no_expansion_switch`, never zero.
pub fn no_expansion_arm_cell(
    manifest: &Manifest,
    candidate: &Candidate,
    repository: &str,
    tasks: u64,
) -> Option<Score> {
    if let Some(cell) = candidate_unavailable_cell(candidate, tasks) {
        return Some(cell);
    }
    if let Some(cell) = index_unavailable_cell_for(manifest, candidate, repository, tasks) {
        return Some(cell);
    }

    let expansion = capability(candidate, "expansion");
    if expansion.map(|capability| capability.status) == Some(CapabilityStatus::Measured) {
        return None;
    }
    Some(Score::not_applicable(
        expansion
            .and_then(|capability| capability.reason.as_deref())
            .unwrap_or("no_expansion_switch"),
    ))
}

/// Build a per-candidate cell for a capability that is not measured.
pub fn unmeasured_cell(capability: Option<&Capability>, tasks: u64) -> Option<Score> {
    let Some(capability) = capability else {
        return Some(Score::not_applicable("capability_not_declared"));
    };
    match capability.status {
        CapabilityStatus::Unavailable => Some(Score::unavailable(
            capability_reason(Some(capability)),
            tasks,
        )),
        CapabilityStatus::NotApplicable => {
            Some(Score::not_applicable(capability_reason(Some(capability))))
        }
        CapabilityStatus::Measured => None,
    }
}

/// Combine measured and unmeasured candidate cells into a metric aggregate.
/// Not-applicable cells contribute to neither side of the coverage fraction.
pub fn aggregate(cells: &[Score], numerator: u64, denominator: u64) -> Score {
    let attempted: u64 = cells.iter().map(|cell| cell.attempted).sum();
    let unavailable: u64 = cells.iter().map(|cell| cell.unavailable).sum();
    if attempted == 0 {
        return Score::unavailable("no_measured_candidate", unavailable);
    }
    Score::measured(numerator, denominator, attempted, unavailable)
}

pub fn is_measured(cell: &Score) -> bool {
    cell.status == CellStatus::Measured
}

/// Combine the same cell measured on several repositories. Not-applicable
/// cells contribute nothing; an unavailable cell keeps its reason when no
/// repository measured the candidate. A single-repository run combines one
/// cell and returns it unchanged.
pub fn combine_scores(cells: &[Score]) -> Score {
    let attempted: u64 = cells.iter().map(|cell| cell.attempted).sum();
    let unavailable: u64 = cells.iter().map(|cell| cell.unavailable).sum();
    let numerator: u64 = cells.iter().map(|cell| cell.numerator).sum();
    let denominator: u64 = cells.iter().map(|cell| cell.denominator).sum();
    if attempted > 0 && denominator > 0 {
        return Score::measured(numerator, denominator, attempted, unavailable);
    }
    if let Some(cell) = cells
        .iter()
        .find(|cell| cell.status == CellStatus::Unavailable)
    {
        return Score::unavailable(
            cell.reason.as_deref().unwrap_or("unavailable"),
            unavailable.max(cell.unavailable),
        );
    }
    if let Some(cell) = cells
        .iter()
        .find(|cell| cell.status == CellStatus::NotApplicable)
    {
        return Score::not_applicable(cell.reason.as_deref().unwrap_or("no_gold_tasks"));
    }
    Score::unavailable("no_measured_candidate", unavailable)
}

/// Load one recorded result and return its deduplicated locations.
pub fn recorded_locations(
    cassette: &ResultCassette,
    candidate: &str,
    repository: &str,
    metric: &str,
    task: &str,
    arm: &str,
) -> Result<Vec<NormalizedLocation>> {
    let Some(record) = cassette.get(candidate, metric, task, arm) else {
        bail!(
            "missing result cassette entry for candidate {candidate}, repository {repository}, metric {metric}, task {task}, arm {arm}"
        );
    };
    if record.error.is_some() {
        return Ok(Vec::new());
    }
    Ok(crate::adapters::dedup_locations(&record.results))
}

pub fn hit_at(
    rank: usize,
    results: &[NormalizedLocation],
    expected: &[NormalizedLocation],
) -> bool {
    results
        .iter()
        .take(rank)
        .any(|candidate| expected.iter().any(|target| matches(candidate, target)))
}

pub fn first_rank(
    results: &[NormalizedLocation],
    expected: &[NormalizedLocation],
) -> Option<usize> {
    results
        .iter()
        .position(|candidate| expected.iter().any(|target| matches(candidate, target)))
}
