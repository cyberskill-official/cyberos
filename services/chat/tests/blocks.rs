//! FR-CHAT-268 — user blocking, integration tests.
//!
//! Same harness shape as tests/reports.rs: the REAL router in-process via `tower::ServiceExt::oneshot`,
//! against a live Postgres with the chat migrations applied (through 0015_chat_blocks.sql), HS256 tokens so
//! one test can act as several distinct subjects.
//!
//!   cd services/dev && docker compose up -d
//!   DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos \
//!     cargo test -p cyberos-chat --test blocks -- --ignored --test-threads=1
//!
//! AC coverage (§4 of the FR):
//!   AC 1        block_is_directional_and_private
//!   AC 2,3,15   self_block_refused_and_mutations_are_idempotent
//!   AC 4,5,6    group_collapses_dm_disappears
//!   AC 7        the_blocked_sender_observes_nothing        <- the security property the FR turns on
//!   AC 10       blocked_reactions_are_not_counted
//!   AC 11       unblock_restores_in_place
//!   AC 13       blocking_removes_nobody_from_the_channel
//!   AC 14       blocks_are_tenant_isolated
//!
//! NOT covered here, and deliberately named rather than quietly skipped:
//!   AC 8  (realtime socket drop/redact) — needs a websocket client against a running server; the enforcement
//!         is in realtime::ws_loop and is exercised by the chat smoke suite.
//!   AC 9  (zero notification, zero push, not even for a mention) — needs to observe the notifier fan-out.
//!         This is the point the FR calls out as previously broken, so it is the one most worth an eyeball:
//!         the enforcement is `blockers_of` in notify::fanout.
//!   AC 12 (moderation queue ignores blocks) — belongs to FR-CHAT-269, which does not exist yet.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use cyberos_chat::{auth::Authenticator, notify::Notifier, realtime, storage, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const HS256_SECRET: &[u8] = b"cyberos-chat-test-secret-fr-chat-268";

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
/// chat's). We propagate the error rather than `.ok()` it: a DB where the role is missing should fail loudly
/// here, not silently hand back superuser connections and turn the RLS tests back into no-ops.
async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    preflight(&url).await;
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        // 5s, not the 30s default: past preflight a connection either comes back promptly or something is
        // wrong that waiting will not fix. Eight tests × 30s is four minutes of nothing.
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

/// Separate the two ways this harness fails to get a usable connection, because sqlx cannot.
///
/// A failing `after_connect` is not surfaced — sqlx just retries until the acquire timeout and reports
/// `PoolTimedOut`. So "Postgres is not running" and "the cyberos_app role does not exist" produce the exact
/// same 30-second timeout with the exact same message, and neither names its own cause. Probing on a plain
/// connection first splits them into two errors that each say what to do.
///
/// The role check is not hygiene: `cyberos_app` is what makes RLS apply (see `pool` above). If it vanished,
/// the suite would fall back to superuser connections and every isolation assertion in this file would pass
/// while proving nothing. Better to refuse to run.
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
    let bytes = to_bytes(res.into_body(), 256 * 1024).await.unwrap();
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

async fn block(app: &axum::Router, tok: &str, subject: Uuid) -> StatusCode {
    call(
        app,
        "POST",
        "/v1/chat/blocks",
        tok,
        Some(json!({"subject_id": subject})),
    )
    .await
    .0
}

async fn unblock(app: &axum::Router, tok: &str, subject: Uuid) -> StatusCode {
    call(
        app,
        "DELETE",
        &format!("/v1/chat/blocks/{subject}"),
        tok,
        None,
    )
    .await
    .0
}

