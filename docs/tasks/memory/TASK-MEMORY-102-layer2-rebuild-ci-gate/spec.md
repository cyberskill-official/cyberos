---
id: TASK-MEMORY-102
title: "Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30min budget + mid-rebuild resume + multi-tenant"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-MEMORY-101, TASK-MEMORY-110, TASK-AI-019]
depends_on: [TASK-MEMORY-101]
blocks: [TASK-MEMORY-107]

source_pages:
  - website/docs/modules/memory.html#rebuild
source_decisions:
  - DEC-070 (Layer 1 source of truth; Layer 2 must be derivable)
  - DEC-071 (rebuild deterministic; CI gates derivability)
  - DEC-072 (chain_anchor cross-check at rebuild time)
  - DEC-185 (30min budget for 100k rows on 4-core GHA runner; SLA for ops rebuild)

language: rust 1.81 + GitHub Actions
service: cyberos/services/memory/
new_files:
  - services/memory/src/bin/rebuild_layer2.rs
  - services/memory/src/rebuild/mod.rs
  - services/memory/src/rebuild/spot_check.rs
  - services/memory/src/rebuild/determinism.rs
  - services/memory/src/cli/rebuild_cli.rs
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/rebuild_determinism_test.rs
  - services/memory/tests/ingest_test.rs
  - .github/workflows/memory-rebuild.yml
modified_files:
  # add `run_one_pass` for rebuild
  - services/memory/src/layer2/ingest.rs
