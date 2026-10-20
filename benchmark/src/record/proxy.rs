//! Local OpenAI-compatible endpoint fixture and the recording client.
//!
//! The endpoint binds to loopback on an ephemeral port, so the record-mode
//! E2E never depends on a third-party service. It serves deterministic
//! fixture completions and embeddings and keeps the request log the E2E uses
//! to prove that no request left the local boundary.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::post,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;

use super::cassette::ModelRecord;

/// One fixture explanation served for a chat-completion query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureCompletion {
    pub text: String,
    pub citations: Vec<FixtureCitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureCitation {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// One request the local endpoint accepted. The peer address is recorded so
/// the E2E can assert every request stayed on loopback.
#[derive(Debug, Clone)]
pub struct EndpointRequest {
    pub path: String,
    pub input: String,
    pub peer: SocketAddr,
}

#[derive(Default)]
struct EndpointState {
    completions: BTreeMap<String, FixtureCompletion>,
    requests: Mutex<Vec<EndpointRequest>>,
    counter: Mutex<u64>,
}

impl EndpointState {
    fn record(&self, path: &str, input: &str, peer: SocketAddr) {
        self.requests
            .lock()
            .expect("endpoint request log")
            .push(EndpointRequest {
                path: path.to_string(),
                input: input.to_string(),
                peer,
            });
    }

    fn next_id(&self) -> u64 {
        let mut counter = self.counter.lock().expect("endpoint request counter");
        *counter += 1;
        *counter
    }
}

/// A deterministic loopback OpenAI-compatible endpoint.
pub struct FakeOpenAiEndpoint {
    addr: SocketAddr,
    state: Arc<EndpointState>,
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for FakeOpenAiEndpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FakeOpenAiEndpoint")
            .field("addr", &self.addr)
            .finish_non_exhaustive()
    }
}

impl FakeOpenAiEndpoint {
    /// Start the fixture on `127.0.0.1:0`. `completions` maps the exact user
    /// query to the explanation the fixture returns.
    pub fn start(completions: BTreeMap<String, FixtureCompletion>) -> std::io::Result<Self> {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?;
        let state = Arc::new(EndpointState {
            completions,
            requests: Mutex::new(Vec::new()),
            counter: Mutex::new(0),
        });
        let server_state = Arc::clone(&state);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let thread = std::thread::Builder::new()
            .name("fake-openai-endpoint".to_string())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("fake endpoint runtime");
                runtime.block_on(async move {
                    let listener =
                        tokio::net::TcpListener::from_std(listener).expect("adopt listener");
                    let app = Router::new()
                        .route("/v1/chat/completions", post(chat_completion))
                        .route("/v1/embeddings", post(embeddings))
                        .with_state(server_state);
                    let _ = axum::serve(
                        listener,
                        app.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .with_graceful_shutdown(async move {
                        let _ = shutdown_rx.await;
                    })
                    .await;
                });
            })?;
        Ok(Self {
            addr,
            state,
            shutdown: Some(shutdown_tx),
            thread: Some(thread),
        })
    }

    /// Base URL in the shape the `benchmark` CLI expects.
    pub fn base_url(&self) -> String {
        format!("http://{}/v1", self.addr)
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn requests(&self) -> Vec<EndpointRequest> {
        self.state
            .requests
            .lock()
            .expect("endpoint request log")
            .clone()
    }

    pub fn request_count(&self) -> usize {
        self.state
            .requests
            .lock()
            .expect("endpoint request log")
            .len()
    }
}

impl Drop for FakeOpenAiEndpoint {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

async fn chat_completion(
    State(state): State<Arc<EndpointState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let query = last_message_content(&request);
    state.record("/v1/chat/completions", &query, peer);
    let Some(completion) = state.completions.get(&query) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"message": format!("no fixture completion for query {query}")}})),
        );
    };
    let content = serde_json::to_string(completion).expect("serialize fixture completion");
    (
        StatusCode::OK,
        Json(json!({
            "id": format!("fixture-completion-{}", state.next_id()),
            "object": "chat.completion",
            "model": request.get("model").cloned().unwrap_or(json!("fixture-completion")),
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": content},
                "finish_reason": "stop",
            }],
        })),
    )
}