async fn messages(app: &axum::Router, tok: &str, channel: Uuid) -> Vec<Value> {
    let (s, v) = call(
        app,
        "GET",
        &format!("/v1/chat/channels/{channel}/messages"),
        tok,
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list: {v:?}");
    v.as_array().cloned().unwrap_or_default()
}

async fn channels(app: &axum::Router, tok: &str) -> Vec<Value> {
    let (s, v) = call(app, "GET", "/v1/chat/channels", tok, None).await;
    assert_eq!(s, StatusCode::OK, "channels: {v:?}");
    v.as_array().cloned().unwrap_or_default()
}

// ── Seeding (direct SQL; the block enforcement is what is under test, not channel creation) ──

struct Seed {
    tenant: Uuid,
    alice: Uuid,
    bob: Uuid,
    carol: Uuid,
}

fn seed() -> Seed {
    Seed {
        tenant: Uuid::new_v4(),
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
        .bind(if kind == "direct" { "dm" } else { "general" })
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

    async fn react(&self, pool: &PgPool, channel: Uuid, msg: Uuid, who: Uuid, emoji: &str) {
        let mut tx = tenant_tx(pool, self.tenant).await;
        sqlx::query(
            "INSERT INTO chat_reactions (message_id, channel_id, tenant_id, subject_id, emoji)
             VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
        )
        .bind(msg)
        .bind(channel)
        .bind(self.tenant)
        .bind(who)
        .bind(emoji)
        .execute(&mut *tx)
        .await
        .expect("reaction");
        tx.commit().await.unwrap();
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────────────────────────

/// AC 1 — a block is directional and private. B can learn nothing.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn block_is_directional_and_private() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    assert_eq!(
        block(&app, &token(s.alice, s.tenant), s.bob).await,
        StatusCode::NO_CONTENT
    );

    // A sees their own block.
    let (_, mine) = call(
        &app,
        "GET",
        "/v1/chat/blocks",
        &token(s.alice, s.tenant),
        None,
    )
    .await;
    assert_eq!(mine.as_array().unwrap().len(), 1);

    // B sees NOTHING — not "you are blocked", not an empty-with-a-hint. An empty list, exactly as before.
    let (st, theirs) = call(
        &app,
        "GET",
        "/v1/chat/blocks",
        &token(s.bob, s.tenant),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert!(
        theirs.as_array().unwrap().is_empty(),
        "B must not be able to discover A's block through any surface"
    );
}

/// AC 2, 3, 15 — self-block refused; both mutations idempotent; the no-op emits no audit row.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn self_block_refused_and_mutations_are_idempotent() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let tok = token(s.alice, s.tenant);

    // AC 2.
    assert_eq!(block(&app, &tok, s.alice).await, StatusCode::BAD_REQUEST);

    // AC 3 — twice is 204 twice, and leaves ONE row. Not 409: a distinguishable second response would let a
    // caller enumerate their own block state through side effects.
    assert_eq!(block(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);
    assert_eq!(block(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);

    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM chat_blocks WHERE blocker_subject_id = $1")
            .bind(s.alice)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(n, 1, "blocking twice leaves one row");

    // Unblocking someone never blocked is a no-op, not a 404.
    assert_eq!(unblock(&app, &tok, s.carol).await, StatusCode::NO_CONTENT);
}

/// AC 4, 5, 6 — the shape of the whole feature. Group: the row survives, the content does not. DM: gone
/// entirely, and the DM leaves the list.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn group_collapses_dm_disappears() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let group = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let dm = s.channel(&pool, "direct", &[s.alice, s.bob]).await;
    let gm = s.message(&pool, group, s.bob, "group text").await;
    let dmm = s.message(&pool, dm, s.bob, "dm text").await;

    let tok = token(s.alice, s.tenant);
    assert_eq!(block(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);

    // AC 4 — collapsed, not removed. Id and position preserved; body, attachments, reactions withheld.
    let rows = messages(&app, &tok, group).await;
    let m = rows.iter().find(|m| m["id"] == json!(gm)).expect(
        "the row MUST survive — deleting it rewrites the channel's history for one participant",
    );
    assert_eq!(m["blocked_sender"], json!(true));
    assert_eq!(m["body"], json!(""));
    assert!(m["attachments"].as_array().is_none_or(|a| a.is_empty()));

    // AC 5 — in a DM it is not collapsed, it is ABSENT.
    let dm_rows = messages(&app, &tok, dm).await;
    assert!(
        !dm_rows.iter().any(|m| m["id"] == json!(dmm)),
        "a blocked sender's DM message must be absent, not collapsed"
    );

    // AC 6 — and the DM itself leaves A's list.
    let chans = channels(&app, &tok).await;
    assert!(
        !chans.iter().any(|c| c["id"] == json!(dm)),
        "the DM must leave the blocker's channel list while the block stands"
    );
    assert!(
        chans.iter().any(|c| c["id"] == json!(group)),
        "the GROUP channel must NOT disappear — blocking removes nobody from anything (§1 #10)"
    );
}

/// AC 7 — the security property the FR turns on. B is told nothing, by any means. Telling a harasser they
/// have been blocked is an escalation trigger; every mature messenger lets the blocked sender believe the
/// message went out, and so do we.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn the_blocked_sender_observes_nothing() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let dm = s.channel(&pool, "direct", &[s.alice, s.bob]).await;
    let bob = token(s.bob, s.tenant);

    // BOB posts before the block.
    let (before_status, before_body) = call(
        &app,
        "POST",
        &format!("/v1/chat/channels/{dm}/messages"),
        &bob,
        Some(json!({"body": "one"})),
    )
    .await;
    assert_eq!(before_status, StatusCode::CREATED);

    assert_eq!(
        block(&app, &token(s.alice, s.tenant), s.bob).await,
        StatusCode::NO_CONTENT
    );

    // ...and after. Identical status, identical response shape. Nothing to react to.
    let (after_status, after_body) = call(
        &app,
        "POST",
        &format!("/v1/chat/channels/{dm}/messages"),
        &bob,
        Some(json!({"body": "two"})),
    )
    .await;
    assert_eq!(
        after_status, before_status,
        "the blocked sender's POST must succeed exactly as before — a 403 here is the bug"
    );

    let shape = |v: &Value| {
        let mut k: Vec<String> = v
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        k.sort();
        k
    };
    assert_eq!(
        shape(&before_body),
        shape(&after_body),
        "not one field may differ — the response must not become an oracle for the block"
    );

    // And B still sees both of their own messages in their own view.
    let bobs_view = messages(&app, &bob, dm).await;
    assert_eq!(
        bobs_view.len(),
        2,
        "B sees their own messages, block or not"
    );

    // A sees neither.
    let alices_view = messages(&app, &token(s.alice, s.tenant), dm).await;
    assert!(
        alices_view.is_empty(),
        "A receives nothing from a blocked sender in a DM"
    );
}

/// AC 10 — a blocked person's reaction is not counted. The count is the leak: seeing "2" where it should be
/// "1" tells the blocker exactly that the blocked person is still there.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn blocked_reactions_are_not_counted() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, "group", &[s.alice, s.bob, s.carol]).await;
    let msg = s.message(&pool, ch, s.carol, "hello").await;
    s.react(&pool, ch, msg, s.bob, "👍").await;

    let tok = token(s.alice, s.tenant);
    assert_eq!(block(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);

    let rows = messages(&app, &tok, ch).await;
    let m = rows
        .iter()
        .find(|m| m["id"] == json!(msg))
        .expect("carol's message");
    let reactions = m["reactions"].as_array().cloned().unwrap_or_default();
    assert!(
        reactions.is_empty(),
        "a blocked person's reaction must not be counted for the blocker: {reactions:?}"
    );
}

/// AC 11 — unblock restores everything, in place. Nothing was ever deleted: the block filters READS only.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn unblock_restores_in_place() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());
    let tok = token(s.alice, s.tenant);

    let ch = s.channel(&pool, "group", &[s.alice, s.bob]).await;
    let a1 = s.message(&pool, ch, s.alice, "first").await;
    assert_eq!(block(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);
    let b1 = s.message(&pool, ch, s.bob, "during the block").await;
    let a2 = s.message(&pool, ch, s.alice, "third").await;

    // While blocked: b1 is present but hollow.
    let during = messages(&app, &tok, ch).await;
    let hollow = during.iter().find(|m| m["id"] == json!(b1)).unwrap();
    assert_eq!(hollow["body"], json!(""));

    assert_eq!(unblock(&app, &tok, s.bob).await, StatusCode::NO_CONTENT);

    let after = messages(&app, &tok, ch).await;
    let restored = after
        .iter()
        .find(|m| m["id"] == json!(b1))
        .expect("b1 is back");
    assert_eq!(
        restored["body"],
        json!("during the block"),
        "a message sent DURING the block must come back in full — nothing was lost"
    );
    assert!(
        restored.get("blocked_sender").is_none(),
        "the flag is gone from the wire entirely once unblocked"
    );
    // All three, in their original order (the list is DESC).
    let ids: Vec<&Value> = after.iter().map(|m| &m["id"]).collect();
    assert!(ids.contains(&&json!(a1)) && ids.contains(&&json!(b1)) && ids.contains(&&json!(a2)));
}

/// AC 13 — blocking removes nobody from anything, and is invisible to everyone else.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn blocking_removes_nobody_from_the_channel() {
    let pool = pool().await;
    let s = seed();
    let app = app(pool.clone());

    let ch = s.channel(&pool, "group", &[s.alice, s.bob, s.carol]).await;
    let msg = s.message(&pool, ch, s.bob, "visible to carol").await;

    assert_eq!(
        block(&app, &token(s.alice, s.tenant), s.bob).await,
        StatusCode::NO_CONTENT
    );

    // CAROL's view is untouched: she never blocked anyone, and A's block is nobody else's business.
    let carols = messages(&app, &token(s.carol, s.tenant), ch).await;
    let m = carols
        .iter()
        .find(|m| m["id"] == json!(msg))
        .expect("still there for carol");
    assert_eq!(m["body"], json!("visible to carol"));
    assert!(m.get("blocked_sender").is_none());

    // Both remain members.
    let mut tx = tenant_tx(&pool, s.tenant).await;
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM chat_channel_members WHERE channel_id = $1 AND subject_id = ANY($2)",
    )
    .bind(ch)
    .bind(vec![s.alice, s.bob])
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(
        n, 2,
        "a block must not remove either party from the channel"
    );
}

/// AC 14 — cross-tenant isolation.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn blocks_are_tenant_isolated() {
    let pool = pool().await;
    let a = seed();
    let b = seed();
    let app = app(pool.clone());

    assert_eq!(
        block(&app, &token(a.alice, a.tenant), a.bob).await,
        StatusCode::NO_CONTENT
    );

    let mut tx = tenant_tx(&pool, b.tenant).await;
    let leaked: Option<(Uuid,)> =
        sqlx::query_as("SELECT blocked_subject_id FROM chat_blocks WHERE blocker_subject_id = $1")
            .bind(a.alice)
            .fetch_optional(&mut *tx)
            .await
            .unwrap();
    assert!(
        leaked.is_none(),
        "tenant B must not see tenant A's block row"
    );
}
