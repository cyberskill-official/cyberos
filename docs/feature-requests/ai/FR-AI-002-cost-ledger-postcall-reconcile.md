---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-002
title: "AI Gateway cost-ledger post-call reconcile"
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
related_frs: [FR-AI-001, FR-AI-003, FR-AI-004]
depends_on: [FR-AI-001, FR-AI-007, FR-AI-003]
blocks: [FR-AI-008, FR-AI-010, FR-AI-021]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cost-gate
  - website/docs/modules/ai.html#bigger-picture
source_decisions:
  - archive/2026-05-14/AUDIT_AND_PLAN.md §3.3 (P0 · slice 1 build placement)
  - docs/feature-requests/ai/FR-AI-001-cost-ledger-precheck.md §2 (two-phase design rationale)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/cost_reconcile.rs
  - services/ai-gateway/tests/cost_reconcile_test.rs
modified_files:
  - services/ai-gateway/src/handlers/chat.rs
  - services/ai-gateway/src/cost_ledger.rs   # add reconcile() alongside precheck()
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - bash: cargo test -p cyberos-ai-gateway cost_reconcile
  - brain: write memories/decisions/ai-invocations/* via canonical Writer (NOT directly)
disallowed_tools:
  - in-place edit of cost_ledger_hold schema (FR-AI-001 owns it)
  - touch services/auth/**
  - bypass the BRAIN writer when finalising a hold (audit-before-action invariant)

# ───── Estimated work ─────
effort_hours: 6
sub_tasks:
  - "0.5h: reconcile() function signature + state machine (held → reconciled | refunded | expired)"
  - "1.0h: actual-cost computation from provider response usage fields"
  - "1.0h: atomic UPSERT into cost_ledger.spent_usd (transactional with hold state transition)"
  - "1.0h: error-path refund (provider 5xx → hold released, no spend recorded)"
  - "1.0h: BRAIN ai.invocation audit row with actual_usd, latency_ms, status"
  - "1.5h: integration test (5 cases: happy / refund / partial / over-estimate / under-estimate)"
risk_if_skipped: "Holds accumulate without ever being settled. Tenant spend tracking diverges from reality — the precheck cap becomes meaningless because spent_usd never advances. Cost gate degrades to a placebo. Provider error paths leak budget."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** expose a `cost_ledger::reconcile(hold_id, outcome)` function that runs synchronously after every LLM-provider call completes (success or failure). Given the `hold_id` returned by `precheck()` (FR-AI-001) and a `CallOutcome` describing what actually happened, the function:

1. **MUST** load the `cost_ledger_hold` row by `hold_id` inside a Postgres transaction.
2. **MUST** refuse to reconcile a hold whose `state != 'held'` (returns `Err(ReconcileError::AlreadyFinalised)`); reconciliation MUST be idempotent — callers retrying a successful reconcile MUST observe the same outcome.
3. **MUST** on `CallOutcome::Success { usage }` — compute `actual_usd` from `usage.prompt_tokens × input_rate + usage.completion_tokens × output_rate` at the resolved provider/model from the hold row; UPDATE the `cost_ledger.spent_usd` (`+= actual_usd`); transition hold `state → 'reconciled'`; emit one `ai.invocation` BRAIN audit row.
4. **MUST** on `CallOutcome::ProviderError { http_status, retryable }` — leave `cost_ledger.spent_usd` UNCHANGED; transition hold `state → 'refunded'`; emit one `ai.invocation_failed` BRAIN audit row carrying `http_status` and `retryable`. No budget is consumed for provider failures the caller did not get value from.
5. **MUST** on `CallOutcome::Cancelled { reason }` (client disconnect mid-stream) — partial refund: compute `actual_usd` from the tokens that DID stream back; update `spent_usd` only for those tokens; transition hold `state → 'reconciled'` (NOT `refunded`); emit `ai.invocation` row with `extra.cancelled: true`.
6. **MUST** complete reconciliation + audit emission inside a single Postgres transaction. Partial state (hold reconciled but spend not updated, or vice versa) MUST NOT be observable to any concurrent reader.
7. **MUST** return synchronously within 80ms p95 — reconciliation is on the hot path of every chat call's response.
8. **SHOULD** emit a `cap_crossed_after_reconcile` event to OBS if `spent_usd` after update crosses the `monthly_cap_usd × warn_threshold` boundary (default 0.80). De-duplication: `cost_ledger.warn_emitted_at` MUST be set atomically with the threshold crossing; subsequent reconciles in the same period that further increase `spent_usd` MUST NOT emit again.
9. **MUST** treat the provider's `finish_reason` as authoritative when constructing `CallOutcome`. If `finish_reason = "stop"`, the call is `Success` regardless of any client-disconnect signal. The `Cancelled` variant is reserved for cases where the provider did not return `finish_reason: stop`.

This FR completes the two-phase cost-gate begun in FR-AI-001. Once it ships, the AI Gateway provides true cost-of-everything accounting — no held budget leaks, no provider-error charges, no double-billing on idempotent retries.

---

## §2 — Why this design (rationale for humans)

**The naive design** would have a single function that calls the provider, then updates the ledger. That works until the provider returns success but the network drops the response; the gateway's view of "we paid" diverges from the provider's view of "we charged". The two-phase pattern (hold + reconcile, with idempotency keys threading both) survives partial failures.

**Why refund on provider-error and not on every non-200**: 429 (rate limit) and 503 (overload) MUST refund — the tenant got no value, charging the budget would penalise them for the gateway's choice of provider. 400 (bad request from the tenant) is interesting: the tenant's prompt was malformed, but the provider may still have done some prompt-validation work and charged us. Slice-1 default: **refund 400 errors too**. The tenant retries with a fixed prompt; that's the right product behaviour. FR-AI-009 (circuit breaker) and FR-AI-021 (operator CLI) get the override knob for the edge cases.

**Why partial refund on cancel and not full refund**: streaming responses already consumed provider tokens by the time the client disconnects. The provider has already charged for them. Refunding the tenant's budget when the provider has already invoiced us makes the gateway eat the loss — fine occasionally, but cancels happen all the time (client tab close, mobile network drop). We charge for what streamed; the tenant got the value of those tokens (their UI rendered them).

**Why "transactional with hold state transition"**: split-brain failures are inevitable in production. If the audit row lands but the ledger update fails, a future precheck sees stale `spent_usd` and may allow a call that pushes us over budget. The cost of the strict transaction is one extra row-lock per call (~2ms); the cost of skipping it is silent budget overruns that surface only on the monthly invoice.

**Why this is FR-AI-002 specifically**: the post-reconcile is the *value-realising* half of the cost gate. FR-AI-001 holds, FR-AI-002 settles, FR-AI-003 records the audit row, FR-AI-004 cleans up holds that never settled. Until all four are in place, the cost gate is incomplete; AI Gateway cannot ship to P0 · slice 2.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signature

```rust
// services/ai-gateway/src/cost_reconcile.rs
pub async fn reconcile(
    hold_id: Uuid,
    outcome: CallOutcome,
    pool: &PgPool,
) -> Result<ReconcileOutcome, ReconcileError>;

pub enum CallOutcome {
    Success {
        usage: ProviderUsage,
        latency_ms: u32,
        cache_state: CacheState,
        provider_request_id: String,
    },
    ProviderError {
        http_status: u16,
        retryable: bool,
        provider_error_message: String,
    },
    Cancelled {
        partial_usage: Option<ProviderUsage>,
        reason: CancelReason,
    },
}

pub struct ProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub enum ReconcileOutcome {
    Reconciled { actual_usd: Decimal, new_spent_total_usd: Decimal, warn_crossed: bool },
    Refunded   { hold_estimated_usd: Decimal, reason: RefundReason },
}

pub enum RefundReason {
    ProviderError { http_status: u16 },
    ProviderUnreachable,
}

pub enum ReconcileError {
    HoldNotFound,
    AlreadyFinalised { current_state: String, original_outcome: ReconcileOutcome },
    DbError(sqlx::Error),
    BrainWriterFailed { stderr: String },
    CostTableMissing { provider: String, model: String },
}
```

### State machine — `cost_ledger_hold.state`

```
held ──reconcile(Success)──► reconciled
held ──reconcile(ProviderError)──► refunded
held ──reconcile(Cancelled{partial})──► reconciled  (with extra.cancelled=true)
held ──reconcile(Cancelled{none})──► refunded
held ──expiry job (FR-AI-004)──► expired
reconciled ──reconcile()──► AlreadyFinalised (no-op, returns same outcome)
refunded   ──reconcile()──► AlreadyFinalised (no-op, returns same outcome)
expired    ──reconcile()──► AlreadyFinalised (logs a warn; FR-AI-001 hold never settled)
```

### SQL — reconcile transaction shape

```sql
BEGIN;
  -- 1. Lock the hold row
  SELECT state, tenant_id, estimated_usd, resolved_provider, resolved_model
    FROM cost_ledger_hold
    WHERE id = $hold_id
    FOR UPDATE;

  -- 2a. (Success path) update ledger + transition hold
  UPDATE cost_ledger
    SET spent_usd = spent_usd + $actual_usd
    WHERE tenant_id = $tenant_id
      AND period = date_trunc('month', NOW())::date;

  UPDATE cost_ledger_hold
    SET state = 'reconciled', actual_usd = $actual_usd, reconciled_at = NOW()
    WHERE id = $hold_id;

  -- 2b. (Refund path) just transition hold; ledger unchanged
  UPDATE cost_ledger_hold
    SET state = 'refunded', refunded_at = NOW(), refund_reason = $reason
    WHERE id = $hold_id;
COMMIT;
```

### Schema migration (additive to FR-AI-001's `0001_cost_ledger.sql`)

```sql
-- services/ai-gateway/migrations/0002_cost_ledger_reconcile.sql
ALTER TABLE cost_ledger_hold
  ADD COLUMN actual_usd      NUMERIC(12,4) NULL,
  ADD COLUMN reconciled_at   TIMESTAMPTZ   NULL,
  ADD COLUMN refunded_at     TIMESTAMPTZ   NULL,
  ADD COLUMN refund_reason   TEXT          NULL,
  ADD COLUMN provider_request_id TEXT      NULL;

ALTER TABLE cost_ledger_hold
  DROP CONSTRAINT IF EXISTS cost_ledger_hold_state_check;

ALTER TABLE cost_ledger_hold
  ADD CONSTRAINT cost_ledger_hold_state_check
    CHECK (state IN ('held','reconciled','refunded','expired'));

-- For warn-threshold de-duplication (AC #7)
ALTER TABLE cost_ledger
  ADD COLUMN warn_emitted_at TIMESTAMPTZ NULL;

-- Index for the OBS dashboard's "AI invocations in last 24h" query
CREATE INDEX cost_ledger_hold_reconciled_at_idx
  ON cost_ledger_hold (reconciled_at DESC)
  WHERE state = 'reconciled';
```

### HTTP integration in the chat handler

```rust
// services/ai-gateway/src/handlers/chat.rs (sketch)
let precheck = cost_ledger::precheck(&req, &pool, &policy).await?;
let hold_id = match precheck {
    PrecheckOutcome::Allow { hold_id, .. } => hold_id,
    PrecheckOutcome::Refuse { reason, current_spent_usd, cap_usd } =>
        return Err(refuse_to_http(reason, current_spent_usd, cap_usd)),
};

let provider_call = router::call_provider(&req, &policy).await;

let outcome = match provider_call {
    Ok(resp) => CallOutcome::Success {
        usage: resp.usage,
        latency_ms: resp.latency_ms,
        cache_state: resp.cache_state,
        provider_request_id: resp.id,
    },
    Err(ProviderError { http_status, retryable, message }) => CallOutcome::ProviderError {
        http_status, retryable, provider_error_message: message,
    },
};

let reconcile_outcome = cost_ledger::reconcile(hold_id, outcome, &pool).await?;

// Build HTTP response from provider_call + reconcile_outcome
build_response(provider_call, reconcile_outcome)
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy path (success)** — Given a hold with `estimated_usd = 0.0085` for tenant `org:test-a` with `spent_usd = 12.50`, when `reconcile(hold_id, Success { usage: {prompt: 120, completion: 450} })` is called, it MUST return `Reconciled { actual_usd: 0.0078 }` (computed from the actual token counts × the provider's rate); MUST update `cost_ledger.spent_usd` to `12.5078`; MUST transition the hold to `state='reconciled'`; MUST emit exactly one `ai.invocation` BRAIN row.
2. **Idempotent retry (success)** — Calling `reconcile()` twice on the same `hold_id` with the same `Success` outcome MUST return `Err(AlreadyFinalised { current_state: "reconciled" })` on the second call; MUST NOT double-count the spend; MUST NOT emit a second BRAIN row.
3. **Provider error refund** — Given a hold with `estimated_usd = 0.0085`, when `reconcile(hold_id, ProviderError { http_status: 503, retryable: true })` is called, it MUST return `Refunded { hold_estimated_usd: 0.0085, reason: ProviderError { http_status: 503 } }`; MUST NOT update `cost_ledger.spent_usd`; MUST transition the hold to `state='refunded'`; MUST emit exactly one `ai.invocation_failed` BRAIN row.
4. **Cancelled with partial stream** — `reconcile(hold_id, Cancelled { partial_usage: Some({prompt: 120, completion: 200}), reason: ClientDisconnect })` MUST compute actual_usd from the 200 streamed completion tokens; MUST update spent_usd by that amount; MUST transition to `'reconciled'`; MUST emit `ai.invocation` row with `extra.cancelled = true`.
5. **Cancelled with no stream** — `reconcile(hold_id, Cancelled { partial_usage: None, reason: TimeoutBeforeFirstToken })` MUST treat as a refund (Refunded); MUST NOT update spent_usd.
6. **Atomicity under crash** — If the process is killed between `UPDATE cost_ledger` and `UPDATE cost_ledger_hold`, the subsequent transaction MUST observe BOTH updates applied OR NEITHER (Postgres MVCC guarantees this for `BEGIN…COMMIT`). Property test: 100 reconcile calls × random crash points × verify ledger consistency.
7. **Warn-threshold crossing emits event** — When reconcile brings `spent_usd` from `0.79 × cap` to `0.81 × cap`, the function MUST set `warn_crossed: true` in the `Reconciled` outcome AND emit an OBS event `cap_crossed_after_reconcile`. Crossing from `0.81 → 0.85` MUST NOT emit a second event (once per period per tenant).
8. **Cost-table missing** — If the hold's `resolved_provider`/`resolved_model` is no longer in the cost-table at reconcile time (rare — only happens if cost-table reloaded between precheck and provider call), `reconcile()` MUST return `Err(CostTableMissing { provider, model })`; MUST NOT transition the hold; the operator CLI (FR-AI-021) provides the recovery path.
9. **Latency budget** — On a warm Postgres pool, `reconcile()` MUST complete within 80ms p95 over a 1000-call integration test. The 30ms budget over precheck's 50ms accounts for the additional Writer subprocess emission (audit row carries more data than the precheck row).
10. **BRAIN row schema** — The `ai.invocation` row MUST carry: `tenant_id`, `agent_persona`, `model_alias`, `resolved_provider`, `resolved_model`, `prompt_tokens`, `completion_tokens`, `actual_usd`, `latency_ms`, `cache_state`, `provider_request_id`, `hold_id`. The `ai.invocation_failed` row MUST carry: same fields minus token/cost fields, plus `http_status`, `retryable`, `provider_error_message`.
11. **HTTP 400 from provider triggers refund** — `reconcile(hold_id, ProviderError { http_status: 400, retryable: false })` MUST return `Refunded { reason: ProviderError { http_status: 400 } }`; MUST NOT update `cost_ledger.spent_usd`; MUST transition the hold to `state='refunded'`. The tenant's prompt was malformed; charging budget for the provider's prompt-validation work would penalise the tenant for our choice of provider.
12. **Partial-cancel cost floor** — When `Cancelled { partial_usage: Some(usage) }` computes `actual_usd < 0.0001` (the `NUMERIC(12,4)` column precision), the reconcile MUST still apply the spend (rounded up to `0.0001`). Rationale: losing the row entirely would diverge total spent from provider invoice; floor at column precision preserves audit completeness.
13. **AlreadyFinalised carries the persisted outcome** — When `reconcile()` is called on a hold whose `state ∈ {reconciled, refunded, expired}`, it MUST return `Err(ReconcileError::AlreadyFinalised { current_state, original_outcome })` where `original_outcome` is reconstructed from the row (i.e., `Reconciled { actual_usd, new_spent_total_usd, warn_crossed }` or `Refunded { hold_estimated_usd, reason }`). Callers re-attempting after timeout MUST receive the same authoritative outcome rather than guessing.
14. **SHOULD** emit OTel metrics: `ai_gateway_reconcile_calls_total{outcome}` (counter; outcome ∈ reconciled/refunded/already_finalised/error), `ai_gateway_reconcile_latency_ms` (histogram), `ai_gateway_holds_reconciled_total` (counter), `ai_gateway_holds_refunded_total{reason}` (counter), `ai_gateway_spend_usd_total{tenant_id,period}` (gauge).
15. **MUST** emit `ai.reconcile_started` BEFORE applying the `UPDATE cost_ledger SET state='finalised', actual_cost_minor=..., reconciled_at=NOW()` per AUTHORING.md §3.8 rule 25 (audit-before-action). The Postgres transaction MUST wrap BOTH the BRAIN emit AND the UPDATE in a single transaction; either commits together or both roll back. The `ai.reconcile_completed` (or `ai.reconcile_failed`) row is emitted post-commit per AUTHORING.md §3.8 rule 26 (pair-write). AC #15 asserts the captured-events order via a `tracing-test` capture: `[reconcile_started, sql_update, COMMIT, reconcile_completed]`.

---

## §5 — Verification method

**Integration test:** `services/ai-gateway/tests/cost_reconcile_test.rs`

```rust
#[tokio::test]
async fn reconcile_success_updates_ledger_and_audit() {
    let env = TestEnv::new().await;
    env.seed_tenant("org:test-a", monthly_cap = 100, spent_usd = 12.50);
    let hold_id = env.seed_hold("org:test-a", estimated_usd = 0.0085).await;

    let outcome = cost_ledger::reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage { prompt_tokens: 120, completion_tokens: 450 },
            latency_ms: 850,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_abc123".to_string(),
        },
        &env.pool,
    ).await.unwrap();

    assert!(matches!(outcome, ReconcileOutcome::Reconciled { .. }));
    let row = env.read_ledger("org:test-a").await;
    assert_eq!(row.spent_usd, dec!(12.5078));
    let hold = env.read_hold(hold_id).await;
    assert_eq!(hold.state, "reconciled");
    assert!(hold.actual_usd.is_some());
    assert!(env.brain_has_row("ai.invocation", &hold_id.to_string()).await);
}

#[tokio::test]
async fn reconcile_provider_error_refunds() { /* AC #3 */ }

