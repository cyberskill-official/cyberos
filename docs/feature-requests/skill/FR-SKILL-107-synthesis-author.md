---
id: FR-SKILL-107
title: "synthesis-author@1 skill — nightly multi-memory auto-evolve composes derived memories from clustered raw captures (P3 — stub scaffold in P1)"
module: SKILL
priority: COULD
status: ready_to_implement
verify: I
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-104, FR-SKILL-105, FR-SKILL-106, FR-MEMORY-108]
depends_on: [FR-SKILL-106]
blocks: [FR-TEN-005]

source_pages:
  - website/docs/skills/synthesis-author.html
source_decisions:
  - DEC-410 (synthesis-author is P3 — depends on multi-memory merging + LLM compose chains)
  - DEC-411 (P1 ships scaffold + audit row; nightly compose deferred to P3)

language: rust 1.81
service: cyberos/skills/synthesis-author/
new_files:
  - skills/synthesis-author/SKILL.md
  - skills/synthesis-author/main.rs
  - skills/synthesis-author/src/lib.rs
  - skills/synthesis-author/tests/synthesis_test.rs
modified_files:
  - cyberos/Cargo.toml
allowed_tools:
  - file_read: skills/synthesis-author/**
  - file_write: skills/synthesis-author/**
  - bash: cd skills/synthesis-author && cargo test
disallowed_tools:
  - run synthesis logic in P1 (per DEC-411)
  - call external LLM from this scaffold (compose chains ship P3)

effort_hours: 3
sub_tasks:
  - "0.5h: SKILL.md (allowed_tools=[MemoryEmit, MemoryRead, MemorySearch])"
  - "0.5h: main.rs + Cargo.toml broker subprocess entrypoint"
  - "1.0h: lib.rs — request_synthesis() stub returning DeferredToP3"
  - "0.5h: memory audit 'memory.synthesis_requested'"
  - "0.5h: synthesis_test.rs — stub returns DeferredToP3"
risk_if_skipped: "Reserves the skill ID. Without scaffold, P3 launch invents the surface from scratch."
---

## §1 — Description (BCP-14 normative)

The `synthesis-author@1` skill **MUST** scaffold the nightly multi-memory synthesis surface; full logic ships P3. The contract:

1. **MUST** ship signed bundle with SKILL.md frontmatter `allowed_tools: [MemoryEmit, MemoryRead, MemorySearch]`; sync_class `shareable` (synthesised memories may sync); tenant_scope `any`.
2. **MUST** expose Rust API `request_synthesis(scope: SynthesisScope, dry_run: bool) -> Result<SynthesisOutcome, SynthesisError>` where SynthesisScope ∈ `Tenant | Engagement | Custom { paths: Vec<String> }`.
3. **MUST** in P1 return `SynthesisOutcome::DeferredToP3 { reason }` regardless of arguments.
4. **MUST** emit memory audit `memory.synthesis_requested` per invocation.
5. **MUST** be invokable via Rust SDK + bash CLI `cyberos-synthesis-author run --scope tenant --dry-run`.
6. **MUST** exit Ok in P1.
7. **MUST** emit OTel `skill_synthesis_requests_total{scope, outcome}`.
8. **SHOULD** P3 slice-1 implementation:
    - Cluster captures via BGE-M3 embeddings (FR-AI-019).
    - Compose summary memory via COO persona (FR-AI-014).
    - Emit as `kind: synthesised, sync_class: shareable` memory.

---

## §2 — Why this design

Same reasoning as FR-SKILL-106 stub: reserve ID, capture UX, audit invocations, defer logic to module-owner FR (here: the eventual P3 synthesis pipeline). Synthesis is high-leverage but expensive (LLM compose chains); shipping the stub now lets early adopters experiment with the surface API without compute cost.

---

## §3 — API contract

```rust
// skills/synthesis-author/src/lib.rs
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisScope {
    Tenant,
    Engagement { engagement_id: uuid::Uuid },
    Custom { paths: Vec<String> },
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum SynthesisOutcome {
    DeferredToP3 { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SynthesisError {
    #[error("broker down")] BrokerDown,
}

pub async fn request_synthesis(scope: SynthesisScope, dry_run: bool) -> Result<SynthesisOutcome, SynthesisError> {
    emit_memory_row("memory.synthesis_requested", serde_json::json!({
        "scope": scope, "dry_run": dry_run,
        "slice_version": "p1-stub", "trace_id": current_trace_id(),
    })).await;
    metrics::counter!("skill_synthesis_requests_total",
        "scope" => format!("{scope:?}"), "outcome" => "deferred_p3").increment(1);
    Ok(SynthesisOutcome::DeferredToP3 {
        reason: "Synthesis ships in P3; this is the scaffold reservation.".into(),
    })
}
```

---

## §4 — Acceptance criteria

1. **DeferredToP3 returned** regardless of args.
2. **memory audit emitted** with `slice_version: "p1-stub"`.
3. **OTel counter increments**.
4. **CLI prints P3-deferral warning**.
5. **SKILL.md validates**.
6. **Signature verified at release**.
7. **Broker enforces narrow allowed_tools**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn returns_deferred() {
    let outcome = request_synthesis(SynthesisScope::Tenant, false).await.unwrap();
    assert!(matches!(outcome, SynthesisOutcome::DeferredToP3 { .. }));
}

#[tokio::test]
async fn audit_emitted() {
    let _ = request_synthesis(SynthesisScope::Tenant, true).await.unwrap();
    let row = memory_test_helper::latest("memory.synthesis_requested").await;
    assert_eq!(row["payload"]["slice_version"], "p1-stub");
}
```

---

## §6 — Implementation skeleton

(API above.)

---

## §7 — Dependencies

- **FR-SKILL-103/104/105** — frontmatter/broker/SDK pattern.
- **FR-SKILL-106** — sibling stub pattern.
- **FR-MEMORY-108** — search (used in P3 implementation).
- **FR-AI-014, FR-AI-019** — persona + embeddings (used in P3).

---

## §8 — Example payloads

```json
{
  "kind": "memory.synthesis_requested",
  "payload": {
    "scope": "tenant",
    "dry_run": true,
    "slice_version": "p1-stub",
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred to P3: cluster algorithm choice, LLM cost budget, sync_class for synthesised memories.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Broker down | UnixStream Err | SynthesisError::BrokerDown | Operator restores |
| Audit emit fails | sev-2 | Stub still returns Ok | Operator restores memory |
| Invalid scope | serde Err | 422 | Caller fixes |

---

## §11 — Implementation notes

- Same stub pattern as FR-SKILL-106; reserves OCI tag for P3 launch.
- `slice_version: "p1-stub"` flips to `"p3-active"` in P3 implementation.
- COULD priority — may be deprioritised at P3 planning if other features outweigh.

---

*End of FR-SKILL-107.*
