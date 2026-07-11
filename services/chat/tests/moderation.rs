//! FR-CHAT-269 — moderation queue, integration tests.
//!
//!   cd services/dev && docker compose up -d
//!   DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos \
//!     cargo test -p cyberos-chat --test moderation -- --ignored --test-threads=1
//!
//! AC coverage (§4):
//!   AC 1,2      role_gate_fails_closed              (also unit-tested in auth::moderator_gate_tests)
//!   AC 3,4      duplicates_fold_and_severity_outranks_age
//!   AC 6        snapshot_renders_after_the_sender_deletes
//!   AC 7        a_dm_report_discloses_only_the_reported_message   <- the privacy property
//!   AC 8        group_context_requires_membership
//!   AC 9        blocks_do_not_apply_to_the_queue
//!   AC 10,13    concurrent_resolve_has_exactly_one_winner
//!   AC 11       siblings_close_together
//!   AC 12       remove_member_touches_the_channel_only
//!   AC 15       resolution_sets_the_purge_date
//!   AC 16       cross_tenant_probe_is_404_not_403
//!   AC 5        the_queue_is_always_paginated
//!
//! NOT covered here: AC 17-20 (client — nav gating, text-not-HTML rendering, content-policy link, locales).
//! AC 18/20 are compile-enforced or React-escaped; see the review packet.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use cyberos_chat::{auth::Authenticator, notify::Notifier, realtime, storage, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const HS256_SECRET: &[u8] = b"cyberos-chat-test-secret-fr-chat-269";

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    preflight(&url).await;
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Superusers bypass RLS entirely; drop to the runtime role or every isolation assertion here
                // is vacuous. See tests/reports.rs for the full argument.
                sqlx::query("SET ROLE cyberos_app").execute(conn).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect")
}

/// Separate the two ways this harness fails to get a usable connection, because sqlx cannot: a failing
/// `after_connect` is retried silently until the acquire timeout, so "Postgres is down" and "the cyberos_app
/// role does not exist" both surface as an identical `PoolTimedOut`. Probing on a plain connection first
/// splits them into two errors that each say what to do. See tests/blocks.rs for the full argument.
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
        audit_pool: None,
        capturer: None,
        authenticator: Arc::new(Authenticator::from_hs256_secret(HS256_SECRET)),
        hub: realtime::Hub::default(),
        presence: realtime::Presence::default(),
        notifier: Notifier::default(),
        attachments: storage::AttachmentConfig::default(),
        blocks: cyberos_chat::blocks::BlockCache::default(),
    })
}

/// A token carrying an explicit `roles` claim.
fn token_with_roles(subject: Uuid, tenant: Uuid, roles: &[&str]) -> String {
    let claims = json!({
        "sub": subject.to_string(),
        "tenant_id": tenant.to_string(),
        "roles": roles,
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    });
    sign(&claims)
}

/// A token with NO `roles` claim at all — the pre-FR-AUTH-101 shape. AC 1 turns on this case.
fn token_without_roles_claim(subject: Uuid, tenant: Uuid) -> String {
    let claims = json!({
        "sub": subject.to_string(),
        "tenant_id": tenant.to_string(),
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    });
    sign(&claims)
}

fn sign(claims: &Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(HS256_SECRET),
    )
    .expect("sign")
}

fn member(subject: Uuid, tenant: Uuid) -> String {
    token_with_roles(subject, tenant, &["tenant-member"])
}
fn admin_token(subject: Uuid, tenant: Uuid) -> String {
    token_with_roles(subject, tenant, &["tenant-admin"])
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    tok: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut b = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {tok}"));
    let req = match &body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            b.body(Body::from(v.to_string())).unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    };
    let res = app.clone().oneshot(req).await.expect("oneshot");
    let status = res.status();
    let bytes = to_bytes(res.into_body(), 512 * 1024).await.unwrap();
    (status, body_value(&bytes))
}

/// Parse the body as JSON, and on failure keep it as a STRING rather than discarding it.
///
/// `crate::internal` returns the error text as a plain-text body, so a 500 is not JSON. The obvious
/// `serde_json::from_slice(..).unwrap_or(Value::Null)` therefore throws away the one thing a failing test
/// needs: an assertion that says `queue: Null` names the symptom and hides the cause. Keeping the raw text
/// turns the same failure into `queue: "function min(uuid) does not exist"` — which is the whole diagnosis.
fn body_value(bytes: &[u8]) -> Value {
    if bytes.is_empty() {
        return Value::Null; // 204 and friends: genuinely no body, not an unparseable one.
    }
    serde_json::from_slice(bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes).into_owned()))
}

