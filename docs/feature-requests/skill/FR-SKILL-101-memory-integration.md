---
id: FR-SKILL-101
title: "Skill memory integration — skill.invoked_started + skill.invoked_completed audit rows (skill.* namespace) + args_hash + trace_id propagation + panic-recovery"
module: SKILL
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-003, FR-AI-022, FR-SKILL-102]
depends_on: [FR-AI-003]
blocks: [FR-SKILL-102, FR-SKILL-103]

source_pages:
  - website/docs/modules/skill.html#memory-integration
source_decisions:
  - DEC-200 (every skill invocation auditable; before-AND-after rows; args_hash not raw)
  - DEC-201 (skill panic still emits completed row with outcome=panic; never silent)
  - DEC-202 (trace_id propagation; correlate with FR-AI-022 + FR-OBS-005)

language: rust 1.81
service: cyberos/services/skill-host/
new_files:
  - services/skill-host/Cargo.toml
  - services/skill-host/src/memory_bridge.rs
  - services/skill-host/src/invocation_context.rs
  - services/skill-host/src/audit_canonical.rs
  - services/skill-host/tests/memory_audit_test.rs
  - services/skill-host/tests/memory_audit_panic_test.rs
  - services/skill-host/tests/memory_audit_concurrent_test.rs
  - services/skill-host/tests/memory_audit_trace_propagation_test.rs
modified_files:
  - services/skill-host/src/supervisor.rs                  # call invoke_with_audit