#[tokio::test]
async fn reconcile_idempotent_double_call() { /* AC #2 */ }

#[tokio::test]
async fn reconcile_cancelled_partial_charges_partial() { /* AC #4 */ }

#[tokio::test]
async fn reconcile_atomicity_under_simulated_crash() {
    // Property test: 100 iterations; abort transaction at random points; verify invariant
    proptest!(|(abort_point in 0..6_u32)| {
        let env = TestEnv::new_with_abort(abort_point);
        let _ = futures::executor::block_on(reconcile_with_abort(&env));
        let row = futures::executor::block_on(env.read_ledger("org:test-a"));
        let hold = futures::executor::block_on(env.read_hold(hold_id));
        // Invariant: ledger spend reflects hold state exactly
        if hold.state == "reconciled" {
            prop_assert!(row.spent_usd >= dec!(12.50) + hold.actual_usd.unwrap());
        }
        if hold.state == "refunded" || hold.state == "held" {
            prop_assert_eq!(row.spent_usd, dec!(12.50));
        }
    });
}
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway cost_reconcile
```

CI gate: same package as FR-AI-001's test; both must pass on every PR touching `services/ai-gateway/**`.

---

## §6 — Implementation skeleton (suggested scaffold)

```rust
// services/ai-gateway/src/cost_reconcile.rs

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub async fn reconcile(
    hold_id: Uuid,
    outcome: CallOutcome,
    pool: &PgPool,
) -> Result<ReconcileOutcome, ReconcileError> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 1. Load + lock the hold
    let hold = sqlx::query_as::<_, HoldRow>(
        "SELECT id, tenant_id, idempotency_key, estimated_usd, resolved_provider, resolved_model, state \
         FROM cost_ledger_hold WHERE id = $1 FOR UPDATE"
    )
    .bind(hold_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ReconcileError::HoldNotFound)?;

    // 2. Idempotency check — reconstruct the original outcome from persisted state (§1 #13)
    if hold.state != "held" {
        let original_outcome = reconstruct_outcome_from_row(&hold);
        return Err(ReconcileError::AlreadyFinalised {
            current_state: hold.state.clone(),
            original_outcome,
        });
    }

    // 3. Branch by outcome
    let result = match outcome {
        CallOutcome::Success { usage, latency_ms, cache_state, provider_request_id } => {
            let actual_usd = compute_actual_cost(&hold, &usage)?;
            apply_success(&mut tx, &hold, actual_usd, latency_ms, &cache_state, &provider_request_id).await?
        }
        CallOutcome::ProviderError { http_status, retryable, provider_error_message } => {
            apply_refund(&mut tx, &hold, RefundReason::ProviderError { http_status }, retryable, &provider_error_message).await?
        }
        CallOutcome::Cancelled { partial_usage: Some(usage), reason } => {
            let actual_usd = compute_actual_cost(&hold, &usage)?;
            apply_partial(&mut tx, &hold, actual_usd, reason).await?
        }
        CallOutcome::Cancelled { partial_usage: None, reason } => {
            apply_refund(&mut tx, &hold, RefundReason::ProviderUnreachable, false, &format!("{:?}", reason)).await?
        }
    };

    // 4. Emit BRAIN audit row INSIDE the transaction's lifetime (audit-before-action)
    //    Implementation: brain_writer::emit() is FR-AI-003. We call it before commit so
    //    a failed audit aborts the transaction.
    let row_kind = match &result {
        ReconcileOutcome::Reconciled { .. } => "ai.invocation",
        ReconcileOutcome::Refunded   { .. } => "ai.invocation_failed",
    };
    brain_writer::emit_for_hold(&hold, &result, row_kind).await
        .map_err(|e| ReconcileError::BrainWriterFailed { stderr: e.to_string() })?;

    // 5. Commit
    tx.commit().await?;
    Ok(result)
}

fn compute_actual_cost(hold: &HoldRow, usage: &ProviderUsage) -> Result<Decimal, ReconcileError> {
    let rate = cost_table::lookup(&hold.resolved_provider, &hold.resolved_model)
        .ok_or(ReconcileError::CostTableMissing {
            provider: hold.resolved_provider.clone(),
            model: hold.resolved_model.clone(),
        })?;
    Ok(
        (Decimal::from(usage.prompt_tokens) / dec!(1000)) * rate.input
      + (Decimal::from(usage.completion_tokens) / dec!(1000)) * rate.output
    )
}
```

*Scaffold above is suggestive, not normative. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies (must exist before this FR can build):**
- FR-AI-001 — shipped or building. This FR consumes `cost_ledger_hold` rows in `state='held'`.
- FR-AI-003 — at least the `brain_writer::emit_for_hold()` function signature stub. The actual chained-audit-row emission is FR-AI-003's job; this FR depends on its signature.
- Cost-table — for slice 1, the same hardcoded `const HashMap` used by FR-AI-001 §6.

**Concept dependencies:**
- Idempotency-key semantics inherited from FR-AI-001 (`UNIQUE (tenant_id, idempotency_key)` on `cost_ledger_hold`).
- Cost-table source — see FR-AI-001 §9 Q1 (resolution applies equally to this FR).

**Operational dependencies:**
- Postgres at `DATABASE_URL` (same as FR-AI-001).
- The integration test reuses FR-AI-001's `TestEnv` helper from `services/ai-gateway/tests/common/mod.rs`.

---

## §8 — Example payloads

### Reconcile call site (after a successful provider call)

```rust
let outcome = router::call_anthropic(&req).await;
// outcome.usage = { prompt_tokens: 120, completion_tokens: 450 }
// outcome.latency_ms = 850
// outcome.cache_state = Miss
// outcome.id = "prv_01HZK9R8M3X5C8Q4"

cost_ledger::reconcile(
    hold_id,
    CallOutcome::Success {
        usage: outcome.usage,
        latency_ms: outcome.latency_ms,
        cache_state: outcome.cache_state,
        provider_request_id: outcome.id,
    },
    &pool,
).await?;
```

### BRAIN `ai.invocation` row written on success

```json
{
  "seq": 18422,
  "ts_ns": 1763112131840000000,
  "op": "put",
  "path": "memories/decisions/ai-invocations/1763112131840_org-cyberskill_01HZK9R7A2B4C8D6.md",
  "extra": {
    "kind": "ai.invocation",
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "model_alias": "chat.smart",
    "resolved_provider": "bedrock",
    "resolved_model": "anthropic.claude-3.5-sonnet",
    "prompt_tokens": 120,
    "completion_tokens": 450,
    "actual_usd": 0.0078,
    "hold_id": "01HZK9R8M3X5C8Q4",
    "latency_ms": 850,
    "cache_state": "miss",
    "provider_request_id": "msg_01ABCxyz...",
    "new_spent_total_usd": 47.2378,
    "warn_crossed": false
  },
  "prev_chain": "...",
  "chain": "..."
}
```

### BRAIN `ai.invocation_failed` row written on provider error

```json
{
  "seq": 18423,
  "ts_ns": 1763112135000000000,
  "op": "put",
  "path": "memories/decisions/ai-invocations/1763112135000_org-cyberskill_01HZK9R7A2B4C8D7.md",
  "extra": {
    "kind": "ai.invocation_failed",
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "resolved_provider": "openai",
    "resolved_model": "gpt-4o",
    "http_status": 503,
    "retryable": true,
    "provider_error_message": "service temporarily unavailable",
    "hold_id": "01HZK9R7A2B4C8D7",
    "refund_amount_usd": 0.012
  },
  "prev_chain": "...",
  "chain": "..."
}
```

### HTTP 200 response shape (consumer sees this on success path)

```json
{
  "id": "ai_01HZK9R7A2B4C8D6",
  "model": "bedrock:anthropic.claude-3.5-sonnet",
  "choices": [{"message": {"role": "assistant", "content": "Q1 OKRs are…"}}],
  "usage": {
    "prompt_tokens": 120,
    "completion_tokens": 450,
    "usd_cost": 0.0078,
    "cache_state": "miss",
    "hold_id": "01HZK9R8M3X5C8Q4",
    "failover_path": "primary",
    "warn_crossed": false
  }
}
```

---

## §9 — Open questions

All resolved 2026-05-15 (round 2). Promoted to §1/§4 normative clauses:

1. **~~400 bad-request refund~~** → §4 AC #11. Was Q1.
2. **~~Cancelled partial-cost floor~~** → §1 #12 (floor at `0.0001` column precision). Was Q2.
3. **~~Warn-threshold de-duplication~~** → §3 schema (`warn_emitted_at` column) + §4 AC #7. Was Q3.
4. **~~Streaming-cancel race~~** → §1 #9 (provider `finish_reason` authoritative). Was Q4.
5. **~~AlreadyFinalised carries original outcome~~** → §1 #13 + §3 enum revision. Was Q5.

---

## §10 — Failure modes inventory

| Failure | Detection | HTTP / Return | Recovery |
|---|---|---|---|
| Hold doesn't exist | `cost_ledger_hold` row missing for `hold_id` | `Err(HoldNotFound)` → `500 INTERNAL_SERVER_ERROR` | Investigate via OBS; likely a precheck-reconcile mismatch (caller passing wrong hold_id) |
| Hold already finalised | `state != 'held'` in row lock | `Err(AlreadyFinalised { current_state, original_outcome })` | Caller treats `original_outcome` as authoritative; safe to retry |
| Cost-table missing model | `cost_table::lookup` returns None at reconcile (cost-rates file was hot-reloaded between precheck and provider call) | `Err(CostTableMissing { provider, model })` | Operator restores cost-table entry; FR-AI-021 `cyberos-ai cost-table reload` fixes |
| Postgres transaction conflict | sqlx `SerializationError` on commit | `Err(DbError(_))` → retry once with backoff; if still fails, `500` | Internal retry handles transient; persistent failure surfaces as `500` |
| BRAIN Writer fails | `brain_writer::emit` returns Err | `Err(BrainWriterFailed { stderr })` → transaction rolls back | Audit-before-commit invariant preserved; caller may retry reconcile |
| Provider error 5xx (refund path) | `CallOutcome::ProviderError { retryable: true }` | `Refunded { reason: ProviderError { http_status } }` | Caller may retry the entire call (new idempotency_key); tenant unaffected |
| Provider error 400 (refund path) | `CallOutcome::ProviderError { http_status: 400 }` | `Refunded { reason: ProviderError { http_status: 400 } }` | Caller fixes prompt; tenant unaffected (§4 AC #11) |
| Cancellation with no partial stream | `Cancelled { partial_usage: None }` | `Refunded { reason: ProviderUnreachable }` | Treated as refund; no spend |
| Cancellation with partial stream | `Cancelled { partial_usage: Some(usage) }` | `Reconciled { actual_usd, ... }` with `extra.cancelled = true` | Partial spend applied; floor at column precision (§1 #12) |
| Warn-threshold de-dup race | Two concurrent reconciles both attempt to emit warn | DB-level `UPDATE ... WHERE warn_emitted_at IS NULL` guarantees one wins | OBS sees exactly one event per (tenant, period) |

---

## §11 — Notes (informational, no normative force)

- The 80ms p95 budget is ~30ms looser than precheck's 50ms because reconcile does more work (compute actual cost, transactional update, larger audit row). Both fit comfortably under the AI Gateway's overall 200ms p99 budget (NFR-AI-001).
- BRAIN row payload sizes (~600 bytes for `ai.invocation`) drive monthly storage estimates: at 100k AI calls/month per tenant × 600 bytes = 60 MB/tenant/month of audit rows. The compaction trigger (`AGENTS.md §7.6`: 5 MB / 5000 rows) will fire monthly per active tenant. This is expected and fine.
- The `provider_request_id` field on the audit row is the primary join key for OBS — when a tenant reports "my call took forever", the OBS investigator looks up the `ai.invocation` row by tenant + time window, pulls the `provider_request_id`, then queries the provider's status API for the full trace.
- After this FR ships, AI Gateway can serve the first dogfood call from CUO. CUO Phase 2 LLM (P1) cannot start until this two-phase gate is live.
- The `expired` state branch is owned by FR-AI-004 (cleanup job), not this FR. This FR only ensures `expired` rows are recognised as `AlreadyFinalised` on a late reconcile attempt.
- **Money-as-BIGINT-minor (AUTHORING.md §3.4 rule 11) — boundary confirmation:** `cost_ledger.estimated_cost_minor` and `actual_cost_minor` are `BIGINT` (i64) in the schema; the `Decimal × tokens` conversion happens at the boundary via `Currency::USD.decimals()` helper which rounds to the minor-unit precision (cents for USD, hundredths for VND). No `FLOAT`/`DOUBLE` anywhere in the storage path or hot-path computation. The rates themselves (FR-AI-007's `rust_decimal::Decimal`) are constants — see FR-AI-007 §11 for the rate-vs-storage boundary explanation. Future maintainers: do not "fix" `actual_cost_minor` to Decimal — currency arithmetic in BIGINT minor is required for SOX-level audit precision per AUTHORING.md §3.4.

---

*End of FR-AI-002. Run `feature-request-audit` next: `cargo run -p cyberos-skill-cli -- run feature-request-audit --input '{"fr_path": "docs/feature-requests/ai/FR-AI-002-cost-ledger-postcall-reconcile.md"}'`*
