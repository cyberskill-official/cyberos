# `prd-author` self-audit invariants (scaffold)

> Truths the PRD author MUST enforce about its own behaviour. Scaffold-only at v0.1.0; runtime engine in v0.3.0.

## Invariants

### INV-001 — refuse triage_verdict: reject briefs

**Statement.** This skill MUST refuse to author a PRD from any `project_brief@1` whose frontmatter `triage_verdict` is `reject`. The seam between "discovery says don't do this" and "now we have a PRD" exists for a reason; bypassing it short-circuits the triage gate.

**Check.** At PHASE_1 (validate brief), parse the brief's frontmatter. If `triage_verdict == reject`, return outcome `REFUSED_REJECTED_BRIEF`; do NOT proceed to subsequent phases; do NOT write any artefact.

**Severity.** `error` (sev-0).

### INV-002 — no llm-implicit authority on Goals

**Statement.** Every numbered item in the PRD's `## Goals` section MUST carry an `<!-- authority: ... -->` marker, AND the marker value MUST NOT be `llm-implicit`. Goals are strong claims; downstream consumers (engineering, sales) trust them. `llm-implicit` (the agent inferred without a citable source) is too weak for goals.

**Check.** Before write, regex against the PRD body — every line in `## Goals` matching `^\d+\. ` MUST be preceded by `<!-- authority: (human-edited|human-confirmed|llm-explicit) -->`. Matches with `llm-implicit` are rejected.

**Severity.** `error` (sev-0).

**Refinement template.**
```
trigger: INV-002 breach: PRD {prd_path} has llm-implicit authority on Goals
observation: Goal #{n} carries authority: llm-implicit. Goals must be at least llm-explicit.
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/prd-author/STANDALONE_INTERVIEW.md
proposed_amendment_section: §"Authority-elevation pass"
proposed_diff: |
  +  After phase 4 (synthesise), run an authority-elevation pass:
  +  for any Goals item still at llm-implicit, ask the user a yes/no question
  +  to elevate to human-confirmed, OR cite a BRAIN entry to elevate to llm-explicit.
minimum_viable: "Add the elevation pass; never write a PRD with llm-implicit goals."
```

### INV-003 — refuse triage_verdict: revise without explicit override

**Statement.** A `triage_verdict: revise` brief is NOT consumed by default. The input envelope MUST carry `proceed_despite_revise: true` (set by the user explicitly) for this skill to author. When set, the resulting PRD body carries `## Reservations Recorded From Discovery` documenting the triage flags + user's choice.

**Check.** At PHASE_1, after frontmatter parse: if `triage_verdict == revise` AND envelope's `proceed_despite_revise != true`, return outcome `REFUSED_REVISE_NEEDS_OVERRIDE`.

**Severity.** `error`.

### INV-004 — every PRD claim cites its source

**Statement.** Every claim in `## Goals`, `## User Stories`, `## Quality Bars`, `## Success Definition` MUST cite its source — either an `<!-- authority: ... -->` marker (above) AND/OR a trailing HTML comment naming the brief section, chat answer ID, or BRAIN memory_id it derives from.

**Check.** Regex against the PRD body — every claim line MUST be flanked by either an authority marker (which is itself the citation for `human-edited` / `human-confirmed`) OR a trailing `<!-- source: ... -->` comment.

**Severity.** `warning` — counts toward `user_correction_streak`.

### INV-005 — scope discipline

**Statement.** No `write_file` lands outside `output_dir` OR the declared `allowed_brain_scopes.write` BRAIN scopes.

**Check.** Walk audit rows of `op:create` or `op:str_replace`; every path is under output_dir OR matches `^\.cyberos-memory/(project|memories/projects|memories/decisions)/.*$`.

**Severity.** `error`.

### INV-006 — BRAIN read budget

**Statement.** Phase 3 (BRAIN-targeted reads) MUST NOT exceed 12 queries OR 60 returned memories. (Slightly higher than requirements-discovery's budget because PRD authoring needs `module:*` reads in addition.)

**Check.** Track during phase 3; on breach, halt phase 3 + record advisory + proceed.

**Severity.** `warning`.

### INV-007 — confidentiality non-degradation

**Statement.** The PRD's `confidentiality` field MUST be ≥ the brief's confidentiality (in the order: public < internal < client_confidential < regulated). PRD authoring CAN tighten confidentiality but MUST NOT loosen it. (e.g., brief was `internal`, PRD becomes `client_confidential` is OK; brief was `regulated`, PRD becomes `internal` is REJECTED.)

**Check.** Compare brief frontmatter `confidentiality` to PRD frontmatter `confidentiality` before write.

**Severity.** `error`.

## Adding a new invariant

Same procedure as siblings.
