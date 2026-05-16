---
# ───── Machine-readable frontmatter (parsed by fr-audit + future fr-catalog renderer) ─────
id: FR-AI-001
title: "AI Gateway cost-ledger pre-call check"
module: AI
priority: MUST
status: accepted
accepted_at: 2026-05-15
accepted_by: Stephen Cheng
verify: T
phase: P0
milestone: P0 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-AI-002, FR-AI-003, FR-AI-004, FR-AI-005]
depends_on: [FR-AI-003, FR-AI-005, FR-AI-007]
blocks: [FR-AI-002, FR-AI-004, FR-TEN-004]

# ───── Source contracts (where the spec authority lives) ─────
source_pages:
  - website/docs/modules/ai.html#cost-gate
  - website/docs/modules/ai.html#bigger-picture
source_decisions:
  - archive/2026-05-14/AUDIT_AND_PLAN.md §3.3 (P0 · slice 1 build placement)
  - archive/2026-05-14/RESEARCH_REVIEW.md §2.4 (reorder before AUTH)

# ───── Build envelope (read by AI agent before code-gen) ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/cost_ledger.rs
  - services/ai-gateway/src/policy.rs
  - services/ai-gateway/migrations/0001_cost_ledger.sql
  - services/ai-gateway/tests/cost_precheck_test.rs
