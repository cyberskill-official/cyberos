# `prd_rubric@1.0` — machine-checkable Product Requirements rubric

> Sourced from `cyberos/skill/contracts/product-requirements-document/CONTRACT.md` and `../../../modules/cuo/docs/module.md` §2(b) Requirements + §4.5 PRD section. Rubric version `1.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `product-requirements-document@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–120 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `product` | required, string | error | false |
| `FM-104` | `status` | required, one of: draft, in_review, approved, in_progress, shipped, archived | error | false |
| `FM-105` | `target_release` | required, SemVer or quarter (`^\d{4}-Q[1-4]$`) | error | false |
| `FM-106` | `created_at` | required, ISO 8601 with timezone | error | true |
| `FM-107` | `ai_authorship` | required, one of: none, assisted, co_authored, generated_then_reviewed | error | false |
| `FM-108` | `provenance.source_path` | required, path exists, readable | error | false |
| `FM-109` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |
| `FM-110` | `linked_sow` | optional but recommended; if present, must resolve to a SOW path | warning | false |
| `FM-111` | `eu_ai_act_risk_class` | required if product is AI-driven; one of: not_ai, minimal, limited, high | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## Vision` | error |
| `SEC-002` | `## Target Users / Personas` | error |
| `SEC-003` | `## Problem Statement` | error |
| `SEC-004` | `## Use Cases` | error |
| `SEC-005` | `## Functional Requirements` | error |
| `SEC-006` | `## Non-Functional Requirements` (ISO/IEC 25010:2023 coverage) | error |
| `SEC-007` | `## Success Metrics` | error |
| `SEC-008` | `## Out of Scope` | error |
| `SEC-009` | `## Rollout Plan` | error |
| `SEC-010` | `## Risks and Mitigations` | error |
| `SEC-011` | `## Open Questions` | warning |
| `SEC-901` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-902` | Section ordering matches SEC-001..010 | warning |
| `SEC-903` | Heading hierarchy well-formed (no H2→H4 jumps; one H1 only) | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Product is AI-driven (i.e. `eu_ai_act_risk_class != not_ai`) | `## AI Risk Assessment` with `### Data Sources`, `### Human Oversight`, `### Failure Modes` H3s | error |
| `COND-002` | Product handles personal data | `## Privacy & Data Handling` block referencing applicable regulations (GDPR / Vietnam Decree 13/2023 PDPD / HIPAA / etc.) | error |
| `COND-003` | Product is customer-facing | `## Customer Quotes` section with ≥1 quote wrapped in `<untrusted_content>` | warning |
| `COND-004` | Product replaces existing system | `## Migration Plan` block with cut-over criteria | error |
| `COND-005` | Product touches a regulated domain (biometrics, hiring, credit, health, education grading) | `## Regulatory Compliance` block enumerating applicable rules + sign-off owners | error |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | Any paragraph in a non-boilerplate section lacks a `source_ref` marker | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Any paragraph in a non-boilerplate section lacks an `authority` marker | error |
| `QA-NFR-001` | NFR section missing ISO/IEC 25010 coverage | §6 doesn't enumerate (or explicitly waive) the 9 quality characteristics: functional suitability, performance efficiency, compatibility, interaction capability, reliability, security, maintainability, flexibility, safety | error → needs_human (`nfr_coverage`) |
| `QA-METRIC-001` | Vanity metric in §7 | Metric without baseline + target + deadline; or only signups/views/followers without definition | warning |
| `QA-METRIC-002` | Unsourced numeric target | Metric uses a target value not derivable from inputs | error → needs_human (`success_metric_targets`) |
| `QA-PERSONA-001` | Persona without "jobs to be done" | A persona under §2 lacks a "wants to" / "needs to" line | warning |
| `QA-USECASE-001` | Use case without acceptance criteria | A use case in §4 lacks a `Given/When/Then` block or equivalent acceptance criterion | warning |
| `QA-AI-001` | AI-product without AI risk assessment | `eu_ai_act_risk_class ∈ {limited, high}` but §4.COND-001 is missing or empty | error → needs_human (`ai_act_risk_boundary`) |
| `QA-RISK-001` | Risk without mitigation | A risk in §10 lacks a `mitigation:` field | warning |
| `QA-DEP-001` | Cross-team dependency without owner | A dependency on another team/product lacks a ticket/owner/commitment | warning → needs_human (`cross_team_dependency`) |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | Quoted passage from source not wrapped | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` from a skeleton stub | warning (open until human resolves) |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (per `references/UNTRUSTED_CONTENT.md` §3) | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor | warning |

## §7  Cross-skill rules (when chained from `product-requirements-document-author` or upstream SOW)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The PRD's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The PRD's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |
| `XCHAIN-003` | `linked_sow` (if present) resolves to a SOW that passed statement-of-work-audit at 10/10 | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source brief / SOW hash differs from PRD's `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff. | warning → needs_human (`stale_artefact_disposition`) |

---

## Rule auto-fix behaviour catalogue

| auto-fixable value | Audit behaviour |
| ------------------ | --------------- |
| `true` | Minimal textual change; mark `fixed`. |
| `false` | Leave `open` or mark `needs_human` per severity. |
| `skeleton` | Insert TODO marker; mark `open` with `todo_inserted: true`. |

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the 8-step algorithm.
- `cyberos/skill/docs/RUBRIC_FORMAT.md` — the rubric format.
- `REPORT_FORMAT.md` (sibling file) — `.audit.md` shape.
- `INVARIANTS.md` (sibling file) — invariant catalog including `deterministic_drift`.
- `cyberos/skill/contracts/product-requirements-document/CONTRACT.md` — the PRD template this rubric audits.
- `../../../modules/cuo/docs/module.md` §2(b) — Requirements stage source.
