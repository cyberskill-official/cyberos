---
id: FR-SKILL-106
title: "memory-sync@1 skill bundle — operator-facing sync trigger that defers to Stage 4 orchestrator (slice-3 stub; full sync ships P2)"
module: SKILL
priority: SHOULD
status: done
verify: I
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-104, FR-SKILL-105, FR-MEMORY-103]
depends_on: [FR-SKILL-105]
blocks: [FR-SKILL-107]

source_pages:
  - website/docs/skills/memory-sync.html
source_decisions:
  - DEC-400 (slice-3 ships skill scaffold + stub broker call; full sync orchestration is FR-MEMORY-103 + P2 Stage 4)
  - DEC-401 (skill emits memory audit 'memory.sync_requested' on every invoke; never silent)

language: rust 1.81
service: cyberos/skills/memory-sync/
new_files:
  - skills/memory-sync/SKILL.md
  - skills/memory-sync/main.rs
  - skills/memory-sync/src/lib.rs
  - skills/memory-sync/tests/sync_test.rs
modified_files:
  - cyberos/Cargo.toml                               # workspace member
allowed_tools:
  - file_read: skills/memory-sync/**
  - file_write: skills/memory-sync/**
  - bash: cd skills/memory-sync && cargo test
disallowed_tools:
  - implement sync logic in this skill (per DEC-400 — FR-MEMORY-103 owns)
  - skip the audit row even in slice-3 stub (per DEC-401)

effort_hours: 4
sub_tasks:
  - "0.5h: SKILL.md frontmatter (allowed_tools=[MemoryEmit])"
  - "0.5h: main.rs + Cargo.toml broker subprocess entrypoint"
  - "1.0h: lib.rs — request_sync(direction, dry_run) -> SyncOutcome stub returning 'deferred to P2'"
  - "0.5h: memory audit row 'memory.sync_requested'"
  - "0.5h: CLI bash wrapper cyberos-memory-sync"
  - "1.0h: sync_test.rs — stub returns DeferredToP2; audit row emitted"
risk_if_skipped: "Without the scaffold, operators have no entry-point to learn the sync UX. The skill registers the surface; logic ships separately. Without DEC-401 audit, even stub invocations are invisible."
---

## §1 — Description (BCP-14 normative)

The `memory-sync@1` skill bundle **MUST** scaffold the sync-trigger surface; full orchestration deferred to FR-MEMORY-103 + P2. The contract:

1. **MUST** ship a signed bundle with SKILL.md frontmatter `allowed_tools: [MemoryEmit]`; sync_class `private`; tenant_scope `pinned`.
2. **MUST** expose Rust API `request_sync(direction: SyncDirection, dry_run: bool) -> Result<SyncOutcome, SyncError>` where SyncDirection ∈ `Push | Pull | Both`.
3. **MUST** in slice-3 return `SyncOutcome::DeferredToP2 { reason: "full orchestration ships in P2 via FR-MEMORY-103" }` regardless of arguments.
4. **MUST** emit memory audit row `memory.sync_requested` per invocation with payload `{direction, dry_run, by_subject_id, slice_version, trace_id}`.
5. **MUST** be invokable via:
    - Rust: `cyberos_memory_sync::request_sync(SyncDirection::Both, false).await`
    - bash: `cyberos-memory-sync push --dry-run`
6. **MUST** exit with `ExitCode::Ok` (slice-3) even though no actual sync occurs; bash CLI prints `"⚠  memory-sync@1 slice-3: deferred to P2 — see FR-MEMORY-103"`.
7. **MUST** emit OTel metric `skill_memory_sync_requests_total{direction, dry_run, outcome}` (counter; outcome ∈ deferred_p2 | error).
8. **SHOULD** when P2 ships, this FR's slice-4 version will delegate to FR-MEMORY-103's sync daemon via Unix socket call.

---

## §2 — Why this design (rationale for humans)

**Why ship a stub (DEC-400)?** Reserves the skill ID (`memory-sync`) in the OCI registry; downstream skills can declare `depends_on: memory-sync@^1`. Slice-3 users learn the UX; full logic lands without breaking their muscle memory.

**Why audit stub invocations (DEC-401)?** Tells the operator "the skill was called but didn't do anything yet" — visible signal that drives P2 prioritisation. Also surfaces accidental-test invocations.

**Why DeferredToP2 not Error?** Error implies bug; Deferred implies known-limitation. Caller code can pattern-match cleanly without try/catch ceremony.

---

## §3 — API contract

### SKILL.md

```markdown
---
id: memory-sync
version: 1.0.0
description: memory multi-device sync trigger (slice-3 stub; full orchestration in P2 via FR-MEMORY-103).
allowed_memory_scopes: []
allowed_tools: [MemoryEmit]
sync_class: private
tenant_scope: pinned
effort_minutes: 1
tags: [memory, sync, p2-pending]
signature:
  algo: ed25519
  public_key_hex: "<release-populated>"
  signature_hex:  "<release-populated>"
---

# memory-sync@1 (slice-3 stub)

```rust
use cyberos_memory_sync::{request_sync, SyncDirection};
let outcome = request_sync(SyncDirection::Both, false).await?;
// outcome = SyncOutcome::DeferredToP2 { reason: "..." }
```
```

### Rust API

```rust
// skills/memory-sync/src/lib.rs
use serde::Serialize;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection { Push, Pull, Both }

#[derive(Clone, Debug, Serialize)]
pub enum SyncOutcome {
    DeferredToP2 { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("broker down")] BrokerDown,
}

pub async fn request_sync(direction: SyncDirection, dry_run: bool) -> Result<SyncOutcome, SyncError> {
    let trace_id = current_trace_id();
    emit_memory_row("memory.sync_requested", serde_json::json!({
        "direction": direction,
        "dry_run": dry_run,
        "slice_version": "slice-3-stub",
        "trace_id": trace_id,
    })).await;
    metrics::counter!("skill_memory_sync_requests_total",
        "direction" => format!("{direction:?}"),
        "dry_run" => dry_run.to_string(),
        "outcome" => "deferred_p2").increment(1);
    Ok(SyncOutcome::DeferredToP2 {
        reason: "full sync orchestration ships in P2 via FR-MEMORY-103".into(),
    })
}
```

### Bash CLI

```bash
#!/usr/bin/env bash
# skills/memory-sync/cli/cyberos-memory-sync
set -euo pipefail
DIRECTION="${1:-both}"   # push | pull | both
DRY_RUN=""
if [[ "${2:-}" == "--dry-run" ]]; then DRY_RUN="--dry-run"; fi
echo "⚠  memory-sync@1 slice-3 — request acknowledged; full orchestration ships in P2 (FR-MEMORY-103)"
exec cyberos-memory-sync-main --direction "$DIRECTION" $DRY_RUN
```

---

## §4 — Acceptance criteria

1. **request_sync returns DeferredToP2** — any direction + dry_run combo → DeferredToP2 outcome.
2. **memory audit row emitted** — `memory.sync_requested` row with `slice_version: "slice-3-stub"`.
3. **OTel counter increments** — `skill_memory_sync_requests_total{outcome="deferred_p2"}` after invoke.
4. **Bash CLI prints warning** — `cyberos-memory-sync` → stderr contains "slice-3" + "FR-MEMORY-103".
5. **SKILL.md validates** — `cyberos skill validate skills/memory-sync/` → exit 0.
6. **Signature verified** — release sign-bundle → cyberos skill verify passes.
7. **Bundle published** — OCI tag `oci://registry.cyberos.world/skills/memory-sync:1.0.0`.
8. **Broker enforces** — skill attempts non-allowed tool → broker denial.

---

## §5 — Verification

```rust
#[tokio::test]
async fn returns_deferred() {
    let outcome = request_sync(SyncDirection::Both, false).await.unwrap();
    assert!(matches!(outcome, SyncOutcome::DeferredToP2 { .. }));
}

#[tokio::test]
async fn audit_emitted() {
    let _ = request_sync(SyncDirection::Push, true).await.unwrap();
    let row = memory_test_helper::latest("memory.sync_requested").await;
    assert_eq!(row["payload"]["direction"], "push");
    assert_eq!(row["payload"]["dry_run"], true);
    assert_eq!(row["payload"]["slice_version"], "slice-3-stub");
}
```

---

## §6 — Implementation skeleton

(API above is the skeleton.)

---

## §7 — Dependencies

- **FR-SKILL-103/104/105** — frontmatter/broker/SDK pattern.
- **FR-MEMORY-103 (downstream sync owner)** — full sync logic ships here in P2.

---

## §8 — Example payloads

```json
{
  "kind": "memory.sync_requested",
  "payload": {
    "direction": "both",
    "dry_run": false,
    "by_subject_id": "7e57c0de-...",
    "slice_version": "slice-3-stub",
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Slice-4: delegate to FR-MEMORY-103 Unix socket for real sync.
- Per-tenant scheduled sync (cron) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Broker down | UnixStream Err | SyncError::BrokerDown | Operator restores |
| Audit emit fails | sev-2 alarm | None (stub completes regardless) | Operator restores memory |
| OTel exporter down | metric buffered | Logged | Restore FR-OBS-001 |
| Bash CLI misuse (typo direction) | clap reject | exit 1 | User fixes |
| Concurrent invocations | independent | Each gets own audit row | None |

---

## §11 — Implementation notes

- The stub deliberately does no work — it's a registration vehicle for the OCI tag + the canonical UX.
- The `slice_version` field in audit payload is a string that will flip to `"slice-4-active"` when real sync ships; operators querying memory can pivot on it.
- The `memory.sync_requested` kind is intentionally distinct from `memory.sync_*` rows FR-MEMORY-103 will emit; this is the *operator request*, not the daemon's *action*.

---

*End of FR-SKILL-106.*
