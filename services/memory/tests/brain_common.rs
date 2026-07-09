//! Shared test harness for the FR-MEMORY-123 BRAIN integration tests.
//!
//! Mirrors the memory integration-test convention (tests/interaction_event_test.rs): the tests require a
//! live Postgres WITH pgvector, with the memory migrations applied, so each test is `#[ignore]` and gates on
//! `DATABASE_URL`. CI boots services/dev/docker-compose.yml (pgvector image) and runs `--ignored`. Local:
//!   docker compose up -d            (in services/dev/)
//!   psql ... -c 'CREATE EXTENSION IF NOT EXISTS vector;'
//!   DATABASE_URL=postgres://... cargo test -p cyberos-memory --test brain_ingest_test -- --ignored
//!
//! The harness applies migrations 0003-0008 idempotently (0006-0008 are new and may not be in a given dev
//! DB), seeds interaction-event rows on l1_audit_log through the canonical emit path (so the chain anchors
//! are real), and drives the brain passes with the STUB embed client (deterministic vectors; no live
//! ai-gateway needed). It cleans up its tenant's rows at the end.

#![allow(dead_code)]

use cyberos_memory::brain::{self, BrainConfig, Caller, EmbedClient};
use cyberos_memory::interaction::{
    emit, AllowAll, ContentRef, EmitOutcome, EventClass, InteractionEvent, Module, SourceChannel,
    TargetRef,
};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// One seeded event's coordinates, returned by `append_interaction_event` for assertions.
#[derive(Clone, Debug)]
pub struct SeededEvent {
    pub source_seq: i64,
    pub audit_row_id: String,
    pub subject_id: Uuid,
}

/// The brain test environment: a pool, a fixed tenant, two subjects (alice + bob), and the stub embed
/// client. Cleans its tenant on `cleanup()`.
pub struct BrainTestEnv {
    pub pool: PgPool,
    pub tenant: Uuid,
    pub alice: Uuid,
    pub bob: Uuid,
    pub gw: EmbedClient,
}

impl BrainTestEnv {
    /// Connect, apply the brain migrations idempotently, and seed the FR-EVAL-001 access grants so the
    /// default caller (alice, self) and a founder can be constructed. A fresh random tenant per test.
    pub async fn new() -> Self {
        let url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("MEMORY_DATABASE_URL"))
            .expect("DATABASE_URL env var");
        let pool = PgPool::connect(&url).await.expect("connect");

        // Apply the schema ONCE per test process (MEM-059). The migration files create functions via
        // CREATE OR REPLACE FUNCTION, so parallel test threads each applying them raced on pg_proc
        // ("duplicate key ... pg_proc_proname_args_nsp_index"); a process-global OnceCell serialises the
        // apply to exactly one run. Migrations remain idempotent (IF NOT EXISTS / DROP POLICY IF EXISTS).
        ensure_schema(&pool).await;

