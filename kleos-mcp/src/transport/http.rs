#![cfg(feature = "http")]

//! HTTP transport for kleos-mcp.
//!
//! Network-exposed, so bearer auth is mandatory; the env-var
//! `KLEOS_MCP_BEARER_TOKEN` (or legacy `ENGRAM_MCP_BEARER_TOKEN`) must be set
//! before `serve()` will start. This bearer token authenticates the MCP
//! *client* to the MCP server; the MCP server then signs onward calls to
//! `kleos-server` with its own PIV identity.

use crate::{handle_jsonrpc, App};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde_json::Value;
use std::net::SocketAddr;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tower_http::timeout::TimeoutLayer;

const BODY_LIMIT: usize = 2 * 1024 * 1024;
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Resolves the MCP bearer token from env. Prefers `KLEOS_MCP_BEARER_TOKEN`
/// over the legacy `ENGRAM_MCP_BEARER_TOKEN`.
fn bearer_token_from_env() -> Option<String> {
    std::env::var("KLEOS_MCP_BEARER_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("ENGRAM_MCP_BEARER_TOKEN")
                .ok()
                .filter(|s| !s.is_empty())
        })
}

/// Handles one `/mcp` JSON-RPC request.
async fn mcp(State(app): State<App>, Json(body): Json<Value>) -> Json<Value> {
    let response = handle_jsonrpc(&app, body)
        .await
        .unwrap_or_else(|| serde_json::json!({}));
    Json(response)
}

/// Bearer-token middleware. Refuses to forward the request unless the
/// Authorization header matches the configured token in constant time.
async fn bearer_auth(request: Request<Body>, next: Next) -> Response {
    let token = match bearer_token_from_env() {
        Some(t) => t,
        None => {
            tracing::error!(
                "KLEOS_MCP_BEARER_TOKEN not set; MCP HTTP transport requires a bearer token"
            );
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "server misconfigured: bearer token not set"
                })),
            )
                .into_response();
        }
    };

    let presented = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    if presented.is_empty()
        || presented.as_bytes().len() != token.as_bytes().len()
        || presented.as_bytes().ct_eq(token.as_bytes()).unwrap_u8() != 1
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid or missing bearer token" })),
        )
            .into_response();
    }

    next.run(request).await
}

/// Assembles the Axum router with auth + size + timeout layers.
pub fn router(app: App) -> Router {
    Router::new()
        .route("/mcp", post(mcp))
        .layer(middleware::from_fn(bearer_auth))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
        ))
        .with_state(app)
}

/// Binds the listener and serves until shutdown.
pub async fn serve(app: App, listen: &str) -> Result<(), String> {
    if bearer_token_from_env().is_none() {
        return Err("KLEOS_MCP_BEARER_TOKEN must be set for HTTP transport".into());
    }
    let addr: SocketAddr = listen
        .parse()
        .map_err(|e| format!("invalid listen address: {e}"))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!(addr = %addr, "MCP HTTP transport listening (auth required)");
    axum::serve(listener, router(app))
        .await
        .map_err(|e| e.to_string())
}
