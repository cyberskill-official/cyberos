---
# ───── Machine-readable frontmatter ─────
id: TASK-AI-003
title: "memory audit-row bridge — canonical Writer for AI Gateway"
module: AI
priority: MUST
status: done
accepted_at: 2026-05-15
accepted_by: Stephen Cheng
verify: T
phase: P0
milestone: P0 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-AI-002, TASK-AI-014, TASK-OBS-008]
depends_on: []
blocks: [TASK-AI-001, TASK-AI-002, TASK-AI-014, TASK-AI-022, TASK-AI-004, TASK-PROJ-002, TASK-SKILL-101, TASK-EMAIL-005]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#bigger-picture
  - website/docs/modules/memory.html#audit-chain
  - website/docs/architecture/audit-chain.html
source_decisions:
  - cyberos/AGENTS.md §3 (file operations) and §6 (audit ledger)
  - cyberos/AGENTS.md §11 (prompt-injection trust model — audit row provenance)
  - archive/2026-05-14/RESEARCH_REVIEW.md §2.4 (audit-before-action invariant)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/memory_writer.rs
  - services/ai-gateway/src/memory_writer/canonical.rs
  - services/ai-gateway/src/memory_writer/subprocess.rs
  - services/ai-gateway/tests/memory_writer_test.rs