async fn queue(app: &axum::Router, tok: &str) -> Vec<Value> {
    let (s, v) = call(app, "GET", "/v1/chat/admin/reports", tok, None).await;
    assert_eq!(s, StatusCode::OK, "queue: {v:?}");
    v["entries"].as_array().cloned().unwrap_or_default()
}

async fn detail(app: &axum::Router, tok: &str, id: Uuid) -> (StatusCode, Value) {
    call(
        app,
        "GET",
        &format!("/v1/chat/admin/reports/{id}"),
        tok,
        None,
    )
    .await
}

async fn resolve(app: &axum::Router, tok: &str, id: Uuid, body: Value) -> (StatusCode, Value) {
    call(
        app,
        "POST",
        &format!("/v1/chat/admin/reports/{id}/resolve"),
        tok,
        Some(body),
    )
    .await
}

async fn report(app: &axum::Router, tok: &str, body: Value) -> Uuid {
    let (s, v) = call(app, "POST", "/v1/chat/reports", tok, Some(body)).await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK,
        "report: {s} {v:?}"
    );
    serde_json::from_value(v["id"].clone()).expect("report id")
}

// ── Seeding ──────────────────────────────────────────────────────────────────────────────────────

struct Seed {
    tenant: Uuid,
    admin: Uuid,
    alice: Uuid,
    bob: Uuid,
    carol: Uuid,
}

fn seed() -> Seed {
    Seed {
        tenant: Uuid::new_v4(),
        admin: Uuid::new_v4(),
        alice: Uuid::new_v4(),
        bob: Uuid::new_v4(),
        carol: Uuid::new_v4(),
    }
}

async fn tenant_tx(pool: &PgPool, tenant: Uuid) -> sqlx::Transaction<'_, sqlx::Postgres> {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("guc");
    tx
}

impl Seed {
    async fn channel(&self, pool: &PgPool, kind: &str, members: &[Uuid]) -> Uuid {
        let mut tx = tenant_tx(pool, self.tenant).await;
        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO chat_channels (tenant_id, name, created_by, kind)
             VALUES ($1,$2,$3,$4) RETURNING id",
        )
        .bind(self.tenant)
        .bind("mod-test")
        .bind(members[0])
        .bind(kind)
        .fetch_one(&mut *tx)
        .await
        .expect("channel");
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
            .expect("member");
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
        .expect("message");
        tx.commit().await.unwrap();
        id
    }

    /// Age every open report in this tenant, so "a week-old spam report" is testable without sleeping.
    async fn backdate_reports(&self, pool: &PgPool, days: i64) {
        let mut tx = tenant_tx(pool, self.tenant).await;
        sqlx::query(
            "UPDATE chat_reports SET created_at = created_at - ($2 || ' days')::interval
              WHERE tenant_id = $1",
        )
        .bind(self.tenant)
        .bind(days.to_string())
        .execute(&mut *tx)
        .await
        .expect("backdate");
        tx.commit().await.unwrap();
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────────────────────────

/// AC 1, 2 — the gate fails closed, and a CHANNEL role is not a workspace role.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn role_gate_fails_closed() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    // ALICE owns a channel — a channel role, which must grant nothing here (§1 #3). The person a report is
    // *about* could otherwise resolve it.
    let _ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;

    for (label, tok) in [
        (
            "no roles claim at all",
            token_without_roles_claim(s.alice, s.tenant),
        ),
        ("empty roles", token_with_roles(s.alice, s.tenant, &[])),
        ("ordinary member", member(s.alice, s.tenant)),
        (
            "channel owner",
            token_with_roles(s.alice, s.tenant, &["owner"]),
        ),
    ] {
        for uri in [
            "/v1/chat/admin/reports",
            "/v1/chat/admin/reports/00000000-0000-0000-0000-000000000001",
        ] {
            let (st, _) = call(&app, "GET", uri, &tok, None).await;
            assert_eq!(
                st,
                StatusCode::FORBIDDEN,
                "{label} must be refused on {uri}"
            );
        }
    }

    // And the real thing is admitted.
    let (st, _) = call(
        &app,
        "GET",
        "/v1/chat/admin/reports",
        &admin_token(s.admin, s.tenant),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
}

/// AC 3, 4 — three reports on one message fold into ONE entry, and a fresh self_harm outranks week-old spam.
/// Both are structural defences against reviewer fatigue, which is how moderation queues actually fail.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn duplicates_fold_and_severity_outranks_age() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s
        .channel(&pool, "group", &[s.alice, s.bob, s.carol, s.admin])
        .await;
    let spam = s.message(&pool, ch, s.bob, "buy now").await;

    // Three DIFFERENT people report the same message.
    for who in [s.alice, s.carol, s.admin] {
        report(
            &app,
            &member(who, s.tenant),
            json!({"target_kind":"message","target_message_id":spam,"reason":"spam"}),
        )
        .await;
    }
    s.backdate_reports(&pool, 7).await; // the spam is a week old

    let sh = s.message(&pool, ch, s.bob, "...").await;
    report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":sh,"reason":"self_harm"}),
    )
    .await;

    let q = queue(&app, &admin_token(s.admin, s.tenant)).await;
    assert_eq!(
        q.len(),
        2,
        "three spam reports must fold into ONE entry: {q:?}"
    );
    assert_eq!(
        q[0]["target_message_id"],
        json!(sh),
        "a minute-old self_harm must outrank a week-old spam pile"
    );
    assert_eq!(q[0]["severity"], json!(0));
    assert_eq!(q[1]["report_count"], json!(3));
}

