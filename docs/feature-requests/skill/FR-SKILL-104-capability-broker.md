---
id: FR-SKILL-104
title: "Capability broker — subprocess sandbox enforces allowed_tools + allowed_memory_scopes at invoke time; tool-name allowlist + path-glob allowlist + timeout enforcement"
module: SKILL
priority: MUST
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-101, FR-SKILL-102, FR-SKILL-103, FR-SKILL-105, FR-MEMORY-101, FR-MEMORY-106, FR-AUTH-003]
depends_on: [FR-SKILL-103]
blocks: [FR-SKILL-105, FR-SKILL-108, FR-SKILL-109, FR-SKILL-110]

source_pages:
  - website/docs/modules/skill.html#capability-broker
  - website/docs/runbooks/skill-broker-runbook.html
source_decisions:
  - DEC-190 (broker is the ONLY tool dispatcher; skills cannot call tools directly)
  - DEC-191 (broker validates every tool call against frontmatter at request time; denial = audit row + error)
  - DEC-192 (broker enforces effort_minutes timeout via SIGTERM after 90% deadline, SIGKILL after 100%)
  - AGENTS.md §11 (skill body is UNTRUSTED text; tool calls require explicit grant per skill)

language: rust 1.81
service: cyberos/services/skill-broker/
new_files:
  - services/skill-broker/src/broker.rs
  - services/skill-broker/src/dispatcher.rs
  - services/skill-broker/src/enforce.rs
  - services/skill-broker/src/registry.rs
  - services/skill-broker/src/tools/mod.rs
  - services/skill-broker/src/tools/bash.rs
  - services/skill-broker/src/tools/read.rs
  - services/skill-broker/src/tools/memory.rs
  - services/skill-broker/src/tools/http.rs
  - services/skill-broker/tests/broker_e2e_test.rs
  - services/skill-broker/tests/enforce_test.rs
  - services/skill-broker/tests/timeout_test.rs
modified_files:
  - services/skill-broker/src/lib.rs                    # re-export broker + dispatcher
  - services/skill-broker/src/main.rs                   # spawn broker daemon listening on Unix socket
