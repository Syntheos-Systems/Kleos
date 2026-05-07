use crate::db::Database;
use crate::json_io::Output;
use crate::tools::{ToolError, ToolResult};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ConsiderApproachesInput {
    pub spec_id: Option<String>,
    pub problem: Option<String>,
    pub approaches: Option<Vec<ApproachItem>>,
    pub chosen_index: Option<usize>,
}

#[derive(Deserialize)]
pub struct ApproachItem {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub pros: Vec<String>,
    #[serde(default)]
    pub cons: Vec<String>,
    pub score: Option<f64>,
}

pub fn consider_approaches(db: &Database, input: ConsiderApproachesInput) -> ToolResult {
    let problem = input
        .problem
        .ok_or_else(|| ToolError::MissingField("problem".into()))?;
    let approaches = input
        .approaches
        .ok_or_else(|| ToolError::MissingField("approaches".into()))?;
    if approaches.len() < 2 {
        return Err(ToolError::InvalidValue(
            "At least 2 approaches required for comparison".into(),
        ));
    }
    if let Some(idx) = input.chosen_index {
        if idx >= approaches.len() {
            return Err(ToolError::InvalidValue(format!(
                "chosen_index {} out of range (have {} approaches)",
                idx,
                approaches.len()
            )));
        }
    }
    if let Some(spec_id) = &input.spec_id {
        let exists: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM specs WHERE id = ?1",
                rusqlite::params![spec_id],
                |row| row.get(0),
            )
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;
        if exists == 0 {
            return Err(ToolError::InvalidValue(format!(
                "spec_id '{}' does not exist",
                spec_id
            )));
        }
    }

    let now = Utc::now().timestamp();
    let mut stored_ids = Vec::with_capacity(approaches.len());
    for (i, approach) in approaches.iter().enumerate() {
        let id = format!("appr_{}", &Uuid::new_v4().to_string()[..8]);
        let chosen = matches!(input.chosen_index, Some(c) if c == i) as i64;
        db.conn()
            .execute(
                r#"INSERT INTO approaches
                   (id, spec_id, created_at, name, description, pros, cons, score, chosen)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                rusqlite::params![
                    id,
                    input.spec_id,
                    now,
                    approach.name,
                    approach.description,
                    serde_json::to_string(&approach.pros)
                        .map_err(|e| ToolError::DatabaseError(e.to_string()))?,
                    serde_json::to_string(&approach.cons)
                        .map_err(|e| ToolError::DatabaseError(e.to_string()))?,
                    approach.score,
                    chosen,
                ],
            )
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;
        stored_ids.push(id);
    }

    let mut output = Output::ok(format!(
        "Stored {} approaches{}",
        approaches.len(),
        match input.chosen_index {
            Some(i) => format!(" (chose #{}: '{}')", i, approaches[i].name),
            None => " (no choice recorded yet)".into(),
        }
    ));
    output.data = Some(serde_json::json!({
        "ids": stored_ids,
        "spec_id": input.spec_id,
        "problem": problem,
        "approaches": approaches.iter().enumerate().map(|(i, a)| {
            serde_json::json!({
                "index": i, "name": a.name, "description": a.description,
                "pros": a.pros, "cons": a.cons, "score": a.score,
                "chosen": Some(i) == input.chosen_index,
            })
        }).collect::<Vec<_>>(),
        "comparison_prompt": format!(
            "Evaluate the following approaches to: {}\n\n{}\n\nFor each approach, weigh pros vs cons. Identify the dominant factor (correctness, complexity, performance, blast radius). Recommend one and justify in 3 sentences or fewer.",
            problem,
            approaches.iter().enumerate().map(|(i, a)| format!(
                "Approach {}: {}\n  Description: {}\n  Pros: {}\n  Cons: {}\n  Score: {}",
                i, a.name, a.description,
                if a.pros.is_empty() { "(none)".to_string() } else { a.pros.join("; ") },
                if a.cons.is_empty() { "(none)".to_string() } else { a.cons.join("; ") },
                a.score.map(|s| s.to_string()).unwrap_or_else(|| "n/a".into()),
            )).collect::<Vec<_>>().join("\n\n"),
        ),
    }));
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn db() -> (tempfile::TempDir, Database) {
        let dir = tempdir().unwrap();
        let db = Database::open(&dir.path().join("forge.db")).unwrap();
        (dir, db)
    }

    #[test]
    fn requires_problem() {
        let (_d, db) = db();
        assert!(matches!(
            consider_approaches(
                &db,
                ConsiderApproachesInput {
                    spec_id: None,
                    problem: None,
                    approaches: None,
                    chosen_index: None,
                }
            ),
            Err(ToolError::MissingField(_))
        ));
    }

    #[test]
    fn requires_two_approaches() {
        let (_d, db) = db();
        let r = consider_approaches(
            &db,
            ConsiderApproachesInput {
                spec_id: None,
                problem: Some("p".into()),
                approaches: Some(vec![ApproachItem {
                    name: "a".into(),
                    description: "d".into(),
                    pros: vec![],
                    cons: vec![],
                    score: None,
                }]),
                chosen_index: None,
            },
        );
        assert!(matches!(r, Err(ToolError::InvalidValue(_))));
    }

    #[test]
    fn rejects_invalid_chosen_index() {
        let (_d, db) = db();
        let r = consider_approaches(
            &db,
            ConsiderApproachesInput {
                spec_id: None,
                problem: Some("p".into()),
                approaches: Some(vec![
                    ApproachItem {
                        name: "a".into(),
                        description: "d".into(),
                        pros: vec![],
                        cons: vec![],
                        score: None,
                    },
                    ApproachItem {
                        name: "b".into(),
                        description: "d".into(),
                        pros: vec![],
                        cons: vec![],
                        score: None,
                    },
                ]),
                chosen_index: Some(99),
            },
        );
        assert!(matches!(r, Err(ToolError::InvalidValue(_))));
    }

    #[test]
    fn rejects_invalid_spec_id() {
        let (_d, db) = db();
        let r = consider_approaches(
            &db,
            ConsiderApproachesInput {
                spec_id: Some("nope".into()),
                problem: Some("p".into()),
                approaches: Some(vec![
                    ApproachItem {
                        name: "a".into(),
                        description: "d".into(),
                        pros: vec![],
                        cons: vec![],
                        score: None,
                    },
                    ApproachItem {
                        name: "b".into(),
                        description: "d".into(),
                        pros: vec![],
                        cons: vec![],
                        score: None,
                    },
                ]),
                chosen_index: None,
            },
        );
        assert!(matches!(r, Err(ToolError::InvalidValue(_))));
    }

    #[test]
    fn happy_path_with_chosen() {
        let (_d, db) = db();
        let r = consider_approaches(
            &db,
            ConsiderApproachesInput {
                spec_id: None,
                problem: Some("p".into()),
                approaches: Some(vec![
                    ApproachItem {
                        name: "a".into(),
                        description: "d1".into(),
                        pros: vec!["p1".into()],
                        cons: vec![],
                        score: Some(7.0),
                    },
                    ApproachItem {
                        name: "b".into(),
                        description: "d2".into(),
                        pros: vec![],
                        cons: vec!["c1".into()],
                        score: Some(4.0),
                    },
                ]),
                chosen_index: Some(0),
            },
        );
        let out = r.unwrap();
        assert!(out.success);
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM approaches WHERE chosen = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
