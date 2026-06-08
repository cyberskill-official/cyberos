use assert_cmd::prelude::*;
use clap::Parser;
use predicates::prelude::*;
use std::process::Command;

use cyberos_ai_gateway::cli::Cli;

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
