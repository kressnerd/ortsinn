use std::path::{Path, PathBuf};

use benchmark::manifest::Manifest;
use benchmark::report::model::Report;
use cucumber::{World, cli::Opts};

#[path = "steps/mod.rs"]
mod steps;

#[derive(Debug, World)]
pub(crate) struct BenchmarkWorld {
    /// Path to the committed fixture manifest.
    pub(crate) manifest_path: PathBuf,
    /// Report output directory for the current scenario.
    pub(crate) out_dir: PathBuf,
    /// Loaded fixture manifest.
    pub(crate) manifest: Option<Manifest>,
    /// Report produced by the last replay run.
    pub(crate) report: Option<Report>,
    /// Report from the completed run compared by the replay-rerun scenario.
    pub(crate) first_report: Option<Report>,
    /// Failure from a run that must not publish a report.
    pub(crate) run_failure: Option<benchmark::runner::RunFailure>,
    /// Recorded model key and candidate omitted by the missing-response
    /// fixture, so the Then-step can name both.
    pub(crate) missing_model: Option<(String, String)>,
    /// Expected and actual repository commits of the mismatch fixture.
    pub(crate) mismatch: Option<(String, String)>,
    /// First synthetic generation (the recorded run).
    pub(crate) synthetic_first: Option<benchmark::r#gen::synthetic_repo::SyntheticRepo>,
    /// Second synthetic generation (the regeneration).
    pub(crate) synthetic_second: Option<benchmark::r#gen::synthetic_repo::SyntheticRepo>,
    /// Report output directory for an E2E CLI scenario.
    pub(crate) e2e_out: PathBuf,
    /// Captured terminal output of the E2E replay run.
    pub(crate) e2e_terminal: String,
    /// Exit code of the last E2E CLI invocation.
    pub(crate) e2e_exit: Option<i32>,
    /// Report path printed by the CLI.
    pub(crate) e2e_report_path: Option<PathBuf>,
    /// Text accumulated for the committed transcript.
    pub(crate) e2e_transcript: String,
    /// Loopback fake OpenAI-compatible endpoint used by the record scenario.
    pub(crate) e2e_endpoint: Option<benchmark::record::proxy::FakeOpenAiEndpoint>,
    /// Report parsed after the record run, before the replay overwrites it.
    pub(crate) e2e_record_report: Option<serde_json::Value>,
    /// Endpoint request count observed before the replay.
    pub(crate) e2e_record_requests: usize,
    /// Candidate answers returned by the adapter test double.
    pub(crate) adapter_answers: Option<Vec<benchmark::adapters::session::CandidateAnswer>>,
    /// Normalized results produced through the candidate session boundary.
    pub(crate) adapter_results: Option<Vec<benchmark::adapters::NormalizedLocation>>,
    /// Candidate the adapter scenario drives, so the Then-steps name it.
    pub(crate) focus_candidate: Option<String>,
    /// Setup deviation the adapter scenario declares.
    pub(crate) setup_deviation: Option<benchmark::manifest::SetupDeviation>,
    /// Loopback endpoint used by the record-mode adapter scenario.
    pub(crate) adapter_endpoint: Option<benchmark::record::proxy::FakeOpenAiEndpoint>,
}

impl Default for BenchmarkWorld {
    fn default() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out_dir = root
            .parent()
            .expect("workspace root")
            .join("target")
            .join("benchmark-fixture-report");
        Self {
            manifest_path: root.join("tests/fixtures/e2e/manifest.yaml"),
            out_dir,
            manifest: None,
            report: None,
            first_report: None,
            run_failure: None,
            missing_model: None,
            mismatch: None,
            synthetic_first: None,
            synthetic_second: None,
            e2e_out: PathBuf::new(),
            e2e_terminal: String::new(),
            e2e_exit: None,
            e2e_report_path: None,
            e2e_transcript: String::new(),
            e2e_endpoint: None,
            e2e_record_report: None,
            e2e_record_requests: 0,
            adapter_answers: None,
            adapter_results: None,
            focus_candidate: None,
            setup_deviation: None,
            adapter_endpoint: None,
        }
    }
}

fn feature_paths() -> Vec<PathBuf> {
    let raw = std::env::var("GIVN_FEATURES").unwrap_or_default();
    let mut paths: Vec<PathBuf> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    if paths.is_empty() {
        paths.push(PathBuf::from("givn/specs"));
    }
    paths
}

fn staged_features(paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let root = Path::new("target").join("givn-features");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;
    let mut count = 0usize;
    for path in paths {
        if path.is_dir() {
            for entry in walkdir(path)? {
                if entry.extension().is_some_and(|ext| ext == "feature") {
                    let name = format!("{count}.feature");
                    std::fs::copy(&entry, root.join(name))?;
                    count += 1;
                }
            }
        } else if path.is_file() {
            let name = format!("{count}.feature");
            std::fs::copy(path, root.join(name))?;
            count += 1;
        }
    }
    anyhow::ensure!(count > 0, "GIVN_FEATURES resolved to no feature files");
    Ok(root)
}

fn walkdir(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let targeted = args
        .iter()
        .any(|arg| arg == "--name" || arg == "-n" || arg == "--tags" || arg == "-t");
    if !targeted {
        // The real scenarios stay @wip until their RED task starts; a
        // baseline run must not execute them.
        // SAFETY: called before the runner starts any thread; no other
        // thread reads the environment concurrently.
        unsafe {
            std::env::set_var("CUCUMBER_FILTER_TAGS", "not @wip");
        }
    }

    let paths = feature_paths();
    let staged = staged_features(&paths).expect("stage feature files");
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        BenchmarkWorld::cucumber()
            .with_cli::<cucumber::cli::Empty>(Opts::parsed())
            .fail_on_skipped()
            // `filter_run` applies the CLI `--name`/`--tags` filters before
            // this closure, `fail_on_skipped` records skipped steps, and
            // `filter_run_and_exit` turns any failed/skipped run into a
            // non-zero process exit.
            .filter_run_and_exit(&staged, |_, _, _| true)
            .await;
    });
}
