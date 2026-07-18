//! Markdown renderers. These are pure string functions so the emitted shape is
//! testable without touching the filesystem or the database.

use crate::emit::model::{ApproachRow, SpecRecord};
use crate::emit::trust::Trust;

/// The irreducible prose a model supplies for one slice. Everything else in a
/// slice document is assembled from stored rows.
pub struct SliceContent {
    /// One-line statement of what this slice set out to do.
    pub intent: String,
    /// One entry per component touched: what it does and under what conditions.
    pub components: Vec<String>,
    /// Non-obvious conditions: root causes, gotchas, and documented limitations.
    pub conditions: Vec<String>,
}

/// Render a bullet list, or a placeholder line when the list is empty, so a
/// section never renders as a bare heading.
fn bullets(items: &[String]) -> String {
    if items.is_empty() {
        return "_None recorded._\n".to_string();
    }
    items
        .iter()
        .map(|i| format!("- {}\n", i))
        .collect::<Vec<_>>()
        .join("")
}

/// Render the chosen approach as a decision block naming the strongest rejected
/// alternative and the reason it lost. Returns an empty string when no approach
/// was marked chosen, because a decision with no chosen path is not a decision.
fn render_decision(approaches: &[ApproachRow], trust: Trust) -> String {
    let Some(chosen) = approaches.iter().find(|a| a.chosen) else {
        return String::new();
    };

    // The strongest rejected alternative is the highest-scoring unchosen
    // approach; absent scores, the first unchosen one.
    let alternative = approaches.iter().filter(|a| !a.chosen).max_by(|a, b| {
        a.score
            .unwrap_or(f64::MIN)
            .partial_cmp(&b.score.unwrap_or(f64::MIN))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut out = format!("\n## Decision: {}\n\n", chosen.name);
    out.push_str(&format!("- **why:** {}\n", chosen.description));
    if let Some(alt) = alternative {
        let reason = if alt.cons.is_empty() {
            "no reason recorded".to_string()
        } else {
            alt.cons.join("; ")
        };
        out.push_str(&format!(
            "- **alternative:** {} -- rejected: {}\n",
            alt.name, reason
        ));
    }
    out.push_str(&format!("- **trust:** {}\n", trust.label()));
    out
}

/// Render a spec's top-level record document.
pub fn render_record(record: &SpecRecord, trust: Trust) -> String {
    let mut out = format!("# Record: {}\n\n", record.task_description);
    out.push_str(&format!(
        "- **spec:** `{}`\n- **type:** {}\n\n",
        record.id, record.task_type
    ));

    out.push_str("## Acceptance criteria\n\n");
    out.push_str(&bullets(&record.acceptance_criteria));

    out.push_str("\n## Edge cases\n\n");
    out.push_str(&bullets(&record.edge_cases));

    if let Some(contract) = &record.interface_contract {
        out.push_str("\n## Interface contract\n\n");
        out.push_str(&format!("```text\n{}\n```\n", contract));
    }

    out.push_str(&render_decision(&record.approaches, trust));

    out.push_str("\n## Verification evidence\n\n");
    if record.verifications.is_empty() {
        out.push_str("_No verification runs recorded._\n");
    } else {
        for v in &record.verifications {
            out.push_str(&format!(
                "- `{}` -- {}\n",
                v.command,
                if v.success { "passed" } else { "failed" }
            ));
        }
    }

    out
}

/// Render one slice document: the model's knowledge-transfer prose plus the
/// decisions and discoveries stored for the spec.
pub fn render_slice(
    index: i64,
    content: &SliceContent,
    record: &SpecRecord,
    trust: Trust,
) -> String {
    let mut out = format!("# Slice {:03}: {}\n\n", index, content.intent);
    out.push_str(&format!("- **spec:** `{}`\n\n", record.id));

    out.push_str("## Components\n\n");
    out.push_str(&bullets(&content.components));

    out.push_str("\n## Hard-won conditions\n\n");
    let mut conditions = content.conditions.clone();
    for learn in &record.learns {
        conditions.push(match &learn.context {
            Some(ctx) => format!("{} ({})", learn.discovery, ctx),
            None => learn.discovery.clone(),
        });
    }
    out.push_str(&bullets(&conditions));

    out.push_str(&render_decision(&record.approaches, trust));

    out
}

#[cfg(test)]
/// Tests for the record and slice renderers.
mod tests {
    use super::*;
    use crate::emit::model::{ApproachRow, LearnRow, SpecRecord, VerificationRow};
    use crate::emit::trust::Trust;

    /// Build a record with one chosen and one rejected approach plus a learning.
    fn record() -> SpecRecord {
        SpecRecord {
            id: "spec_1".into(),
            task_description: "Add a thing".into(),
            task_type: "feature".into(),
            acceptance_criteria: vec!["it works".into()],
            edge_cases: vec!["empty input".into()],
            interface_contract: Some("fn thing() -> u8".into()),
            approaches: vec![
                ApproachRow {
                    name: "Direct".into(),
                    description: "Do it directly".into(),
                    pros: vec!["simple".into()],
                    cons: vec![],
                    score: Some(8.0),
                    chosen: true,
                },
                ApproachRow {
                    name: "Indirect".into(),
                    description: "Add a layer".into(),
                    pros: vec![],
                    cons: vec!["slower".into()],
                    score: Some(5.0),
                    chosen: false,
                },
            ],
            learns: vec![LearnRow {
                discovery: "The cache lies on cold start".into(),
                context: Some("found while tracing".into()),
                tags: vec!["cache".into()],
            }],
            verifications: vec![VerificationRow {
                command: "cargo test".into(),
                success: true,
                criteria_index: Some(0),
            }],
        }
    }

    /// The record renders its intent, criteria, edge cases, and contract.
    #[test]
    fn record_renders_spec_fields() {
        let md = render_record(&record(), Trust::SpecVerified);
        assert!(md.contains("# Record: Add a thing"));
        assert!(md.contains("it works"));
        assert!(md.contains("empty input"));
        assert!(md.contains("fn thing() -> u8"));
    }

    /// The record renders the chosen approach as a decision naming the rejected
    /// alternative and its rejection reason.
    #[test]
    fn record_renders_decision_with_alternative() {
        let md = render_record(&record(), Trust::SpecVerified);
        assert!(md.contains("## Decision: Direct"));
        assert!(md.contains("**alternative:**"));
        assert!(md.contains("Indirect"));
        assert!(md.contains("slower"));
    }

    /// The trust label appears verbatim so its caveat cannot be lost.
    #[test]
    fn record_renders_trust_label() {
        let md = render_record(&record(), Trust::SpecVerified);
        assert!(md.contains("not separately proved"));
    }

    /// A record with no approaches renders without a decisions section rather
    /// than emitting an empty heading.
    #[test]
    fn record_without_approaches_omits_decisions() {
        let mut r = record();
        r.approaches.clear();
        let md = render_record(&r, Trust::Unverified);
        assert!(!md.contains("## Decision:"));
    }

    /// A slice renders its components and its hard-won conditions under distinct
    /// headings, and carries the learnings captured for the spec.
    #[test]
    fn slice_renders_knowledge_transfer() {
        let content = SliceContent {
            intent: "wire it up".into(),
            components: vec!["Renderer -- turns rows into markdown".into()],
            conditions: vec!["Empty specs still render".into()],
        };
        let md = render_slice(2, &content, &record(), Trust::SpecVerified);
        assert!(md.contains("# Slice 002: wire it up"));
        assert!(md.contains("## Components"));
        assert!(md.contains("Renderer -- turns rows into markdown"));
        assert!(md.contains("## Hard-won conditions"));
        assert!(md.contains("Empty specs still render"));
        assert!(md.contains("The cache lies on cold start"));
    }
}
