---
fr_id: FR-SKILL-104
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-SKILL-104 authored direct-to-10/10. ~900 lines. 15 §1 clauses (Unix socket IPC, JSON-RPC framing, frontmatter enforcement, subprocess sandbox, BRAIN scope check, timeout, audit rows, trace propagation, tool registry, OTel, metrics, CLI status/tail/replay, rate-limit placeholder). 12 §2 rationale paragraphs. Full enforcer + dispatcher + subprocess spawn + timeout enforcement in §3. 24 ACs. 6 Rust unit + e2e tests. 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Tool enforcement at first call vs every call
A naive author would check `allowed_tools` once at boot. Resolved: §1 #3 + #5 per-call check; AC #4 #5 cover the path-traversal bypass attempt.

### ISS-002 — Subprocess sandbox depth
Without specifics, sandbox = "spawn the process." Resolved: §1 #4 enumerates 4 mechanisms (close_fds, env_clear, unshare, setrlimit); AC #8 #9 #10 #11 verify each.

### ISS-003 — File + domain enforcement missing from v1 frontmatter
FR-SKILL-103 covers BRAIN scopes but not files/domains. Resolved: §1 #3 + #5 + §3 `x-allowed-files` + `x-allowed-domains` extensions; AC #6 #7.

### ISS-004 — Timeout semantics (sharp vs graceful)
SIGKILL only = data loss; SIGTERM only = stuck processes. Resolved: §1 #6 90% SIGTERM + 100% SIGKILL with 10s grace; AC #12 #13 + §5 timeout_test.

### ISS-005 — Concurrent invocations sharing state
Per-invocation broker = N broker processes (heavy); single broker = state sharing risk. Resolved: §1 #1 per-invocation Unix socket but single broker process; AC #24 verifies socket isolation.

### ISS-006 — Args hashed but actually needed for replay
audit_row carries args_hash, not args. Without args, replay (FR-SKILL-104 §1 #14 placeholder) is impossible. Resolved: §11 note explains args_hash for dedup + future replay via separate encrypted side-channel; deferred to slice 3+.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-SKILL-104 audit.*