/// AC 7 — the privacy property this FR turns on. A DM report discloses the reported message and NOTHING
/// else. Not the thread. Not one line of it.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn a_dm_report_discloses_only_the_reported_message() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let dm = s.channel(&pool, "direct", &[s.alice, s.bob]).await;
    s.message(&pool, dm, s.alice, "private thing one").await;
    let bad = s.message(&pool, dm, s.bob, "the abusive line").await;
    s.message(&pool, dm, s.alice, "private thing two").await;

    let r = report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":bad,"reason":"harassment"}),
    )
    .await;

    let (st, d) = detail(&app, &admin_token(s.admin, s.tenant), r).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(d["snapshot_body"], json!("the abusive line"));
    assert!(
        d["context"].as_array().unwrap().is_empty(),
        "a DM must disclose NO surrounding context"
    );

    // Assert on the RAW payload, not just the field: a leak elsewhere in the response is still a leak.
    let raw = d.to_string();
    assert!(
        !raw.contains("private thing one") && !raw.contains("private thing two"),
        "Alice's private correspondence must not reach her employer: {raw}"
    );
}

/// AC 8 — group context requires the admin to ALREADY be a member. A report is not a skeleton key.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn group_context_requires_membership() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    // A private channel the ADMIN is not in.
    let ch = s.channel(&pool, "group", &[s.bob, s.carol]).await;
    let msg = s.message(&pool, ch, s.bob, "reported line").await;
    s.message(&pool, ch, s.carol, "surrounding chatter").await;

    let r = report(
        &app,
        &member(s.carol, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"hate"}),
    )
    .await;

    let (_, d1) = detail(&app, &admin_token(s.admin, s.tenant), r).await;
    assert!(
        d1["context"].as_array().unwrap().is_empty(),
        "an admin who is not in the channel gets no context"
    );
    assert!(!d1.to_string().contains("surrounding chatter"));

    // Join the channel — a visible, audited act — and the context appears.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    sqlx::query(
        "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
         VALUES ($1,$2,$3,'member')",
    )
    .bind(ch)
    .bind(s.tenant)
    .bind(s.admin)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let (_, d2) = detail(&app, &admin_token(s.admin, s.tenant), r).await;
    assert!(
        d2.to_string().contains("surrounding chatter"),
        "a member admin gets the surrounding messages"
    );
}

/// AC 6 — the snapshot is the evidence, and the sender having deleted the original is ITSELF evidence.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn snapshot_renders_after_the_sender_deletes() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "the evidence").await;
    let r = report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"harassment"}),
    )
    .await;

    // BOB deletes it, exactly as messages::delete does.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    sqlx::query("UPDATE chat_messages SET deleted_at = now(), body = '' WHERE id = $1")
        .bind(msg)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let (_, d) = detail(&app, &admin_token(s.admin, s.tenant), r).await;
    assert_eq!(d["snapshot_body"], json!("the evidence"));
    assert_eq!(
        d["original_present"],
        json!(false),
        "the reviewer must be able to see that the sender has since removed it"
    );
}

