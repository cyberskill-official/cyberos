# Task template profiles (TASK-CUO-208) - normative, both templates side by side

<!-- verification preamble (executable, TASK-SKILL-117 convention):
  grep -c '^## engineering-spec@1' THIS_FILE            -> 1
  grep -c '^## task@1' THIS_FILE             -> 1
  grep -c 'rule families' THIS_FILE                     -> >= 2 (one list per profile)
  newest exemplars validate: any docs/tasks/**/task-*.md with '## §1 - Description'
  parses per the engineering-spec grammar below; any with 'template: task@1'
  parses per the task grammar below.
-->

Resolution chain (who decides which template a NEW task uses):

1. explicit per-invocation operator override, else
2. `.cyberos/config.yaml` `task_template` (TASK-CUO-207), else
3. default `engineering-spec@1`.

The resolved template MUST be echoed in /create-tasks' PLAN (value + source) so the
operator approves template + content together. Mixed-template repos are normal: resolution is per
invocation batch; the AUDIT judges every file by its own detected template (below), never the repo default.

## Detection (audit side; from the file itself, TASK-CUO-208 §1 #4)

- frontmatter `template: task@1` present -> task@1 (FM-004 binds it)
- `## §1 - Description` .. `## §11` section grammar -> engineering-spec@1
- BOTH markers or NEITHER -> needs_human naming the conflict; never a guessed profile

## engineering-spec@1

- Frontmatter: `id, title, module, priority (MUST|SHOULD|COULD), status (10-value enum), class
  (product|improvement), verify, phase, owner, created, shipped, memory_chain_hash, related_tasks,
  depends_on, blocks, source_pages, source_decisions, language, service, new_files, modified_files`.
- Sections, in order: `## §1 - Description` (numbered BCP-14 clauses), `## §2 - Why this design`,
  `## §3 - Contract`, `## §4 - Acceptance criteria`, `## §5 - Verification`, `## §6 - Implementation
  skeleton`, `## §7 - Dependencies`, `## §8 - Example payloads`, `## §9 - Open questions`,
  `## §10 - Failure modes inventory`, `## §11 - Implementation notes`. End marker: `*End of task-X.*`.
- Authoring rules: task-author SKILL.md §12 (the single normative home; not duplicated here).
- Audit rule families: the §12 structural sub-rule set + `TRACE` (TRACE-001..005, spec-vs-implementation
  traceability per RUBRIC.md §9) + `QA` + `SAFE`. 10/10 bar; needs_human semantics per RUBRIC.md.

## Task@1

- Frontmatter: `template: task@1` (FM-004) plus the FM-101..111 field set (title, author,
  department, status, priority p0..p3, created_at, ai_authorship, feature_type, eu_ai_act_risk_class,
  target_release?, client_visible) - normative in RUBRIC.md §2-§3.
- Sections, in order: `## Summary`, `## Problem`, `## Proposed Solution`, `## Alternatives Considered`,
  `## Success Metrics`, `## Scope`, `## Dependencies` (SEC-001..007), plus conditional sections per
  COND-001..004 (Customer Quotes / Sales-CS Summary / AI Risk Assessment / AI Authorship Disclosure).
- Authoring rules (equivalent of §12 for this profile): every SEC section non-empty (SEC-008);
  quotes inside `<untrusted_content>` with attribution outside (COND-001); metrics carry
  baseline + target + deadline (QA-004); risk class honest to body content (QA-001..003);
  grafted `§4 Acceptance criteria`/`§5 Verification` sections MAY be added - TRACE rules then apply
  to exactly those sections (RUBRIC.md §9's existing statement).
- Audit rule families: `FM` + `SEC` + `COND` + `QA` + `SAFE` (+ `TRACE` only where the grafted
  §4/§5 sections are present). Same 10/10 bar, same needs_human semantics.

Rule families are cited BY NAME (not rule id): rule additions inside a family inherit automatically;
adding a new family requires touching both this file and RUBRIC.md (TASK-CUO-208 §10 #3).
