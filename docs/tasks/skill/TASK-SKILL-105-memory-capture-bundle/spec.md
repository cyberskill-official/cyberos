---
id: TASK-SKILL-105
title: "memory-capture@1 skill bundle — canonical SDK-style entry point for emitting memory capture rows from tools, scripts, and external integrations"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: skill
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-101, TASK-SKILL-102, TASK-SKILL-103, TASK-SKILL-104, TASK-SKILL-106, TASK-MEMORY-101, TASK-MEMORY-106, TASK-MEMORY-107, TASK-MEMORY-109, TASK-MEMORY-111]
depends_on: [TASK-SKILL-103, TASK-SKILL-104]
blocks: [TASK-SKILL-106]

source_pages:
  - website/docs/skills/memory-capture.html
  - website/docs/runbooks/memory-capture-skill-runbook.html
source_decisions:
  - DEC-200 (memory-capture@1 is THE canonical capture entry; every emit goes through it)
  - DEC-201 (skill bundle MUST be signed via TASK-SKILL-102 OCI signing path)
  - DEC-202 (bundle ships a Rust SDK + Python wrapper + bash CLI in one .skill artifact)
  - DEC-203 (idempotency via content-hash dedup at SDK level before broker call)

language: rust 1.81 + python 3.11 + bash
service: cyberos/skills/memory-capture/
new_files:
  - skills/memory-capture/SKILL.md
  # skill binary (broker subprocess entrypoint)
  - skills/memory-capture/main.rs
  - skills/memory-capture/Cargo.toml
  # public Rust SDK
  - skills/memory-capture/src/lib.rs
  - skills/memory-capture/src/emit.rs
  - skills/memory-capture/src/dedup.rs
  - skills/memory-capture/sdk-python/cyberos_memory_capture/__init__.py
  - skills/memory-capture/sdk-python/setup.py
  # bash wrapper for one-shot CLI use
  - skills/memory-capture/cli/cyberos-memory-capture
  - skills/memory-capture/tests/sdk_rust_test.rs
  - skills/memory-capture/tests/sdk_python_test.py
  - skills/memory-capture/tests/cli_e2e_test.sh
  # uses TASK-SKILL-102 signing infra
  - skills/memory-capture/scripts/sign-bundle.sh
modified_files:
  # workspace member for SDK crate
  - cyberos/Cargo.toml
