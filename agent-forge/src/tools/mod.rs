//! Agent-forge tool registry. Each submodule implements one CLI subcommand.
//! Shared types (`ToolResult`, `ToolError`) and the session-active marker live here.

pub mod approaches;
pub mod ast;
pub mod comments;
pub mod hypothesis;
pub mod session;
pub mod skills;
pub mod spec;
pub mod stats;
pub mod think;
pub mod verify;

use crate::json_io::Output;

/// Standard return type for every tool: structured `Output` on success, `ToolError` on failure.
pub type ToolResult = Result<Output, ToolError>;

/// Categorised failure modes for tool execution; rendered to the JSON output's `error` field.
#[derive(Debug)]
pub enum ToolError {
    MissingField(String),
    InvalidValue(String),
    DatabaseError(String),
    IoError(String),
}

/// Render `ToolError` as a short human string for the CLI's error output.
impl std::fmt::Display for ToolError {
    /// Human-readable form used when an error bubbles to the CLI output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::MissingField(s) => write!(f, "Missing required field: {}", s),
            ToolError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            ToolError::DatabaseError(s) => write!(f, "Database error: {}", s),
            ToolError::IoError(s) => write!(f, "I/O error: {}", s),
        }
    }
}

/// Marker impl so `ToolError` plays nicely with `?` and any `dyn Error` chain.
impl std::error::Error for ToolError {}

/// Mark a forge artifact as the currently-active gate state for Claude Code's
/// enforce-agent-forge.sh hook. Best-effort: failures here must not abort the
/// caller, since the DB record (the source of truth) is already committed.
///
/// Writes ~/.claude/session-env/agent-forge-active with "<id>:<kind>".
pub fn set_session_active(id: &str, kind: &str) {
    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let dir = std::path::PathBuf::from(home)
        .join(".claude")
        .join("session-env");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("agent-forge-active"), format!("{}:{}", id, kind));
}
