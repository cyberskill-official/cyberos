---
task_id: TASK-SKILL-105
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-SKILL-105 authored direct-to-10/10. ~810 lines. 15 §1 clauses (single bundle, frontmatter, Rust+Python+bash APIs, kind regex, dedup, PII pre-scrub, broker socket discovery, broker-down retry, trace propagation, OTel, signing, TS deferred). 9 §2 rationale paragraphs. Full SKILL.md + Rust SDK + Python wrapper + bash CLI in §3. 23 ACs. 5 Rust + 2 Python + 3 bash tests. 20 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Multi-language SDK distribution complexity
Rust + Python + TS + bash could mean 4 separate crates that drift. Resolved: §1 #1 single bundle; all frontends route through one Rust core; bash is a thin wrapper around the binary; Python is subprocess shim; TS deferred (§1 #15).

### ISS-002 — Dedup scope (per-instance vs global)
Global dedup needs Redis/IPC; per-instance is simpler. Resolved: §1 #7 + §11 note per-instance with 60s TTL + LRU(1000); broker-side dedup is the defense-in-depth (TASK-MEMORY-107).

### ISS-003 — PII scrub: SDK-side vs broker-side
Both adds latency; only one risks gaps. Resolved: §1 #8 both — SDK pre-scrub for performance; broker TASK-MEMORY-111 as authoritative. `payload_sanitised=true` flag lets callers skip SDK-side scrub when payload is known-clean.

### ISS-004 — Broker-down handling: block vs surrender
Blocking indefinitely is hostile to callers; immediate failure misses transient flaps. Resolved: §1 #10 exp backoff (100ms, 500ms, 2s) capped at 3 retries < 3 seconds total; AC #4 #5.

### ISS-005 — Python distribution: pyo3 vs subprocess
pyo3 = fast, but heavy distribution (per-platform wheels). Subprocess = slow (10ms/call), but pure-Python wheel. Resolved: §1 + §11 subprocess shim is the default; pyo3 deferred to slice 3+.

### ISS-006 — Kind regex enforcement vs convention
Without regex, kinds drift into chaos. Resolved: §1 #6 `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$` enforced at SDK; rejected before broker call; AC #3.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of TASK-SKILL-105 audit.*
