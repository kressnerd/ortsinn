//! A synchronous MCP stdio client over `rmcp`.
//!
//! The candidate adapters are synchronous, so the client owns a Tokio runtime
//! and blocks on every request. The runtime lives as long as the session; the
//! child process is killed when the service is dropped.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};

pub struct McpClient {
    runtime: tokio::runtime::Runtime,
    service: RunningService<RoleClient, ()>,
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("McpClient")
    }
}

impl McpClient {
    /// Start one candidate MCP server over stdio.
    pub fn spawn(program: &Path, args: &[String], cwd: &Path) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("build the MCP client runtime")?;
        let program = program.to_path_buf();
        let args = args.to_vec();
        let cwd = cwd.to_path_buf();
        let service = runtime
            .block_on(async move {
                let mut command = tokio::process::Command::new(&program);
                command.args(&args).current_dir(&cwd);
                let transport = TokioChildProcess::new(command).map_err(|error| {
                    anyhow!("spawn MCP server {}: {error}", program.display())
                })?;
                ().serve(transport)
                    .await
                    .map_err(|error| anyhow!("initialize MCP server {}: {error}", program.display()))
            })?;
        Ok(Self { runtime, service })
    }

    /// The tool names the server declares after initialization.
    pub fn tool_names(&self) -> Result<Vec<String>> {
        let tools = self
            .runtime
            .block_on(async { self.service.list_all_tools().await })
            .map_err(|error| anyhow!("list MCP tools: {error}"))?;
        Ok(tools
            .into_iter()
            .map(|tool| tool.name.into_owned())
            .collect())
    }

    /// Call one declared tool. The result is serialized so adapters can read
    /// the text content without depending on the transport types.
    pub fn call(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let arguments = arguments
            .as_object()
            .cloned()
            .unwrap_or_default();
        let params = CallToolRequestParams::new(name.to_string()).with_arguments(arguments);
        let result = self
            .runtime
            .block_on(async { self.service.call_tool(params).await })
            .map_err(|error| anyhow!("call MCP tool {name}: {error}"))?;
        serde_json::to_value(result).context("serialize MCP tool result")
    }
}

/// Read the first text block out of a serialized `CallToolResult`.
pub fn first_text(result: &serde_json::Value) -> Result<String> {
    result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .and_then(|content| {
            content
                .iter()
                .find_map(|item| item.get("text").and_then(serde_json::Value::as_str))
        })
        .map(str::to_string)
        .ok_or_else(|| anyhow!("MCP tool result has no text content"))
}
