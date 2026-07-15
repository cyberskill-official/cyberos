//! TASK-CHAT-267 — in-app content reporting, integration tests.
//!
//! Drives the REAL router in-process (`tower::ServiceExt::oneshot`), the same way
//! `services/auth/tests/admin_subject_create_test.rs` does, against a live Postgres with the chat
//! migrations applied (through 0014_chat_reports.sql). Tokens are minted HS256 — `Authenticator::
//! from_hs256_secret` is the documented test/local verifier — which lets a test act as several distinct
//! subjects across two tenants, which is what the membership (§1 #3) and isolation (§1 #9) clauses need.
//!
//! Requires Postgres; every test is `#[ignore]`, mirroring the memory and auth integration-test convention.
//! CI boots services/dev/docker-compose.yml and runs `-- --ignored`. Local:
//!
//!   cd services/dev && docker compose up -d
//!   # apply chat migrations (0001..0014) to the dev DB, then:
//!   DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos \
//!     cargo test -p cyberos-chat --test reports -- --ignored --test-threads=1
//!
//! `--test-threads=1`: the rate-limit test (AC 9) burns this subject's hourly budget, and the tests share
//! one database. Each test seeds its own tenant + subjects (fresh UUIDs), so they do not collide on data —
//! but the rate-limit counter is per-subject and the suite is cheaper to reason about run serially.
//!
//! AC coverage map (§4 of the task):
//!   AC 1,2,3   snapshot_survives_edit_and_delete
//!   AC 4       non_member_cannot_report_a_message
//!   AC 5       subject_report_needs_no_shared_channel
//!   AC 6,13,14 closed_enums_and_caps_are_enforced
//!   AC 7       duplicate_report_is_idempotent_not_conflict
//!   AC 8       resolved_report_does_not_block_a_new_one
//!   AC 9       rate_limit_fires_before_target_lookup
//!   AC 12      reports_are_tenant_isolated
//! AC 10/11 (the audit row) are asserted in `evidence_stays_in_the_report_row` below — but note the audit
//! chain only writes rows when `audit_pool` is Some; with it None the emit is a log line. The test
//! therefore asserts the negative that matters and is checkable here (no content leaves the report row),
//! and the positive (exactly one row) is covered by the chat smoke suite against a configured stack.
//! AC 15/16 (dialog keyboard + i18n) are the web tests — apps/web/src/components/__tests__/ReportDialog.test.tsx.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use cyberos_chat::{auth::Authenticator, notify::Notifier, realtime, storage, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const HS256_SECRET: &[u8] = b"cyberos-chat-test-secret-task-chat-267";

// ── Harness ──────────────────────────────────────────────────────────────────────────────────────

/// Connects, then drops to `cyberos_app` — the runtime app's database identity.
///
/// This `SET ROLE` is load-bearing, not hygiene. `DATABASE_URL` points at the `cyberos` superuser, and
/// **PostgreSQL superusers bypass row-level security entirely** — `FORCE ROW LEVEL SECURITY` does not apply
/// to them. Connect as the superuser and every tenant-isolation assertion in this file becomes vacuous: the
/// policy never runs, so the test proves nothing while looking green. `cyberos_app` is NOLOGIN and not the
/// table owner, so RLS applies to it exactly as it does in production.
///
/// Mirrors services/auth/tests/admin_subject_create_test.rs. The role and its grants come from auth's
/// 0004_rls_roles.sql (`ALTER DEFAULT PRIVILEGES` covers every table created after it, which is all of
/// chat's). We propagate the error rather than `.ok()` it: a DB where the role is missing must not silently
/// hand back superuser connections and turn the RLS tests back into no-ops.
///
/// Propagating is necessary but NOT sufficient, and it is worth being precise about why. An `Err` out of
/// `after_connect` is not reported — sqlx discards it and retries until the acquire timeout, then says
/// `PoolTimedOut`. So the failure is loud in the sense that the suite stops, and useless in the sense that
/// the message is the same one you get when Postgres is simply not running. `preflight` is what makes the
/// two distinguishable.
async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    preflight(&url).await;
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE cyberos_app").execute(conn).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect")
}

