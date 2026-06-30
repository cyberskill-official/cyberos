//! FR-EVAL-002 - rubric tenant-isolation (RLS) integration test (§1 #1, §4 #1).
//!
//! Proves the per-tenant row-level security on the rubric tables: a rubric published in tenant A is
//! invisible to a caller in tenant B. Mirrors the tenant-isolation assertion in `endpoints_test.rs` and the
//! RLS idiom of `0001_governance.sql`. `#[ignore]` by default, gates on `EVAL_DATABASE_URL` (falling back to
//! `DATABASE_URL`). Local:
//!   docker compose up -d        (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval -- --ignored

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

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn rubric_is_tenant_isolated() {
    // §4 #1 - tenant A's rubric is invisible to tenant B. Publish a version in A, then read it from B's
    // tenant scope and confirm RLS hides it (resolve_effective finds nothing; list_items returns empty).
    let pool = pool().await;
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let founder = Uuid::new_v4();

    // Build + publish a rubric in tenant A.
    let r = rubric::authoring::create_rubric(&pool, None, tenant_a, founder, "A's rubric")
        .await
        .unwrap();
    let v = rubric::versioning::open_version(&pool, None, tenant_a, founder, r.id)
        .await
        .unwrap();
    rubric::authoring::add_item(&pool, None, tenant_a, founder, v.id, &good_item())
        .await
        .unwrap();
    rubric::versioning::publish_version(
        &pool,
        None,
        tenant_a,
        founder,
        true,
        v.id,
        "2026-01-01".parse().unwrap(),
    )
    .await
    .unwrap();

    // From tenant A: the published version resolves and its item is visible.
    let eff_a =
        rubric::versioning::resolve_effective(&pool, tenant_a, r.id, "2026-03-01".parse().unwrap())
            .await
            .unwrap();
    assert_eq!(eff_a.version_no, 1);
    let items_a = rubric::authoring::list_items(&pool, tenant_a, v.id)
        .await
        .unwrap();
    assert_eq!(items_a.len(), 1, "tenant A sees its own item");

    // From tenant B: RLS hides A's rubric entirely - resolve finds no effective version, and a direct
    // item listing of A's version id returns zero rows.
    let eff_b =
        rubric::versioning::resolve_effective(&pool, tenant_b, r.id, "2026-03-01".parse().unwrap())
            .await;
    assert!(
        matches!(eff_b, Err(RubricError::NoEffectiveVersion)),
        "tenant B must not see tenant A's published version"
    );
    let items_b = rubric::authoring::list_items(&pool, tenant_b, v.id)
        .await
        .unwrap();
    assert!(
        items_b.is_empty(),
        "RLS must hide tenant A's items from tenant B"
    );
}
