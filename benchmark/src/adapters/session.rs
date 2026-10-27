//! The candidate session boundary: one trait over each candidate's documented
//! interface, plus the normalized-result mapping the adapters share.
//!
//! The harness asks a session what the candidate actually exposes. It never
//! infers a capability from documentation, and it never turns a file-only
//! answer into a symbol resolution.

use std::collections::BTreeMap;
use std::fmt;

use crate::adapters::{dedup_locations, ExplanationResponse, NormalizedLocation};

/// A documented operation a candidate may expose. The capability map is
/// derived from this list, never from a README claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Typo,
    NaturalLanguage,
    Concept,
    ConceptEquivalence,
    Explanation,
    Expansion,
}

impl Operation {
    /// Every operation the harness can measure.
    pub const ALL: [Operation; 6] = [
        Operation::Typo,
        Operation::NaturalLanguage,
        Operation::Concept,
        Operation::ConceptEquivalence,
        Operation::Explanation,
        Operation::Expansion,
    ];

    /// The manifest capability this operation measures.
    pub fn capability(self) -> &'static str {
        match self {
            Operation::Typo => "typo",
            Operation::NaturalLanguage => "natural_language",
            Operation::Concept => "concept",
            Operation::ConceptEquivalence => "concept_equivalence",
            Operation::Explanation => "explanation",
            Operation::Expansion => "expansion",
        }
    }
}

/// One answer from the candidate's documented query operation, before the
/// harness normalizes it. A file-only answer has no line range and must stay a
/// file resolution; it is never promoted to a symbol match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateAnswer {
    pub path: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub symbol: String,
    pub kind: Option<String>,
}

impl CandidateAnswer {
    /// A symbol answer with a line range.
    pub fn symbol(
        path: impl Into<String>,
        start_line: u32,
        end_line: u32,
        symbol: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            start_line: Some(start_line),
            end_line: Some(end_line),
            symbol: symbol.into(),
            kind: None,
        }
    }

    /// A file-only answer. It carries no line range and no symbol, so it can
    /// only ever match at file resolution.
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            start_line: None,
            end_line: None,
            symbol: String::new(),
            kind: None,
        }
    }

    /// Normalize one answer. A complete line range becomes a symbol
    /// resolution; anything else stays a file resolution (the normalized
    /// record uses `0` lines to mean "no range").
    pub fn normalize(&self) -> NormalizedLocation {
        match (self.start_line, self.end_line) {
            (Some(start_line), Some(end_line)) => NormalizedLocation {
                path: self.path.clone(),
                start_line,
                end_line,
                symbol: self.symbol.clone(),
                kind: self.kind.clone(),
                resolution: Some("symbol".to_string()),
            },
            _ => NormalizedLocation {
                path: self.path.clone(),
                start_line: 0,
                end_line: 0,
                symbol: String::new(),
                kind: self.kind.clone(),
                resolution: Some("file".to_string()),
            },
        }
    }
}

/// Normalize one candidate answer list: preserve the candidate's order,
/// deduplicate by `(path, start_line, end_line, symbol)` without reordering,
/// and keep a file-only answer as a file resolution.
pub fn normalize_answers(answers: &[CandidateAnswer]) -> Vec<NormalizedLocation> {
    let locations: Vec<NormalizedLocation> = answers.iter().map(CandidateAnswer::normalize).collect();
    dedup_locations(&locations)
}

/// One normalized query request against the candidate's documented interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRequest {
    pub repository: String,
    pub metric: String,
    pub task: String,
    pub query: String,
}

/// One explanation request against the candidate's documented interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainRequest {
    pub repository: String,
    pub task: String,
    pub query: String,
}

/// A typed session failure. The reason is the closed label the report records;
/// the detail is human context and never replaces the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionError {
    pub reason: String,
    pub detail: Option<String>,
}

impl SessionError {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            detail: None,
        }
    }

    pub fn with_detail(reason: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            detail: Some(detail.into()),
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(formatter, "{}: {detail}", self.reason),
            None => formatter.write_str(&self.reason),
        }
    }
}

impl std::error::Error for SessionError {}

/// One session over a candidate's documented interface.
///
/// The real implementation is the candidate's MCP/CLI session; `FakeSession`
/// is the test double used by the executable specs. Both cross the same
/// normalization boundary, so the report contract does not fork per candidate.
pub trait CandidateSession {
    /// Probe the documented interface before any index or query. A failing
    /// probe marks the candidate unavailable; it never produces zero cells.
    fn probe(&mut self) -> Result<(), SessionError>;

    /// The operations the candidate actually exposes.
    fn operations(&self) -> Vec<Operation>;

    /// The installed setup, including any documented setup deviation.
    fn setup(&self) -> crate::manifest::CandidateSetup {
        crate::manifest::CandidateSetup::default()
    }

    /// Index one fixed repository. A candidate without a working index fails
    /// with its typed reason and the run continues.
    fn index(&mut self, _repository: &str) -> Result<(), SessionError> {
        Ok(())
    }

    /// Run one documented query operation.
    fn query(&mut self, request: &QueryRequest) -> Result<Vec<CandidateAnswer>, SessionError>;

    /// Run one documented explanation operation.
    fn explain(&mut self, request: &ExplainRequest) -> Result<ExplanationResponse, SessionError>;
}

