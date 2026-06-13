use assert_cmd::prelude::*;
use clap::Parser;
use cyberos_ai_gateway::cli::auth::{OperatorClaims, Role};
use cyberos_ai_gateway::cli::{expiry, Cli, ExpiryAction};
use predicates::prelude::*;
use sqlx::postgres::PgPoolOptions;
use std::process::Command;
use uuid::Uuid;

#[test]
fn version_returns_binary_version() {
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cyberos-ai"));
}

#[test]
fn top_level_help_lists_operator_subcommands() {
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("usage"))
        .stdout(predicate::str::contains("models"))
        .stdout(predicate::str::contains("policy"))
        .stdout(predicate::str::contains("failover"))
        .stdout(predicate::str::contains("invoice"))
        .stdout(predicate::str::contains("breaker"))
        .stdout(predicate::str::contains("expiry"))
        .stdout(predicate::str::contains("memory"));
}

#[test]
fn subcommands_parse_with_expected_flags() {
    Cli::try_parse_from([
        "cyberos-ai",
        "usage",
        "--tenant",
        "org:test",
        "--month",
        "2026-05",
    ])
    .unwrap();
    Cli::try_parse_from(["cyberos-ai", "models", "list"]).unwrap();
    Cli::try_parse_from(["cyberos-ai", "models", "pricing"]).unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "policy",
        "set",
        "org:test",
        "--cap-usd",
        "200",
        "--zdr-required",
        "true",
        "--residency",
        "eu-1",
        "--allowed-personas",
        "persona-a",
        "persona-b",
        "--confirm",
    ])
    .unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "policy",
        "validate",
        "config/tenants/EXAMPLE.tenant.yaml",
    ])
    .unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "policy",
        "diff",
        "org:test",
        "--vs",
        "config/tenants/EXAMPLE.tenant.yaml",
    ])
    .unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "failover",
        "drill",
        "bedrock:claude-3-5-sonnet",
        "--duration",
        "30",
        "--confirm",
        "--prod-confirmed-aware",
    ])
    .unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "invoice",
        "export",
        "org:test",
        "--period",
        "2026-05",
        "--format",
        "json",
    ])
    .unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "breaker",
        "reset",
        "bedrock:claude-3-5-sonnet",
        "--confirm",
    ])
    .unwrap();
    Cli::try_parse_from(["cyberos-ai", "expiry", "repair", "--confirm"]).unwrap();
    Cli::try_parse_from(["cyberos-ai", "memory", "emit", "row.yaml", "--dry-run"]).unwrap();
    Cli::try_parse_from([
        "cyberos-ai",
        "memory",
        "audit-trail",
        "org:test",
        "--since",
        "2026-05-01T00:00:00Z",
    ])
    .unwrap();
}

#[test]
fn missing_operator_token_exits_with_auth_failed_code() {
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.env_remove("CYBEROS_AI_OPERATOR_TOKEN")
        .arg("usage")
        .arg("--tenant")
        .arg("org:test")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("auth_failed"));
}

#[test]
fn shell_completion_emits_script_without_auth() {
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cyberos-ai"));
}

#[tokio::test]
async fn expiry_repair_deletes_duplicate_hold_expired_rows_when_live_enabled() {
    if std::env::var_os("CYBEROS_AI_GATEWAY_TEST_MEMORY_WRITES").is_none()
        || std::env::var_os("CYBEROS_STORE").is_none()
    {
        eprintln!(
            "CYBEROS_AI_GATEWAY_TEST_MEMORY_WRITES=1 and CYBEROS_STORE are required; skipping live CLI expiry repair case"
        );
        return;
    }

    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        eprintln!("DATABASE_URL not set; skipping live CLI expiry repair case");
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| panic!("DATABASE_URL is set but Postgres connection failed: {e}"));

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS memory_rows (
            seq BIGSERIAL PRIMARY KEY,
            ts TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            kind TEXT NOT NULL,
            payload JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let hold_id = format!("cli-test-{}", Uuid::new_v4());
    sqlx::query(
        "INSERT INTO memory_rows (kind, payload)
         VALUES ('ai.hold_expired', jsonb_build_object('tenant_id', 'org:test', 'hold_id', $1)),
                ('ai.hold_expired', jsonb_build_object('tenant_id', 'org:test', 'hold_id', $1))",
    )
    .bind(&hold_id)
    .execute(&pool)
    .await
    .unwrap();

    let claims = OperatorClaims {
        operator_id: "ops@cyberos.world".to_string(),
        roles: vec![Role::Admin],
        exp: None,
    };

    expiry::run(ExpiryAction::Repair, false, true, &claims, &pool)
        .await
        .unwrap();

    let duplicate_groups: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::int8
         FROM (
             SELECT payload->>'hold_id'
             FROM memory_rows
             WHERE kind = 'ai.hold_expired' AND payload->>'hold_id' = $1
             GROUP BY payload->>'hold_id'
             HAVING COUNT(*) > 1
         ) d",
    )
    .bind(&hold_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(duplicate_groups.0, 0);

    let remaining: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::int8
         FROM memory_rows
         WHERE kind = 'ai.hold_expired' AND payload->>'hold_id' = $1",
    )
    .bind(&hold_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(remaining.0, 1);
}