allowed_tools:
  - file_read: services/skill-broker/**
  - file_write: services/skill-broker/{src,tests}/**
  - bash: cd services/skill-broker && cargo test
disallowed_tools:
  - allow skill subprocess to inherit broker's file descriptors (per §1 #4 — seal stdin/stdout/stderr only)
  - dispatch a tool call without checking allowed_tools (per DEC-191)
  - dispatch a MemoryRead with path outside allowed_memory_scopes (per §1 #5)

effort_hours: 12
sub_tasks:
  - "0.5h: broker.rs — Broker struct holding tool registry + invocation handlers"
  - "1.0h: dispatcher.rs — JSON-RPC over Unix socket; one request = one tool call; one response"
  - "1.0h: enforce.rs — pre-dispatch checks: tool in allowed_tools? path in allowed_memory_scopes? request size ≤ 1MB?"
  - "0.5h: registry.rs — global MCP_TOOL_REGISTRY; FR-SKILL-104 owns registration; FR-SKILL-103 validates names against it"
  - "1.5h: tools/bash.rs — sandboxed bash exec (no env inheritance; no PATH; timeout from frontmatter)"
  - "1.0h: tools/read.rs — file read with PATH allowlist (skill must declare files in `allowed_files` extension; default deny)"
  - "1.0h: tools/memory.rs — MemoryRead + MemorySearch + MemoryEmit; path-glob check against allowed_memory_scopes"
  - "1.0h: tools/http.rs — HttpFetch + HttpPost with domain allowlist (skill declares `allowed_domains`)"
  - "1.0h: timeout enforcement — tokio::time::timeout at 90% deadline; SIGTERM; 10s grace then SIGKILL"
  - "1.5h: broker_e2e_test.rs — invoke skill that calls Read + MemoryEmit; verify enforce works; verify timeout"
  - "1.0h: enforce_test.rs — tool-not-allowed → reject; scope-not-allowed → reject; oversized request → reject"
  - "1.0h: timeout_test.rs — skill runs forever; SIGTERM at 0.9×; SIGKILL at 1.0×; broker.tool_calls_total{outcome='timeout'} increments"
risk_if_skipped: "Without enforcement, FR-SKILL-103's frontmatter is decorative — a skill declares `allowed_tools: [Read]` but can call Bash, MemoryEmit, anything. Every skill becomes a potential privilege-escalation vector. Without per-call path-glob check on memory, a skill granted `memories/projects/cyberos/**` can read `memories/people/founders/compensation`. Without timeout, a skill stuck in an infinite loop holds the broker's connection forever. Without sandbox sealing, the skill inherits broker's env vars (potentially containing the JWT signing key) — total auth compromise. This FR is load-bearing; skipping it makes the entire SKILL module security-theatre."
---

## §1 — Description (BCP-14 normative)

The capability broker **MUST** be the sole tool dispatcher for skills. Skills communicate with the broker via JSON-RPC over a Unix socket; every tool call is authorised against the skill's frontmatter at request time. The contract:

1. **MUST** listen on a per-skill Unix socket `/tmp/cyberos-skill-broker.<skill_invocation_id>.sock` created at invoke time and removed at termination. The socket is the ONLY IPC channel — the skill subprocess has no other access to broker state.
2. **MUST** speak JSON-RPC 2.0 over length-prefixed framing (4-byte BE u32 length || JSON body). Methods: `tool.call`, `tool.list`, `broker.status`. Request size capped at 1 MB; response size capped at 16 MB.
3. **MUST** enforce frontmatter at every `tool.call`:
    - `tool.method` MUST be in the skill's `allowed_tools` list AND in the global `MCP_TOOL_REGISTRY`.
    - `tool.method` MUST NOT be in the skill's `disallowed_tools` list (denylist overrides).
    - For `MemoryRead` + `MemorySearch`: every requested `path` MUST match at least one glob in `allowed_memory_scopes`.
    - For `Read` + `Edit` + `Write`: paths MUST match `allowed_files` (`x-allowed-files: [...]` frontmatter extension; default deny).
    - For `HttpFetch` + `HttpPost`: hostnames MUST match `allowed_domains` (`x-allowed-domains: [...]` extension; default deny).
    - Violation → return JSON-RPC error `{"code": -32603, "data": {"reason": "tool_not_allowed" | "scope_violation" | "domain_violation" | "file_violation"}}`; broker emits `skill.tool_denied` audit row carrying the violation details.
4. **MUST** sandbox the skill subprocess:
    - `close_fds(3..MAX)` — close all inherited file descriptors except stdin/stdout/stderr.
    - `env_clear()` then re-set only `CYBEROS_BROKER_SOCKET`, `CYBEROS_SKILL_ID`, `CYBEROS_INVOCATION_ID`, `CYBEROS_TENANT_ID`, `RUST_LOG=warn`.
    - Set `unshare(CLONE_NEWPID)` on Linux when broker has CAP_SYS_ADMIN; otherwise skip with WARN log.
    - Set `setrlimit(RLIMIT_AS, 512MB)`, `RLIMIT_CPU, 60s)` (override per frontmatter `effort_minutes * 60`), `RLIMIT_NPROC, 8)`.
5. **MUST** enforce `allowed_memory_scopes` path-glob check using `globset` (same crate FR-SKILL-103 uses for parse-time validation). Globs are evaluated against the post-canonicalisation memory path (per AGENTS.md §0.4). The check happens AT EVERY `MemoryRead`/`MemorySearch`/`MemoryEmit` call — even within the same invocation; this prevents glob bypass via mid-session reconfiguration.
6. **MUST** enforce `effort_minutes` timeout:
    - At T=0, start a tokio timer of `effort_minutes * 60 * 0.9` seconds.
    - At T=90%, send SIGTERM to subprocess; broker continues to accept the tool.call response on the socket for up to 10s.
    - At T=100%, send SIGKILL; broker closes socket; emits `skill.timeout` audit row.
6.5. **MUST** treat default `effort_minutes` as 30 minutes (1800s) when frontmatter doesn't specify.
7. **MUST** emit memory audit rows for every tool dispatch:
    - `skill.tool_call_started` BEFORE dispatch with `{invocation_id, skill_id, tool_name, args_hash, trace_id}`.
    - `skill.tool_call_completed` AFTER with `{invocation_id, tool_name, outcome (success|error|denied|timeout), duration_ms, result_hash, trace_id}`.
    - `skill.tool_denied` on enforcement violation with `{invocation_id, skill_id, tool_name, reason, attempted_args_hash, trace_id}`.
    - `skill.timeout` when SIGKILL fires with `{invocation_id, skill_id, elapsed_ms, last_tool_call, trace_id}`.
8. **MUST** propagate W3C TraceContext (per FR-AI-022) — every tool.call includes a `traceparent` field; the broker injects this into the underlying tool (e.g. `HttpFetch` adds it as a request header; `MemoryEmit` writes it into the row payload).
9. **MUST** register the canonical tool set at broker startup. The registry is a static `MCP_TOOL_REGISTRY: BTreeMap<ToolName, Box<dyn Tool>>`. Native tools: `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `MemoryRead`, `MemorySearch`, `MemoryEmit`, `HttpFetch`, `HttpPost`. MCP-server-provided tools are registered dynamically when the broker connects to an MCP server (slice-3+).
10. **MUST** emit OTel span `skill.broker.tool_call` per call with attributes `skill_id`, `tool_name`, `outcome`, `duration_ms`, `args_size_bytes`, `result_size_bytes`. Per-tool spans cascade — e.g. `Bash` further emits `skill.tool.bash.exec`.
11. **MUST** emit OTel metrics:
    - `skill_broker_tool_calls_total{tool_name, outcome}` (counter; outcome ∈ success | error | denied | timeout).
    - `skill_broker_tool_duration_seconds{tool_name}` (histogram).
    - `skill_broker_active_invocations` (gauge).
    - `skill_broker_socket_bytes_total{direction}` (counter; direction ∈ tx | rx).
12. **MUST** expose `cyberos skill broker status` showing active invocations, total tool calls today, denied calls today (alerting signal).
13. **MUST** support `cyberos skill broker tail --invocation <id>` to live-stream tool calls for one invocation (operator debugging).
14. **SHOULD** support `cyberos skill broker replay --invocation <id>` reconstructing a past invocation's tool-call sequence from memory audit rows (slice-3+; placeholder).
15. **SHOULD** support per-skill rate limits (max tool calls per minute) — slice-3+; default unbounded.

---

## §2 — Why this design (rationale for humans)

**Why a broker process at all (§1 #1)?** Without a broker, every skill would need its own tool implementation. The broker centralises: tool implementations (one impl of `Bash`, not N), enforcement (one place to check `allowed_tools`), audit (one place to emit memory rows), observability (one place to attach OTel spans). Skills become thin wrappers around `tool.call` JSON-RPC.

**Why Unix sockets (§1 #1)?** Per-invocation socket means: (a) cleanup is trivial (delete file on exit), (b) skill subprocess can't talk to other invocations' brokers, (c) no network surface. The socket path embeds `invocation_id` so debugging is easy (`lsof | grep <id>`).

**Why JSON-RPC 2.0 (§1 #2)?** Standard protocol; widely supported by client libraries; clear error semantics (`{"error": {"code": N, "data": {...}}}`). Length-prefixed framing avoids JSON-parsing-state machines.

**Why enforcement at EVERY tool call (§1 #3 + #5)?** A skill granted broad scopes early in execution could narrow itself later (e.g. reading frontmatter from a different SKILL.md it just downloaded). Per-call checks prevent any drift: every call validated against the originally-loaded frontmatter, no mid-flight changes.

**Why subprocess sandbox details (§1 #4)?** Three threat models:
- **FD leak**: broker may have an open writer-process FD (per FR-MEMORY-101); skill could write to memory bypassing the broker. `close_fds` prevents.
- **Env leak**: broker's env contains `CYBEROS_JWT_SECRET` for signing tokens; skill could read & exfiltrate. `env_clear` prevents.
- **PID namespace** isolates the skill from seeing other processes on the host (defense-in-depth; not a strong boundary alone).
- **rlimit** prevents fork bombs (`NPROC`) and memory exhaustion (`AS`).

**Why path canonicalisation before glob check (§1 #5)?** A skill could try `MemoryRead("memories/projects/cyberos/../people/founders/compensation")` — without canonicalisation, the glob `memories/projects/cyberos/**` might match (depending on glob library); with canonicalisation, the path normalises to `memories/people/founders/compensation` and the glob fails.

**Why 90% / 100% timeout split (§1 #6)?** SIGTERM lets the skill clean up (close files, flush buffers, exit gracefully). 10% grace period (e.g. 3 minutes on a 30-min budget) is enough for most graceful-shutdown patterns. SIGKILL after 100% guarantees the broker reclaims resources.

**Why audit BOTH started AND completed (§1 #7)?** "Started without completed" = crash; investigators see exactly where the failure occurred. "Completed without started" = impossible (broker writes started before dispatch); used as an integrity check.

**Why args_hash, not args (§1 #7)?** Same reason FR-MEMORY-109 hashes Claude Code tool args — secrets in args (Bash commands like `curl -H "Bearer ..."`). Hash for dedup + audit; never store raw.

**Why default-deny everywhere (§1 #3)?** Default-allow is unsafe by construction. A skill that declares no `allowed_memory_scopes` should have no memory access (not "everything"); a skill that declares no `x-allowed-files` should not be able to read arbitrary files. Frontmatter is opt-in for capabilities.

**Why x-prefixed extensions for files + domains (§1 #3)?** The v1 schema (FR-SKILL-103) is frozen at 5 required + 8 optional fields. Adding `allowed_files` + `allowed_domains` as required would break existing skills; making them v2 fields requires a major bump. The `x-` extension namespace (per FR-SKILL-103 §1 #5) is the right escape hatch — these are de facto required for skills that read files or call HTTP, but v1 schema doesn't enforce them.

**Why per-tool span cascade (§1 #10)?** Aggregated dashboards: "how often does the `Bash` tool fail vs succeed?", "what's the p99 latency of MemorySearch in this skill?" Per-call spans roll up into per-tool histograms.

**Why `cyberos skill broker tail` (§1 #13)?** Operators debugging a misbehaving skill need real-time tool-call visibility. Reading audit rows post-hoc has latency. Tail is the equivalent of `tail -f` for skill broker IPC.

---

## §3 — API contract

### JSON-RPC over Unix socket

```jsonc
// Request: tool.call
{
  "jsonrpc": "2.0",
  "id":      "call-123",
  "method":  "tool.call",
  "params": {
    "tool":  "MemoryRead",
    "args":  {"path": "memories/projects/cyberos/notes/auth-design.md"},
    "traceparent": "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
  }
}

// Response: success
{
  "jsonrpc": "2.0",
  "id":      "call-123",
  "result":  {"body": "...", "content_hash": "9b0e8c5..."}
}

// Response: denied (enforcement violation)
{
  "jsonrpc": "2.0",
  "id":      "call-123",
  "error": {
    "code":    -32603,
    "message": "tool_not_allowed",
    "data":    {"reason": "tool_not_in_allowed_tools", "tool": "Bash", "skill_id": "memory-capture"}
  }
}
```

### Enforce module

```rust
// services/skill-broker/src/enforce.rs
use crate::frontmatter::SkillFrontmatter;
use globset::GlobSet;

#[derive(thiserror::Error, Debug)]
pub enum EnforceError {
    #[error("tool {tool:?} is not in skill's allowed_tools")]
    ToolNotAllowed { tool: String, allowed: Vec<String> },
    #[error("tool {tool:?} is in skill's disallowed_tools")]
    ToolDenied   { tool: String },
    #[error("memory scope violation: path {path:?} not in allowed_memory_scopes")]
    ScopeViolation { path: String },
    #[error("file violation: path {path:?} not in x-allowed-files")]
    FileViolation { path: String },
    #[error("domain violation: host {host:?} not in x-allowed-domains")]
    DomainViolation { host: String },
    #[error("request body too large: {n} > {cap}")]
    RequestTooLarge { n: usize, cap: usize },
}

pub struct Enforcer {
    allowed_tools:      std::collections::BTreeSet<String>,
    disallowed_tools:   std::collections::BTreeSet<String>,
    memory_scope_globs:  GlobSet,
    file_globs:         GlobSet,
    domain_globs:       GlobSet,
}

impl Enforcer {
    pub fn from_frontmatter(fm: &SkillFrontmatter) -> anyhow::Result<Self> {
        let allowed_tools: std::collections::BTreeSet<String> =
            fm.allowed_tools.iter().map(|t| format!("{t:?}")).collect();
        let disallowed_tools: std::collections::BTreeSet<String> =
            fm.disallowed_tools.iter().map(|t| format!("{t:?}")).collect();

        let mut memory_b = globset::GlobSetBuilder::new();
        for p in &fm.allowed_memory_scopes { memory_b.add(globset::Glob::new(p)?); }
        let memory_scope_globs = memory_b.build()?;

        let mut file_b = globset::GlobSetBuilder::new();
        if let Some(serde_yaml::Value::Sequence(seq)) = fm.x_extensions.get("x-allowed-files") {
            for v in seq { if let Some(s) = v.as_str() { file_b.add(globset::Glob::new(s)?); } }
        }
        let file_globs = file_b.build()?;

        let mut domain_b = globset::GlobSetBuilder::new();
        if let Some(serde_yaml::Value::Sequence(seq)) = fm.x_extensions.get("x-allowed-domains") {
            for v in seq { if let Some(s) = v.as_str() { domain_b.add(globset::Glob::new(s)?); } }
        }
        let domain_globs = domain_b.build()?;

        Ok(Self { allowed_tools, disallowed_tools, memory_scope_globs, file_globs, domain_globs })
    }

    pub fn check_tool(&self, tool: &str) -> Result<(), EnforceError> {
        if self.disallowed_tools.contains(tool) { return Err(EnforceError::ToolDenied { tool: tool.into() }); }
        if !self.allowed_tools.contains(tool)   { return Err(EnforceError::ToolNotAllowed {
            tool: tool.into(),
            allowed: self.allowed_tools.iter().cloned().collect(),
        }); }
        Ok(())
    }

    pub fn check_memory_path(&self, path: &str) -> Result<(), EnforceError> {
        // Path canonicalisation: handle `..` segments
        let canonical = canonicalise_memory_path(path);
        if !self.memory_scope_globs.is_match(&canonical) {
            return Err(EnforceError::ScopeViolation { path: canonical });
        }
        Ok(())
    }

    pub fn check_file_path(&self, path: &str) -> Result<(), EnforceError> {
        let canonical = std::fs::canonicalize(path).ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| path.into());
        if !self.file_globs.is_match(&canonical) {
            return Err(EnforceError::FileViolation { path: canonical });
        }
        Ok(())
    }

    pub fn check_domain(&self, url: &str) -> Result<(), EnforceError> {
        let host = url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
        if !self.domain_globs.is_match(&host) {
            return Err(EnforceError::DomainViolation { host });
        }
        Ok(())
    }

    pub fn check_request_size(&self, body: &[u8]) -> Result<(), EnforceError> {
        const CAP: usize = 1_048_576;  // 1 MB
        if body.len() > CAP { return Err(EnforceError::RequestTooLarge { n: body.len(), cap: CAP }); }
        Ok(())
    }
}

fn canonicalise_memory_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "."  => continue,
            ".."      => { parts.pop(); }
            _         => parts.push(seg),
        }
    }
    parts.join("/")
}
```

### Dispatcher

```rust
// services/skill-broker/src/dispatcher.rs
use crate::enforce::Enforcer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use std::time::Duration;
use std::sync::Arc;

pub struct Dispatcher {
    pub enforcer:     Arc<Enforcer>,
    pub invocation_id: uuid::Uuid,
    pub skill_id:     String,
    pub trace_id:     String,
    pub registry:     Arc<crate::registry::ToolRegistry>,
    pub memory_writer: Arc<cyberos_memory_writer::MemoryWriter>,
}

impl Dispatcher {
    pub async fn run_one_invocation(&self, socket_path: &std::path::Path, deadline: tokio::time::Instant) {
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path).expect("bind socket");
        let (mut stream, _) = listener.accept().await.expect("accept");

        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => { break; }
                req = read_frame(&mut stream) => {
                    let req = match req { Ok(r) => r, Err(_) => break };
                    if let Err(e) = self.enforcer.check_request_size(&req) {
                        let _ = write_frame(&mut stream, error_response(e)).await;
                        continue;
                    }
                    let response = self.dispatch_one(req).await;
                    let _ = write_frame(&mut stream, response).await;
                }
            }
        }
    }

    async fn dispatch_one(&self, req_bytes: Vec<u8>) -> Vec<u8> {
        let req: JsonRpcReq = match serde_json::from_slice(&req_bytes) {
            Ok(r) => r,
            Err(_) => return error_response_str("invalid_json"),
        };
        // Pre-dispatch enforcement
        if let Err(e) = self.enforcer.check_tool(&req.params.tool) {
            self.emit_denied(&req, &e).await;
            return error_response(e);
        }
        // Per-tool scope/file/domain check
        if let Err(e) = self.check_args(&req.params.tool, &req.params.args) {
            self.emit_denied(&req, &e).await;
            return error_response(e);
        }
        // Audit: started
        self.emit_started(&req).await;
        // Dispatch
        let start = std::time::Instant::now();
        let result = self.registry.get(&req.params.tool)
            .expect("tool exists; registry populated at startup")
            .invoke(&req.params.args, &self.trace_id).await;
        // Audit: completed
        let duration = start.elapsed();
        self.emit_completed(&req, &result, duration).await;
        serde_json::to_vec(&JsonRpcResp { jsonrpc: "2.0".into(), id: req.id, result: Some(result.into()) , error: None }).unwrap()
    }

    fn check_args(&self, tool: &str, args: &serde_json::Value) -> Result<(), crate::enforce::EnforceError> {
        match tool {
            "MemoryRead" | "MemorySearch" | "MemoryEmit" => {
                if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
                    self.enforcer.check_memory_path(p)?;
                }
            }
            "Read" | "Write" | "Edit" => {
                if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
                    self.enforcer.check_file_path(p)?;
                }
            }
            "HttpFetch" | "HttpPost" => {
                if let Some(u) = args.get("url").and_then(|v| v.as_str()) {
                    self.enforcer.check_domain(u)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    // emit_* methods elided; they write canonical::skill_* audit rows via memory_writer
}
```

### Subprocess spawn

```rust
// services/skill-broker/src/broker.rs (excerpt)
pub async fn spawn_skill_subprocess(
    skill_id: &str,
    socket_path: &std::path::Path,
    tenant_id: uuid::Uuid,
    effort_minutes: u32,
) -> anyhow::Result<tokio::process::Child> {
    use std::os::unix::process::CommandExt;
    use tokio::process::Command;

    let mut cmd = Command::new(format!("/opt/cyberos/skills/{skill_id}/main"));
    cmd.env_clear();
    cmd.env("CYBEROS_BROKER_SOCKET",  socket_path);
    cmd.env("CYBEROS_SKILL_ID",       skill_id);
    cmd.env("CYBEROS_INVOCATION_ID",  uuid::Uuid::new_v4().to_string());
    cmd.env("CYBEROS_TENANT_ID",      tenant_id.to_string());
    cmd.env("RUST_LOG",               "warn");
    // unsafe: pre_exec runs in the child after fork, before exec
    unsafe { cmd.pre_exec(move || {
        // close FDs 3..MAX (stdin=0, stdout=1, stderr=2 inherited)
        for fd in 3..1024 { libc::close(fd); }
        // rlimit
        let cpu = libc::rlimit { rlim_cur: (effort_minutes as u64 * 60), rlim_max: (effort_minutes as u64 * 60) };
        libc::setrlimit(libc::RLIMIT_CPU, &cpu);
        let mem = libc::rlimit { rlim_cur: 512 * 1024 * 1024, rlim_max: 512 * 1024 * 1024 };
        libc::setrlimit(libc::RLIMIT_AS, &mem);
        let proc = libc::rlimit { rlim_cur: 8, rlim_max: 8 };
        libc::setrlimit(libc::RLIMIT_NPROC, &proc);
        Ok(())
    }); }
    let child = cmd.spawn()?;
    Ok(child)
}
```

### Timeout enforcement

```rust
// Inside the invocation orchestrator
pub async fn enforce_timeout(child: &mut tokio::process::Child, effort_minutes: u32) {
    let total = std::time::Duration::from_secs((effort_minutes as u64) * 60);
    let warn_at = total.mul_f32(0.9);
    let sigterm_at = tokio::time::Instant::now() + warn_at;
    let sigkill_at = tokio::time::Instant::now() + total;

    tokio::select! {
        _ = tokio::time::sleep_until(sigterm_at) => {
            if let Some(pid) = child.id() {
                tracing::warn!(pid, "approaching deadline; sending SIGTERM");
                unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            }
            tokio::time::sleep_until(sigkill_at).await;
            if let Some(pid) = child.id() {
                if child.try_wait().ok().flatten().is_none() {
                    tracing::error!(pid, "deadline exceeded; sending SIGKILL");
                    unsafe { libc::kill(pid as i32, libc::SIGKILL); }
                }
            }
        }
        result = child.wait() => {
            // Normal exit
            tracing::info!(?result, "skill subprocess exited within deadline");
        }
    }
}
```

---

## §4 — Acceptance criteria

1. **Tool call succeeds for allowed tool** — skill with `allowed_tools: [MemoryRead]` calls `tool.call({tool: "MemoryRead", args: {path: "memories/projects/cyberos/notes/x.md"}})` → success response.
2. **Tool not in allowed_tools rejected** — same skill calls `Bash` → JSON-RPC error `tool_not_in_allowed_tools`; `skill.tool_denied` audit row emitted.
3. **Tool in disallowed_tools rejected** — skill with `disallowed_tools: [Bash]` calls Bash → rejected (denylist overrides allowlist).
4. **memory scope check works** — skill with `allowed_memory_scopes: ["memories/projects/cyberos/**"]` reads `memories/projects/cyberos/x.md` → ok; reads `memories/people/y.md` → `scope_violation`.
5. **Path canonicalisation prevents glob bypass** — skill tries `memories/projects/cyberos/../people/y.md` → canonicalises to `memories/people/y.md` → `scope_violation`.
6. **File path enforcement** — skill with `x-allowed-files: ["/tmp/**"]` reads `/tmp/x.txt` → ok; reads `/etc/passwd` → `file_violation`.
7. **Domain enforcement** — skill with `x-allowed-domains: ["api.example.com"]` fetches `https://api.example.com/x` → ok; fetches `https://evil.com/y` → `domain_violation`.
8. **Subprocess env cleared** — skill prints `env | grep CYBEROS` → output contains only `BROKER_SOCKET`, `SKILL_ID`, `INVOCATION_ID`, `TENANT_ID`; no `JWT_SECRET`, `AWS_*`, etc.
9. **Subprocess FDs closed** — skill checks `/proc/self/fd/` (Linux) → only fds 0/1/2 present; no leaked broker fds.
10. **rlimit enforced (memory)** — skill mallocs 1 GB → SIGSEGV / process killed (RLIMIT_AS = 512 MB).
11. **rlimit enforced (fork)** — skill spawns 16 children → fails after 8 (RLIMIT_NPROC).
12. **SIGTERM at 90%** — skill with `effort_minutes: 1` (60s); broker sends SIGTERM at T=54s.
13. **SIGKILL at 100%** — same skill ignores SIGTERM; broker SIGKILLs at T=60s; `skill.timeout` audit row emitted.
14. **Audit: started + completed pair** — every successful tool call → exactly one `skill.tool_call_started` + one `skill.tool_call_completed` with matching invocation_id + trace_id.
15. **Audit: denied row** — every enforcement violation → `skill.tool_denied` with `reason`, `attempted_args_hash`, `trace_id`.
16. **Request size cap** — 2 MB request body → `request_too_large` error; broker continues to accept further requests on same socket.
17. **JSON-RPC error code -32603** — every enforcement error uses code -32603 (Internal error) per JSON-RPC 2.0 conventions.
18. **W3C trace propagation: HttpFetch** — tool.call carries `traceparent`; emitted HTTP request to backend has the same `traceparent` header.
19. **W3C trace propagation: MemoryEmit** — emitted memory row's payload.trace_id matches.
20. **Broker status CLI** — `cyberos skill broker status` → JSON with `active_invocations`, `tool_calls_today`, `denied_today`.
21. **Broker tail CLI** — `cyberos skill broker tail --invocation <id>` → line per tool call as they occur.
22. **OTel span per tool call** — exporter receives `skill.broker.tool_call` per call with `skill_id`, `tool_name`, `outcome`, `duration_ms`.
23. **Metric: per-outcome counter** — fixture run → counters `tool_calls_total{outcome="success"}`, `{outcome="denied"}`, `{outcome="timeout"}` correct.
24. **Two concurrent invocations don't share sockets** — invocation A's socket at `/tmp/cyberos-skill-broker.<A>.sock`; invocation B at `<B>.sock`; A cannot connect to B's socket.

---

## §5 — Verification

```rust
// services/skill-broker/tests/enforce_test.rs
#[test]
fn tool_not_allowed_rejected() {
    let fm = SkillFrontmatter::test_with(["Read"]);
    let enforcer = Enforcer::from_frontmatter(&fm).unwrap();
    assert!(matches!(enforcer.check_tool("Bash"), Err(EnforceError::ToolNotAllowed { .. })));
}

#[test]
fn memory_scope_glob_match() {
    let mut fm = SkillFrontmatter::test_empty();
    fm.allowed_memory_scopes = vec!["memories/projects/cyberos/**".into()];
    let e = Enforcer::from_frontmatter(&fm).unwrap();
    assert!(e.check_memory_path("memories/projects/cyberos/notes/x.md").is_ok());
    assert!(matches!(e.check_memory_path("memories/people/y.md"), Err(EnforceError::ScopeViolation { .. })));
}

#[test]
fn path_traversal_caught() {
    let mut fm = SkillFrontmatter::test_empty();
    fm.allowed_memory_scopes = vec!["memories/projects/cyberos/**".into()];
    let e = Enforcer::from_frontmatter(&fm).unwrap();
    let res = e.check_memory_path("memories/projects/cyberos/../people/y.md");
    assert!(matches!(res, Err(EnforceError::ScopeViolation { .. })));
}
```

```rust
// services/skill-broker/tests/broker_e2e_test.rs
#[tokio::test]
async fn dispatch_memoryread_succeeds() {
    let env = BrokerTestEnv::with_skill("test-read", &["MemoryRead"], &["memories/projects/cyberos/**"]);
    let client = env.client().await;
    let resp = client.call("MemoryRead", json!({"path": "memories/projects/cyberos/x.md"})).await.unwrap();
    assert!(resp["body"].is_string());
}

#[tokio::test]
async fn dispatch_bash_denied() {
    let env = BrokerTestEnv::with_skill("test-no-bash", &["MemoryRead"], &[]);
    let client = env.client().await;
    let resp = client.call("Bash", json!({"cmd": "ls"})).await;
    assert!(resp.unwrap_err().to_string().contains("tool_not_in_allowed_tools"));
    let row = env.memory.latest("skill.tool_denied").await;
    assert_eq!(row["payload"]["reason"], "tool_not_in_allowed_tools");
}

#[tokio::test]
async fn timeout_enforced() {
    let env = BrokerTestEnv::with_skill("test-timeout", &["Bash"], &[]);
    env.set_effort_minutes(1).await;  // 60s
    // Skill that sleeps 120s — should be SIGKILLed at 60s
    let start = std::time::Instant::now();
    let exit = env.run_skill("sleep 120").await;
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_secs(75));   // 60 + 10 grace + slack
    assert!(elapsed > std::time::Duration::from_secs(55));   // not premature
    let timeout_row = env.memory.latest("skill.timeout").await;
    assert_eq!(timeout_row["payload"]["skill_id"], "test-timeout");
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **FR-SKILL-103 (upstream)** — frontmatter parser; this FR consumes parsed `SkillFrontmatter`.
- **FR-SKILL-101 (upstream)** — memory integration (pre/post audit rows pattern).
- **FR-SKILL-102 (related)** — OCI registry distributes signed bundles; broker reads them.
- **FR-SKILL-105 (downstream)** — memory-capture@1 skill is the first canonical user.
- **FR-MEMORY-101** — MemoryWriter (used by broker to emit audit rows).
- **FR-MEMORY-106** — sync_class semantics.
- **FR-AUTH-003** — RLS on MemoryRead enforces tenant_id scope.

---

## §8 — Example payloads

### `skill.tool_call_started`

```json
{
  "kind": "skill.tool_call_started",
  "payload": {
    "invocation_id": "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "skill_id":      "memory-capture",
    "tool_name":     "MemoryEmit",
    "args_hash":     "9b0e8c5...",
    "trace_id":      "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `skill.tool_denied`

```json
{
  "kind": "skill.tool_denied",
  "payload": {
    "invocation_id":      "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "skill_id":           "memory-capture",
    "tool_name":          "Bash",
    "reason":             "tool_not_in_allowed_tools",
    "attempted_args_hash": "ab12cd...",
    "trace_id":           "0af7651916cd43dd8448eb211c80319c"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Cross-skill IPC (skill A calls skill B via broker) — slice 3+.
- Per-skill rate limits — slice 3+.
- Replay tool from memory audit rows — slice 3+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Skill subprocess crashes mid-dispatch | child.wait Err | `skill.invoked_completed` with outcome=panic; broker socket cleanup | Operator inspects audit row |
| Skill never connects to socket | listener.accept timeout (60s) | broker times out; emits `skill.no_connect` row | Operator checks skill binary |
| Skill makes 1000 rapid tool calls | Per-call enforce check; high throughput | Each enforced; no special-case | Slice-3+: per-skill rate limit |
| Tool implementation throws | tool.invoke Err | error response; tool_call_completed with outcome=error | Operator investigates tool |
| MemoryWriter unavailable | emit Err | Audit row dropped (logged); tool call still completes | Operator restores writer |
| Invalid JSON-RPC on socket | parse Err | error response with code -32700 | Skill should send valid JSON |
| Tool not in registry | registry.get None | error response with `unknown_tool` | Author updates allowed_tools or skill code |
| Glob match on tampered path | canonicalisation catches | scope_violation | None — by design |
| OS lacks pid namespace (macOS) | unshare unavailable | WARN log; skip; rlimit still applied | Acceptable degradation |
| Socket path collision | listener bind Err | broker exits 1; logged | Operator runs cleanup script |
| Subprocess SIGKILL by OS (OOM, etc) | child wait | `skill.timeout` not emitted; instead `skill.invoked_completed` outcome=killed | Operator investigates |
| Long-running tool (MemorySearch with 100k results) | timeout enforcement | SIGTERM at 90%; if tool yields response in grace, emit completed | Operator considers smaller queries |
| HTTP tool: TLS handshake fails | tool returns Err | tool_call_completed outcome=error; row carries error message | Operator checks network |
| Skill writes 16 MB+ response | response cap | broker truncates to 16 MB; logs WARN; metric increments | Operator considers chunked tools |
| Concurrent invocations of same skill | Different invocation_id + socket | Both run independently | By design |
| Frontmatter changed between invocations | Each invocation re-loads | Old invocation uses old frontmatter; new uses new | By design (no live-reload) |
| Broker process restart mid-invocation | child still running but socket gone | Skill connect fails; subprocess exits | Broker re-spawns on next invocation |
| Audit row write fails mid-dispatch | memory_writer Err | Row queued; emit in background; tool call still completes | Operator restores memory |
| Unknown tool name in MCP_TOOL_REGISTRY | startup validation | Broker refuses to start; sev-1 alarm | Operator fixes registry |
| Two skills want different versions of same tool | tool versioning v2+ | v1: registry is single source; tools are global | v2+ design |

---

## §11 — Implementation notes

- The Unix socket pattern means broker process must stay alive for the duration of invocation; it owns the socket file. On broker crash, the file is orphaned; FR-MEMORY-110's sweeper cleans `/tmp/cyberos-skill-broker.*.sock` files older than 5 minutes.
- `unsafe { cmd.pre_exec(...) }` runs in the child between fork and exec; the closure must be async-signal-safe. `libc::close` and `libc::setrlimit` are; `tracing::info!` is NOT (allocates).
- `RLIMIT_AS` (address space) is enforced by the kernel; `RLIMIT_DATA` is more granular but less reliably available across glibc versions.
- `unshare(CLONE_NEWPID)` requires CAP_SYS_ADMIN on Linux; in non-root cyberos installs, we skip with a WARN. Per-skill PID isolation is defense-in-depth, not the primary boundary.
- `globset` is used both at parse time (FR-SKILL-103) and at enforce time (this FR); single glob library = consistent semantics.
- The dispatcher's `check_args` pattern-matches on tool name. For MCP tools (dynamic at startup), the registry entry carries an `arg_validator` fn that performs the equivalent check.
- The trace_id is per-invocation, not per-tool-call. Tool calls within one invocation share trace_id; cascade spans connect them.
- `skill.tool_denied` rows include `attempted_args_hash` (NOT raw args) for the same reason FR-MEMORY-109 hashes Claude Code tool args: args may contain secrets.
- `cyberos skill broker tail` is implemented as a memory query subscription (the audit rows arrive in real-time); avoids needing a separate IPC channel.
- Per-tool span cascade is OTel-native: child spans inherit parent's trace; each tool implementation calls `tracing::info_span!("skill.tool.bash.exec", ...)` to create the child.

---

*End of FR-SKILL-104.*
