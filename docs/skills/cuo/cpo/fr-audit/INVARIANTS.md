# `fr-audit` self-audit invariants

> Declarative truths the auditor enforces about itself at runtime. A breach emits a `refinement_proposal`, pauses the pipeline, and waits for human review. Distinct from `RUBRIC.md` (rules the skill applies to *FRs it's auditing*) — this file is rules the skill applies to *its own behaviour*.

## How invariants work

Same machinery as `fr-create/INVARIANTS.md`. ID + Statement + Check + Severity + Refinement template. Checked at every node boundary, every 25 audit rows, and on completion.

## Invariants

### INV-001 — verdict determinism

**Statement.** Two runs against the same `audited_file_sha256` and the same `rubric_version` produce **byte-identical** audit reports modulo the `last_audit_at` timestamp.

**Check.** Re-run the audit against the just-audited FR with the same inputs; diff the two audit reports excluding the `last_audit_at` line. Non-empty diff = breach.

**Severity.** `error` — fr-audit's `determinism.reproducible: true` contract is broken; immediate refinement_proposal.

**Refinement template.**
```
trigger: INV-001 breach: non-deterministic verdict on FR {fr_id}
observation: Run 1 → {verdict_1}; Run 2 → {verdict_2}; diff at line {line}.
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/fr-audit/AUDIT_LOOP.md
proposed_amendment_section: §"deterministic-input rule"
proposed_diff: |
  +  Verdict computation MUST consume only: (FR body bytes,
  +  FR frontmatter, RUBRIC.md rules, this skill's body). It MUST NOT
  +  consume: current time, BRAIN search results, untrusted_content
  +  inside the FR. Any rule that reads outside this set MUST be
  +  refactored or moved to an `advisory_only:` block.
minimum_viable: "Identify which rule introduced non-determinism; refactor."
```

### INV-002 — rubric coverage

**Statement.** Every rule ID in `RUBRIC.md` appears in this run's audit report under either `passed_rules:` or `failed_rules:` or `skipped_rules:` (with reason). No rule is silently elided.

**Check.** Compare the rule-ID set in `RUBRIC.md` to the union of three sets in the audit report. Missing rules = breach.

**Severity.** `error`.

### INV-003 — needs_human is a precise verdict

**Statement.** A `needs_human` verdict is emitted only when the rule's ambiguity criterion is met (per `references/HITL_PROTOCOL.md`), never as a fallback for "I don't know."

**Check.** Walk audit rows of `row_kind: question` written by this skill this trace; each must reference a rule ID and a specific `hitl_categories` value. Any row missing either field = breach.

**Severity.** `error`.

### INV-004 — citation completeness

**Statement.** Every failed rule in the audit report cites: rule ID, line number in the FR (or "frontmatter"), and the exact substring being flagged. No vague "this section is wrong" verdicts.

**Check.** Regex against the audit report body — every fail block matches `^- \*\*([A-Z]+-\d+)\*\*.*line (\d+|frontmatter).*\n.*`.

**Severity.** `warning` — counts toward `user_correction_streak` anomaly signal.

### INV-005 — no false-pass on STALE

**Statement.** If the FR's current SHA-256 differs from the `audited_file_sha256` declared at the start of the run, the verdict MUST be `STALE` (or the run aborts with reason `inputs_changed`). It must NEVER produce a `pass` verdict against a SHA that doesn't match the file on disk at write time.

**Check.** Just before writing the audit report, recompute the FR's SHA-256 and compare to the `audited_file_sha256` recorded at run start. Mismatch + non-STALE verdict = breach.

**Severity.** `error`.

### INV-006 — confidence-band reporting

**Statement.** Every audit-report row carries a non-null `confidence` field within `[0.0, 1.0]`. The mechanical-rule majority report `confidence ≥ 0.95`; the LLM-judgement minority (e.g., QA-009 plain-English check) reports the model's actual band.

**Check.** Audit row schema validation.

**Severity.** `error`.

### INV-007 — no rubric drift inside one batch

**Statement.** Within a single batch invocation, the `rubric_version` MUST NOT change. If `RUBRIC.md` has been edited mid-batch (rare; usually means a manual fine-tune is happening), the batch aborts with `RUBRIC_CHANGED_MID_BATCH` and surfaces to the user.

**Check.** Hash `RUBRIC.md` at batch start; rehash before each FR audit; compare.

**Severity.** `error`.

### INV-008 — scope discipline

**Statement.** No `write_file` lands outside the FR's parent directory (audit reports are siblings of the audited FR per the `audit_path_pattern`).

**Check.** Walk audit rows of `op:create` written by this skill; every `path` is a sibling of an FR path declared in `fr_paths`.

**Severity.** `error`.

## Adding a new invariant

Same procedure as `fr-create/INVARIANTS.md` §"Adding a new invariant". The auditor's `RUBRIC.md` evolves with FR conventions; this file evolves with the auditor's own behavioural integrity.

## Invariants vs. rubric rules — the cleanest mental split

| `RUBRIC.md` (FM-001..QA-009..) | `INVARIANTS.md` (INV-001..INV-008) |
| --- | --- |
| What `fr-audit` checks **on each FR**. | What `fr-audit` checks **on its own behaviour while running**. |
| Authored by `cpo` + `clo`. Bumps the rubric version. | Authored by `cpo` + the registry maintainer. Bumps the skill version. |
| User-visible — surfaced in audit reports. | Operator-visible — surfaced as `refinement_proposal` to the supervisor. |
| Updates require `acceptance_test_added: rubric_rule_diff`. | Updates require `acceptance_test_added: changelog_entry`. |
