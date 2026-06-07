//! `cyberos-ai` operator CLI — FR-AI-021 full surface.

use clap::Parser;
use cyberos_ai_gateway::cli::exit_codes::ExitCode;
use cyberos_ai_gateway::cli::{auth, Cli, Command};
use std::process;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let json = cli.json;
    let confirm = cli.confirm;

    // Handle completions before auth check
    if let Command::Completions(ref args) = cli.command {
        cyberos_ai_gateway::cli::completions::run(args.shell.clone());
        return;
    }

    // Authenticate
    let claims = match auth::require_token() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("auth_failed: {e}");
            process::exit(ExitCode::AuthError as i32);
        }
    };

    // Build Postgres pool
    let pool = match build_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("remote_unreachable: {e}");
            process::exit(ExitCode::NetworkError as i32);
        }
    };

    let result = match cli.command {
        Command::Usage(args) => {
            cyberos_ai_gateway::cli::usage::run(args, json, &claims, &pool).await
        }
        Command::Models(args) => {
            cyberos_ai_gateway::cli::models::run(args.action, json, &claims, &pool).await
        }
        Command::Policy(args) => {
            cyberos_ai_gateway::cli::policy::run(args.action, json, confirm, &claims, &pool).await
        }
        Command::Failover(args) => {
            cyberos_ai_gateway::cli::failover::run(args.action, json, &claims, &pool).await
        }
        Command::Invoice(args) => {
            cyberos_ai_gateway::cli::invoice::run(args.action, json, &claims, &pool).await
        }
        Command::Breaker(args) => {
            cyberos_ai_gateway::cli::breaker::run(args.action, json, &claims, &pool).await
        }
        Command::Expiry(args) => {
            cyberos_ai_gateway::cli::expiry::run(args.action, json, &claims, &pool).await
        }
        Command::Memory(args) => {
            cyberos_ai_gateway::cli::memory::run(args.action, json, &claims, &pool).await
        }
        Command::Completions(_) => unreachable!(),
    };

    match result {
        Ok(()) => process::exit(ExitCode::Ok as i32),
        Err(e) => {
            eprintln!("{e}");
            process::exit(e.exit_code() as i32);
        }
    }
}

async fn build_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/cyberos".to_string());
    sqlx::PgPool::connect(&url).await
}
