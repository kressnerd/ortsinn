//! Draft gold proposal. A draft is written under `gold/drafts/` and is never
//! loaded or scored: only a reviewed file the manifest references reaches
//! `Gold::load`. The maintainer reviews a draft, moves it into the reviewed
//! tree, and adds its manifest entry with `reviewed_by`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Directory, relative to the gold root, that holds unreviewed proposals.
pub const DRAFTS_DIR: &str = "drafts";

/// Path of an unreviewed draft file for one repository and metric.
pub fn draft_path(gold_root: &Path, repository: &str, metric: &str) -> PathBuf {
    gold_root
        .join(DRAFTS_DIR)
        .join(repository)
        .join(format!("{metric}.jsonl"))
}

/// Write unreviewed draft tasks. The reviewed tree is never touched, and the
/// manifest is never updated, so a draft cannot silently join a gold set.
pub fn write_draft(
    gold_root: &Path,
    repository: &str,
    metric: &str,
    tasks: &[serde_json::Value],
) -> Result<PathBuf> {
    let path = draft_path(gold_root, repository, metric);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create draft directory {}", parent.display()))?;
    }
    let mut text = String::new();
    for task in tasks {
        text.push_str(&serde_json::to_string(task).context("serialize draft task")?);
        text.push('\n');
    }
    fs::write(&path, text).with_context(|| format!("write draft {}", path.display()))?;
    Ok(path)
}
