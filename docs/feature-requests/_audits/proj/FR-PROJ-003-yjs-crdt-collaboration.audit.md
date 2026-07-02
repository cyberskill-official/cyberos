---
fr_id: FR-PROJ-003
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
---

## §1 — Verdict summary

FR-PROJ-003 authored direct-to-10/10. ~920 lines. 13 §1 clauses (Y.Doc structure, snapshot semantics, restore, server-as-relay, JWT + RLS auth, scalar LWW with companion columns, awareness throttle, offline buffer, memory audit rows, W3C trace propagation, OTel metrics, Postgres-down degradation, write-path lock-out). 11 §2 rationale paragraphs. Full SQL migration + Rust relay + snapshot scheduler + TypeScript client + LWW handler in §3. 25 ACs. 5 Rust integration tests + 2 TypeScript tests. 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — CRDT scope (everything vs just rich-text)
Using Y.Map for scalars adds 50+ bytes overhead per 12-byte value. Resolved: §1 #1 + DEC-241 explicit split — Y.Text + Y.Array("comments") for rich content; plain Postgres + LWW for scalars; §2 rationale explains the tradeoff.

### ISS-002 — Server-side merge vs relay-only
Server-merge introduces a second authority that may diverge from clients. Resolved: §1 #4 + DEC-243 relay-only; clients converge via Yjs internal semantics; server persists + replicates.

### ISS-003 — Snapshot frequency tradeoff
Per-edit snapshots = write storm; once-daily = slow replay. Resolved: §1 #2 + DEC-242 every 60s; 50-snapshot cap; AC #6 + #9 verify cadence + pruning.

### ISS-004 — LWW tie-break determinism
Equal timestamps without deterministic tie-break = race. Resolved: §1 #6 lexicographic on subject_id; AC #5 verifies.

### ISS-005 — Reconnect replay performance
Naive: replay full update log = 50k operations × 10s for 4-hour offline session. Resolved: §1 #3 snapshot + binlog-since-snapshot; AC #8 verifies; §11 SQL prune query for snapshot retention.

### ISS-006 — Direct description writes bypass CRDT
A naive REST `PATCH /issues/:id description=X` would lose concurrent edits. Resolved: §1 #13 + §11 the canonical `issues` table HAS NO description column; description materialised from latest snapshot via view; AC #25 verifies the absence of a direct path.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-PROJ-003 audit.*
