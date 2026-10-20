//! Report model. `report.json` is the source of truth; `report.md` is
//! rendered from this model.

use serde::{Deserialize, Serialize};

use crate::manifest::RunMode;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub run: RunReport,
    pub inputs: Inputs,
    pub candidates: Vec<CandidateReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub id: String,
    pub mode: RunMode,
    pub command: String,
    pub manifest_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inputs {
    pub repositories: Vec<RepositoryReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryReport {
    pub id: String,
    pub url: String,
    pub revision: String,
    pub commit: String,
    pub language: String,
    pub loc: LineCountReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCountReport {
    pub language_code_lines: u64,
    pub total_code_lines: u64,
    pub tool: String,
    pub command: String,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateReport {
    pub id: String,
    pub version: String,
    pub commit: String,
    pub install: String,
    pub adapter: String,
    pub adapter_version: u32,
}
