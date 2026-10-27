//! Per-power published gold minimums and the review-count check.

use std::fmt;

use anyhow::{Context, Result};

use crate::manifest::{GoldMinimumsMode, GoldSet, Manifest, Power};

/// Published minimums for a primary repository: typo, natural-language,
/// explanation, concept, and concept-equivalence tasks.
pub const PRIMARY_MINIMUMS: &[(&str, u64)] = &[
    ("typo", 50),
    ("natural_language", 30),
    ("explanation", 20),
    ("concept", 50),
    ("concept_equivalence", 20),
];

/// Published minimums for a secondary repository: half of the primary set.
pub const SECONDARY_MINIMUMS: &[(&str, u64)] = &[
    ("typo", 25),
    ("natural_language", 15),
    ("explanation", 10),
    ("concept", 25),
    ("concept_equivalence", 10),
];

/// A manifest-referenced gold set whose reviewed task count is below the
/// published minimum for its repository's power. The run stops before any
/// task is scored and publishes no report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldBelowMinimum {
    pub repository: String,
    pub metric: String,
    pub minimum: u64,
    pub reviewed: u64,
}

impl fmt::Display for GoldBelowMinimum {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "gold_below_minimum: gold set {} {} has {} reviewed tasks, below the published minimum {}",
            self.repository, self.metric, self.reviewed, self.minimum
        )
    }
}

impl std::error::Error for GoldBelowMinimum {}

/// Enforce the published per-power minimum for one manifest-referenced set.
/// A fixture manifest opts out of the published counts; a published manifest
/// must declare a minimum for every referenced metric and every repository
/// must carry its power label.
pub fn ensure_published(manifest: &Manifest, set: &GoldSet, reviewed: u64) -> Result<()> {
    let Some(minimums) = &manifest.gold_minimums else {
        return Ok(());
    };
    if minimums.mode == GoldMinimumsMode::Fixture {
        return Ok(());
    }
    let repository = manifest.repository(&set.repository).with_context(|| {
        format!(
            "gold set {} names repository {}, which the manifest does not fix",
            set.metric, set.repository
        )
    })?;
    let power = repository.power.with_context(|| {
        format!(
            "repository {} has no power label for its published gold minimum",
            repository.id
        )
    })?;
    let (table, label) = match power {
        Power::Primary => (&minimums.primary, "primary"),
        Power::Secondary => (&minimums.secondary, "secondary"),
    };
    let minimum = table.get(&set.metric).copied().with_context(|| {
        format!(
            "manifest declares no {label} gold minimum for {}",
            set.metric
        )
    })?;
    if reviewed < minimum {
        return Err(GoldBelowMinimum {
            repository: set.repository.clone(),
            metric: set.metric.clone(),
            minimum,
            reviewed,
        }
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::GoldMinimums;
    use std::fs;
    use std::path::PathBuf;

    fn manifest_dir(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("benchmark-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create minimums fixture directory");
        fs::write(
            root.join("manifest.yaml"),
            r#"schema_version: 1
run_id: minimums-test
mode: replay
repositories:
  - id: repo-a
    url: "test://repo-a"
    revision: r1
    commit: "1111111111111111111111111111111111111111"
    language: python
    power: primary
    checkout: .
    loc: {language_code_lines: 1, total_code_lines: 1, tool: test, command: test, exclusions: []}
candidates:
  - id: candidate-a
    version: v1
    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    install: test
    adapter: fixture
    adapter_version: 1
gold_minimums:
  mode: published
  primary: {typo: 50, natural_language: 30, explanation: 20, concept: 50, concept_equivalence: 20}
  secondary: {typo: 25, natural_language: 15, explanation: 10, concept: 25, concept_equivalence: 10}
cassettes:
  results: {path: results.jsonl, fingerprint: "sha256:00"}
  model: {path: model.jsonl, fingerprint: "sha256:00"}
"#,
        )
        .expect("write minimums manifest");
        root
    }

    fn gold_set(metric: &str, tasks: u64) -> GoldSet {
        GoldSet {
            repository: "repo-a".to_string(),
            metric: metric.to_string(),
            version: 1,
            tasks,
            fingerprint: "sha256:00".to_string(),
            reviewed_by: Some("maintainer--ortsinn".to_string()),
            path: format!("gold/{metric}.jsonl"),
        }
    }

    #[test]
    fn s17_below_minimum_is_typed_for_the_repository_power_and_fixture_mode_is_exempt() {
        let root = manifest_dir("gold-minimums");
        let manifest = Manifest::load(&root.join("manifest.yaml")).expect("load minimums manifest");
        let minimums: &GoldMinimums = manifest.gold_minimums.as_ref().expect("published minimums");
        for (metric, count) in PRIMARY_MINIMUMS {
            assert_eq!(minimums.primary.get(*metric), Some(count));
        }
        for (metric, count) in SECONDARY_MINIMUMS {
            assert_eq!(minimums.secondary.get(*metric), Some(count));
        }

        let error = ensure_published(&manifest, &gold_set("typo", 49), 49)
            .expect_err("a primary set below 50 typo tasks must fail");
        let typed = error
            .downcast_ref::<GoldBelowMinimum>()
            .expect("typed below-minimum error");
        assert_eq!(typed.repository, "repo-a");
        assert_eq!(typed.metric, "typo");
        assert_eq!(typed.minimum, 50);
        assert_eq!(typed.reviewed, 49);
        assert!(error.to_string().contains("gold_below_minimum"));
        ensure_published(&manifest, &gold_set("typo", 50), 50).expect("a primary set at 50 passes");

        let mut secondary = Manifest::load(&root.join("manifest.yaml")).expect("reload manifest");
        secondary.repositories[0].power = Some(Power::Secondary);
        let error = ensure_published(&secondary, &gold_set("typo", 24), 24)
            .expect_err("a secondary set below 25 typo tasks must fail");
        let typed = error
            .downcast_ref::<GoldBelowMinimum>()
            .expect("typed below-minimum error");
        assert_eq!(typed.minimum, 25);
        assert_eq!(typed.reviewed, 24);
        ensure_published(&secondary, &gold_set("typo", 25), 25)
            .expect("a secondary set at 25 passes");

        let mut fixture = Manifest::load(&root.join("manifest.yaml")).expect("reload manifest");
        if let Some(minimums) = fixture.gold_minimums.as_mut() {
            minimums.mode = GoldMinimumsMode::Fixture;
        }
        ensure_published(&fixture, &gold_set("typo", 1), 1)
            .expect("fixture mode opts out of the published minimums");
        let _ = fs::remove_dir_all(&root);
    }
}