modified_files:
  - services/ai-gateway/src/lib.rs
  - services/ai-gateway/src/handlers/chat.rs
  - services/ai-gateway/Cargo.toml
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,migrations}/**
  - bash: cargo test -p cyberos-ai-gateway
  - bash: cargo sqlx migrate run
  - brain: write memories/decisions/ai-invocations/* via canonical Writer (NOT directly)
disallowed_tools:
  - touch services/auth/** (AUTH is downstream of AI Gateway at P0 · slice 1)
  - in-place edit of services/ai-gateway/src/lib.rs `lib::run()` signature
  - bypass cost_ledger.precheck() from any new code path

# ───── Estimated work (for human triage + scheduling) ─────
effort_hours: 8
sub_tasks:
  - "0.5h: sqlx migration for cost_ledger table (3 columns + 2 indexes)"
  - "1.0h: TenantPolicy YAML parser + load-from-config"
  - "2.0h: precheck() function with token-estimate + cap check"
  - "1.5h: hold creation + 60s TTL job"
  - "1.5h: BRAIN audit row emission via Writer subprocess"
  - "1.5h: integration test (real Postgres container)"
risk_if_skipped: "Every consumer module (CUO, KB, CHAT) will hit raw provider APIs with no cost cap. Tenant surprise-bill blast radius is unbounded. This is the protocol-level invariant the P0 · slice 1 reorder exists to enforce."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** expose a `cost_ledger::precheck(request)` function that runs synchronously before any LLM-provider call. Given a `ChatCompleteRequest` with `tenant_id` and `agent_persona`, the function:

1. **MUST** estimate the request's USD cost from the prompt token count × the provider's per-1k-token rate at the resolved model alias.
2. **MUST** read the tenant's `monthly_cap_usd` and current MTD `spent_usd` from the `cost_ledger` table (Postgres).
3. **MUST** refuse with `402 PAYMENT_REQUIRED` if `estimated_cost + current_spent > monthly_cap_usd × 1.0` (the hard floor).
4. **SHOULD** emit a `warn_threshold_crossed` event to OBS if `current_spent` crosses `monthly_cap_usd × warn_threshold` (default 0.80).
5. **MUST** create a `cost_ledger_hold` row in Postgres with `estimated_usd`, `tenant_id`, `idempotency_key`, `expires_at = now() + 60s` before returning `allow`.
6. **MUST** emit one `ai.precheck` audit row on the local BRAIN via the canonical Writer subprocess before returning `allow` (audit-before-action invariant).
7. **MUST** return synchronously within 50ms p95 — the precheck is on the hot path of every chat call.
8. **MUST** load the cost-rate table from `services/ai-gateway/config/cost_rates.yaml` at gateway startup (loader machinery from FR-AI-005 is reused). The hardcoded `const HashMap` is REMOVED for slice 1; cost rates live in YAML from day 1.
9. **MUST** refuse with `503 POLICY_NOT_FOUND` when `policy::load_for_tenant()` returns `PolicyError::PolicyMissing`. No silent default policy. The error body MUST carry `{error: "tenant_policy_missing", tenant_id, contact: "ops@cyberos.world"}`.
10. **MUST** validate the `idempotency_key` as a free-string of length 1..=64 ASCII printable characters. Longer or shorter MUST return `400 BAD_REQUEST` with `{error: "invalid_idempotency_key"}`. The Writer's canonical-JSON serialiser rejects non-printable characters by design.
11. **MUST** emit the `ai.precheck` BRAIN audit row synchronously (subprocess fork; ~30ms latency). FR-AI-008 (slice 2 PyO3 spike) explores in-process PyO3 for the latency gain; until then, the audit-before-action invariant outweighs the latency cost.
12. **MUST** execute the cap-check and hold-creation inside a single Postgres `BEGIN ... COMMIT` block. The `SELECT … FOR UPDATE` on the `cost_ledger` row prevents two concurrent calls from both passing the cap-check when the sum of their estimated costs would exceed the cap. Without this serialisation, two near-simultaneous calls each see `current.spent_usd = $X` and each pass the check, then both insert holds — eventually exceeding cap.
13. **MUST** check `policy.ai_policy.allowed_personas` (if non-null) against `req.agent_persona`; refuse with `403 FORBIDDEN` and `{error: "persona_not_allowed", allowed: [...]}` if not in the list. This enforces the per-tenant persona-pinning policy from FR-AI-005's schema.
14. **SHOULD** emit OTel metrics: `ai_gateway_precheck_calls_total{outcome}` (counter; outcome ∈ allow/refuse/error), `ai_gateway_precheck_latency_ms` (histogram), `ai_gateway_holds_created_total` (counter), `ai_gateway_budget_warns_total{tenant_id}` (counter).

This is the cost-of-everything gate per the AI Gateway page §0 Role 1 and §2.5. Every consumer (CUO, KB, CHAT, PROJ inline genie, OBS auto-triage) inherits this gate; there is no bypass path.

---

## §2 — Why this design (rationale for humans)

**The naive design** would compute cost post-hoc — let the provider charge, then check budget. That works until the first tenant blows past their cap mid-month and discovers it on the invoice. The trust loss is permanent.

**The two-phase design** (precheck + post-reconcile, this FR + FR-AI-002) borrows from database transaction theory: estimate the cost first, hold it, then reconcile. Holds expire after 60s if the post-check never arrives — defensive. The pre-call cost overhead is ~5ms of Postgres I/O.

**The hard-stop default at 1.0×** (not 1.1× or "soft warn") is deliberate. Soft caps in finance always become hard losses when the tenant disputes the overage. The override path exists (`policy.emergency_override: true`) but requires CFO sign-off recorded in BRAIN — *not* an automatic fallback.

**Why this is FR-AI-001 specifically** (not FR-AUTH-001): per research review §2.4, AI Gateway ships at P0 · slice 1 *before* AUTH. The X-Tenant header (HMAC-signed by a deployment secret) substitutes for JWT auth in slice 1; once AUTH ships at P0 · slice 2, the tenant_id source switches to the AUTH JWT claim per AI Gateway page §2.5. This FR therefore uses the X-Tenant header path; FR-AI-006 (slice 2) replaces it with JWT extraction.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signature

```rust
// services/ai-gateway/src/cost_ledger.rs
pub async fn precheck(
    req: &ChatCompleteRequest,
    pool: &PgPool,
    policy: &TenantPolicy,
) -> Result<PrecheckOutcome, PrecheckError>;

pub enum PrecheckOutcome {
    Allow { hold_id: Uuid, estimated_usd: Decimal, ttl_seconds: u32 },
    Refuse { reason: RefuseReason, current_spent_usd: Decimal, cap_usd: Decimal },
}

pub enum RefuseReason {
    BudgetCapExceeded,
    TenantSuspended,           // reserved for slice 2 (TenantPolicy.suspended flag)
    ProviderUnavailable,       // resolved-provider cost-table missing
    InvalidIdempotencyKey,
    PersonaNotAllowed,         // §1 #13 — policy.allowed_personas restricts the persona
}

pub enum PrecheckError {
    DbError(sqlx::Error),
    BrainWriterFailed { stderr: String },
    PolicyLoadFailed,
    CostEstimateFailed { reason: String },
}
```

### HTTP behavior on the chat handler

| Caller intent | HTTP response |
|---|---|
| `precheck → Allow` then provider call succeeds | `200 OK` + chat response body; post-reconcile fires (FR-AI-002) |
| `precheck → Refuse{BudgetCapExceeded}` | `402 PAYMENT_REQUIRED` + `{error: "budget_cap_exceeded", current_spent_usd, cap_usd, suggest: "downshift to chat.fast or wait until next cycle"}` |
| `precheck → Refuse{TenantSuspended}` | `403 FORBIDDEN` + `{error: "tenant_suspended", contact: "billing@..."}` |
| `precheck → Refuse{ProviderUnavailable}` | `503 SERVICE_UNAVAILABLE` + `{error: "no_cost_table_for_model", model: "..."}` (don't proceed without cost data) |
| `precheck → Err(_)` | `500 INTERNAL_SERVER_ERROR` + opaque `request_id` (do NOT leak internals) |

### Postgres schema (`migrations/0001_cost_ledger.sql`)

```sql
CREATE TABLE cost_ledger (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       TEXT NOT NULL,
    period          DATE NOT NULL,             -- first-of-month UTC
    spent_usd       NUMERIC(12,4) NOT NULL DEFAULT 0,
    monthly_cap_usd NUMERIC(12,2) NOT NULL,
    UNIQUE (tenant_id, period)
);

CREATE TABLE cost_ledger_hold (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        TEXT NOT NULL,
    idempotency_key  TEXT NOT NULL,
    estimated_usd    NUMERIC(12,4) NOT NULL,
    resolved_provider TEXT NOT NULL,
    resolved_model   TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ NOT NULL,
    state            TEXT NOT NULL CHECK (state IN ('held','reconciled','expired','refused')),
    UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX cost_ledger_period_idx ON cost_ledger (tenant_id, period);
CREATE INDEX cost_ledger_hold_expiry_idx ON cost_ledger_hold (expires_at) WHERE state = 'held';
```

### Tenant policy YAML shape (FR-AI-005 builds the loader; this FR consumes it)

```yaml
# config/tenants/<tenant_id>.yaml (slice-1 location; moves to TEN module at P2)
tenant_id: org:cyberskill
ai_policy:
  monthly_cap_usd: 150
  warn_threshold: 0.80
  hard_stop: true
  emergency_override:
    enabled: true
    requires: ["cfo_signoff", "audit_row"]
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy path (allow)** — Given tenant `org:test-a` with `monthly_cap_usd: 100` and `spent_usd: 50`, when `precheck()` is called with an estimated cost of $5, it MUST return `Allow { hold_id, estimated_usd: 5.00, ttl_seconds: 60 }` and MUST insert exactly one row into `cost_ledger_hold` with `state='held'`.
2. **Refuse on over-budget** — Same tenant with `spent_usd: 98`, estimated cost $5, MUST return `Refuse { reason: BudgetCapExceeded }`; MUST NOT insert any hold row; the HTTP handler MUST return `402 PAYMENT_REQUIRED`.
3. **Exact-cap edge** — Tenant with `spent_usd: 95`, estimated cost $5.00, MUST return `Allow` (boundary inclusive: spent + estimated == cap is permitted; only strictly-over refused).
4. **Idempotent retry** — Calling `precheck` with the same `idempotency_key` twice MUST return the existing hold (same `hold_id`), MUST NOT insert a second row.
5. **Audit row emitted** — Every `Allow` MUST be preceded by a BRAIN audit row at `memories/decisions/ai-invocations/<ts_ns>_<tenant>_<idempotency_key>.md` with kind `ai.precheck`, `extra.tenant_id`, `extra.estimated_usd`, `extra.resolved_provider`. If BRAIN Writer fails, the function MUST return `Err(BrainWriterFailed)` and MUST NOT return `Allow`.
6. **Hold TTL** — A `cost_ledger_hold` row with `expires_at < NOW()` and `state='held'` MUST be transitioned to `state='expired'` by the cleanup job within 60s of expiry (cleanup job is FR-AI-004; this FR only writes the row correctly).
7. **Provider cost-table missing** — If `policy.primary_provider` resolves to a model alias with no entry in the cost-table fixture, `precheck()` MUST return `Refuse { reason: ProviderUnavailable }`; MUST emit a `Provider-Unavailable` warn-level log to OBS.
8. **Latency budget** — On a warm Postgres pool, `precheck()` MUST complete within 50ms p95 over a 1000-call integration test. Measured via `tokio::time::Instant`.

---

## §5 — Verification method

**Integration test:** `services/ai-gateway/tests/cost_precheck_test.rs`

```rust
#[tokio::test]
async fn precheck_allows_under_budget() {
    let env = TestEnv::new().await;
    env.seed_tenant("org:test-a", monthly_cap = 100, spent_usd = 50);
    let req = chat_request("org:test-a", prompt_tokens = 1000, model = "chat.smart");

    let outcome = cost_ledger::precheck(&req, &env.pool, &env.policy).await.unwrap();

    assert!(matches!(outcome, PrecheckOutcome::Allow { .. }));
    assert_eq!(env.count_holds("org:test-a").await, 1);
    assert!(env.brain_has_row("ai.precheck", &req.idempotency_key).await);
}
```

Full test suite (8 cases, one per acceptance criterion) at the same file. Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway cost_precheck
```

CI gate: this test file is in the `cyberos-ai-gateway` package; CI runs it on every PR touching `services/ai-gateway/**`. Failure blocks merge.

---

## §6 — Implementation skeleton (suggested scaffold for AI-agent code-gen)

```rust
// services/ai-gateway/src/cost_ledger.rs

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

const HOLD_TTL_SECONDS: u32 = 60;
const IDEMPOTENCY_KEY_MAX_LEN: usize = 64;

pub async fn precheck(
    req: &ChatCompleteRequest,
    pool: &PgPool,
    policy: &TenantPolicy,
) -> Result<PrecheckOutcome, PrecheckError> {
    // 0. Validate idempotency key (§1 #10)
    validate_idempotency_key(&req.idempotency_key)
        .map_err(|reason| PrecheckError::CostEstimateFailed { reason })?;

    // 0b. Persona pinning check (§1 #13)
    if let Some(allowed) = &policy.ai_policy.allowed_personas {
        if !allowed.contains(&req.agent_persona) {
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::PersonaNotAllowed,
                current_spent_usd: dec!(0),
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
    }

    // 1. Resolve provider + model from alias
    let (provider, model) = resolve_model_alias(&req.model_alias, policy)
        .ok_or(PrecheckError::CostEstimateFailed { reason: "no_provider".into() })?;

    // 2. Estimate cost
    let cost_per_1k = cost_table::lookup(&provider, &model)
        .ok_or_else(|| PrecheckError::CostEstimateFailed { reason: "no_cost_entry".into() })?;
    let estimated_usd = (Decimal::from(req.prompt_tokens) / Decimal::from(1000)) * cost_per_1k.input
                      + (Decimal::from(req.expected_completion_tokens) / Decimal::from(1000)) * cost_per_1k.output;

    // 3. Open transaction (§1 #12 — cap check + hold insert must be serialised per tenant)
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 3a. Read current MTD spend INSIDE the transaction with FOR UPDATE row lock
    let current = sqlx::query_as::<_, CostLedgerRow>(
        "INSERT INTO cost_ledger (tenant_id, period, monthly_cap_usd) \
         VALUES ($1, date_trunc('month', NOW())::date, $2) \
         ON CONFLICT (tenant_id, period) DO UPDATE SET tenant_id = EXCLUDED.tenant_id \
         RETURNING *"
    ).bind(&req.tenant_id).bind(policy.ai_policy.monthly_cap_usd).fetch_one(&mut *tx).await?;

    // 3b. Re-lock the row explicitly (the UPSERT above already locked it via ON CONFLICT,
    //     but make the intent explicit for future readers)
    sqlx::query("SELECT 1 FROM cost_ledger WHERE tenant_id = $1 AND period = date_trunc('month', NOW())::date FOR UPDATE")
        .bind(&req.tenant_id).execute(&mut *tx).await?;

    // 4. Cap check (boundary inclusive — `spent + estimated == cap` is permitted)
    if current.spent_usd + estimated_usd > policy.ai_policy.monthly_cap_usd {
        tx.rollback().await?;   // release the row lock; no hold is created
        return Ok(PrecheckOutcome::Refuse {
            reason: RefuseReason::BudgetCapExceeded,
            current_spent_usd: current.spent_usd,
            cap_usd: policy.ai_policy.monthly_cap_usd,
        });
    }

    // 5. Insert hold INSIDE the same transaction (idempotent via UNIQUE on (tenant_id, idempotency_key))
    let hold_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO cost_ledger_hold (tenant_id, idempotency_key, estimated_usd, resolved_provider, resolved_model, expires_at, state) \
         VALUES ($1, $2, $3, $4, $5, NOW() + INTERVAL '60 seconds', 'held') \
         ON CONFLICT (tenant_id, idempotency_key) DO UPDATE SET state = cost_ledger_hold.state \
         RETURNING id"
    ).bind(&req.tenant_id).bind(&req.idempotency_key).bind(estimated_usd)
     .bind(&provider).bind(&model).fetch_one(&mut *tx).await?;

    // 6. Emit BRAIN audit row BEFORE commit (audit-before-action invariant — if BRAIN fails,
    //    we rollback the hold). The Writer subprocess holds <memory-root>/.lock for its own
    //    serialization; the Postgres tx holds the cost_ledger row lock. Two orthogonal locks.
    brain_writer::emit(
        canonical::precheck(
            &req.tenant_id, &req.agent_persona, &req.model_alias,
            &provider, &model, estimated_usd, current.spent_usd,
            &req.idempotency_key,
        ),
    ).await.map_err(|e| {
        // BRAIN failure rolls back the hold; precheck refuses the call
        // (tx will rollback when dropped at function end since commit didn't happen)
        PrecheckError::BrainWriterFailed { stderr: e.to_string() }
    })?;

    // 7. Commit — the hold + ledger row lock are now durable
    tx.commit().await?;

    // 8. (post-commit) Emit OBS metric — non-critical, fire-and-forget
    metrics::PRECHECK_CALLS.with_label_values(&["allow"]).inc();
    metrics::HOLDS_CREATED.inc();

    Ok(PrecheckOutcome::Allow { hold_id, estimated_usd, ttl_seconds: HOLD_TTL_SECONDS })
}

fn validate_idempotency_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > IDEMPOTENCY_KEY_MAX_LEN {
        return Err("invalid_idempotency_key: length must be 1..=64".into());
    }
    if !key.chars().all(|c| c.is_ascii_graphic()) {
        return Err("invalid_idempotency_key: charset must be ASCII printable".into());
    }
    Ok(())
}
```

*Scaffold above is suggestive, not normative. The acceptance criteria (§4) are the contract.*

---

## §7 — Dependencies

**Code dependencies (must exist before this FR can build):**
- BRAIN module — shipped. The `cyberos.core.writer.Writer` subprocess CLI MUST be available at `cyberos memory put`. Used in step §6 of the skeleton.
- `cyberos-ai-gateway` crate skeleton — created by FR-AI-000 (if needed) or as part of this FR's first commit. Cargo workspace member.

**Concept dependencies (must be agreed before this FR can be audited):**
- Model-alias resolution rules (FR-AI-005 will formalise; for slice 1 use a hardcoded enum).
- Cost-table source (FR-AI-007; for slice 1 hardcode 3 providers' rates as a const HashMap).

**Operational dependencies:**
- A Postgres instance reachable at `DATABASE_URL`. For the integration test, the CI uses a Postgres 16 container via `testcontainers-rs`.

---

## §8 — Example payloads

### Request (HTTP body)

```http
POST /v1/chat/completions HTTP/1.1
Host: ai.cyberos.com
X-Tenant: org:cyberskill                     # slice-1 only; replaced by JWT at P0 · slice 2
X-CyberOS-Module: cuo                         # which module is the cost centre
X-Idempotency-Key: 01HZK9R7A2B4C8D6E1F3G5H8
Content-Type: application/json

{
  "model": "chat.smart",
  "messages": [
    {"role": "system", "content": "You are Genie. Be concise."},
    {"role": "user", "content": "summarise Q1 OKRs"}
  ],
  "agent_persona": "cuo-cpo@0.4.1",
  "max_tokens": 1024,
  "stream": false
}
```

### Response — Allow path (HTTP 200, after provider call succeeds — FR-AI-002 handles the actual provider call + post-reconcile)

```json
{
  "id": "ai_01HZK9R7A2B4C8D6",
  "model": "bedrock:anthropic.claude-3.5-sonnet",
  "choices": [{"message":{"role":"assistant","content":"Q1 OKRs are…"}}],
  "usage": {
    "prompt_tokens": 120,
    "completion_tokens": 450,
    "usd_cost": 0.0078,
    "cache_state": "miss",
    "hold_id": "01HZK9R8M3X5C8Q4",
    "failover_path": "primary"
  }
}
```

### Response — Refuse path (HTTP 402)

```json
{
  "error": "budget_cap_exceeded",
  "current_spent_usd": 148.20,
  "cap_usd": 150.00,
  "estimated_usd": 5.50,
  "suggest": "downshift to chat.fast or wait until 2026-06-01"
}
```

### BRAIN audit row written

```json
{
  "seq": 18421,
  "ts_ns": 1763112131000000000,
  "op": "put",
  "path": "memories/decisions/ai-invocations/1763112131_org-cyberskill_01HZK9R7A2B4C8D6.md",
  "extra": {
    "kind": "ai.precheck",
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "model_alias": "chat.smart",
    "resolved_provider": "bedrock",
    "resolved_model": "anthropic.claude-3.5-sonnet",
    "estimated_usd": 0.0085,
    "current_spent_usd": 47.23
  },
  "prev_chain": "...",
  "chain": "..."
}
```

---

## §9 — Open questions

All resolved 2026-05-15. Promoted to §1 normative clauses:

1. **~~Cost-table source~~** → §1 #8 (YAML config at `services/ai-gateway/config/cost_rates.yaml`).
2. **~~Missing tenant policy~~** → §1 #9 (`503 POLICY_NOT_FOUND`; no silent default).
3. **~~Idempotency-Key format~~** → §1 #10 (free-string 1..=64 ASCII printable).
4. **~~Audit-row sync vs async~~** → §1 #11 (synchronous in slice 1; FR-AI-008 spikes PyO3).

---

## §10 — Failure modes inventory (all error paths)

| Failure | Detection | HTTP | Recovery |
|---|---|---|---|
| Tenant policy file missing | `policy::load_for_tenant` returns `PolicyMissing` | `503 POLICY_NOT_FOUND` | Operator adds the YAML; gateway re-reads on file-watch event (FR-AI-005 §1 #4) |
| Tenant policy invalid YAML | Caught at gateway startup by `init_loader`; gateway exits 1 | Gateway doesn't serve | Operator fixes YAML; redeploy. Loud failure by design (no silent defaults) |
| Budget cap exceeded | `current.spent_usd + estimated_usd > monthly_cap_usd` | `402 PAYMENT_REQUIRED` | Operator raises cap or tenant waits to next period; FR-AI-021 CLI exposes `cyberos-ai policy raise-cap` |
| Cost-table missing model | `cost_table::lookup` returns None | `503 SERVICE_UNAVAILABLE` (`no_cost_table_for_model`) | Operator adds model to `cost_rates.yaml`; gateway hot-reloads |
| Persona not allowed | `policy.allowed_personas` set + `req.agent_persona` not in list | `403 FORBIDDEN` (`persona_not_allowed`) | Operator updates policy or caller fixes agent_persona |
| Invalid idempotency key | Length/charset check fails | `400 BAD_REQUEST` (`invalid_idempotency_key`) | Caller fixes; retry with valid key |
| Tenant suspended (slice-2 future) | Policy carries `suspended: true` | `403 FORBIDDEN` (`tenant_suspended`) | Out of scope for slice 1; placeholder reserved |
| BRAIN Writer subprocess fails | `brain_writer::emit` returns Err | `500 INTERNAL_SERVER_ERROR` (opaque request_id) | Operator investigates via OBS; precheck refuses the call (audit-before-action invariant) |
| Postgres pool exhausted / unreachable | `sqlx::Error` from any query | `500 INTERNAL_SERVER_ERROR` (opaque request_id) | Operator investigates; failure surfaces in OBS |
| Concurrent cap-race (two prechecks for same tenant) | Detected by `FOR UPDATE` row lock | One Allow, one Refuse | Postgres MVCC handles correctly; verified by AC #1 of the property-test |
| Hold UNIQUE constraint hit on idempotency_key | Retry with same `idempotency_key` from caller | Returns existing hold | Idempotent by design (§1 #4 of FR-AI-001 — return the existing hold_id) |

---

## §11 — Notes (informational, no normative force)

- Slice 1 of AI Gateway is 5 FRs (FR-AI-001..005). This one is the gate; FR-AI-002 is the post-reconcile; FR-AI-003 is the BRAIN audit-bridge; FR-AI-004 is the hold-expiry cleanup job; FR-AI-005 is the tenant-policy loader.
- After this FR ships, every other module (CUO, KB, CHAT, …) inherits the cost gate automatically by calling `ai_gateway::chat_complete()` instead of a raw provider SDK. The "no SDK in any other module" rule (AI Gateway page §0) is enforced architecturally, not by lint.
- The 50ms p95 latency budget is tight but achievable: Postgres read (~5ms) + Postgres write (~10ms) + BRAIN Writer subprocess (~30ms). The subprocess is the bottleneck — see §9 Q4.
- This FR's verification test fixture should land in `services/ai-gateway/tests/fixtures/cost_ledger/` and become a reusable property-test seed for FR-AI-002 + FR-AI-004.

---

*End of FR-AI-001. Run `fr-audit` next: `cargo run -p cyberos-skill-cli -- run fr-audit --input '{"fr_path": "docs/feature-requests/ai/FR-AI-001-cost-ledger-precheck.md"}'`*