/// Probe on a plain connection so "Postgres is down" and "the cyberos_app role is missing" become two
/// different, actionable errors instead of one indistinguishable `PoolTimedOut`.
async fn preflight(url: &str) {
    use sqlx::Connection;
    let mut conn = sqlx::PgConnection::connect(url).await.unwrap_or_else(|e| {
        panic!(
            "cannot reach Postgres at DATABASE_URL: {e}\n  \
             start it with:  cd services/dev && docker compose up -d"
        )
    });
    let (has_role,): (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app')")
            .fetch_one(&mut conn)
            .await
            .expect("probe pg_roles");
    assert!(
        has_role,
        "the cyberos_app role is missing from this database. RLS would not apply and every tenant-isolation \
         assertion in this suite would be vacuously green.\n  \
         apply services/auth/migrations/0004_rls_roles.sql to it"
    );
    conn.close().await.ok();
}

fn app(pool: PgPool) -> axum::Router {
    cyberos_chat::router(AppState {
        pool,
        // None: the audit emit degrades to a log line, which is what we want here. The point of AC 10 is
        // that content does NOT reach the chain; a chain that is not wired cannot leak it either.
        audit_pool: None,
        capturer: None,
        authenticator: Arc::new(Authenticator::from_hs256_secret(HS256_SECRET)),
        hub: realtime::Hub::default(),
        presence: realtime::Presence::default(),
        notifier: Notifier::default(),
        attachments: storage::AttachmentConfig::default(),
        // TASK-CHAT-268 added this field; reports does not exercise blocking, so a cold cache.
        blocks: cyberos_chat::blocks::BlockCache::default(),
    })
}

/// An HS256 token for `subject` in `tenant`. Shape mirrors what the auth service mints (sub, tenant_id,
/// roles, exp); the chat Authenticator in HS256 mode does not validate `aud`.
fn token(subject: Uuid, tenant: Uuid) -> String {
    let claims = json!({
        "sub": subject.to_string(),
        "tenant_id": tenant.to_string(),
        "roles": ["member"],
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    });
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(HS256_SECRET),
    )
    .expect("sign")
}

async fn post_report(app: &axum::Router, tok: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/v1/chat/reports")
        .header("authorization", format!("Bearer {tok}"))
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.expect("oneshot");
    let status = res.status();
    let bytes = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    (status, body_value(&bytes))
}

/// Parse the body as JSON, and on failure keep it as a STRING rather than discarding it. `crate::internal`
/// returns error text as a plain-text body, so a 500 is not JSON; `unwrap_or(Value::Null)` would silently
/// drop the one thing a failing assertion needs to be diagnosable.
fn body_value(bytes: &[u8]) -> Value {
    if bytes.is_empty() {
        return Value::Null; // 204 and friends: genuinely no body, not an unparseable one.
    }
    serde_json::from_slice(bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes).into_owned()))
}

/// Raw-string variant, for a body that is not valid against the closed enums (AC 13) and so cannot be
/// constructed through `json!` + the typed request.
async fn post_report_raw(app: &axum::Router, tok: &str, raw: &str) -> StatusCode {
    let req = Request::builder()
        .method("POST")
        .uri("/v1/chat/reports")
        .header("authorization", format!("Bearer {tok}"))
        .header("content-type", "application/json")
        .body(Body::from(raw.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.expect("oneshot").status()
}

/// Everything one test needs: a tenant, two subjects, and (optionally) a channel with a message in it.
struct Seed {
    tenant: Uuid,
    alice: Uuid,
    bob: Uuid,
}

/// Fresh UUIDs per test, so tests sharing one database never collide on data. Nothing is written until a
/// channel or message is asked for — a subject-only test (AC 5) needs no rows at all.
fn seed() -> Seed {
    Seed {
        tenant: Uuid::new_v4(),
        alice: Uuid::new_v4(),
        bob: Uuid::new_v4(),
    }
}

impl Seed {
    /// A channel owned by `owner`, with `members` joined. Written with the tenant GUC set so RLS admits it.
    async fn channel(&self, pool: &PgPool, owner: Uuid, members: &[Uuid]) -> Uuid {
        let mut tx = tenant_tx(pool, self.tenant).await;
        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO chat_channels (tenant_id, name, created_by) VALUES ($1,$2,$3) RETURNING id",
        )
        .bind(self.tenant)
        .bind("reports-test")
        .bind(owner)
        .fetch_one(&mut *tx)
        .await
        .expect("insert channel");
        for m in members {
            sqlx::query(
                "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
                 VALUES ($1,$2,$3,'member') ON CONFLICT DO NOTHING",
            )
            .bind(id)
            .bind(self.tenant)
            .bind(m)
            .execute(&mut *tx)
            .await
            .expect("insert member");
        }
        tx.commit().await.unwrap();
        id
    }

    async fn message(&self, pool: &PgPool, channel: Uuid, sender: Uuid, body: &str) -> Uuid {
        let mut tx = tenant_tx(pool, self.tenant).await;
        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO chat_messages (tenant_id, channel_id, sender_subject_id, body)
             VALUES ($1,$2,$3,$4) RETURNING id",
        )
        .bind(self.tenant)
        .bind(channel)
        .bind(sender)
        .bind(body)
        .fetch_one(&mut *tx)
        .await
        .expect("insert message");
        tx.commit().await.unwrap();
        id
    }
}

