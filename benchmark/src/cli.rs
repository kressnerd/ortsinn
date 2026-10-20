//! `benchmark run` CLI.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use crate::manifest::RunMode;
use crate::runner;

#[derive(Debug, Parser)]
#[command(name = "benchmark", version, about = "Ortsinn benchmark harness")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the fixed benchmark inputs.
    Run {
        /// Fixed-input manifest.
        #[arg(long)]
        manifest: PathBuf,
        /// Recording mode.
        #[arg(long, value_enum, default_value_t = ModeArg::Replay)]
        mode: ModeArg,
        /// Report output directory.
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ModeArg {
    Replay,
    Record,
}

impl From<ModeArg> for RunMode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Replay => RunMode::Replay,
            ModeArg::Record => RunMode::Record,
        }
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run {
            manifest,
            mode,
            out,
        } => match RunMode::from(mode) {
            RunMode::Replay => {
                let report = runner::run_replay(&manifest, &out)?;
                println!(
                    "benchmark {} completed: {} candidates, report at {}",
                    report.run.id,
                    report.candidates.len(),
                    out.join("report.json").display()
                );
                Ok(())
            }
            RunMode::Record => bail!("record mode is not available in this scenario"),
        },
    }
}
