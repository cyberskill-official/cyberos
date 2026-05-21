---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-021
title: "cyberos-ai operator CLI (usage · models · policy · failover · invoice · breaker · expiry · memory) with --confirm + --json + audit"
module: AI
priority: MUST
status: done
verify: T
phase: P0
milestone: P0 · slice 5
slice: 5
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_frs: [FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004, FR-AI-005, FR-AI-007, FR-AI-008, FR-AI-009, FR-AI-014, FR-AI-015, FR-AI-022]
depends_on: [FR-AI-005, FR-AI-008, FR-AI-002, FR-AI-004, FR-AI-009]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#operator-cli
  - website/docs/runbooks/ai-gateway-ops.html
source_decisions:
  - DEC-097 (ops self-service requirement; reduce founder bandwidth on routine ops)
  - DEC-099 (single CLI binary; subcommands not separate scripts)
  - archive/2026-05-14/RESEARCH_REVIEW.md §7.1 (CLI auth via short-lived token + role-gated subcommands)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/bin/cyberos_ai.rs
  - services/ai-gateway/src/cli/mod.rs
  - services/ai-gateway/src/cli/auth.rs
  - services/ai-gateway/src/cli/output.rs
  - services/ai-gateway/src/cli/usage.rs
  - services/ai-gateway/src/cli/models.rs
  - services/ai-gateway/src/cli/policy.rs
  - services/ai-gateway/src/cli/failover.rs
  - services/ai-gateway/src/cli/invoice.rs
  - services/ai-gateway/src/cli/breaker.rs
  - services/ai-gateway/src/cli/expiry.rs
  - services/ai-gateway/src/cli/memory.rs
  - services/ai-gateway/src/cli/exit_codes.rs
  - services/ai-gateway/src/cli/json_schemas.rs
  - services/ai-gateway/tests/cli_test.rs
  - services/ai-gateway/tests/cli_audit_test.rs
  - services/ai-gateway/tests/cli_failover_drill_safety_test.rs
  - services/ai-gateway/tests/cli_json_schema_test.rs
  - services/ai-gateway/docs/cli-reference.md
