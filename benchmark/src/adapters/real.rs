//! Shared plumbing for the real candidate adapters: fixed-input context,
//! bounded command execution, and normalized location conversion.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::manifest::{Candidate, Manifest, Repository};

/// The fixed-input context one real session needs. It owns a copy of the
/// manifest slice so the runner can mutate its own manifest while the session
/// is alive.
#[derive(Debug, Clone)]
pub struct RealContext {
    pub candidate: Candidate,
    pub repositories: Vec<Repository>,
    pub base: PathBuf,
    pub embedding_endpoint: Option<String>,
    deadlines: BTreeMap<String, u64>,
}

impl RealContext {
    pub fn new(
        candidate: &Candidate,
        manifest: &Manifest,
        base: &Path,
        embedding_endpoint: Option<&str>,
    ) -> Self {
        let mut deadlines = BTreeMap::new();
        for repository in &manifest.repositories {
            let seconds = if repository.synthetic.is_some() {
                manifest
                    .index_deadlines
                    .get("synthetic_seconds")
                    .copied()
            } else {
                candidate.index_deadline(manifest)
            };
            if let Some(seconds) = seconds {
                deadlines.insert(repository.id.clone(), seconds);
            }
        }
        Self {
            candidate: candidate.clone(),
            repositories: manifest.repositories.clone(),
            base: base.to_path_buf(),
            embedding_endpoint: embedding_endpoint.map(str::to_string),
            deadlines,
        }
    }

    pub fn install(&self) -> PathBuf {
        self.base.join(&self.candidate.install)
    }

    pub fn repository(&self, id: &str) -> Result<&Repository> {
        self.repositories
            .iter()
            .find(|repository| repository.id == id)
            .with_context(|| format!("manifest does not fix repository {id}"))
    }

    pub fn checkout(&self, id: &str) -> Result<PathBuf> {
        Ok(self.base.join(&self.repository(id)?.checkout))
    }

    pub fn deadline(&self, id: &str) -> Option<u64> {
        self.deadlines.get(id).copied()
    }

    pub fn relative_path(&self, repository: &str, path: &str) -> Result<String> {
        let checkout = self.checkout(repository)?;
        let path = Path::new(path);
        let path = path.strip_prefix(&checkout).unwrap_or(path);
        Ok(path.to_string_lossy().replace('\\', "/"))
    }
}

/// One bounded command execution.
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub code: Option<i32>,
    pub timed_out: bool,
    pub seconds: f64,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutcome {
    pub fn succeeded(&self) -> bool {
        !self.timed_out && self.code == Some(0)
    }

    pub fn detail(&self) -> String {
        let text = if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };
        text.lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

/// Run a command, killing it at the deadline. stdout and stderr are drained on
/// separate threads so a verbose candidate cannot fill a pipe and deadlock.
pub fn run_command(
    program: &Path,
    args: &[String],
    cwd: &Path,
    timeout: Option<Duration>,
) -> Result<CommandOutcome> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("start {}", program.display()))?;
    let stdout = child.stdout.take().context("child stdout was not piped")?;
    let stderr = child.stderr.take().context("child stderr was not piped")?;
    let stdout_reader = std::thread::spawn(move || read_all(stdout));
    let stderr_reader = std::thread::spawn(move || read_all(stderr));

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().context("wait for child")? {
            break Some(status);
        }
        if let Some(timeout) = timeout {
            if started.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None;
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let seconds = started.elapsed().as_secs_f64();
    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();
    Ok(CommandOutcome {
        code: status.and_then(|status| status.code()),
        timed_out,
        seconds,
        stdout,
        stderr,
    })
}

fn read_all(mut stream: impl Read) -> String {
    let mut text = String::new();
    let _ = stream.read_to_string(&mut text);
    text
}

/// The SCIP language selector a candidate indexer expects.
pub fn language_selector(language: &str) -> Option<&'static str> {
    match language {
        "typescript" | "javascript" => Some("ts"),
        "python" => Some("py"),
        "rust" => Some("rs"),
        "go" => Some("go"),
        _ => None,
    }
}

/// A measured index that exceeded its deadline is a timeout, not a zero.
pub fn index_timeout(deadline: Option<u64>) -> Duration {
    Duration::from_secs(deadline.unwrap_or(14_400))
}

pub fn require_deadline(deadline: Option<u64>) -> Result<Duration> {
    let Some(deadline) = deadline else {
        bail!("manifest declares no index deadline for this repository");
    };
    Ok(Duration::from_secs(deadline))
}
