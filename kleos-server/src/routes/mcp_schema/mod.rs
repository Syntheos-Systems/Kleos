//! MCP tool schema endpoint.
//!
//! GET /mcp/schema returns the registered tool definitions, auth-gated.
//! Dispatch is handled by the kleos-mcp sidecar process, which proxies
//! tool calls to kleos-server over HTTP with PIV signing.

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Public router merged outside the auth middleware stack.
pub fn public_router() -> Router<AppState> {
    Router::new()
}

/// Authenticated router merged inside the api_routes middleware stack.
pub fn router() -> Router<AppState> {
    Router::new().route("/mcp/schema", get(get_mcp_schema))
}

/// Returns every registered MCP tool definition as a JSON array.
async fn get_mcp_schema() -> Json<Value> {
    let tools = kleos_mcp::tools::registry();
    Json(json!({ "tools": tools }))
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
