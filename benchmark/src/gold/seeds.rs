//! Reviewed gold seeds for the real reference repositories.
//!
//! A seed names one definition in a fixed checkout. The proposer reads the
//! seed file and the checkout together: the seed supplies the reviewed query
//! intent, the checkout supplies the reviewed line range. A seed whose
//! definition cannot be located, or that carries no documentation, is a
//! review error rather than a silently skipped task.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Reference language of one seed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeedLanguage {
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "rust")]
    Rust,
}

/// The reviewed seeds of one repository.
#[derive(Debug, Clone, Deserialize)]
pub struct SeedFile {
    pub repository: String,
    pub language: SeedLanguage,
    pub seeds: Vec<Seed>,
}

/// One reviewed definition. `query` fields are optional overrides; when they
/// are absent the proposer derives the query deterministically from the
/// definition, so the seed file records only what the maintainer decided.
#[derive(Debug, Clone, Deserialize)]
pub struct Seed {
    pub path: String,
    pub symbol: String,
    #[serde(default)]
    pub concept: Option<String>,
    #[serde(default)]
    pub natural_language: Option<String>,
    #[serde(default)]
    pub equivalence: Option<String>,
    #[serde(default)]
    pub typo: Option<String>,
}

impl SeedFile {
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            fs::read_to_string(path).with_context(|| format!("read seed file {}", path.display()))?;
        let seeds: Self = serde_yaml_ng::from_str(&text)
            .with_context(|| format!("parse seed file {}", path.display()))?;
        anyhow::ensure!(
            !seeds.seeds.is_empty(),
            "seed file {} has no seeds",
            path.display()
        );
        for seed in &seeds.seeds {
            anyhow::ensure!(
                !seed.symbol.trim().is_empty(),
                "seed file {}: {} names an empty symbol",
                path.display(),
                seed.path
            );
        }
        Ok(seeds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn s12_seed_file_requires_a_language_and_a_symbol() {
        let dir = std::env::temp_dir().join(format!(
            "benchmark-seeds-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("seeds.yaml");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "repository: r\nlanguage: rust\nseeds:\n  - path: a.rs\n    symbol: ''")
            .unwrap();
        let error = SeedFile::load(&path).expect_err("an empty symbol is a review error");
        assert!(
            error.to_string().contains("empty symbol"),
            "unexpected error: {error:#}"
        );
        fs::write(
            &path,
            "repository: r\nlanguage: rust\nseeds:\n  - path: a.rs\n    symbol: map\n",
        )
        .unwrap();
        let seeds = SeedFile::load(&path).expect("a named seed loads");
        assert_eq!(seeds.language, SeedLanguage::Rust);
        assert_eq!(seeds.seeds[0].symbol, "map");
        fs::remove_dir_all(&dir).ok();
    }
}
