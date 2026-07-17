---
audited_file: docs/tasks/improvement/TASK-IMP-119-cone-drift-audit/spec.md
audited_file_sha256: 12efb3cc3e50bc20
audited_body_sha256_prefix: 2c5656f3fe074c26
rubric: audit_rubric@2.0
audited_at: 2026-07-17T17:45:00+07:00
auditor: claude-fable-5
verdict: pass
score: 10/10
machine_floor: task-lint 0 errors, 1 info (TRACE-001)
---

# Audit - TASK-IMP-119

Machine floor first per TASK-IMP-084: 0 errors. TRACE-001 info is the `## 1. Clauses` heading shape,
as with 117/118.

## Findings

ISS-001 (info, accepted): TRACE-001 heading shape. 1.1-1.7 each cite a test; AC1-AC7 each cite back.

ISS-002 (accepted, load-bearing): AC7 requires the tool to name EXACTLY the three files that escaped
on 2026-07-17 and no others. The cone data in §Problem is measured - the parent ran the diff against
both declared cones mechanically - so this AC is checkable against a real commit range, not a
fixture. A tool that cannot find the case that motivated it is decoration; this AC is what stops
that.

ISS-003 (accepted, and the sharpest choice here): 1.5 REPORTS rather than refuses. That looks weaker
than the fail-closed posture used everywhere else today (the version guard, relUnderRoot,
batch-select on undeclared cones). It is the right call for a different reason: all three of
2026-07-17`"'s escapes were CORRECT discoveries about WRONG specs. A hard block would have stopped
three good implementations to enforce three bad cones. §Alternatives records that a refusal is
revisitable once the escape RATE is known - and makes that a §11d question, which is honest about
what evidence the decision needs.

ISS-004 (accepted): §Alternatives rejects deriving the cone from the diff, on the grounds that a
cone which cannot be wrong makes batch-select"s
