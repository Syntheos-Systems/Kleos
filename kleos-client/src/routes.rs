//! Route registry. One entry per HTTP route on `kleos-server`. Both the
//! `kleos-mcp` tool list and the runtime dispatcher iterate this slice -- so
//! adding a new route means a single new entry here, nothing else.

use serde_json::Value;

/// HTTP method for a registered route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// HTTP-verb formatting for the canonical envelope.
impl Method {
    /// The method as the uppercase HTTP verb string used for signing.
    pub fn as_verb(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        }
    }
}

/// Required scope on the server side. Surfaced for documentation; the server
/// is the authoritative enforcer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Read,
    Write,
    Admin,
}

/// One MCP tool == one kleos-server HTTP route.
///
/// `path` is a template; `{name}` segments are filled from the corresponding
/// top-level field of the call arguments before the request goes out, and the
/// field is dropped from the JSON body so it does not duplicate the path.
#[derive(Clone, Copy, Debug)]
pub struct Route {
    /// Canonical tool name, e.g. "memory.store".
    pub name: &'static str,
    /// Back-compat aliases. Same dispatch target.
    pub aliases: &'static [&'static str],
    pub method: Method,
    pub path: &'static str,
    pub scope: Scope,
    pub description: &'static str,
    /// Raw JSON Schema literal for the tool's input. Parsed once at startup.
    pub input_schema: &'static str,
}

/// Walks the registry and returns the route whose canonical name or aliases match.
pub fn find_by_name(name: &str) -> Option<&'static Route> {
    ROUTES.iter().find(|r| r.name == name || r.aliases.contains(&name))
}

