//! DB-slice store-of-record integration tests (FR-MCP-007/008), gated behind `#[ignore]`.
//!
//! These exercise `elicitation_pg` and `tasks_pg` against a live Postgres - the path the in-memory
//! unit tests cannot cover: persistence, KMS sealing at rest, caller-scoping (DEC-1159), idempotency,
//! and restart-resume (a fresh connection still sees the row). They are ignored by default so a no-DB
//! `cargo test` stays green; `scripts/local_verify.sh` runs them via `--include-ignored` with infra up.
//!
//! Setup uses the root tenant (nil UUID, seeded by auth migration 0001) and inserts a throwaway `agent`
//! subject under the `app.current_tenant_id` GUC (subjects is FORCE RLS); the mcp_* tables carry no RLS,
//! so the round-trip itself needs no GUC. Each test cleans up the rows it created.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::elicitation::RespondOutcome;
use crate::kms::{EnvKeyKms, Kms};
use crate::tasks::CancelOutcome;
use crate::{elicitation_pg, tasks_pg};

const ROOT_TENANT: &str = "00000000-0000-0000-0000-000000000000";

/// Connect a fresh pool to the local dev Postgres. A second call simulates a gateway restart.
async fn connect_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for the DB-slice test (bring infra up first)");
    PgPool::connect(&url).await.expect("connect to Postgres")
}

/// A deterministic 32-byte AEAD key via the public constructor (the field is private to `kms`).
fn test_kms() -> EnvKeyKms {
    EnvKeyKms::from_base64_key(&STANDARD.encode([7u8; 32])).expect("32-byte key")
}

/// Insert a throwaway `agent` subject under the root tenant and return its id. Agents need no password,
/// so the only constraint to satisfy is the handle format. Runs under the GUC because subjects is FORCE RLS.
async fn seed_agent_subject(pool: &PgPool) -> Uuid {
    let handle = format!("@mcp-sor-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let mut tx = pool.begin().await.expect("begin subject tx");
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .expect("set tenant guc");
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO subjects (tenant_id, handle, kind)
         VALUES ($1::uuid, $2, 'agent') RETURNING id",
    )
    .bind(ROOT_TENANT)
    .bind(&handle)
    .fetch_one(&mut *tx)
    .await
    .expect("insert agent subject");
    tx.commit().await.expect("commit subject");
    id
}

async fn delete_elicitation(pool: &PgPool, id: Uuid) {
    let _ = sqlx::query("DELETE FROM mcp_elicitations WHERE elicitation_id = $1")
        .bind(id)
        .execute(pool)
        .await;
}

async fn delete_task(pool: &PgPool, id: Uuid) {
    let _ = sqlx::query("DELETE FROM mcp_tasks WHERE task_id = $1")
        .bind(id)
        .execute(pool)
        .await;
}

