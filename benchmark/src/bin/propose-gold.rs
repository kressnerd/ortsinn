//! Build, review, or check the real gold sets.
//!
//! A proposal is a function of the reviewed seed file and the fixed checkout.
//! `--mode drafts` writes `gold/drafts/<repository>/<metric>.jsonl`, which the
//! loader never reads. `--mode reviewed` writes the reviewed tree after the
//! maintainer accepts the drafts. `--mode check` reloads the manifest through
//! `Gold::load` and compares every reviewed file with a fresh proposal, so the
//! committed gold cannot drift from its seeds or checkout.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use benchmark::gold::{self, propose, seeds::SeedFile};
use benchmark::manifest::{Manifest, Power};
use benchmark::r#gen::synthetic_repo;

#[derive(Debug, Parser)]
#[command(name = "propose-gold")]
struct Args {
    /// Fixed-input manifest whose repositories and minima the gold follows.
    #[arg(long)]
    manifest: PathBuf,
    /// Gold root that holds seeds, drafts, and the reviewed tree.
    #[arg(long)]
    gold: PathBuf,
    #[arg(long, value_enum)]
    mode: Mode,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Drafts,
    Reviewed,
    Check,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let manifest = Manifest::load(&args.manifest)?;
    let base = Manifest::base_dir(&args.manifest);
    let minimums = manifest
        .gold_minimums
        .as_ref()
        .context("the manifest declares no gold minimums")?;
    let mut summary = Vec::new();
    for repository in &manifest.repositories {
        let power = repository
            .power
            .with_context(|| format!("repository {} has no power label", repository.id))?;
        let counts = match power {
            Power::Primary => &minimums.primary,
            Power::Secondary => &minimums.secondary,
        };
        let sets = if let Some(_synthetic) = &repository.synthetic {
            let generated = synthetic_repo::generate();
            synthetic_repo::gold_files(&generated)
        } else {
            let seed_path = args
                .gold
                .join(propose::SEEDS_DIR)
                .join(format!("{}.yaml", repository.id));
            let seed_file = SeedFile::load(&seed_path)?;
            let root = base.join(&repository.checkout);
            let definitions = propose::resolve_seeds(&root, &seed_file)?;
            propose::propose(&seed_file, &definitions, counts, &id_prefix(&repository.id))?
        };
        for metric in propose::METRICS {
            let tasks = sets
                .get(metric)
                .with_context(|| format!("{} has no {metric} proposals", repository.id))?;
            let expected = counts
                .get(metric)
                .copied()
                .with_context(|| format!("the manifest declares no {metric} minimum"))?;
            anyhow::ensure!(
                tasks.len() as u64 == expected,
                "{} {metric}: proposed {} tasks, expected {expected}",
                repository.id,
                tasks.len()
            );
            match args.mode {
                Mode::Drafts => {
                    let path = propose::write_draft(&args.gold, &repository.id, metric, tasks)?;
                    summary.push(entry(&repository.id, metric, tasks.len(), &path, None)?);
                }
                Mode::Reviewed => {
                    let path =
                        propose::write_reviewed(&args.gold, &repository.id, metric, tasks)?;
                    summary.push(entry(&repository.id, metric, tasks.len(), &path, None)?);
                }
                Mode::Check => {
                    let path = propose::reviewed_path(&args.gold, &repository.id, metric);
                    propose::check_reviewed(&args.gold, &repository.id, metric, tasks)?;
                    let reviewer = manifest
                        .gold
                        .iter()
                        .find(|set| set.repository == repository.id && set.metric == metric)
                        .and_then(|set| set.reviewed_by.clone());
                    summary.push(entry(
                        &repository.id,
                        metric,
                        tasks.len(),
                        &path,
                        reviewer.as_deref(),
                    )?);
                }
            }
        }
    }
    if matches!(args.mode, Mode::Check) {
        let loaded = gold::Gold::load(&manifest, &base)?;
        summary.push(serde_json::json!({
            "loaded": {
                "typo": loaded.typo.len(),
                "natural_language": loaded.natural_language.len(),
                "explanation": loaded.explanation.len(),
                "concept": loaded.concept.len(),
                "concept_equivalence": loaded.concept_equivalence.len(),
            },
        }));
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn entry(
    repository: &str,
    metric: &str,
    tasks: usize,
    path: &Path,
    reviewer: Option<&str>,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "repository": repository,
        "metric": metric,
        "tasks": tasks,
        "fingerprint": propose::fingerprint(path)?,
        "reviewed_by": reviewer,
    }))
}

/// Stable id prefix per repository. Task ids are cassette keys, so they must
/// stay unique across every repository of the manifest.
fn id_prefix(repository: &str) -> String {
    match repository {
        "typescript-reference" => "ts".to_string(),
        "python-reference" => "py".to_string(),
        "rust-reference" => "rs".to_string(),
        "synthetic-monorepo" => "syn".to_string(),
        other => other
            .chars()
            .filter(char::is_ascii_alphabetic)
            .take(2)
            .collect::<String>()
            .to_lowercase(),
    }
}
