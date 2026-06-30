//! FR-EVAL-002 human-curated authoring (§1 #2 #3 #4 #5, §1 #12). The create / add-item flow a human (the
//! founder or a designated rubric admin, gated by FR-EVAL-001 in the handler) uses to build a draft version.
//!
//! Authoring is the ONLY path that puts a row into the rubric in this slice, and it is human-only: every
//! item is validated by [`super::model::validate_item`] before insert, so an uncited item (§1 #2), a missing
//! Vietnamese title (§1 #5), an obligation with no obligation_kind (§1 #3), or a check_params shape that
//! does not match its check_type (§1 #4) is rejected at write time. Nothing here scores a person or calls a
//! model. The GENIE/Lumi draft path (DEC-2602) - where the assistant proposes items into `draft` - needs the
//! AI gateway and is a deliberately separate, later slice; see [`super::draft_genie`].
//!
//! Items may only be added to a `draft` version; the database's immutability trigger refuses an insert into
//! a published/superseded version's item set, and this layer rejects it early with a clean error. Every
//! authoring mutation emits a hash-chained audit row (§1 #11) on the memory chain, best-effort after commit,
//! the same contract `crate::handlers` uses.

use uuid::Uuid;

use crate::audit;
use crate::db::{self, Pool};

use super::model::{validate_item, Rubric, RubricError, RubricItem, RubricItemDraft, VersionState};

/// Create a named rubric framework for a tenant (§1 #1). Idempotent on `(tenant_id, name)` is NOT applied -
/// a duplicate name is a unique-violation the handler maps to a 409 - because a rubric is a deliberate,
/// named artifact. Emits `eval.rubric_drafted` (framework creation is the first step of curation).
pub async fn create_rubric(
    pool: &Pool,
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    actor_subject_id: Uuid,
    name: &str,
) -> Result<Rubric, RubricError> {
    let name = name.trim();
    let mut tx = db::tenant_tx(pool, &tenant_id).await?;
    let rubric: Rubric = sqlx::query_as(
        "INSERT INTO rubric (tenant_id, name, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, tenant_id, name, created_at, created_by",
    )
    .bind(tenant_id)
    .bind(name)
    .bind(actor_subject_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    audit::emit_governance(
        audit_pool,
        tenant_id,
        actor_subject_id,
        audit::kind::RUBRIC_DRAFTED,
        serde_json::json!({
            "rubric_id": rubric.id,
            "name": rubric.name,
            "actor_subject_id": actor_subject_id,
        }),
    )
    .await;
    Ok(rubric)
}

/// Add a human-authored, clause-cited item to a DRAFT version (§1 #2). The flow:
///   1. validate the draft ([`validate_item`]) - uncited / missing-vi / bad obligation_kind / bad check
///      shape are rejected here, before any write (§1 #2 #3 #4 #5);
///   2. confirm the target version is `draft` (an item cannot be added to a published version, §1 #6);
///   3. insert the item, authored_by='human';
///   4. emit `eval.rubric_drafted` with the citation (§1 #11).
///
/// `weight` defaults to 0 when the draft omits it (relative within a version; the roll-up is FR-EVAL-003's).
pub async fn add_item(
    pool: &Pool,
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    actor_subject_id: Uuid,
    version_id: Uuid,
    draft: &RubricItemDraft,
) -> Result<RubricItem, RubricError> {
    // §1 #2 #3 #4 #5 - validate before any write.
    validate_item(draft)?;

    let mut tx = db::tenant_tx(pool, &tenant_id).await?;

    // §1 #6 - items may only be added to a draft version. (The trigger also enforces this for
    // published/superseded versions; this gives a clean typed error instead of a trigger exception.)
    let state: Option<VersionState> =
        sqlx::query_scalar("SELECT state FROM rubric_version WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&mut *tx)
            .await?;
    match state {
        None => {
            let _ = tx.commit().await;
            return Err(RubricError::NotFound);
        }
        Some(VersionState::Draft) => {}
        Some(_) => {
            let _ = tx.commit().await;
            return Err(RubricError::NotMutable);
        }
    }

    let weight = draft.weight.unwrap_or(rust_decimal::Decimal::ZERO);
    let item: RubricItem = sqlx::query_as(
        "INSERT INTO rubric_item
            (rubric_version_id, tenant_id, source_doc, clause_ref, source_quote_vi, source_quote_en,
             item_kind, obligation_kind, check_type, check_params, weight,
             title_vi, title_en, description_vi, description_en, authored_by, needs_clause_ref)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 'human', FALSE)
         RETURNING id, rubric_version_id, tenant_id, source_doc, clause_ref, source_quote_vi,
                   source_quote_en, item_kind, obligation_kind, check_type, check_params, weight,
                   title_vi, title_en, description_vi, description_en, authored_by, genie_confidence,
                   needs_clause_ref, edited_by_subject_id",
    )
    .bind(version_id)
    .bind(tenant_id)
    .bind(draft.source_doc)
    .bind(draft.clause_ref.trim())
    .bind(draft.source_quote_vi.as_deref())
    .bind(draft.source_quote_en.as_deref())
    .bind(draft.item_kind)
    .bind(draft.obligation_kind)
    .bind(draft.check_type)
    .bind(&draft.check_params)
    .bind(weight)
    .bind(draft.title_vi.trim())
    .bind(draft.title_en.as_deref())
    .bind(draft.description_vi.as_deref())
    .bind(draft.description_en.as_deref())
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    audit::emit_governance(
        audit_pool,
        tenant_id,
        actor_subject_id,
        audit::kind::RUBRIC_DRAFTED,
        serde_json::json!({
            "rubric_version_id": version_id,
            "item_id": item.id,
            "authored_by": "human",
            "source_doc": item.source_doc.as_str(),
            "clause_ref": item.clause_ref,
            "actor_subject_id": actor_subject_id,
        }),
    )
    .await;
    Ok(item)
}

/// List the items of a version (§1 #12, the read side of authoring). Tenant-scoped by RLS. FR-EVAL-003 reads
/// only the items of a PUBLISHED version (via [`super::versioning::resolve_effective`] then this); a draft
/// version's items are visible here to the authoring surface but are not yet operative.
pub async fn list_items(
    pool: &Pool,
    tenant_id: Uuid,
    version_id: Uuid,
) -> Result<Vec<RubricItem>, RubricError> {
    let mut tx = db::tenant_tx(pool, &tenant_id).await?;
    let items: Vec<RubricItem> = sqlx::query_as(
        "SELECT id, rubric_version_id, tenant_id, source_doc, clause_ref, source_quote_vi,
                source_quote_en, item_kind, obligation_kind, check_type, check_params, weight,
                title_vi, title_en, description_vi, description_en, authored_by, genie_confidence,
                needs_clause_ref, edited_by_subject_id
           FROM rubric_item WHERE rubric_version_id = $1
          ORDER BY created_at",
    )
    .bind(version_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(items)
}
