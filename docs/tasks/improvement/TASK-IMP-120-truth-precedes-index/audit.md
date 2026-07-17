---
audited_file: docs/tasks/improvement/TASK-IMP-120-truth-precedes-index/spec.md
audited_file_sha256: 6d61ababa50e836c
audited_body_sha256_prefix: 4afefdf9a83c47ec
rubric: audit_rubric@2.0
audited_at: 2026-07-17T17:45:00+07:00
auditor: claude-fable-5
verdict: pass
score: 10/10
machine_floor: task-lint 0 errors, 1 info (TRACE-001)
---

# Audit - TASK-IMP-120

Machine floor first per TASK-IMP-084: 0 errors. TRACE-001 info is the `## 1. Clauses` heading shape,
as with 117/118/119.

## Findings

ISS-001 (info, accepted): TRACE-001 heading shape. 1.1-1.5 cite tests; AC1-AC6 cite back.

ISS-002 (accepted, and the reason this is p1): two instances in one day, opposite directions, two
different actors - the agent on TASK-IMP-116 and a swarm sub-agent on TASK-IMP-028. Both were caught
by a manual reconcile someone chose to run. The fix was already named in 116's goal file BEFORE the
second instance happened, which is why this is a task and not a discovery. A gap named and not
closed is a gap that recurs; it did, within hours.

ISS-003 (accepted): the Alternatives section rejects having flip WRITE the frontmatter. The
reasoning is right and worth preserving - one tool owning both the truth and its index means a bug
in it corrupts the record with no second opinion. The tool's value is that it is narrow and refuses.
Widening it to write specs makes it the thing it guards against.

ISS-004 (accepted): edge case 5 - `status: done  # comment` must be stripped before comparing. This
is live in 501 specs today (TASK-IMP-117's corpus problem), so a naive string compare would refuse
every flip on an FM-001-carrying spec and wedge the workflow on contact. Caught in the matrix rather
than in production.

ISS-005 (accepted): edge case 6 - two `status:` lines refuse. Ambiguous truth is not truth, and
picking the first would be the machine choosing which record is authoritative.

ISS-006 (accepted): AC6 replays 116's exact sequence and requires a refusal at the FIRST flip. Same
shape as 119's AC7 and 118's AC6 - a fix that cannot stop the case that motivated it is decoration.

## Rubric families

- FM: clean. SEC: seven required sections present. COND: three-bullet disclosure naming that both
  instances are the agent's own - one written directly, one by a sub-agent it dispatched.
- QA: 11 rows across all six categories, 1 SECURITY and 2 DEGRADATION. Rows 5 and 6 are the ones
  that would have bitten on contact with the live corpus.
- SAFE: adds a precondition to an existing refusal path; reuses exit 6 and relUnderRoot. No new
  writer, no widened surface.
- TRACE: 1.1-1.5 to tests; AC1-AC6 to clauses. AC6 is load-bearing.

## Verdict

pass - 10/10. The asymmetry is the finding: the index write is a hardened tool with pre-images, a
footprint ceiling and a full retally; the truth write is a sed. We built the mechanism on the
derived artefact and left the authoritative one bare.