/// Test double for the candidate session. It drives the adapter mapping
/// without starting a candidate process.
#[derive(Debug, Clone)]
pub struct FakeSession {
    candidate: String,
    operations: Vec<Operation>,
    setup: crate::manifest::CandidateSetup,
    probe_result: Result<(), SessionError>,
    answers: BTreeMap<String, Vec<CandidateAnswer>>,
    explanations: BTreeMap<String, ExplanationResponse>,
}

impl FakeSession {
    /// A session that probes successfully and exposes exactly `operations`.
    pub fn exposing(candidate: &str, operations: Vec<Operation>) -> Self {
        Self {
            candidate: candidate.to_string(),
            operations,
            setup: crate::manifest::CandidateSetup::default(),
            probe_result: Ok(()),
            answers: BTreeMap::new(),
            explanations: BTreeMap::new(),
        }
    }

    /// A session whose documented interface cannot be started.
    pub fn failing_probe(candidate: &str, reason: &str, detail: &str) -> Self {
        Self {
            candidate: candidate.to_string(),
            operations: Vec::new(),
            setup: crate::manifest::CandidateSetup::default(),
            probe_result: Err(SessionError::with_detail(reason, detail)),
            answers: BTreeMap::new(),
            explanations: BTreeMap::new(),
        }
    }

    pub fn with_setup(mut self, setup: crate::manifest::CandidateSetup) -> Self {
        self.setup = setup;
        self
    }

    pub fn answering(mut self, task: &str, answers: Vec<CandidateAnswer>) -> Self {
        self.answers.insert(task.to_string(), answers);
        self
    }

    pub fn explaining(mut self, task: &str, response: ExplanationResponse) -> Self {
        self.explanations.insert(task.to_string(), response);
        self
    }

    pub fn candidate(&self) -> &str {
        &self.candidate
    }
}

impl CandidateSession for FakeSession {
    fn probe(&mut self) -> Result<(), SessionError> {
        self.probe_result.clone()
    }

    fn operations(&self) -> Vec<Operation> {
        self.operations.clone()
    }

    fn setup(&self) -> crate::manifest::CandidateSetup {
        self.setup.clone()
    }

    fn query(&mut self, request: &QueryRequest) -> Result<Vec<CandidateAnswer>, SessionError> {
        self.answers.get(&request.task).cloned().ok_or_else(|| {
            SessionError::with_detail(
                "no_fake_answer",
                format!(
                    "candidate {} has no fake answer for task {}",
                    self.candidate, request.task
                ),
            )
        })
    }

    fn explain(&mut self, request: &ExplainRequest) -> Result<ExplanationResponse, SessionError> {
        self.explanations.get(&request.task).cloned().ok_or_else(|| {
            SessionError::with_detail(
                "no_fake_explanation",
                format!(
                    "candidate {} has no fake explanation for task {}",
                    self.candidate, request.task
                ),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::Citation;

    /// S13: the normalized results keep the candidate's order, deduplicate
    /// without reordering, and keep a file-only answer at file resolution.
    #[test]
    fn s13_normalization_preserves_order_and_file_resolution() {
        let answers = vec![
            CandidateAnswer::symbol("src/a.py", 10, 10, "helper_10"),
            CandidateAnswer::symbol("src/a.py", 10, 10, "helper_10"),
            CandidateAnswer::file("src/b.py"),
            CandidateAnswer::symbol("src/c.py", 30, 30, "helper_30"),
        ];
        let normalized = normalize_answers(&answers);
        assert_eq!(
            normalized
                .iter()
                .map(|location| location.path.as_str())
                .collect::<Vec<_>>(),
            ["src/a.py", "src/b.py", "src/c.py"]
        );
        assert_eq!(normalized[0].symbol, "helper_10");
        assert_eq!(normalized[0].resolution.as_deref(), Some("symbol"));
        assert_eq!(normalized[1].resolution.as_deref(), Some("file"));
        assert_eq!(normalized[1].start_line, 0);
        assert_eq!(normalized[1].end_line, 0);
        assert!(normalized[1].symbol.is_empty());
        assert_eq!(normalized[2].symbol, "helper_30");
    }

    /// The fake session returns the candidate's answers unchanged; the
    /// normalization boundary is what preserves order and resolution.
    #[test]
    fn fake_session_returns_documented_answers() {
        let mut session = FakeSession::exposing("fixture-ocbi", vec![Operation::Typo]).answering(
            "typo-1",
            vec![CandidateAnswer::file("src/a.py")],
        );
        let answers = session
            .query(&QueryRequest {
                repository: "fixture-repo".to_string(),
                metric: "typo".to_string(),
                task: "typo-1".to_string(),
                query: "helper_1".to_string(),
            })
            .expect("fake answer");
        assert_eq!(answers.len(), 1);
        let explanation = session.explain(&ExplainRequest {
            repository: "fixture-repo".to_string(),
            task: "exp-1".to_string(),
            query: "explain".to_string(),
        });
        assert_eq!(
            explanation.expect_err("no explanation registered").reason,
            "no_fake_explanation"
        );
        let response = ExplanationResponse {
            text: "text".to_string(),
            citations: vec![Citation {
                path: "src/a.py".to_string(),
                start_line: 1,
                end_line: 1,
            }],
        };
        let mut session = session.explaining("exp-1", response);
        assert!(
            session
                .explain(&ExplainRequest {
                    repository: "fixture-repo".to_string(),
                    task: "exp-1".to_string(),
                    query: "explain".to_string(),
                })
                .is_ok()
        );
    }
}
