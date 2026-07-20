---
artefact: implementation-plan@1
task_id: TASK-IMP-082
created: 2026-07-16
estimate_pts: 2
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 3)
---
# Implementation plan - TASK-IMP-082

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Fingerprint derivation in tools/docs-site/render-status-hub.mjs - collect every discovered spec.md into specFiles during the existing discovery loop (:128), then corpusFingerprint(): sha256 streamed per file in Buffer.compare-sorted repo-relative order, then CHANGELOG.md, then VERSION, when present; 'fp-' + first 12 hex (§1.1, §1.6; rows 1, 2, 4, 5, 7).
2. Default swap + pin preserved - `const COMMIT = process.env.CYBEROS_COMMIT || corpusFingerprint()`; gitCommit() removed (unused - the default path must not read git state at all); the :304-306 comment rewritten to describe fingerprint semantics in the file's voice; header line 8 stamp claim updated (§1.2, §1.4, §1.6; rows 3, 8, 10). The three surfaces (:400, :454, :463) already read COMMIT - verified no other derivation exists (§1.7).
3. scripts/tests/test_render_stamp.sh - t01-t06 named per the spec's ACs, harness style of the scripts/tests peers, fixtures in the renderer's discovery shape, invocation per the task-migrate.sh contract; lands in the run_all glob with zero wiring (§1.3, §1.5, §1.8; every matrix row's "covered by").
4. Peer-suite compatibility - the two assertions that pinned the planted fake-HEAD sha (test_render_status_hub.sh:32, test_render_roadmap.sh:36) updated minimally to the fp- grammar; recorded in code-review.md (§1.1 ripple; no production surface).

Pattern conformance: node stdlib only (node:crypto joins node:fs/path/url), no child processes, no wall clock, honest failures untouched; bash suite mirrors peer counters/summary/exit contract. No new dependencies. Out of scope honored: hooks, lenses, chunks, KPI math, freshness CLI untouched.

Estimate: 2 pts (~3 h) - matches spec effort_hours: 3. Actual: renderer diff +24/-20 lines across 3 files, one new 143-line suite, five artefacts.
