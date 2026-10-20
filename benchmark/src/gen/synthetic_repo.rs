//! Deterministic ten-million-line synthetic monorepo.
//!
//! The generator streams one code line at a time so the repository never has
//! to be materialized in memory. Generation is a pure function of the fixed
//! seed and generator version: the same specification produces the same file
//! paths, file contents, content digest, and gold locations.
//!
//! The recorded fixed-input fingerprint (`sha256:synthetic-demo-v1`) is the
//! identifier the manifest records, exactly like the fixture gold and cassette
//! fingerprints. `content_digest` is the SHA-256 over the sorted
//! `(relative path, file content digest)` list and proves regeneration is
//! byte-for-byte stable.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::{Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::manifest::hex;

/// Fixed seed recorded in the manifest.
pub const SEED: u64 = 20261020;
/// Fixed generator version recorded in the manifest.
pub const GENERATOR_VERSION: u32 = 1;
/// Exactly ten million non-blank, non-comment code lines.
pub const CODE_LINES: u64 = 10_000_000;
/// Recorded fixed-input fingerprint.
pub const RECORDED_FINGERPRINT: &str = "sha256:synthetic-demo-v1";
/// Lines per generated file; every language total divides evenly.
pub const LINES_PER_FILE: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    Python,
    Rust,
}

impl Language {
    pub fn name(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::Python => "python",
            Self::Rust => "rust",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::TypeScript => "ts",
            Self::Python => "py",
            Self::Rust => "rs",
        }
    }

    fn code_lines(self) -> u64 {
        match self {
            Self::TypeScript => 4_000_000,
            Self::Python => 3_000_000,
            Self::Rust => 3_000_000,
        }
    }

    fn file_count(self) -> usize {
        (self.code_lines() / u64::from(LINES_PER_FILE)) as usize
    }

    fn path(self, file_index: usize) -> String {
        format!(
            "packages/{}/src/module_{file_index:04}.{}",
            self.name(),
            self.extension()
        )
    }

    fn write_line(self, line: &mut Vec<u8>, symbol_line: u64) {
        line.clear();
        match self {
            Self::TypeScript => {
                line.extend_from_slice(b"export function helper_");
                push_u64(line, symbol_line);
                line.extend_from_slice(b": number { return ");
                push_u64(line, symbol_line);
                line.extend_from_slice(b"; }");
            }
            Self::Python => {
                line.extend_from_slice(b"def helper_");
                push_u64(line, symbol_line);
                line.extend_from_slice(b"() -> int: return ");
                push_u64(line, symbol_line);
            }
            Self::Rust => {
                line.extend_from_slice(b"pub fn helper_");
                push_u64(line, symbol_line);
                line.extend_from_slice(b"() -> u64 { ");
                push_u64(line, symbol_line);
                line.extend_from_slice(b" }");
            }
        }
    }
}

