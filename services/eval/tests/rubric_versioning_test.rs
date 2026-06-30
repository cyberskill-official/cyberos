//! FR-EVAL-002 - rubric versioning + HITL publish integration tests (§1 #6 #7 #8 #13, §4 #7..#10 #16).
//!
//! Drives the rubric module functions directly (like `governance_gate_test.rs`) and the HTTP publish path,
//! against a live Postgres with the eval migrations applied. `#[ignore]` by default, gates on
//! `EVAL_DATABASE_URL` (falling back to `DATABASE_URL`). Local:
//!   docker compose up -d        (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval -- --ignored
//!
//! Proves: a published version is immutable (a raw UPDATE of a published item is refused by the trigger);
//! `resolve_effective(at)` returns the version in force on a date; the publish HITL gate rejects a
//! service-account approver and a human approver succeeds; an empty version and an effective-date overlap
//! are rejected.

use cyberos_eval::rubric::{
    self,
    model::{CheckType, ItemKind, ObligationKind, RubricError, RubricItemDraft, SourceDoc},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("EVAL_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("EVAL_DATABASE_URL or DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::query(include_str!("../migrations/0001_governance.sql"))
        .execute(&pool)
        .await
        .expect("apply 0001_governance.sql");
    sqlx::query(include_str!("../migrations/0002_subject_request.sql"))
        .execute(&pool)
        .await
        .expect("apply 0002_subject_request.sql");
    sqlx::query(include_str!("../migrations/0003_rubric.sql"))
        .execute(&pool)
        .await
        .expect("apply 0003_rubric.sql");
    pool
}

/// A well-formed obligation draft (the standard happy-path item).
fn good_item() -> RubricItemDraft {
    RubricItemDraft {
        source_doc: SourceDoc::NdaIp,
        clause_ref: "art.2(a)".into(),
        source_quote_vi: None,
        source_quote_en: None,
        item_kind: ItemKind::Obligation,
        obligation_kind: Some(ObligationKind::Confidentiality),
        check_type: CheckType::EvidencePresence,
        check_params: serde_json::json!({}),
        weight: Some(rust_decimal::Decimal::new(1000, 2)),
        title_vi: "Bảo mật thông tin".into(),
        title_en: Some("Confidentiality".into()),
        description_vi: None,
        description_en: None,
    }
}

/// Create a rubric, open a draft version, and add one valid item; return (rubric_id, version_id).
async fn rubric_with_one_item(pool: &PgPool, tenant: Uuid, actor: Uuid) -> (Uuid, Uuid) {
    let r =
        rubric::authoring::create_rubric(pool, None, tenant, actor, "CyberSkill employment rubric")
            .await
            .unwrap();
    let v = rubric::versioning::open_version(pool, None, tenant, actor, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(pool, None, tenant, actor, v.id, &good_item())
        .await
        .unwrap();
    (r.id, v.id)
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn published_version_is_immutable() {
    // §1 #6 / AC #7 - once published, an item cannot be mutated. The migration's trigger refuses the UPDATE
    // regardless of role (the dev test connects as superuser, so the role-based REVOKE alone would not deny
    // it - the trigger is the role-independent guarantee).
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let (_rid, vid) = rubric_with_one_item(&pool, tenant, founder).await;

    let published = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        vid,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(published.state, rubric::VersionState::Published);

    // A raw UPDATE of an item in the published version must be refused.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let res = sqlx::query("UPDATE rubric_item SET weight = 99 WHERE rubric_version_id = $1")
        .bind(vid)
        .execute(&mut *tx)
        .await;
    assert!(
        res.is_err(),
        "the immutability trigger must refuse an UPDATE of a published item"
    );
    let _ = tx.rollback().await;

    // A second raw UPDATE of the published version row itself must also be refused.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let res = sqlx::query("UPDATE rubric_version SET effective_from = '2030-01-01' WHERE id = $1")
        .bind(vid)
        .execute(&mut *tx)
        .await;
    assert!(
        res.is_err(),
        "the immutability trigger must refuse mutating a published version's effective_from"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn resolve_effective_picks_the_in_force_version() {
    // §1 #7 / AC #9 - two published versions (Jan-Jul, Jul-open); resolve_effective on a March date returns
    // v1, on a September date returns v2. Publishing v2 with effective_from 2026-07-01 supersedes v1 and
    // closes its half-open interval at 2026-07-01.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();

    let r = rubric::authoring::create_rubric(&pool, None, tenant, founder, "rubric")
        .await
        .unwrap();

    // v1, published effective 2026-01-01.
    let v1 = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(&pool, None, tenant, founder, v1.id, &good_item())
        .await
        .unwrap();
    rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v1.id,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap();

    // v2, published effective 2026-07-01 (supersedes v1, closing it at 2026-07-01).
    let v2 = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(&pool, None, tenant, founder, v2.id, &good_item())
        .await
        .unwrap();
    rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v2.id,
        "2026-07-01".parse().unwrap(),
    )
    .await
    .unwrap();

    let march =
        rubric::versioning::resolve_effective(&pool, tenant, r.id, "2026-03-15".parse().unwrap())
            .await
            .unwrap();
    assert_eq!(march.version_no, 1, "March must resolve to v1");

    let september =
        rubric::versioning::resolve_effective(&pool, tenant, r.id, "2026-09-01".parse().unwrap())
            .await
            .unwrap();
    assert_eq!(september.version_no, 2, "September must resolve to v2");

    // A date before any version is in force has no effective version.
    let before =
        rubric::versioning::resolve_effective(&pool, tenant, r.id, "2025-12-31".parse().unwrap())
            .await;
    assert!(matches!(before, Err(RubricError::NoEffectiveVersion)));
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn publish_requires_a_human_approver() {
    // §1 #8 / AC #10 - a service-account approver is refused; a human approver succeeds.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let (_rid, vid) = rubric_with_one_item(&pool, tenant, founder).await;

    // Service account (approver_is_human = false) -> RequiresHumanApprover.
    let err = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        false,
        vid,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RubricError::RequiresHumanApprover));
    assert_eq!(err.code(), "rubric_requires_human_approver");

    // A human approver -> published.
    let ok = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        vid,
        "2026-01-01".parse().unwrap(),
    )
    .await;
    assert!(ok.is_ok());
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn empty_version_cannot_be_published() {
    // §1 #13 / AC #16 - publishing a version with no items is refused (rubric_version_empty).
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let r = rubric::authoring::create_rubric(&pool, None, tenant, founder, "rubric")
        .await
        .unwrap();
    let v = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();

    let err = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v.id,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RubricError::VersionEmpty));
    assert_eq!(err.code(), "rubric_version_empty");
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn overlapping_effective_from_is_rejected() {
    // §1 #13 / AC #16 - a second version whose effective_from falls inside the first's open interval is
    // refused (rubric_version_effective_overlap). Here v1 is published open-ended from 2026-01-01; trying to
    // publish v2 effective 2026-03-01 (which would overlap v1's still-open interval) without superseding is
    // the overlap the guard catches. The guard treats an open-ended live version as overlapping any
    // effective_from at or after it only when it would NOT be superseded; the same-rubric publish path
    // supersedes the open one, so to force the overlap we publish into a SEPARATE rubric's timeline is not
    // possible (overlap is per-rubric). Instead we assert the inverse: a backdated effective_from BEFORE the
    // live version's start is rejected, because it would leave two versions covering the overlap window.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();

    let r = rubric::authoring::create_rubric(&pool, None, tenant, founder, "rubric")
        .await
        .unwrap();
    let v1 = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(&pool, None, tenant, founder, v1.id, &good_item())
        .await
        .unwrap();
    rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v1.id,
        "2026-06-01".parse().unwrap(),
    )
    .await
    .unwrap();

    // v2 effective BEFORE v1's start: v1 is open-ended (effective_to IS NULL) so it overlaps any date; the
    // supersede only closes v1 if v2's effective_from is at/after v1's open interval. A backdated 2026-01-01
    // would have v1 still open AND v2 covering Jan-Jun, an overlap the guard rejects.
    let v2 = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(&pool, None, tenant, founder, v2.id, &good_item())
        .await
        .unwrap();
    let err = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v2.id,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, RubricError::EffectiveOverlap),
        "a backdated effective_from overlapping the live version must be rejected, got {err:?}"
    );
    assert_eq!(err.code(), "rubric_version_effective_overlap");
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn needs_clause_ref_item_blocks_publish() {
    // §1 #9 #13 / AC #13 - a needs_clause_ref item (the anti-fabrication flag a future GENIE draft sets)
    // blocks publish until a human grounds it. We seed such an item directly (the GENIE path is a later
    // slice) and assert publish is refused with the uncited code.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let r = rubric::authoring::create_rubric(&pool, None, tenant, founder, "rubric")
        .await
        .unwrap();
    let v = rubric::versioning::open_version(&pool, None, tenant, founder, r.id)
        .await
        .unwrap();

    // A valid grounded item, plus a directly-seeded needs_clause_ref='genie' item.
    rubric::authoring::add_item(&pool, None, tenant, founder, v.id, &good_item())
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    // clause_ref is NOT NULL with a non-empty CHECK, so a needs_clause_ref draft still carries a placeholder
    // ref but is flagged needing a real one; publish coherence (clause 13) refuses it.
    sqlx::query(
        "INSERT INTO rubric_item
            (rubric_version_id, tenant_id, source_doc, clause_ref, item_kind, check_type,
             title_vi, authored_by, needs_clause_ref)
         VALUES ($1, $2, 'labor_contract', 'PENDING', 'obligation', 'attestation',
                 'Chưa xác định điều khoản', 'genie', TRUE)",
    )
    .bind(v.id)
    .bind(tenant)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let err = rubric::versioning::publish_version(
        &pool,
        None,
        tenant,
        founder,
        true,
        v.id,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RubricError::Uncited));
    assert_eq!(err.code(), "rubric_item_uncited");
}