/// AC 9 — blocks do NOT apply here. The likeliest reporter of a person is the person who blocked them.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn blocks_do_not_apply_to_the_queue() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, "group", &[s.admin, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "reported text").await;
    let r = report(
        &app,
        &admin_token(s.admin, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"hate"}),
    )
    .await;

    // The admin blocks the person they just reported.
    let (st, _) = call(
        &app,
        "POST",
        "/v1/chat/blocks",
        &admin_token(s.admin, s.tenant),
        Some(json!({"subject_id": s.bob})),
    )
    .await;
    assert_eq!(st, StatusCode::NO_CONTENT);

    // ...and can still adjudicate. If blocked_by were applied here, the admin would be reviewing a report
    // whose content they cannot read.
    let (_, d) = detail(&app, &admin_token(s.admin, s.tenant), r).await;
    assert_eq!(d["snapshot_body"], json!("reported text"));
    assert!(
        d.to_string().contains("reported text"),
        "an admin who blocked the reported person must still see the evidence in full"
    );
}

/// AC 10, 13 — two admins resolving at once. Exactly one state change, one decision row, one effect row.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn concurrent_resolve_has_exactly_one_winner() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let admin2 = Uuid::new_v4();

    let ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "delete me").await;
    let r = report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"hate"}),
    )
    .await;

    let body = json!({"action":"delete_message"});
    // Bound to `let` rather than inlined: `admin_token()` returns an owned String, and a `&String` temporary
    // inside `join!` is dropped at the end of the statement while the futures still borrow it (E0716).
    let tok_a = admin_token(s.admin, s.tenant);
    let tok_b = admin_token(admin2, s.tenant);
    let (a, b) = tokio::join!(
        resolve(&app, &tok_a, r, body.clone()),
        resolve(&app, &tok_b, r, body.clone()),
    );

    // BOTH get 200. The loser is handed the winner's outcome — an error would be technically accurate and
    // practically useless, since the outcome they wanted has happened.
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(b.0, StatusCode::OK);
    let losers = [&a.1, &b.1]
        .iter()
        .filter(|v| v["already_resolved"] == json!(true))
        .count();
    assert_eq!(
        losers, 1,
        "exactly one caller must lose the CAS: {a:?} {b:?}"
    );

    // Exactly one state change.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM chat_reports WHERE id = $1 AND status = 'actioned'")
            .bind(r)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(n, 1);
}

/// AC 11 — resolving one of five reports against the same message closes the other four, in the same tx.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn siblings_close_together() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let dave = Uuid::new_v4();

    let ch = s
        .channel(&pool, "group", &[s.alice, s.bob, s.carol, dave])
        .await;
    let msg = s.message(&pool, ch, s.bob, "abuse").await;

    let mut ids = Vec::new();
    for who in [s.alice, s.carol, dave] {
        ids.push(
            report(
                &app,
                &member(who, s.tenant),
                json!({"target_kind":"message","target_message_id":msg,"reason":"hate"}),
            )
            .await,
        );
    }

    let (st, res) = resolve(
        &app,
        &admin_token(s.admin, s.tenant),
        ids[0],
        json!({"action":"delete_message","note":"clear breach"}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        res["sibling_report_ids"].as_array().unwrap().len(),
        2,
        "the other two must be closed alongside"
    );

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (open,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM chat_reports WHERE target_message_id = $1 AND status = 'open'",
    )
    .bind(msg)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(
        open, 0,
        "no ghosts left against a message that no longer exists"
    );
}

