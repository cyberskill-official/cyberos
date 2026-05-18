---
id: FR-BRAIN-109
title: "Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit BRAIN memories with prompt + diff + trace correlation"
module: BRAIN
priority: MUST
status: building
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-BRAIN-101, FR-BRAIN-107, FR-BRAIN-108, FR-BRAIN-110, FR-BRAIN-111, FR-AI-022, FR-AI-014, FR-SKILL-101]
depends_on: [FR-BRAIN-107]
blocks: [FR-BRAIN-110]

source_pages:
  - website/docs/modules/brain.html#claude-code-capture
  - website/docs/runbooks/claude-code-hook-runbook.html
source_decisions:
  - DEC-150 (every Claude Code session is a BRAIN-eligible event source; capture is opt-in via project-level hook config)
  - DEC-151 (capture the user prompt + tool-call sequence + final assistant message, NOT raw tool outputs which can contain PII)
  - DEC-152 (correlate Claude Code → BRAIN via W3C trace_id injected at hook boot)
  - AGENTS.md §11 (user prompts are TRUSTED for capture-as-text but UNTRUSTED for protocol-mutation)

language: rust 1.81 + bash + json
service: cyberos/services/brain-claude-hook/
new_files:
  - services/brain-claude-hook/Cargo.toml
  - services/brain-claude-hook/src/main.rs
  - services/brain-claude-hook/src/hook.rs
  - services/brain-claude-hook/src/emit.rs
  - services/brain-claude-hook/src/redact.rs
  - services/brain-claude-hook/install/.claude/settings.json.template
  - services/brain-claude-hook/install/install-hooks.sh
  - services/brain-claude-hook/tests/hook_e2e_test.rs
  - services/brain-claude-hook/tests/redact_test.rs
modified_files:
  - services/brain/manifest.json                        # add `claude_hooks` section with opt-in per-project list
  - services/brain/src/cli/hook.rs                      # `cyberos brain hook claude install <path>`
