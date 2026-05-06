# `srs-author` self-audit invariants (scaffold)

## Invariants

### INV-001 — refuse non-pass PRDs

**Statement.** Refuse to author from a `prd@1` whose `*.audit.md` reports `overall_status != pass`.

**Check.** PHASE_1: parse audit report frontmatter; if `overall_status != pass`, return `REFUSED_NON_PASS_PRD`.

**Severity.** `error` (sev-0).

### INV-002 — no llm-implicit on System Architecture

**Statement.** Every claim in `## System Architecture` MUST carry `<!-- authority: ... -->` and MUST NOT be `llm-implicit`. Architecture decisions are strong technical claims; engineering trusts them.

**Severity.** `error` (sev-0).

### INV-003 — every API surface entry cites its source

**Statement.** Every row in `## API Surface` table cites either a PRD User Story (e.g., `<!-- source: prd Story 1 -->`) OR a chat answer ID OR an explicit `<!-- new: not in PRD -->` marker. No phantom endpoints.

**Severity.** `warning`.

### INV-004 — scope discipline

**Statement.** No write outside `output_dir` or declared BRAIN write scopes.

**Severity.** `error`.

### INV-005 — NFR measurability

**Statement.** Every line in `## Non-Functional Requirements` carries a measurable threshold (number + unit + measurement context). No "should be fast" / "should be reliable".

**Check.** Regex: each `- ` bullet under the section must match `\d+\s*(ms|s|min|%|rpm|qps|GB|MB|TB|9s)`.

**Severity.** `warning`.
