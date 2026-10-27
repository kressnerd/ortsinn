//! Fixed-input setup validation shared by the setup scripts and the specs.
//!
//! The manifest is the only origin of candidate and repository pins. The setup
//! lock records the probes and resolved toolchain versions plus a copy of the
//! pins it installed and verified, tagged with the manifest as their origin.
//! Validation never re-resolves a pin: it compares the lock back to the
//! manifest and checks that a checkout still points at the pinned commit.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::manifest::{Manifest, Power};

/// Process exit code for a fixed input that does not match its pin.
pub const EXIT_INPUT_ERROR: i32 = 2;

/// The committed setup lock. It is not a pin origin: every `pinned_by` names
/// the manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetupLock {
    pub schema_version: u32,
    /// Manifest this lock was produced from, relative to the project root.
    pub manifest: String,
    /// UTC timestamp of the setup pass.
    pub recorded_at: String,
    pub toolchains: BTreeMap<String, String>,
    pub candidates: BTreeMap<String, LockedCandidate>,
    pub repositories: BTreeMap<String, LockedRepository>,
    pub endpoints: Endpoints,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LockedCandidate {
    pub version: String,
    pub commit: String,
    /// Origin of the pin. Always the manifest path.
    pub pinned_by: String,
    pub probe: Probe,
    pub toolchain: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Probe {
    /// `ok` when the documented interface started and answered its probe.
    pub status: String,
    pub command: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LockedRepository {
    /// `None` for the generated synthetic input, which has no git commit.
    #[serde(default)]
    pub commit: Option<String>,
    pub pinned_by: String,
    #[serde(default)]
    pub synthetic: Option<LockedSynthetic>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LockedSynthetic {
    pub seed: u64,
    pub generator_version: u32,
    pub content_fingerprint: String,
}

/// Hybrid model boundary. `configured` is false until the operator supplies
/// the external explanation endpoint; the placeholder model is never used by
/// a run.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Endpoints {
    pub embedding: Endpoint,
    pub explanation: Endpoint,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Endpoint {
    pub configured: bool,
    pub locality: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub query_budget: Option<u64>,
}

impl SetupLock {
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            fs::read_to_string(path).with_context(|| format!("read setup lock {}", path.display()))?;
        let lock: Self = serde_json::from_str(&text)
            .with_context(|| format!("parse setup lock {}", path.display()))?;
        Ok(lock)
    }
}

/// Compare the committed lock with the manifest it installs. This is the
/// validation `setup.sh` calls; it reads no network and starts no candidate.
pub fn validate_pins(manifest_path: &Path, lock_path: &Path) -> Result<()> {
    let manifest = Manifest::load(manifest_path)?;
    let lock = SetupLock::load(lock_path)?;
    anyhow::ensure!(
        lock.schema_version == 1,
        "unsupported setup lock schema version {}",
        lock.schema_version
    );
    anyhow::ensure!(
        !lock.manifest.is_empty(),
        "setup lock does not name the manifest it was produced from"
    );

    for candidate in &manifest.candidates {
        let entry = lock.candidates.get(&candidate.id).with_context(|| {
            format!(
                "setup lock has no probe record for candidate {}",
                candidate.id
            )
        })?;
        anyhow::ensure!(
            entry.version == candidate.version,
            "candidate pin mismatch for {}: manifest version {}, lock version {}",
            candidate.id,
            candidate.version,
            entry.version
        );
        anyhow::ensure!(
            entry.commit == candidate.commit,
            "candidate pin mismatch for {}: manifest commit {}, lock commit {}",
            candidate.id,
            candidate.commit,
            entry.commit
        );
        anyhow::ensure!(
            is_commit(&entry.commit),
            "candidate {} commit {} is not a 40-character commit",
            candidate.id,
            entry.commit
        );
        anyhow::ensure!(
            pin_origin_is_manifest(&entry.pinned_by, manifest_path),
            "candidate {} pin origin {} is not the manifest",
            candidate.id,
            entry.pinned_by
        );
        anyhow::ensure!(
            !entry.probe.status.is_empty() && entry.probe.status != "missing",
            "setup lock has no probe result for candidate {}",
            candidate.id
        );
        anyhow::ensure!(
            !entry.toolchain.is_empty(),
            "setup lock has no toolchain versions for candidate {}",
            candidate.id
        );
    }

    for repository in &manifest.repositories {
        let entry = lock.repositories.get(&repository.id).with_context(|| {
            format!(
                "setup lock has no record for repository {}",
                repository.id
            )
        })?;
        anyhow::ensure!(
            pin_origin_is_manifest(&entry.pinned_by, manifest_path),
            "repository {} pin origin {} is not the manifest",
            repository.id,
            entry.pinned_by
        );
        match (&repository.synthetic, &entry.synthetic) {
            (Some(manifest_synthetic), Some(lock_synthetic)) => {
                anyhow::ensure!(
                    manifest_synthetic.seed == lock_synthetic.seed
                        && manifest_synthetic.generator_version
                            == lock_synthetic.generator_version
                        && manifest_synthetic.content_fingerprint
                            == lock_synthetic.content_fingerprint,
                    "synthetic fixed-input pin mismatch for {}",
                    repository.id
                );
                anyhow::ensure!(
                    entry.commit.is_none() || entry.commit.as_deref() == Some(""),
                    "synthetic repository {} must not record a git commit",
                    repository.id
                );
            }
            (None, _) => {
                let locked_commit = entry.commit.as_deref().with_context(|| {
                    format!(
                        "setup lock has no commit for repository {}",
                        repository.id
                    )
                })?;
                anyhow::ensure!(
                    locked_commit == repository.commit,
                    "repository pin mismatch for {}: manifest commit {}, lock commit {}",
                    repository.id,
                    repository.commit,
                    locked_commit
                );
                anyhow::ensure!(
                    is_commit(locked_commit),
                    "repository {} commit {} is not a 40-character commit",
                    repository.id,
                    locked_commit
                );
            }
            (Some(_), None) => anyhow::bail!(
                "setup lock has no synthetic pin for repository {}",
                repository.id
            ),
        }
    }

    validate_endpoints(&lock.endpoints)?;
    Ok(())
}

fn validate_endpoints(endpoints: &Endpoints) -> Result<()> {
    anyhow::ensure!(
        !endpoints.embedding.locality.is_empty(),
        "setup lock does not record the embedding endpoint locality"
    );
    anyhow::ensure!(
        endpoints.embedding.locality == "loopback",
        "embedding endpoint locality must be loopback, recorded {}",
        endpoints.embedding.locality
    );
    anyhow::ensure!(
        endpoints
            .embedding
            .model
            .as_deref()
            .is_some_and(|model| !model.is_empty()),
        "setup lock does not record the embedding model name"
    );
    anyhow::ensure!(
        !endpoints.explanation.locality.is_empty(),
        "setup lock does not record the explanation endpoint locality"
    );
    anyhow::ensure!(
        endpoints
            .explanation
            .model
            .as_deref()
            .is_some_and(|model| !model.is_empty()),
        "setup lock does not record the explanation model name"
    );
    let budget = endpoints
        .explanation
        .query_budget
        .context("setup lock does not record the explanation-query budget")?;
    anyhow::ensure!(
        budget <= 80,
        "explanation-query budget {budget} exceeds the reviewed-gold ceiling 80"
    );
    Ok(())
}

/// Verify the materialized fixed inputs against the manifest pins: every
/// public checkout points at its commit, every candidate installation path
/// exists, and the synthetic checkout exists. No network, no candidate
/// process, and no scoring.
pub fn validate_fixed_inputs(manifest_path: &Path) -> Result<()> {
    let manifest = Manifest::load(manifest_path)?;
    let base = Manifest::base_dir(manifest_path);
    for repository in &manifest.repositories {
        let checkout = base.join(&repository.checkout);
        if repository.synthetic.is_some() {
            anyhow::ensure!(
                checkout.is_dir(),
                "fixed input mismatch for synthetic repository {}: checkout {} does not exist",
                repository.id,
                checkout.display()
            );
            continue;
        }
        anyhow::ensure!(
            checkout.is_dir(),
            "fixed input mismatch for repository {}: checkout {} does not exist",
            repository.id,
            checkout.display()
        );
        let actual = git_head(&checkout).with_context(|| {
            format!(
                "fixed input mismatch for repository {}: cannot read the checkout commit in {}",
                repository.id,
                checkout.display()
            )
        })?;
        anyhow::ensure!(
            actual == repository.commit,
            "fixed input mismatch for repository {}: expected commit {}, actual commit {}",
            repository.id,
            repository.commit,
            actual
        );
    }
    for candidate in &manifest.candidates {
        let install = PathBuf::from(&candidate.install);
        let install = if install.is_absolute() {
            install
        } else {
            base.join(install)
        };
        anyhow::ensure!(
            install.exists(),
            "fixed input mismatch for candidate {}: installation {} does not exist",
            candidate.id,
            install.display()
        );
    }
    Ok(())
}

fn git_head(checkout: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .context("run git rev-parse HEAD")?;
    anyhow::ensure!(
        output.status.success(),
        "git rev-parse HEAD failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn pin_origin_is_manifest(origin: &str, manifest_path: &Path) -> bool {
    if origin.is_empty() {
        return false;
    }
    let normalized = origin.replace('\\', "/");
    normalized == "benchmark/manifest.yaml"
        || manifest_path
            .to_string_lossy()
            .replace('\\', "/")
            .ends_with(&normalized)
        || normalized.ends_with("/manifest.yaml")
}

/// The primary repository carries the full published gold minima; every other
/// repository is lower-power.
pub fn is_primary(power: Option<Power>) -> bool {
    matches!(power, Some(Power::Primary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        Candidate, CandidateEvidence, CandidateSetup, CassetteRef, Cassettes, LineCount, Power,
        Repository, RunMode,
    };

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "benchmark-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp directory");
        dir
    }

    fn manifest() -> Manifest {
        Manifest {
            schema_version: 1,
            run_id: "test".to_string(),
            mode: RunMode::Replay,
            repositories: vec![Repository {
                id: "typescript-reference".to_string(),
                url: "https://example.invalid/typescript-reference".to_string(),
                revision: "v1".to_string(),
                commit: "1e4744d68260a7cb91b62b12edc3f6a2187faaf1".to_string(),
                language: "typescript".to_string(),
                power: Some(Power::Primary),
                minimum_code_lines: Some(200_000),
                loc: LineCount {
                    language_code_lines: 731_537,
                    total_code_lines: 731_537,
                    tool: "tokei".to_string(),
                    command: "tokei -t TypeScript".to_string(),
                    exclusions: Vec::new(),
                },
                checkout: "checkout".to_string(),
                synthetic: None,
            }],
            candidates: vec![Candidate {
                id: "ocbi".to_string(),
                version: "v0.37.0".to_string(),
                commit: "573b71c3d5cfaba3a83d602f7534605047a01363".to_string(),
                install: "work/tools/ocbi".to_string(),
                adapter: "ocbi".to_string(),
                adapter_version: 1,
                evidence: Some(CandidateEvidence {
                    version: "https://example.invalid/v0.37.0".to_string(),
                    commit:
                        "https://example.invalid/573b71c3d5cfaba3a83d602f7534605047a01363"
                            .to_string(),
                }),
                capabilities: BTreeMap::new(),
                setup: CandidateSetup::default(),
                index_deadline_seconds: None,
                unavailable: None,
            }],
            targets: BTreeMap::new(),
            targets_file: None,
            gold_minimums: None,
            gold: Vec::new(),
            cassettes: Cassettes {
                results: CassetteRef {
                    path: "results.jsonl".to_string(),
                    fingerprint: String::new(),
                },
                model: CassetteRef {
                    path: "model.jsonl".to_string(),
                    fingerprint: String::new(),
                },
            },
            index_deadlines: BTreeMap::new(),
        }
    }

    fn lock(manifest_path: &Path) -> SetupLock {
        let mut candidates = BTreeMap::new();
        candidates.insert(
            "ocbi".to_string(),
            LockedCandidate {
                version: "v0.37.0".to_string(),
                commit: "573b71c3d5cfaba3a83d602f7534605047a01363".to_string(),
                pinned_by: manifest_path.to_string_lossy().to_string(),
                probe: Probe {
                    status: "ok".to_string(),
                    command: "ocbi --help".to_string(),
                    detail: Some("Usage".to_string()),
                },
                toolchain: BTreeMap::from([("node".to_string(), "v24.21.0".to_string())]),
            },
        );
        let mut repositories = BTreeMap::new();
        repositories.insert(
            "typescript-reference".to_string(),
            LockedRepository {
                commit: Some("1e4744d68260a7cb91b62b12edc3f6a2187faaf1".to_string()),
                pinned_by: manifest_path.to_string_lossy().to_string(),
                synthetic: None,
            },
        );
        SetupLock {
            schema_version: 1,
            manifest: manifest_path.to_string_lossy().to_string(),
            recorded_at: "2026-10-27T00:00:00Z".to_string(),
            toolchains: BTreeMap::from([("rustc".to_string(), "rustc 1.99.1".to_string())]),
            candidates,
            repositories,
            endpoints: Endpoints {
                embedding: Endpoint {
                    configured: true,
                    locality: "loopback".to_string(),
                    base_url: Some("http://127.0.0.1:11434/v1".to_string()),
                    model: Some("nomic-embed-text".to_string()),
                    query_budget: None,
                },
                explanation: Endpoint {
                    configured: false,
                    locality: "external".to_string(),
                    base_url: None,
                    model: Some("unconfigured".to_string()),
                    query_budget: Some(80),
                },
            },
        }
    }

    /// S3: the setup lock must agree with the manifest pins, record probes and
    /// toolchains, keep the manifest as the only pin origin, and record both
    /// endpoint localities/models and the explanation budget.
    #[test]
    fn s3_setup_lock_agrees_with_pins_and_records_endpoints() {
        let dir = temp_dir("s3");
        let manifest_path = dir.join("manifest.yaml");
        fs::write(
            &manifest_path,
            serde_yaml_ng::to_string(&manifest()).expect("serialize manifest"),
        )
        .expect("write manifest");
        let lock_path = dir.join("setup.lock.json");
        let valid = lock(&manifest_path);
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&valid).expect("serialize lock"),
        )
        .expect("write lock");
        validate_pins(&manifest_path, &lock_path).expect("valid pins agree");

        let mut pin_mismatch = valid.clone();
        pin_mismatch
            .candidates
            .get_mut("ocbi")
            .expect("candidate")
            .commit = "5ff375db862c17dbf40168fa3924696baf1839a6".to_string();
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&pin_mismatch).expect("serialize lock"),
        )
        .expect("write lock");
        let error = validate_pins(&manifest_path, &lock_path).expect_err("pin mismatch must fail");
        assert!(format!("{error:#}").contains("candidate pin mismatch"), "{error:#}");

        let mut self_origin = valid.clone();
        self_origin
            .candidates
            .get_mut("ocbi")
            .expect("candidate")
            .pinned_by = "benchmark/setup.lock.json".to_string();
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&self_origin).expect("serialize lock"),
        )
        .expect("write lock");
        let error = validate_pins(&manifest_path, &lock_path).expect_err("self origin must fail");
        assert!(format!("{error:#}").contains("not the manifest"), "{error:#}");

        let mut no_explanation_model = valid;
        no_explanation_model.endpoints.explanation.model = None;
        fs::write(
            &lock_path,
            serde_json::to_string_pretty(&no_explanation_model).expect("serialize lock"),
        )
        .expect("write lock");
        let error =
            validate_pins(&manifest_path, &lock_path).expect_err("missing endpoint model must fail");
        assert!(
            format!("{error:#}").contains("explanation model name"),
            "{error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    fn git_commit(checkout: &Path) -> String {
        let run = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(checkout)
                .args(["-c", "user.name=benchmark", "-c", "user.email=benchmark@example.invalid"])
                .args(args)
                .output()
                .expect("run git");
            assert!(output.status.success(), "git {args:?} failed");
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };
        run(&["init", "-q"]);
        run(&["add", "file.ts"]);
        run(&["commit", "-q", "-m", "checkout"]);
        run(&["rev-parse", "HEAD"])
    }

    /// S4: a checkout whose HEAD differs from the manifest pin stops
    /// validation with exit code 2 and the input's name; a missing candidate
    /// installation does the same, and no report is written.
    #[test]
    fn s4_fixed_input_drift_is_named_and_stops_before_report() {
        let dir = temp_dir("s4");
        let checkout = dir.join("checkout");
        fs::create_dir_all(&checkout).expect("create checkout");
        fs::write(checkout.join("file.ts"), "export const value = 1;\n").expect("write file");
        let actual = git_commit(&checkout);

        let mut manifest = manifest();
        manifest.repositories[0].checkout = checkout.to_string_lossy().to_string();
        manifest.candidates[0].install = "/bin/sh".to_string();
        let manifest_path = dir.join("manifest.yaml");
        fs::write(
            &manifest_path,
            serde_yaml_ng::to_string(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");

        let error = validate_fixed_inputs(&manifest_path).expect_err("wrong checkout must fail");
        let message = format!("{error:#}");
        assert!(message.contains("typescript-reference"), "{message}");
        assert!(message.contains("expected commit"), "{message}");
        assert!(message.contains("actual commit"), "{message}");
        assert!(message.contains(&actual), "{message}");
        assert_eq!(EXIT_INPUT_ERROR, 2);
        let out = dir.join("out");
        for name in ["report.json", "report.md"] {
            assert!(!out.join(name).exists(), "drift wrote {name}");
        }

        manifest.repositories[0].commit = actual;
        manifest.candidates[0].install = dir.join("missing-tool").to_string_lossy().to_string();
        fs::write(
            &manifest_path,
            serde_yaml_ng::to_string(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");
        let error = validate_fixed_inputs(&manifest_path).expect_err("missing install must fail");
        let message = format!("{error:#}");
        assert!(message.contains("ocbi"), "{message}");
        assert!(message.contains("installation"), "{message}");
        assert!(message.contains("does not exist"), "{message}");
        for name in ["report.json", "report.md"] {
            assert!(!out.join(name).exists(), "missing install wrote {name}");
        }

        manifest.candidates[0].install = "/bin/sh".to_string();
        fs::write(
            &manifest_path,
            serde_yaml_ng::to_string(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");
        validate_fixed_inputs(&manifest_path).expect("matching checkout and install pass");

        let _ = fs::remove_dir_all(&dir);
    }
}