fn push_u64(buffer: &mut Vec<u8>, value: u64) {
    if value == 0 {
        buffer.push(b'0');
        return;
    }
    let mut digits = [0u8; 20];
    let mut index = digits.len();
    let mut remaining = value;
    while remaining > 0 {
        index -= 1;
        digits[index] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
    }
    buffer.extend_from_slice(&digits[index..]);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyntheticLocation {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyntheticGoldTask {
    pub id: String,
    pub metric: String,
    pub query: String,
    pub expected: SyntheticLocation,
    /// The line as it was actually emitted at the expected location.
    pub observed_line: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedFile {
    pub path: String,
    pub language: String,
    pub code_lines: u64,
    pub content_digest: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyntheticRepo {
    pub seed: u64,
    pub generator_version: u32,
    pub recorded_fingerprint: String,
    pub content_digest: String,
    pub code_lines: u64,
    pub lines_by_language: BTreeMap<String, u64>,
    pub files: Vec<GeneratedFile>,
    pub gold: Vec<SyntheticGoldTask>,
}

struct GoldSpec {
    metric: &'static str,
    id: &'static str,
    language: Language,
    file_index: usize,
    line_index: u32,
}

const GOLD_SPECS: [GoldSpec; 9] = [
    GoldSpec {
        metric: "typo",
        id: "syn-typo-1",
        language: Language::TypeScript,
        file_index: 0,
        line_index: 10,
    },
    GoldSpec {
        metric: "typo",
        id: "syn-typo-2",
        language: Language::Python,
        file_index: 3,
        line_index: 25,
    },
    GoldSpec {
        metric: "typo",
        id: "syn-typo-3",
        language: Language::Rust,
        file_index: 7,
        line_index: 40,
    },
    GoldSpec {
        metric: "natural_language",
        id: "syn-nl-1",
        language: Language::TypeScript,
        file_index: 10,
        line_index: 100,
    },
    GoldSpec {
        metric: "natural_language",
        id: "syn-nl-2",
        language: Language::Python,
        file_index: 20,
        line_index: 200,
    },
    GoldSpec {
        metric: "natural_language",
        id: "syn-nl-3",
        language: Language::Rust,
        file_index: 30,
        line_index: 300,
    },
    GoldSpec {
        metric: "concept",
        id: "syn-concept-1",
        language: Language::TypeScript,
        file_index: 399,
        line_index: 9_990,
    },
    GoldSpec {
        metric: "concept",
        id: "syn-concept-2",
        language: Language::Python,
        file_index: 299,
        line_index: 9_990,
    },
    GoldSpec {
        metric: "concept",
        id: "syn-concept-3",
        language: Language::Rust,
        file_index: 299,
        line_index: 9_990,
    },
];

/// Generate in memory, hashing every line without writing a repository tree.
pub fn generate() -> SyntheticRepo {
    generate_with(None).expect("in-memory generation cannot fail")
}

/// Stream the generated repository into `root` while hashing it.
pub fn generate_to(root: &Path) -> Result<SyntheticRepo> {
    generate_with(Some(root))
}

fn generate_with(root: Option<&Path>) -> Result<SyntheticRepo> {
    let specs = &GOLD_SPECS;
    let mut gold: Vec<SyntheticGoldTask> = specs
        .iter()
        .map(|spec| {
            let symbol_line = symbol_line(spec.file_index, spec.line_index);
            SyntheticGoldTask {
                id: spec.id.to_string(),
                metric: spec.metric.to_string(),
                query: format!("definition of helper_{symbol_line}"),
                expected: SyntheticLocation {
                    path: spec.language.path(spec.file_index),
                    start_line: spec.line_index,
                    end_line: spec.line_index,
                    symbol: format!("helper_{symbol_line}"),
                },
                observed_line: String::new(),
            }
        })
        .collect();

    let mut files = Vec::new();
    let mut lines_by_language = BTreeMap::new();
    let mut total_lines = 0u64;
    let mut line: Vec<u8> = Vec::with_capacity(96);

    for language in [Language::TypeScript, Language::Python, Language::Rust] {
        let mut language_lines = 0u64;
        for file_index in 0..language.file_count() {
            let path = language.path(file_index);
            let mut file_gold: HashMap<u32, usize> = HashMap::new();
            for (index, task) in gold.iter().enumerate() {
                if task.expected.path == path {
                    file_gold.insert(task.expected.start_line, index);
                }
            }

            let mut writer = match root {
                Some(root) => {
                    let full = root.join(&path);
                    if let Some(parent) = full.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    Some(BufWriter::new(fs::File::create(&full)?))
                }
                None => None,
            };
            let mut hasher = Sha256::new();
            for line_index in 1..=LINES_PER_FILE {
                language_lines += 1;
                let symbol_line = symbol_line(file_index, line_index);
                language.write_line(&mut line, symbol_line);
                hasher.update(&line);
                hasher.update(b"\n");
                if let Some(writer) = writer.as_mut() {
                    writer.write_all(&line)?;
                    writer.write_all(b"\n")?;
                }
                if let Some(index) = file_gold.get(&line_index) {
                    gold[*index].observed_line =
                        std::str::from_utf8(&line).expect("synthetic lines are UTF-8").to_string();
                }
            }
            if let Some(mut writer) = writer {
                writer.flush()?;
            }
            total_lines += u64::from(LINES_PER_FILE);
            files.push(GeneratedFile {
                path,
                language: language.name().to_string(),
                code_lines: u64::from(LINES_PER_FILE),
                content_digest: format!("sha256:{}", hex(&hasher.finalize())),
            });
        }
        lines_by_language.insert(language.name().to_string(), language_lines);
    }

    ensure!(
        total_lines == CODE_LINES,
        "generated {total_lines} code lines, expected {CODE_LINES}"
    );
    ensure!(
        gold.iter().all(|task| !task.observed_line.is_empty()),
        "a synthetic gold location did not resolve to a generated line"
    );

    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut root_hasher = Sha256::new();
    for file in &files {
        root_hasher.update(file.path.as_bytes());
        root_hasher.update(b"\0");
        root_hasher.update(file.content_digest.as_bytes());
        root_hasher.update(b"\n");
    }

    Ok(SyntheticRepo {
        seed: SEED,
        generator_version: GENERATOR_VERSION,
        recorded_fingerprint: RECORDED_FINGERPRINT.to_string(),
        content_digest: format!("sha256:{}", hex(&root_hasher.finalize())),
        code_lines: total_lines,
        lines_by_language,
        files,
        gold,
    })
}

/// Global symbol number for a file-local line. It is embedded in the line
/// text, so a gold location can be checked against the emitted content.
fn symbol_line(file_index: usize, line_index: u32) -> u64 {
    file_index as u64 * u64::from(LINES_PER_FILE) + u64::from(line_index)
}
