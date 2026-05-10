use serde::Deserialize;

/// Query parameters for GET /skills.
#[derive(Debug, Deserialize)]
pub(super) struct ListSkillsParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub agent: Option<String>,
}

/// Body for POST /skills/search.
#[derive(Debug, Deserialize)]
pub(super) struct SearchSkillsBody {
    pub query: String,
    pub limit: Option<usize>,
}

/// Body for POST /skills/{id}/execute recording an execution outcome.
#[derive(Debug, Deserialize)]
pub(super) struct RecordExecutionBody {
    pub success: bool,
    pub duration_ms: Option<f64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

/// Query parameters for GET /skills/{id}/executions.
#[derive(Debug, Deserialize)]
pub(super) struct GetExecutionsParams {
    pub limit: Option<usize>,
}

/// Body for POST /skills/{id}/judge.
#[derive(Debug, Deserialize)]
pub(super) struct JudgeBody {
    pub judge_agent: String,
    pub score: f64,
    pub rationale: Option<String>,
}

/// Body for POST /tools/quality.
#[derive(Debug, Deserialize)]
pub(super) struct RecordToolQualityBody {
    pub tool_name: String,
    pub agent: String,
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub error_type: Option<String>,
}

/// Query parameters for GET /skills/dashboard/stats.
#[derive(Debug, Deserialize)]
pub(super) struct StatsParams {
    pub sort_by: Option<String>,
    pub limit: Option<usize>,
}

/// Body for POST /skills/capture -- derive a skill from a natural-language description.
#[derive(Debug, Deserialize)]
pub(super) struct CaptureBody {
    pub description: String,
    pub agent: Option<String>,
}

/// Body for POST /skills/derive -- synthesise a skill from parent skill IDs.
#[derive(Debug, Deserialize)]
pub(super) struct DeriveBody {
    pub parent_ids: Vec<i64>,
    pub direction: String,
    pub agent: Option<String>,
}

/// Body for POST /skills/cloud/search.
#[derive(Debug, Deserialize)]
pub(super) struct CloudSearchBody {
    pub query: String,
    pub limit: Option<usize>,
}

/// Body for POST /skills/cloud/upload.
#[derive(Debug, Deserialize)]
pub(super) struct CloudUploadBody {
    pub name: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
}

/// Body for POST /skills/sync -- filesystem import from allowed directories.
#[derive(Debug, Deserialize)]
pub(super) struct SyncSkillsBody {
    pub dirs: Option<Vec<String>>,
}

/// Body for POST /skills/execute -- run a task using matched skills as context.
#[derive(Debug, Deserialize)]
pub(super) struct ExecuteSkillsBody {
    pub task: String,
    #[allow(dead_code)]
    pub skill_dirs: Option<Vec<String>>,
    #[allow(dead_code)]
    pub search_scope: Option<String>,
}

/// Body for POST /skills/upload -- publish a locally-tracked skill to the cloud.
#[derive(Debug, Deserialize)]
pub(super) struct UploadSkillBody {
    pub skill_dir: String,
    #[allow(dead_code)]
    pub visibility: Option<String>,
    #[allow(dead_code)]
    pub origin: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Query parameters for GET /skills/evolution/recent.
#[derive(Debug, Deserialize)]
pub(super) struct EvolutionRecentParams {
    pub hours: Option<u32>,
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Skills Cloud (v50+) request bodies
// ---------------------------------------------------------------------------

/// Body for POST /skills/find -- hybrid keyword + semantic search with optional filters.
#[derive(Debug, Deserialize)]
pub(super) struct FindSkillsBody {
    pub query: String,
    pub limit: Option<usize>,
    pub kind: Option<String>,
    pub plugin: Option<String>,
    pub tag: Option<String>,
    pub include_deprecated: Option<bool>,
}

/// Body for POST /skills/aliases/resolve -- resolve a fuzzy alias into ranked candidates.
#[derive(Debug, Deserialize)]
pub(super) struct ResolveAliasBody {
    pub query: String,
    pub limit: Option<usize>,
}

/// Body for POST /skills/{id}/aliases -- attach a named alias to a skill.
#[derive(Debug, Deserialize)]
pub(super) struct AddAliasBody {
    pub alias: String,
    pub confidence: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
}

/// Body for POST /bundles -- create or upsert a skill bundle by name.
#[derive(Debug, Deserialize)]
pub(super) struct CreateBundleBody {
    pub name: String,
    pub description: Option<String>,
    pub auto_generated: Option<bool>,
}

/// Body for POST /bundles/{id}/skills -- add one skill to a bundle.
#[derive(Debug, Deserialize)]
pub(super) struct AddBundleMemberBody {
    pub skill_id: i64,
}

/// Body for POST /skills/{id}/materialize -- record where an agent was written to disk.
#[derive(Debug, Deserialize)]
pub(super) struct RecordMaterializationBody {
    pub target_path: String,
    pub content_hash: String,
}
