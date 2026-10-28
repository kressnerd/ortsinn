//! CKB adapter over its documented MCP stdio server.
//!
//! The adapter asks the server for its declared tools and maps only those:
//! `searchSymbols` for retrieval and `explainSymbol`/`explainPath` for the
//! explanation capability. No capability is inferred from documentation.

use std::time::Duration;

use anyhow::Result;
use serde_json::{Value, json};

use super::mcp::{McpClient, first_text};
use super::real::{RealContext, language_selector, run_command};
use super::session::{
    CandidateAnswer, CandidateSession, ExplainRequest, Operation, QueryRequest, SessionError,
};
use crate::adapters::ExplanationResponse;

const RESULT_LIMIT: u32 = 10;

#[derive(Debug)]
pub struct CkbSession {
    context: RealContext,
    client: Option<McpClient>,
    client_repository: Option<String>,
    tools: Vec<String>,
}

impl CkbSession {
    pub fn new(context: RealContext) -> Self {
        Self {
            context,
            client: None,
            client_repository: None,
            tools: Vec::new(),
        }
    }

    fn ensure_client(&mut self, repository: &str) -> Result<&McpClient, SessionError> {
        if self.client_repository.as_deref() != Some(repository) {
            let checkout = self
                .context
                .checkout(repository)
                .map_err(|error| SessionError::with_detail("missing_checkout", error.to_string()))?;
            let args = vec![
                "mcp".to_string(),
                "--stdio".to_string(),
                "--repo".to_string(),
                checkout.display().to_string(),
            ];
            let client = McpClient::spawn(&self.context.install(), &args, &checkout)
                .map_err(|error| SessionError::with_detail("session_failed", error.to_string()))?;
            self.tools = client
                .tool_names()
                .map_err(|error| SessionError::with_detail("session_failed", error.to_string()))?;
            self.client = Some(client);
            self.client_repository = Some(repository.to_string());
        }
        self.client
            .as_ref()
            .ok_or_else(|| SessionError::new("session_failed"))
    }

    fn search(
        &mut self,
        repository: &str,
        query: &str,
    ) -> Result<Vec<CandidateAnswer>, SessionError> {
        let client = self.ensure_client(repository)?;
        let result = client
            .call(
                "searchSymbols",
                json!({"query": query, "limit": RESULT_LIMIT}),
            )
            .map_err(|error| SessionError::with_detail("query_failed", error.to_string()))?;
        if result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(SessionError::with_detail(
                "query_failed",
                first_text(&result).unwrap_or_else(|_| "tool reported an error".to_string()),
            ));
        }
        let text = first_text(&result)
            .map_err(|error| SessionError::with_detail("query_failed", error.to_string()))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|error| SessionError::with_detail("query_failed", error.to_string()))?;
        let symbols = value
            .pointer("/data/symbols")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let checkout = self
            .context
            .checkout(repository)
            .map_err(|error| SessionError::with_detail("missing_checkout", error.to_string()))?;
        Ok(symbols
            .iter()
            .filter_map(|symbol| candidate_answer(&checkout, symbol))
            .collect())
    }
}

fn candidate_answer(checkout: &std::path::Path, symbol: &Value) -> Option<CandidateAnswer> {
    let location = symbol.get("location")?;
    let raw_path = location
        .get("fileId")
        .or_else(|| location.get("path"))
        .and_then(Value::as_str)?;
    let path = std::path::Path::new(raw_path)
        .strip_prefix(checkout)
        .unwrap_or(std::path::Path::new(raw_path))
        .to_string_lossy()
        .replace('\\', "/");
    let start = location.get("startLine").and_then(Value::as_u64)? as u32;
    let end = location
        .get("endLine")
        .and_then(Value::as_u64)
        .map(|line| line as u32)
        .unwrap_or(start);
    Some(CandidateAnswer {
        path,
        start_line: Some(start),
        end_line: Some(end),
        symbol: symbol
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        kind: symbol
            .get("kind")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ckb_search_symbols_keeps_the_declared_order_and_range() {
        let symbols: Vec<Value> = serde_json::from_str(
            r#"[
                {"name":"helper_55","kind":"function","location":{"fileId":"src/math_utils.py","startLine":55,"endLine":55}},
                {"name":"helper_14","kind":"function","location":{"fileId":"src/math_utils.py","startLine":14,"endLine":14}}
            ]"#,
        )
        .expect("parse symbols");
        let answers: Vec<CandidateAnswer> = symbols
            .iter()
            .filter_map(|symbol| candidate_answer(std::path::Path::new("/repo"), symbol))
            .collect();
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].symbol, "helper_55");
        assert_eq!(answers[0].start_line, Some(55));
        assert_eq!(answers[1].symbol, "helper_14");
    }
}

impl CandidateSession for CkbSession {
    fn probe(&mut self) -> Result<(), SessionError> {
        let Some(repository) = self.context.repositories.first().map(|repo| repo.id.clone()) else {
            return Err(SessionError::new("no_fixed_input"));
        };
        self.ensure_client(&repository)?;
        Ok(())
    }

    fn operations(&self) -> Vec<Operation> {
        let mut operations = Vec::new();
        if self.tools.iter().any(|tool| tool == "searchSymbols") {
            operations.extend([
                Operation::Typo,
                Operation::NaturalLanguage,
                Operation::Concept,
                Operation::ConceptEquivalence,
            ]);
        }
        if self
            .tools
            .iter()
            .any(|tool| tool == "explainSymbol" || tool == "explainPath")
        {
            operations.push(Operation::Explanation);
        }
        operations
    }

    fn setup(&self) -> crate::manifest::CandidateSetup {
        let mut setup = self.context.candidate.setup.clone();
        setup.environment.insert(
            "mcp_transport".to_string(),
            "stdio (rmcp)".to_string(),
        );
        setup
    }

    fn index(&mut self, repository: &str) -> Result<(), SessionError> {
        let checkout = self
            .context
            .checkout(repository)
            .map_err(|error| SessionError::with_detail("missing_checkout", error.to_string()))?;
        let Some(language) = self
            .context
            .repository(repository)
            .ok()
            .and_then(|repository| language_selector(&repository.language))
        else {
            return Err(SessionError::with_detail(
                "index_unsupported_language",
                format!(
                    "CKB has no documented indexer for repository {repository}"
                ),
            ));
        };
        let init = run_command(
            &self.context.install(),
            &["init".to_string()],
            &checkout,
            Some(Duration::from_secs(120)),
        )
        .map_err(|error| SessionError::with_detail("index_failed", error.to_string()))?;
        if !init.succeeded() {
            return Err(SessionError::with_detail(
                "index_failed",
                format!("ckb init exited {:?}: {}", init.code, init.detail()),
            ));
        }
        let deadline = self.context.deadline(repository);
        let outcome = run_command(
            &self.context.install(),
            &["index".to_string(), "--lang".to_string(), language.to_string()],
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
                format!("ckb index exited {:?}: {}", outcome.code, outcome.detail()),
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
        Err(SessionError::new("external_explanation_model"))
    }
}

