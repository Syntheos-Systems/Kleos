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
];
