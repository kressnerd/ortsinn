//! Proves the local E2E fixture endpoint is reachable over loopback.
//!
//! The `@e2e` record/replay scenario starts the same endpoint; this test is
//! the setup proof that the runner can reach it and that every request is
//! recorded on the local boundary.

use std::collections::BTreeMap;

use benchmark::record::proxy::{
    FakeOpenAiEndpoint, FixtureCitation, FixtureCompletion, is_loopback,
};
use serde_json::{Value, json};

fn completion() -> FixtureCompletion {
    FixtureCompletion {
        text: "This computes the value.".to_string(),
        citations: vec![FixtureCitation {
            path: "src/math_utils.py".to_string(),
            start_line: 10,
            end_line: 10,
        }],
    }
}

#[test]
fn the_e2e_runner_reaches_the_local_fixture_endpoint() {
    let mut completions = BTreeMap::new();
    completions.insert("Explain helper_10".to_string(), completion());
    let endpoint = FakeOpenAiEndpoint::start(completions).expect("start fixture endpoint");
    let base = endpoint.base_url();
    assert!(
        base.starts_with("http://127.0.0.1:"),
        "fixture endpoint is loopback: {base}"
    );

    let client = reqwest::blocking::Client::new();
    let chat: Value = client
        .post(format!("{base}/chat/completions"))
        .json(&json!({
            "model": "fixture-completion",
            "messages": [{"role": "user", "content": "Explain helper_10"}],
        }))
        .send()
        .expect("reach fixture endpoint")
        .json()
        .expect("parse fixture completion");
    let content = chat
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .expect("fixture completion content");
    let normalized: Value = serde_json::from_str(content).expect("normalize fixture completion");
    assert_eq!(normalized["text"], "This computes the value.");

    let embedding: Value = client
        .post(format!("{base}/embeddings"))
        .json(&json!({"model": "nomic-embed-text", "input": "fixture"}))
        .send()
        .expect("reach fixture endpoint")
        .json()
        .expect("parse fixture embedding");
    assert_eq!(
        embedding
            .pointer("/data/0/embedding")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(8)
    );

    let requests = endpoint.requests();
    assert_eq!(requests.len(), 2, "both requests reached the endpoint");
    assert_eq!(requests[0].path, "/v1/chat/completions");
    assert_eq!(requests[1].path, "/v1/embeddings");
    assert!(
        requests.iter().all(|request| is_loopback(request.peer)),
        "every request stayed on loopback: {requests:?}"
    );
}