modified_files:
  - services/ai-gateway/Cargo.toml   # add serde_json, sha2, base64, tokio process features
  - services/ai-gateway/src/lib.rs   # export memory_writer module
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_read: cyberos/{src,scripts}/**   # read-only access to canonical Writer source
  - file_write: services/ai-gateway/{src,tests}/**
  - bash: cargo test -p cyberos-ai-gateway memory_writer
  - bash: python3 -m cyberos.writer --help   # smoke-test the Writer CLI handshake
disallowed_tools:
  - direct write to <memory-root>/audit/*.binlog   (MUST route through canonical Writer)
  - direct write to <memory-root>/HEAD   (MUST route through canonical Writer)
  - in-process Python (PyO3) — deferred to TASK-AI-008 (slice 2 spike)
  - bypass the .lock seqlock when emitting

# ───── Estimated work ─────
effort_hours: 5
subtasks:
  - "0.5h: enumerate row kinds emitted by AI Gateway (ai.precheck, ai.invocation, ai.invocation_failed, ai.hold_expired, ai.persona_loaded)"
  - "1.0h: canonical JSON shape per AGENTS.md §6.2 (sorted keys, NFC, BE u64 framing)"
  - "1.0h: subprocess wrapper around `python3 -m cyberos.writer put` with stdin payload"
  - "1.0h: synchronous result handling + chain-hash extraction from stdout"
  - "1.0h: error taxonomy + retry policy (network-style transient errors only)"
  - "0.5h: integration test (real Writer subprocess, real memory, real chain verification)"
risk_if_skipped: "AI Gateway cannot emit audit rows. TASK-AI-001 and TASK-AI-002 both call into this bridge; both block. Beyond that: any future AI call would slip through with no provenance, violating the audit-before-action invariant and breaking EU AI Act Art. 12."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** provide a `memory_writer` Rust module that emits chained, content-addressed audit rows to the local memory via the canonical Writer subprocess. The module exposes one public emission function and a typed set of row kinds; every other AI Gateway path that needs to write to memory MUST route through this module.

The `memory_writer::emit()` function:

1. **MUST** accept a typed `MemoryEmit` struct describing the row (kind, path, extra payload).
2. **MUST** spawn `python3 -m cyberos.writer put` as a subprocess, pipe the canonical-JSON payload on stdin, wait for the subprocess to exit, and read the resulting `{seq, chain, ts_ns}` triple on stdout.
3. **MUST** treat a non-zero subprocess exit code as a hard failure (`Err(MemoryWriterError::WriterFailed)`); callers MUST NOT proceed with the action the row was guarding.
4. **MUST** complete within 50ms p95 for a single row; budget includes process spawn (~15ms), file lock acquisition (~3ms), durable-sync (~10ms), stdout read (~2ms). The remaining 20ms is the canonical-JSON serialisation + the Writer's invariant checks.
5. **MUST** be re-entrant: multiple AI Gateway tokio tasks calling `emit()` concurrently MUST serialise through the Writer's `.lock` (POSIX `LOCK_EX`) — no parallel chain advances are permitted.
6. **MUST** validate the payload's canonical form before invoking the subprocess: NFC-normalised UTF-8, sorted keys, no insignificant whitespace, integers in their natural form. Per `AGENTS.md §6.2`.
7. **MUST** verify the returned `chain` hash equals `SHA-256(canonical(record_minus_chain) || prev_chain)`; refuse to return `Ok` to the caller if the verification fails (catches a corrupted Writer).
8. **MUST** support the following closed set of `ai.*` row kinds (updated 2026-05-16 as additional FRs were specified):
    - Slice-1 initial: `ai.precheck`, `ai.invocation`, `ai.invocation_failed`, `ai.hold_expired`, `ai.persona_loaded`.
    - Added by TASK-AI-009 (circuit breaker): `ai.failover_triggered`.
    - Added by TASK-AI-015 (ZDR enforcement): `ai.zdr_violation`.
    - Added by TASK-AI-016 (residency pinning): `ai.residency_violation`.
    - Added by TASK-AI-017 (per-tenant cache): `ai.cache_hit`.
    - Added by TASK-AI-021 (operator CLI) — operator-action rows in `ai.cli_*` sub-namespace: `ai.cli_policy_updated`, `ai.cli_failover_drill`, `ai.cli_invoice_exported`, `ai.cli_breaker_reset`, `ai.cli_expiry_repaired`, `ai.cli_memory_emitted`.
    - Cross-module rows in adjacent namespaces (NOT owned by this bridge but emitted via the module's own writer; registered here per task-audit skill §3.2 rule 8 for the canonical closed-set): `auth.*` per TASK-AUTH-001/002/004/006; `memory.sync_row_filtered` per TASK-MEMORY-106; `skill.invoked_started` + `skill.invoked_completed` per TASK-SKILL-101; `obs.*` per FR-OBS module.
    - **CHAT module rows** (added via the strict-redo 2026-05-16 P.M. expansion of TASK-CHAT-002..012; emitted by services/chat/*, services/chat-memory-bridge/*, services/chat-importer/*, services/chat-lumi/*, services/chat-push/*, services/chat-dsar/*):
        - TASK-CHAT-002 (authbridge): `chat.session_started`.
        - TASK-CHAT-003 (Fargate deployment): `chat.deployment_provisioned`, `chat.deployment_inventory`, `chat.deployment_drift_detected`, `chat.tier_upgraded`, `chat.pitr_test_passed`, `chat.deployment_warning`, `chat.maintenance_started`, `chat.deployment_blocked`, `chat.tf_state_recovered`, `chat.module_deprecated`.
        - TASK-CHAT-005 (memory bridge): `chat.message`, `chat.message_edited`, `chat.message_deleted`, `chat.channel_created`, `chat.channel_archived`, `chat.message_reacted`, `chat.message_unreacted`, `chat.user_joined_channel`, `chat.user_left_channel`, `chat.message_attachment`, `chat.bridge_heartbeat`, `chat.bridge_lag_alert`, `chat.bridge_misconfigured`, `chat.bridge_shutdown`, `chat.bridge_redaction_failed`.
        - TASK-CHAT-006 (Slack import): `chat.import_started`, `chat.import_step_completed`, `chat.import_finished`, `chat.import_warning`, `chat.import_verification_failed`, `chat.import_aborted`, `chat.import_failed`, `chat.import_rate_limit_exhausted`.
        - TASK-CHAT-007 (Zalo import): reuses the TASK-CHAT-006 `chat.import_*` family AND adds `chat.import_unsupported_zalo_version`, `chat.import_timestamp_ambiguous`, `chat.import_conversation_completed`.
        - TASK-CHAT-008 (Lumi mention): `chat.lumi_invoked`, `chat.lumi_skipped`, `chat.lumi_error`.
        - TASK-CHAT-009 (retro capture): `chat.retro_capture_started`, `chat.retro_capture_completed`, `chat.retro_capture_cancelled`, `chat.retro_capture_expired`, `chat.retro_capture_truncated`.
        - TASK-CHAT-010 (decommission signal): `chat.decommission_signal`, `chat.decommission_state_changed`, `chat.decommission_snoozed`, `chat.decommission_unsnoozed`.
        - TASK-CHAT-011 (mobile push): `chat.push_delivered`, `chat.push_failed`, `chat.push_dnd_dropped`.
        - TASK-CHAT-012 (DSAR export): `chat.dsar_requested`, `chat.dsar_exporting`, `chat.dsar_delivered`, `chat.dsar_fully_delivered`, `chat.dsar_acknowledged`, `chat.dsar_failed`, `chat.dsar_expired`, `chat.dsar_url_reused`.
    - **PROJ module rows** (added via the strict-redo 2026-05-16 P.M. expansion of TASK-PROJ-002, 005..018; emitted by services/proj-sync/* and web/proj-client/*):
        - TASK-PROJ-001 (issue schema baseline): `proj.issue_status_changed` (superseded by TASK-PROJ-002's `proj.decision` for status transitions; status_changed retained for non-decision metadata updates).
        - TASK-PROJ-002 (memory-anchored decision): `proj.decision`, `proj.decision_retracted`.
        - TASK-PROJ-005 (rate-card schema): `proj.rate_card_created`, `proj.rate_card_superseded`, `proj.rate_card_billable_default_changed`, `proj.rate_card_corrected`, `proj.rate_card_currency_mismatch`.
        - TASK-PROJ-006 (billable cascade): `proj.billable_resolved`, `proj.member_billable_override_set`, `proj.task_class_billable_set`.
        - TASK-PROJ-007 (billing modes): `proj.billing_mode_set`, `proj.billing_mode_changed`, `proj.milestone_invoiced`, `proj.milestone_cancelled`, `proj.milestone_added`, `proj.retainer_overage_emitted`, `proj.retainer_overage_streak`, `proj.retainer_rollover_carry_forward`, `proj.billing_mixed_currency_period`.
        - TASK-PROJ-008 (history events): `proj.issue_mutated`, `proj.chain_tampered`.
        - TASK-PROJ-009 (memory links): `proj.memory_link_created`, `proj.memory_link_removed`, `proj.memory_link_traversed`.
        - TASK-PROJ-010 (citation drift): `proj.citation_drift_detected`, `proj.drift_remediated`.
        - TASK-PROJ-011 (blocker detector): `proj.blocker_detected`, `proj.blocker_resolved`, `proj.blocker_resolved_diff`, `proj.blocker_stale`, `proj.blocker_escalated`, `proj.blocker_cycle_detected`.
        - TASK-PROJ-012 (cycle review): `proj.cycle_review_drafted`, `proj.cycle_review_accepted`, `proj.cycle_review_iterated`, `proj.cycle_review_skipped`, `proj.cycle_goal_updated`.
        - TASK-PROJ-013 (estimate calibration): `proj.estimate_calibration_computed`, `proj.calibration_drift_alert`, `proj.calibration_backfilled`.
        - TASK-PROJ-014 (Kanban): `proj.kanban_card_moved`, `proj.kanban_card_move_undone`, `proj.wip_limit_overridden`.
        - TASK-PROJ-015 (Timeline): `proj.timeline_bar_moved`.
        - TASK-PROJ-016 (Gantt): `proj.dependency_added`, `proj.dependency_removed`, `proj.dependency_near_cycle`, `proj.critical_path_recomputed`.
        - TASK-PROJ-017 (Brief Modal): `proj.brief_modal_opened`.
        - TASK-PROJ-018 (design tokens): n/a — visual/CI FR; no audit rows.
   Adding a new `ai.*` kind requires (a) a Rust enum variant, (b) a serde-derive update, (c) a unit test, (d) a docs cross-reference to the kind's purpose, (e) a one-line update to this clause. Adding a new `chat.*` or `proj.*` kind requires the same plus a one-line append to the per-FR sublist above so this clause stays the closed-set source of truth across the platform.
9. **MUST** NOT bypass the Writer to write directly to `<memory-root>/audit/*.binlog` or `<memory-root>/HEAD`. The Writer is the only legal mutator (per `AGENTS.md §14.1`).
10. **MUST** run a startup health check via `python3 -m cyberos.writer --version` as a synchronous subprocess call during AI Gateway init. Non-zero exit MUST cause the gateway to exit 1 with stderr `memory_writer: Writer subprocess unavailable: <reason>`. This catches misconfigured deploys at deploy time, not first-request time.
11. **MUST NOT** expose `memory_writer::emit_batch()` in slice 1. Callers needing N rows MUST call `emit()` N times (or use `futures::future::join_all`). TASK-AI-008 (slice 2 PyO3 spike) re-introduces a real batching API.
12. **MUST** log `ChainHashMismatch` errors at sev-1 via `tracing::error!` with structured fields: `expected_chain` (hex), `actual_chain` (hex), `seq`, `tenant_id`, `payload_canonical_hash`. The OBS dashboard (TASK-OBS-007) routes sev-1 chain events to PagerDuty until TASK-OBS-007 lands; until then, `tracing::error!` lines surface via journald.
13. **SHOULD** keep chain-hash verification ON by default in both dev and prod. The 5µs per emit cost is negligible compared to the 30ms subprocess fork. A feature flag `memory_writer/skip-verify` exists for ad-hoc debugging but MUST NOT be enabled in prod (CI gate enforces).
14. **SHOULD** emit OTel metrics: `memory_writer_emit_calls_total{kind,outcome}` (counter; outcome ∈ ok/err), `memory_writer_emit_latency_ms` (histogram), `memory_writer_chain_mismatches_total` (counter; sev-1 dashboard alarm at > 0), `memory_writer_writer_unavailable_total` (counter), `memory_writer_path_rejections_total{reason}` (counter).
15. **MUST NOT** expose a `dry_run` mode in slice 1. TASK-AI-021 (operator CLI) adds a `cyberos-ai memory emit --dry-run` subcommand that validates payload without emission; the bridge itself stays single-purpose.
16. **MUST** emit history events as pairs per task-audit skill §3.8 rule 26: every `ai.precheck_started` row MUST be followed by either `ai.precheck_completed` OR `ai.precheck_failed` within 30s (the precheck deadline). The same pairing rule applies to `ai.reconcile_started` / `ai.reconcile_completed | ai.reconcile_failed`. Standalone `*_started` rows are crash signals — an OBS Grafana lint alerts when `count(*_started) - count(*_completed) - count(*_failed) > 0` over any 5-minute window for any (tenant, module).
17. **MUST** include `extra.trace_id: String` (32-char lower-hex W3C `trace-id` form per task-audit skill §3.7 rule 23) in every emitted `MemoryRow`. The format MUST be produced via OTel `TraceId` Display (NOT Debug — Debug yields `TraceId(0af7…)` while Display yields the 32-char hex per task-audit skill §3.7 rule 24). AC #17 verifies via regex `^[0-9a-f]{32}$` against every emitted row's `extra.trace_id` in the round-trip test.

This FR is the load-bearing piece beneath every other AI Gateway audit emission. It is also the slowest part of the call hot path (subprocess fork dominates the precheck's 50ms budget) — getting it right matters for both correctness and throughput.

---

## §2 — Why this design (rationale for humans)

**Why a subprocess and not an in-process Python binding (PyO3)?** Three reasons. (1) The canonical Writer is the *only* code authorised to mutate `<memory-root>/`; running it as a separate process gives us strong isolation and a single, auditable code path for every memory write across the whole platform. (2) PyO3 adds a build-time dependency on a specific Python version and a runtime dependency on the GIL; both make the AI Gateway harder to ship and harder to test. (3) The 30ms subprocess overhead is acceptable for slice 1; TASK-AI-008 (slice 2 spike) revisits PyO3 if real-world traffic justifies it.

**Why call it synchronously on the precheck/reconcile hot path?** The audit-before-action invariant (`AGENTS.md §11`, research review §2.4) requires that the audit row reach the chain *before* the action it guards. An async-spawn-and-return pattern would let a precheck `Allow` return before its `ai.precheck` row lands; a process crash between those two events would leave us with a chain that doesn't match what the gateway actually did. Synchronous emission costs us 30ms; async emission would cost us the integrity of the chain. Pick correctness.

**Why is the chain-hash verification step needed?** The canonical Writer is trusted, but verification is cheap (one SHA-256 over ~600 bytes ≈ 5µs) and catches an entire class of bugs: a Writer that silently drops fields, a Writer that injects fields, a Writer whose canonical-JSON serialiser diverges from the AI Gateway's. The mismatch would silently break the chain; better to refuse the row and fail loudly.

**Why route every kind through one bridge instead of one bridge per kind?** Future kinds will land (`ai.cache_hit`, `ai.failover_triggered`, `ai.persona_stamp_mismatch`). One bridge with an enum gives us a single audit-emission entry point that every CI gate, every integration test, and every prod observability dashboard can hook into. A bridge-per-kind ecology fragments faster than a Vietnamese coffee shop's signage.

**Why does the bridge not own retries?** Because the only retryable failures (`.lock` contention, Writer subprocess transient SIGTERM) are already retried inside the Writer itself. From the AI Gateway's perspective, a `MemoryWriterError` is final: either the row landed (Ok) or it didn't (Err); there is no "maybe". The caller's retry policy lives in the caller's domain (e.g., precheck retries the entire `precheck()` call, not just the audit emission).

---

## §3 — API contract

### Public function signatures

```rust
// services/ai-gateway/src/memory_writer.rs
pub async fn emit(emit_request: MemoryEmit) -> Result<EmittedRow, MemoryWriterError>;

// Slice-1 startup health check
pub async fn check_writer_available() -> Result<WriterVersion, MemoryWriterError>;

pub struct MemoryEmit {
    pub kind: AiInvocationKind,
    pub path: String,                  // memory file path under <memory-root>/, validated by Writer
    pub extra: serde_json::Value,      // closed schema per kind; validated by the typed builder below
    // NOTE: dedup_key is deferred to TASK-AI-008 (slice 2 PyO3 spike). Slice-1 callers must
    // tolerate duplicate audit rows on crash recovery; TASK-AI-021 operator CLI provides a
    // `cyberos-ai expiry repair` command to dedupe after the fact.
}

pub struct WriterVersion {
    pub semver: String,
    pub commit: String,
    pub schema_version: u32,
}

pub enum AiInvocationKind {
    Precheck,           // emitted by TASK-AI-001
    Invocation,         // emitted by TASK-AI-002 (success path)
    InvocationFailed,   // emitted by TASK-AI-002 (refund path)
    HoldExpired,        // emitted by TASK-AI-004 (cleanup job)
    PersonaLoaded,      // emitted by TASK-AI-014 (persona stamping)
}

pub struct EmittedRow {
    pub seq: u64,
    pub ts_ns: u64,
    pub chain: [u8; 32],
    pub path: String,
}

pub enum MemoryWriterError {
    WriterFailed { exit_code: i32, stderr: String },
    WriterUnreachable { reason: String },
    CanonicalisationFailed { reason: String },
    ChainHashMismatch { expected: [u8; 32], got: [u8; 32] },
    PathRejected { path: String, reason: String },  // path traversal, invalid kind, etc.
    Timeout { waited_ms: u32 },                     // subprocess hung > 5s
}
```

### Typed builders (closed schema per kind)

```rust
// services/ai-gateway/src/memory_writer/canonical.rs

pub fn precheck(
    tenant_id: &str,
    agent_persona: &str,
    model_alias: &str,
    resolved_provider: &str,
    resolved_model: &str,
    estimated_usd: Decimal,
    current_spent_usd: Decimal,
    idempotency_key: &str,
) -> MemoryEmit { /* … */ }

