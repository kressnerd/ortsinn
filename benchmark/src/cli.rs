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
        out: Option<PathBuf>,
        /// Terminal width used for the summary. Defaults to `COLUMNS` or 80.
        #[arg(long)]
        width: Option<usize>,
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
            width,
        } => {
            let out = out.unwrap_or_else(|| PathBuf::from("reports/benchmark"));
            let width = width.unwrap_or_else(crate::report::render::terminal_width);
            match RunMode::from(mode) {
                RunMode::Replay => {
                    let report = runner::run_replay(&manifest, &out)?;
                    print!("{}", crate::report::render::terminal_summary(&report, width));
                    println!("Report: {}", out.join("report.json").display());
                    Ok(())
                }
                RunMode::Record => bail!("record mode is not available in this scenario"),
            }
        }
    }
}