allowed_tools:
  - file_read: services/memory/**
  - file_write: services/memory/{src,tests}/**, .github/workflows/memory-rebuild.yml
  - bash: cd services/memory && cargo build --bin rebuild_layer2
  - bash: cd services/memory && cargo test rebuild
disallowed_tools:
  #11)
  - run rebuild against production Layer 2 without explicit `--prod-confirmed-aware` (per §1
  #4)
  - skip spot-check verification (per §1
  - lower 30min CI budget without explicit task amendment (per DEC-185)
  #6)
  - skip determinism check on rebuild output (per §1

effort_hours: 10
subtasks:
  - "0.5h: bin/rebuild_layer2.rs — clap CLI binary"
  - "1.0h: rebuild/mod.rs — wipe + ingest single-pass + verify"
  - "1.0h: rebuild/spot_check.rs — random-sample 100 Layer 1 rows; recompute embeddings; cosine compare"
  - "1.0h: rebuild/determinism.rs — SHA-256 of sorted layer2_memories; two runs equal"
  - "0.5h: cli/rebuild_cli.rs — operator preview (--dry-run + --tenant)"
  - "0.5h: Production safety guard (--prod-confirmed-aware + interactive Y)"
  - "0.5h: layer2/ingest.rs `run_one_pass` (terminates after cursor catches up)"
  - "1.0h: memory-rebuild.yml CI workflow (4-core ubuntu-22.04 + pgvector + AGE + BGE sidecar)"
  - "1.0h: Mid-rebuild crash resume (kill mid-ingest; restart picks up from cursor)"
  - "0.5h: Schema-mismatch detection (rebuild fails if layer2_memories has unmigrated columns)"
  - "1.5h: Tests — happy + deterministic + resume + chain-mismatch + spot-check + 30min-budget"
  - "0.5h: Multi-tenant rebuild (rebuild all tenants in parallel; assert no cross-leak)"
risk_if_skipped: "DEC-070 'Layer 2 is derivable from Layer 1' becomes unverified. A subtle bug in ingest could produce non-derivable state; rebuild would fail in production at the wrong time. CI gate catches at PR time. Without spot-check, embedding drift (e.g., BGE model upgrade) goes undetected. Without determinism check, hidden non-determinism (e.g., unsorted iteration) could produce different Layer 2 state on each rebuild."
---

## §1 — Description (BCP-14 normative)

A CI job + standalone binary `rebuild_layer2` **MUST** verify that Layer 2 can be rebuilt from Layer 1 deterministically. Each rebuild:

1. **MUST** be CI-gated on every PR touching `services/memory/**` OR `crates/cyberos-obs-sdk/**` (the latter affects ingest instrumentation) OR the workflow file itself.
2. **MUST** spin up a fresh Postgres instance (pgvector + AGE extensions) AND a BGE-M3 sidecar for the rebuild duration. The CI job uses `services` declaration in `.github/workflows/memory-rebuild.yml`.
3. **MUST** stream every Layer 1 binlog row (per tenant) through `layer2::ingest::run_one_pass` until cursor catches up to the chain head.
4. **MUST** verify post-rebuild via four checks:
- **Row-count check**: `count(layer2_memories WHERE tenant_id = T) == count(layer1_chain WHERE tenant_id = T)`.
- **Spot-check**: random-sample 100 Layer 1 rows; recompute BGE embedding via the SAME sidecar version; compare to stored embedding; cosine ≥ 0.99 (allowance for floating-point variance).
- **Chain-anchor check**: every layer2_memories row's `chain_anchor` matches `SHA-256(canonical(corresponding Layer 1 row))`.
- **Determinism check**: run rebuild twice in succession; assert `SHA-256(sorted(layer2_memories rows))` is byte-identical between runs.
5. **MUST** complete within 30 minutes for a 100K-row tenant binlog on a 4-core GitHub Actions `ubuntu-22.04` runner. SLA for ops-initiated rebuilds.
6. **MUST** be deterministic — running rebuild twice produces byte-identical layer2_memories state. Non-determinism sources (unsorted iteration, time-dependent values, RNG) MUST be eliminated; determinism check is the enforcement.
7. **MUST** include `cyberos-memory rebuild-layer2 --tenant <id> [--dry-run]` CLI for operator preview. `--dry-run` shows what would be rebuilt without executing.
8. **MUST** fail the PR if any verification step fails. Failure message identifies which check failed AND which row(s) caused it.
9. **MUST** support mid-rebuild crash resume — kill the rebuild process mid-ingest; restart with same `--tenant`; resume from cursor; final state matches uninterrupted run.
10. **MUST** detect schema mismatch — if `layer2_memories` table has unmigrated columns (e.g., new column added in PR but migration not applied), rebuild fails fast with `SchemaMismatch` before wasting time.
11. **MUST** apply production safety guard — running `rebuild_layer2` against a production Postgres requires `--prod-confirmed-aware` flag AND interactive Y confirmation (similar to TASK-AUTH-006 §1 #11). Production rebuild WIPES then RECREATES Layer 2 — destructive even though derivable.
12. **MUST** rebuild ALL tenants in parallel via tokio task per tenant (single-tenant serial would not finish 100 tenants × 1000 rows in 30min budget).
13. **MUST** assert no cross-tenant leakage during rebuild — after multi-tenant rebuild, query each tenant under RLS context; assert each sees only own rows.
14. **SHOULD** emit OTel metrics:
- `memory_rebuild_duration_seconds{tenant_id}` (histogram).
- `memory_rebuild_rows_ingested{tenant_id}` (gauge).
- `memory_rebuild_spot_check_pass{tenant_id}` (gauge; 0-100).
- `memory_rebuild_failures_total{stage}` (counter).

---

## §2 — Why this design (rationale for humans)

**Why CI gate on every memory PR (§1 #1)?** Layer 2 derivability is the load-bearing invariant. Without CI enforcement, a subtle bug (e.g., new column in ingest path that isn't deterministic) ships to production AND breaks rebuild capability — discovered only when ops actually needs to rebuild (typically during incident).

**Why fresh Postgres + BGE per CI (§1 #2)?** Reusing infra would let prior runs' state contaminate. Fresh instance ensures the rebuild actually works from scratch, not "works because state happens to be correct."

**Why 4 verification checks (§1 #4)?** Each catches a different failure mode:
- Row-count: ingest dropped rows.
- Spot-check: embedding logic regressed (wrong model? wrong text extraction?).
- Chain-anchor: canonicalisation regressed (different bytes hash differently).
- Determinism: hidden non-determinism (RNG, time, unsorted iteration).

Together, they cover the most-likely Layer 2 regression classes.

**Why cosine ≥ 0.99 in spot-check (§1 #4)?** BGE-M3 is deterministic in principle but floating-point arithmetic across CPU generations produces tiny variations. 0.99 cosine = essentially identical with float-tolerance margin. Above 0.99 catches model regressions; below would false-positive on hardware variance.

**Why determinism check (§1 #6)?** Hidden non-determinism is a class of bug that's invisible until it bites. Sources: `HashMap` iteration order (Rust randomises by default), system time in canonicalisation, RNG seeded from clock. The double-rebuild check forces determinism as a structural property.

**Why 30min budget for 100K rows (DEC-185)?** Operators rebuild during incidents — speed matters. 30min is the SLA; longer means ops sits idle waiting. Math: 100K rows × ~10ms/row = 1000s = ~17min steady-state; 30min covers warmup + spot-check + verification.

**Why production safety guard (§1 #11)?** Rebuild WIPES Layer 2 (truncate + re-ingest). On production, this means search returns empty results until rebuild completes (~17min). Operators MUST consciously accept the downtime.

**Why parallel multi-tenant (§1 #12)?** 100 tenants × serial would take 100 × 17min = 28h. Parallel via tokio task per tenant scales linearly with cores. 4-core CI runner handles 4 tenants concurrently; full parallelism on production hardware.

**Why cross-tenant leak check post-rebuild (§1 #13)?** Multi-tenant parallelism could let one tenant's data leak into another's via shared state (e.g., wrong RLS context). The post-rebuild check catches this.

**Why schema mismatch fail-fast (§1 #10)?** Without it, rebuild proceeds, INSERT fails on unknown column, partial state results. Fail-fast at start saves time + prevents partial state.

---

## §3 — API contract

```rust
// services/memory/src/rebuild/mod.rs
use uuid::Uuid;
use sqlx::PgPool;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RebuildConfig {
    pub bge_url: String,
    pub spot_check_count: u32,           // default 100
    pub determinism_check: bool,         // CI sets true; ops --dry-run sets false
    pub timeout: Duration,
}

#[derive(Debug, Serialize)]
pub struct RebuildReport {
    pub tenant_id: Uuid,
    pub rows_ingested: u64,
    pub duration_seconds: u64,
    pub final_chain_anchor: String,      // hex of last row
    pub spot_check_pass: u32,
    pub spot_check_fail: u32,
    pub determinism_hash: Option<String>, // None if --dry-run
    pub stage_durations: HashMap<String, u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum RebuildError {
    #[error("schema mismatch: column {column} not in layer2_memories")]
    SchemaMismatch { column: String },
    #[error("row count mismatch: layer1={layer1} layer2={layer2}")]
    RowCountMismatch { layer1: u64, layer2: u64 },
    #[error("spot check failed: {pass}/{total} cosine >= 0.99")]
    SpotCheckFailed { pass: u32, total: u32 },
    #[error("chain anchor mismatch at seq={seq}")]
    ChainAnchorMismatch { seq: i64 },
    #[error("determinism violated: hash1={hash1} hash2={hash2}")]
    DeterminismViolated { hash1: String, hash2: String },
    #[error("cross-tenant leak: tenant {tenant} has rows from {leaked_from}")]
    CrossTenantLeak { tenant: Uuid, leaked_from: Uuid },
    #[error("production rebuild blocked: {reason}")]
    ProductionRebuildBlocked { reason: String },
    #[error("timeout (> {budget_minutes}m)")]
    Timeout { budget_minutes: u32 },
    #[error("ingest error: {0}")]
    Ingest(#[from] anyhow::Error),
}

pub async fn rebuild_layer2(
    tenant_id: Uuid, target_pool: &PgPool, source_root: &Path, config: &RebuildConfig,
) -> Result<RebuildReport, RebuildError> {
    let started = Instant::now();

    // §1 #10 schema check
    schema_validate(target_pool).await?;

    // §1 #11 production guard checked at CLI layer, but also here
    if std::env::var("CYBEROS_DEPLOYMENT_TIER").as_deref() == Ok("production")
       && std::env::var("CYBEROS_REBUILD_PROD_CONFIRMED").as_deref() != Ok("yes") {
        return Err(RebuildError::ProductionRebuildBlocked { reason: "missing CYBEROS_REBUILD_PROD_CONFIRMED".into() });
    }

    let mut stage_durations = HashMap::new();
    let t = Instant::now();

    // Wipe Layer 2 for this tenant
    rls::with_tenant(target_pool, tenant_id, |tx| async move {
        sqlx::query("DELETE FROM layer2_memories").execute(&mut **tx).await?;
        sqlx::query("DELETE FROM layer2_ingest_cursor WHERE tenant_id = $1").bind(tenant_id).execute(&mut **tx).await?;
        Ok(())
    }).await?;
    stage_durations.insert("wipe".into(), t.elapsed().as_secs());

    // Run ingest until cursor catches up
    let t = Instant::now();
    let rows = layer2::ingest::run_one_pass(tenant_id, target_pool, source_root, &config.bge_url).await?;
    stage_durations.insert("ingest".into(), t.elapsed().as_secs());

    // Row count check
    let t = Instant::now();
    let layer1_count = layer1::row_count(source_root, tenant_id).await?;
    let layer2_count: i64 = rls::with_tenant(target_pool, tenant_id, |tx| async move {
        sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories").fetch_one(&mut **tx).await
    }).await?;
    if layer1_count as i64 != layer2_count {
        return Err(RebuildError::RowCountMismatch { layer1: layer1_count, layer2: layer2_count as u64 });
    }
    stage_durations.insert("row_count".into(), t.elapsed().as_secs());

    // Spot-check
    let t = Instant::now();
    let (pass, fail) = spot_check::random_sample(tenant_id, target_pool, source_root, &config.bge_url, config.spot_check_count).await?;
    if pass < config.spot_check_count {
        return Err(RebuildError::SpotCheckFailed { pass, total: config.spot_check_count });
    }
    stage_durations.insert("spot_check".into(), t.elapsed().as_secs());

    // Chain-anchor check
    let t = Instant::now();
    chain_anchor_verify_all(tenant_id, target_pool, source_root).await?;
    stage_durations.insert("chain_anchor".into(), t.elapsed().as_secs());

    // Determinism check
    let determinism_hash = if config.determinism_check {
        let t = Instant::now();
        let hash1 = determinism::hash_layer2(tenant_id, target_pool).await?;
        let _ = rebuild_layer2(tenant_id, target_pool, source_root, &RebuildConfig {
            determinism_check: false, ..config.clone()
        }).await?;
        let hash2 = determinism::hash_layer2(tenant_id, target_pool).await?;
        if hash1 != hash2 {
            return Err(RebuildError::DeterminismViolated { hash1: hex::encode(hash1), hash2: hex::encode(hash2) });
        }
        stage_durations.insert("determinism".into(), t.elapsed().as_secs());
        Some(hex::encode(hash1))
    } else { None };

    Ok(RebuildReport {
        tenant_id, rows_ingested: rows as u64,
        duration_seconds: started.elapsed().as_secs(),
        final_chain_anchor: hex::encode(layer1::final_chain_anchor(source_root, tenant_id).await?),
        spot_check_pass: pass, spot_check_fail: fail,
        determinism_hash, stage_durations,
    })
}
```

```rust
// services/memory/src/rebuild/spot_check.rs
pub async fn random_sample(
    tenant_id: Uuid, target_pool: &PgPool, source_root: &Path,
    bge_url: &str, n: u32,
) -> Result<(u32, u32), RebuildError> {
    let rows = layer1::random_sample_rows(source_root, tenant_id, n).await?;
    let mut pass = 0u32;
    let mut fail = 0u32;
    for row in rows {
        let original_embedding: Vec<f32> = rls::with_tenant(target_pool, tenant_id, |tx| async move {
            sqlx::query_scalar("SELECT embedding FROM layer2_memories WHERE seq = $1").bind(row.seq).fetch_one(&mut **tx).await
        }).await?;

        let recomputed = bge::embed(bge_url, &row.body).await?;
        let cos = cosine_similarity(&original_embedding, &recomputed);
        if cos >= 0.99 { pass += 1; } else { fail += 1; }
    }
    Ok((pass, fail))
}
```

```rust
// services/memory/src/rebuild/determinism.rs
pub async fn hash_layer2(tenant_id: Uuid, pool: &PgPool) -> Result<[u8; 32], RebuildError> {
    let rows: Vec<(i64, Vec<u8>, Vec<u8>)> = rls::with_tenant(pool, tenant_id, |tx| async move {
        sqlx::query_as(
            "SELECT seq, embedding::bytea, chain_anchor FROM layer2_memories
             WHERE tenant_id = current_setting('app.tenant_id', true)::uuid
             ORDER BY seq"
        ).fetch_all(&mut **tx).await
    }).await?;

    let mut h = sha2::Sha256::new();
    for (seq, embedding, anchor) in rows {
        h.update(seq.to_be_bytes());
        h.update(&embedding);
        h.update(&anchor);
    }
    Ok(h.finalize().into())
}
```

```yaml
# .github/workflows/memory-rebuild.yml
name: memory Layer-2 Rebuild Gate
on:
  pull_request:
    paths:
      - 'services/memory/**'
      - 'crates/cyberos-obs-sdk/**'
      - '.github/workflows/memory-rebuild.yml'

jobs:
  rebuild:
    runs-on: ubuntu-22.04
    timeout-minutes: 35
    services:
      postgres:
        image: pgvector/pgvector:pg16
        env: { POSTGRES_PASSWORD: pass }
        ports: ['5432:5432']
        options: --health-cmd "pg_isready" --health-interval 5s
      bge-sidecar:
        image: cyberos/bge-m3-sidecar:latest
        ports: ['5060:5060']
        options: --health-cmd "curl -f http://localhost:5060/health" --health-interval 10s
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install AGE extension
        run: psql -h localhost -U postgres -c "CREATE EXTENSION age;"
      - name: Run migrations
        working-directory: services/memory
        run: sqlx migrate run
      - name: Generate test fixture (100K rows)
        working-directory: services/memory
        run: cargo run --bin generate_test_fixture -- --rows 100000
      - name: Run rebuild
        working-directory: services/memory
        env: { BGE_URL: http://localhost:5060 }
        run: |
          cargo build --release --bin rebuild_layer2
          time ./target/release/rebuild_layer2 \
            --tenant 550e8400-e29b-41d4-a716-446655440000 \
            --source-root .test-fixture-binlog/ \
            --bge-url $BGE_URL
      - name: Verify report
        working-directory: services/memory
        run: cat rebuild_report.json | jq -e '.spot_check_pass == 100 and .determinism_hash != null'
```

---

## §4 — Acceptance criteria

1. CI runs on every memory/** PR.
2. Fresh DB → rebuild succeeds with 100/100 spot checks.
3. Rebuild deterministic — two consecutive runs produce identical hash.
4. Rebuild < 30min for 100K rows on 4-core GHA runner.
5. Mid-rebuild crash → resume from cursor; final state matches uninterrupted.
6. Bad Layer 1 row (chain mismatch) → fail with explicit row info (seq + reason).
7. Schema mismatch (e.g., new column added to layer2_memories without migration) → CI fails BEFORE wasting time.
8. Spot check < 100 → CI fails with which samples mismatched (cosine values).
9. Row count mismatch → fails with `layer1=X layer2=Y`.
10. `--dry-run` flag shows what would be rebuilt without executing.
11. Production safety: `CYBEROS_DEPLOYMENT_TIER=production` + missing `CYBEROS_REBUILD_PROD_CONFIRMED` → exits with `ProductionRebuildBlocked`.
12. Multi-tenant parallel rebuild — 5 tenants × 20K rows each finishes in < 30min.
13. Cross-tenant leak check — after multi-tenant rebuild, each tenant's RLS-scoped count matches their own.
14. OTel metrics emit per stage.

---

## §5 — Verification

```rust
// services/memory/tests/ingest_test.rs
#[tokio::test]
async fn fresh_rebuild_succeeds_with_100_spot_checks() {
    let pool = test_pool().await;
    let source = test_helper::make_test_binlog(100).await;   // 100 rows
    let tenant = test_tenant().await;
    let report = rebuild_layer2(tenant, &pool, &source, &test_config()).await.unwrap();
    assert_eq!(report.spot_check_pass, 100);
    assert_eq!(report.spot_check_fail, 0);
}

#[tokio::test]
async fn rebuild_deterministic_two_runs_identical() {
    let pool = test_pool().await;
    let source = test_helper::make_test_binlog(50).await;
    let tenant = test_tenant().await;
    let r1 = rebuild_layer2(tenant, &pool, &source, &test_config_with_determinism()).await.unwrap();
    let r2 = rebuild_layer2(tenant, &pool, &source, &test_config_with_determinism()).await.unwrap();
    assert_eq!(r1.determinism_hash.unwrap(), r2.determinism_hash.unwrap());
}

#[tokio::test]
async fn schema_mismatch_fails_fast() {
    let pool = test_pool().await;
    sqlx::query("ALTER TABLE layer2_memories ADD COLUMN extra_unmigrated TEXT").execute(&pool).await.unwrap();
    let err = rebuild_layer2(test_tenant().await, &pool, &test_source(), &test_config()).await.expect_err("expected SchemaMismatch");
    assert!(matches!(err, RebuildError::SchemaMismatch { .. }));
}

#[tokio::test]
async fn mid_rebuild_crash_resumes() {
    let pool = test_pool().await;
    let source = test_helper::make_test_binlog(1000).await;
    let tenant = test_tenant().await;
    test_helper::inject_panic_at_seq(500);
    let r1 = std::panic::catch_unwind(|| {
        tokio_test::block_on(rebuild_layer2(tenant, &pool, &source, &test_config()))
    });
    test_helper::clear_panic_injection();
    let r2 = rebuild_layer2(tenant, &pool, &source, &test_config()).await.unwrap();
    assert_eq!(r2.rows_ingested, 1000);   // resumed; total still correct
}

#[tokio::test]
async fn chain_anchor_mismatch_fails_with_row_info() {
    let pool = test_pool().await;
    let source = test_helper::make_test_binlog(10).await;
    let tenant = test_tenant().await;
    test_helper::corrupt_layer1_row(&source, tenant, 5).await;
    let err = rebuild_layer2(tenant, &pool, &source, &test_config()).await.expect_err("expected ChainAnchorMismatch");
    match err {
        RebuildError::ChainAnchorMismatch { seq } => assert_eq!(seq, 5),
        e => panic!("wrong: {e:?}"),
    }
}

#[tokio::test]
async fn production_rebuild_blocked_without_env() {
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "production");
    std::env::remove_var("CYBEROS_REBUILD_PROD_CONFIRMED");
    let err = rebuild_layer2(test_tenant().await, &test_pool().await, &test_source(), &test_config()).await.expect_err("expected ProductionRebuildBlocked");
    assert!(matches!(err, RebuildError::ProductionRebuildBlocked { .. }));
}

#[tokio::test]
async fn multi_tenant_parallel_no_cross_leak() {
    let pool = test_pool().await;
    let source = test_helper::make_test_binlog_for_tenants(5, 100).await;
    let tenants: Vec<Uuid> = (0..5).map(|i| test_tenant_n(i)).collect();
    let mut joinset = tokio::task::JoinSet::new();
    for t in tenants.clone() {
        let pool = pool.clone();
        let source = source.clone();
        joinset.spawn(async move {
            rebuild_layer2(t, &pool, &source, &test_config()).await
        });
    }
    while let Some(r) = joinset.join_next().await { r.unwrap().unwrap(); }

    for t in &tenants {
        let count: i64 = rls::with_tenant(&pool, *t, |tx| async move {
            sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories").fetch_one(&mut **tx).await
        }).await.unwrap();
        assert_eq!(count, 100);
    }
}
```

---

## §6 — Implementation skeleton

See §3.

```rust
// services/memory/src/bin/rebuild_layer2.rs
use clap::Parser;

#[derive(Parser)]
#[command(name = "rebuild_layer2", version)]
struct Cli {
    #[arg(long)] tenant: Uuid,
    #[arg(long)] source_root: PathBuf,
    #[arg(long)] bge_url: String,
    #[arg(long)] dry_run: bool,
    #[arg(long)] prod_confirmed_aware: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let pool = build_pool().await?;

    if std::env::var("CYBEROS_DEPLOYMENT_TIER").as_deref() == Ok("production") && cli.prod_confirmed_aware {
        prompt_interactive_y_n("Rebuild production Layer 2? This wipes existing data. [y/N] ")?;
        std::env::set_var("CYBEROS_REBUILD_PROD_CONFIRMED", "yes");
    }

    let config = RebuildConfig {
        bge_url: cli.bge_url, spot_check_count: 100,
        determinism_check: !cli.dry_run, timeout: Duration::from_secs(30 * 60),
    };

    if cli.dry_run {
        println!("Would rebuild tenant {} from {}", cli.tenant, cli.source_root.display());
        return Ok(());
    }

    let report = rebuild_layer2(cli.tenant, &pool, &cli.source_root, &config).await?;
    std::fs::write("rebuild_report.json", serde_json::to_string_pretty(&report)?)?;
    println!("✅ Rebuild succeeded:\n{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
```

---

## §7 — Dependencies

- **TASK-MEMORY-101** — ingest_loop's `run_one_pass` is invoked.
- **TASK-AI-019** — BGE-M3 sidecar.
- **TASK-AUTH-003** — RLS pattern.
- Crates: `clap@4`, `sqlx@0.7`, `tokio`, `serde`, `chrono`, `sha2`, `hex`.
- GitHub Actions runner (4-core ubuntu-22.04).
- pgvector + AGE Postgres image (cached).

---

## §8 — Example payloads

### Rebuild report

```json
{
  "tenant_id": "550e8400-...",
  "rows_ingested": 100000,
  "duration_seconds": 1023,
  "final_chain_anchor": "a3f9c8d7...",
  "spot_check_pass": 100,
  "spot_check_fail": 0,
  "determinism_hash": "9d6e3a2b1c0f8e7d...",
  "stage_durations": {
    "wipe": 2,
    "ingest": 945,
    "row_count": 1,
    "spot_check": 50,
    "chain_anchor": 15,
    "determinism": 10
  }
}
```

### CI failure (spot-check)

```text
❌ FAIL: spot_check_failed: 87/100 cosine >= 0.99
   sample seq=1234: cosine = 0.85 (likely BGE model drift)
   sample seq=5678: cosine = 0.92
   ...
```

### Production block

```text
❌ ProductionRebuildBlocked: missing CYBEROS_REBUILD_PROD_CONFIRMED
   Set --prod-confirmed-aware AND respond Y to interactive prompt
```

---

## §9 — Open questions

All resolved. Deferred:
- Incremental rebuild (rebuild only since last cursor) — slice 4+.
- Per-region rebuild — slice 5+.
- Background rebuild (no downtime) via shadow tables — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Rebuild > 30min | CI timeout | Fail | Investigate slow ingest (BGE capacity? DB contention?) |
| Spot check < 100 | spot_check::random_sample | Fail with which seqs mismatched | Investigate embedding drift OR ingest bug |
| Chain anchor mismatch | chain_anchor_verify_all | Fail with seq | Layer 1 corruption; investigate |
| Determinism violated | hash1 != hash2 | Fail | Engineer fixes non-determinism source |
| Row count mismatch | counts compared | Fail | Investigate ingest dropping rows |
| Schema mismatch | schema_validate | Fail fast | Operator runs migration first |
| Mid-rebuild panic | catch in test | Resume on retry | By design |
| Cross-tenant leak | post-rebuild RLS query | Fail with which tenants leaked | Investigate parallel ingest |
| BGE sidecar down at CI | CI service health | Job fails | Operator investigates image |
| Production rebuild without env | env check | ProductionRebuildBlocked | Operator adds env |
| Production rebuild without interactive Y | tty check + prompt | Cancelled | Operator confirms |
| Determinism check exceeds 30min | second rebuild slow | Fail | Investigate CI runner |
| Spot check uses wrong BGE version | URL config | Cosine ~0.5 (very different) | Pin version |
| `--dry-run` accidentally executes | flag check | Confirm via tests | By design |
| Concurrent rebuild attempt (two CI runs) | not gated; assumes serial | Each operates on own DB | By design (CI ephemeral) |

---

## §11 — Notes

- Rebuild is the load-bearing test for DEC-070..072. Without it, derivability is unverified.
- Spot check uses cosine ≥ 0.99 to allow float-precision variance across hardware. Below 0.99 = real model regression.
- Determinism check via double-rebuild forces structural determinism. Hidden non-determinism (HashMap iteration, RNG) is caught.
- 30min budget for 100K rows × 4-core CI is tight but achievable. Beyond 30min, ops incidents extend.
- Production safety guard (env var + interactive Y) prevents accidental production wipes.
- Multi-tenant parallel uses tokio task per tenant — natural failure isolation; cores limit concurrency.
- `--dry-run` for operator preview before destructive rebuild.
- The CI workflow ships cached pgvector + AGE Postgres image to reduce setup time.

---

*End of TASK-MEMORY-102. Status: done (implemented 2026-05-23).*

## As built (2026-07-02)

Apache AGE was removed; rebuild targets the relational l2_edge + pgvector layer.
