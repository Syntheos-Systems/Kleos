//! kleos-mcp -- Model Context Protocol server that proxies MCP `tools/call`
//! requests onto kleos-server HTTP routes, signing each request with the
//! local PIV / Ed25519 identity.
//!
//! Every tool corresponds to one entry in `kleos_client::routes::ROUTES`.
//! Adding a new tool means adding a route entry; no per-tool handler code.

pub mod tools;
pub mod transport;

use kleos_client::Client;
use serde_json::{json, Value};
use std::sync::Arc;

/// Application state -- a single shared HTTP client.
#[derive(Clone)]
pub struct App {
    pub client: Arc<Client>,
}

/// App lifecycle and bootstrap helpers.
impl App {
    /// Bootstrap an App from environment variables.
    ///
    /// Reads `KLEOS_URL` for the server endpoint (default
    /// `http://127.0.0.1:4200`, matching the rest of the workspace) and
    /// loads a PIV / Ed25519 signer via the standard
    /// `RequestSigner::from_env_or_file` path.
    pub fn from_env() -> Result<Self, String> {
        let base_url =
            std::env::var("KLEOS_URL").unwrap_or_else(|_| "http://127.0.0.1:4200".to_string());
        let host_label = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());
        let agent_label = std::env::var("KLEOS_AGENT_LABEL").unwrap_or_else(|_| "kleos-mcp".into());
        let model_label = std::env::var("KLEOS_MODEL_LABEL").unwrap_or_else(|_| "none".into());

        let signer = kleos_lib::auth_piv::RequestSigner::from_env_or_file(
            &host_label,
            &agent_label,
            &model_label,
        )
        .map_err(|e| format!("PIV identity load failed: {e}"))?;

        let api_key = std::env::var("KLEOS_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty());

        if signer.is_none() && api_key.is_none() {
            return Err(
                "no auth configured: set KLEOS_IDENTITY_PATH (or run `kleos-cli identity init`) \
                 for PIV signing, or set KLEOS_API_KEY as a bearer fallback. Refusing to start \
                 unauthenticated."
                    .to_string(),
            );
        }

        let client = Client::new(base_url, api_key, signer);
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

/// Returns the MCP protocol version this server speaks.
fn protocol_version() -> &'static str {
    "2024-11-05"
}

/// Builds a JSON-RPC success response.
fn response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// Builds a JSON-RPC error response with the given code and message.
fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message }})
}

/// Extracts the request id from a JSON-RPC envelope.
fn request_id(req: &Value) -> Option<Value> {
    req.get("id").cloned()
}

/// Handles one JSON-RPC request and returns the response envelope (or None for notifications).
#[tracing::instrument(skip(app, req), fields(method = req.get("method").and_then(|v| v.as_str()).unwrap_or("")))]
pub async fn handle_jsonrpc(app: &App, req: Value) -> Option<Value> {
    let id = request_id(&req);
    let method = req.get("method")?.as_str()?;

    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
    match method {
        "initialize" => {
            let id = id?;
            Some(response(
                id,
                json!({
                    "protocolVersion": protocol_version(),
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "kleos-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            ))
        }
        "notifications/initialized" => None,
        "tools/list" => {
            let id = id?;
            Some(response(id, json!({ "tools": tools::registry() })))
        }
        "tools/call" => {
            let id = id?;
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let call_result = tools::dispatch(app, &name, arguments).await;
            match call_result {
                Ok(value) => Some(response(
                    id,
                    json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
                        }],
                        "structuredContent": value,
                        "isError": false
                    }),
                )),
                Err(message) => Some(response(
                    id,
                    json!({
                        "content": [{ "type": "text", "text": message }],
                        "isError": true
                    }),
                )),
            }
        }
        _ => id.map(|id| error_response(id, -32601, "method not found")),
    }
}
