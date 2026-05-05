# `fr-create` self-audit invariants

> Declarative truths the skill enforces about itself at runtime. Checked at every node boundary, every 25 audit rows, and on completion (per `self_audit.check_at` in `SKILL.md`). A breach emits a `refinement_proposal` envelope, pauses the pipeline, and waits for human review.

## How invariants work

Each invariant has:

- **ID** — `INV-NNN`, monotonic per-skill, never reused.
- **Statement** — the declarative truth.
- **Check** — a deterministic test (SQL against `genie.action_log`, or a Python predicate against the manifest state, or a regex on emitted FR bodies).
- **Severity** — `error` (immediate breach) / `warning` (counts toward anomaly streaks per `self_audit.anomaly_signals`) / `info` (reported to chat but doesn't pause).
- **Refinement template** — the structured `refinement_proposal` payload to emit on breach.

## Invariants

### INV-001 — citation completeness

**Statement.** Every emitted FR carries at least one citation (per QA-007 in `cuo/cpo/fr-audit/RUBRIC.md`) for every factual claim in its `## Problem` and `## Success Metrics` sections.

**Check.** For each FR written this run: `grep -E '^(- \[|cite: |source: )' fr_body` returns ≥1 match per `## Problem` paragraph and ≥1 match per metric in `## Success Metrics`.

**Severity.** `error`.

**Refinement template.**
```
trigger: INV-001 breach on FR-{NNN}: missing citation in {section}
observation: {section} contains {N} factual claims; {M} citations found.
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/fr-create/SKILL.md
proposed_amendment_section: §"WORKER phase" step W2
proposed_diff: |
  +  Before writing, verify every paragraph in `## Problem` and every
  +  metric in `## Success Metrics` has at least one inline citation
  +  to a requirements-file location. If missing, generate the citation
  +  from the source span used to derive the claim, OR flag the FR for
  +  HITL with reason `MISSING_EVIDENCE`.
minimum_viable: "Add citation-completeness check to W3 WRITE step."
```

### INV-002 — manifest ↔ disk parity

**Statement.** After every node boundary, every FR listed in `manifest.json` `frs:` map exists at its `file_path` AND its disk SHA-256 matches `frs[FR].fr_hash`.

**Check.** Compute `sha256(file_contents)` for each `frs[FR].file_path`; compare to `frs[FR].fr_hash`. Mismatch or missing file = breach.

**Severity.** `error`.

**Refinement template.**
```
trigger: INV-002 breach: manifest claims {fr_id} at {path} hash {expected}; disk shows {observed_or_missing}
observation: {one of: file missing, hash mismatch, manifest stale}
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/fr-create/references/MANIFEST_SCHEMA.md
proposed_amendment_section: §3.4 "Write discipline"
proposed_diff: |
  +  After every `write_file(fr.file_path, body)`, the skill MUST
  +  re-read the file and verify SHA-256 matches before writing the
  +  manifest update. If the read-back fails, append op:"revert" to
  +  the audit log per AGENTS.md §4.4 step 4.
minimum_viable: "Add post-write read-back verification to W3."
```

### INV-003 — coverage of source ingestion

**Statement.** Every memory write derived from PRD ingestion carries `ingestion_coverage:` per AGENTS.md §4.10, and `processed_lines / source_lines ≥ 0.99` UNLESS `intentional_summary: true` with a populated `summary_reason:`.

**Check.** Walk audit rows of `op:create | op:str_replace` with `provenance.source = doc` written by this skill in this trace. Each must have `ingestion_coverage:` populated and pass the ratio rule.

**Severity.** `error`.

**Refinement template.**
```
trigger: INV-003 breach: PRD digest {memory_id} has coverage {ratio} (<0.99) without intentional_summary flag
observation: PRD source: {source_path} ({source_lines} lines); processed: {processed_lines} lines.
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/fr-create/SKILL.md
proposed_amendment_section: §"PLAN phase" step 1
proposed_diff: |
  +  When reading requirements files, paginate the read sequentially
  +  (Read tool offset/limit; or Bash with chunk processing) and
  +  track high-water mark. Before writing any digest memory, compute
  +  ingestion_coverage and verify ≥0.99 OR set intentional_summary:
  +  true with summary_reason. Per AGENTS.md §4.10.
minimum_viable: "Surface coverage stat at end of PLAN phase."
```

### INV-004 — FR-ID uniqueness within batch

**Statement.** No two FRs in this batch share the same `FR-NNN` prefix or the same slug.

**Check.** `len({fr.id for fr in frs}) == len(frs)` AND `len({fr.slug for fr in frs}) == len(frs)`.

**Severity.** `error`.

### INV-005 — no fabrication beyond confidence band

**Statement.** Any frontmatter field with declared `provenance.confidence < 0.5` is flagged for HITL (must appear in `hitl_categories`), never auto-emitted as fact.

**Check.** Walk the FR's frontmatter; for every field whose source is `inference`, verify `provenance.confidence ≥ defer_below` OR the field appears in the manifest's `hitl_pending` list.

**Severity.** `error`.

### INV-006 — scope discipline

**Statement.** No `write_file` call lands outside `output_dir`.

**Check.** Walk audit rows of `op:create` written by this skill; every `path` must be `output_dir/...`.

**Severity.** `error` — immediate refusal + refinement_proposal.

### INV-007 — EU AI Act non-degradation

**Statement.** No FR has `eu_ai_act_risk_class: minimal` set when `feature_type ∈ {user_facing, integration}` AND any `<untrusted_content>` block in the FR's `## Customer Quotes` or `## Problem` is non-empty.

**Check.** Regex against the rendered FR.

**Severity.** `error`.

**Reasoning.** This catches the failure mode where the model auto-classifies a customer-facing AI feature as "minimal" without a determining fact, contra `references/EU_AI_ACT_DECISION_TREE.md` and the CPO persona's voice rules.

### INV-008 — confidence-band reporting

**Statement.** Every FR's audit row carries a non-null `confidence` field within `[0.0, 1.0]`.

**Check.** SQL — `SELECT count(*) FROM genie.action_log WHERE skill_id = 'cuo/cpo/fr-create' AND trace_id = $1 AND row_kind = 'artefact_write' AND (confidence IS NULL OR confidence < 0 OR confidence > 1)` returns 0.

**Severity.** `error`.

## Adding a new invariant

1. Pick the next ID — read this file's last `INV-NNN`, increment.
2. Write the four-part block: Statement, Check, Severity, Refinement template.
3. Add a CHANGELOG entry under the next PATCH bump (or MINOR if the invariant changes user-visible behaviour).
4. The runtime validator runs the new invariant on the next session start and any in-flight pipeline picks it up at its next node boundary.

## Invariants vs. failure modes

`references/FAILURE_MODES.md` catalogs **boot/runtime errors** — the skill refuses to start because something is structurally wrong (BOOT-001 through BOOT-008, plus CONTRACT_DRIFT, INPUTS_CHANGED, STALE_OVERWRITE, EXHAUSTED). Those fail closed: emit error, don't proceed.

This file catalogs **runtime invariants** — the skill is running fine, producing outputs, and we want to detect if its outputs are *behaviourally drifting* from what we expect. Those produce `refinement_proposal` envelopes and pause the pipeline for human review (auto-refinement loop per registry README Part 6) or hit the manual fine-tune flow (Part 7) if the proposals exceed the configured threshold.

Both files together make the skill self-auditing.