async fn embeddings(
    State(state): State<Arc<EndpointState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let input = request
        .get("input")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    state.record("/v1/embeddings", &input, peer);
    let digest = Sha256::digest(input.as_bytes());
    let embedding: Vec<f64> = digest[..8]
        .iter()
        .map(|byte| f64::from(*byte) / 255.0)
        .collect();
    (
        StatusCode::OK,
        Json(json!({
            "object": "list",
            "model": request.get("model").cloned().unwrap_or(json!("fixture-embedding")),
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": embedding,
            }],
        })),
    )
}

fn last_message_content(request: &Value) -> String {
    request
        .get("messages")
        .and_then(Value::as_array)
        .and_then(|messages| messages.last())
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// A recording client bound to a loopback OpenAI-compatible endpoint.
///
/// The constructor rejects a non-loopback endpoint, so record mode cannot
/// silently send model traffic outside the local recording boundary.
#[derive(Debug)]
pub struct ModelRecorder {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl ModelRecorder {
    pub fn new(endpoint: &str) -> Result<Self> {
        ensure_loopback(endpoint)?;
        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Send one explanation request and normalize the recorded response.
    pub fn explanation(&self, candidate: &str, task: &str, query: &str) -> Result<ModelRecord> {
        let body = json!({
            "model": "fixture-completion",
            "messages": [{"role": "user", "content": query}],
            "temperature": 0,
        });
        let value = self.post("chat/completions", &body)?;
        let content = value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .context("fixture completion has no message content")?;
        let response: Value = serde_json::from_str(content)
            .with_context(|| format!("parse recorded completion for {candidate}/{task}"))?;
        Ok(ModelRecord {
            key: format!("model:explanation:{candidate}:{task}"),
            candidate: candidate.to_string(),
            task: task.to_string(),
            query: query.to_string(),
            layer: "explanation".to_string(),
            response,
        })
    }

    /// Send one embedding request and record the returned vector.
    pub fn embedding(&self, candidate: &str, model: &str) -> Result<ModelRecord> {
        let input = format!("{candidate} fixture embedding");
        let body = json!({"model": model, "input": input});
        let value = self.post("embeddings", &body)?;
        let embedding = value
            .pointer("/data/0/embedding")
            .cloned()
            .context("fixture embedding response has no vector")?;
        Ok(ModelRecord {
            key: format!("model:embedding:{candidate}:{model}"),
            candidate: candidate.to_string(),
            task: "embedding".to_string(),
            query: input,
            layer: "embedding".to_string(),
            response: json!({"model": model, "embedding": embedding}),
        })
    }

    fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}/{path}", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .with_context(|| format!("record model request to {url}"))?;
        let status = response.status();
        let text = response
            .text()
            .with_context(|| format!("read fixture response from {url}"))?;
        if !status.is_success() {
            bail!("fixture endpoint {url} returned {status}: {text}");
        }
        serde_json::from_str(&text).with_context(|| format!("parse fixture response from {url}"))
    }
}

/// Reject a model endpoint that is not loopback. Record mode is only allowed
/// to talk to the local recording boundary.
pub fn ensure_loopback(endpoint: &str) -> Result<()> {
    let url = reqwest::Url::parse(endpoint)
        .with_context(|| format!("parse model endpoint {endpoint}"))?;
    let Some(host) = url.host_str() else {
        bail!("model endpoint {endpoint} has no host");
    };
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    if !loopback {
        bail!("model endpoint {endpoint} is not loopback; record mode is local-only");
    }
    Ok(())
}

/// True when the peer address is loopback.
pub fn is_loopback(peer: SocketAddr) -> bool {
    matches!(peer.ip(), IpAddr::V4(address) if address.is_loopback())
        || matches!(peer.ip(), IpAddr::V6(address) if address.is_loopback())
}
