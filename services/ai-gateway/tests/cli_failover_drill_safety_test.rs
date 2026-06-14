use cyberos_ai_gateway::cli::auth::{OperatorClaims, Role};
use cyberos_ai_gateway::cli::failover;
use cyberos_ai_gateway::cli::{CliError, FailoverAction};
use sqlx::postgres::PgPoolOptions;
use std::sync::OnceLock;

static ENV_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static tokio::sync::Mutex<()> {
    ENV_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://localhost/cyberos")
        .unwrap()
}

fn claims(roles: Vec<Role>) -> OperatorClaims {
    OperatorClaims {
        operator_id: "ops@cyberos.world".to_string(),
        roles,
        exp: None,
    }
}

#[tokio::test]
async fn failover_drill_requires_admin_role() {
    let _guard = env_lock().lock().await;
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "staging");
    let err = failover::run(
        FailoverAction::Drill {
            target: "bedrock:claude-3-5-sonnet".to_string(),
            duration: 30,
            prod_confirmed_aware: false,
        },
        false,
        false,
        &claims(vec![Role::Mutate]),
        &lazy_pool(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, CliError::InsufficientRole { .. }));
}

#[tokio::test]
async fn failover_drill_requires_global_confirm() {
    let _guard = env_lock().lock().await;
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "staging");
    let err = failover::run(
        FailoverAction::Drill {
            target: "bedrock:claude-3-5-sonnet".to_string(),
            duration: 30,
            prod_confirmed_aware: false,
        },
        false,
        false,
        &claims(vec![Role::Admin]),
        &lazy_pool(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, CliError::DestructiveWithoutConfirm));
    assert_eq!(err.exit_code(), 4);
}

#[tokio::test]
async fn production_failover_drill_requires_extra_guard() {
    let _guard = env_lock().lock().await;
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "production");
    let err = failover::run(
        FailoverAction::Drill {
            target: "bedrock:claude-3-5-sonnet".to_string(),
            duration: 30,
            prod_confirmed_aware: false,
        },
        false,
        true,
        &claims(vec![Role::Admin]),
        &lazy_pool(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, CliError::DestructiveWithoutConfirm));
    assert_eq!(err.exit_code(), 4);
}
