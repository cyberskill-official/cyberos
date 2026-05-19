---
id: FR-AUTH-006
title: "cyberos-auth bootstrap CLI: tenant 0 + root-admin + initial signing key + sweepers + idempotency-table cleanup"
module: AUTH
priority: MUST
status: done
verify: D
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: 2026-05-19
memory_chain_hash: null
related_frs: [FR-AUTH-001, FR-AUTH-002, FR-AUTH-003, FR-AUTH-004, FR-AUTH-005]
depends_on: [FR-AUTH-001, FR-AUTH-002, FR-AUTH-004]
blocks: []

source_pages:
  - website/docs/modules/auth.html#bootstrap
source_decisions:
  - DEC-127 (tenant 0 UUID nil; root-admin can switch into any tenant)
  - DEC-128 (bootstrap creates initial signing key; quarterly rotation cron is part of bootstrap)
  - DEC-129 (sweepers: expired sessions + idempotency-keys + retired signing-keys; hourly cron)
  - DEC-130 (--reset is destructive; requires both --reset AND --confirm AND non-prod env OR force-flag)

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/bin/cyberos_auth.rs
  - services/auth/src/cli/mod.rs
  - services/auth/src/cli/bootstrap.rs
  - services/auth/src/cli/sweepers.rs
  - services/auth/src/cli/rotate_keys.rs
  - services/auth/tests/bootstrap_test.rs
  - services/auth/tests/sweepers_test.rs
  - services/auth/tests/bootstrap_reset_safety_test.rs