        Self {
            pool,
            tenant: Uuid::new_v4(),
            alice: Uuid::new_v4(),
            bob: Uuid::new_v4(),
            gw: EmbedClient::stub(),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    pub fn tenant(&self) -> Uuid {
        self.tenant
    }
    pub fn subject_alice(&self) -> Uuid {
        self.alice
    }
    pub fn subject_bob(&self) -> Uuid {
        self.bob
    }
    pub fn gw(&self) -> &EmbedClient {
        &self.gw
    }
    pub fn cfg(&self) -> BrainConfig {
        BrainConfig::default()
    }

    /// A caller entitled to exactly the listed subjects via `manager_of` grants (plus their own self path).
    /// The caller's own subject is a fresh random viewer so the grants are the only cross-subject access.
    pub async fn caller_entitled_to(&self, subjects: &[Uuid]) -> Caller {
        let viewer = Uuid::new_v4();
        for s in subjects {
            self.grant(viewer, *s, "manager_of").await;
        }
        Caller {
            tenant_id: self.tenant,
            viewer_subject_id: viewer,
        }
    }

    /// A founder caller (may see anyone in the tenant).
    pub async fn founder_caller(&self) -> Caller {
        let viewer = Uuid::new_v4();
        self.grant(viewer, Uuid::nil(), "founder").await;
        Caller {
            tenant_id: self.tenant,
            viewer_subject_id: viewer,
        }
    }

    /// Insert an access grant row (founder grants ignore the target; manager_of/self bind the pair).
    pub async fn grant(&self, viewer: Uuid, target: Uuid, scope: &str) {
        sqlx::query(
            "INSERT INTO access_grant (tenant_id, viewer_subject_id, target_subject_id, scope, granted_by)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(self.tenant)
        .bind(viewer)
        .bind(target)
        .bind(scope)
        .bind(viewer)
        .execute(&self.pool)
        .await
        .expect("insert access_grant");
    }

    /// Append an interaction event for `subject` with `body` text, at the current time. Returns its seq +
    /// audit_row_id. The event chains as a real l1_audit_log row via the canonical emit path (AllowAll gate
    /// stands in for an acknowledged subject), so its chain anchor verifies under the read-time check.
    pub async fn append_interaction_event(
        &self,
        subject: Uuid,
        verb: &str,
        body_text: &str,
    ) -> SeededEvent {
        self.append_interaction_event_at(subject, verb, body_text, now_ns(), None)
            .await
    }

    /// Append an interaction event at a specific `occurred_at_ns` (for tiering by age) and optional channel.
    /// The `body_text` is stuffed into an attribute so the embedded body differs per call (the canonical
    /// body includes the attributes), giving the stub embedder distinct vectors per distinct text.
    pub async fn append_interaction_event_at(
        &self,
        subject: Uuid,
        verb: &str,
        body_text: &str,
        occurred_at_ns: i64,
        channel: Option<Uuid>,
    ) -> SeededEvent {
        let mut b = InteractionEvent::builder(Module::Chat, verb, EventClass::Content)
            .tenant(self.tenant)
            .subject(subject)
            .occurred_at_ns(occurred_at_ns)
            .content(ContentRef::None)
            .source(SourceChannel::Web)
            .attribute("text", serde_json::json!(body_text));
        if let Some(ch) = channel {
            b = b.target(TargetRef::Channel { id: ch.to_string() });
        }
        let ev = b.build().expect("valid event");
        let EmitOutcome::Recorded { seq } = emit(&self.pool, &ev, &AllowAll).await.expect("emit")
        else {
            panic!("expected Recorded");
        };
        SeededEvent {
            source_seq: seq,
            audit_row_id: brain::BrainEvent::make_audit_row_id(self.tenant, seq),
            subject_id: subject,
        }
    }

    /// Run one brain ingest pass for the tenant (consume + embed + UPSERT).
    pub async fn run_ingest_once(&self) {
        brain::ingest_worker::ingest_one_tenant(self.tenant, &self.pool, &self.gw)
            .await
            .expect("ingest");
    }

    /// Run the summarise pass + force a subject re-summarise so a summary exists for assertions.
    pub async fn run_summarize_once(
        &self,
        scope_kind: &str,
        scope_id: &str,
        subject: Option<Uuid>,
    ) {
        brain::summarize::resummarize_now(
            &self.pool,
            self.tenant,
            scope_kind,
            scope_id,
            subject,
            &self.gw,
        )
        .await
        .expect("resummarize");
        brain::summarize::run_summary_pass(&self.pool, self.tenant)
            .await
            .expect("summary pass");
    }

    /// Run one tiering pass.
    pub async fn run_tiering_pass(&self) {
        brain::tiering::run_tier_pass(&self.pool, self.tenant, &self.cfg())
            .await
            .expect("tier pass");
    }

    /// The current tier counts for the tenant.
    pub async fn tier_counts(&self) -> brain::tiering::TierCounts {
        brain::tiering::tier_counts(&self.pool, self.tenant)
            .await
            .expect("tier counts")
    }

    /// The current chain HEAD seq for the tenant (max seq), for the read-only invariant assertion.
    pub async fn audit_head(&self) -> i64 {
        sqlx::query_scalar("SELECT COALESCE(MAX(seq), 0) FROM l1_audit_log WHERE tenant_id = $1")
            .bind(self.tenant)
            .fetch_one(&self.pool)
            .await
            .expect("audit head")
    }

    /// Corrupt a Layer-1 row's body so its read-time anchor recompute no longer matches what it advertises
    /// — simulates a tamper for the chain-anchor-mismatch test. Mutates only the body (the stored
    /// chain_anchor_hex stays the old value, so recompute != advertised).
    pub async fn corrupt_layer1_row(&self, seq: i64, new_body: &str) {
        sqlx::query("UPDATE l1_audit_log SET body = $1 WHERE tenant_id = $2 AND seq = $3")
            .bind(new_body)
            .bind(self.tenant)
            .bind(seq)
            .execute(&self.pool)
            .await
            .expect("corrupt row");
    }

    /// Count brain embedding rows for the tenant in a given tier.
    pub async fn embedding_count(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM brain_event_embedding WHERE tenant_id = $1")
            .bind(self.tenant)
            .fetch_one(&self.pool)
            .await
            .expect("count")
    }

    /// Read one embedding row's tier + audit_row_id by seq.
    pub async fn embedding_row(&self, seq: i64) -> Option<(String, String, Option<String>)> {
        sqlx::query_as(
            "SELECT tier, audit_row_id, embed_state::text FROM brain_event_embedding
              WHERE tenant_id = $1 AND source_seq = $2",
        )
        .bind(self.tenant)
        .bind(seq)
        .fetch_optional(&self.pool)
        .await
        .expect("embedding row")
    }

    /// Delete every row this env created (brain tables + chain), keeping the shared dev DB clean.
    pub async fn cleanup(&self) {
        for sql in [
            "DELETE FROM brain_event_embedding WHERE tenant_id = $1",
            "DELETE FROM brain_summary WHERE tenant_id = $1",
            "DELETE FROM brain_ingest_cursor WHERE tenant_id = $1",
            "DELETE FROM brain_tier_watermark WHERE tenant_id = $1",
            "DELETE FROM access_grant WHERE tenant_id = $1",
            "DELETE FROM l1_audit_log WHERE tenant_id = $1",
        ] {
            sqlx::query(sql)
                .bind(self.tenant)
                .execute(&self.pool)
                .await
                .ok();
        }
    }
}

/// Current ns since the Unix epoch.
pub fn now_ns() -> i64 {
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
}

/// Ns offset N days in the past from now (for seeding aged events).
pub fn days_ago_ns(days: i64) -> i64 {
    now_ns() - days * 86_400 * 1_000_000_000
}

/// Build a recall query from a string with the given drill flag.
pub fn query(q: &str) -> brain::RecallQuery {
    serde_json::from_value(serde_json::json!({ "q": q })).expect("query")
}

/// Apply the full brain-test schema exactly once per test process (MEM-059). Guarded by a process-global
/// `OnceCell` so parallel `#[tokio::test]` threads do not race on `CREATE OR REPLACE FUNCTION` (pg_proc). The
/// migration set includes 0009 (fail-closed RLS, MEM-002) so the DB-backed tests run against the SHIPPED
/// schema; superuser bypasses RLS, so they still pass (enforcement is proven in tests/brain_rls_test.rs).
async fn ensure_schema(pool: &PgPool) {
    static MIGRATED: OnceCell<()> = OnceCell::const_new();
    MIGRATED
        .get_or_init(|| async {
            for sql in [
                include_str!("../migrations/0003_layer1_audit_log.sql"),
                include_str!("../migrations/0004_l1_event_type.sql"),
                include_str!("../migrations/0005_interaction_event.sql"),
                include_str!("../migrations/0006_brain_event_embeddings.sql"),
                include_str!("../migrations/0007_brain_summaries.sql"),
                include_str!("../migrations/0008_brain_tier_cursor.sql"),
                include_str!("../migrations/0009_rls_fail_closed.sql"),
                ACCESS_GRANT_DDL,
            ] {
                apply_lenient(pool, sql).await;
            }
        })
        .await;
}

/// Apply a migration leniently (swallow "already exists" so re-runs against a migrated DB are no-ops). Uses
/// `raw_sql` (simple query protocol) because the migration files are multi-statement — `query` (extended
/// protocol) rejects them with "cannot insert multiple commands into a prepared statement". The brain
/// migrations use IF NOT EXISTS / DROP POLICY IF EXISTS, so re-application is clean; this still guards the
/// non-idempotent generated-column add in 0004.
async fn apply_lenient(pool: &PgPool, sql: &str) {
    if let Err(e) = sqlx::raw_sql(sql).execute(pool).await {
        let msg = e.to_string();
        if !msg.contains("already exists") {
            panic!("migration failed: {msg}");
        }
    }
}

/// The FR-EVAL-001 `access_grant` table DDL (a self-contained copy of the eval governance migration's grant
/// table + its lookup index), applied leniently so the brain tests do not depend on the eval crate's
/// migration files being present in this DB. Matches services/eval/migrations/0001_governance.sql exactly.
const ACCESS_GRANT_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS access_grant (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id          UUID NOT NULL,
    viewer_subject_id  UUID NOT NULL,
    target_subject_id  UUID NOT NULL,
    scope              TEXT NOT NULL CHECK (scope IN ('founder','manager_of','self')),
    granted_by         UUID NOT NULL,
    granted_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS access_grant_lookup_idx
    ON access_grant (tenant_id, viewer_subject_id, target_subject_id)
    WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS access_grant_target_idx
    ON access_grant (tenant_id, target_subject_id);
"#;