/// Best-effort subject cleanup (child mcp_* rows must be gone first; subjects is FORCE RLS).
async fn delete_subject(pool: &PgPool, id: Uuid) {
    if let Ok(mut tx) = pool.begin().await {
        let _ =
            sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
                .execute(&mut *tx)
                .await;
        let _ = sqlx::query("DELETE FROM subjects WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await;
        let _ = tx.commit().await;
    }
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL); run via scripts/local_verify.sh or --include-ignored"]
async fn elicitation_persists_seals_and_is_caller_scoped() {
    let pool = connect_pool().await;
    let kms = test_kms();
    let tenant = Uuid::nil();
    let caller = seed_agent_subject(&pool).await;
    let other = seed_agent_subject(&pool).await;

    let id = elicitation_pg::create_confirmation(
        &pool,
        tenant,
        caller,
        "cyberos.demo.echo",
        json!({ "prompt": "delete it?" }),
    )
    .await
    .expect("create confirmation");

    // Pending: no verdict yet, and the owner's poll surfaces it while another caller's does not.
    assert_eq!(
        elicitation_pg::confirmation_state(&pool, id, caller)
            .await
            .expect("state pending"),
        None
    );
    let mine = elicitation_pg::pending(&pool, caller)
        .await
        .expect("poll mine");
    assert!(mine.iter().any(|e| e["elicitation_id"] == json!(id)));
    let theirs = elicitation_pg::pending(&pool, other)
        .await
        .expect("poll other");
    assert!(theirs.iter().all(|e| e["elicitation_id"] != json!(id)));

    // Respond accept -> recorded, then the verdict reads straight back from Postgres.
    match elicitation_pg::respond(&pool, &kms, id, caller, json!({ "confirmed": true }))
        .await
        .expect("respond accept")
    {
        RespondOutcome::Recorded { confirmed } => assert!(confirmed),
        other => panic!("expected Recorded{{true}}, got {other:?}"),
    }
    assert_eq!(
        elicitation_pg::confirmation_state(&pool, id, caller)
            .await
            .expect("state recorded"),
        Some(true)
    );
    // DEC-1159: a different caller cannot read the verdict.
    assert_eq!(
        elicitation_pg::confirmation_state(&pool, id, other)
            .await
            .expect("state cross-caller"),
        None
    );

    // Idempotent identical resubmit, and a foreign caller cannot respond to someone else's row.
    match elicitation_pg::respond(&pool, &kms, id, caller, json!({ "confirmed": true }))
        .await
        .expect("idempotent resubmit")
    {
        RespondOutcome::AlreadyRecorded { confirmed } => assert!(confirmed),
        other => panic!("expected AlreadyRecorded, got {other:?}"),
    }
    match elicitation_pg::respond(&pool, &kms, id, other, json!({ "confirmed": false }))
        .await
        .expect("foreign respond")
    {
        RespondOutcome::NotFound => {}
        other => panic!("expected NotFound, got {other:?}"),
    }

    // Sealed at rest: the stored blob is not plaintext but opens back to the payload.
    let blob = sqlx::query_scalar::<_, Option<Vec<u8>>>(
        "SELECT response_payload_kms_blob FROM mcp_elicitations WHERE elicitation_id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("read blob")
    .expect("blob present after respond");
    assert!(
        !contains_subslice(&blob, b"confirmed"),
        "payload must be KMS-sealed, not plaintext"
    );
    let opened: serde_json::Value =
        serde_json::from_slice(&kms.open(&blob).expect("kms open")).expect("opened json");
    assert_eq!(opened["confirmed"], json!(true));

    // Restart-resume: a fresh connection still sees the verdict (in-memory store would have lost it).
    let pool2 = connect_pool().await;
    assert_eq!(
        elicitation_pg::confirmation_state(&pool2, id, caller)
            .await
            .expect("state after reconnect"),
        Some(true)
    );

    delete_elicitation(&pool, id).await;
    delete_subject(&pool, caller).await;
    delete_subject(&pool, other).await;
}

#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL); run via scripts/local_verify.sh or --include-ignored"]
async fn task_persists_seals_result_and_is_caller_scoped() {
    let pool = connect_pool().await;
    let kms = test_kms();
    let tenant = Uuid::nil();
    let caller = seed_agent_subject(&pool).await;
    let other = seed_agent_subject(&pool).await;

    let task = tasks_pg::start(
        &pool,
        &kms,
        tenant,
        caller,
        "cyberos.demo.echo",
        json!({ "x": 1 }),
    )
    .await
    .expect("start task");

    // Input is sealed at rest and opens back to the original.
    let in_blob = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT input_payload_kms_blob FROM mcp_tasks WHERE task_id = $1",
    )
    .bind(task)
    .fetch_one(&pool)
    .await
    .expect("read input blob");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&kms.open(&in_blob).expect("open input"))
            .expect("input json"),
        json!({ "x": 1 })
    );

    // Running and visible to the owner; invisible to another caller (DEC-1159).
    let running = tasks_pg::status_view(&pool, &kms, task, caller)
        .await
        .expect("status running")
        .expect("owner sees task");
    assert_eq!(running["status"], json!("running"));
    assert!(tasks_pg::status_view(&pool, &kms, task, other)
        .await
        .expect("status cross-caller")
        .is_none());

    // Complete with a result; it round-trips through the seal on read.
    assert!(tasks_pg::complete(&pool, &kms, task, json!({ "ok": true }))
        .await
        .expect("complete task"));
    let done = tasks_pg::status_view(&pool, &kms, task, caller)
        .await
        .expect("status completed")
        .expect("owner sees completed");
    assert_eq!(done["status"], json!("completed"));
    assert_eq!(done["result"], json!({ "ok": true }));

    // Cancelling a terminal task is a no-op transition.
    match tasks_pg::cancel(&pool, task, caller)
        .await
        .expect("cancel terminal")
    {
        CancelOutcome::AlreadyTerminal => {}
        other => panic!("expected AlreadyTerminal, got {other:?}"),
    }

    // Restart-resume: a fresh connection still sees the completed task.
    let pool2 = connect_pool().await;
    let resumed = tasks_pg::status_view(&pool2, &kms, task, caller)
        .await
        .expect("status after reconnect")
        .expect("resumed task");
    assert_eq!(resumed["status"], json!("completed"));

    delete_task(&pool, task).await;
    delete_subject(&pool, caller).await;
    delete_subject(&pool, other).await;
}
