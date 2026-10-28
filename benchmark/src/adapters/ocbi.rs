//! open-codebase-index adapter.
//!
//! The documented interface is the `ocbi` CLI. The adapter configures the
//! candidate's embedding provider at the loopback OpenAI-compatible endpoint
//! and parses the documented search output order; it never reorders or
//! promotes a file-only answer.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::json;

use super::real::{CommandOutcome, RealContext, run_command};
use super::session::{
    CandidateAnswer, CandidateSession, ExplainRequest, Operation, QueryRequest, SessionError,
};
use crate::adapters::ExplanationResponse;

const RESULT_LIMIT: &str = "10";
const EMBEDDING_DIMENSIONS: u64 = 768;

#[derive(Debug)]
pub struct OcbiSession {
    context: RealContext,
    config_path: PathBuf,
}

impl OcbiSession {
    pub fn new(context: RealContext) -> Result<Self> {
        let Some(endpoint) = &context.embedding_endpoint else {
            bail!("open-codebase-index requires the loopback embedding endpoint");
        };
        let config_path = context.base.join("work/ocbi-config.json");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let model = context
            .candidate
            .setup
            .embedding
            .as_ref()
            .and_then(|embedding| embedding.model.clone())
            .unwrap_or_else(|| "nomic-embed-text".to_string());
        let config = json!({
            "embeddingProvider": "custom",
            "customProvider": {
                "baseUrl": endpoint.trim_end_matches('/'),
                "model": model,
                "dimensions": EMBEDDING_DIMENSIONS,
            }
        });
        std::fs::write(
            &config_path,
            serde_json::to_vec_pretty(&config).context("serialize ocbi config")?,
        )
        .with_context(|| format!("write {}", config_path.display()))?;
        Ok(Self {
            context,
            config_path,
        })
    }

    fn command(&self, args: &[&str]) -> Result<CommandOutcome> {
        let mut owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        owned.push("--config".to_string());
        owned.push(self.config_path.display().to_string());
        run_command(
            &self.context.install(),
            &owned,
            &self.context.base,
            Some(Duration::from_secs(600)),
        )
    }

    fn search(&self, repository: &str, query: &str) -> Result<Vec<CandidateAnswer>, SessionError> {
        let checkout = self
            .context
            .checkout(repository)
            .map_err(|error| SessionError::with_detail("missing_checkout", error.to_string()))?;
        let outcome = self
            .command(&[
                "search",
                query,
                "--limit",
                RESULT_LIMIT,
                "--project",
                &checkout.display().to_string(),
            ])
            .map_err(|error| SessionError::with_detail("query_failed", error.to_string()))?;
        if !outcome.succeeded() {
            return Err(SessionError::with_detail(
                "query_failed",
                format!("ocbi search exited {:?}: {}", outcome.code, outcome.detail()),
            ));
        }
        Ok(parse_search_output(&checkout, repository, &outcome.stdout))
    }
}

impl CandidateSession for OcbiSession {
    fn probe(&mut self) -> Result<(), SessionError> {
        let outcome = run_command(
            &self.context.install(),
            &["--help".to_string()],
            &self.context.base,
            Some(Duration::from_secs(60)),
        )
        .map_err(|error| SessionError::with_detail("probe_failed", error.to_string()))?;
        if !outcome.succeeded() {
            return Err(SessionError::with_detail(
                "probe_failed",
                format!("ocbi --help exited {:?}: {}", outcome.code, outcome.detail()),
            ));
        }
        Ok(())
    }

    fn operations(&self) -> Vec<Operation> {
        vec![
            Operation::Typo,
            Operation::NaturalLanguage,
            Operation::Concept,
            Operation::ConceptEquivalence,
        ]
    }

    fn setup(&self) -> crate::manifest::CandidateSetup {
        self.context.candidate.setup.clone()
    }

