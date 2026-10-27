//! Gold-set loading. The fixture gold files are committed and fixed.

pub mod propose;

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::adapters::NormalizedLocation;
use crate::manifest::{GoldSet, Manifest};

/// A manifest-referenced gold set the maintainer has not reviewed. The run
/// stops before any task is scored or any report is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreviewedGoldSet {
    pub repository: String,
    pub metric: String,
}

impl std::fmt::Display for UnreviewedGoldSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unreviewed gold set {} {}: the manifest references it without a reviewed_by entry",
            self.repository, self.metric
        )
    }
}

impl std::error::Error for UnreviewedGoldSet {}

/// Only a reviewed set the manifest references may be scored. A draft under
/// the drafts directory that the manifest does not reference never reaches
/// this loader.
fn ensure_reviewed(set: &GoldSet) -> Result<()> {
    let reviewed = set.reviewed_by.as_deref().unwrap_or_default().trim();
    anyhow::ensure!(
        !reviewed.is_empty(),
        UnreviewedGoldSet {
            repository: set.repository.clone(),
            metric: set.metric.clone(),
        }
    );
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoldTask {
    pub id: String,
    #[serde(default)]
    pub query: String,
    pub expected: Vec<NormalizedLocation>,
    /// Repository this task scores. The loader sets it from the gold set's
    /// manifest entry; the committed file does not repeat it.
    #[serde(default)]
    pub repository: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineRange {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExplanationTask {
    pub id: String,
    #[serde(default)]
    pub query: String,
    pub target: NormalizedLocation,
    pub required_facts: Vec<Vec<String>>,
    pub acceptable_evidence: Vec<LineRange>,
    /// Repository this task scores; set by the loader from the gold set.
    #[serde(default)]
    pub repository: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConceptEntry {
    pub id: String,
    #[serde(default)]
    pub query: String,
    pub expected: Vec<NormalizedLocation>,
    #[serde(default)]
    pub equivalence: bool,
}

#[derive(Debug, Clone)]
pub struct Gold {
    pub typo: Vec<GoldTask>,
    pub natural_language: Vec<GoldTask>,
    pub explanation: Vec<ExplanationTask>,
    pub concept: Vec<GoldTask>,
    pub concept_equivalence: Vec<GoldTask>,
    pub sets: Vec<GoldSet>,
}

impl Gold {
    pub fn load(manifest: &Manifest, base: &Path) -> Result<Self> {
        let mut gold = Self {
            typo: Vec::new(),
            natural_language: Vec::new(),
            explanation: Vec::new(),
            concept: Vec::new(),
            concept_equivalence: Vec::new(),
            sets: manifest.gold.clone(),
        };
        for set in &manifest.gold {
            let repository = manifest.repository(&set.repository).with_context(|| {
                format!(
                    "gold set {} names repository {}, which the manifest does not fix",
                    set.metric, set.repository
                )
            })?;
            ensure_reviewed(set)?;
            let root = base.join(&repository.checkout);
            anyhow::ensure!(
                root.is_dir(),
                "gold validation: checkout {} for repository {} does not exist",
                root.display(),
                repository.id
            );
            let path = base.join(&set.path);
            crate::manifest::ensure_declared_fingerprint(
                &path,
                &set.fingerprint,
                &format!("gold set {} {}", set.repository, set.metric),
            )
            .with_context(|| format!("verify gold file {}", path.display()))?;
            let label = format!("{} {}", repository.id, set.metric);
            match set.metric.as_str() {
                "typo" => {
                    let mut tasks: Vec<GoldTask> = read_tasks(&path)?;
                    validate_task_locations(&root, &tasks, &label)?;
                    for task in &mut tasks {
                        task.repository.clone_from(&set.repository);
                    }
                    gold.typo.extend(tasks);
                }
                "natural_language" => {
                    let mut tasks: Vec<GoldTask> = read_tasks(&path)?;
                    validate_task_locations(&root, &tasks, &label)?;
                    for task in &mut tasks {
                        task.repository.clone_from(&set.repository);
                    }
                    gold.natural_language.extend(tasks);
                }
                "explanation" => {
                    let mut tasks: Vec<ExplanationTask> = read_explanation(&path)?;
                    validate_explanation_locations(&root, &tasks, &label)?;
                    for task in &mut tasks {
                        task.repository.clone_from(&set.repository);
                    }
                    gold.explanation.extend(tasks);
                }
                "concept" | "concept_equivalence" => {
                    let equivalence = set.metric == "concept_equivalence";
                    let entries: Vec<ConceptEntry> = read_jsonl(&path)?;
                    for entry in &entries {
                        if entry.equivalence != equivalence {
                            continue;
                        }
                        let task = GoldTask {
                            id: entry.id.clone(),
                            query: entry.query.clone(),
                            expected: entry.expected.clone(),
                            repository: set.repository.clone(),
                        };
                        validate_task_locations(&root, std::slice::from_ref(&task), &label)?;
                        if equivalence {
                            gold.concept_equivalence.push(task);
                        } else {
                            gold.concept.push(task);
                        }
                    }
                }
                other => anyhow::bail!("unsupported gold metric {other}"),
            }
        }
        ensure_declared_count(manifest, "typo", gold.typo.len())?;
        ensure_declared_count(
            manifest,
            "natural_language",
            gold.natural_language.len(),
        )?;
        ensure_declared_count(manifest, "explanation", gold.explanation.len())?;
        ensure_declared_count(manifest, "concept", gold.concept.len())?;
        ensure_declared_count(
            manifest,
            "concept_equivalence",
            gold.concept_equivalence.len(),
        )?;
        Ok(gold)
    }
}

/// The manifest declares each gold set's reviewed task count. A gold file
/// that lost or gained a task fails the load instead of silently changing the
/// measurement denominator.
fn ensure_declared_count(manifest: &Manifest, metric: &str, actual: usize) -> Result<()> {
    let declared: u64 = manifest
        .gold
        .iter()
        .filter(|set| set.metric == metric)
        .map(|set| set.tasks)
        .sum();
    anyhow::ensure!(
        actual as u64 == declared,
        "{metric} gold task count does not match the manifest: declared {declared}, loaded {actual}"
    );
    Ok(())
}

/// A gold task that cannot be checked against the fixed checkout. The
/// message names the task, path, and failed range; the run stops before any
/// score is computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldValidationError {
    pub detail: String,
}

impl std::fmt::Display for GoldValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for GoldValidationError {}

/// Every expected target must exist at the fixed checkout and its line range
/// must fall inside the file. A task that cites a missing path or an
/// out-of-range line fails the run before any score is computed.
fn validate_task_locations(root: &Path, tasks: &[GoldTask], label: &str) -> Result<()> {
    for task in tasks {
        for location in &task.expected {
            validate_location(root, location, &format!("{label} task {}", task.id))?;
        }
    }
    Ok(())
}

/// The explanation rubric cites a target and an acceptable-evidence range in
/// that target's file; both must be inside the fixed checkout.
fn validate_explanation_locations(
    root: &Path,
    tasks: &[ExplanationTask],
    label: &str,
) -> Result<()> {
    for task in tasks {
        let context = format!("{label} task {}", task.id);
        let path = validate_location(root, &task.target, &context)?;
        let lines = line_count(&path)?;
        for range in &task.acceptable_evidence {
            anyhow::ensure!(
                range.start_line >= 1 && range.end_line >= range.start_line,
                "{context}: acceptable evidence range {}..={} is not ordered",
                range.start_line,
                range.end_line
            );
            anyhow::ensure!(
                u64::from(range.end_line) <= lines,
                "{context}: acceptable evidence range {}..={} exceeds {} lines",
                range.start_line,
                range.end_line,
                lines
            );
        }
    }
    Ok(())
}

fn validate_location(
    root: &Path,
    location: &NormalizedLocation,
    context: &str,
) -> Result<PathBuf> {
    validate_location_inner(root, location, context).map_err(|error| {
        GoldValidationError {
            detail: format!("{error:#}"),
        }
        .into()
    })
}

fn validate_location_inner(
    root: &Path,
    location: &NormalizedLocation,
    context: &str,
) -> Result<PathBuf> {
    anyhow::ensure!(
        location.start_line >= 1,
        "{context}: gold path {} start line must be at least 1",
        location.path
    );
    anyhow::ensure!(
        location.end_line >= location.start_line,
        "{context}: gold path {} line range {}..={} is not ordered",
        location.path,
        location.start_line,
        location.end_line
    );
    let relative = Path::new(&location.path);
    anyhow::ensure!(
        !relative.is_absolute()
            && relative
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
        "{context}: gold path {} must stay inside the checkout",
        location.path
    );
    let canonical_root = root
        .canonicalize()
        .with_context(|| format!("{context}: canonicalize checkout {}", root.display()))?;
    let canonical = root
        .join(relative)
        .canonicalize()
        .with_context(|| format!("{context}: gold path {} does not exist at the fixed checkout", location.path))?;
    anyhow::ensure!(
        canonical.starts_with(&canonical_root),
        "{context}: gold path {} resolves outside the checkout",
        location.path
    );
    let lines = line_count(&canonical)?;
    anyhow::ensure!(
        u64::from(location.end_line) <= lines,
        "{context}: gold path {} line range {}..={} exceeds {} lines",
        location.path,
        location.start_line,
        location.end_line,
        lines
    );
    Ok(canonical)
}

fn line_count(path: &Path) -> Result<u64> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read gold target {}", path.display()))?;
    Ok(text.lines().count() as u64)
}

fn read_tasks(path: &Path) -> Result<Vec<GoldTask>> {
    read_jsonl(path)
}

fn read_explanation(path: &Path) -> Result<Vec<ExplanationTask>> {
    read_jsonl(path)
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read gold file {}", path.display()))?;
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        out.push(
            serde_json::from_str(line).with_context(|| {
                format!("parse gold file {} line {}", path.display(), index + 1)
            })?,
        );
    }
    Ok(out)
}

/// True when the candidate location is the expected location.
pub fn matches(candidate: &NormalizedLocation, expected: &NormalizedLocation) -> bool {
    candidate.path == expected.path
        && candidate.start_line == expected.start_line
        && candidate.end_line == expected.end_line
        && candidate.symbol == expected.symbol
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gold_set(reviewed_by: Option<&str>) -> GoldSet {
        GoldSet {
            repository: "fixture-repo".to_string(),
            metric: "typo".to_string(),
            version: 1,
            tasks: 5,
            fingerprint: "sha256:00".to_string(),
            reviewed_by: reviewed_by.map(str::to_string),
            path: "gold/typo.jsonl".to_string(),
        }
    }

    #[test]
    fn s10_unreviewed_referenced_gold_set_is_typed_and_specifically_named() {
        for missing in [None, Some(""), Some("   ")] {
            let error = ensure_reviewed(&gold_set(missing))
                .expect_err("a manifest-referenced set without a reviewer must fail");
            let typed = error
                .downcast_ref::<UnreviewedGoldSet>()
                .expect("the failure is the typed unreviewed gold set error");
            assert_eq!(typed.repository, "fixture-repo");
            assert_eq!(typed.metric, "typo");
            assert!(error.to_string().contains("unreviewed gold set"));
            assert!(error.to_string().contains("fixture-repo"));
            assert!(error.to_string().contains("typo"));
        }
        assert!(ensure_reviewed(&gold_set(Some("maintainer--ortsinn"))).is_ok());
    }
}
