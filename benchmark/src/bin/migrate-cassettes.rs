//! One-shot migration of the committed result cassette's `model_requests`
//! keys to the query-digest form. The result cassette is a record-path input,
//! so its keys cannot be derived by the record path and must be rewritten once
//! against the gold set they were recorded from.
//!
//! The model cassette is *not* rewritten here: the record path regenerates it
//! under the same keys (see the "Fixture migration" section of the design).

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use benchmark::gold::Gold;
use benchmark::manifest::Manifest;
use benchmark::record::cassette::{self, RecordedResult};
use benchmark::record::keys;

#[derive(Debug, Parser)]
#[command(name = "migrate-cassettes")]
struct Args {
    /// Manifest whose gold sets and result cassette are migrated.
    #[arg(long)]
    manifest: PathBuf,
    /// Directory that receives `results.jsonl`.
    #[arg(long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let manifest = Manifest::load(&args.manifest)?;
    let base = Manifest::base_dir(&args.manifest);
    let gold = Gold::load(&manifest, &base)?;
    let source = base.join(&manifest.cassettes.results.path);
    let text = fs::read_to_string(&source)
        .with_context(|| format!("read result cassette {}", source.display()))?;

    let mut migrated = String::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut result: RecordedResult = serde_json::from_str(line).with_context(|| {
            format!(
                "parse result cassette {} line {}",
                source.display(),
                index + 1
            )
        })?;
        let mut rewritten = line.to_string();
        for dependency in &mut result.model_requests {
            let resolved = keys::resolve(
                &gold,
                &result.repository,
                &result.metric,
                &result.candidate,
                &result.task,
                &dependency.key,
            )
            .with_context(|| {
                format!(
                    "migrate result cassette line {} ({}/{}/{})",
                    index + 1,
                    result.candidate,
                    result.metric,
                    result.task
                )
            })?;
            rewritten = rewrite_key(&rewritten, &dependency.key, &resolved.key)?;
            dependency.key = resolved.key;
        }
        migrated.push_str(&rewritten);
        migrated.push('\n');
    }

    fs::create_dir_all(&args.out)
        .with_context(|| format!("create output directory {}", args.out.display()))?;
    let target = args.out.join("results.jsonl");
    fs::write(&target, migrated)
        .with_context(|| format!("write migrated result cassette {}", target.display()))?;
    println!("{}", cassette::fingerprint(&target)?);
    Ok(())
}

/// Replace one quoted JSON key with its migrated form, leaving every other
/// byte of the recorded line untouched.
fn rewrite_key(line: &str, old: &str, new: &str) -> Result<String> {
    let quoted_old = serde_json::to_string(old).context("quote recorded model key")?;
    let quoted_new = serde_json::to_string(new).context("quote migrated model key")?;
    anyhow::ensure!(
        line.contains(&quoted_old),
        "recorded model key {old} does not appear in its result line"
    );
    Ok(line.replace(&quoted_old, &quoted_new))
}