allowed_tools:
  - file_read: skills/memory-capture/**, services/memory/**
  - file_write: skills/memory-capture/**
  - bash: cd skills/memory-capture && cargo test
  - bash: cd skills/memory-capture && python3 -m pytest sdk-python
disallowed_tools:
  - call memory_writer directly from SDK (per DEC-200 — must go through broker per TASK-SKILL-104)
  - skip dedup before broker call (per DEC-203 — reduces broker pressure 10× under churn)
  - emit unsigned bundle to OCI registry (per DEC-201)

effort_hours: 9
subtasks:
  - "0.5h: SKILL.md frontmatter (id=memory-capture, version=1.0.0, allowed_tools=[MemoryEmit], allowed_memory_scopes=memories/**)"
  - "0.5h: Cargo.toml + main.rs (subprocess entrypoint that connects to broker socket and exposes `cb capture` API)"
  - "1.0h: src/lib.rs — public Rust SDK with `MemoryCapture::new().emit(kind, payload, scope).await`"
  - "1.0h: src/emit.rs — bridge to broker via JSON-RPC; PII-scrub via TASK-MEMORY-111 before send"
  - "1.0h: src/dedup.rs — in-memory LRU(1000) content-hash dedup; identical emits within 60s deduplicated"
  - "1.0h: sdk-python — pyo3 wrapper OR subprocess shim (decision: subprocess for simpler distribution)"
  - "0.5h: cli/cyberos-memory-capture — one-shot bash wrapper: `echo '{...}' | cyberos-memory-capture --kind notes.captured`"
  - "1.0h: sign-bundle.sh — calls TASK-SKILL-102 signing pipeline; outputs .skill artifact"
  - "1.5h: sdk_rust_test.rs — happy + dedup + error propagation"
  - "1.0h: sdk_python_test.py — Python SDK e2e"
  - "0.5h: cli_e2e_test.sh — pipe-stdin + flag-arg + exit codes"
risk_if_skipped: "Without a canonical capture entry, every tool that wants to emit a memory row reinvents the wheel: connection management, dedup, PII scrub, signature, error handling. Different tools end up with subtly different audit-row shapes — operators querying memory can't filter consistently. The skill is the contract: 'if you import memory-capture@1, your rows look right by construction.' Without the multi-language SDK, only Rust callers can emit; the Python/bash escape hatch is needed for ad-hoc scripts. Without dedup, the same idempotent operation (e.g. nightly cron emitting status) creates thousands of identical rows."
---

## §1 — Description (BCP-14 normative)

The `memory-capture@1` skill bundle **MUST** be the canonical, signed, OCI-distributed SDK for emitting memory capture rows. The bundle:

1. **MUST** ship as a single `.skill` artifact containing: (a) the Rust skill binary `main` (broker subprocess entrypoint), (b) a Rust SDK crate (`cyberos-memory-capture`), (c) a Python wrapper (`cyberos_memory_capture`), (d) a bash CLI (`cyberos-memory-capture`). All three language frontends route through the same Rust core.
2. **MUST** carry a valid SKILL.md frontmatter per TASK-SKILL-103 schema v1:
- `id: memory-capture`, `version: 1.0.0`, `description: "Canonical entry point for emitting memory capture rows."`
- `allowed_tools: [MemoryEmit]` (only — never Bash, Read, HttpFetch).
- `allowed_memory_scopes: ["memories/**"]` (broad emit scope; finer-grained scoping is the caller's frontmatter responsibility).
- `sync_class: shareable` (rows emitted by this skill MAY sync to Cloud memory per TASK-MEMORY-106).
- `signature` populated via TASK-SKILL-102 signing pipeline.
3. **MUST** expose the following stable API in the Rust SDK:
    ```rust
    pub struct MemoryCapture { /* opaque */ }

    impl MemoryCapture {
        pub async fn new() -> Result<Self, CaptureError>;
        pub async fn emit(&self, kind: &str, payload: serde_json::Value, scope: EmitScope) -> Result<EmitOutcome, CaptureError>;
    }

    pub struct EmitOutcome { pub row_id: String, pub seq: u64, pub trace_id: String, pub deduped: bool }

    pub enum EmitScope {
        Private,                                  // never sync
        Shareable { acl: Vec<String> },           // sync per AGENTS.md §15
    }

    pub enum CaptureError { BrokerDown, Denied(String), PiiScrubFailed, Timeout, InvalidKind, InvalidPayload }
    ```
4. **MUST** expose the equivalent Python API:
    ```python
    from cyberos_memory_capture import MemoryCapture, EmitScope, CaptureError

    cb = MemoryCapture()
    outcome = cb.emit(
        kind="notes.captured",
        payload={"title": "Auth design", "body": "..."},
        scope=EmitScope.shareable(acl=["@alice", "@bob"]),
    )
    print(outcome.row_id, outcome.deduped)
    ```
5. **MUST** expose the bash CLI:
    ```text
    $ echo '{"title":"x","body":"y"}' | cyberos-memory-capture --kind notes.captured --scope private
    {"row_id":"...","seq":42,"trace_id":"0af7...","deduped":false}
    ```
6. **MUST** validate `kind` against the canonical kind regex `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$` (e.g. `notes.captured`, `tool.invoked`). Invalid kind → `CaptureError::InvalidKind`.
7. **MUST** dedup identical emits at SDK level before broker call:
- Compute `content_hash = blake3(canonical_json(kind, payload, scope))`.
- Check in-memory LRU (size 1000, TTL 60s).
- On hit → return `EmitOutcome { deduped: true, ... }` with the prior outcome's row_id; no broker call.
- On miss → call broker; cache result.
8. **MUST** PII-scrub payload via TASK-MEMORY-111's ruleset BEFORE broker call. Scrub failure → `CaptureError::PiiScrubFailed`; caller can retry with `payload_sanitized = true` flag (telling SDK that caller has already sanitised and we trust the input).
9. **MUST** auto-discover the broker socket from `CYBEROS_BROKER_SOCKET` env var (set by TASK-SKILL-104 when subprocess is launched). Outside a broker invocation (e.g. ad-hoc CLI use), the SDK connects to the default broker socket `/run/cyberos/broker.sock` (system-scope) or `~/.cyberos/broker.sock` (user-scope).
10. **MUST** handle broker-down with exponential backoff: 3 retries at 100ms, 500ms, 2s. If all fail → `CaptureError::BrokerDown`; SDK returns immediately (does not block caller indefinitely).
11. **MUST** propagate W3C TraceContext: read `TRACEPARENT` env var (set by upstream) or generate a new trace_id per `MemoryCapture::new()` call. Every emit carries this trace_id.
12. **MUST** emit OTel span `skill.memory_capture.emit` per call with attributes `kind`, `dedup_hit`, `payload_size_bytes`, `pii_scrub_match_count`, `duration_ms`.
13. **MUST** emit OTel metrics:
- `skill_memory_capture_emits_total{kind, outcome}` (counter; outcome ∈ emitted | deduped | denied | broker_down | pii_failed).
- `skill_memory_capture_dedup_hit_ratio` (gauge).
14. **MUST** be signed via TASK-SKILL-102 release pipeline; the OCI tag is `oci://registry.cyberos.world/skills/memory-capture:1.0.0`; immutable.
15. **SHOULD** provide a TypeScript SDK in slice 3+ (placeholder; web/node integrations).

