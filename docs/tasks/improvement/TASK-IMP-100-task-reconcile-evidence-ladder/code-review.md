# TASK-IMP-100 code review

Reviewer: parent ship-tasks agent (batch 5). Diff: 3 new files (task-reconcile.mjs 276+,
test_task_reconcile.sh, modules/skill/task-reconcile/SKILL.md) + tools/install/build.sh (+3).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | rungs 1-4 read-only, 5 only under --run-tests, no writes outside --out | t04 whole-tree fingerprint identical across runs; rung5 gated on the flag |
| 1.2 | report carries rung verdicts, drift_score, hitl: required, exactly one recommendation | reconcile-report@1 frontmatter; t01-t04 read `recommendation` from --json |
| 1.3 | recommendation mapping | t01 resume_at_phase, t02 route_back, t03 adopt_candidate, t04 not_applicable + drift->route_back |
| 1.4 | both artefact homes accepted | t03 bundle arm (a .workflow bundle naming the artefacts flips adopt -> resume) |
| 1.5 | SKILL.md: machine-floor loop + no-silent-execution rule | recorded greps (gate log AC 6) |
| 1.6 | build.sh vendors it; suite gates the payload copy; run_all glob | t05 (byte-parity + a live run from the payload copy); `chain OK: 25 referenced, 53 vendored` |

## Judgment

- **The design changed under dogfooding, twice, and that is the point.** Run against the live
  corpus, R1 reds every correctly-shipped task: audits bind a whole-FILE sha while ship-tasks
  rewrites `status`/`shipped` in that same file. First correction: verify the binding at the
  audit COMMIT and judge drift on the normative half. That exposed the second, sharper fact -
  the recorded sha matches no committed version at all, because authoring hashes the spec and
  then flips status before committing. The audited bytes never existed in history. That is the
  086 class in miniature, found in our own procedure by our own instrument.
- **What I did with it**: the binding gap is reported as a NOTE with the substantive check
  still performed (normative half, audit commit vs HEAD) rather than a red. A tool that reds
  the entire corpus on a hygiene artifact is a tool nobody runs twice. The gap itself is filed
  as a follow-up finding (IMP-19) - fixing the audit convention is not this task's cone.
- **Composition, not reimplementation**: R1 shells to task-lint, R3 to ship-manifest verify.
  When those tools are absent the rungs say so and never invent a verdict.
- **Read-only is load-bearing**, so it is asserted mechanically (t04), not promised in prose.
- **Blast radius**: three new files and a vendor line. The one behavioral coupling is the
  chain: naming a skill in ship-tasks means the payload must carry it in both trees, which
  build.sh's own gate enforced immediately.

## Disclosures

1. **build.sh VENDORED_SKILLS entry** - the spec named build.sh for the docs-tools copy; the
   chain-coverage gate additionally required the skill in the vendored list. Same file, one
   more line, mechanically forced by the gate.
2. **test_full_sdp_payload.sh skill-count pin 52 -> 53** - a deliberate pin (like the version
   pins) that a new vendored skill legitimately moves. Not in the spec's modified_files;
   disclosed rather than silently absorbed.
3. **First SKILL.md frontmatter was rejected by build.sh's schema gate** (name must equal the
   dir, description bounded). Rewritten to the sibling contract; no scope change.

Verdict: no open findings. One follow-up finding filed (IMP-19, audit binding hygiene).

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
