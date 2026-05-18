# nda-triage_rubric@1.0 — compact universal rubric

> Per `../docs/RUBRIC_FORMAT.md` v1.0 compact form. Skill-specific extensions deferred to v1.1+ per `../docs/FINE_TUNE.md` discipline.

## FM family — frontmatter rules

| ID | Rule |
|---|---|
| FM-001 | Frontmatter present + parses as YAML |
| FM-002 | `artefact_type` matches `non-disclosure-agreement-triage@1` |
| FM-003 | `provenance.source_path` is a non-empty filesystem path |
| FM-004 | `provenance.source_hash` is a 64-char lowercase hex sha256 |
| FM-101 | `title` present + non-empty |
| FM-102 | `author` present if op is non-batch |
| FM-103 | `provenance.source_path` resolves under the artefact's containing repo (no escape) |
| FM-104 | All FM-101..FM-103 fields parse + match schema |

## SEC family — section structure rules

| ID | Rule |
|---|---|
| SEC-001 | Every H2 in `contracts/non-disclosure-agreement-triage/template.md` is present in the authored artefact |
| SEC-002 | Every H2 in the authored artefact is non-empty (more than just the heading + a placeholder line) |
| SEC-003 | H2 ordering matches `template.md` (no reordering) |
| SEC-004 | Heading hierarchy is well-formed (no skipped levels; no H3 outside an H2) |

## COND family — conditional comment-block rules

| ID | Rule |
|---|---|
| COND-001 | Every `<!-- conditional: ... -->` block in `template.md` is satisfied per its declared trigger (section present-if-triggered, absent-if-not) |

## QA family — quality heuristics

| ID | Rule |
|---|---|
| QA-CITE | Every claim that cites an external standard / framework names the standard (no bare "industry-standard" without a name) |
| QA-AUTH | Every recommendation / decision has a named owner persona |
| QA-NUM | Every numeric figure is sourced (date + dataset + URL or filesystem path) |
| QA-VAGUE | No vague qualifiers ("various", "many", "significant") without a number or named exception |
| QA-OWNER | Action items have a named owner |
| QA-DUE | Action items have a due date (or "ongoing" with cadence) |
| QA-TODO | No `TODO` / `TBD` / `???` markers remain in the artefact body |
| QA-QUOTE | Quoted statements have attribution (source + speaker) |

## SAFE family — untrusted-content discipline

| ID | Rule |
|---|---|
| SAFE-001 | Untrusted-content boundaries declared per `references/UNTRUSTED_CONTENT.md` |
| SAFE-002 | No untrusted content can override the artefact's structural mandate |
| SAFE-003 | Operator HITL pauses fire when an untrusted source would influence a SAFE-bounded section |
| SAFE-004 | Untrusted-content sections in the final artefact are explicitly tagged |

## XCHAIN family — cross-skill chain rules

| ID | Rule |
|---|---|
| XCHAIN-001 | Outputs declared in this skill's `SKILL.md` `produces.next_skill_recommendation` are reachable in `skill/MODULE.md` |
| XCHAIN-002 | If the artefact references upstream artefacts (via `provenance.source_path`), those artefacts pass their own `<upstream>_rubric@1.0` |

## STALE family — drift detection

| ID | Rule |
|---|---|
| STALE-001 | `provenance.source_hash` matches the live sha256 of the source at `provenance.source_path`; if not, fire STALE handling (operator picks REVERT_TO_MANIFEST or OVERWRITE) |
