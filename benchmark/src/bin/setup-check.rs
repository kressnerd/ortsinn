//! Fixed-input validation used by `setup.sh`, `fetch-repos.sh`, and the specs.
//!
//! Exits 2 and names the mismatched input when a pin or checkout drifts. The
//! optional `--print-json` mode emits the manifest pins for the shell scripts
//! so they never parse or re-resolve a pin themselves.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use benchmark::manifest::Manifest;
use benchmark::setup;

#[derive(Debug, Parser)]
#[command(name = "setup-check")]
struct Args {
    /// Fixed-input manifest.
    #[arg(long)]
    manifest: PathBuf,
    /// Committed setup lock. Required unless `--print-json`.
    #[arg(long)]
    lock: Option<PathBuf>,
    /// Verify the materialized checkouts and candidate installations.
    #[arg(long)]
    fixed_inputs: bool,
    /// Print the manifest pins as JSON for the setup scripts.
    #[arg(long)]
    print_json: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.print_json {
        let manifest = Manifest::load(&args.manifest)?;
        println!("{}", serde_json::to_string(&manifest)?);
        return Ok(());
    }
    let lock = args
        .lock
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--lock is required unless --print-json is used"))?;
    if let Err(error) = setup::validate_pins(&args.manifest, lock) {
        eprintln!("setup validation failed: {error:#}");
        std::process::exit(setup::EXIT_INPUT_ERROR);
    }
    if args.fixed_inputs {
        if let Err(error) = setup::validate_fixed_inputs(&args.manifest) {
            eprintln!("fixed-input validation failed: {error:#}");
            std::process::exit(setup::EXIT_INPUT_ERROR);
        }
    }
    println!("setup validation passed: {}", args.manifest.display());
    Ok(())
}