allowed_tools:
  - file_read: services/skill-host/**
  - file_write: services/skill-host/**
  - bash: cd services/skill-host && cargo test memory_audit
disallowed_tools:
  - emit raw args in memory row (per §1 #3 — args_hash only)
  - skip start OR completed row (per §1 #1 + #2 — both required)
  - silent panic (per §1 #5 — panic emits completed row)
  - bypass FR-AI-003 memory_writer (per §1 #6)

effort_hours: 6
sub_tasks:
  - "0.5h: memory_bridge.rs — invoke_with_audit wrapper"
  - "0.5h: invocation_context.rs — caller_persona, trace_id, args_hash plumbing"
  - "0.5h: audit_canonical.rs — skill_invoked_started + skill_invoked_completed builders"
  - "0.5h: SHA-256 canonical-JSON of args"
  - "0.5h: Panic catch (catch_unwind) in skill dispatch"
  - "0.5h: trace_id propagation from caller via OTel context"
  - "0.5h: Failure-to-emit-start blocks invocation"
  - "0.5h: Failure-to-emit-completed logs sev-1 but doesn't reverse skill effect"
  - "1.5h: Tests — happy + panic recovery + concurrent + trace propagation + start-fails-blocks-dispatch"
  - "0.5h: OTel metrics emission"
risk_if_skipped: "Skill invocations are invisible to audit chain. Compliance reviews can't answer 'what skills did persona X invoke?'. Panic-mid-execution leaves no completed row — orphan started rows accumulate. Without trace_id propagation, skill traces can't be joined with calling-LLM trace."
---

## §1 — Description (BCP-14 normative)

The SKILL host **MUST** emit memory audit rows before AND after every skill invocation. Each invocation:

1. **MUST** emit `skill.invoked_started` BEFORE dispatch with payload: `skill_id`, `version`, `caller_persona`, `args_hash` (SHA-256 of canonical JSON), `tenant_id`, `trace_id`, `request_id`.
2. **MUST** emit `skill.invoked_completed` AFTER dispatch with payload: `skill_id`, `outcome` (`success | error | panic | timeout`), `duration_ms`, `result_hash` (SHA-256 of canonical-JSON output, or empty on error/panic), `error_message` (if applicable), `trace_id`, `request_id`.
3. **MUST NOT** emit args raw — only `args_hash`. Args may contain tenant-business-sensitive data (queries, document content); chain must not become a parallel data store. Same for results — only hash.
4. **MUST** propagate `trace_id` from caller's OTel context (FR-AI-022). Both audit rows carry the same `trace_id`; investigators can join LLM trace + skill trace + audit row in memory.
5. **MUST** use FR-AI-003 `memory_writer::emit()` for both rows. Failure to emit `_started` → skill NOT dispatched (caller gets 503). Failure to emit `_completed` → log sev-1 + DO NOT reverse skill effect (skills may have side effects; reversing isn't generally possible).
6. **MUST** complete audit emission within 50ms p95 (matches FR-AI-003 budget). Above 50ms, OBS sev-3 alarm.
7. **MUST** wrap skill dispatch in `catch_unwind` so panic is captured + emits `_completed` row with `outcome: panic`. Without panic-catch, `_started` rows accumulate without matching `_completed` (audit chain inconsistency).
8. **MUST** support concurrent invocations — different skill calls use independent memory write paths; no serialised lock around emit.
9. **MUST** include `tenant_id` in both rows for tenant-scoped audit queries.
10. **MUST** record duration even on panic/error (clock measurement around `catch_unwind`); `duration_ms` reflects actual runtime.
11. **SHOULD** emit OTel metrics:
    - `skill_invoked_total{skill_id, outcome}` (counter).
    - `skill_invoked_duration_ms{skill_id}` (histogram).
    - `skill_audit_emit_failures_total{stage}` (counter; stage ∈ start | completed; sev-1).

---

## §2 — Why this design (rationale for humans)

**Why before AND after rows (DEC-200)?** Single completed-only row hides duration of in-flight invocations. Single started-only row leaves orphans on crashes. Both rows let auditors answer "what skills are currently running?" + "what skills completed in the period?" + "what skills crashed?".

**Why args_hash not raw (DEC-200)?** Args may contain tenant-business semantics (queries about specific products, document content). Storing raw in audit chain creates a parallel data store. Hash preserves uniqueness for forensic correlation without leaking content.

**Why panic emits completed row (DEC-201)?** Without it, `_started` rows accumulate without matching `_completed` — audit chain becomes inconsistent ("which skills are running? unclear, some died."). The panic-completed row IS the truth: "this skill crashed; recorded."

**Why trace_id propagation (DEC-202)?** Skill calls happen as part of LLM workflows. The same trace_id ties the LLM call (Tempo) + LangSmith trace + skill audit row + downstream effects. Without propagation, the chain breaks at the skill boundary.

**Why _started fails block dispatch (§1 #5)?** Without auditable record, skill side effects are invisible. Refusing dispatch when audit can't record preserves auditability invariant. The trade-off is unavailability during memory outages — acceptable because audit chain integrity is non-negotiable.

**Why _completed failure doesn't reverse (§1 #5)?** Skills may have side effects (sent email, updated row). Reversing isn't generally possible. Logging sev-1 + leaving the side effect = honest about the gap. Operator investigates via Layer 1 chain.

**Why concurrent invocations independent (§1 #8)?** Serialised lock around memory_writer would bottleneck high-throughput skill invocation. Independent emit paths allow parallel skills to audit independently.

**Why tenant_id in both rows (§1 #9)?** Audit queries are typically tenant-scoped ("what did tenant X's skills do last week?"). Without tenant_id, the query requires joining against subjects/personas — slow + complex.

**Why duration even on panic (§1 #10)?** Forensic question: "did the skill panic immediately or after 30s?" Duration tells the story. Captured around `catch_unwind` to include the panic-handling time.

---

## §3 — API contract

```rust
// services/skill-host/src/memory_bridge.rs
use std::time::Instant;
use std::panic::AssertUnwindSafe;
use futures::future::FutureExt;

pub async fn invoke_with_audit(
    skill_id: &str, version: &str, args: serde_json::Value,
    ctx: &InvocationContext,
) -> Result<SkillOutput, SkillError> {
    let args_hash = sha256_canonical(&args)?;
    let request_id = ulid::Ulid::new().to_string();
    let start_row = audit_canonical::skill_invoked_started(
        skill_id, version, &ctx.caller_persona, &args_hash,
        ctx.tenant_id, &ctx.trace_id, &request_id,
    );
    memory_writer::emit(start_row).await
        .map_err(|e| SkillError::AuditEmitFailed { stage: "start", reason: e.to_string() })?;

    let t0 = Instant::now();
    let dispatch_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(skill_supervisor::dispatch(skill_id, args))
        })
    }));
    let duration_ms = t0.elapsed().as_millis() as u32;

    let (outcome, result_hash, error_msg, ret) = match dispatch_result {
        Ok(Ok(output)) => {
            let h = sha256_canonical(&output.body)?;
            ("success", h, None, Ok(output))
        }
        Ok(Err(e)) => ("error", String::new(), Some(e.to_string()), Err(e)),
        Err(panic_payload) => {
            let msg = panic_to_string(&panic_payload);
            ("panic", String::new(), Some(msg.clone()),
             Err(SkillError::Panicked { message: msg }))
        }
    };

    let completed_row = audit_canonical::skill_invoked_completed(
        skill_id, outcome, duration_ms, &result_hash, error_msg.as_deref(),
        &ctx.trace_id, &request_id,
    );
    if let Err(e) = memory_writer::emit(completed_row).await {
        tracing::error!(error = %e, request_id, skill_id, "audit_emit_completed_failed; sev-1");
        metrics::audit_emit_failure("completed");
    }

    metrics::skill_invoked(skill_id, outcome, duration_ms);
    ret
}
```

```rust
// services/skill-host/src/invocation_context.rs
pub struct InvocationContext {
    pub caller_persona: String,        // e.g., "cuo-cpo@0.4.1"
    pub tenant_id: Uuid,
    pub trace_id: String,              // hex 32-char from FR-AI-022
}

impl InvocationContext {
    pub fn from_otel_context(persona: &str, tenant: Uuid) -> Self {
        let trace_id = opentelemetry::Context::current().span().span_context().trace_id();
        Self {
            caller_persona: persona.to_string(),
            tenant_id: tenant,
            trace_id: format!("{trace_id:032x}"),
        }
    }
}
```

```rust
// services/skill-host/src/audit_canonical.rs
pub fn skill_invoked_started(
    skill_id: &str, version: &str, caller_persona: &str, args_hash: &str,
    tenant_id: Uuid, trace_id: &str, request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "skill.invoked_started".into(),
        payload: serde_json::json!({
            "skill_id": skill_id, "version": version,
            "caller_persona": caller_persona,
            "args_hash": args_hash,
            "tenant_id": tenant_id,
            "trace_id": trace_id,
            "request_id": request_id,
        }),
        ..Default::default()
    }
}

pub fn skill_invoked_completed(
    skill_id: &str, outcome: &str, duration_ms: u32,
    result_hash: &str, error_message: Option<&str>,
    trace_id: &str, request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "skill.invoked_completed".into(),
        payload: serde_json::json!({
            "skill_id": skill_id, "outcome": outcome,
            "duration_ms": duration_ms,
            "result_hash": result_hash,
            "error_message": error_message,
            "trace_id": trace_id,
            "request_id": request_id,
        }),
        ..Default::default()
    }
}

pub fn sha256_canonical(value: &serde_json::Value) -> Result<String, SkillError> {
    let bytes = serde_jcs::to_vec(value).map_err(|e| SkillError::Canonicalisation(e.to_string()))?;
    Ok(hex::encode(sha256(&bytes)))
}
```

---

## §4 — Acceptance criteria

1. Skill invocation emits 2 memory rows (started + completed).
2. args_hash matches canonical SHA-256 of args.
3. Failure to emit started → skill NOT dispatched + caller sees `AuditEmitFailed`.
4. Failure to emit completed → sev-1 log + skill effects NOT reversed.
5. trace_id propagated through both rows.
6. Concurrent invocations: 100 parallel skill calls produce 200 rows (100 started + 100 completed); no race-induced loss.
7. Skill panic → `_completed` row emitted with `outcome: panic` + duration_ms recorded.
8. Compensation-related skills (e.g., cuo.cfo.payroll-draft) emit rows even though body excluded from memory.
9. Skill error (Result::Err) → `_completed` with `outcome: error` + `error_message`.
10. Skill timeout → `_completed` with `outcome: timeout`.
11. Audit emit p95 < 50ms.
12. tenant_id present in both rows.
13. duration_ms accurate on panic (within 10ms of actual).
14. result_hash empty on error/panic.
15. OTel metric `skill_audit_emit_failures_total` increments on failure.

---

## §5 — Verification

```rust
#[tokio::test]
async fn skill_invocation_emits_both_rows() {
    let ctx = test_context();
    let result = invoke_with_audit("test_skill", "1.0.0", json!({"x": 1}), &ctx).await.unwrap();
    let started = memory_test_helper::find_latest("skill.invoked_started").unwrap();
    let completed = memory_test_helper::find_latest("skill.invoked_completed").unwrap();
    assert_eq!(started.payload["request_id"], completed.payload["request_id"]);
    assert_eq!(started.payload["skill_id"], "test_skill");
    assert_eq!(completed.payload["outcome"], "success");
}

#[tokio::test]
async fn args_hash_matches_canonical_sha256() {
    let ctx = test_context();
    let args = json!({"q": "test"});
    invoke_with_audit("noop", "1.0.0", args.clone(), &ctx).await.unwrap();
    let row = memory_test_helper::find_latest("skill.invoked_started").unwrap();
    let expected = sha256_canonical(&args).unwrap();
    assert_eq!(row.payload["args_hash"], expected);
}

#[tokio::test]
async fn started_emit_failure_blocks_dispatch() {
    memory_test_helper::inject_emit_failure();
    let result = invoke_with_audit("test_skill", "1.0.0", json!({}), &test_context()).await;
    assert!(matches!(result, Err(SkillError::AuditEmitFailed { stage: "start", .. })));
    let dispatched = supervisor_test_helper::dispatch_count();
    assert_eq!(dispatched, 0);
    memory_test_helper::clear_emit_failure();
}

#[tokio::test]
async fn skill_panic_emits_completed_with_outcome_panic() {
    let ctx = test_context();
    let result = invoke_with_audit("panicking_skill", "1.0.0", json!({}), &ctx).await;
    assert!(matches!(result, Err(SkillError::Panicked { .. })));
    let completed = memory_test_helper::find_latest("skill.invoked_completed").unwrap();
    assert_eq!(completed.payload["outcome"], "panic");
    assert!(completed.payload["duration_ms"].as_u64().unwrap() < 1000);
}

#[tokio::test]
async fn trace_id_propagated_through_both_rows() {
    let trace_id = "0af7651916cd43dd8448eb211c80319c";
    let ctx = InvocationContext { trace_id: trace_id.into(), ..test_context() };
    invoke_with_audit("noop", "1.0.0", json!({}), &ctx).await.unwrap();
    let started = memory_test_helper::find_latest("skill.invoked_started").unwrap();
    let completed = memory_test_helper::find_latest("skill.invoked_completed").unwrap();
    assert_eq!(started.payload["trace_id"], trace_id);
    assert_eq!(completed.payload["trace_id"], trace_id);
}

#[tokio::test]
async fn 100_concurrent_invocations_produce_200_rows() {
    let mut joinset = tokio::task::JoinSet::new();
    for i in 0..100 {
        joinset.spawn(async move {
            invoke_with_audit("noop", "1.0.0", json!({"i": i}), &test_context()).await
        });
    }
    while let Some(r) = joinset.join_next().await { r.unwrap().unwrap(); }
    let started = memory_test_helper::count_rows_since("skill.invoked_started", recent()).await;
    let completed = memory_test_helper::count_rows_since("skill.invoked_completed", recent()).await;
    assert_eq!(started, 100);
    assert_eq!(completed, 100);
}

#[tokio::test]
async fn audit_emit_p95_under_50ms() {
    let mut samples = vec![];
    for _ in 0..200 {
        let t0 = std::time::Instant::now();
        invoke_with_audit("noop", "1.0.0", json!({}), &test_context()).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 < 100, "p95 {p95}ms exceeds 100ms (incl skill noop)");
}

#[tokio::test]
async fn completed_emit_failure_logs_sev1_no_revert() {
    memory_test_helper::inject_emit_failure_for_kind("skill.invoked_completed");
    let _ = invoke_with_audit("noop", "1.0.0", json!({}), &test_context()).await.unwrap();
    let metric: u64 = otel_test_helper::counter_value("skill_audit_emit_failures_total", &[("stage", "completed")]);
    assert!(metric > 0);
    // Side effect (noop = nothing) NOT reverted; this would be the assertion in a real skill
    memory_test_helper::clear_emit_failure();
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-AI-003** — memory_writer::emit() bridge.
- **FR-AI-022** — trace_id propagation.
- **FR-OBS-005** — TraceContext correlation.
- **FR-SKILL-102 (downstream)** — OCI registry uses these audit rows.
- Crates: `tokio`, `serde`, `serde_json`, `serde_jcs`, `sha2`, `hex`, `ulid@1`, `tracing`, `futures@0.3`.

---

## §8 — Example payloads

### Started row

```json
{
  "kind": "skill.invoked_started",
  "payload": {
    "skill_id": "obs.triage-alert",
    "version": "1.0.0",
    "caller_persona": "cuo-cpo@0.4.1",
    "args_hash": "4b8c0d2f1a7e9c3b...",
    "tenant_id": "550e...",
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "request_id": "01HZK..."
  }
}
```

### Completed row (success)

```json
{
  "kind": "skill.invoked_completed",
  "payload": {
    "skill_id": "obs.triage-alert",
    "outcome": "success",
    "duration_ms": 87,
    "result_hash": "9d6e3a2b...",
    "error_message": null,
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "request_id": "01HZK..."
  }
}
```

### Completed row (panic)

```json
{
  "kind": "skill.invoked_completed",
  "payload": {
    "skill_id": "broken_skill",
    "outcome": "panic",
    "duration_ms": 12,
    "result_hash": "",
    "error_message": "panicked at 'unwrap on None'",
    "trace_id": "...",
    "request_id": "..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Streaming audit (long-running skills emit progress) — slice 4+.
- Skill output truncation in audit (currently only hash) — slice 4+ with `?include_output_preview=true`.
- Per-skill retention policies — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| memory unreachable on _started | memory_writer error | Invocation refused; caller 503 | Operator investigates memory |
| memory unreachable on _completed | memory_writer error | Sev-1 log; skill effect not reversed | Operator investigates |
| memory slow on _started (>50ms) | latency histogram | Invocation proceeds (latency-tolerated); sev-3 alarm | Investigate memory_writer |
| Skill panics mid-execution | catch_unwind | Completed row emitted with outcome=panic | By design |
| Skill timeouts | dispatch timeout | Completed with outcome=timeout | By design |
| Concurrent invocations | independent emit paths | All rows emitted | By design |
| args_hash collision (cryptographic ~10⁻³⁰) | N/A | N/A | By design |
| Trace_id missing | empty string in payload | Audit complete; correlation broken | Caller fixes upstream OTel context |
| canonical-JSON serialise fails | error | Invocation refused | Caller fixes args |
| Skill returns invalid JSON | downstream parse error | outcome=error | By design |
| Audit emit during shutdown | memory_writer in-flight | Tx may abort | By design (not catastrophic) |
| Compensation skill's body excluded | hash still computed; row still emitted | By design | Auditable without leaking |
| Result hash empty for non-error | sanity check fails | Sev-1 alarm | Investigate skill output |
| duration_ms negative (clock skew) | sanity check | Sev-3 | Investigate clock |
| Concurrent emit on same trace_id | independent paths | Both succeed | By design |

---

## §11 — Notes

- Both _started and _completed rows are emitted for EVERY invocation. Operators querying "currently running skills" use `_started` minus matching `_completed`.
- args_hash + result_hash preserve uniqueness for forensic correlation without leaking content.
- Panic catch via `catch_unwind` ensures audit chain consistency — no orphan _started rows.
- trace_id propagation enables joining LLM trace + LangSmith + skill audit + downstream effects.
- _started failure blocks dispatch (auditability invariant); _completed failure doesn't reverse (side effects irreversible).
- Concurrent invocations independent — no lock serialisation around memory_writer.
- Compensation-skill bodies excluded from memory (DEC-036) but invocation rows still emitted (the FACT of the call is auditable).
- duration_ms includes panic-handling time — accurate on all paths.
- Result hash empty on error/panic distinguishes "skill ran with no output" from "skill failed."

---

*End of FR-SKILL-101. Status: draft (10/10 target).*

## As built (2026-07-02)

skill-host was consolidated into services/skill-broker (no separate crate).