pub fn invocation(
    tenant_id: &str,
    agent_persona: &str,
    model_alias: &str,
    resolved_provider: &str,
    resolved_model: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    actual_usd: Decimal,
    hold_id: Uuid,
    latency_ms: u32,
    cache_state: CacheState,
    provider_request_id: &str,
    new_spent_total_usd: Decimal,
    warn_crossed: bool,
    cancelled: bool,
) -> MemoryEmit { /* … */ }

pub fn invocation_failed(
    tenant_id: &str,
    agent_persona: &str,
    resolved_provider: &str,
    resolved_model: &str,
    http_status: u16,
    retryable: bool,
    provider_error_message: &str,
    hold_id: Uuid,
    refund_amount_usd: Decimal,
) -> MemoryEmit { /* … */ }

pub fn hold_expired(
    tenant_id: &str,
    hold_id: Uuid,
    expired_at: DateTime<Utc>,
    refund_amount_usd: Decimal,
) -> MemoryEmit { /* … */ }

pub fn persona_loaded(
    tenant_id: &str,
    persona_id: &str,
    persona_version: &str,
    source_path: &str,
    source_hash: [u8; 32],
) -> MemoryEmit { /* … */ }
```

The typed builders prevent any AI Gateway code from emitting a row with the wrong shape — the call-site is statically constrained.

### Subprocess handshake

```text
spawn:        python3 -m cyberos.writer put
stdin:        { "path": "...", "body": "...", "meta": { "kind": "...", ... } }
              (single line of canonical JSON, terminated by '\n')