/// Substitutes `{key}` segments in the template with values from `args`,
/// removing those keys from `args` so they do not duplicate in the body.
pub fn render_path(template: &str, args: &mut Value) -> Result<String, String> {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after_open = &rest[open + 1..];
        let close = after_open
            .find('}')
            .ok_or_else(|| format!("malformed path template: {template}"))?;
        let key = &after_open[..close];
        let value = args
            .as_object_mut()
            .and_then(|m| m.remove(key))
            .ok_or_else(|| format!("missing path argument '{key}' for template {template}"))?;
        match value {
            Value::String(s) => out.push_str(&s),
            Value::Number(n) => out.push_str(&n.to_string()),
            other => out.push_str(&other.to_string()),
        }
        rest = &after_open[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The route registry. Hand-maintained for now; one entry per HTTP route.
///
/// Entries here drive both the MCP `tools/list` payload and the dispatcher.
/// The kleos-cli aliases on memory/skills/etc preserve back-compat with
/// existing MCP client configurations that may reference the old names.
pub static ROUTES: &[Route] = &[
    // -- memory -----------------------------------------------------------
    Route {
        name: "memory.store",
        aliases: &[],
        method: Method::Post,
        path: "/store",
        scope: Scope::Write,
        description: "Store a new memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "content": {"type": "string"},
                "category": {"type": "string"},
                "source": {"type": "string"},
                "importance": {"type": "integer"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "session_id": {"type": "string"},
                "is_static": {"type": "boolean"},
                "space_id": {"type": "integer"}
            },
            "required": ["content"]
        }"#,
    },
    Route {
        name: "memory.search",
        aliases: &[],
        method: Method::Post,
        path: "/search",
        scope: Scope::Read,
        description: "Hybrid search across memories.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"},
                "category": {"type": "string"},
                "source": {"type": "string"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "threshold": {"type": "number"},
                "space_id": {"type": "integer"},
                "include_forgotten": {"type": "boolean"},
                "mode": {"type": "string"},
                "question_type": {"type": "string"},
                "expand_relationships": {"type": "boolean"},
                "include_links": {"type": "boolean"},
                "latest_only": {"type": "boolean"},
                "source_filter": {"type": "string"}
            },
            "required": ["query"]
        }"#,
    },
    Route {
        name: "memory.get",
        aliases: &[],
        method: Method::Get,
        path: "/memory/{id}",
        scope: Scope::Read,
        description: "Fetch a memory by id.",
        input_schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "integer"}},
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.list",
        aliases: &[],
        method: Method::Get,
        path: "/list",
        scope: Scope::Read,
        description: "Paginated listing of memories.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "limit": {"type": "integer"},
                "offset": {"type": "integer"},
                "category": {"type": "string"},
                "source": {"type": "string"},
                "space_id": {"type": "integer"},
                "include_forgotten": {"type": "boolean"},
                "include_archived": {"type": "boolean"}
            }
        }"#,
    },
    Route {
        name: "memory.update",
        aliases: &[],
        method: Method::Post,
        path: "/memory/{id}/update",
        scope: Scope::Write,
        description: "Update a memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer"},
                "content": {"type": "string"},
                "category": {"type": "string"},
                "source": {"type": "string"},
                "importance": {"type": "integer"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "status": {"type": "string"}
            },
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.delete",
        aliases: &[],
        method: Method::Delete,
        path: "/memory/{id}",
        scope: Scope::Write,
        description: "Delete a memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "integer"}},
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.mark_forgotten",
        aliases: &[],
        method: Method::Post,
        path: "/memory/{id}/forget",
        scope: Scope::Write,
        description: "Soft-delete a memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "integer"}, "reason": {"type": "string"}},
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.mark_archived",
        aliases: &[],
        method: Method::Post,
        path: "/memory/{id}/archive",
        scope: Scope::Write,
        description: "Archive a memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "integer"}},
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.mark_unarchived",
        aliases: &[],
        method: Method::Post,
        path: "/memory/{id}/unarchive",
        scope: Scope::Write,
        description: "Restore an archived memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "integer"}},
            "required": ["id"]
        }"#,
    },
    Route {
        name: "memory.recall",
        aliases: &[],
        method: Method::Post,
        path: "/recall",
        scope: Scope::Read,
        description: "Recall by query, returns ranked memories.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"}
            },
            "required": ["query"]
        }"#,
    },
    // -- skills -----------------------------------------------------------
    Route {
        name: "skill.search",
        aliases: &[],
        method: Method::Post,
        path: "/skills/search",
        scope: Scope::Read,
        description: "Search skill registry.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"},
                "tags": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["query"]
        }"#,
    },
    Route {
        name: "skill.execute",
        aliases: &[],
        method: Method::Post,
        path: "/skills/execute",
        scope: Scope::Read,
        description: "Match skills to a task and return the most relevant skill content.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "task": {"type": "string"},
                "limit": {"type": "integer"}
            },
            "required": ["task"]
        }"#,
    },
    Route {
        name: "skill.upload",
        aliases: &[],
        method: Method::Post,
        path: "/skills/capture",
        scope: Scope::Write,
        description: "Create a new skill record.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "code": {"type": "string"},
                "language": {"type": "string"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "agent": {"type": "string"}
            },
            "required": ["name", "code"]
        }"#,
    },
    Route {
        name: "skill.fix",
        aliases: &[],
        method: Method::Post,
        path: "/skills/{skill_id}/fix",
        scope: Scope::Write,
        description: "Run LLM-driven repair on a skill.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "skill_id": {"type": "integer"},
                "direction": {"type": "string"}
            },
            "required": ["skill_id"]
        }"#,
    },
    // -- activity ---------------------------------------------------------
    Route {
        name: "activity.report",
        aliases: &[],
        method: Method::Post,
        path: "/activity",
        scope: Scope::Write,
        description: "Unified fan-out hub (Chiasm, Axon, Broca, Thymus, Skills, Memory).",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "action": {"type": "string"},
                "summary": {"type": "string"},
                "project": {"type": "string"},
                "agent": {"type": "string"},
                "metadata": {"type": "object"}
            },
            "required": ["action", "summary"]
        }"#,
    },
    // -- health -----------------------------------------------------------
    Route {
        name: "health",
        aliases: &[],
        method: Method::Get,
        path: "/health",
        scope: Scope::Read,
        description: "Server liveness probe.",
        input_schema: r#"{"type": "object"}"#,
    },
    // -- handoffs ---------------------------------------------------------
    Route {
        name: "handoffs.store",
        aliases: &["handoffs.dump"],
        method: Method::Post,
        path: "/handoffs",
        scope: Scope::Write,
        description: "Store a new handoff dump.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "agent": {"type": "string"},
                "handoff_type": {"type": "string"},
                "content": {"type": "string"},
                "metadata": {"type": "object"}
            },
            "required": ["content"]
        }"#,
    },
    Route {
        name: "handoffs.list",
        aliases: &[],
        method: Method::Get,
        path: "/handoffs",
        scope: Scope::Read,
        description: "List handoffs.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}, "offset": {"type":"integer"}, "agent": {"type":"string"}, "handoff_type": {"type":"string"}}}"#,
    },
    Route {
        name: "handoffs.latest",
        aliases: &[],
        method: Method::Get,
        path: "/handoffs/latest",
        scope: Scope::Read,
        description: "Fetch the most recent handoff.",
        input_schema: r#"{"type": "object", "properties": {"agent": {"type":"string"}, "handoff_type": {"type":"string"}}}"#,
    },
    Route {
        name: "handoffs.search",
        aliases: &[],
        method: Method::Get,
        path: "/handoffs/search",
        scope: Scope::Read,
        description: "Search handoffs by query string.",
        input_schema: r#"{"type": "object", "properties": {"query": {"type":"string"}, "limit": {"type":"integer"}}, "required": ["query"]}"#,
    },
    Route {
        name: "handoffs.stats",
        aliases: &[],
        method: Method::Get,
        path: "/handoffs/stats",
        scope: Scope::Read,
        description: "Aggregate handoff statistics.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "handoffs.gc",
        aliases: &[],
        method: Method::Post,
        path: "/handoffs/gc",
        scope: Scope::Write,
        description: "Garbage-collect old handoffs.",
        input_schema: r#"{"type": "object", "properties": {"tiered": {"type":"boolean"}, "keep": {"type":"integer"}}}"#,
    },
    Route {
        name: "handoffs.delete",
        aliases: &[],
        method: Method::Delete,
        path: "/handoffs/{id}",
        scope: Scope::Write,
        description: "Delete a specific handoff.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    // -- jobs -------------------------------------------------------------
    Route {
        name: "jobs.list",
        aliases: &[],
        method: Method::Get,
        path: "/jobs",
        scope: Scope::Read,
        description: "List background jobs.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}, "status": {"type":"string"}}}"#,
    },
    Route {
        name: "jobs.pending",
        aliases: &[],
        method: Method::Get,
        path: "/jobs/pending",
        scope: Scope::Read,
        description: "List pending jobs.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "jobs.running",
        aliases: &[],
        method: Method::Get,
        path: "/jobs/running",
        scope: Scope::Read,
        description: "List currently running jobs.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "jobs.failed",
        aliases: &[],
        method: Method::Get,
        path: "/jobs/failed",
        scope: Scope::Read,
        description: "List failed jobs.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "jobs.retry",
        aliases: &[],
        method: Method::Post,
        path: "/jobs/retry",
        scope: Scope::Write,
        description: "Retry failed jobs (all matching).",
        input_schema: r#"{"type": "object", "properties": {"all": {"type":"boolean"}, "kind": {"type":"string"}}}"#,
    },
    Route {
        name: "jobs.retry_by_id",
        aliases: &[],
        method: Method::Post,
        path: "/jobs/{id}/retry",
        scope: Scope::Write,
        description: "Retry a specific job by id.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    Route {
        name: "jobs.purge",
        aliases: &[],
        method: Method::Post,
        path: "/jobs/purge",
        scope: Scope::Admin,
        description: "Permanently remove jobs matching the criteria.",
        input_schema: r#"{"type": "object", "properties": {"status": {"type":"string"}, "before": {"type":"string"}}}"#,
    },
    Route {
        name: "jobs.cleanup",
        aliases: &[],
        method: Method::Post,
        path: "/jobs/cleanup",
        scope: Scope::Admin,
        description: "Garbage-collect completed jobs.",
        input_schema: r#"{"type": "object", "properties": {"keep": {"type":"integer"}}}"#,
    },
    Route {
        name: "jobs.stats",
        aliases: &[],
        method: Method::Get,
        path: "/jobs/stats",
        scope: Scope::Read,
        description: "Job queue statistics.",
        input_schema: r#"{"type": "object"}"#,
    },
    // -- fsrs (spaced repetition) -----------------------------------------
    Route {
        name: "fsrs.review",
        aliases: &[],
        method: Method::Post,
        path: "/fsrs/review",
        scope: Scope::Write,
        description: "Record an FSRS review outcome for a memory.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type":"integer"},
                "rating": {"type":"integer", "description":"1=Again, 2=Hard, 3=Good, 4=Easy"}
            },
            "required": ["memory_id", "rating"]
        }"#,
    },
    Route {
        name: "fsrs.state",
        aliases: &[],
        method: Method::Get,
        path: "/fsrs/state",
        scope: Scope::Read,
        description: "Fetch FSRS scheduling state for a memory.",
        input_schema: r#"{"type": "object", "properties": {"memory_id": {"type":"integer"}}, "required": ["memory_id"]}"#,
    },
    Route {
        name: "fsrs.init",
        aliases: &[],
        method: Method::Post,
        path: "/fsrs/init",
        scope: Scope::Admin,
        description: "Backfill FSRS state for existing memories.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}}}"#,
    },
    Route {
        name: "fsrs.recall_due",
        aliases: &[],
        method: Method::Get,
        path: "/fsrs/recall-due",
        scope: Scope::Read,
        description: "List memories due for spaced-repetition reinforcement.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}}}"#,
    },
    // -- tasks (chiasm) ---------------------------------------------------
    Route {
        name: "tasks.list",
        aliases: &[],
        method: Method::Get,
        path: "/tasks",
        scope: Scope::Read,
        description: "List Chiasm coordination tasks.",
        input_schema: r#"{"type": "object", "properties": {"agent": {"type":"string"}, "project": {"type":"string"}, "status": {"type":"string"}, "limit": {"type":"integer"}}}"#,
    },
    Route {
        name: "tasks.create",
        aliases: &["services.chiasm_create_task"],
        method: Method::Post,
        path: "/tasks",
        scope: Scope::Write,
        description: "Create a Chiasm coordination task.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "agent": {"type": "string"},
                "project": {"type": "string"},
                "title": {"type": "string"},
                "summary": {"type": "string"},
                "status": {"type": "string"}
            },
            "required": ["agent", "project", "title"]
        }"#,
    },
    Route {
        name: "tasks.stats",
        aliases: &[],
        method: Method::Get,
        path: "/tasks/stats",
        scope: Scope::Read,
        description: "Aggregate task statistics.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "tasks.history",
        aliases: &[],
        method: Method::Get,
        path: "/tasks/{id}/history",
        scope: Scope::Read,
        description: "Task status history.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    Route {
        name: "tasks.feed",
        aliases: &[],
        method: Method::Get,
        path: "/feed",
        scope: Scope::Read,
        description: "Cross-task activity feed.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}}}"#,
    },
    // -- sessions ---------------------------------------------------------
    Route {
        name: "sessions.get",
        aliases: &[],
        method: Method::Get,
        path: "/sessions/{id}",
        scope: Scope::Read,
        description: "Fetch a session record.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"string"}}, "required": ["id"]}"#,
    },
    Route {
        name: "sessions.append",
        aliases: &[],
        method: Method::Post,
        path: "/sessions/{id}/append",
        scope: Scope::Write,
        description: "Append an entry to a session.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type":"string"},
                "role": {"type":"string"},
                "content": {"type":"string"},
                "metadata": {"type":"object"}
            },
            "required": ["id", "content"]
        }"#,
    },
    // -- scratchpad -------------------------------------------------------
    Route {
        name: "scratchpad.list",
        aliases: &[],
        method: Method::Get,
        path: "/scratch",
        scope: Scope::Read,
        description: "List scratchpad entries.",
        input_schema: r#"{"type": "object", "properties": {"session": {"type":"string"}}}"#,
    },
    Route {
        name: "scratchpad.put",
        aliases: &[],
        method: Method::Put,
        path: "/scratch",
        scope: Scope::Write,
        description: "Store or update a scratchpad entry.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "session": {"type":"string"},
                "key": {"type":"string"},
                "value": {}
            },
            "required": ["session", "key", "value"]
        }"#,
    },
    Route {
        name: "scratchpad.delete_session",
        aliases: &[],
        method: Method::Delete,
        path: "/scratch/{session}",
        scope: Scope::Write,
        description: "Delete every scratchpad entry for a session.",
        input_schema: r#"{"type": "object", "properties": {"session": {"type":"string"}}, "required": ["session"]}"#,
    },
    Route {
        name: "scratchpad.delete_key",
        aliases: &[],
        method: Method::Delete,
        path: "/scratch/{session}/{key}",
        scope: Scope::Write,
        description: "Delete a specific scratchpad key.",
        input_schema: r#"{"type": "object", "properties": {"session": {"type":"string"}, "key": {"type":"string"}}, "required": ["session", "key"]}"#,
    },
    Route {
        name: "scratchpad.promote",
        aliases: &[],
        method: Method::Post,
        path: "/scratch/{session}/promote",
        scope: Scope::Write,
        description: "Promote a scratchpad session to durable memory.",
        input_schema: r#"{"type": "object", "properties": {"session": {"type":"string"}}, "required": ["session"]}"#,
    },
    // -- episodes ---------------------------------------------------------
    Route {
        name: "episodes.list",
        aliases: &[],
        method: Method::Get,
        path: "/episodes",
        scope: Scope::Read,
        description: "List episodes.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}, "offset": {"type":"integer"}}}"#,
    },
    Route {
        name: "episodes.create",
        aliases: &[],
        method: Method::Post,
        path: "/episodes",
        scope: Scope::Write,
        description: "Create a new episode.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "title": {"type":"string"},
                "summary": {"type":"string"},
                "started_at": {"type":"string"},
                "metadata": {"type":"object"}
            },
            "required": ["title"]
        }"#,
    },
    Route {
        name: "episodes.get",
        aliases: &[],
        method: Method::Get,
        path: "/episodes/{id}",
        scope: Scope::Read,
        description: "Fetch one episode by id.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    Route {
        name: "episodes.update",
        aliases: &[],
        method: Method::Patch,
        path: "/episodes/{id}",
        scope: Scope::Write,
        description: "Update an episode.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type":"integer"},
                "title": {"type":"string"},
                "summary": {"type":"string"},
                "metadata": {"type":"object"}
            },
            "required": ["id"]
        }"#,
    },
    Route {
        name: "episodes.assign_memories",
        aliases: &[],
        method: Method::Post,
        path: "/episodes/{id}/memories",
        scope: Scope::Write,
        description: "Attach memories to an episode.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type":"integer"},
                "memory_ids": {"type":"array","items":{"type":"integer"}}
            },
            "required": ["id", "memory_ids"]
        }"#,
    },
    Route {
        name: "episodes.finalize",
        aliases: &[],
        method: Method::Post,
        path: "/episodes/{id}/finalize",
        scope: Scope::Write,
        description: "Close an episode and lock its membership.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    // -- prompts ----------------------------------------------------------
    Route {
        name: "prompts.get",
        aliases: &[],
        method: Method::Get,
        path: "/prompt",
        scope: Scope::Read,
        description: "Fetch the active prompt template.",
        input_schema: r#"{"type": "object", "properties": {"name": {"type":"string"}}}"#,
    },
    Route {
        name: "prompts.generate",
        aliases: &["context.generate_prompt"],
        method: Method::Post,
        path: "/prompt/generate",
        scope: Scope::Read,
        description: "Pack memories into a prompt for a given task.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "task": {"type": "string"},
                "format": {"type": "string"},
                "limit": {"type": "integer"}
            },
            "required": ["task"]
        }"#,
    },
    Route {
        name: "prompts.header",
        aliases: &["context.get_header"],
        method: Method::Post,
        path: "/header",
        scope: Scope::Read,
        description: "Generate a task header from cross-agent activity.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "task": {"type": "string"},
                "agents": {"type": "array", "items": {"type":"string"}}
            }
        }"#,
    },
    // -- personality ------------------------------------------------------
    Route {
        name: "personality.detect",
        aliases: &[],
        method: Method::Post,
        path: "/personality/detect",
        scope: Scope::Read,
        description: "Detect personality cues from a sample of text.",
        input_schema: r#"{"type": "object", "properties": {"text": {"type":"string"}}, "required": ["text"]}"#,
    },
    Route {
        name: "personality.profile",
        aliases: &[],
        method: Method::Get,
        path: "/personality/profile",
        scope: Scope::Read,
        description: "Fetch the current personality profile.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "personality.update_profile",
        aliases: &[],
        method: Method::Post,
        path: "/personality/profile/update",
        scope: Scope::Write,
        description: "Update fields on the personality profile.",
        input_schema: r#"{"type": "object", "properties": {"traits": {"type":"object"}, "notes": {"type":"string"}}}"#,
    },
    // -- ingestion --------------------------------------------------------
    Route {
        name: "ingestion.text",
        aliases: &[],
        method: Method::Post,
        path: "/ingest",
        scope: Scope::Write,
        description: "Ingest raw text into the memory pipeline.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "text": {"type":"string"},
                "category": {"type":"string"},
                "source": {"type":"string"},
                "tags": {"type":"array","items":{"type":"string"}}
            },
            "required": ["text"]
        }"#,
    },
    Route {
        name: "ingestion.bulk",
        aliases: &[],
        method: Method::Post,
        path: "/import/bulk",
        scope: Scope::Write,
        description: "Bulk import of memories from a structured payload.",
        input_schema: r#"{"type": "object", "properties": {"items": {"type":"array"}}, "required": ["items"]}"#,
    },
    Route {
        name: "ingestion.json",
        aliases: &[],
        method: Method::Post,
        path: "/import/json",
        scope: Scope::Write,
        description: "Import memories from a JSON dump.",
        input_schema: r#"{"type": "object", "properties": {"data": {"type":"object"}}, "required": ["data"]}"#,
    },
    Route {
        name: "ingestion.upload_init",
        aliases: &[],
        method: Method::Post,
        path: "/ingest/upload/init",
        scope: Scope::Write,
        description: "Begin a chunked upload session.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "filename": {"type":"string"},
                "size": {"type":"integer"},
                "sha256": {"type":"string"},
                "mime": {"type":"string"}
            },
            "required": ["filename", "size"]
        }"#,
    },
    Route {
        name: "ingestion.upload_chunk",
        aliases: &[],
        method: Method::Post,
        path: "/ingest/upload/chunk",
        scope: Scope::Write,
        description: "Submit one chunk in a chunked upload.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "upload_id": {"type":"string"},
                "offset": {"type":"integer"},
                "data": {"type":"string","description":"base64-encoded chunk"}
            },
            "required": ["upload_id", "offset", "data"]
        }"#,
    },
    Route {
        name: "ingestion.upload_complete",
        aliases: &[],
        method: Method::Post,
        path: "/ingest/upload/complete",
        scope: Scope::Write,
        description: "Finalise a chunked upload.",
        input_schema: r#"{"type": "object", "properties": {"upload_id": {"type":"string"}}, "required": ["upload_id"]}"#,
    },
    Route {
        name: "ingestion.upload_abort",
        aliases: &[],
        method: Method::Post,
        path: "/ingest/upload/abort",
        scope: Scope::Write,
        description: "Abort a chunked upload.",
        input_schema: r#"{"type": "object", "properties": {"upload_id": {"type":"string"}}, "required": ["upload_id"]}"#,
    },
    Route {
        name: "ingestion.upload_status",
        aliases: &[],
        method: Method::Get,
        path: "/ingest/upload/{upload_id}/status",
        scope: Scope::Read,
        description: "Check status of a chunked upload.",
        input_schema: r#"{"type": "object", "properties": {"upload_id": {"type":"string"}}, "required": ["upload_id"]}"#,
    },
    // -- inbox ------------------------------------------------------------
    Route {
        name: "inbox.list",
        aliases: &[],
        method: Method::Get,
        path: "/inbox",
        scope: Scope::Read,
        description: "List pending inbox items.",
        input_schema: r#"{"type": "object", "properties": {"limit": {"type":"integer"}}}"#,
    },
    Route {
        name: "inbox.approve",
        aliases: &[],
        method: Method::Post,
        path: "/inbox/{id}/approve",
        scope: Scope::Write,
        description: "Approve an inbox item.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    Route {
        name: "inbox.reject",
        aliases: &[],
        method: Method::Post,
        path: "/inbox/{id}/reject",
        scope: Scope::Write,
        description: "Reject an inbox item.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}, "reason": {"type":"string"}}, "required": ["id"]}"#,
    },
    Route {
        name: "inbox.edit",
        aliases: &[],
        method: Method::Post,
        path: "/inbox/{id}/edit",
        scope: Scope::Write,
        description: "Edit an inbox item before approving.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}, "patch": {"type":"object"}}, "required": ["id", "patch"]}"#,
    },
    Route {
        name: "inbox.bulk",
        aliases: &[],
        method: Method::Post,
        path: "/inbox/bulk",
        scope: Scope::Write,
        description: "Apply an action to multiple inbox items at once.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "ids": {"type":"array","items":{"type":"integer"}},
                "action": {"type":"string"}
            },
            "required": ["ids", "action"]
        }"#,
    },
    // -- projects ---------------------------------------------------------
    Route {
        name: "projects.list",
        aliases: &[],
        method: Method::Get,
        path: "/projects",
        scope: Scope::Read,
        description: "List projects.",
        input_schema: r#"{"type": "object"}"#,
    },
    Route {
        name: "projects.create",
        aliases: &[],
        method: Method::Post,
        path: "/projects",
        scope: Scope::Write,
        description: "Create a project.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "name": {"type":"string"},
                "description": {"type":"string"}
            },
            "required": ["name"]
        }"#,
    },
    Route {
        name: "projects.get",
        aliases: &[],
        method: Method::Get,
        path: "/projects/{id}",
        scope: Scope::Read,
        description: "Fetch a project by id.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}}, "required": ["id"]}"#,
    },
    Route {
        name: "projects.update",
        aliases: &[],
        method: Method::Put,
        path: "/projects/{id}",
        scope: Scope::Write,
        description: "Update a project.",
        input_schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type":"integer"},
                "name": {"type":"string"},
                "description": {"type":"string"}
            },
            "required": ["id"]
        }"#,
    },
    Route {
        name: "projects.link_memory",
        aliases: &[],
        method: Method::Put,
        path: "/projects/{id}/memories/{mid}",
        scope: Scope::Write,
        description: "Link a memory to a project.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}, "mid": {"type":"integer"}}, "required": ["id", "mid"]}"#,
    },
    Route {
        name: "projects.unlink_memory",
        aliases: &[],
        method: Method::Delete,
        path: "/projects/{id}/memories/{mid}",
        scope: Scope::Write,
        description: "Unlink a memory from a project.",
        input_schema: r#"{"type": "object", "properties": {"id": {"type":"integer"}, "mid": {"type":"integer"}}, "required": ["id", "mid"]}"#,
    },
];
