//! Materialize the fixed synthetic monorepo and print its generated identity.
//!
//! `fetch-repos.sh` calls this once; the manifest records the printed content
//! fingerprint and the setup validation compares the checkout.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "generate-synthetic")]
struct Args {
    /// Directory to stream the generated repository into.
    #[arg(long)]
    root: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.root)?;
    let repo = benchmark::r#gen::synthetic_repo::generate_to(&args.root)?;
    println!(
        "{}",
        serde_json::json!({
            "seed": repo.seed,
            "generator_version": repo.generator_version,
            "recorded_fingerprint": repo.recorded_fingerprint,
            "content_fingerprint": repo.content_digest,
            "code_lines": repo.code_lines,
            "lines_by_language": repo.lines_by_language,
        })
    );
    Ok(())
}
