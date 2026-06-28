//! Attention notes — persistent, tenant-scoped sticky reminders for agents.

use crate::db::Database;
use crate::{EngError, Result};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionNote {
    pub id: i64,
    pub content: String,
    pub priority: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteRequest {
    pub content: String,
    pub priority: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: Option<String>,
    pub priority: Option<i64>,
}

#[tracing::instrument(skip(db, req), fields(priority = ?req.priority))]
pub async fn create_note(db: &Database, req: CreateNoteRequest, user_id: i64) -> Result<AttentionNote> {
    let priority = req.priority.unwrap_or(5);
    let id = db
        .write(move |conn| {
            conn.execute(
                "INSERT INTO attention_notes (content, priority, user_id) VALUES (?1, ?2, ?3)",
                params![req.content, priority, user_id],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .await?;

    get_note(db, id, user_id).await
}

#[tracing::instrument(skip(db), fields(limit))]
pub async fn list_notes(db: &Database, user_id: i64, limit: i64) -> Result<Vec<AttentionNote>> {
    db.read(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content, priority, created_at, updated_at
             FROM attention_notes
             WHERE user_id = ?1
             ORDER BY priority DESC, created_at ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![user_id, limit], row_to_note)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    })
    .await
}

#[tracing::instrument(skip(db))]
pub async fn get_note(db: &Database, id: i64, user_id: i64) -> Result<AttentionNote> {
    db.read(move |conn| {
        conn.query_row(
            "SELECT id, content, priority, created_at, updated_at
             FROM attention_notes WHERE id = ?1 AND user_id = ?2",
            params![id, user_id],
            |row| row_to_note(row),
        )
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                EngError::NotFound(format!("attention note {id} not found"))
            } else {
                e.into()
            }
        })
    })
    .await
}

#[tracing::instrument(skip(db, req))]
pub async fn update_note(db: &Database, id: i64, req: UpdateNoteRequest, user_id: i64) -> Result<AttentionNote> {
    db.write(move |conn| {
        let mut sets = vec!["updated_at = datetime('now')".to_string()];
        let mut vals: Vec<rusqlite::types::Value> = Vec::new();
        let mut idx = 1usize;

        if let Some(c) = &req.content {
            sets.push(format!("content = ?{idx}"));
            vals.push(rusqlite::types::Value::Text(c.clone()));
            idx += 1;
        }
        if let Some(p) = req.priority {
            sets.push(format!("priority = ?{idx}"));
            vals.push(rusqlite::types::Value::Integer(p));
            idx += 1;
        }

        let id_idx = idx;
        vals.push(rusqlite::types::Value::Integer(id));
        let user_idx = idx + 1;
        vals.push(rusqlite::types::Value::Integer(user_id));

        let sql = format!(
            "UPDATE attention_notes SET {} WHERE id = ?{id_idx} AND user_id = ?{user_idx}",
            sets.join(", ")
        );
        let changed = conn
            .execute(&sql, rusqlite::params_from_iter(vals.iter().cloned()))
            .map_err(EngError::Database)?;
        if changed == 0 {
            return Err(EngError::NotFound(format!("attention note {id} not found")));
        }
        Ok(())
    })
    .await?;

    get_note(db, id, user_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete_note(db: &Database, id: i64, user_id: i64) -> Result<()> {
    db.write(move |conn| {
        let changed = conn
            .execute(
                "DELETE FROM attention_notes WHERE id = ?1 AND user_id = ?2",
                params![id, user_id],
            )
            .map_err(EngError::Database)?;
        if changed == 0 {
            return Err(EngError::NotFound(format!("attention note {id} not found")));
        }
        Ok(())
    })
    .await
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttentionNote> {
    Ok(AttentionNote {
        id: row.get(0)?,
        content: row.get(1)?,
        priority: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}
