//! Smoke tests for kleos-mcp's JSON-RPC envelope handling and tool registry.
//!
//! These tests do NOT hit a real kleos-server. They verify the surface
//! the MCP client sees: protocol version, tools/list payload shape, and
//! that calls to unknown tools return an error envelope. Live wire tests
//! belong in a separate smoke harness pointed at `$KLEOS_URL`, so the
//! unit tests stay green offline.
//!
//! `test_app` constructs an `App` pointed at a non-routable address; the
//! tests below never invoke a route, only the registry + dispatch paths.

use kleos_client::Client;
use kleos_mcp::{handle_jsonrpc, App};
use serde_json::json;
use std::sync::Arc;

/// Builds an App that will never be asked to make a network call.
fn test_app() -> App {
    let client = Client::new("http://127.0.0.1:1".into(), None, None);
    App {
        client: Arc::new(client),
    }
}

/// `initialize` must echo the supported MCP protocol version and the server name.
#[tokio::test]
async fn initialize_returns_protocol_version() {
    let app = test_app();
    let resp = handle_jsonrpc(
        &app,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
    )
    .await
    .expect("initialize must return a response");
    assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(resp["result"]["serverInfo"]["name"], "kleos-mcp");
}

/// `tools/list` must enumerate the registry, including the daily-driver tools.
#[tokio::test]
async fn tools_list_includes_memory_search() {
    let app = test_app();
    let resp = handle_jsonrpc(
        &app,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    )
    .await
    .expect("tools/list must return a response");
    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools must be an array");
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(
        names.contains(&"memory.search"),
        "memory.search must be in tools/list, got {:?}",
        names
    );
    assert!(
        names.contains(&"memory.store"),
        "memory.store must be in tools/list, got {:?}",
        names
    );
}

/// Unknown tool names must return an error result, not a JSON-RPC fault.
#[tokio::test]
async fn unknown_tool_returns_error_envelope() {
    let app = test_app();
    let resp = handle_jsonrpc(
        &app,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "definitely.not.a.real.tool",
                "arguments": {}
            }
        }),
    )
    .await
    .expect("tools/call must always return a response");
    assert_eq!(resp["result"]["isError"], true);
}
