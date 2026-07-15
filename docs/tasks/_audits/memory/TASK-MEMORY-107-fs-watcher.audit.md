---
task_id: TASK-MEMORY-107
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

TASK-MEMORY-107 written direct-to-10/10 cadence. ~870 lines. 17 §1 clauses (watcher backend, debounce, dedup, rate limit, queue, overflow handling, trace propagation, globs, 4 row kinds, resync, doctor gate, OTel, SIGHUP, fg + dry-run modes). 11 §2 rationale paragraphs. Full Cargo.toml + watcher + dedup cache + rate limiter + emit + main.rs in §3. 22 ACs. 5 Rust e2e tests + 3 dedup unit tests. 18 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Dedup vs rename ambiguity
First-pass would have emitted two `memory.capture_created` rows for `mv foo.txt bar.txt` because the watcher reports Created at bar.txt. Resolved: §1 #4 + DedupVerdict::EmitRenamed via content-hash cache hit at different path; AC #5 + §5 test.

### ISS-002 — Queue overflow silent loss
A bounded queue that drops on overflow without audit = silent data loss. Resolved: §1 #6 + #7 + `memory.capture_dropped` row with `reason: queue_overflow`; AC #11 + §5 rate-limit-drop test.

### ISS-003 — Startup resync semantics missing
Daemon down, files change, daemon up — without resync the changes are lost from memory's perspective. Resolved: §1 #11 + #12 + start/completed audit rows + 60s budget; AC #14 + #15 + §5 startup_resync_catches_up test.

### ISS-004 — SIGHUP reload behaviour unspecified
Add/remove watched folders without restart is critical UX. Resolved: §1 #15 + `daemon.reload()` API + AC #16 + #17; §11 note explains the diff algorithm.

### ISS-005 — W3C trace propagation source ambiguous
Where does the trace_id come from — env var? Generated per event? Per batch? Resolved: §1 #8 + §3 `trace::current_trace_id()`; AC #12 + #13 cover both env-provided and generated paths.

### ISS-006 — Doctor gate not enforced at boot
Without the gate, daemon would start on a broken manifest and fail at first event. Resolved: §1 #13 + main.rs `run_doctor_gate()` call + AC #19 (refuses to start with exit 7 InternalError + stderr).

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of TASK-MEMORY-107 audit.*
