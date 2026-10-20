//! Gold-set loading. The fixture gold files are committed and fixed.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::adapters::NormalizedLocation;
use crate::manifest::{GoldSet, Manifest};

#[derive(Debug, Clone, Deserialize)]
pub struct GoldTask {
    pub id: String,
    #[serde(default)]
    pub query: String,
    pub expected: Vec<NormalizedLocation>,
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
            let path = base.join(&set.path);
            match set.metric.as_str() {
                "typo" => gold.typo = read_tasks(&path)?,
                "natural_language" => gold.natural_language = read_tasks(&path)?,
                "explanation" => gold.explanation = read_explanation(&path)?,
                "concept" => {
                    let entries: Vec<ConceptEntry> = read_jsonl(&path)?;
                    for entry in entries {
                        let task = GoldTask {
                            id: entry.id,
                            query: entry.query,
                            expected: entry.expected,
                        };
                        if entry.equivalence {
                            gold.concept_equivalence.push(task);
                        } else {
                            gold.concept.push(task);
                        }
                    }
                }
                other => anyhow::bail!("unsupported gold metric {other}"),
            }
        }
        anyhow::ensure!(
            gold.typo.len() as u64 == manifest.gold_set("typo").map(|set| set.tasks).unwrap_or(0),
            "typo gold task count does not match the manifest"
        );
        anyhow::ensure!(
            gold.natural_language.len() as u64
                == manifest
                    .gold_set("natural_language")
                    .map(|set| set.tasks)
                    .unwrap_or(0),
            "natural-language gold task count does not match the manifest"
        );
        anyhow::ensure!(
            gold.explanation.len() as u64
                == manifest
                    .gold_set("explanation")
                    .map(|set| set.tasks)
                    .unwrap_or(0),
            "explanation gold task count does not match the manifest"
        );
        anyhow::ensure!(
            gold.concept.len() as u64
                == manifest
                    .gold_set("concept")
                    .map(|set| set.tasks)
                    .unwrap_or(0),
            "concept gold task count does not match the manifest"
        );
        Ok(gold)
    }
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