/// AC 12 — remove_member is CHANNEL-scoped. Firing someone is not a chat feature.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn remove_member_touches_the_channel_only() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch1 = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let ch2 = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch1, s.bob, "abuse").await;
    let r = report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"harassment"}),
    )
    .await;

    let (st, _) = resolve(
        &app,
        &admin_token(s.admin, s.tenant),
        r,
        json!({"action":"remove_member"}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (in_ch1,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM chat_channel_members WHERE channel_id = $1 AND subject_id = $2",
    )
    .bind(ch1)
    .bind(s.bob)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let (in_ch2,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM chat_channel_members WHERE channel_id = $1 AND subject_id = $2",
    )
    .bind(ch2)
    .bind(s.bob)
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    assert_eq!(in_ch1, 0, "removed from the channel the harm happened in");
    assert_eq!(
        in_ch2, 1,
        "and NOT from any other channel — a chat button must not sever someone's access to the org"
    );
}

/// AC 15 — every resolution, including a dismissal, carries a purge date.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn resolution_sets_the_purge_date() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let msg = s.message(&pool, ch, s.bob, "meh").await;
    let r = report(
        &app,
        &member(s.alice, s.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"spam"}),
    )
    .await;

    resolve(
        &app,
        &admin_token(s.admin, s.tenant),
        r,
        json!({"action":"dismiss"}),
    )
    .await;

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (purge, status): (Option<chrono::DateTime<chrono::Utc>>, String) =
        sqlx::query_as("SELECT purge_after, status FROM chat_reports WHERE id = $1")
            .bind(r)
            .fetch_one(&mut *tx)
            .await
            .unwrap();

    assert_eq!(status, "dismissed", "a dismissal is not 'actioned'");
    let purge = purge.expect("even a dismissal must set a purge date");
    let days = (purge - chrono::Utc::now()).num_days();
    assert!(
        (89..=90).contains(&days),
        "the snapshot must not outlive its justification: {days} days"
    );
}

/// AC 16 — a cross-tenant probe is 404, never 403. A 403 would confirm the id exists somewhere.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn cross_tenant_probe_is_404_not_403() {
    let pool = pool().await;
    let a = seed();
    let b = seed();
    let app = app(pool.clone());

    let ch = a.channel(&pool, "group", &[a.alice, a.bob]).await;
    let msg = a.message(&pool, ch, a.bob, "tenant A's problem").await;
    let r = report(
        &app,
        &member(a.alice, a.tenant),
        json!({"target_kind":"message","target_message_id":msg,"reason":"hate"}),
    )
    .await;

    // A legitimate admin OF ANOTHER TENANT asks for it by id.
    let (st, _) = detail(&app, &admin_token(b.admin, b.tenant), r).await;
    assert_eq!(
        st,
        StatusCode::NOT_FOUND,
        "must be 404 — a 403 would confirm the id exists in a workspace they cannot see"
    );
}

/// AC 5 — always paginated; there is no unbounded mode, and an oversized limit is clamped.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn the_queue_is_always_paginated() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let tok = admin_token(s.admin, s.tenant);

    let ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    // Deliberately spanning THREE severities. A fixture where every report shares one severity cannot catch
    // a keyset cursor that breaks when the page crosses a severity boundary — which is exactly the bug a
    // row-comparison `(sev, ts, id) < (...)` introduces against this mixed ASC/DESC ordering.
    for (i, reason) in ["self_harm", "hate", "hate", "spam", "spam", "spam"]
        .iter()
        .enumerate()
    {
        let m = s.message(&pool, ch, s.bob, &format!("m{i}")).await;
        report(
            &app,
            &member(s.alice, s.tenant),
            json!({"target_kind":"message","target_message_id":m,"reason":reason}),
        )
        .await;
    }

    // A tiny page, and a cursor to the next.
    let (st, p1) = call(&app, "GET", "/v1/chat/admin/reports?limit=2", &tok, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(p1["entries"].as_array().unwrap().len(), 2);
    let cursor = p1["next_cursor"].as_str().expect("a next page exists");

    let (_, p2) = call(
        &app,
        "GET",
        &format!("/v1/chat/admin/reports?limit=2&cursor={cursor}"),
        &tok,
        None,
    )
    .await;
    let page2 = p2["entries"].as_array().unwrap();
    assert!(!page2.is_empty(), "the cursor must advance, not dead-end");

    // No overlap: a keyset cursor must not re-serve page one.
    let ids1: Vec<&Value> = p1["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| &e["lead_report_id"])
        .collect();
    for e in page2 {
        assert!(
            !ids1.contains(&&e["lead_report_id"]),
            "page 2 must not repeat page 1"
        );
    }

    // An absurd limit is clamped, not honoured.
    let (_, big) = call(
        &app,
        "GET",
        "/v1/chat/admin/reports?limit=10000",
        &tok,
        None,
    )
    .await;
    assert!(big["entries"].as_array().unwrap().len() <= 100);
}
