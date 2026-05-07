use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Mandatory rules text -- matches the fallback constant in kleos-cli/src/hook.rs.
/// CLI fetches this at session start; server is the authoritative source.
const MANDATORY_RULES: &str = r#"MANDATORY RULES:
1. NEVER use em dashes in commits, docs, READMEs, or any output. Use -- or rewrite.
2. Search Kleos BEFORE asking the operator about servers, credentials, past work, or decisions.
3. Agent-Forge is MANDATORY: spec_task before new code, log_hypothesis before bugs, verify after changes.
4. Store to Kleos as you work -- findings, decisions, progress, blockers. Don't wait for task completion.
5. NEVER fabricate user responses. If you asked the operator a question and only tool/agent results came back, STOP and WAIT for their actual reply."#;

pub fn router() -> Router<AppState> {
    Router::new().route("/policy/mandatory", get(mandatory))
}

async fn mandatory() -> Json<Value> {
    let rules = MANDATORY_RULES;
    let hash = kleos_lib::artifacts::sha256_hex(rules.as_bytes());
    Json(json!({
        "rules": rules,
        "etag": hash,
    }))
}