async fn tenant_tx(pool: &PgPool, tenant: Uuid) -> sqlx::Transaction<'_, sqlx::Postgres> {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("set guc");
    tx
}

async fn count_reports(pool: &PgPool, tenant: Uuid) -> i64 {
    let mut tx = tenant_tx(pool, tenant).await;
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM chat_reports WHERE tenant_id = $1")
        .bind(tenant)
        .fetch_one(&mut *tx)
        .await
        .expect("count");
    n
}

// ── Tests ────────────────────────────────────────────────────────────────────────────────────────

/// AC 1, 2, 3 — the clause the whole task turns on (§1 #4). The sender can edit and soft-delete the message
/// after it is reported; the snapshot must still hold the ORIGINAL text and the original sender.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn snapshot_survives_edit_and_delete() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, s.alice, &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "something abusive").await;

    let (status, body) = post_report(
        &app,
        &token(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"harassment"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "report accepted: {body:?}");

    // BOB now edits and soft-deletes his own message — the exact abuse the snapshot exists to defeat.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    sqlx::query("UPDATE chat_messages SET body = $2, edited_at = now() WHERE id = $1")
        .bind(msg)
        .bind("actually something nice")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE chat_messages SET deleted_at = now() WHERE id = $1")
        .bind(msg)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (snap_body, snap_sender): (Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT snapshot_body, snapshot_sender_id FROM chat_reports WHERE target_message_id = $1",
    )
    .bind(msg)
    .fetch_one(&mut *tx)
    .await
    .expect("report row");

    assert_eq!(
        snap_body.as_deref(),
        Some("something abusive"),
        "snapshot must hold the ORIGINAL text, not the edit, and not null"
    );
    assert_eq!(
        snap_sender,
        Some(s.bob),
        "snapshot pins the original sender"
    );
}

/// AC 4 — §1 #3. A person who cannot see the content cannot report it.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn non_member_cannot_report_a_message() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    // BOB's private channel; ALICE is not in it.
    let ch = s.channel(&pool, s.bob, &[s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "hello").await;

    let (status, _) = post_report(
        &app,
        &token(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"spam"}),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(count_reports(&pool, s.tenant).await, 0, "no row written");
}

