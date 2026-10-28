//! The committed provenance record for a real record run.
//!
//! The record is evidence, not a log: it names the exact command, both
//! endpoint localities and models, the explanation budget and consumed count,
//! every manifest pin, every index outcome, and the fingerprints of the
//! manifest, targets, and both cassettes. Scenarios 7, 11, and 15 cross-check
//! it against the committed manifest, setup lock, cassettes, and report.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::manifest::{Capability, Manifest, Power};
use crate::record::cassette;
use crate::setup::SetupLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub schema_version: u32,
    pub recording_mode: String,
    pub command: String,
    pub utc_start: String,
    pub utc_end: String,
    pub host: Host,
    pub toolchains: BTreeMap<String, String>,
    pub endpoints: Endpoints,
    pub candidates: Vec<CandidatePin>,
    pub repositories: Vec<RepositoryPin>,
    pub index_outcomes: Vec<IndexOutcome>,
    pub fingerprints: Fingerprints,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Host {
    pub os: String,
    pub kernel: String,
    pub architecture: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoints {
    pub embedding: Endpoint,
    pub explanation: Endpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub locality: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_budget: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries_consumed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CandidatePin {
    pub id: String,
    pub version: String,
    pub commit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<UnavailablePin>,
    #[serde(default)]
    pub capabilities: BTreeMap<String, Capability>,
    #[serde(default)]
    pub setup: crate::manifest::CandidateSetup,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnavailablePin {
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryPin {
    pub id: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexOutcome {
    pub candidate: String,
    pub repository: String,
    pub status: String,
    pub seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fingerprints {
    pub manifest: String,
    pub targets: String,
    pub model_cassette: String,
    pub result_cassette: String,
}

pub fn utc_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn build(
    command: &str,
    utc_start: &str,
    utc_end: &str,
    manifest: &Manifest,
    base: &Path,
    lock: &SetupLock,
    model_cassette: &Path,
    result_cassette: &Path,
    queries_consumed: u64,
) -> Result<Provenance> {
    let embedding = &lock.endpoints.embedding;
    let explanation = &lock.endpoints.explanation;
    Ok(Provenance {
        schema_version: 1,
        recording_mode: "record".to_string(),
        command: command.to_string(),
        utc_start: utc_start.to_string(),
        utc_end: utc_end.to_string(),
        host: Host {
            os: std::env::consts::OS.to_string(),
            kernel: kernel_version(),
            architecture: std::env::consts::ARCH.to_string(),
        },
        toolchains: lock.toolchains.clone(),
        endpoints: Endpoints {
            embedding: Endpoint {
                locality: embedding.locality.clone(),
                model: embedding.model.clone().unwrap_or_default(),
                query_budget: None,
                queries_consumed: None,
            },
            explanation: Endpoint {
                locality: explanation.locality.clone(),
                model: explanation.model.clone().unwrap_or_default(),
                query_budget: explanation.query_budget,
                queries_consumed: Some(queries_consumed),
            },
        },
        candidates: manifest
            .candidates
            .iter()
            .map(|candidate| CandidatePin {
                id: candidate.id.clone(),
                version: candidate.version.clone(),
                commit: candidate.commit.clone(),
                unavailable: candidate.unavailable.as_ref().map(|unavailable| {
                    UnavailablePin {
                        reason: unavailable.reason.clone(),
                        detail: unavailable.detail.clone(),
                    }
                }),
                capabilities: candidate.capabilities.clone(),
                setup: candidate.setup.clone(),
            })
            .collect(),
        repositories: manifest
            .repositories
            .iter()
            .map(|repository| RepositoryPin {
                id: repository.id.clone(),
                commit: repository.commit.clone(),
                power: repository.power.map(power_label),
            })
            .collect(),
        index_outcomes: manifest
            .candidates
            .iter()
            .flat_map(|candidate| {
                candidate.setup.indexes.iter().map(move |index| IndexOutcome {
                    candidate: candidate.id.clone(),
                    repository: index.repository.clone(),
                    status: index.status.clone(),
                    seconds: index.seconds,
                    deadline_seconds: index.deadline_seconds,
                    reason: index.reason.clone(),
                })
            })
            .collect(),
        fingerprints: Fingerprints {
            manifest: manifest.fingerprint()?,
            targets: targets_fingerprint(manifest, base)?,
            model_cassette: cassette::fingerprint(model_cassette)?,
            result_cassette: cassette::fingerprint(result_cassette)?,
        },
    })
}

fn power_label(power: Power) -> String {
    match power {
        Power::Primary => "primary".to_string(),
        Power::Secondary => "secondary".to_string(),
    }
}

fn kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Fingerprint the committed targets. A manifest with a `targets_file`
/// fingerprints that file; an inline manifest fingerprints its target map.
pub fn targets_fingerprint(manifest: &Manifest, base: &Path) -> Result<String> {
    if let Some(targets_file) = &manifest.targets_file {
        return file_fingerprint(&base.join(targets_file));
    }
    let bytes = serde_json::to_vec(&manifest.targets)
        .context("serialize inline targets")?;
    Ok(format!("sha256:{}", hex(Sha256::digest(&bytes))))
}

pub fn file_fingerprint(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(format!("sha256:{}", hex(Sha256::digest(&bytes))))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn load(path: &Path) -> Result<Provenance> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read provenance {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse provenance {}", path.display()))
}

/// Write the provenance record and return its fingerprint.
pub fn write(provenance: &Provenance, path: &Path) -> Result<String> {
    let bytes = serde_json::to_vec_pretty(provenance)
        .context("serialize provenance record")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(path, &bytes)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(format!("sha256:{}", hex(Sha256::digest(&bytes))))
}