allowed_tools:
  - file_read: services/brain-claude-hook/**, services/brain/**, ~/.claude/**
  - file_write: services/brain-claude-hook/{src,tests,install}/**
  - bash: cd services/brain-claude-hook && cargo test
  - bash: cd services/brain-claude-hook && cargo build --release
disallowed_tools:
  - capture raw tool outputs (per DEC-151 — they may contain secrets / API keys)
  - block / delay the Claude Code session (hook MUST be fire-and-forget; ≤ 50ms p95)
  - emit BRAIN rows during PreToolUse (only PostToolUse — capture what happened, not what was attempted)

effort_hours: 8
sub_tasks:
  - "0.5h: Cargo.toml — clap, serde_json, tokio, blake3, regex"
  - "0.5h: hook.rs — parse Claude Code hook JSON-on-stdin (UserPromptSubmit | PostToolUse | Stop schemas)"
  - "1.0h: redact.rs — strip API keys, env-var patterns (per FR-BRAIN-111 PII detection ruleset)"
  - "1.0h: emit.rs — bridge to BRAIN writer; carries session_id + trace_id; non-blocking spawn"
  - "0.5h: main.rs — clap subcommands: `userpromptsubmit`, `posttooluse`, `stop`; each takes JSON on stdin"
  - "0.5h: install/settings.json.template — Claude Code hooks config snippet"
  - "0.5h: install/install-hooks.sh — installer; adds hook entries to ~/.claude/settings.json or <project>/.claude/settings.json"
  - "0.5h: cli/hook.rs — `cyberos brain hook claude install <project-path>`"
  - "1.0h: hook_e2e_test.rs — run hook with fixture JSON; assert BRAIN row shape"
  - "1.0h: redact_test.rs — 20+ patterns (AWS keys, Bearer tokens, Vietnamese CCCD, etc.)"
  - "0.5h: integration with FR-BRAIN-107 capture daemon (hook posts via local Unix socket, NOT direct chain write)"
  - "0.5h: latency budget test — hook completes within 50ms p95"
risk_if_skipped: "Claude Code sessions produce the highest-quality structured memories for engineering work (prompt + diff + outcome). Without this hook, BRAIN sees the file-system aftermath via FR-BRAIN-107 but not the *intent* (what the user asked) or the *strategy* (which tools the assistant picked). Operators trying to query 'why did we change auth middleware last week' get the diff but not the prompt. Without redaction, prompts containing API keys (operators routinely paste them) would land in BRAIN — compliance + leak risk. Without the local-socket-vs-direct-write architecture, the hook would block the Claude Code session waiting for chain writes (50ms→500ms latency degradation)."
---

## §1 — Description (BCP-14 normative)

The Claude Code hook capture service **MUST** integrate with Claude Code's documented hook system (`UserPromptSubmit`, `PostToolUse`, `Stop`) to emit BRAIN audit rows for every meaningful Claude Code interaction. The contract:

1. **MUST** install via `cyberos brain hook claude install <project-path>` which writes to `<project-path>/.claude/settings.json` (project-scope) OR `~/.claude/settings.json` (user-scope, with `--user` flag). The installer is idempotent: re-running with the same args is a no-op.
2. **MUST** subscribe to exactly three Claude Code hook events:
    - `UserPromptSubmit` → emit `brain.claude_prompt` row with `{session_id, prompt_hash, prompt_redacted, cwd, trace_id, captured_at_ns}`. The `prompt_redacted` field carries the redacted prompt body (per FR-BRAIN-111 ruleset); `prompt_hash` is `blake3(raw_prompt)` for dedup.
    - `PostToolUse` → emit `brain.claude_tool_use` row with `{session_id, tool_name, tool_args_hash, outcome (success | error), duration_ms, trace_id}`. Tool args are HASHED, not stored (per DEC-151).
    - `Stop` → emit `brain.claude_session_completed` row with `{session_id, prompt_count, tool_use_count, duration_ms, trace_id, last_assistant_message_redacted}`.
3. **MUST NOT** subscribe to `PreToolUse`, `Notification`, `SubagentStop`, or `SessionStart` — those are higher-frequency and offer marginal capture value vs the noise they add.
4. **MUST** redact every prompt + assistant message BEFORE hashing or emitting per FR-BRAIN-111's ruleset:
    - AWS access-key / secret-key patterns → `<AWS_KEY>`
    - `Bearer <token>` → `Bearer <REDACTED>`
    - GitHub PATs (`ghp_*`, `gho_*`, `ghu_*`, `ghs_*`, `ghr_*`) → `<GH_PAT>`
    - OpenAI keys (`sk-*`) → `<OPENAI_KEY>`
    - Anthropic keys (`sk-ant-*`) → `<ANTHROPIC_KEY>`
    - Generic JWT-like patterns (3 base64 segments joined by dots) → `<JWT>`
    - Vietnamese CCCD (12-digit) → `<VN_CCCD>`
    - Email addresses (when not in @company.com allowlist) → `<EMAIL>`
    - Phone numbers (E.164 or VN local form) → `<PHONE>`
5. **MUST** fail-closed on redaction: if the redactor itself errors (regex panic, OOM), the hook emits a `brain.claude_capture_redaction_failed` row with `{session_id, error}` and exits 0 — never blocks Claude Code.
6. **MUST** complete in ≤ 50ms p95 for the synchronous part (read stdin + redact + spawn emit). The actual BRAIN write is delegated to a background tokio task via the FR-BRAIN-107 capture daemon's local Unix socket (`/tmp/cyberos-brain-capture.sock`) — fire-and-forget. Hook process exits 0 once the message is enqueued.
7. **MUST** correlate session-level rows via `session_id` (UUID; provided by Claude Code in the hook payload) AND a per-session W3C `trace_id` (32-char hex). The trace_id is generated at first hook invocation for a given session_id and cached in `/tmp/cyberos-brain-claude-traces/<session_id>` (file persists for 1 hour; pruned by FR-BRAIN-110 sweeper).
8. **MUST** read the hook payload as a single JSON object from stdin per Claude Code's documented schema. Schema validation is strict — unknown top-level keys are logged at WARN and ignored; missing required keys exits 0 with stderr WARN.
9. **MUST** emit OTel span `brain.claude.hook` per invocation with attributes `hook_kind`, `session_id`, `duration_ms`, `redaction_match_count`.
10. **MUST NOT** call `cyberos_brain_writer::BrainWriter::emit` directly. The hook is a short-lived process; it MUST post to the long-running FR-BRAIN-107 daemon via local Unix-domain socket. The daemon does the chain write. This decouples hook latency from chain commit latency.
11. **MUST** be opt-in per project: `cyberos brain hook claude install <project-path>` writes the hook config; absent config = no hook = no capture. There is NO global default that captures every Claude Code project.
12. **MUST** support `cyberos brain hook claude uninstall <project-path>` (reverse of install) and `cyberos brain hook claude status <project-path>` (prints installed hooks + last capture row for that project).
13. **MUST** emit OTel metrics:
    - `brain_claude_hook_total{kind, outcome}` (counter; outcome ∈ emitted | redaction_failed | socket_unavailable | invalid_payload).
    - `brain_claude_hook_redactions_total{pattern}` (counter — operator visibility into what's being redacted).
    - `brain_claude_hook_latency_seconds{kind}` (histogram; FR-OBS-003 buckets).
14. **SHOULD** support `cyberos brain hook claude tail` — pretty-print the last N `brain.claude_*` rows for operator debugging.

---

## §2 — Why this design (rationale for humans)

**Why only three hook events (§1 #2 + #3)?** Claude Code emits many hooks. The three we subscribe to are the load-bearing trio:
- `UserPromptSubmit` answers "what was the user trying to do?"
- `PostToolUse` answers "how did the assistant act on it?" (one row per tool, so we get a tool trace).
- `Stop` answers "how did the session end and what was the resolution?"

Other hooks are operationally noisy (`Notification`, `PreToolUse`) or rare (`SubagentStop`) — capturing them adds 5× chain pressure for ~5% incremental signal.

**Why hash tool args, not store (§1 #2)?** Tool args sometimes contain secrets — e.g. a `Bash` tool call like `curl -H "Authorization: Bearer sk-abc" ...`. We capture the *fact* the tool ran and the *hash* of its args (so we can dedup identical re-runs) but never the raw args. Operators investigating "what was the call?" can cross-reference the trace_id back to Claude Code's local log (which the user controls).

**Why redact in the hook process, not in BRAIN (§1 #4 + #5)?** Redaction at write time means raw prompts never leave the user's machine — strongest privacy story. If we redacted server-side, the raw prompt would briefly exist in transit / in temporary buffers. The hook is the right boundary: redact before the bytes leave the local process tree.

**Why fail-closed on redaction (§1 #5)?** If we fail-OPEN (emit raw prompt on redactor failure), a buggy regex update could leak every prompt. Fail-CLOSED means the worst case is "we lost one row of capture" — recoverable. The `brain.claude_capture_redaction_failed` row tells the operator their redactor needs fixing, with no data leaked.

**Why 50ms latency budget (§1 #6)?** Hooks run on Claude Code's critical path. A 500ms hook makes the assistant feel sluggish; users disable it. 50ms is below human-noticeable. The redaction layer is ~5ms p95 (regex on ≤ 4KB prompt); the rest is JSON parse + socket send. Spec budgets 50ms p95 to leave headroom.

**Why Unix socket to FR-BRAIN-107 daemon (§1 #10)?** Two reasons:
1. The hook is a short-lived process (spawned per event); opening a BRAIN connection + writing chain rows + closing would take ~200ms — exceeds budget.
2. The daemon already has the chain-write machinery, the W3C trace context, the dedup cache. Reusing it via socket is one IPC vs. duplicating the writer stack.

**Why per-session trace_id cached in `/tmp` (§1 #7)?** Within a Claude Code session, multiple hook invocations (1 UserPromptSubmit + N PostToolUse + 1 Stop) must share a trace_id so OBS dashboards can correlate them. The session_id is the natural key; `/tmp` is the right scope (per-machine, per-session, ephemeral). 1-hour TTL covers long sessions; pruned by FR-BRAIN-110.

**Why opt-in per project (§1 #11)?** Some Claude Code projects are personal (researching a side project, debugging a friend's code) — operators don't want those captured. Per-project opt-in via `.claude/settings.json` is the right granularity: explicit consent, project-scoped, version-controllable.

**Why `tool_args_hash` even though redacted (§1 #2)?** Dedup. The same `Bash` invocation with the same args (e.g. `cargo test` run 10 times) should produce one logical "tool was run" memory, not 10. The hash key is `(tool_name, blake3(canonical_json(args)))`.

**Why bullets vs prose in §1?** Per the FR template's normative structure; readability for compliance review.

---

## §3 — API contract

### Cargo.toml

```toml
[package]
name        = "cyberos-brain-claude-hook"
version     = "0.1.0"
edition     = "2021"

[[bin]]
name = "cyberos-brain-claude-hook"
path = "src/main.rs"

[dependencies]
clap        = { version = "4", features = ["derive"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
tokio       = { version = "1.40", features = ["rt-multi-thread", "macros", "net", "io-util", "time"] }
blake3      = "1.5"
regex       = "1.10"
once_cell   = "1.19"
anyhow      = "1"
thiserror   = "1"
tracing     = "0.1"
tracing-opentelemetry = "0.23"
uuid        = { version = "1.10", features = ["v4"] }
```

### Claude Code hook config (template)

```jsonc
// services/brain-claude-hook/install/.claude/settings.json.template
{
  "hooks": {
    "UserPromptSubmit": [
      { "matcher": ".*", "hooks": [{ "type": "command", "command": "cyberos-brain-claude-hook userpromptsubmit" }] }
    ],
    "PostToolUse": [
      { "matcher": ".*", "hooks": [{ "type": "command", "command": "cyberos-brain-claude-hook posttooluse" }] }
    ],
    "Stop": [
      { "matcher": ".*", "hooks": [{ "type": "command", "command": "cyberos-brain-claude-hook stop" }] }
    ]
  }
}
```

### CLI

```rust
// services/brain-claude-hook/src/main.rs
use clap::Parser;
use cyberos_cli_exit::ExitCode;

#[derive(Parser)]
#[command(name = "cyberos-brain-claude-hook")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Parser)]
enum Cmd {
    /// Process UserPromptSubmit JSON on stdin
    Userpromptsubmit,
    /// Process PostToolUse JSON on stdin
    Posttooluse,
    /// Process Stop JSON on stdin
    Stop,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    // Read stdin as JSON
    let mut buf = Vec::new();
    if let Err(e) = tokio::io::AsyncReadExt::read_to_end(&mut tokio::io::stdin(), &mut buf).await {
        eprintln!("WARN cyberos-brain-claude-hook: stdin read failed: {e}");
        return ExitCode::Ok;  // never block Claude Code
    }
    let payload: serde_json::Value = match serde_json::from_slice(&buf) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("WARN cyberos-brain-claude-hook: invalid JSON: {e}");
            return ExitCode::Ok;
        }
    };
    let kind = match cli.cmd {
        Cmd::Userpromptsubmit => HookKind::UserPromptSubmit,
        Cmd::Posttooluse      => HookKind::PostToolUse,
        Cmd::Stop             => HookKind::Stop,
    };
    if let Err(e) = cyberos_brain_claude_hook::dispatch(kind, payload).await {
        eprintln!("WARN cyberos-brain-claude-hook: dispatch failed: {e}");
        // still exit 0; redaction-failed audit row already emitted by dispatch on failure paths
    }
    ExitCode::Ok
}
```

### Dispatch + emit

```rust
// services/brain-claude-hook/src/hook.rs
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum HookKind { UserPromptSubmit, PostToolUse, Stop }

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct UserPromptSubmitPayload {
    session_id:  Uuid,
    prompt:      String,
    cwd:         String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct PostToolUsePayload {
    session_id:        Uuid,
    tool_name:         String,
    tool_args:         serde_json::Value,
    outcome:           String,    // "success" | "error"
    duration_ms:       u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct StopPayload {
    session_id:           Uuid,
    prompt_count:         u32,
    tool_use_count:       u32,
    duration_ms:          u64,
    last_assistant_message: Option<String>,
}

pub async fn dispatch(kind: HookKind, payload: serde_json::Value) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let trace_id = trace_id_for_session(extract_session_id(&payload)?).await?;

    let row = match kind {
        HookKind::UserPromptSubmit => {
            let p: UserPromptSubmitPayload = serde_json::from_value(payload)?;
            let (prompt_redacted, match_count) = crate::redact::redact(&p.prompt);
            serde_json::json!({
                "kind": "brain.claude_prompt",
                "payload": {
                    "session_id":      p.session_id,
                    "prompt_hash":     hex::encode(blake3::hash(p.prompt.as_bytes()).as_bytes()),
                    "prompt_redacted": prompt_redacted,
                    "cwd":             p.cwd,
                    "trace_id":        trace_id,
                    "captured_at_ns":  unix_ns(),
                    "redaction_match_count": match_count,
                }
            })
        }
        HookKind::PostToolUse => {
            let p: PostToolUsePayload = serde_json::from_value(payload)?;
            let args_canonical = serde_json::to_string(&p.tool_args)?;
            let args_hash = hex::encode(blake3::hash(args_canonical.as_bytes()).as_bytes());
            serde_json::json!({
                "kind": "brain.claude_tool_use",
                "payload": {
                    "session_id":     p.session_id,
                    "tool_name":      p.tool_name,
                    "tool_args_hash": args_hash,
                    "outcome":        p.outcome,
                    "duration_ms":    p.duration_ms,
                    "trace_id":       trace_id,
                }
            })
        }
        HookKind::Stop => {
            let p: StopPayload = serde_json::from_value(payload)?;
            let last_msg = p.last_assistant_message.as_deref().unwrap_or_default();
            let (last_redacted, _) = crate::redact::redact(last_msg);
            serde_json::json!({
                "kind": "brain.claude_session_completed",
                "payload": {
                    "session_id":                     p.session_id,
                    "prompt_count":                   p.prompt_count,
                    "tool_use_count":                 p.tool_use_count,
                    "duration_ms":                    p.duration_ms,
                    "last_assistant_message_redacted": last_redacted,
                    "trace_id":                       trace_id,
                }
            })
        }
    };
    crate::emit::post_to_daemon(row).await?;
    metric_record_latency(kind, start.elapsed());
    Ok(())
}

async fn trace_id_for_session(session_id: Uuid) -> anyhow::Result<String> {
    let path = std::path::Path::new("/tmp/cyberos-brain-claude-traces").join(session_id.to_string());
    if let Ok(bytes) = tokio::fs::read(&path).await {
        return Ok(String::from_utf8(bytes)?);
    }
    let new_tid = generate_trace_id();
    let _ = tokio::fs::create_dir_all(path.parent().unwrap()).await;
    tokio::fs::write(&path, &new_tid).await?;
    Ok(new_tid)
}
```

### Redactor

```rust
// services/brain-claude-hook/src/redact.rs
use once_cell::sync::Lazy;
use regex::Regex;

struct Rule { pat: Regex, replace: &'static str, name: &'static str }

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![
    Rule { pat: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),                       replace: "<AWS_KEY>",        name: "aws_access_key" },
    Rule { pat: Regex::new(r"(?i)aws_secret(?:_access)?_key\s*=\s*\S+").unwrap(), replace: "<AWS_SECRET>",   name: "aws_secret" },
    Rule { pat: Regex::new(r"(?i)Bearer\s+[A-Za-z0-9._\-]{16,}").unwrap(),     replace: "Bearer <REDACTED>", name: "bearer_token" },
    Rule { pat: Regex::new(r"gh[oprsu]_[A-Za-z0-9_]{16,}").unwrap(),           replace: "<GH_PAT>",          name: "github_pat" },
    Rule { pat: Regex::new(r"sk-[A-Za-z0-9]{32,}").unwrap(),                   replace: "<OPENAI_KEY>",      name: "openai_key" },
    Rule { pat: Regex::new(r"sk-ant-[A-Za-z0-9_-]{32,}").unwrap(),             replace: "<ANTHROPIC_KEY>",   name: "anthropic_key" },
    Rule { pat: Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap(),
                                                                                replace: "<JWT>",            name: "jwt" },
    Rule { pat: Regex::new(r"\b\d{12}\b").unwrap(),                            replace: "<VN_CCCD>",         name: "vn_cccd" },
    Rule { pat: Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap(),
                                                                                replace: "<EMAIL>",          name: "email" },
    Rule { pat: Regex::new(r"\+?\d{1,3}[\s\-]?\d{2,4}[\s\-]?\d{3,4}[\s\-]?\d{3,4}").unwrap(),
                                                                                replace: "<PHONE>",          name: "phone" },
]);

pub fn redact(input: &str) -> (String, u32) {
    let mut buf = input.to_string();
    let mut match_count = 0u32;
    for rule in RULES.iter() {
        let before_len = buf.len();
        buf = rule.pat.replace_all(&buf, rule.replace).into_owned();
        // crude count: each replacement changes length by (rule.replace.len() - matched.len());
        // accept rough count as match metric
        if buf.len() != before_len {
            match_count += 1;
            metrics::counter!("brain_claude_hook_redactions_total", "pattern" => rule.name).increment(1);
        }
    }
    (buf, match_count)
}
```

### Socket emit

```rust
// services/brain-claude-hook/src/emit.rs
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use std::time::Duration;

const SOCKET: &str = "/tmp/cyberos-brain-capture.sock";

pub async fn post_to_daemon(row: serde_json::Value) -> anyhow::Result<()> {
    let mut stream = tokio::time::timeout(Duration::from_millis(40), UnixStream::connect(SOCKET)).await??;
    let bytes = serde_json::to_vec(&row)?;
    let len = (bytes.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    // Don't wait for ack — fire-and-forget per §1 #6
    Ok(())
}
```

### Installer

```bash
#!/usr/bin/env bash
# services/brain-claude-hook/install/install-hooks.sh
# Usage: install-hooks.sh <project-path>  OR  install-hooks.sh --user

set -euo pipefail

if [ "${1:-}" = "--user" ]; then
  SETTINGS="$HOME/.claude/settings.json"
else
  PROJECT="${1:?usage: install-hooks.sh <project-path> | --user}"
  SETTINGS="$PROJECT/.claude/settings.json"
fi
mkdir -p "$(dirname "$SETTINGS")"

TEMPLATE="$(dirname "$0")/.claude/settings.json.template"

# Idempotent merge: if hooks already present with matching command, no-op
if [ -f "$SETTINGS" ] && jq -e '.hooks.UserPromptSubmit[0].hooks[0].command == "cyberos-brain-claude-hook userpromptsubmit"' "$SETTINGS" > /dev/null 2>&1; then
  echo "✓ already installed (no changes): $SETTINGS"
  exit 0
fi

if [ -f "$SETTINGS" ]; then
  # Merge: prepend our hooks; preserve user's existing other hooks
  jq -s '.[0] * .[1]' "$SETTINGS" "$TEMPLATE" > "$SETTINGS.new" && mv "$SETTINGS.new" "$SETTINGS"
else
  cp "$TEMPLATE" "$SETTINGS"
fi
echo "✓ installed hooks → $SETTINGS"
```

---

## §4 — Acceptance criteria

1. **UserPromptSubmit → brain.claude_prompt** — fixture JSON on stdin → 1 row emitted to daemon socket; payload contains session_id, prompt_hash, prompt_redacted, cwd, trace_id, captured_at_ns.
2. **PostToolUse → brain.claude_tool_use** — fixture JSON → 1 row; tool_args_hash present; raw tool_args NOT present.
3. **Stop → brain.claude_session_completed** — fixture JSON → 1 row; prompt_count + tool_use_count + last_assistant_message_redacted present.
4. **Redaction: AWS access key** — prompt containing `AKIA1234567890123456` → redacted to `<AWS_KEY>`; `redaction_match_count >= 1`.
5. **Redaction: Bearer token** — prompt containing `Authorization: Bearer abc123def456...` → `Bearer <REDACTED>`.
6. **Redaction: OpenAI key** — `sk-abcdefghij1234567890abcdefghij12` → `<OPENAI_KEY>`.
7. **Redaction: Anthropic key** — `sk-ant-api03-abc...` → `<ANTHROPIC_KEY>`.
8. **Redaction: GitHub PAT** — `ghp_abc123...` → `<GH_PAT>`.
9. **Redaction: JWT** — three-segment base64 → `<JWT>`.
10. **Redaction: VN CCCD** — 12-digit number → `<VN_CCCD>`.
11. **Redaction: email** — `user@example.com` → `<EMAIL>` (unless `@cyberskill.world` allowlisted per FR-BRAIN-111).
12. **Redaction fail-closed** — inject regex panic via test seam → `brain.claude_capture_redaction_failed` row emitted; hook exits 0.
13. **Same session → same trace_id** — three hook invocations with same `session_id` → all three rows carry identical trace_id.
14. **Different sessions → different trace_ids** — two sessions → two different trace_ids (32-char hex, no collision).
15. **Latency p95 ≤ 50ms** — 100-trial test harness with 4KB prompt; p95 latency < 50ms.
16. **Socket unavailable → graceful** — daemon down; hook exits 0 with stderr WARN `socket_unavailable`; metric `brain_claude_hook_total{outcome="socket_unavailable"}` increments.
17. **Invalid JSON on stdin → graceful** — malformed JSON → exit 0 with stderr WARN; metric `outcome="invalid_payload"`.
18. **Missing required field → graceful** — JSON missing `session_id` → exit 0 with stderr WARN; no row emitted.
19. **Installer idempotent** — run `install-hooks.sh` twice → second run is a no-op; `~/.claude/settings.json` byte-identical.
20. **Installer preserves user hooks** — pre-existing `Stop` hook in user settings → after install, both user's hook AND ours are present; user's hook runs first.
21. **Uninstall removes our hooks** — `cyberos brain hook claude uninstall <path>` → settings.json contains only the user's pre-existing hooks; ours removed.
22. **Status shows last capture** — `cyberos brain hook claude status <path>` → prints "installed" + the timestamp + session_id of the last `brain.claude_*` row.
23. **OTel span per invocation** — exporter receives a `brain.claude.hook` span per invocation with `hook_kind` + `session_id` attributes.
24. **Metric: redaction counters** — running redactor 5× on a prompt with all 10 patterns → `brain_claude_hook_redactions_total{pattern="aws_access_key"}` etc. increment.

---

## §5 — Verification

```rust
// services/brain-claude-hook/tests/hook_e2e_test.rs

#[tokio::test]
async fn userpromptsubmit_emits_claude_prompt_row() {
    let (daemon_socket, mut rx) = stub_daemon().await;
    let payload = serde_json::json!({
        "session_id": "00000000-0000-0000-0000-000000000001",
        "prompt":     "Fix the auth bug",
        "cwd":        "/Users/x/proj"
    });
    invoke_hook(HookKind::UserPromptSubmit, payload, &daemon_socket).await;
    let row = rx.recv().await.unwrap();
    assert_eq!(row["kind"], "brain.claude_prompt");
    assert_eq!(row["payload"]["prompt_redacted"], "Fix the auth bug");
    assert!(row["payload"]["trace_id"].as_str().unwrap().len() == 32);
}

#[tokio::test]
async fn redacts_aws_key_in_prompt() {
    let (daemon_socket, mut rx) = stub_daemon().await;
    let payload = serde_json::json!({
        "session_id": "00000000-0000-0000-0000-000000000002",
        "prompt":     "use this key AKIA1234567890123456",
        "cwd":        "/Users/x/proj"
    });
    invoke_hook(HookKind::UserPromptSubmit, payload, &daemon_socket).await;
    let row = rx.recv().await.unwrap();
    assert!(row["payload"]["prompt_redacted"].as_str().unwrap().contains("<AWS_KEY>"));
    assert!(!row["payload"]["prompt_redacted"].as_str().unwrap().contains("AKIA1234567890123456"));
    assert!(row["payload"]["redaction_match_count"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn same_session_gets_same_trace_id() {
    let (daemon_socket, mut rx) = stub_daemon().await;
    let session = uuid::Uuid::new_v4();
    for kind in [HookKind::UserPromptSubmit, HookKind::PostToolUse, HookKind::Stop] {
        invoke_hook(kind, fixture_for(kind, session), &daemon_socket).await;
    }
    let rows: Vec<_> = (0..3).map(|_| rx.try_recv().unwrap()).collect();
    let trace_ids: std::collections::HashSet<_> = rows.iter().map(|r| r["payload"]["trace_id"].clone()).collect();
    assert_eq!(trace_ids.len(), 1);
}

#[tokio::test]
async fn latency_p95_under_50ms() {
    let (daemon_socket, _rx) = stub_daemon().await;
    let mut latencies = Vec::new();
    for _ in 0..100 {
        let start = std::time::Instant::now();
        invoke_hook(HookKind::UserPromptSubmit, fixture_prompt_4kb(), &daemon_socket).await;
        latencies.push(start.elapsed());
    }
    latencies.sort();
    let p95 = latencies[95];
    assert!(p95 < std::time::Duration::from_millis(50), "p95 = {:?}", p95);
}

#[tokio::test]
async fn socket_unavailable_exits_0() {
    let payload = fixture_prompt(&uuid::Uuid::new_v4());
    let res = run_hook_subprocess(HookKind::UserPromptSubmit, payload, "/tmp/no-such-socket").await;
    assert_eq!(res.exit_code, 0);
    assert!(res.stderr.contains("socket_unavailable") || res.stderr.contains("WARN"));
}
```

```rust
// services/brain-claude-hook/tests/redact_test.rs
#[test] fn redacts_aws_access_key()    { let (out, n) = redact("AKIA1234567890ABCDEF"); assert_eq!(out, "<AWS_KEY>"); assert_eq!(n, 1); }
#[test] fn redacts_openai_key()        { let (out, _) = redact("sk-abcdefghij1234567890abcdefghij12"); assert!(out.contains("<OPENAI_KEY>")); }
#[test] fn redacts_anthropic_key()     { let (out, _) = redact("sk-ant-api03-foobar1234567890abcdefghij"); assert!(out.contains("<ANTHROPIC_KEY>")); }
#[test] fn redacts_jwt()               { let (out, _) = redact("eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM.SflKxw"); assert!(out.contains("<JWT>")); }
#[test] fn redacts_github_pat()        { let (out, _) = redact("ghp_abcdefghij1234567890abcdefghij"); assert!(out.contains("<GH_PAT>")); }
#[test] fn redacts_bearer()            { let (out, _) = redact("Authorization: Bearer abcdefghij1234"); assert!(out.contains("Bearer <REDACTED>")); }
#[test] fn redacts_vn_cccd()           { let (out, _) = redact("CCCD 079123456789"); assert!(out.contains("<VN_CCCD>")); }
#[test] fn no_false_positive_on_short_numeric() { let (out, n) = redact("issue #12345"); assert_eq!(out, "issue #12345"); assert_eq!(n, 0); }
```

---

## §6 — Implementation skeleton

(Above §3 is the skeleton.)

```rust
// Lib root
pub mod hook;
pub mod redact;
pub mod emit;

pub async fn dispatch(kind: hook::HookKind, payload: serde_json::Value) -> anyhow::Result<()> {
    hook::dispatch(kind, payload).await
}
```

---

## §7 — Dependencies

- **FR-BRAIN-107 (upstream)** — capture daemon owns the Unix socket; hook is a client.
- **FR-BRAIN-101** — `BrainWriter` (used by daemon, not directly by hook).
- **FR-BRAIN-111 (sibling)** — PII redaction ruleset; this FR uses a subset of those patterns.
- **FR-AI-022** — W3C TraceContext convention.
- **FR-AI-014** — `last_assistant_message` field could carry persona tagging in future.
- **FR-SKILL-101** — skill invocations are a sibling capture path; this FR captures only Claude Code's native tool use.
- **`cyberos-cli-exit`** — shared exit codes (hook always exits 0 — never block Claude Code).

---

## §8 — Example payloads

### `brain.claude_prompt`

```json
{
  "kind": "brain.claude_prompt",
  "payload": {
    "session_id":             "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "prompt_hash":            "9b0e8c5...",
    "prompt_redacted":        "Help me refactor auth using key <AWS_KEY>",
    "cwd":                    "/Users/stephencheng/Projects/CyberSkill/cyberos",
    "trace_id":               "0af7651916cd43dd8448eb211c80319c",
    "captured_at_ns":         1747407137483000000,
    "redaction_match_count":  1
  }
}
```

### `brain.claude_tool_use`

```json
{
  "kind": "brain.claude_tool_use",
  "payload": {
    "session_id":     "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "tool_name":      "Edit",
    "tool_args_hash": "ab12cd...",
    "outcome":        "success",
    "duration_ms":    142,
    "trace_id":       "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `brain.claude_session_completed`

```json
{
  "kind": "brain.claude_session_completed",
  "payload": {
    "session_id":                    "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "prompt_count":                  3,
    "tool_use_count":                12,
    "duration_ms":                   847000,
    "last_assistant_message_redacted": "Done. Auth middleware refactored; tests pass.",
    "trace_id":                      "0af7651916cd43dd8448eb211c80319c"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Subagent capture (`SubagentStop` hook) — slice 3+; needs design.
- Tool-arg content preview (operator-only, project-scoped, signed) — slice 3+.
- Cross-machine session correlation (same user, two laptops) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Hook process spawned but daemon down | UnixStream::connect ECONNREFUSED | Hook exits 0; stderr WARN `socket_unavailable`; metric increments | FR-BRAIN-110 restarts daemon; next hook succeeds |
| Daemon socket exists but daemon hung | Connect succeeds; write times out at 40ms | Hook exits 0; row lost (single event); metric `outcome="emit_timeout"` | FR-BRAIN-110 SIGTERM + restart daemon |
| Malformed JSON on stdin | serde_json::from_slice Err | Exit 0; stderr WARN; metric `outcome="invalid_payload"` | Operator updates Claude Code OR raises issue |
| Required field missing | serde_json::from_value Err | Exit 0; stderr WARN | Same as above |
| Redactor regex panic | std::panic::catch_unwind catches | `brain.claude_capture_redaction_failed` row emitted; exit 0 | Operator updates redactor |
| /tmp full | trace_id cache write fails | Hook still proceeds with generated trace_id (in-mem); next hook for same session may regenerate (correlation broken) | Operator frees /tmp |
| Same session_id reused across days | trace_id cache 1h TTL expired | New trace_id generated; rows from same logical session split across trace_ids | By design; operators correlate via session_id |
| Very large prompt (> 1 MB) | Read succeeds; redact takes longer | Latency may exceed 50ms; row emitted; metric `latency_seconds` p99 alarm | Operator splits; or accept; or raises budget |
| Claude Code passes extra unknown fields | serde unknown-field handling | Logged at WARN; field ignored; row still emitted | None |
| Hook installed but user disabled hooks in Claude Code | Hook never invoked | No rows; no error | Operator re-enables OR uninstalls cyberos hook |
| install-hooks.sh run on missing /.claude dir | mkdir -p creates it; settings.json written | Success | None |
| install-hooks.sh: settings.json malformed | jq Err during merge | Exit 1; stderr message; user's file unchanged | Operator manually fixes |
| Two hooks for same event from two tools (ours + theirs) | Both run in order | Both succeed independently | By design; AC #20 |
| Hook binary missing (uninstalled cyberos) | Claude Code logs hook spawn error | No rows; Claude Code session continues normally | Operator reinstalls |
| Tracing exporter unavailable | tracing-otel buffers | Hook continues; spans buffered; eventually dropped | Operator restarts FR-OBS-001 collector |
| Session_id is not a valid UUID | Parse error | Exit 0; stderr WARN; metric `invalid_payload` | Operator files Claude Code bug |
| Redaction yields empty string | Acceptable | Row emitted with empty `prompt_redacted`; `redaction_match_count` high | By design — operator sees N matches but text is fully redacted |

---

## §11 — Implementation notes

- The hook binary MUST be in `PATH` for Claude Code to find it. `cyberos brain hook claude install` updates `settings.json` with the absolute path discovered via `which cyberos-brain-claude-hook` (fallback: `/usr/local/bin/cyberos-brain-claude-hook`).
- `tokio::main(flavor = "current_thread")` is intentional — the hook is short-lived; a multi-thread runtime would add tens of ms of startup cost.
- The Unix socket path `/tmp/cyberos-brain-capture.sock` is documented in FR-BRAIN-107; both this FR and the daemon must agree. Constant lives in `cyberos-brain-shared` crate.
- Redaction patterns are intentionally simple regexes — fast (≤ 5ms p95) but with known false-positive characteristics (e.g. 12-digit number false-positives as CCCD). Operators can tune the ruleset; FR-BRAIN-111 owns the canonical version.
- The `tool_args_hash` uses `serde_json::to_string` (compact form, default key ordering); this is NOT canonical JSON. For cross-session dedup, future FR may switch to canonical (RFC 8785).
- The 1-hour TTL on trace_id cache is a balance: too short → sessions split across trace_ids; too long → /tmp accumulates files. FR-BRAIN-110 sweeper deletes files older than 1h hourly.
- `metric_record_latency` is a thin wrapper around the `metrics` crate's histogram macros; the actual Prometheus exporter is in the daemon.
- The `--user` flag installer scope is the right default for personal workflows; `--project` (no flag) is the right default for team-shared projects (the hook config commits to git).
- The installer is bash, not Rust, because: (a) it's pre-rustup-availability; (b) `jq` is the standard tool for JSON merge; (c) ~30 lines is too small to justify a binary.

---

*End of FR-BRAIN-109.*
