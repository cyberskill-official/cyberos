//! `cyberos-memory-admin` — operator CLI for the memory service.
//!
//! Subcommands:
//!   * `rebuild --tenant <UUID>` — wipe + re-ingest Layer 2 for one tenant
//!     (FR-MEMORY-102). Useful after a Layer-2 schema migration or to recover
//!     from a corruption event.
//!   * `reconcile --tenant <UUID> [--sample 100]` — non-destructive sample
//!     verification that l2_memory's chain_anchor matches what we'd compute
//!     today. Reports mismatches. The 30-minute reconcile cadence is run
//!     by an OBS-scheduled cron that shells out to this binary.

use cyberos_cli_exit::ExitCode;
use cyberos_memory::{brain, rebuild};
use cyberos_types::TenantId;
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cyberos_memory=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
        return ExitCode::UsageError;
    }

    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL env var required");
            return ExitCode::ConfigError;
        }
    };
    let pool = match PgPool::connect(&url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("postgres connect failed: {e}");
            return ExitCode::NetworkError;
        }
    };

    match args[1].as_str() {
        "rebuild" => match parse_tenant(&args) {
            Ok(t) => match rebuild::run_full(&pool, t).await {
                Ok(s) => {
                    println!("✓ rebuild complete for tenant {t}");
                    println!("  rows_truncated:  {}", s.rows_truncated);
                    println!("  rows_reingested: {}", s.rows_reingested);
                    println!("  batches:         {}", s.batches);
                    println!("  duration:        {}s", s.duration_secs);
                    ExitCode::Ok
                }
                Err(e) => {
                    eprintln!("rebuild failed: {e}");
                    ExitCode::Generic
                }
            },
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::UsageError
            }
        },
        "reconcile" => match parse_tenant(&args) {
            Ok(t) => {
                let sample: i64 = args
                    .windows(2)
                    .find(|w| w[0] == "--sample")
                    .and_then(|w| w[1].parse().ok())
                    .unwrap_or(100);
                match rebuild::reconcile(&pool, t, sample).await {
                    Ok(s) => {
                        println!(
                            "reconcile: {} of {} passed; {} failed; {} ms",
                            s.passed, s.sample_size, s.failed, s.duration_ms
                        );
                        if s.failed > 0 {
                            eprintln!("\nFailures:");
                            for f in &s.failures {
                                eprintln!(
                                    "  seq={} path={} stored={} recomputed={}",
                                    f.seq,
                                    f.path,
                                    &f.stored_anchor[..16],
                                    &f.recomputed_anchor[..16.min(f.recomputed_anchor.len())]
                                );
                            }
                            ExitCode::Generic
                        } else {
                            ExitCode::Ok
                        }
                    }
                    Err(e) => {
                        eprintln!("reconcile failed: {e}");
                        ExitCode::Generic
                    }
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::UsageError
            }
        },
        // FR-MEMORY-123 §1 #14 — BRAIN backfill / rebuild paths. The derived lens is reproducible from the
        // Layer-1 chain; these re-derive it. The embedding client is the env (ai-gateway) one — these still
        // route every embedding through the gateway (residency + spend cap), never a provider directly.
        "brain-rebuild" => match parse_tenant(&args) {
            Ok(t) => {
                let gw = brain::EmbedClient::from_env();
                match brain::backfill::rebuild(t.as_uuid(), &pool, &gw).await {
                    Ok(n) => {
                        println!("✓ brain rebuild complete for tenant {t}: {n} events re-derived from Layer 1");
                        ExitCode::Ok
                    }
                    Err(e) => {
                        eprintln!("brain rebuild failed: {e}");
                        ExitCode::Generic
                    }
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::UsageError
            }
        },
        "brain-reembed" => match parse_tenant(&args) {
            Ok(t) => {
                let model = args
                    .windows(2)
                    .find(|w| w[0] == "--model")
                    .map(|w| w[1].clone())
                    .unwrap_or_else(|| brain::embed_client::BRAIN_EMBED_MODEL.to_string());
                let gw = brain::EmbedClient::from_env();
                let mut total = 0usize;
                loop {
                    match brain::backfill::reembed(t.as_uuid(), &pool, &gw, &model, 256).await {
                        Ok(0) => break,
                        Ok(n) => total += n,
                        Err(e) => {
                            eprintln!("brain reembed failed: {e}");
                            return ExitCode::Generic;
                        }
                    }
                }
                println!("✓ brain reembed complete for tenant {t}: {total} rows -> model {model}");
                ExitCode::Ok
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::UsageError
            }
        },
        "brain-resummarize" => {
            match parse_tenant(&args) {
                Ok(t) => {
                    let gw = brain::EmbedClient::from_env();
                    match brain::backfill::resummarize(t.as_uuid(), &pool, &gw, None).await {
                        Ok(n) => {
                            println!("✓ brain resummarize complete for tenant {t}: {n} scopes re-summarised");
                            ExitCode::Ok
                        }
                        Err(e) => {
                            eprintln!("brain resummarize failed: {e}");
                            ExitCode::Generic
                        }
                    }
                }
                Err(msg) => {
                    eprintln!("{msg}");
                    ExitCode::UsageError
                }
            }
        }
        "brain-reindex" => match brain::backfill::reindex_hot_hnsw(&pool).await {
            Ok(()) => {
                println!("✓ brain hot HNSW index reindexed");
                ExitCode::Ok
            }
            Err(e) => {
                eprintln!("brain reindex failed: {e}");
                ExitCode::Generic
            }
        },
        "--help" | "-h" => {
            usage();
            ExitCode::Ok
        }
        other => {
            eprintln!("unknown subcommand: {other}");
            usage();
            ExitCode::UsageError
        }
    }
}

fn parse_tenant(args: &[String]) -> Result<TenantId, String> {
    let raw = args
        .windows(2)
        .find(|w| w[0] == "--tenant")
        .map(|w| &w[1])
        .ok_or_else(|| "--tenant <UUID> required".to_string())?;
    let uuid: Uuid = raw
        .parse()
        .map_err(|e: uuid::Error| format!("bad UUID: {e}"))?;
    Ok(TenantId(uuid))
}

fn usage() {
    eprintln!("cyberos-memory-admin — operator CLI for the memory service");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  cyberos-memory-admin rebuild   --tenant <UUID>");
    eprintln!("  cyberos-memory-admin reconcile --tenant <UUID> [--sample 100]");
    eprintln!("  cyberos-memory-admin brain-rebuild     --tenant <UUID>            # FR-MEMORY-123: re-derive embeddings + summaries from Layer 1");
    eprintln!("  cyberos-memory-admin brain-reembed     --tenant <UUID> [--model bge-m3]   # migrate to a new embedding model version");
    eprintln!("  cyberos-memory-admin brain-resummarize --tenant <UUID>            # rebuild all current summaries from events");
    eprintln!("  cyberos-memory-admin brain-reindex                                # REINDEX the partial hot HNSW index");
    eprintln!();
    eprintln!("ENV:");
    eprintln!("  DATABASE_URL    must point at the memory postgres database");
    eprintln!("  AI_GATEWAY_URL  ai-gateway base for embeddings (default http://127.0.0.1:8080); residency + spend cap enforced there");
}
