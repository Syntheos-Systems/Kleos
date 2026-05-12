//! MCP tool registry and dispatcher.
//!
//! All tools are derived from `kleos_client::ROUTES`. `registry()` emits one
//! `tools/list` entry per canonical name plus each alias; `dispatch()` looks
//! up the route and forwards the call through `app.client.call_route(...)`.

use crate::App;
use kleos_client::{find_by_name, ROUTES};
use serde_json::{json, Value};

/// Returns the full tool registry as JSON objects suitable for an MCP
/// `tools/list` response. Each route yields one entry per canonical name
/// plus one entry per alias (back-compat).
pub fn registry() -> Vec<Value> {
    let mut out = Vec::with_capacity(ROUTES.len() * 2);
    for route in ROUTES {
        let schema: Value = serde_json::from_str(route.input_schema).unwrap_or_else(|_| {
            json!({ "type": "object", "additionalProperties": true })
        });
        out.push(json!({
            "name": route.name,
            "description": route.description,
            "inputSchema": schema,
        }));
        for alias in route.aliases {
            let schema_clone: Value = serde_json::from_str(route.input_schema).unwrap_or_else(
                |_| json!({ "type": "object", "additionalProperties": true }),
            );
            out.push(json!({
                "name": alias,
                "description": route.description,
                "inputSchema": schema_clone,
            }));
        }
    }
    out
}

/// Routes an MCP tool call to the registered HTTP route. The arguments are
/// passed straight through; path templates extract the relevant fields.
#[tracing::instrument(skip(app, args), fields(name = %name))]
pub async fn dispatch(app: &App, name: &str, args: Value) -> Result<Value, String> {
    let route = find_by_name(name).ok_or_else(|| format!("unknown tool: {name}"))?;
    app.client.call_route(route, args).await
}
