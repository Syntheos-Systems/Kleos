//! Review-gate regression coverage for the non-context surfaces closed in the
//! deep-sweep remediation (cluster 1): the write-path self-approval lockdown and
//! the tag-search read path. Complements `context_review_gate.rs` (assembly) and
//! `review_gate_search.rs` (hybrid search).

use kleos_lib::db::Database;
use kleos_lib::memory;
use kleos_lib::memory::types::{StoreRequest, UpdateRequest};
use rusqlite::params;

/// Build an embedding-free store request, optionally tagged.
fn store_req(content: &str, user_id: i64, tags: Option<Vec<String>>) -> StoreRequest {
    StoreRequest {
        content: content.to_string(),
        category: "general".to_string(),
        source: "test".to_string(),
        importance: 5,
        tags,
        embedding: None,
        chunk_embeddings: None,
        session_id: None,
        is_static: Some(false),
        user_id: Some(user_id),
        space_id: None,
        parent_memory_id: None,
        sync_id: None,
        artifacts: None,
        created_at: None,
    }
}

/// Persist one owned memory and return its id.
async fn store(db: &Database, content: &str, user_id: i64, tags: Option<Vec<String>>) -> i64 {
    memory::store(db, store_req(content, user_id, tags), None, false)
        .await
        .expect("store")
        .id
}

/// Force a memory into the review-gate pending state.
async fn mark_pending(db: &Database, id: i64) {
    db.write(move |conn| {
        conn.execute(
            "UPDATE memories SET status = 'pending' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    })
    .await
    .expect("mark pending");
}

/// A partial update carrying only a client-supplied status change.
fn status_update(status: &str) -> UpdateRequest {
    UpdateRequest {
        content: None,
        category: None,
        importance: None,
        tags: None,
        is_static: None,
        status: Some(status.to_string()),
        embedding: None,
        chunk_embeddings: None,
    }
}

/// The write-path lockdown: `update()` must NOT let a client flip its own pending
/// memory to approved. Approval is inbox-only. Before the fix, `req.status` was
/// applied verbatim, so `POST /memory/{id}/update {"status":"approved"}`
/// self-approved past the gate.
#[tokio::test]
async fn update_ignores_client_supplied_status() {
    let db = Database::connect_memory().await.expect("db");
    let user_id = 1;
    let id = store(&db, "an unreviewed high-importance claim", user_id, None).await;
    mark_pending(&db, id).await;

    let updated = memory::update(&db, id, status_update("approved"), user_id, false)
        .await
        .expect("update");
    assert_eq!(
        updated.status, "pending",
        "client-supplied status must be ignored; memory must stay pending"
    );

    // A content edit must still succeed and still not change status. update()
    // versions the row, so target the new latest id, not the superseded one.
    let mut content_update = status_update("approved");
    content_update.content = Some("an unreviewed high-importance claim, edited".to_string());
    let updated = memory::update(&db, updated.id, content_update, user_id, false)
        .await
        .expect("update");
    assert_eq!(
        updated.status, "pending",
        "status must remain pending across edits"
    );
}

/// Tag search must withhold pending memories: `search_by_tags` previously lacked
/// both `status != 'pending'` and `is_archived = 0`.
#[tokio::test]
async fn search_by_tags_withholds_pending() {
    let db = Database::connect_memory().await.expect("db");
    let user_id = 1;
    let tag = vec!["deploy".to_string()];
    let approved = store(
        &db,
        "approved deploy note about harbor",
        user_id,
        Some(tag.clone()),
    )
    .await;
    let pending = store(
        &db,
        "pending deploy note about thicket",
        user_id,
        Some(tag.clone()),
    )
    .await;
    mark_pending(&db, pending).await;

    let hits = memory::search_by_tags(&db, user_id, &tag, false, 20)
        .await
        .expect("tag search");
    let ids: Vec<i64> = hits.iter().map(|m| m.id).collect();
    assert!(
        ids.contains(&approved),
        "approved tagged memory must surface ({ids:?})"
    );
    assert!(
        !ids.contains(&pending),
        "pending tagged memory must NOT surface through tag search ({ids:?})"
    );
}