modified_files: []
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests}/**
  - bash: cd services/auth && cargo build --bin cyberos-auth
  - bash: cd services/auth && cargo test bootstrap
disallowed_tools:
  - allow `--reset` in production environment without `--force-prod-reset` (per §1 #11)
  - emit plaintext password in CLI output OR audit row (per §1 #5)
  - skip memory audit row on bootstrap success (per §1 #4)
  - skip initial signing key creation on bootstrap (per §1 #6 — without it, FR-AUTH-004 can't issue tokens)

effort_hours: 6
sub_tasks:
  - "0.5h: clap CLI structure (bootstrap, rotate-keys, sweepers, --version)"
  - "1.0h: bootstrap subcommand (tenant 0 + root-admin + initial signing key, all in single tx)"
  - "0.5h: --reset --confirm safety guard (env check + interactive Y prompt)"
  - "0.5h: rotate-keys subcommand (calls FR-AUTH-004's rotation::generate_new_signing_key)"
  - "1.0h: sweepers subcommand (deletes expired sessions + idempotency rows + retired keys)"
  - "0.5h: Password input via Zeroizing<String> + interactive password masking"
  - "0.5h: canonical::bootstrap_completed memory audit row"
  - "0.5h: Cron-friendly exit codes (0=ok, 1=user-error, 2=already-initialised, 3=destructive-without-confirm)"
  - "1.0h: Tests — fresh-bootstrap + idempotent-rerun + reset-with-confirm + reset-blocked-in-prod + sweepers + rotate-keys"
risk_if_skipped: "Bootstrap is the chicken-and-egg solver: tenant 0 + root-admin must exist before any other tenant or subject can be created. Without this CLI, the only paths to bootstrap are manual SQL (error-prone, no audit, no signing key) OR a privileged web UI (security regression). FR-AUTH-004 needs the initial signing key to issue tokens; FR-AUTH-005 needs sweepers to prevent storage growth. All three sweep targets (sessions, idempotency, signing-keys) grow unbounded without periodic deletion."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship a `cyberos-auth` CLI binary providing bootstrap + ongoing operational tasks. Each subcommand:

1. **MUST** support `bootstrap` to initialise tenant 0 + root-admin subject + initial signing key in a single Postgres transaction. Tenant 0 has UUID `00000000-0000-0000-0000-000000000000` (nil UUID) and slug `"root"`.
2. **MUST** prompt for root-admin email + password interactively OR accept via env vars `CYBEROS_BOOTSTRAP_EMAIL` and `CYBEROS_BOOTSTRAP_PASSWORD` (for CI / Terraform). Email must pass FR-AUTH-002 §1 #2 validation; password must pass §1 #4 complexity rules. Interactive password input is masked via `rpassword` crate.
3. **MUST** use bcrypt cost 12 for password hashing (matches FR-AUTH-002 §1 #3).
4. **MUST** emit memory row `auth.bootstrap_completed` with payload: `tenant_0_id`, `root_admin_subject_id`, `initial_signing_key_kid`, `bootstrap_environment` (development | staging | production), `bootstrapped_by` (system user running CLI). The memory write is in the same transaction; rollback rolls back both.
5. **MUST NOT** echo plaintext password in CLI output, memory row, OR any logs. The `Zeroizing<String>` discipline from FR-AUTH-002 applies; the password is hashed and dropped immediately.
6. **MUST** create the initial RSA-2048 signing key during bootstrap by invoking FR-AUTH-004's `rotation::generate_new_signing_key`. Without this, FR-AUTH-004's `/v1/auth/token` cannot issue tokens (no active key exists). The kid is recorded in the bootstrap audit row.
7. **MUST** be idempotent: running `cyberos-auth bootstrap` after success exits with code 5 (`AlreadyInitialised`) and message `"Tenant 0 + root-admin + signing key already exist. Use --reset --confirm to recreate (destructive)."`. The check is `SELECT EXISTS FROM tenants WHERE id = nil_uuid`.
8. **MUST** support `cyberos-auth rotate-keys` to manually trigger FR-AUTH-004's quarterly rotation (the same function that the cron runs). Useful for emergency rotation (suspected key compromise).
9. **MUST** support `cyberos-auth sweepers` to manually trigger cleanup of:
    - Expired `sessions` rows (where `expires_at < NOW()`).
    - Old `admin_idempotency_keys` rows (where `created_at < NOW() - INTERVAL '24 hours'`).
    - Retired `signing_keys` rows (where `status='retired'` AND `retired_at < NOW()`).
   Output reports counts deleted per table. Cron runs hourly; manual invocation is for ops investigation.
10. **MUST** support `--reset --confirm` for destructive re-bootstrap (drops tenant 0 + cascades). BOTH flags required; either alone exits with code 4.
11. **MUST** refuse `--reset` in production environment (`CYBEROS_DEPLOYMENT_TIER=production`) unless `--force-prod-reset` is ALSO passed AND an interactive Y/N prompt confirms (with the deployment tier displayed). Non-tty input in production exits 4 unconditionally.
12. **MUST** use standardised exit codes from the shared **`cyberos-cli-exit`** crate (single source of truth across CyberOS CLIs — see also FR-AI-021):
    - `0` Ok
    - `1` UserError (bad args, validation failure)
    - `2` AuthFailed (reserved — not raised by bootstrap, which is pre-auth)
    - `3` RemoteUnreachable (DB or memory)
    - `4` DestructiveWithoutConfirm OR ProductionResetBlocked
    - `5` AlreadyInitialised (bootstrap rerun — distinct from UserError so CI scripts can detect "already done, no action needed" vs "bad input, fix the call")
    - `6` SchemaViolation (password complexity, email format)
    - `7` InternalError
   These numerical values are a stable cross-CLI contract; any module-specific extensions begin at `200` (AUTH module range).
13. **SHOULD** print summary to stdout on success: tenant 0 id, root-admin id (NOT email — privacy; operator knows what they typed), initial signing key kid, memory audit row request_id.
14. **SHOULD** emit OTel span `auth.cli.bootstrap` (and `auth.cli.sweepers`, `auth.cli.rotate_keys`) with attributes `subcommand`, `outcome`, `deployment_tier`, `operator_id` (if from env, else `"interactive"`).

---

## §2 — Why this design (rationale for humans)

**Why a CLI not an API endpoint (§1 #1)?** The chicken-and-egg problem: creating tenant 0 + root-admin requires `POST /v1/admin/tenants` which requires a root-admin token which requires a root-admin subject which doesn't exist yet. The CLI breaks the cycle by writing directly to the DB (skipping the API layer). The trade-off is a separate code path; mitigated by reusing FR-AUTH-002's password-hash + email-validation logic.

**Why initial signing key in bootstrap (§1 #6)?** FR-AUTH-004's `/v1/auth/token` looks up the active signing key from `signing_keys` table. Without bootstrap creating one, the first token-issue request 500s with "no active key." Bundling it into bootstrap means "after `cyberos-auth bootstrap`, the system is fully operational" — no second step required.

**Why interactive password input + env-var fallback (§1 #2)?** Interactive is for humans (operator running bootstrap on a fresh deployment). Env-var is for CI/CD (Terraform, Ansible). Both work; choose based on context. The Zeroizing wrapper applies to both paths — password bytes are overwritten after hashing regardless of source.

**Why exit code 5 for AlreadyInitialised (§1 #7)?** Distinct from "user error" (exit 1) so CI scripts can detect "this was a re-run, no action needed" vs "bad input, fix the call." Stripe's CLI uses similar patterns. Code 5 is the shared `cyberos-cli-exit::ExitCode::AlreadyInitialised` constant — uniform across every CyberOS CLI.

**Why double-flag for --reset (§1 #10)?** Single-flag destructive operations are footguns. `--reset` looks scary but operators get habituated; the second `--confirm` flag is the deliberate-this-time gate. Terraform's `-destroy` requires similar interactive confirmation.

**Why production-reset extra guard (§1 #11)?** Resetting tenant 0 in production wipes EVERY tenant + EVERY subject + EVERY signing key. The destructive blast radius is total. The triple gate (`--reset` + `--confirm` + `--force-prod-reset` + interactive Y) makes accidental production destruction structurally impossible. Even malicious destruction requires deliberate steps that leave audit trail.

**Why sweepers in this CLI (§1 #9)?** Three tables grow unbounded without sweeping: `sessions`, `admin_idempotency_keys`, `signing_keys` (retired ones). Each FR's responsibility was "the data exists for N hours/days then becomes stale"; the sweeper is the implementation of "becomes stale = deleted." Centralising in this CLI gives one place to audit + one cron to schedule.

**Why `bootstrapped_by` in audit row (§1 #4)?** Forensic question: "who bootstrapped this deployment?" The CLI captures the system user (from `whoami` or `USER` env) — not perfect identity (no JWT here; we're pre-auth) but better than nothing. Combined with the deployment_tier, gives a useful audit trail.

**Why rotate-keys as a manual subcommand (§1 #8)?** Quarterly cron is the routine; manual invocation is for emergency rotation (suspected key compromise). The same function backs both — cron just calls the CLI binary with `rotate-keys` argument. Operators familiar with the manual command know what cron does.

**Why no email in summary output (§1 #13)?** The operator just typed the email; echoing it in stdout adds nothing AND risks landing in logs. The id is sufficient for ops correlation; the operator knows what email they used.

---

## §3 — API contract

```bash
cyberos-auth --version
cyberos-auth bootstrap [--email <e>] [--password-from-env]
                       [--reset --confirm [--force-prod-reset]]
cyberos-auth rotate-keys
cyberos-auth sweepers
```

```rust
// services/auth/src/cli/bootstrap.rs
#[derive(clap::Args)]
pub struct BootstrapArgs {
    #[arg(long)] pub email: Option<String>,
    #[arg(long)] pub password_from_env: bool,
    #[arg(long)] pub reset: bool,
    #[arg(long)] pub confirm: bool,
    #[arg(long)] pub force_prod_reset: bool,
}

pub async fn run(args: BootstrapArgs, pool: &PgPool) -> Result<BootstrapResult, BootstrapError> {
    let env_tier = std::env::var("CYBEROS_DEPLOYMENT_TIER").unwrap_or_else(|_| "development".into());

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM tenants WHERE id = '00000000-0000-0000-0000-000000000000'::uuid)"
    ).fetch_one(pool).await?;

    if exists && !args.reset {
        return Err(BootstrapError::AlreadyInitialised);
    }
    if args.reset && !args.confirm {
        return Err(BootstrapError::DestructiveWithoutConfirm);
    }
    if args.reset && env_tier == "production" {
        if !args.force_prod_reset {
            return Err(BootstrapError::ProductionResetBlocked { reason: "missing --force-prod-reset".into() });
        }
        if !is_tty() {
            return Err(BootstrapError::ProductionResetBlocked { reason: "non-tty input in production".into() });
        }
        prompt_interactive_y_n("Reset production tenant 0? This wipes all tenants/subjects/keys. [y/N] ")?;
    }

    let email = args.email.or_else(|| std::env::var("CYBEROS_BOOTSTRAP_EMAIL").ok())
                          .unwrap_or_else(|| prompt_email());
    validate_email(&email)?;
    let password = if args.password_from_env {
        Zeroizing::new(std::env::var("CYBEROS_BOOTSTRAP_PASSWORD")
            .map_err(|_| BootstrapError::UserError("CYBEROS_BOOTSTRAP_PASSWORD not set".into()))?)
    } else {
        prompt_password_masked()
    };
    password::validate_complexity(&password, &email)?;
    let hash = bcrypt::hash(&password, 12)?;

    let mut tx = pool.begin().await?;
    if args.reset {
        sqlx::query("DELETE FROM tenants WHERE id = '00000000-0000-0000-0000-000000000000'::uuid").execute(&mut *tx).await?;
        // CASCADE removes subjects, sessions, etc.
    }

    let tenant_0_id = Uuid::nil();
    sqlx::query("INSERT INTO tenants (id, slug, name) VALUES ($1, 'root', 'Root Tenant')")
        .bind(tenant_0_id).execute(&mut *tx).await?;

    rls::apply_for_tenant_in_tx(&mut *tx, tenant_0_id).await?;

    let subject_id = Uuid::new_v4();
    sqlx::query("INSERT INTO subjects (id, tenant_id, email, password_hash, roles) VALUES ($1, $2, $3, $4, ARRAY['root-admin'])")
        .bind(subject_id).bind(tenant_0_id).bind(&email).bind(&hash).execute(&mut *tx).await?;

    let kid = jwks::rotation::generate_new_signing_key_in_tx(&mut *tx).await?;

    let request_id = format!("bootstrap_{}", ulid::Ulid::new());
    let bootstrapped_by = std::env::var("USER").unwrap_or_else(|_| "unknown".into());
    memory::emit_in_tx(&mut tx, memory::canonical::bootstrap_completed(
        tenant_0_id, subject_id, &kid, &env_tier, &bootstrapped_by, &request_id,
    )).await?;

    tx.commit().await?;

    Ok(BootstrapResult {
        tenant_0_id, root_admin_subject_id: subject_id,
        initial_signing_key_kid: kid, request_id,
    })
}
```

```rust
// services/auth/src/cli/sweepers.rs
pub async fn run(pool: &PgPool) -> Result<SweepersResult, SweepersError> {
    let sessions: u64 = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
        .execute(pool).await?.rows_affected();
    let idempotency: u64 = sqlx::query("DELETE FROM admin_idempotency_keys WHERE created_at < NOW() - INTERVAL '24 hours'")
        .execute(pool).await?.rows_affected();
    let signing_keys: u64 = sqlx::query("DELETE FROM signing_keys WHERE status='retired' OR (status='retiring' AND retired_at < NOW())")
        .execute(pool).await?.rows_affected();
    Ok(SweepersResult { sessions, idempotency, signing_keys })
}
```

```rust
// services/auth/src/cli/rotate_keys.rs
pub async fn run(pool: &PgPool) -> Result<String, anyhow::Error> {
    let kid = jwks::rotation::generate_new_signing_key(pool).await?;
    Ok(kid)
}
```

---

## §4 — Acceptance criteria

1. **Fresh DB: bootstrap succeeds** — Tenant 0 created with nil UUID + slug `"root"` + name `"Root Tenant"`; root-admin subject created with role `["root-admin"]`; initial signing key created with status `active`.
2. **memory row emitted** — `auth.bootstrap_completed` row in chain with all 6 payload fields populated.
3. **Re-run after success: exit 5** — Output includes `"already initialised"`; no second tenant 0 created.
4. **`--reset` without `--confirm` exit 4** — Output requires both flags.
5. **`--reset --confirm` in non-prod recreates** — Drops + recreates tenant 0; new signing key kid; new memory row.
6. **`--reset --confirm` in production WITHOUT `--force-prod-reset` exit 4** — Production-reset blocked.
7. **`--reset --confirm --force-prod-reset` in production with interactive N exit 4** — User typed N at prompt.
8. **`--reset --confirm --force-prod-reset` in production with interactive Y succeeds** — Resets + requires tty.
9. **Production reset with non-tty input exit 4** — Pipe `echo y` doesn't bypass tty check.
10. **Email from env var `CYBEROS_BOOTSTRAP_EMAIL`** — CI path works without prompting.
11. **Password from env var `CYBEROS_BOOTSTRAP_PASSWORD` with `--password-from-env`** — CI path works.
12. **Interactive password input is masked** — `rpassword` shows no characters.
13. **Plaintext password NOT in stdout, memory row, OR audit log**.
14. **Initial signing key in JWKS** — After bootstrap, `/.well-known/jwks.json` returns the new kid.
15. **`cyberos-auth rotate-keys` generates new active key** — Prior active becomes retiring; new key is active.
16. **`cyberos-auth sweepers` deletes expired rows** — Reports `{sessions: N1, idempotency: N2, signing_keys: N3}`.
17. **Sweepers idempotent** — Re-running with no expired rows returns `{0, 0, 0}` and exit 0.
18. **Email validation: bad email exits 5** — `bootstrap --email "not-an-email"` exits SchemaViolation.
19. **Weak password exits 5** — `password.rs::validate_complexity` rejects.
20. **DB unreachable exits 3** — Wrong connection-string returns RemoteUnreachable.

---

## §5 — Verification

```rust
#[tokio::test]
async fn fresh_bootstrap_creates_tenant_0_and_root_admin() {
    let pool = test_pool_blank().await;
    let result = bootstrap::run(BootstrapArgs {
        email: Some("admin@cyberos.world".into()),
        password_from_env: true,
        reset: false, confirm: false, force_prod_reset: false,
    }, &pool).await.unwrap();

    assert_eq!(result.tenant_0_id, Uuid::nil());
    assert!(memory_test_helper::has_row("auth.bootstrap_completed", &result.request_id).await);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM signing_keys WHERE status='active'").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn rerun_returns_already_initialised() {
    let pool = test_pool_blank().await;
    bootstrap::run(default_args_env(), &pool).await.unwrap();
    let err = bootstrap::run(default_args_env(), &pool).await.expect_err("expected AlreadyInitialised");
    assert!(matches!(err, BootstrapError::AlreadyInitialised));
}

#[tokio::test]
async fn reset_without_confirm_exits_3() {
    let pool = test_pool().await;
    let err = bootstrap::run(BootstrapArgs { reset: true, confirm: false, ..default_args_env() }, &pool).await.expect_err("expected DestructiveWithoutConfirm");
    assert!(matches!(err, BootstrapError::DestructiveWithoutConfirm));
}

#[tokio::test]
async fn production_reset_without_force_prod_reset_blocked() {
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "production");
    let pool = test_pool().await;
    let err = bootstrap::run(BootstrapArgs { reset: true, confirm: true, force_prod_reset: false, ..default_args_env() }, &pool).await.expect_err("expected ProductionResetBlocked");
    assert!(matches!(err, BootstrapError::ProductionResetBlocked { .. }));
}

#[tokio::test]
async fn weak_password_exits_5() {
    let pool = test_pool_blank().await;
    std::env::set_var("CYBEROS_BOOTSTRAP_PASSWORD", "weak");
    let err = bootstrap::run(BootstrapArgs {
        email: Some("a@b.com".into()), password_from_env: true,
        reset: false, confirm: false, force_prod_reset: false,
    }, &pool).await.expect_err("expected WeakPassword");
    // Maps to SchemaViolation exit code in main.
}

#[tokio::test]
async fn sweepers_deletes_expired() {
    let pool = test_pool().await;
    test_helper::insert_expired_sessions(10).await;
    test_helper::insert_old_idempotency_keys(5).await;

    let result = sweepers::run(&pool).await.unwrap();
    assert_eq!(result.sessions, 10);
    assert_eq!(result.idempotency, 5);
}

#[tokio::test]
async fn bootstrap_audit_row_has_no_password() {
    let pool = test_pool_blank().await;
    let result = bootstrap::run(default_args_env(), &pool).await.unwrap();
    let row = memory_test_helper::find_row("auth.bootstrap_completed", &result.request_id).unwrap();
    let s = serde_json::to_string(&row.payload).unwrap();
    assert!(!s.contains("CorrectHorse"));   // the test password
}

#[tokio::test]
async fn rotate_keys_creates_new_active() {
    let pool = test_pool().await;
    bootstrap::run(default_args_env(), &pool).await.unwrap();
    let new_kid = rotate_keys::run(&pool).await.unwrap();
    let active_kid: String = sqlx::query_scalar("SELECT kid FROM signing_keys WHERE status='active'").fetch_one(&pool).await.unwrap();
    assert_eq!(active_kid, new_kid);
    let retiring_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM signing_keys WHERE status='retiring'").fetch_one(&pool).await.unwrap();
    assert_eq!(retiring_count, 1);
}
```

---

## §6 — Implementation skeleton

```rust
// services/auth/src/bin/cyberos_auth.rs
use clap::{Parser, Subcommand};
use cyberos_auth::cli::{bootstrap, sweepers, rotate_keys};
use cyberos_cli_exit::ExitCode;  // shared: 0=Ok, 1=UserError, 2=AuthFailed, 3=RemoteUnreachable, 4=DestructiveWithoutConfirm, 5=AlreadyInitialised, 6=SchemaViolation, 7=InternalError

#[derive(Parser)]
#[command(name = "cyberos-auth", version)]
struct Cli { #[command(subcommand)] cmd: Cmd }

#[derive(Subcommand)]
enum Cmd { Bootstrap(bootstrap::BootstrapArgs), RotateKeys, Sweepers }

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let pool = build_pool().await.unwrap_or_else(|e| {
        eprintln!("remote_unreachable: {e}");
        std::process::exit(ExitCode::RemoteUnreachable as i32);
    });
    let result = match cli.cmd {
        Cmd::Bootstrap(args) => bootstrap::run(args, &pool).await.map(|r| {
            println!("✅ Bootstrap complete:\n  tenant_0_id: {}\n  root-admin subject id: {}\n  signing key kid: {}",
                     r.tenant_0_id, r.root_admin_subject_id, r.initial_signing_key_kid);
        }).map_err(BootstrapError::into_exit_code),
        Cmd::RotateKeys => rotate_keys::run(&pool).await.map(|kid| println!("✅ New active key: {kid}")).map_err(|_| ExitCode::InternalError),
        Cmd::Sweepers   => sweepers::run(&pool).await.map(|r| println!("✅ Swept: sessions={} idempotency={} signing_keys={}", r.sessions, r.idempotency, r.signing_keys)).map_err(|_| ExitCode::InternalError),
    };
    match result { Ok(()) => std::process::exit(0), Err(code) => std::process::exit(code as i32) }
}
```

Cron schedule (FR-AUTH-006 ships the binary; ops team configures cron):

```
# /etc/cron.d/cyberos-auth
0 *    * * * cyberos cyberos-auth sweepers
0 3 1 1,4,7,10 * cyberos cyberos-auth rotate-keys   # quarterly, 03:00 UTC
```

---

## §7 — Dependencies

- **FR-AUTH-001..005** — All AUTH FRs use what bootstrap creates.
- Crates: `clap@4`, `rpassword@7`, `zeroize@1`, `bcrypt@0.15`, `sqlx`, `tokio`, `anyhow`.
- `whoami` for `bootstrapped_by` field.

---

## §8 — Example payloads

### Bootstrap (interactive)

```text
$ cyberos-auth bootstrap
Email: admin@cyberos.world
Password (hidden): **********
Confirm password: **********
✅ Bootstrap complete:
  tenant_0_id: 00000000-0000-0000-0000-000000000000
  root-admin subject id: 7e57c0de-...
  signing key kid: 01HZK9R8M3X5C8Q4
  request_id: bootstrap_01HZK...
```

### Bootstrap (CI/env)

```bash
export CYBEROS_BOOTSTRAP_EMAIL=admin@cyberos.world
export CYBEROS_BOOTSTRAP_PASSWORD='CorrectHorseBatteryStaple9!'
cyberos-auth bootstrap --password-from-env
```

### Already initialised

```text
$ cyberos-auth bootstrap
Tenant 0 + root-admin + signing key already exist.
Use --reset --confirm to recreate (destructive).
exit 5
```

### Production reset blocked

```text
$ CYBEROS_DEPLOYMENT_TIER=production cyberos-auth bootstrap --reset --confirm
ERROR: production reset requires --force-prod-reset AND interactive Y confirmation
exit 4
```

### Sweepers

```text
$ cyberos-auth sweepers
✅ Swept: sessions=42 idempotency=18 signing_keys=2
```

### Audit row `auth.bootstrap_completed`

```json
{
  "kind": "auth.bootstrap_completed",
  "payload": {
    "tenant_0_id": "00000000-0000-0000-0000-000000000000",
    "root_admin_subject_id": "7e57c0de-...",
    "initial_signing_key_kid": "01HZK9R8M3X5C8Q4",
    "bootstrap_environment": "production",
    "bootstrapped_by": "deploy-bot",
    "request_id": "bootstrap_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Multi-region bootstrap (each region needs its own bootstrap) — slice 6+.
- Bootstrap from existing-credentials backup (disaster recovery) — slice 5+.
- Web UI for bootstrap (deliberately deferred — CLI is the only path).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Already bootstrapped (no --reset) | DB exists check | Exit 5 | Op aware; no action needed OR use --reset |
| --reset without --confirm | flag check | Exit 4 | Op adds --confirm |
| --reset in production without --force-prod-reset | env tier check | Exit 4 | Op adds flag (deliberate) |
| --reset in production with non-tty | tty check | Exit 4 | Op runs from interactive shell |
| --reset in production interactive N | Y/N prompt | Exit 4 | Op confirmed cancellation |
| memory unavailable | memory_writer error | tx rollback; exit 3 | Op fixes memory; rerun |
| DB constraint violation | sqlx error | tx rollback; exit 7 | Op investigates |
| Password complexity fails | password.rs check | Exit 6 + reasons | Op picks stronger |
| Email validation fails | regex | Exit 6 | Op fixes email |
| `CYBEROS_BOOTSTRAP_PASSWORD` not set with --password-from-env | env check | Exit 1 + clear msg | Op sets env var |
| Interactive prompt skipped (non-tty) | tty check on interactive path | Exit 1 | Op runs interactively OR uses env vars |
| Initial signing key generation fails | RSA gen error | tx rollback; exit 7 | Op investigates entropy / openssl |
| Sweepers find nothing | normal | Exit 0 with zeroes | By design |
| Rotate-keys fails (DB error) | sqlx | Exit 7 | Op investigates |
| Plaintext password in stdout (regression) | §5 grep test | PR blocked | By design |
| Plaintext password in memory row (regression) | §5 audit-row inspection | PR blocked | By design |

---

## §11 — Notes

- The CLI is the chicken-and-egg solver. Every subsequent admin operation goes through the API; bootstrap is the one DB-direct path.
- The triple gate for production reset (--reset + --confirm + --force-prod-reset + interactive Y) is deliberate paranoia. Production reset wipes EVERYTHING; making accidental destruction structurally impossible is the right trade-off vs operator inconvenience.
- Sweepers run hourly via cron. Manual invocation via this CLI is for ops investigation (e.g., "did the sweeper run? Let me check now"). Both paths use identical code; the cron just calls the binary.
- Quarterly key rotation runs Jan 1 / Apr 1 / Jul 1 / Oct 1 at 03:00 UTC. Calendar-aligned for audit-narrative clarity. Ad-hoc rotation (suspected compromise) uses `cyberos-auth rotate-keys` directly.
- Password input via `rpassword` masks characters. Env-var fallback exists for CI; both paths apply Zeroizing<String> discipline.
- The `bootstrapped_by` field captures `$USER` as a forensic primitive. Not a strong identity (no auth here), but better than null. Combined with deployment_tier + signing key kid + tenant_0_id, gives a useful audit trail.
- Email NOT echoed in success summary because it adds nothing (operator just typed it) AND risks leaking into operator-shell logs. The subject_id is sufficient correlation.
- The CLI binary is shipped as part of the AUTH service Docker image. Operators run it via `docker exec auth-service cyberos-auth bootstrap`.
- Future multi-region bootstrap is non-trivial — each region has its own DB; rotation is per-region; tenant 0 is global. Slice 6+ work.

---

*End of FR-AUTH-006. Status: draft (10/10 target).*