/// AC 5 — §1 #3. The DM path: harassment arrives from someone whose channels you do not share, and that is
/// exactly the case that must remain reportable.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn subject_report_needs_no_shared_channel() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    // ALICE and BOB share nothing at all — no channel is created.

    let (status, _) = post_report(
        &app,
        &token(s.alice, s.tenant),
        json!({"target_kind":"subject","target_subject_id":s.bob,"reason":"harassment"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

/// AC 7 — §1 #6. A double-tap is not an error, and a distinct "already reported" response would be an
/// oracle for a prior report.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn duplicate_report_is_idempotent_not_conflict() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, s.alice, &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "spam spam spam").await;
    let body = json!({"target_kind":"message","target_message_id":msg,"reason":"spam"});
    let tok = token(s.alice, s.tenant);

    let (s1, b1) = post_report(&app, &tok, body.clone()).await;
    let (s2, b2) = post_report(&app, &tok, body.clone()).await;

    assert_eq!(s1, StatusCode::CREATED);
    assert_eq!(s2, StatusCode::OK, "second report is 200, NOT 409");
    assert_eq!(b1["id"], b2["id"], "same report id returned");
    assert_eq!(count_reports(&pool, s.tenant).await, 1, "exactly one row");
}

/// AC 8 — the partial unique index is scoped to `status = 'open'`, so the same person can misbehave twice.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn resolved_report_does_not_block_a_new_one() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, s.alice, &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "again").await;
    let body = json!({"target_kind":"message","target_message_id":msg,"reason":"spam"});
    let tok = token(s.alice, s.tenant);

    let (s1, _) = post_report(&app, &tok, body.clone()).await;
    assert_eq!(s1, StatusCode::CREATED);

    // TASK-CHAT-269 is the only real writer of `status`; here we stand in for it.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    sqlx::query("UPDATE chat_reports SET status = 'dismissed', resolved_at = now() WHERE target_message_id = $1")
        .bind(msg)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let (s2, _) = post_report(&app, &tok, body).await;
    assert_eq!(
        s2,
        StatusCode::CREATED,
        "a dismissed report does not block a new one"
    );
    assert_eq!(count_reports(&pool, s.tenant).await, 2);
}

/// AC 9 — §1 #7. The limit is checked BEFORE the target lookup. The 21st call targets a message id that does
/// not exist: if the check ran after the lookup this would 404, and a rate-limited caller could use the
/// endpoint to tell a real message id from a fake one.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn rate_limit_fires_before_target_lookup() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let tok = token(s.alice, s.tenant);

    // Burn the budget against 20 distinct subjects (each a fresh uuid, so the open-report dedup index does
    // not swallow them and every call really writes a row).
    for i in 0..20 {
        let victim = Uuid::new_v4();
        let (status, body) = post_report(
            &app,
            &tok,
            json!({"target_kind":"subject","target_subject_id":victim,"reason":"spam"}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "report {i} accepted: {body:?}");
    }

    let (status, _) = post_report(
        &app,
        &tok,
        json!({"target_kind":"message","target_message_id":Uuid::new_v4(),"reason":"spam"}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "must be 429, NOT 404 — a 404 here leaks that the message id is unknown"
    );
}

/// AC 12 — §1 #9. A report raised in one workspace is not readable from another under any query.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn reports_are_tenant_isolated() {
    let pool = pool().await;
    let a = seed();
    let b = seed(); // a different tenant entirely
    let app = app(pool.clone());

    let (status, body) = post_report(
        &app,
        &token(a.alice, a.tenant),
        json!({"target_kind":"subject","target_subject_id":a.bob,"reason":"hate"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: Uuid = serde_json::from_value(body["id"].clone()).unwrap();

    // Read it with tenant B's GUC set. RLS is FORCEd, so even a direct SELECT * sees nothing.
    let mut tx = tenant_tx(&pool, b.tenant).await;
    let leaked: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM chat_reports WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .expect("query");
    assert!(leaked.is_none(), "tenant B must not see tenant A's report");
}

/// AC 6, 13, 14 — the closed enums and the caps. Each of these is enforced twice (handler + DB CHECK); the
/// handler's 400 is what the client sees.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn closed_enums_and_caps_are_enforced() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let tok = token(s.alice, s.tenant);

    // AC 13 — "because" is not a reason. Must be 400, not axum's default 422 for a Json rejection.
    let raw = format!(
        r#"{{"target_kind":"subject","target_subject_id":"{}","reason":"because"}}"#,
        s.bob
    );
    assert_eq!(
        post_report_raw(&app, &tok, &raw).await,
        StatusCode::BAD_REQUEST
    );

    // AC 6 — you cannot report yourself.
    let (status, _) = post_report(
        &app,
        &tok,
        json!({"target_kind":"subject","target_subject_id":s.alice,"reason":"spam"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // AC 14 — detail is capped at 1000 characters.
    let (status, _) = post_report(
        &app,
        &tok,
        json!({"target_kind":"subject","target_subject_id":s.bob,"reason":"spam",
               "detail":"x".repeat(1001)}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // A target shape that contradicts target_kind.
    let (status, _) = post_report(
        &app,
        &tok,
        json!({"target_kind":"message","target_subject_id":s.bob,"reason":"spam"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    assert_eq!(
        count_reports(&pool, s.tenant).await,
        0,
        "not one of these wrote a row"
    );
}

/// AC 10 — the report row holds the evidence; the audit payload must never carry it. Asserted here on the
/// stored row: `detail` and `snapshot_body` live in chat_reports (under RLS, deletable when the report is
/// resolved) and nowhere else. The audit chain is hash-chained and replicated into the memory module —
/// durable and hard to rewrite, which is exactly wrong for a copy of content someone asked us to remove.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn evidence_stays_in_the_report_row() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, s.alice, &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "abusive text here").await;

    let (status, _) = post_report(
        &app,
        &token(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"hate",
               "detail":"private note"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (body, detail, reason): (Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT snapshot_body, detail, reason FROM chat_reports WHERE target_message_id = $1",
    )
    .bind(msg)
    .fetch_one(&mut *tx)
    .await
    .expect("report row");

    assert_eq!(body.as_deref(), Some("abusive text here"));
    assert_eq!(detail.as_deref(), Some("private note"));
    assert_eq!(reason, "hate");
}
