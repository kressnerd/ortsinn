//! Serena adapter over its documented MCP stdio server.
//!
//! Serena exposes a language-server symbol search and a regex text search, not
//! a natural-language retrieval operation. The adapter therefore records the
//! regex search as a separately labelled proxy arm and never claims the
//! natural-language family from documentation.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Result;
use serde_json::{Value, json};

use super::mcp::{McpClient, first_text};
use super::real::RealContext;
use super::session::{
    CandidateAnswer, CandidateSession, ExplainRequest, Operation, QueryRequest, SessionError,
};
use crate::adapters::ExplanationResponse;
use crate::manifest::{Capability, CapabilityStatus};

const REGEX_PROXY_ARM: &str = "regex_pattern_search_proxy";

#[derive(Debug)]
pub struct SerenaSession {
    context: RealContext,
    client: Option<McpClient>,
    client_repository: Option<String>,
    tools: Vec<String>,
}

impl SerenaSession {
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
                "start-mcp-server".to_string(),
                "--project".to_string(),
                checkout.display().to_string(),
                "--context".to_string(),
                "desktop-app".to_string(),
                "--agent-interface".to_string(),
                "tools".to_string(),
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

    fn regex_proxy(
        &mut self,
        repository: &str,
        query: &str,
    ) -> Result<Vec<CandidateAnswer>, SessionError> {
        let client = self.ensure_client(repository)?;
        let result = client
            .call(
                "search_for_pattern",
                json!({
                    "substring_pattern": query,
                    "max_answer_chars": 200_000,
                }),
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
        Ok(parse_pattern_output(&text))
    }
}

/// Serena's documented pattern output is a JSON object from file to matches:
/// `{"src/x.py": [{"line": 0, "text": "..."}]}`. Lines are zero-based.
fn parse_pattern_output(text: &str) -> Vec<CandidateAnswer> {
    let Some(start) = text.find('{') else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&text[start..]) else {
        return Vec::new();
    };
    let Some(files) = value.as_object() else {
        return Vec::new();
    };
    let mut answers = Vec::new();
    for (path, matches) in files {
        let Some(matches) = matches.as_array() else {
            continue;
        };
        for entry in matches {
            let Some(line) = entry.get("line").and_then(Value::as_u64) else {
                continue;
            };
            let line = line as u32 + 1;
            answers.push(CandidateAnswer {
                path: path.clone(),
                start_line: Some(line),
                end_line: Some(line),
                symbol: String::new(),
                kind: None,
            });
        }
    }
    answers
}

impl CandidateSession for SerenaSession {
    fn probe(&mut self) -> Result<(), SessionError> {
        let Some(repository) = self.context.repositories.first().map(|repo| repo.id.clone()) else {
            return Err(SessionError::new("no_fixed_input"));
        };
        self.ensure_client(&repository)?;
        if !self.tools.iter().any(|tool| tool == "search_for_pattern") {
            return Err(SessionError::with_detail(
                "probe_failed",
                "Serena did not declare search_for_pattern",
            ));
        }
        Ok(())
    }

    fn operations(&self) -> Vec<Operation> {
        Vec::new()
    }

    fn extra_capabilities(&self) -> BTreeMap<String, Capability> {
        let mut capabilities = BTreeMap::new();
        capabilities.insert(
            "natural_language".to_string(),
            Capability {
                status: CapabilityStatus::Unavailable,
                reason: Some("no_natural_language_search".to_string()),
            },
        );
        capabilities.insert(
            REGEX_PROXY_ARM.to_string(),
            Capability {
                status: CapabilityStatus::Measured,
                reason: None,
            },
        );
        for family in ["typo", "concept", "concept_equivalence", "expansion"] {
            capabilities.insert(
                family.to_string(),
                Capability {
                    status: CapabilityStatus::NotApplicable,
                    reason: Some(format!("no_{family}_operation")),
                },
            );
        }
        capabilities
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
        let deadline = self.context.deadline(repository);
        let outcome = super::real::run_command(
            &self.context.install(),
            &[
                "project".to_string(),
                "index".to_string(),
                checkout.display().to_string(),
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
                format!(
                    "serena project index exited {:?}: {}",
                    outcome.code,
                    outcome.detail()
                ),
            ));
        }
        Ok(())
    }

    fn query(&mut self, request: &QueryRequest) -> Result<Vec<CandidateAnswer>, SessionError> {
        if request.metric == "natural_language" && request.arm == REGEX_PROXY_ARM {
            return self.regex_proxy(&request.repository, &request.query);
        }
        Err(SessionError::new("no_expansion_switch"))
    }

    fn explain(&mut self, _request: &ExplainRequest) -> Result<ExplanationResponse, SessionError> {
        Err(SessionError::new("no_explanation_operation"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serena_pattern_output_is_file_and_line_resolution() {
        let text = "{\"src/math_utils.py\": [{\"line\": 0, \"text\": \"def helper_1\"}, {\"line\": 9, \"text\": \"def helper_10\"}]}";
        let answers = parse_pattern_output(text);
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].path, "src/math_utils.py");
        assert_eq!(answers[0].start_line, Some(1));
        assert_eq!(answers[1].start_line, Some(10));
        assert_eq!(answers[1].symbol, "");
    }
}