modified_files:
  - services/ai-gateway/Cargo.toml                                # clap@4, comfy-table@7, jsonschema@0.18
  - services/ai-gateway/src/memory_writer.rs                       # add canonical::cli_* row builders
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,docs}/**
  - bash: cargo build --bin cyberos-ai --release
  - bash: cargo test -p cyberos-ai-gateway cli
disallowed_tools:
  - skip memory audit row when CLI mutates tenant policy or breaker state (per §1 #4)
  - allow destructive operations without `--confirm` (per §1 #5)
  - emit unstable JSON shape (per §1 #8 — JSON schema is a versioned contract)
  - bypass operator authentication (per §1 #6 — every mutating call requires `CYBEROS_AI_OPERATOR_TOKEN`)
  - run `failover drill` against production traffic without safety guard (per §1 #11)

# ───── Estimated work ─────
effort_hours: 14
sub_tasks:
  - "0.5h: clap-based CLI structure (subcommands + global flags --json --confirm --tenant)"
  - "0.5h: `cli/auth.rs` — operator token verification + role-gating (read vs mutate)"
  - "0.5h: `cli/exit_codes.rs` — re-export shared `cyberos-cli-exit::ExitCode` (0=ok, 1=user-error, 2=auth-failed, 3=remote-unreachable, 4=destructive-without-confirm, 5=already-initialised, 6=schema-violation, 7=internal-error)"
  - "0.5h: `cli/json_schemas.rs` + JSON schema files (versioned per command output)"
  - "1.0h: `usage` (per-tenant MTD spend, calls, top models; --month, --tenant, --json)"
  - "0.5h: `models list` + `models pricing`"
  - "1.0h: `policy set <tenant> --field=value --confirm` (multi-field mutation; emits diff for review)"
  - "0.5h: `policy validate <yaml-file>` (no remote calls; validates schema only)"
  - "1.0h: `policy diff <tenant> --vs <yaml-file>` (compare against pending change)"
  - "1.0h: `failover drill <provider:model> --duration <s> --confirm` with safety guard"
  - "1.0h: `invoice export <tenant> --period YYYY-MM --format csv|json|pdf` (PDF via wkhtmltopdf)"
  - "0.5h: `breaker status` + `breaker reset <provider:model> --confirm`"
  - "0.5h: `expiry status` + `expiry repair --confirm` (FR-AI-004 dedupe)"
  - "0.5h: `memory emit --dry-run <yaml>` + `memory emit --confirm <yaml>` (canonical-row builder + emit)"
  - "0.5h: `memory audit-trail <tenant> --since <iso8601>` (search + filter ai.* rows)"
  - "0.5h: memory audit row builders for every mutating command (canonical::cli_policy_updated, etc.)"
  - "1.0h: cli_test.rs — smoke tests per subcommand"
  - "1.0h: cli_audit_test.rs — every mutating command emits the expected audit row"
  - "0.5h: cli_failover_drill_safety_test.rs — refuses without `--prod-confirmed-aware` flag in production env"
  - "0.5h: cli_json_schema_test.rs — every --json output validates against the documented schema"
  - "0.5h: cli-reference.md generation (clap-derived help → markdown)"
  - "0.5h: Shell completions (bash, zsh, fish) via clap_complete"
risk_if_skipped: "Ops tasks (cap adjustments, failover testing, invoice exports, hold dedupe, breaker reset) require direct Postgres + memory access. Founder bandwidth bottleneck (RSK-09 from research review). FR-AI-004's crash-recovery path explicitly DEPENDS on this CLI's `expiry repair` command for dedup. Without the CLI, every ops question becomes a code-change PR — a 30-minute ticket becomes a 3-hour deploy. Worse: routine mutations (cap bumps for new tenants) become founder-only operations, blocking onboarding throughput at the slowest possible bottleneck."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** ship a `cyberos-ai` operator CLI binary providing ops-level commands. Each subcommand obeys the following:

1. **MUST** be deterministic: same inputs (CLI args + Postgres + memory state) → same outputs. Idempotency is required for read commands; mutating commands must emit the same audit row regardless of repeat invocation.
2. **MUST** print human-readable output by default using `comfy-table` for tabular data; the `--json` flag produces machine-readable output validated against a JSON schema versioned per command (per §1 #8).
3. **MUST** support `--tenant <id>` filtering on every relevant subcommand (usage, policy, invoice, audit-trail). Without `--tenant`, the command operates across all tenants (admin-token required per §1 #6).
4. **MUST** emit a memory audit row for ANY mutating operation (policy set, breaker reset, failover drill, expiry repair, memory emit) via the dedicated `canonical::cli_*` builders. The row carries `command`, `args`, `operator_id` (from token), `request_id`, `outcome`, AND a SHA-256 of the full command line for replay identification. Audit-before-action invariant from FR-AI-001 §1 #6 applies.
5. **MUST** refuse mutating operations without an explicit `--confirm` flag. Without `--confirm`, the command prints a DIFF (current state → proposed state) and exits with code 4 (`DESTRUCTIVE_WITHOUT_CONFIRM`). The diff is human-readable AND parseable (operators can pipe to `tee` to log review evidence).
6. **MUST** authenticate the operator via `CYBEROS_AI_OPERATOR_TOKEN` environment variable (short-lived JWT signed by the deployment secret). The token carries `operator_id` (kebab-case email) and `roles` (`read | mutate | admin`). Read-only commands accept any role; mutating commands require `mutate` or `admin`; the `failover drill` and `expiry repair` commands require `admin`. Missing or invalid token exits with code 2 (`AUTH_FAILED`); insufficient role exits with code 2 with a clear message.
7. **MUST** emit standardised exit codes per the shared `cyberos-cli-exit::ExitCode` re-export (cross-CLI contract; see also FR-AUTH-006):
    - `0` — success
    - `1` — user error (invalid args, validation failure, no such tenant)
    - `2` — auth failed OR insufficient role
    - `3` — remote unreachable (Postgres, memory, gateway)
    - `4` — destructive operation without `--confirm`
    - `5` — already initialised (reserved — bootstrap-only path; not raised by operator CLI commands but value is reserved cluster-wide)
    - `6` — schema violation (input YAML invalid)
    - `7` — internal error
    - `6` — internal error (panic, unexpected state)
   Exit codes are part of the CLI's stable contract; scripts and CI pipelines depend on them.
8. **MUST** version `--json` output schemas. Each command's JSON output starts with `{"schema_version":"v1",...}`; consumers parse the version first and select the appropriate parser. Schema files live in `cli/json_schemas/<command>.v1.json` (JSON Schema draft-07). Bumping a schema requires explicit FR amendment AND inclusion of the prior schema for one release cycle.
9. **MUST** include the subcommand catalogue (each subcommand has `--help` listing its specific flags):

   | Command | Purpose | Mutating | Required role | Audit row kind |
   |---|---|---|---|---|
   | `cyberos-ai usage [--tenant <id>] [--month YYYY-MM]` | MTD spend, call count, top models | No | read | — |
   | `cyberos-ai models list` | List supported aliases × providers × models | No | read | — |
   | `cyberos-ai models pricing` | Show cost-table rates | No | read | — |
   | `cyberos-ai policy set <tenant> --field=value [...] --confirm` | Update tenant policy fields | Yes | mutate | `ai.cli_policy_updated` |
   | `cyberos-ai policy validate <yaml-file>` | Validate without applying | No | read | — |
   | `cyberos-ai policy diff <tenant> --vs <yaml-file>` | Compare against pending change | No | read | — |
   | `cyberos-ai failover drill <provider:model> --duration <s> --confirm` | Force a 5xx storm to test failover | Yes | admin | `ai.cli_failover_drill` |
   | `cyberos-ai invoice export <tenant> --period YYYY-MM --format csv\|json\|pdf` | Generate invoice | No | read | `ai.cli_invoice_exported` (informational) |
   | `cyberos-ai breaker status` | Show all breaker states | No | read | — |
   | `cyberos-ai breaker reset <provider:model> --confirm` | Force breaker to Closed | Yes | mutate | `ai.cli_breaker_reset` |
   | `cyberos-ai expiry status` | Hold-expiry job health | No | read | — |
   | `cyberos-ai expiry repair --confirm` | Dedupe duplicate `ai.hold_expired` rows (FR-AI-004) | Yes | admin | `ai.cli_expiry_repaired` |
   | `cyberos-ai memory emit --dry-run <yaml>` | Validate canonical-row payload | No | read | — |
   | `cyberos-ai memory emit --confirm <yaml>` | Emit a manual canonical row | Yes | admin | `ai.cli_memory_emitted` |
   | `cyberos-ai memory audit-trail <tenant> --since <iso8601>` | Search memory ai.* rows | No | read | — |

10. **MUST** support `policy set` taking multiple `--field=value` flags in one invocation: `policy set org:cyberskill --cap-usd=200 --zdr-required=true --residency=eu-1 --confirm`. The diff (printed without `--confirm` AND echoed with `--confirm`) shows BEFORE/AFTER for every changed field. Atomicity: either all fields apply OR none (single Postgres transaction).
11. **MUST** apply a safety guard to `failover drill`: in a production environment (env var `CYBEROS_DEPLOYMENT_TIER=production`), the command additionally requires `--prod-confirmed-aware` flag AND prompts for an interactive Y/N confirmation with the deployment tier displayed. Drills in `production` are not forbidden (they're sometimes necessary) but must be deliberate.
12. **MUST** produce parseable diff output for `policy set` without `--confirm`. The diff is printed in unified-diff-like format AND, with `--json`, also as a structured JSON: `{"changes":[{"field":"cap_usd","before":150,"after":200},...]}`. Operators can pipe to `tee policy_change.diff` for change-management evidence.
13. **MUST** include `policy validate` as a no-network operation: parses the YAML, runs FR-AI-005's schema validator, prints errors (line-by-line if multiple) OR success. Useful for pre-deploy CI gates that validate policy YAMLs before merge.
14. **MUST** redact secrets from CLI output: tenant policy fields marked `secret: true` in FR-AI-005's schema (e.g., API keys for tenant-supplied managed-provider credentials) are displayed as `<REDACTED>` in both human and JSON output. Output of `policy diff` MUST NOT echo the secret value; operators see "secret-changed" indicator without the value.
15. **SHOULD** generate shell completions for bash, zsh, and fish via `clap_complete`. The `cyberos-ai completions <shell>` subcommand outputs the completion script.
16. **SHOULD** emit OTel metrics on CLI use:
    - `cyberos_ai_cli_invocations_total{command, outcome, role}` (counter).
    - `cyberos_ai_cli_latency_ms{command}` (histogram; surfaces slow commands).
    - `cyberos_ai_cli_destructive_without_confirm_total{command}` (counter; tracks operator-error frequency).

---

## §2 — Why this design (rationale for humans)

**Why a single CLI binary?** One install target, one mental model for ops. Operators learn `cyberos-ai <subcommand>` once and discover features via `--help`. Scattered scripts (`update_cap.sh`, `reset_breaker.sh`, `dedupe_holds.sh`) become a documentation problem the moment there are more than three; a single binary with subcommands is a navigation problem solved by clap. The packaging story (one Rust binary, statically linked) ships in the existing release artefact pipeline without separate dependency management.

**Why JSON output mode (§1 #2)?** Future scripts + dashboards consume CLI output. Without `--json`, parsing the human-readable output is fragile (table layout changes, columns reordered, Unicode in cell values). JSON output is a stable contract. Versioning the schema (§1 #8) lets us evolve fields without breaking existing consumers. The split (human default + JSON opt-in) means interactive ops sessions are pleasant AND scripts are robust.

**Why audit rows on mutating operations (§1 #4)?** The same audit-before-action invariant that the gateway itself enforces. Operator mutations are part of the system's history; "who reset the breaker last Tuesday" must be answerable from the memory chain. Without CLI audit rows, mutations become invisible (the gateway records the EFFECT but not the OPERATOR). The SHA-256 of the command line in the audit row is a small but useful detail — it lets a forensic replay reconstruct the exact invocation.

**Why `--confirm` requirement (§1 #5)?** A keystroke error like `cyberos-ai policy set org:cyberskill --cap-usd 0 --confirm` (intended `200`) instantly zeros a tenant's budget. Two-step confirmation (run-first-without-confirm-to-see-diff, then run-with-confirm) prevents the worst class of operator errors. The diff is the safety net; the operator visually verifies the change before re-running. Exit code 4 is distinct from "user error" (exit 1) so scripts can detect "operator forgot --confirm" specifically.

**Why operator token authentication (§1 #6)?** The CLI runs from operator workstations / bastion hosts; binary access alone shouldn't grant mutate authority (a compromised laptop ≠ compromised production). The short-lived JWT model (token expires in 8 hours; operator refreshes via internal SSO) limits blast radius. Role-gating prevents read-only operators from accidentally invoking `failover drill` (which is admin-only).

**Why standardised exit codes (§1 #7)?** CI scripts wrap CLI calls; they need to distinguish failure modes. "User error" (exit 1) means fix the args; "auth failed" (exit 2) means refresh the token; "remote unreachable" (exit 3) means a transient infrastructure issue (retry). Without distinct codes, every failure looks the same and scripts can't react appropriately.

**Why versioned JSON schemas (§1 #8)?** The JSON output IS the API for consumer scripts. Adding a field is non-breaking; removing or renaming a field IS breaking. Schema versions make this explicit: `"schema_version":"v1"` consumers parse v1; future v2 consumers parse v2; both versions can coexist for one release cycle. Without versioning, a subtle schema change breaks every downstream script silently.

**Why safety guard on `failover drill` (§1 #11)?** A drill deliberately trips a circuit breaker — provider calls fail for the drill's duration. In `staging`, this is fine (no real users). In `production`, this affects real tenants for the drill window. The double-opt-in (`--confirm` + `--prod-confirmed-aware` + interactive prompt) makes production drills DELIBERATE — operators have to consciously choose to impact production traffic. Production drills aren't forbidden (sometimes you need to validate failover under real load) but they require ceremony.

**Why secret redaction (§1 #14)?** Tenant policies may carry tenant-supplied API keys (for managed-provider credentials the tenant owns). Echoing those in CLI output (especially in shared sessions, logged commands, or piped output) is a data-exposure incident. The `secret: true` schema annotation + `<REDACTED>` rendering is the boundary; operators see "the field changed" without seeing the value.

**Why `policy diff` as a separate command (§1 #9)?** `policy set --confirm` shows the diff after the fact (in audit + echo). `policy diff <tenant> --vs <file>` shows the diff WITHOUT mutating. This supports change-management workflows: "what would this YAML change?" → review diff → approve → apply via `policy set`. Separating preview from apply is operational hygiene.

**Why is this MUST priority (vs the COULD on FR-AI-020)?** Ops tasks aren't optional. FR-AI-004's crash-recovery path explicitly REQUIRES `expiry repair` to clean up duplicate hold-expired rows after a crash. Without the CLI, the dedupe is manual SQL — error-prone, no audit trail, founder-only. FR-AI-021 is in the critical path for Slice 5 ops.

**Why `memory emit --dry-run` (§1 #9)?** Manual canonical-row emission is rare but necessary (e.g., backfilling rows missed during an outage). `--dry-run` validates the YAML payload against the row's schema without writing — operators can construct + validate before emitting. Same precedence as `policy validate`.

---

## §3 — API contract (formal spec for AI-agent implementers)

### CLI structure (clap)

```rust
// services/ai-gateway/src/cli/mod.rs
use clap::{Parser, Subcommand, Args};
use chrono::NaiveDate;

#[derive(Parser)]
#[command(name = "cyberos-ai", version)]
pub struct Cli {
    #[arg(long, global = true)] pub json: bool,
    #[arg(long, global = true)] pub confirm: bool,
    #[command(subcommand)] pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Usage(UsageArgs),
    Models(ModelsArgs),
    Policy(PolicyArgs),
    Failover(FailoverArgs),
    Invoice(InvoiceArgs),
    Breaker(BreakerArgs),
    Expiry(ExpiryArgs),
    Memory(MemoryArgs),
    Completions(CompletionsArgs),
}

#[derive(Args)]
pub struct UsageArgs {
    #[arg(long)] pub tenant: Option<String>,
    #[arg(long)] pub month: Option<NaiveDate>,
}

#[derive(Args)]
pub struct PolicyArgs {
    #[command(subcommand)] pub action: PolicyAction,
}

#[derive(Subcommand)]
pub enum PolicyAction {
    Set { tenant: String,
          #[arg(long)] cap_usd: Option<f64>,
          #[arg(long)] zdr_required: Option<bool>,
          #[arg(long)] residency: Option<Residency>,
          #[arg(long)] allowed_personas: Option<Vec<String>> },
    Validate { yaml_file: PathBuf },
    Diff { tenant: String, #[arg(long = "vs")] vs: PathBuf },
}

#[derive(Args)]
pub struct FailoverArgs {
    #[command(subcommand)] pub action: FailoverAction,
}

#[derive(Subcommand)]
pub enum FailoverAction {
    Drill {
        target: String,                                   // "bedrock:claude-3-5-sonnet"
        #[arg(long, default_value_t = 60)] duration: u32,  // seconds
        #[arg(long)] prod_confirmed_aware: bool,
    },
}

// ... similar Args structs for Invoice, Breaker, Expiry, Memory ...
```

### Exit codes

Re-exported from the shared crate **`cyberos-cli-exit`** (single source of truth across all CyberOS CLIs — see also FR-AUTH-006). CI scripts depend on these numerical values; they are a stable contract once locked.

```rust
// crates/cyberos-cli-exit/src/lib.rs  (shared across services/ai-gateway, services/auth, etc.)
#[repr(i32)]
pub enum ExitCode {
    Ok                          = 0,
    UserError                   = 1,
    AuthFailed                  = 2,
    RemoteUnreachable           = 3,
    DestructiveWithoutConfirm   = 4,
    AlreadyInitialised          = 5,
    SchemaViolation             = 6,
    InternalError               = 7,
}

// services/ai-gateway/src/cli/exit_codes.rs
pub use cyberos_cli_exit::ExitCode;
```

> **Note on numeric stability:** values 0–7 are normative across all CyberOS CLIs.  Module-specific codes start at `100` (AI), `200` (AUTH), `300` (memory), `400` (OBS) — any per-module extension must avoid collision with the shared range above.

### Authentication

```rust
// services/ai-gateway/src/cli/auth.rs

pub struct OperatorClaims {
    pub operator_id: String,           // "<localpart>@cyberos.world"
    pub roles: Vec<Role>,              // ["read", "mutate"] or ["admin"]
    pub exp: i64,                      // unix timestamp
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role { Read, Mutate, Admin }

pub fn parse_token(token: &str) -> Result<OperatorClaims, AuthError> {
    let claims: OperatorClaims = jsonwebtoken::decode(token, &PUBKEY, &Validation::default())?
        .claims;
    Ok(claims)
}

pub fn require_role(claims: &OperatorClaims, needed: Role) -> Result<(), AuthError> {
    if claims.roles.contains(&Role::Admin) { return Ok(()); }
    if needed == Role::Read { return Ok(()); }   // any role can read
    if needed == Role::Mutate && claims.roles.contains(&Role::Mutate) { return Ok(()); }
    Err(AuthError::InsufficientRole { has: claims.roles.clone(), needs: needed })
}
```

### JSON schema versioning

```rust
// services/ai-gateway/src/cli/json_schemas.rs
pub fn validate_output<T: Serialize>(command: &str, version: &str, value: &T) -> Result<(), String> {
    let schema_path = format!("src/cli/json_schemas/{command}.{version}.json");
    let schema = serde_json::from_str(&std::fs::read_to_string(schema_path).unwrap()).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();
    let json = serde_json::to_value(value).unwrap();
    let result = compiled.validate(&json);
    if let Err(errors) = result {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        return Err(msgs.join("; "));
    }
    Ok(())
}
```

### Audit row builders

```rust
// services/ai-gateway/src/memory_writer.rs (additions)
pub mod canonical {
    pub fn cli_policy_updated(
        operator_id: &str, tenant: &str, changes: &[FieldChange],
        command_sha256: &str, request_id: &str,
    ) -> AuditRow {
        AuditRow {
            kind: "ai.cli_policy_updated".into(),
            payload: serde_json::json!({
                "operator_id": operator_id,
                "tenant": tenant,
                "changes": changes,
                "command_sha256": command_sha256,
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }

    pub fn cli_breaker_reset(operator_id: &str, target: &str, command_sha256: &str, request_id: &str) -> AuditRow {
        AuditRow { kind: "ai.cli_breaker_reset".into(), payload: serde_json::json!({
            "operator_id": operator_id, "target": target,
            "command_sha256": command_sha256, "request_id": request_id,
        }), ..Default::default() }
    }

    pub fn cli_failover_drill(operator_id: &str, target: &str, duration_s: u32,
                              tier: &str, command_sha256: &str, request_id: &str) -> AuditRow {
        AuditRow { kind: "ai.cli_failover_drill".into(), payload: serde_json::json!({
            "operator_id": operator_id, "target": target, "duration_s": duration_s,
            "deployment_tier": tier, "command_sha256": command_sha256, "request_id": request_id,
        }), ..Default::default() }
    }

    pub fn cli_expiry_repaired(operator_id: &str, deduped_count: u32,
                               command_sha256: &str, request_id: &str) -> AuditRow {
        AuditRow { kind: "ai.cli_expiry_repaired".into(), payload: serde_json::json!({
            "operator_id": operator_id, "deduped_count": deduped_count,
            "command_sha256": command_sha256, "request_id": request_id,
        }), ..Default::default() }
    }

    pub fn cli_memory_emitted(operator_id: &str, emitted_kind: &str,
                             command_sha256: &str, request_id: &str) -> AuditRow {
        AuditRow { kind: "ai.cli_memory_emitted".into(), payload: serde_json::json!({
            "operator_id": operator_id, "emitted_kind": emitted_kind,
            "command_sha256": command_sha256, "request_id": request_id,
        }), ..Default::default() }
    }
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **`cyberos-ai --version` returns binary version** — Exit 0 with version string.
2. **`cyberos-ai usage --tenant org:test`** prints MTD spend, call count, top 5 models. Exit 0.
3. **`cyberos-ai usage ... --json`** returns JSON validating against `usage.v1.json` schema.
4. **`cyberos-ai policy set <t> --cap-usd 200`** without `--confirm` prints diff + exits with code 4.
5. **`cyberos-ai policy set <t> --cap-usd 200 --confirm`** updates Postgres + emits `ai.cli_policy_updated` memory row.
6. **Multi-field policy set atomic** — `policy set <t> --cap-usd=200 --zdr-required=true --confirm` updates BOTH fields in a single transaction; partial failure rolls back.
7. **`cyberos-ai breaker status`** shows current breaker state for every (provider, model) registered.
8. **`cyberos-ai breaker reset bedrock:claude-3-5-sonnet --confirm`** transitions Open→Closed + emits `ai.cli_breaker_reset`.
9. **`cyberos-ai invoice export <t> --period 2026-05 --format json`** produces all `ai.invocation` rows for that period.
10. **`cyberos-ai expiry repair --confirm`** removes duplicate `ai.hold_expired` rows (FR-AI-004 dedupe).
11. **Each subcommand has `--help`** listing flags + exits 0.
12. **Auth: missing token exits 2** — Unset `CYBEROS_AI_OPERATOR_TOKEN`; any subcommand exits with code 2 and message `auth_failed: missing CYBEROS_AI_OPERATOR_TOKEN`.
13. **Auth: insufficient role exits 2** — read-only token + `policy set --confirm` exits 2 with `insufficient_role: needed mutate; have [read]`.
14. **`failover drill` in production requires `--prod-confirmed-aware`** — `CYBEROS_DEPLOYMENT_TIER=production` + drill without flag exits 4 with `production drill requires --prod-confirmed-aware AND interactive Y confirmation`.
15. **`policy diff` no-mutate** — Reads tenant policy from Postgres, parses YAML file, prints unified-diff format without writing; exits 0.
16. **`policy validate` no-network** — Parses YAML; calls FR-AI-005 schema validator; prints errors line-by-line on failure (exit 5) or `valid` on success (exit 0).
17. **JSON output validates against versioned schema** — `cli_json_schema_test.rs` runs each subcommand with `--json`; output passes JSON Schema validation against `cli/json_schemas/<command>.v1.json`.
18. **Secret redaction** — Policy field marked `secret: true` is displayed as `<REDACTED>` in both human and JSON output; `policy diff` shows `secret-changed: true` indicator without the value.
19. **Shell completions emit valid scripts** — `cyberos-ai completions bash` produces a sourceable bash completion script.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/cli_test.rs
use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn version_returns_binary_version() {
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.arg("--version").assert().success().stdout(predicates::str::contains("cyberos-ai"));
}

#[test]
fn usage_prints_table_for_tenant() {
    set_test_token_with_role("read");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["usage", "--tenant", "org:test"]).assert().success()
        .stdout(predicates::str::contains("PERIOD:"))
        .stdout(predicates::str::contains("CAP:"))
        .stdout(predicates::str::contains("SPENT:"));
}

#[test]
fn policy_set_without_confirm_exits_4_with_diff() {
    set_test_token_with_role("mutate");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["policy", "set", "org:test", "--cap-usd", "200"])
        .assert().code(4)
        .stdout(predicates::str::contains("cap_usd: 150 → 200"));
}

#[test]
fn policy_set_with_confirm_updates_and_emits_audit() {
    set_test_token_with_role("mutate");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["policy", "set", "org:test", "--cap-usd", "200", "--confirm"])
        .assert().success();
    let row = memory_test_helper::find_latest_row("ai.cli_policy_updated").unwrap();
    assert_eq!(row.payload["tenant"], "org:test");
    assert_eq!(row.payload["changes"][0]["field"], "cap_usd");
    assert_eq!(row.payload["changes"][0]["before"], 150.0);
    assert_eq!(row.payload["changes"][0]["after"], 200.0);
}

#[test]
fn missing_token_exits_2() {
    std::env::remove_var("CYBEROS_AI_OPERATOR_TOKEN");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["usage", "--tenant", "org:test"])
        .assert().code(2)
        .stderr(predicates::str::contains("auth_failed"));
}

#[test]
fn insufficient_role_exits_2() {
    set_test_token_with_role("read");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["policy", "set", "org:test", "--cap-usd", "200", "--confirm"])
        .assert().code(2)
        .stderr(predicates::str::contains("insufficient_role"));
}

#[test]
fn failover_drill_in_production_requires_extra_flag() {
    set_test_token_with_role("admin");
    std::env::set_var("CYBEROS_DEPLOYMENT_TIER", "production");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["failover", "drill", "bedrock:claude-3-5-sonnet", "--duration", "30", "--confirm"])
        .assert().code(4)
        .stderr(predicates::str::contains("--prod-confirmed-aware"));
}

#[test]
fn breaker_reset_with_confirm_emits_audit() {
    set_test_token_with_role("mutate");
    breaker::force_open("bedrock:claude-3-5-sonnet");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["breaker", "reset", "bedrock:claude-3-5-sonnet", "--confirm"])
        .assert().success();
    assert_eq!(breaker::state("bedrock:claude-3-5-sonnet"), BreakerState::Closed);
    let row = memory_test_helper::find_latest_row("ai.cli_breaker_reset").unwrap();
    assert_eq!(row.payload["target"], "bedrock:claude-3-5-sonnet");
}

#[test]
fn expiry_repair_dedupes_duplicate_rows() {
    set_test_token_with_role("admin");
    test_helper::insert_duplicate_hold_expired_rows(5);
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    cmd.args(&["expiry", "repair", "--confirm"])
        .assert().success()
        .stdout(predicates::str::contains("deduped: 5"));
    let row = memory_test_helper::find_latest_row("ai.cli_expiry_repaired").unwrap();
    assert_eq!(row.payload["deduped_count"], 5);
}

#[test]
fn json_output_validates_against_usage_v1_schema() {
    set_test_token_with_role("read");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    let output = cmd.args(&["usage", "--tenant", "org:test", "--json"])
        .output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "v1");
    let schema = serde_json::from_str(&std::fs::read_to_string("src/cli/json_schemas/usage.v1.json").unwrap()).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();
    assert!(compiled.is_valid(&json));
}

#[test]
fn secret_redacted_in_output() {
    set_test_token_with_role("read");
    test_helper::set_tenant_secret_field("org:test", "openai_api_key", "sk-real-key");
    let mut cmd = Command::cargo_bin("cyberos-ai").unwrap();
    let stdout = String::from_utf8(cmd.args(&["policy", "validate", "tenants/org_test/policy.yaml"])
        .output().unwrap().stdout).unwrap();
    assert!(stdout.contains("<REDACTED>"));
    assert!(!stdout.contains("sk-real-key"));
}
```

```bash
cargo build --bin cyberos-ai --release
./target/release/cyberos-ai --version
./target/release/cyberos-ai usage --tenant org:test --month 2026-05 --json | jq .schema_version
cargo test -p cyberos-ai-gateway cli
```

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/bin/cyberos_ai.rs
use cyberos_ai_gateway::cli::{Cli, Command, exit_codes::ExitCode, auth};
use clap::Parser;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let token = std::env::var("CYBEROS_AI_OPERATOR_TOKEN").unwrap_or_default();
    let claims = match auth::parse_token(&token) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("auth_failed: {e}");
            process::exit(ExitCode::AuthFailed as i32);
        }
    };
    let pool = build_pool().await.unwrap_or_else(|e| {
        eprintln!("remote_unreachable: {e}");
        process::exit(ExitCode::RemoteUnreachable as i32);
    });

    let result = match cli.command {
        Command::Usage(args)        => cli::usage::run(args, &cli, &claims, &pool).await,
        Command::Models(args)       => cli::models::run(args, &cli, &claims, &pool).await,
        Command::Policy(args)       => cli::policy::run(args, &cli, &claims, &pool).await,
        Command::Failover(args)     => cli::failover::run(args, &cli, &claims, &pool).await,
        Command::Invoice(args)      => cli::invoice::run(args, &cli, &claims, &pool).await,
        Command::Breaker(args)      => cli::breaker::run(args, &cli, &claims, &pool).await,
        Command::Expiry(args)       => cli::expiry::run(args, &cli, &claims, &pool).await,
        Command::Memory(args)        => cli::memory::run(args, &cli, &claims, &pool).await,
        Command::Completions(args)  => cli::completions::run(args).await,
    };
    match result {
        Ok(_)  => process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            process::exit(e.exit_code() as i32);
        }
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-005** — `policy::validate_yaml()` + `policy::apply()` exposed to the CLI; this FR depends on the schema being stable.
- **FR-AI-008** — `breaker::status_all()` + `breaker::reset(target)`; the breaker module exposes admin entry-points.
- **FR-AI-001** — Cost-ledger Postgres tables for `usage` + `invoice` aggregations.
- **FR-AI-002** — `ai.invocation` rows are the source for invoice export.
- **FR-AI-003** — memory audit-row bridge; this FR adds `canonical::cli_*` builders.
- **FR-AI-004** — `expiry_repair` directly closes FR-AI-004's slice-1 dedup limitation (AC #9).
- **FR-AI-007** — `cost_table::lookup()` for `models pricing`.
- **FR-AI-014** — Persona-handle parsing for `policy set --allowed-personas`.
- **FR-AI-022 (downstream)** — OTel CLI metrics consumed by the operator dashboard.

### Concept dependencies (shared types)

- `Role::{Read, Mutate, Admin}` is the operator-permission primitive; tokens carry roles.
- `ExitCode` enum is a stable contract; CI scripts depend on numerical values.
- Versioned JSON schemas (`<command>.v1.json`) are the consumer-facing API.
- `FieldChange { field, before, after }` is the diff primitive used by `policy set`/`diff` AND audit rows.
- `command_sha256` is the audit-replay primitive (full command line hashed).

### Operational / external

- Rust crates: `clap@4` (with `derive` feature), `clap_complete@4` (shell completions), `comfy-table@7`, `tokio`, `sqlx`, `anyhow`, `jsonwebtoken`, `jsonschema@0.18`, `serde`, `serde_json`, `serde_yaml`, `chrono`, `predicates` (test-only), `assert_cmd` (test-only).
- Internal SSO that issues operator tokens (out of scope; assumed available).
- `wkhtmltopdf` (or `weasyprint`) on the operator's machine for `invoice export --format pdf`.
- Postgres + memory reachability from the operator's host (typically via bastion / VPN).

---

## §8 — Example payloads

### `usage` human output

```text
$ cyberos-ai usage --tenant org:cyberskill --month 2026-05
PERIOD: 2026-05  (1-15 of 31 days; period in progress)
TENANT: org:cyberskill
CAP:    $150.00
SPENT:  $47.23  (31.5% of cap)
CALLS:  3,421   (avg $0.0138 per call)

Top 5 models by spend:
MODEL                                       SPEND     CALLS    AVG
anthropic.claude-3-5-sonnet-20241022-v2:0   $32.10    1,205    $0.0266
anthropic.claude-3-haiku-20240307-v1:0      $ 8.45    1,890    $0.0045
gpt-4o-mini                                 $ 4.20      280    $0.0150
text-embedding-3-small                      $ 2.48        4    $0.6200
bge-m3                                      $ 0.00       42    $0.0000  (self-hosted)
```

### `usage --json` output (validates against `usage.v1.json`)

```json
{
  "schema_version": "v1",
  "tenant": "org:cyberskill",
  "month": "2026-05",
  "cap_usd": 150.0,
  "spent_usd": 47.23,
  "spent_pct": 31.5,
  "calls": 3421,
  "top_models_by_spend": [
    {"model": "anthropic.claude-3-5-sonnet-20241022-v2:0", "spend_usd": 32.10, "calls": 1205},
    {"model": "anthropic.claude-3-haiku-20240307-v1:0",     "spend_usd": 8.45,  "calls": 1890}
  ]
}
```

### `policy set --confirm` audit row

```json
{
  "kind": "ai.cli_policy_updated",
  "ts_ns": 1747526400000000000,
  "payload": {
    "operator_id": "stephen@cyberos.world",
    "tenant": "org:cyberskill",
    "changes": [
      { "field": "cap_usd", "before": 150.0, "after": 200.0 },
      { "field": "zdr_required", "before": false, "after": true }
    ],
    "command_sha256": "4b8c0d2f...",
    "request_id": "cli_01HZK..."
  }
}
```

### `policy set` diff (no `--confirm`)

```text
$ cyberos-ai policy set org:cyberskill --cap-usd 200 --zdr-required true
DIFF for tenant org:cyberskill:
  cap_usd:        150 → 200
  zdr_required:   false → true

To apply, re-run with --confirm
exit 4
```

### `breaker status` human output

```text
$ cyberos-ai breaker status
PROVIDER     MODEL                                STATE    FAILURES   NEXT_HALF_OPEN
bedrock      anthropic.claude-3-5-sonnet-...      Open     5 / 60s    in 18s
bedrock      anthropic.claude-3-haiku-...         Closed   0          —
anthropic    claude-3-5-sonnet-20241022           Closed   0          —
openai       gpt-4o                               HalfOpen 1 / 60s    probe in flight
bge          bge-m3                               Closed   0          —
```

### `failover drill` audit row

```json
{
  "kind": "ai.cli_failover_drill",
  "payload": {
    "operator_id": "stephen@cyberos.world",
    "target": "bedrock:claude-3-5-sonnet-20241022-v2:0",
    "duration_s": 60,
    "deployment_tier": "staging",
    "command_sha256": "9a7b...",
    "request_id": "cli_drill_01..."
  }
}
```

### `expiry repair` output

```text
$ cyberos-ai expiry repair --confirm
Scanning for duplicate ai.hold_expired rows…
Found 5 duplicates:
  hold_id=01HZK... (2 copies)
  hold_id=01HZL... (3 copies)
Deduped: 5 rows removed
Audit: ai.cli_expiry_repaired emitted
```

### Production drill safety guard output

```text
$ cyberos-ai failover drill bedrock:claude-3-5-sonnet --duration 60 --confirm
ERROR: production drill requires --prod-confirmed-aware AND interactive Y confirmation
       you are about to deliberately fail provider calls for 60s in PRODUCTION
       affected tenants: 47
       affected aliases: chat.smart, chat.long
exit 4
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Multi-operator collaboration (e.g., one operator initiates a drill, another approves) — out of scope; current model is single-operator-per-command.
- Web-based operator UI alongside the CLI — slice 6+; CLI is the slice-5 surface.
- `cyberos-ai watch` for real-time tail of OBS metrics — out of scope; `tail -f` on the OBS log suffices for slice 5.
- `cyberos-ai backfill` for historical cost-ledger reconstruction — slice 6+; current model is forward-only audit.
- Per-tenant operator scoping (an operator with mutate rights only for `tenant_a`) — slice 6+; current model is global mutate role.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Postgres unreachable | sqlx connect error | CLI exits 3 (RemoteUnreachable) with clear message | Operator investigates DB connectivity |
| memory unreachable | memory_writer error | Mutating commands fail (exit 3); read-only succeed (degraded) | Operator investigates memory |
| Invalid YAML in `policy validate` | parse error | Print errors line-by-line; exit 5 (SchemaViolation) | Operator fixes file |
| Missing `--confirm` on destructive | clap arg check | Print diff + exit 4 | Operator re-runs with `--confirm` |
| Output format mismatch (`--json` requested but error path uses text) | Always-JSON-on-error rule | Errors emit JSON when `--json` set | By design |
| Missing `CYBEROS_AI_OPERATOR_TOKEN` | auth check | Exit 2 with `auth_failed: missing token` | Operator obtains token via SSO |
| Expired token | JWT exp validation | Exit 2 with `auth_failed: token expired` | Operator refreshes token |
| Insufficient role (read-only attempts mutate) | role check | Exit 2 with `insufficient_role: needed mutate; have [read]` | Operator obtains escalated token |
| `failover drill` in production without safety flags | env-aware safety guard | Exit 4 with affected-tenants + affected-aliases displayed | Operator adds `--prod-confirmed-aware` AND confirms interactively |
| `policy set` partial failure (one field invalid) | atomic transaction rolls back | Exit 5 (SchemaViolation); no fields applied | Operator fixes the bad field; re-runs |
| `expiry repair` finds zero duplicates | informational | Exit 0 with `deduped: 0` | By design |
| `memory emit --confirm` row schema invalid | canonical-row validator | Exit 5 (SchemaViolation) | Operator fixes YAML payload |
| JSON output schema drift | `cli_json_schema_test` fails in CI | PR blocked | Bump schema version OR fix output |
| Secret leaks in CLI output | `cli_secret_redaction_test` asserts `<REDACTED>` | Test fails → PR blocked | Add field to redaction list |
| Operator token compromised | Audit chain shows mutations from unexpected operator_id | Sev-1 incident | Rotate token; investigate |
| Long-running command (e.g., invoice export 100K rows) | timeout in CI | Exit 6 (InternalError) | Operator investigates query plan; pagination |
| `policy diff` shows secret-changed without value | by design (§1 #14) | Operator sees boolean indicator | By design |
| Shell completion script malformed | `cyberos-ai completions bash` test asserts validity | Test fails | Fix clap_complete generation |
| Stale Postgres connection (long-lived CLI session) | sqlx pool reconnect | Self-resolves | By design |
| `failover drill` interactive prompt skipped via piped input | tty check | If non-tty AND production, exit 4 unconditionally | Operator runs from interactive shell |
| Audit row emit failure during destructive operation | memory_writer error AFTER mutation | Sev-1 alert: "mutation succeeded but audit row failed" | Operator manually emits via `memory emit` |

---

## §11 — Notes

- The CLI is the single point of ops surface. Avoid scattered scripts; everything goes through `cyberos-ai`. New ops capabilities ship as new subcommands, not as standalone tools.
- `failover drill` is the most novel command — it deliberately trips a circuit breaker to verify failover works under controlled conditions. Useful PRE-incident; sev-2 OBS event records the drill so operators reviewing post-incident logs can distinguish drills from real failures.
- `expiry repair` directly closes the slice-1 limitation in FR-AI-004 AC #9 (manual SQL was the workaround). The CLI command makes this a routine ops task instead of a database query.
- The role-gating model (read / mutate / admin) is intentionally simple. Per-tenant scoping (an operator who can only mutate `tenant_a`) is slice 6+; the slice-5 model is "global mutate" with per-command audit attribution.
- The `command_sha256` field in audit rows is small but powerful — a forensic investigator can reconstruct the exact CLI invocation from the chain. Without it, "operator X reset breaker Y" is the audit; with it, "operator X ran exactly `cyberos-ai breaker reset bedrock:claude-3-5-sonnet --confirm` as `command_sha256:<hash>`" is the audit.
- The `--prod-confirmed-aware` flag for `failover drill` is deliberately verbose. The flag name itself is the safety: nobody types it accidentally. Combined with the interactive Y prompt, accidental production drills become structurally impossible.
- The JSON schema versioning (§1 #8) is the consumer-script stability primitive. Without it, every minor output change risks breaking downstream automation. With it, schema evolution is opt-in.
- The `secret: true` schema annotation (§1 #14) is the operator-data-exposure boundary. Tenant-supplied API keys are real secrets; CLI output flows through ops sessions, terminals, ssh logs — anywhere unintended audiences might see. Defaulting to `<REDACTED>` is the safe choice; explicit `--show-secrets` (admin-only, audit-emitting) could be added in slice 6 if needed.
- `policy diff` is the change-management primitive. Production policy changes typically go: PR with new YAML → ops reviews diff via `policy diff` → approval → operator applies via `policy set --confirm`. The diff command is the review boundary.
- Shell completions (§1 #15) are a small UX win but a real one — operators discover commands and flags via tab-completion rather than re-reading `--help`.

---

*End of FR-AI-021. Status: draft (10/10 target).*