expected exit: 0 on success, 1 on schema rejection, 2 on lock contention, 3 on path traversal
stdout (on 0): { "seq": <u64>, "ts_ns": <u64>, "chain": "<hex>" }
              (single line, terminated by '\n')
stderr (on 0): empty (debug logs go to syslog via the Writer's own logging)
stderr (on >0): JSON error: { "code": "<id>", "detail": "<text>" }
timeout:      5 seconds; if exceeded, kill -SIGTERM, then kill -SIGKILL after 1s grace
```

### Canonical JSON shape per AGENTS.md §6.2

```json
{
  "path": "memories/decisions/ai-invocations/1763112131000_org-cyberskill_01HZK9R7A2B4C8D6.md",
  "body": "---\nkind: ai.invocation\ntenant_id: org:cyberskill\n…\n---\n",
  "meta": {
    "kind": "ai.invocation",
    "actor": "agent:cyberos-ai-gateway",
    "actor_version": "0.1.0",
    "extra": {
      "tenant_id": "org:cyberskill",
      "agent_persona": "cuo-cpo@0.4.1",
      "actual_usd": 0.0078,
      "prompt_tokens": 120,
      "completion_tokens": 450,
      "hold_id": "01HZK9R8M3X5C8Q4"
    }
  }
}
```

(Sorted keys; NFC UTF-8; no insignificant whitespace; integers natural form.)

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy path (precheck row)** — `memory_writer::emit(canonical::precheck(...))` MUST return `Ok(EmittedRow { seq, ts_ns, chain })`. The row MUST appear at `<memory-root>/memories/ai-invocations/<ts_ns>_<tenant>_<key>.md`. The returned `seq` MUST equal the memory's HEAD seq counter immediately after emission. The chain hash MUST equal SHA-256(canonical(row_minus_chain) ‖ prev_chain).
2. **Concurrent emission serialises** — 16 tokio tasks calling `emit()` concurrently MUST result in 16 distinct rows on the chain, with monotonically increasing seq numbers and `chain` linkage preserved end-to-end. Test: parallel `for i in 0..16 { tokio::spawn(emit(...)) }`; assert seq is contiguous and chain verification passes for all 16 rows.
3. **Writer non-zero exit → Err** — When the subprocess returns exit code `3` (path traversal), `emit()` MUST return `Err(MemoryWriterError::PathRejected)` carrying the stderr's `detail` field; MUST NOT emit a row; MUST NOT mutate `<memory-root>/`.
4. **Writer timeout → kill + Err** — When the subprocess hangs for >5s (test injects a `time.sleep(10)` via a Writer fixture), `emit()` MUST send SIGTERM, wait 1s, send SIGKILL, and return `Err(MemoryWriterError::Timeout { waited_ms: 5000 })`. MUST NOT leave a zombie process; MUST NOT leave a partial row on disk.
5. **Chain-hash mismatch → Err** — When the Writer returns a `chain` that does NOT match the AI Gateway's recomputation (test injects a corrupted-Writer fixture), `emit()` MUST return `Err(MemoryWriterError::ChainHashMismatch)`; MUST log a sev-1 OBS event; MUST NOT return `Ok`.
6. **Canonical-JSON divergence → Err** — When the payload contains non-NFC UTF-8 (test injects `"café"` with combining-acute instead of pre-composed), `emit()` MUST normalise to NFC before sending. Round-trip: payload-in → canonical → payload-out → assert NFC-form equality.
7. **Path validation** — `MemoryEmit { path: "../escape.md", ... }` MUST be rejected at the bridge boundary (BEFORE subprocess spawn) with `Err(MemoryWriterError::PathRejected { reason: "traversal" })`. Same for absolute paths and paths under reserved directories (`audit/`, `index/`).
8. **Latency budget** — `emit()` MUST complete within 50ms p95 over a 1000-call integration test against a real memory store on tmpfs. Per-call breakdown logged via `tracing` spans for TASK-AI-022.
9. **Typed-builder fidelity** — Each of the 5 typed builders, when emitted, MUST produce a row whose `extra` matches the builder's input args 1:1 (no field renames, no silent type coercions). Test: round-trip `canonical::invocation(...)` → emit → re-read the row from disk → assert field equality.
10. **No-leak under panic** — If the caller's tokio task is cancelled mid-emit, the subprocess MUST still be reaped (Drop impl on `EmitGuard`). Test: spawn 100 emits, cancel half at random points; assert `ps -ef | grep cyberos.writer` returns 0 after a 1s settling period.

---

## §5 — Verification method

**Unit tests:** `services/ai-gateway/src/memory_writer/canonical.rs` has 5 typed-builder unit tests (one per kind) verifying that the builder produces the documented JSON shape with sorted keys.

**Integration tests:** `services/ai-gateway/tests/memory_writer_test.rs`

```rust
#[tokio::test]
async fn emit_precheck_happy_path() {
    let env = TestMemory::new_tmpfs().await;
    let outcome = memory_writer::emit(
        canonical::precheck(
            "org:test-a", "cuo-cpo@0.4.1", "chat.smart",
            "bedrock", "anthropic.claude-3.5-sonnet",
            dec!(0.0085), dec!(47.23), "01HZK9R7A2B4C8D6",
        )
    ).await.unwrap();

    assert!(outcome.seq > 0);
    assert!(env.row_exists_at(&outcome.path).await);
    let on_disk = env.read_row(outcome.seq).await;
    assert_eq!(on_disk.chain, outcome.chain);
    assert!(env.verify_chain_from_genesis().await);
}

#[tokio::test]
async fn emit_concurrent_serialises() {
    let env = TestMemory::new_tmpfs().await;
    let handles: Vec<_> = (0..16).map(|i| {
        let path = format!("test_concurrent_{}", i);
        tokio::spawn(memory_writer::emit(test_emit_with_path(&path)))
    }).collect();

    let outcomes: Vec<_> = futures::future::join_all(handles).await
        .into_iter().map(|r| r.unwrap().unwrap()).collect();

    let mut seqs: Vec<u64> = outcomes.iter().map(|o| o.seq).collect();
    seqs.sort();
    for w in seqs.windows(2) { assert_eq!(w[1], w[0] + 1, "seq gap"); }
    assert!(env.verify_chain_from_genesis().await);
}

#[tokio::test]
async fn emit_writer_timeout_returns_err() {
    let env = TestMemory::with_hung_writer().await;
    let outcome = memory_writer::emit(test_emit()).await;
    assert!(matches!(outcome, Err(MemoryWriterError::Timeout { .. })));
    assert_no_zombie_writer().await;
}

#[tokio::test]
async fn emit_chain_hash_mismatch_rejects() {
    let env = TestMemory::with_corrupted_writer().await;
    let outcome = memory_writer::emit(test_emit()).await;
    assert!(matches!(outcome, Err(MemoryWriterError::ChainHashMismatch { .. })));
}

#[tokio::test]
async fn emit_path_traversal_rejects_before_spawn() {
    let outcome = memory_writer::emit(MemoryEmit {
        kind: AiInvocationKind::Precheck,
        path: "../../etc/passwd".to_string(),
        extra: json!({}),
    }).await;
    assert!(matches!(outcome, Err(MemoryWriterError::PathRejected { .. })));
    // Writer subprocess MUST NOT have been spawned
    assert_eq!(WRITER_SPAWN_COUNT.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn emit_normalises_nfc() {
    let env = TestMemory::new_tmpfs().await;
    // "café" with COMBINING ACUTE (U+0065 U+0301), not pre-composed (U+00E9)
    let payload_with_decomposed = "cafe\u{0301}";
    let outcome = memory_writer::emit(test_emit_with_extra_string(payload_with_decomposed)).await.unwrap();
    let on_disk = env.read_row(outcome.seq).await;
    // After NFC normalisation, the body bytes should contain U+00E9 (0xC3 0xA9 in UTF-8)
    assert!(on_disk.body_bytes.windows(2).any(|w| w == [0xC3, 0xA9]),
        "expected NFC-normalised pre-composed é");
    assert!(!on_disk.body_bytes.windows(2).any(|w| w == [0xCC, 0x81]),
        "expected combining-acute U+0301 to be normalised away");
}
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway memory_writer
```

**Property test:** chain integrity property — for any sequence of N emits (N in 1..50), the chain hash at row N MUST equal SHA-256(canonical(row_N_without_chain) ‖ chain_{N-1}). Run via `cargo proptest`.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/memory_writer.rs

use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use sha2::{Sha256, Digest};

const WRITER_BIN: &str = "python3";
const WRITER_ARGS: &[&str] = &["-m", "cyberos.writer", "put"];
const WRITER_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn emit(req: MemoryEmit) -> Result<EmittedRow, MemoryWriterError> {
    // 1. Validate path before spawning
    validate_path(&req.path).map_err(|reason| MemoryWriterError::PathRejected {
        path: req.path.clone(), reason,
    })?;

    // 2. Canonicalise payload
    let body = build_body_markdown(&req)?;
    let payload = canonical::serialise(&CanonicalPayload {
        path: &req.path,
        body: &body,
        meta: &Meta::for_kind(&req.kind, &req.extra),
    }).map_err(|e| MemoryWriterError::CanonicalisationFailed { reason: e })?;

    // 3. Spawn Writer
    let mut child = Command::new(WRITER_BIN)
        .args(WRITER_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| MemoryWriterError::WriterUnreachable { reason: e.to_string() })?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // 4. Pipe payload + drop stdin to signal EOF
    let payload_bytes = format!("{}\n", payload).into_bytes();
    let write_fut = async move {
        let mut stdin = stdin;
        stdin.write_all(&payload_bytes).await?;
        stdin.shutdown().await?;
        Ok::<_, std::io::Error>(())
    };

    // 5. Read stdout + stderr concurrently with timeout
    let (write_res, exit_res, stdout_buf, stderr_buf) = match timeout(WRITER_TIMEOUT, async {
        let (w, e, so, se) = tokio::join!(
            write_fut,
            child.wait(),
            read_all(stdout),
            read_all(stderr),
        );
        (w, e, so, se)
    }).await {
        Ok(t) => t,
        Err(_) => {
            // Kill subprocess on timeout
            let _ = child.start_kill();
            tokio::time::sleep(Duration::from_secs(1)).await;
            return Err(MemoryWriterError::Timeout { waited_ms: 5000 });
        }
    };

    write_res.map_err(|e| MemoryWriterError::WriterUnreachable { reason: e.to_string() })?;
    let exit = exit_res.map_err(|e| MemoryWriterError::WriterUnreachable { reason: e.to_string() })?;

    if !exit.success() {
        return Err(MemoryWriterError::WriterFailed {
            exit_code: exit.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&stderr_buf?).to_string(),
        });
    }

    // 6. Parse stdout → EmittedRow
    let row: WriterStdout = serde_json::from_slice(&stdout_buf?)
        .map_err(|e| MemoryWriterError::WriterFailed {
            exit_code: 0, stderr: format!("stdout parse: {}", e),
        })?;

    // 7. Verify chain hash
    let expected = compute_expected_chain(&payload, &row.prev_chain);
    let actual = hex::decode(&row.chain).map_err(|_| MemoryWriterError::ChainHashMismatch {
        expected, got: [0u8; 32],
    })?;
    if expected[..] != actual[..] {
        let mut got = [0u8; 32];
        got.copy_from_slice(&actual);
        return Err(MemoryWriterError::ChainHashMismatch { expected, got });
    }

    Ok(EmittedRow { seq: row.seq, ts_ns: row.ts_ns, chain: expected, path: req.path })
}

fn validate_path(path: &str) -> Result<(), String> {
    if path.starts_with('/') || path.starts_with('\\') { return Err("absolute".into()); }
    if path.contains("..") { return Err("traversal".into()); }
    for reserved in ["audit/", "index/", "HEAD", ".lock"] {
        if path.starts_with(reserved) { return Err(format!("reserved: {}", reserved)); }
    }
    Ok(())
}

fn compute_expected_chain(canonical_payload: &str, prev_chain_hex: &str) -> [u8; 32] {
    let prev = hex::decode(prev_chain_hex).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical_payload.as_bytes());
    hasher.update(&prev);
    hasher.finalize().into()
}
```

*Scaffold above is suggestive. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies:**
- `cyberos.writer` Python module — already shipped as part of memory Layer 1. The AI Gateway invokes it as a subprocess.
- `cyberos.canonical` Python serialiser — used by the Writer; the AI Gateway side has its own Rust impl that MUST produce byte-identical output.

**Concept dependencies:**
- `AGENTS.md §3, §4, §6` — file ops, atomic write, audit ledger format.
- Memory path schema (`memory.schema.json#/definitions/MemoryPath`).

**Operational dependencies:**
- `python3` ≥ 3.11 on the PATH at runtime (containers ship it; CI ensures it).
- A live `<memory-root>/` (test fixtures use tmpfs).

**Versioning:** the Writer's CLI contract is pinned. If the Writer changes its stdin/stdout shape, this FR must be re-audited. The contract version travels in the memory manifest under `manifest.writer_cli_version`.

---

## §8 — Example payloads

### Caller in TASK-AI-001 (precheck)

```rust
memory_writer::emit(
    canonical::precheck(
        &req.tenant_id,
        &req.agent_persona,
        &req.model_alias,
        &resolved_provider,
        &resolved_model,
        estimated_usd,
        current.spent_usd,
        &req.idempotency_key,
    )
).await.map_err(|e| PrecheckError::MemoryWriterFailed { stderr: e.to_string() })?;
```

### Subprocess stdin (canonical JSON, single line, NL-terminated)

```
{"meta":{"actor":"agent:cyberos-ai-gateway","actor_version":"0.1.0","extra":{"agent_persona":"cuo-cpo@0.4.1","current_spent_usd":47.23,"estimated_usd":0.0085,"idempotency_key":"01HZK9R7A2B4C8D6","model_alias":"chat.smart","resolved_model":"anthropic.claude-3.5-sonnet","resolved_provider":"bedrock","tenant_id":"org:cyberskill"},"kind":"ai.precheck"},"body":"---\nkind: ai.precheck\nactor: agent:cyberos-ai-gateway\ntenant_id: org:cyberskill\n…\n---\n","path":"memories/ai-invocations/1763112131000_org-cyberskill_01HZK9R7A2B4C8D6.md"}
```

### Subprocess stdout (canonical JSON, single line, NL-terminated)

```
{"chain":"a3f9c8d7e6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9","prev_chain":"7f8e9d0c1b2a3940516273849506a7b8c9dae0f12345678987654321fedcba98","seq":18421,"ts_ns":1763112131000000000}
```

### EmittedRow returned to caller

```rust
EmittedRow {
    seq: 18421,
    ts_ns: 1763112131000000000,
    chain: [0xa3, 0xf9, /* … 30 more bytes */ 0xf9],
    path: "memories/ai-invocations/1763112131000_org-cyberskill_01HZK9R7A2B4C8D6.md".to_string(),
}
```

### Subprocess error case (exit code 3, path traversal)

```
stderr: {"code":"path_traversal","detail":"path '../escape.md' resolves outside <memory-root>"}
```

The bridge surfaces this as:

```rust
Err(MemoryWriterError::PathRejected { path: "../escape.md".to_string(), reason: "path_traversal: …".to_string() })
```

---

## §9 — Open questions

All resolved 2026-05-15 (round 2). Promoted to §1 normative clauses:

1. **~~emit_batch() in slice 1~~** → §1 #11 (NOT exposed; callers loop). Was Q1.
2. **~~Chain-hash verification cost~~** → §1 #13 (ON by default; feature flag for dev only). Was Q2.
3. **~~python3 missing at startup~~** → §1 #10 (startup health check). Was Q3.
4. **~~OBS event on ChainHashMismatch~~** → §1 #12 (sev-1 `tracing::error!`). Was Q4.
5. **~~dry_run mode~~** → §1 #15 (NOT in bridge; TASK-AI-021 CLI subcommand). Was Q5.
2. **Chain-hash verification cost** — 5µs per emit, but 16 concurrent emits could compound. Profile in §5 integration test; if it pushes p95 over 50ms, gate behind a feature flag (`memory_writer/verify-chain`) that defaults on in dev and off in prod. Decision: leave on by default, profile, revise if needed.
3. **What if `python3` isn't on the PATH?** — Slice-1 proposal: hard-fail at AI Gateway startup with a clear error message. The runtime container MUST ship Python; the dev environment MUST install it. This is enforced at deploy time, not runtime.
4. **OBS event emission on ChainHashMismatch** — Sev-1 ("chain corruption") is the right severity, but TASK-OBS-003 isn't shipped yet at this FR's build time. Proposal: log a structured `chain_hash_mismatch` event via `tracing::error!`; OBS slice 1 (TASK-OBS-007) will route those to PagerDuty when it lands.
5. **Should the bridge expose a `dry_run` mode?** — Useful for TASK-AI-021 (operator CLI) to validate a payload without emitting. Slice-1 proposal: yes, but as an opt-in flag on `emit_batch`, not on `emit`; keeps the hot path simple.

---

## §10 — Failure modes inventory

| Failure | Detection | HTTP / Return | Recovery |
|---|---|---|---|
| Writer subprocess not on PATH at startup | `check_writer_available()` non-zero exit | Gateway exits 1 with stderr | Operator installs python3; redeploys |
| Writer subprocess hangs (> 5s) | tokio timeout fires | `Err(MemoryWriterError::Timeout)`; SIGTERM then SIGKILL | Caller retries; if persistent, OBS sev-2 alert |
| Writer subprocess non-zero exit | `child.wait()` returns non-success | `Err(WriterFailed { exit_code, stderr })` | Caller bubbles up; TASK-AI-021 `cyberos-ai memory doctor` diagnoses |
| Path traversal in caller-supplied path | `validate_path` regex check | `Err(PathRejected)` BEFORE spawn | Caller fixes; this is a programmer error |
| Reserved path (`audit/`, `index/`, `HEAD`, `.lock`) | `validate_path` substring check | `Err(PathRejected)` BEFORE spawn | Same as above |
| Canonical-JSON serialisation failure | `canonical::serialise` returns Err | `Err(CanonicalisationFailed)` BEFORE spawn | Caller fixes payload (most likely non-NFC UTF-8) |
| Chain-hash mismatch | Local recomputation diverges from Writer stdout | `Err(ChainHashMismatch)` + sev-1 log | Sev-1 OBS event; investigate Writer; do NOT trust the row |
| `python3` missing at runtime | Subprocess spawn errors `ENOENT` | `Err(WriterUnreachable)` | Gateway should have caught at startup (§1 #10); if not, sev-1 |
| Concurrent .lock contention | Writer subprocess waits on lock; latency increases | Eventually succeeds | Normal; no action needed unless p99 exceeds budget |
| Writer subprocess killed mid-write | child exits non-zero with partial row on disk | `Err(WriterFailed)` | Writer's own recovery handles partial row; bridge surfaces the error |
| OOM during subprocess | child SIGKILL'd by kernel | `Err(WriterFailed { exit_code: -9 })` | OBS sev-2; capacity review |

---

## §11 — Notes

- This FR is the single biggest latency contributor on the AI Gateway hot path (50ms of the gateway's ~200ms p95 budget). PyO3 (TASK-AI-008) could cut that to <10ms; we defer the optimisation until real traffic confirms the win.
- The Writer subprocess is the only path that holds `<memory-root>/.lock`. The AI Gateway never touches the lock directly — that's the rule from `AGENTS.md §14.1` made into code.
- The five row kinds defined here (`ai.precheck`, `ai.invocation`, `ai.invocation_failed`, `ai.hold_expired`, `ai.persona_loaded`) are the closed initial set for slice 1. Subsequent FRs extend the closed set explicitly via §1 #8: `ai.failover_triggered` (TASK-AI-009), `ai.zdr_violation` (TASK-AI-015), `ai.residency_violation` (TASK-AI-016), `ai.cache_hit` (TASK-AI-017), and the `ai.cli_*` family (TASK-AI-021). Every new kind needs the same minimal PR shape — enum variant + typed builder + unit test + update to §1 #8.
- The `actor` field is `agent:cyberos-ai-gateway` (not a human subject id). This matches `AGENTS.md` §11's distinction between trusted and untrusted authors: only the canonical Writer can write to the chain, but the *originating actor* on the row is the AI Gateway service.
- This FR is the first non-memory module to depend on the canonical Writer subprocess. Once it lands, the pattern can be lifted into other modules (CHAT, CUO, EMAIL) by duplicating the Rust module into each service crate. A later refactor will extract it into a shared `cyberos-memory-client` crate.

---

*End of TASK-AI-003. Run `task-audit` next: `cargo run -p cyberos-skill-cli -- run task-audit --input '{"fr_path": "docs/tasks/ai/TASK-AI-003-memory-audit-bridge/spec.md"}'`*
