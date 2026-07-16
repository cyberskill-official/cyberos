---
artefact: observability-injection@1
task_id: TASK-IMP-082
branch_coverage_estimate: 100
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - pure render tooling, no runtime service surface; every failure branch already announces)
---
# Observability injection - TASK-IMP-082

Honest vacuity statement: this task adds no service, no daemon, no network path, and no new
failure branch that could go silent. The change swaps the derivation of one constant inside a
run-to-completion renderer whose entire observable surface IS its output:

- The stamp itself is the observability. It is printed on three user-visible surfaces (header
  meta render-status-hub.mjs:454, footer :463, cs-data JSON `commit` field :400) and is now a
  content-addressed fingerprint - `fp-` prefixed precisely so a human or script can tell at a
  glance whether a page carries a fingerprint or a pinned sha. Freshness checking becomes
  greppable: audit-fleet.sh:174-181 already extracts the recorded stamp and re-renders against
  it; with a fingerprint that check stops false-positiving on unchanged corpora.
- Failure branches all pre-exist and all announce: missing docs/tasks, missing CHANGELOG,
  missing VERSION, broken frontmatter die() with the offending path (strict) or WARN to stderr
  (lenient); zero releases exits 1 naming CHANGELOG.md. The fingerprint path adds no branch
  that can fail separately - it reads exactly the files the renderer already read (a vanished
  file between discovery and hashing would throw node's own ENOENT loudly, non-zero exit).
- No logging added, deliberately: the renderer's summary line (:485-495) is the wrapper's
  (task-migrate.sh) progress surface and the determinism doctrine forbids wall-clock or
  environment noise in output. A per-render "fingerprint=" log line would be redundant with
  the page bytes and would tempt someone to parse logs instead of the page.
- The regression suite is the standing detector: scripts/tests/test_render_stamp.sh t01-t06
  runs under scripts/tests/run_all.sh's glob on every gate, so a stamp regression (chase
  returning, pin breaking, git creeping back in) is caught by a named, counted test rather
  than by a human noticing `git status` churn - which was exactly the failure mode this task
  retires (IMPROVEMENT_HANDOFF.md IMP-01).

PII: none - the hash consumes task specs, CHANGELOG, VERSION already published on the page;
the stamp is a 12-hex digest, not reversible content. No redaction policy needed.
