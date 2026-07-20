# TASK-IMP-102 code review

Reviewer: parent ship-tasks agent (batch 5, third member). Diff: task-audit/SKILL.md (3 field re-statements + §12), task-reconcile.mjs (R1 preference), test_task_reconcile.sh (t06).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | normative half defined; body field recorded | task-audit §12 (field list named); TASK-IMP-102's OWN audit.md carries `audited_body_sha256_prefix: 5c530084993c87d5` |
| 1.2 | file field retained, stated as provenance not binding | §12's second bullet; `payload_hash_field` comment |
| 1.3 | re_entrancy + fixity re-stated against the body hash | SKILL.md:86/146/191 (all three now name audited_body_sha256) |
| 1.4 | R1 prefers body; legacy falls back, gap never a verdict | t06 four arms: flip-proof pass, drift red, legacy-honest via audit commit, legacy-dishonest gap-noted-but-pass |
| 1.5 | suite arms | `test_task_reconcile: pass=6 fail=0` |

## Judgment

- **This is the finding fixed at its source, not papered over.** TASK-IMP-100 made reconcile tolerate a broken convention (gap = note). 102 removes the breakage: from now on an audit records a hash that means something for the life of the task. The tolerance stays for the corpus that predates the rule - legacy is read, never rewritten.
- **The skill's own claims were the real defect.** `re_entrancy: idempotent_on_audited_file_sha256` and a fixity note promising byte-stability "for a given audited_file_sha256" were false the moment ship-tasks flipped a status - the document promised a stable key over a field designed to change. Fixing the recorded field without fixing those claims would have left the lie in a quieter place. All three now name the body hash.
- **Its own audit is the first witness.** TASK-IMP-102's audit.md carries the new field, computed over its own normative half - the convention exists in the corpus from the moment the rule does, and the task's binding survives the very flips this batch performs on it.
- **Why not enforce it in task-lint**: that is a rubric change (FM family) which would red every legacy audit in the corpus on the next lint. Out of scope by design, named in Non-Goals.
- **Blast radius**: three files; no existing audit invalidated; no behavior change for tasks whose audits predate the rule beyond a clearer note.
- **Security**: hashing and prose; no execution surface.

## Disclosures

None beyond the spec. The §12 placement continues task-audit's own §-history convention (§11 Rework Mode, 2026-05-20) rather than inventing a new location.

Verdict: no open findings. IMP-19 is closed by this task.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
