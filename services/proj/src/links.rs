//! FR-PROJ-001 §1 #9 — bidirectional symmetric link writer.

use crate::audit;
use crate::errors::{IssueError, IssueResult};
use crate::types::{Actor, LinkType};
use sqlx::PgPool;
use uuid::Uuid;

/// Insert a link from `issue_id` → `linked_to_id`. If the type has an
/// inverse (per `LinkType::inverse`), insert the inverse row too in the
/// same transaction.
///
/// Returns `(forward_inserted, inverse_inserted)` so callers can audit
/// the operation. The audit row from `audit::issue_linked` carries the
/// forward direction.
pub async fn create_link(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    linked_to_id: Uuid,
    link_type: LinkType,
) -> IssueResult<(bool, bool)> {
    if issue_id == linked_to_id {
        return Err(IssueError::SelfLink);
    }

    let mut tx = db.begin().await?;
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(actor.tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let forward = sqlx::query(
        "INSERT INTO issue_links (issue_id, linked_to_id, link_type)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(issue_id)
    .bind(linked_to_id)
    .bind(link_type.as_str())
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let inverse_inserted = if let Some(inv) = link_type.inverse() {
        let r = sqlx::query(
            "INSERT INTO issue_links (issue_id, linked_to_id, link_type)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(linked_to_id)
        .bind(issue_id)
        .bind(inv.as_str())
        .execute(&mut *tx)
        .await?
        .rows_affected();
        r > 0
    } else {
        false
    };

    let _row = audit::issue_linked(
        actor.tenant_id,
        issue_id,
        linked_to_id,
        link_type,
        actor.subject_id,
    );
    // memory write transport binds in the binary; here we surface the row
    // shape via a logged event for slice 1.
    tracing::info!(
        kind = "proj.issue_linked",
        issue_id = %issue_id,
        linked_to_id = %linked_to_id,
        link_type = link_type.as_str(),
        "audit row constructed"
    );

    tx.commit().await?;

    Ok((forward > 0, inverse_inserted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn self_link_is_rejected_at_validation_layer() {
        // No DB required — the self-link branch returns before opening tx.
        let pool: Result<PgPool, _> = PgPool::connect_lazy("postgres://invalid/invalid");
        let pool = pool.expect("lazy connect should not actually connect");
        let actor = Actor {
            tenant_id: Uuid::new_v4(),
            subject_id: Uuid::new_v4(),
        };
        let id = Uuid::new_v4();
        let r = create_link(&pool, actor, id, id, LinkType::Blocks).await;
        match r {
            Err(IssueError::SelfLink) => {}
            other => panic!("expected SelfLink, got {other:?}"),
        }
    }
}
