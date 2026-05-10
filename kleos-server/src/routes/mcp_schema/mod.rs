//! MCP tool schema and dispatch endpoints.
//!
//! Both routes live behind the auth middleware stack:
//! - GET  /mcp/schema   -- returns the registered tool definitions (read-only but auth-gated)
//! - POST /mcp/dispatch -- dispatches a tool call; enforces per-tool scope requirements

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::Auth;
use crate::state::AppState;

/// Public router merged outside the auth middleware stack.
pub fn public_router() -> Router<AppState> {
    Router::new()
}

/// Authenticated router merged inside the api_routes middleware stack.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mcp/schema", get(get_mcp_schema))
        .route("/mcp/dispatch", post(dispatch_handler))
}

/// Returns every registered MCP tool definition as a JSON array.
async fn get_mcp_schema() -> Json<Value> {
    let tools = kleos_mcp::tools::registry();
    Json(json!({ "tools": tools }))
}

/// Accepted fields for POST /mcp/dispatch.
#[derive(Deserialize)]
struct DispatchBody {
    /// MCP tool name (e.g. "memory.store", "structural.analyze").
    name: String,
    /// Tool-specific arguments as a JSON object.
    #[serde(default)]
    arguments: Value,
}

/// Dispatches a single MCP tool call through kleos_mcp::tools::dispatch.
/// Constructs a kleos_mcp::App from the server's shared state so
/// structural tools (which need in-process database access) work
/// identically to the local MCP server.
async fn dispatch_handler(
    Auth(auth): Auth,
    State(state): State<AppState>,
    Json(body): Json<DispatchBody>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let scope = kleos_mcp::tools::required_scope(&body.name);
    if !auth.has_scope(&scope) {
        return Err(AppError(kleos_lib::EngError::Auth(format!(
            "tool '{}' requires {:?} scope",
            body.name, scope
        ))));
    }

    let app = kleos_mcp::App {
        db: Arc::clone(&state.db),
        config: Arc::clone(&state.config),
        llm: state.llm.clone(),
    };

    match kleos_mcp::tools::dispatch(&app, &body.name, body.arguments).await {
        Ok(result) => Ok((StatusCode::OK, Json(result))),
        Err(e) => Err(AppError(e)),
    }
}

#[cfg(test)]
/// Unit tests for the MCP schema and dispatch endpoints.
mod tests {
    use super::*;

    /// Verify the schema handler returns a non-empty tools array with
    /// the three required MCP fields on each entry.
    #[tokio::test]
    async fn mcp_schema_returns_tool_definitions() {
        let Json(body) = get_mcp_schema().await;
        let tools = body["tools"].as_array().expect("tools should be an array");
        assert!(
            !tools.is_empty(),
            "registry should contain at least one tool"
        );

        for tool in tools {
            assert!(tool["name"].is_string(), "tool missing name");
            assert!(tool["description"].is_string(), "tool missing description");
            assert!(tool["inputSchema"].is_object(), "tool missing inputSchema");
        }
    }
}
