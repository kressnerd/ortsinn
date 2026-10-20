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
        paths.push(PathBuf::from("givn/changes/benchmark-gate/specs"));
    }
    paths
}

fn staged_features(paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let root = Path::new("target").join("cucumber-features");
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
            .run_and_exit(&staged)
            .await;
    });
}