---

## §2 — Why this design (rationale for humans)

**Why a skill at all (§1 #1)?** Three motives. First, ENFORCEMENT: routing emit through the TASK-SKILL-104 capability broker ensures every memory write is authorised, audited, and PII-scrubbed (no path that bypasses these). Second, IDEMPOTENCY: the SDK's dedup layer (§1 #7) cuts duplicate emit load by ~10× under realistic churn. Third, DISTRIBUTION: a signed OCI artifact gives operators a single answer to "how do I emit a memory?" — `cargo add cyberos-memory-capture` or `pip install cyberos-memory-capture`.

**Why three language frontends (§1 #1)?** Rust SDK is the canonical caller (other services). Python wrapper is for data-science scripts (operators routinely have Jupyter notebooks). Bash CLI is for shell scripts + ops one-liners. All three route through the same Rust core for consistency.

**Why allowed_tools = [MemoryEmit] only (§1 #2)?** Defense-in-depth. The skill has zero need for Bash/Read/HttpFetch; granting them would expand attack surface. The capability broker enforces this — if a future SKILL.md PR adds Bash by mistake, the broker rejects every Bash call at invoke time.

**Why kind regex (§1 #6)?** A namespaced kind (`module.event_type`) lets operators query "all rows from the notes module" via prefix match. Without regex enforcement, kinds drift into `notes-captured` vs `notes.captured` vs `notes_captured` chaos. The regex locks the convention; TASK-AI-003's closed-set list is the authoritative registry.

**Why dedup TTL 60s (§1 #7)?** Empirical: cron-style emitters (status pings, watchdog heartbeats) repeat at minute-scale; same-minute repeats are duplicates. Repeats > 1 minute apart are probably meaningful (system state changed). 60s splits the population well; LRU(1000) covers ~16 unique kind/payload combos per minute (more than enough for a typical service).

**Why PII-scrub before broker call (§1 #8)?** The broker also runs TASK-MEMORY-111's scan (defense-in-depth) but pre-scrubbing means: (a) payload size is smaller on the wire, (b) broker rejects only true PII slips (not "every emit needs scrubbing"). The `payload_sanitized = true` flag lets explicit callers skip the SDK-level scrub — useful when the payload is already known-clean (e.g. generated structured data, not user text).

**Why exp backoff on broker-down (§1 #10)?** A transient broker restart shouldn't crash callers. 3 retries × spaced-100ms-500ms-2s gives the broker time to recover before we surrender. Total wait < 3 seconds = bounded caller delay.

**Why W3C trace propagation (§1 #11)?** Traces let operators correlate "user prompt → tool call → memory row" across service boundaries. Without propagation, the memory row appears orphaned; with it, OBS dashboards show the full chain.

**Why TypeScript SDK deferred (§1 #15)?** Slice 1+2 has no node/web callers. Adding TS now is YAGNI. When portal (slice 3+) needs to emit from browser → backend, TS SDK lands.

---

## §3 — API contract

### SKILL.md

```markdown
---
id: memory-capture
version: 1.0.0
description: Canonical entry point for emitting memory capture rows.
allowed_memory_scopes:
  - memories/**
allowed_tools:
  - MemoryEmit
sync_class: shareable
tenant_scope: any
effort_minutes: 5
tags: [memory, capture, foundation, sdk]
signature:
  algo: ed25519
  public_key_hex: "<release-time-populated>"
  signature_hex:  "<release-time-populated>"
---

# memory-capture@1

The canonical entry point for emitting memory capture rows.

## Quick start (Rust)

```rust
use cyberos_memory_capture::{MemoryCapture, EmitScope};

let cb = MemoryCapture::new().await?; let outcome = cb.emit(
    "notes.captured",
    serde_json::json!({ "title": "Auth design", "body": "..." }),
    EmitScope::Shareable { acl: vec!["@alice".into()] },
).await?; println!("emitted row {} (deduped={})", outcome.row_id, outcome.deduped);
```

## Quick start (Python)

```python
from cyberos_memory_capture import MemoryCapture, EmitScope cb = MemoryCapture() outcome = cb.emit(kind="notes.captured", payload={"title": "x"}, scope=EmitScope.private())
```

## Quick start (bash)

```bash
echo '{"title":"x"}' | cyberos-memory-capture --kind notes.captured --scope private
```
```

### Rust SDK

```rust
// skills/memory-capture/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("broker socket unreachable after 3 retries")] BrokerDown,
    #[error("broker denied: {0}")]                        Denied(String),
    #[error("PII scrub failed; retry with payload_sanitized=true if known-clean")] PiiScrubFailed,
    #[error("broker call timed out (> effort_minutes deadline)")] Timeout,
    #[error("invalid kind (must match ^[a-z][a-z0-9_]*\\.[a-z][a-z0-9_]*$): {0}")] InvalidKind(String),
    #[error("invalid payload: {0}")]                      InvalidPayload(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmitScope {
    Private,
    Shareable { acl: Vec<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmitOutcome {
    pub row_id:   String,
    pub seq:      u64,
    pub trace_id: String,
    pub deduped:  bool,
}

pub struct MemoryCapture {
    socket_path: std::path::PathBuf,
    trace_id:    String,
    dedup:       lru::LruCache<[u8; 32], EmitOutcome>,
}

impl MemoryCapture {
    pub async fn new() -> Result<Self, CaptureError> {
        let socket_path = std::env::var("CYBEROS_BROKER_SOCKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| default_broker_socket());
        let trace_id = std::env::var("TRACEPARENT")
            .ok()
            .and_then(parse_traceparent_for_trace_id)
            .unwrap_or_else(generate_trace_id);
        Ok(Self {
            socket_path,
            trace_id,
            dedup: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
        })
    }

    pub async fn emit(&mut self, kind: &str, payload: serde_json::Value, scope: EmitScope)
        -> Result<EmitOutcome, CaptureError>
    {
        // §1 #6: kind regex
        if !is_valid_kind(kind) { return Err(CaptureError::InvalidKind(kind.into())); }

        // §1 #7: dedup
        let key = compute_dedup_key(kind, &payload, &scope);
        if let Some(prior) = self.dedup.get(&key) {
            metrics::counter!("skill_memory_capture_emits_total", "kind" => kind.to_owned(), "outcome" => "deduped").increment(1);
            return Ok(EmitOutcome { deduped: true, ..prior.clone() });
        }

        // §1 #8: PII scrub (calls broker's MemoryEmit which itself runs TASK-MEMORY-111;
        // but we ALSO pre-scrub here to reduce broker work)
        let scrubbed = pre_scrub_payload(&payload).map_err(|_| CaptureError::PiiScrubFailed)?;

        // §1 #9 #10: broker call with retry
        let outcome = self.call_broker_with_retry(kind, scrubbed, &scope).await?;

        // Cache for future dedup
        self.dedup.put(key, outcome.clone());
        metrics::counter!("skill_memory_capture_emits_total", "kind" => kind.to_owned(), "outcome" => "emitted").increment(1);
        Ok(outcome)
    }

    async fn call_broker_with_retry(&self, kind: &str, payload: serde_json::Value, scope: &EmitScope)
        -> Result<EmitOutcome, CaptureError>
    {
        let delays = [std::time::Duration::from_millis(100),
                      std::time::Duration::from_millis(500),
                      std::time::Duration::from_secs(2)];
        for (i, delay) in delays.iter().enumerate() {
            match call_broker_once(&self.socket_path, kind, &payload, scope, &self.trace_id).await {
                Ok(o) => return Ok(o),
                Err(CaptureError::BrokerDown) if i + 1 < delays.len() => {
                    tokio::time::sleep(*delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(CaptureError::BrokerDown)
    }
}

fn is_valid_kind(kind: &str) -> bool {
    static RX: once_cell::sync::Lazy<regex::Regex> =
        once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$").unwrap());
    RX.is_match(kind)
}

fn compute_dedup_key(kind: &str, payload: &serde_json::Value, scope: &EmitScope) -> [u8; 32] {
    let canonical = serde_json::to_string(&serde_json::json!({
        "kind": kind, "payload": payload, "scope": scope,
    })).unwrap();
    *blake3::hash(canonical.as_bytes()).as_bytes()
}
```

### Python wrapper (subprocess shim)

```python
# skills/memory-capture/sdk-python/cyberos_memory_capture/__init__.py
import json
import subprocess
import os
from dataclasses import dataclass
from typing import List, Optional, Union

class CaptureError(Exception): pass
class BrokerDown(CaptureError): pass
class Denied(CaptureError): pass
class PiiScrubFailed(CaptureError): pass

@dataclass
class EmitScope:
    private: bool
    acl: Optional[List[str]] = None
    @classmethod
    def private_(cls):                           return cls(private=True)
    @classmethod
    def shareable(cls, acl: List[str] = None):  return cls(private=False, acl=acl or [])
    def to_dict(self):
        if self.private: return "Private"
        return {"Shareable": {"acl": self.acl or []}}

@dataclass
class EmitOutcome:
    row_id: str
    seq: int
    trace_id: str
    deduped: bool

class MemoryCapture:
    def __init__(self):
        self.binary = os.environ.get("CYBEROS_MEMORY_CAPTURE_BIN", "cyberos-memory-capture")

    def emit(self, kind: str, payload: dict, scope: EmitScope) -> EmitOutcome:
        req = {"kind": kind, "payload": payload, "scope": scope.to_dict()}
        proc = subprocess.run(
            [self.binary, "--json"],
            input=json.dumps(req).encode(),
            capture_output=True,
            timeout=10,
        )
        if proc.returncode != 0:
            err = proc.stderr.decode("utf-8", errors="replace")
            if "broker_down" in err:        raise BrokerDown(err)
            if "denied" in err:             raise Denied(err)
            if "pii_scrub_failed" in err:   raise PiiScrubFailed(err)
            raise CaptureError(err)
        resp = json.loads(proc.stdout)
        return EmitOutcome(**resp)
```

### Bash CLI wrapper

```bash
#!/usr/bin/env bash
# skills/memory-capture/cli/cyberos-memory-capture
# Reads JSON payload from stdin OR positional arg; prints EmitOutcome JSON to stdout.
set -euo pipefail

KIND=""
SCOPE="private"
ACL=()
JSON=false
SANITISED=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --kind)             KIND="$2";       shift 2 ;;
    --scope)            SCOPE="$2";      shift 2 ;;
    --acl)              ACL+=("$2");     shift 2 ;;
    --json)             JSON=true;       shift   ;;
    --payload-sanitised) SANITISED=true; shift   ;;
    -h|--help)
      cat <<EOF
usage: cyberos-memory-capture --kind <module.event> [--scope private|shareable] [--acl @actor]* [--payload-sanitised]
  stdin: JSON payload
  output: EmitOutcome JSON (default) or human-readable text
EOF
      exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[ -n "$KIND" ] || { echo "ERROR: --kind required" >&2; exit 1; }
PAYLOAD=$(cat)

# Invoke the actual skill binary via broker socket (skill binary is installed in $PATH at OCI install time)
exec cyberos-memory-capture-main --kind "$KIND" --scope "$SCOPE" \
  $(printf -- "--acl %s " "${ACL[@]:-}") \
  ${SANITISED:+--payload-sanitised} \
  <<< "$PAYLOAD"
```

---

## §4 — Acceptance criteria

1. **Rust SDK: emit succeeds** — `MemoryCapture::new()` + `emit("notes.captured", ..., Private)` → `EmitOutcome { row_id != "", deduped: false }`.
2. **Rust SDK: dedup hit** — second identical emit within 60s → `EmitOutcome { deduped: true, row_id == prior.row_id }`.
3. **Rust SDK: kind regex enforced** — `emit("InvalidKind", ...)` → `Err(InvalidKind)`.
4. **Rust SDK: broker-down retry** — broker down for 1s then up → SDK retries 3×; succeeds on 3rd; total delay < 3s.
5. **Rust SDK: broker-down surrender** — broker down for 30s → `Err(BrokerDown)` after retries.
6. **Rust SDK: PII scrub** — payload with `AKIA0123456789012345` → scrubbed before broker call; broker receives `<AWS_KEY>` instead.
7. **Rust SDK: payload_sanitised flag** — flag set → SDK skips its own scrub; broker still scrubs (defense-in-depth).
8. **Rust SDK: W3C trace_id from env** — `TRACEPARENT=00-0af...-...-01` → `EmitOutcome.trace_id == "0af..."`.
9. **Rust SDK: W3C trace_id generated when env absent** — no TRACEPARENT → fresh 32-char hex trace_id per `new()`.
10. **Python SDK: emit succeeds** — equivalent of #1 from Python.
11. **Python SDK: dedup hit** — equivalent of #2.
12. **Python SDK: exception types** — broker down → `BrokerDown` exception; denied → `Denied`; PII failure → `PiiScrubFailed`.
13. **Bash CLI: stdin JSON** — `echo '{"x":1}' | cyberos-memory-capture --kind a.b` → `EmitOutcome` JSON to stdout, exit 0.
14. **Bash CLI: missing --kind** — `cyberos-memory-capture` → exit 1 with stderr.
15. **Bash CLI: --acl multi** — `--acl @alice --acl @bob` → scope becomes `Shareable { acl: ["@alice", "@bob"] }`.
16. **SKILL.md validates** — `cyberos skill validate skills/memory-capture/` → exit 0 (TASK-SKILL-103 validator).
17. **SKILL.md signature verified** — release-time signature → validate returns `signature: ok`.
18. **Broker enforcement: only MemoryEmit allowed** — try to call `Bash` from inside this skill → broker returns `tool_not_in_allowed_tools`.
19. **Bundle signed via TASK-SKILL-102** — `sign-bundle.sh` produces `.skill` artifact; `cyberos skill verify <artifact>` returns `signed_by: <key>`.
20. **Dedup TTL: 60s expiry** — first emit; wait 61s; identical emit → fresh (not deduped).
21. **OTel span per emit** — exporter receives `skill.memory_capture.emit` with attrs.
22. **Metric: outcome counter** — counters `skill_memory_capture_emits_total{outcome="emitted"}`, `{outcome="deduped"}` increment correctly.
23. **CYBEROS_BROKER_SOCKET env override** — set env to `/tmp/test.sock`; SDK connects to that socket.

---

## §5 — Verification

```rust
// skills/memory-capture/tests/sdk_rust_test.rs

#[tokio::test]
async fn emit_succeeds_against_stub_broker() {
    let broker = StubBroker::start().await;
    std::env::set_var("CYBEROS_BROKER_SOCKET", broker.socket_path());
    let mut cb = MemoryCapture::new().await.unwrap();
    let outcome = cb.emit("notes.captured", json!({"title": "x"}), EmitScope::Private).await.unwrap();
    assert!(!outcome.deduped);
    assert!(outcome.row_id.starts_with("row-"));
}

#[tokio::test]
async fn dedup_within_60s() {
    let broker = StubBroker::start().await;
    std::env::set_var("CYBEROS_BROKER_SOCKET", broker.socket_path());
    let mut cb = MemoryCapture::new().await.unwrap();
    let first  = cb.emit("notes.captured", json!({"title": "y"}), EmitScope::Private).await.unwrap();
    let second = cb.emit("notes.captured", json!({"title": "y"}), EmitScope::Private).await.unwrap();
    assert_eq!(first.row_id, second.row_id);
    assert!(!first.deduped);
    assert!(second.deduped);
    assert_eq!(broker.call_count().await, 1);  // only one broker call
}

#[tokio::test]
async fn invalid_kind_rejected() {
    let mut cb = MemoryCapture::test().await;
    let err = cb.emit("InvalidCamelCase", json!({}), EmitScope::Private).await.unwrap_err();
    assert!(matches!(err, CaptureError::InvalidKind(_)));
}

#[tokio::test]
async fn pii_scrubbed_before_broker() {
    let broker = StubBroker::start().await;
    std::env::set_var("CYBEROS_BROKER_SOCKET", broker.socket_path());
    let mut cb = MemoryCapture::new().await.unwrap();
    let _ = cb.emit("notes.captured", json!({"body": "key AKIA0123456789012345"}), EmitScope::Private).await;
    let last = broker.last_payload().await;
    assert!(!last.to_string().contains("AKIA0123456789012345"));
    assert!(last.to_string().contains("<AWS_KEY>"));
}

#[tokio::test]
async fn broker_down_retries_3_times_then_surrenders() {
    std::env::set_var("CYBEROS_BROKER_SOCKET", "/tmp/no-such-socket");
    let mut cb = MemoryCapture::new().await.unwrap();
    let start = std::time::Instant::now();
    let err = cb.emit("notes.captured", json!({}), EmitScope::Private).await.unwrap_err();
    assert!(matches!(err, CaptureError::BrokerDown));
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(2600));  // 100 + 500 + 2000 = 2600ms minimum
    assert!(elapsed <  Duration::from_secs(5));
}
```

```python
# skills/memory-capture/tests/sdk_python_test.py
import subprocess
import json

def test_emit_succeeds(stub_broker):
    from cyberos_memory_capture import MemoryCapture, EmitScope
    cb = MemoryCapture()
    outcome = cb.emit("notes.captured", {"title": "py"}, EmitScope.private_())
    assert outcome.row_id
    assert not outcome.deduped

def test_dedup_python(stub_broker):
    from cyberos_memory_capture import MemoryCapture, EmitScope
    cb = MemoryCapture()
    a = cb.emit("notes.captured", {"x": 1}, EmitScope.private_())
    b = cb.emit("notes.captured", {"x": 1}, EmitScope.private_())
    assert b.deduped
    assert a.row_id == b.row_id
```

```bash
# skills/memory-capture/tests/cli_e2e_test.sh
set -euo pipefail

echo '{"title":"cli"}' | cyberos-memory-capture --kind notes.captured --json > /tmp/result.json
[ "$(jq -r .row_id /tmp/result.json | head -c 4)" = "row-" ]
[ "$(jq -r .deduped /tmp/result.json)" = "false" ]

# Missing --kind
! cyberos-memory-capture --json < /dev/null 2>&1 | grep -q "ERROR"

# Idempotent
echo '{"x":2}' | cyberos-memory-capture --kind a.b --json > /tmp/r1.json
echo '{"x":2}' | cyberos-memory-capture --kind a.b --json > /tmp/r2.json
[ "$(jq -r .deduped /tmp/r2.json)" = "true" ]
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **TASK-SKILL-103 (upstream)** — frontmatter schema.
- **TASK-SKILL-104 (upstream)** — capability broker enforces `allowed_tools: [MemoryEmit]`.
- **TASK-SKILL-102** — OCI distribution + signature pipeline.
- **TASK-MEMORY-101** — MemoryWriter (called via broker).
- **TASK-MEMORY-106** — sync_class enforcement.
- **TASK-MEMORY-107** — capture daemon (handles emit forwarding).
- **TASK-MEMORY-109** — Claude Code hook is a SIBLING capture path (also writes via TASK-MEMORY-107).
- **TASK-MEMORY-111** — PII detection used inside SDK pre-scrub.

---

## §8 — Example payloads

### EmitOutcome (Rust)

```json
{
  "row_id":   "row-01HZK9R8M3X5C8Q4ABCDEF",
  "seq":      4287,
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "deduped":  false
}
```

### Dedup hit (subsequent call)

```json
{
  "row_id":   "row-01HZK9R8M3X5C8Q4ABCDEF",
  "seq":      4287,
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "deduped":  true
}
```

---

## §9 — Open questions

All resolved. Deferred:
- TypeScript SDK — slice 3+ (per §1 #15).
- Streaming emit (emit a large body in chunks) — slice 4+; current 16 MB body cap suffices.
- Schema-versioning of payload (consumer-side enforced) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Broker socket missing | UnixStream::connect Err | exp backoff retry; final `BrokerDown` | Wait for TASK-MEMORY-110 to restart |
| Broker rejects (denied) | JSON-RPC error | `CaptureError::Denied(reason)` | Caller fixes frontmatter |
| PII scrub crashes | catch_unwind | `PiiScrubFailed`; caller may retry with sanitised flag | Author updates ruleset |
| Invalid kind | regex no-match | `InvalidKind` | Caller fixes kind string |
| Payload not serializable | serde_json::to_string Err | `InvalidPayload` | Caller passes serde-compatible value |
| Dedup cache full | LRU evicts oldest | New entry replaces; possible false-miss after 1000 unique emits | Acceptable |
| Subprocess (Python wrapper) timeout | subprocess.run timeout=10 | CaptureError raised | Python caller retries |
| Subprocess return code != 0 | stderr parse | Specific exception per error class | Python caller handles |
| W3C trace_id env malformed | parse fails | Falls back to generated trace_id | Logged WARN |
| Multiple SDK instances dedup-cache-mismatch | each has own LRU | Slight inefficiency (duplicate emits across instances) | Acceptable (broker-side dedup catches) |
| OTel exporter unavailable | metric buffered | Metric eventually dropped after buffer fills | Operator restarts TASK-OBS-001 |
| Signature missing at sign-bundle.sh time | release key missing | sign-bundle fails; release blocked | Operator restores key |
| OCI registry push fails | crane push Err | Release blocked | Operator retries |
| Python pyo3 native build fails | wheel build Err | Skill installable as subprocess shim (current default) | By design |
| Memory leak in long-running caller | LRU bounded | Cache size capped; safe | None |
| `payload_sanitised: true` lies (raw PII slips through) | Broker's TASK-MEMORY-111 scan catches it | Broker denies; caller error | Caller stops lying |
| Concurrent emit on same SDK instance | Mutex on dedup cache | Serialised; throughput bounded but correct | Use multiple instances for parallelism |
| Broker upgrade mid-emit | connection drop | exp backoff catches new broker | Acceptable |
| Bash CLI subprocess SIGKILL by parent | broker call may complete or not | Exit non-zero; caller retries | Caller responsibility |

---

## §11 — Implementation notes

- The bundle ships as a single `.skill` tar.zst per TASK-SKILL-102 format; contents: `main` binary + `SKILL.md` + `sdk-rust/` + `sdk-python/` + `cli/`. Total bundle size target: < 5 MB.
- The Rust SDK is published to crates.io as `cyberos-memory-capture = "1.0"`; Python wrapper to PyPI as `cyberos-memory-capture`; both depend on the binary being installed via OCI.
- The Python wrapper is a subprocess shim (not pyo3) for simpler distribution: pure-Python wheel, no native build. Trade-off: ~10ms per call for subprocess spawn. Acceptable for typical capture workloads.
- The dedup LRU is per-SDK-instance, not global. Long-lived callers (daemons) keep one MemoryCapture instance; short-lived callers (scripts) get fresh dedup each invocation. Broker-side has its own dedup as defense-in-depth.
- `payload_sanitised: true` is honoured by the SDK but the broker still runs TASK-MEMORY-111 scrub. Pre-scrub is a performance optimisation, not a security boundary.
- The `kind` regex (`module.event_type`) is shared with TASK-AI-003's closed-set list; this skill emits canonical kinds documented there.
- The bash CLI delegates to the binary (`cyberos-memory-capture-main`); the wrapper just parses flags and constructs JSON. This keeps the bash file < 50 lines.
- The Python `EmitScope.private_()` method name has a trailing underscore because `private` is a reserved-adjacent Python keyword in some contexts; we use `private_` for clarity.
- The release sign-bundle.sh is a thin wrapper around TASK-SKILL-102's signing pipeline; it doesn't duplicate signing logic, just composes the inputs.
- TypeScript SDK is deferred to slice 3+; placeholder noted to prevent author confusion about scope.

---

*End of TASK-SKILL-105.*