    fn index(&mut self, repository: &str) -> Result<(), SessionError> {
        let checkout = self
            .context
            .checkout(repository)
            .map_err(|error| SessionError::with_detail("missing_checkout", error.to_string()))?;
        let deadline = self.context.deadline(repository);
        let outcome = run_command(
            &self.context.install(),
            &[
                "index".to_string(),
                "--project".to_string(),
                checkout.display().to_string(),
                "--config".to_string(),
                self.config_path.display().to_string(),
            ],
            &checkout,
            deadline.map(Duration::from_secs),
        )
        .map_err(|error| SessionError::with_detail("index_failed", error.to_string()))?;
        if outcome.timed_out {
            return Err(SessionError::with_detail(
                "index_timeout",
                format!("index exceeded {} s", deadline.unwrap_or_default()),
            ));
        }
        if !outcome.succeeded() {
            return Err(SessionError::with_detail(
                "index_failed",
                format!("ocbi index exited {:?}: {}", outcome.code, outcome.detail()),
            ));
        }
        Ok(())
    }

    fn query(&mut self, request: &QueryRequest) -> Result<Vec<CandidateAnswer>, SessionError> {
        if request.arm != "default" {
            return Err(SessionError::new("no_expansion_switch"));
        }
        self.search(&request.repository, &request.query)
    }

    fn explain(&mut self, _request: &ExplainRequest) -> Result<ExplanationResponse, SessionError> {
        Err(SessionError::new("no_explanation_operation"))
    }
}

/// Parse the documented `ocbi search` output:
///
/// ```text
/// [1] function_definition "text_helper_7" in /repo/src/text_utils.py:7-8 (score: 0.96)
/// ```
///
/// The order is the candidate's order and is preserved exactly.
fn parse_search_output(
    checkout: &Path,
    _repository: &str,
    stdout: &str,
) -> Vec<CandidateAnswer> {
    let mut answers = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        let Some((_, rest)) = line.split_once("] ") else {
            continue;
        };
        let Some((kind, rest)) = rest.split_once(' ') else {
            continue;
        };
        let Some((symbol, rest)) = quoted(rest) else {
            continue;
        };
        let Some(location) = rest.trim().strip_prefix("in ") else {
            continue;
        };
        let location = location
            .split(" (score:")
            .next()
            .unwrap_or(location)
            .trim();
        let Some((path, range)) = location.rsplit_once(':') else {
            continue;
        };
        let path = Path::new(path)
            .strip_prefix(checkout)
            .unwrap_or(Path::new(path))
            .to_string_lossy()
            .replace('\\', "/");
        let (start_line, end_line) = match range.split_once('-') {
            Some((start, end)) => match (start.parse::<u32>(), end.parse::<u32>()) {
                (Ok(start), Ok(end)) => (Some(start), Some(end)),
                _ => (None, None),
            },
            None => match range.parse::<u32>() {
                Ok(line) => (Some(line), Some(line)),
                Err(_) => (None, None),
            },
        };
        answers.push(CandidateAnswer {
            path,
            start_line,
            end_line,
            symbol: symbol.to_string(),
            kind: Some(kind.to_string()),
        });
    }
    answers
}

fn quoted(text: &str) -> Option<(&str, &str)> {
    let start = text.find('"')?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some((&rest[..end], &rest[end + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocbi_search_output_keeps_the_candidate_order_and_ranges() {
        let stdout = "\
Found 2 results for \"helper value\":

[1] function_definition \"text_helper_7\" in /repo/src/text_utils.py:7-8 (score: 0.96)
[2] function_definition \"helper_31\" in /repo/src/math_utils.py:30-31 (score: 0.44)
";
        let answers = parse_search_output(Path::new("/repo"), "repo", stdout);
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].path, "src/text_utils.py");
        assert_eq!(answers[0].start_line, Some(7));
        assert_eq!(answers[0].end_line, Some(8));
        assert_eq!(answers[1].symbol, "helper_31");
    }
}
