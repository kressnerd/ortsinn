//! Canonical model-cassette key policy.
//!
//! The recorder, the replay check, and the fixture migration all resolve a
//! recorded model key against the current gold query. Resolving rather than
//! trusting the recorded key is what makes a changed gold query a missing
//! response instead of a silently reused one.

use anyhow::{Context, Result};

use super::cassette::{self, parse_model_key};
use crate::gold::Gold;

/// One model request the current gold set implies for a recorded result.
pub struct ResolvedModelRequest {
    /// `explanation` or `embedding`.
    pub layer: &'static str,
    /// Canonical key built from the current gold query.
    pub key: String,
    /// Current gold query the key digests.
    pub query: String,
}

/// Resolve a recorded model key for `(candidate, task)` against the gold query
/// of `(repository, metric, task)`. The digest segment of `recorded_key` is
/// deliberately ignored: it identifies the old request, not the current one.
pub fn resolve(
    gold: &Gold,
    repository: &str,
    metric: &str,
    candidate: &str,
    task: &str,
    recorded_key: &str,
) -> Result<ResolvedModelRequest> {
    let parsed = parse_model_key(recorded_key).with_context(|| {
        format!("model cassette key {recorded_key} for candidate {candidate} is not a canonical model key")
    })?;
    anyhow::ensure!(
        parsed.candidate == candidate,
        "model cassette key {recorded_key} names candidate {}, but the result names {candidate}",
        parsed.candidate
    );
    let query = gold_query(gold, repository, metric, task).with_context(|| {
        format!(
            "fixed-input mismatch: the result cassette names task {task} for repository {repository} and metric {metric}, which the gold sets do not define"
        )
    })?;
    let (layer, key) = match parsed.layer {
        "explanation" => {
            anyhow::ensure!(
                parsed.ident == task,
                "model cassette key {recorded_key} names task {}, but the result names {task}",
                parsed.ident
            );
            (
                "explanation",
                cassette::explanation_key(candidate, task, query),
            )
        }
        "embedding" => (
            "embedding",
            cassette::embedding_key(candidate, parsed.ident, query),
        ),
        other => anyhow::bail!("model cassette key {recorded_key} names unknown layer {other}"),
    };
    Ok(ResolvedModelRequest {
        layer,
        key,
        query: query.to_string(),
    })
}

/// The current gold query for one recorded task.
pub fn gold_query<'a>(
    gold: &'a Gold,
    repository: &str,
    metric: &str,
    task_id: &str,
) -> Option<&'a str> {
    match metric {
        "explanation" => gold
            .explanation
            .iter()
            .find(|task| task.repository == repository && task.id == task_id)
            .map(|task| task.query.as_str()),
        "typo" => gold
            .typo
            .iter()
            .find(|task| task.repository == repository && task.id == task_id)
            .map(|task| task.query.as_str()),
        "natural_language" => gold
            .natural_language
            .iter()
            .find(|task| task.repository == repository && task.id == task_id)
            .map(|task| task.query.as_str()),
        "concept" => gold
            .concept
            .iter()
            .find(|task| task.repository == repository && task.id == task_id)
            .map(|task| task.query.as_str()),
        "concept_equivalence" => gold
            .concept_equivalence
            .iter()
            .find(|task| task.repository == repository && task.id == task_id)
            .map(|task| task.query.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::NormalizedLocation;
    use crate::gold::{ExplanationTask, Gold, LineRange};

    fn gold_with_query(query: &str) -> Gold {
        Gold {
            typo: Vec::new(),
            natural_language: Vec::new(),
            explanation: vec![ExplanationTask {
                id: "exp-1".to_string(),
                query: query.to_string(),
                target: NormalizedLocation {
                    path: "src/math_utils.py".to_string(),
                    start_line: 10,
                    end_line: 10,
                    symbol: "helper_10".to_string(),
                    kind: None,
                    resolution: None,
                },
                required_facts: vec![vec!["computes".to_string()]],
                acceptable_evidence: vec![LineRange {
                    start_line: 10,
                    end_line: 10,
                }],
                repository: "fixture-repo".to_string(),
            }],
            concept: Vec::new(),
            concept_equivalence: Vec::new(),
            sets: Vec::new(),
        }
    }

    #[test]
    fn s9_changed_query_is_a_distinct_key_and_never_reuses_the_recorded_response() {
        let original = cassette::explanation_key("fixture-ckb", "exp-1", "Explain helper_10");
        let gold = gold_with_query("Explain helper_10");
        let unchanged = resolve(
            &gold,
            "fixture-repo",
            "explanation",
            "fixture-ckb",
            "exp-1",
            &original,
        )
        .expect("resolve the unchanged query");
        assert_eq!(unchanged.key, original);
        assert_eq!(unchanged.query, "Explain helper_10");

        let changed_gold = gold_with_query("Explain helper_10 in detail");
        let changed = resolve(
            &changed_gold,
            "fixture-repo",
            "explanation",
            "fixture-ckb",
            "exp-1",
            &original,
        )
        .expect("resolve the changed query");
        assert_ne!(changed.key, original);
        assert!(changed.key.ends_with(&cassette::query_digest("Explain helper_10 in detail")));
        assert_eq!(changed.query, "Explain helper_10 in detail");

        // A legacy, query-free key migrates to the same canonical key; the
        // recorded digest segment is never trusted.
        let legacy = resolve(
            &gold,
            "fixture-repo",
            "explanation",
            "fixture-ckb",
            "exp-1",
            "model:explanation:fixture-ckb:exp-1",
        )
        .expect("resolve a legacy key");
        assert_eq!(legacy.key, original);
    }

    #[test]
    fn s9_query_digest_uses_the_normalized_query_text() {
        assert_eq!(
            cassette::query_digest("  Explain   helper_10\n"),
            cassette::query_digest("Explain helper_10")
        );
        assert_ne!(
            cassette::query_digest("Explain helper_10"),
            cassette::query_digest("Explain helper_11")
        );
    }
}
