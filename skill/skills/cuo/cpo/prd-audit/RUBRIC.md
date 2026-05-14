# `prd_rubric@1.0` — machine-checkable PRD audit rubric

> Locked at registry v0.2.5. Bumping requires coordinated update of `cyberos/docs/contracts/prd/CONTRACT.md` (the contract body) and `cuo/cpo/prd-audit/SKILL.md` CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in audit reports so reports diff cleanly across iterations.

> **Advisory-leaning by design (Q4 of registry v0.2.4).** Most rules are `warning`-severity. Only structural-correctness rules are `error`. Reviewers can accept warning-severity issues at PRD-approval time; error-severity issues block approval until resolved.

## §15.1 Frontmatter — structural

| rule_id | Check | Severity |
| --- | --- | --- |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error |
| `FM-002` | All keys are `snake_case` | error |
| `FM-003` | No duplicate keys | error |
| `FM-004` | `template` key present and equals `prd@1` | error |

## §15.2 Frontmatter — per-field

| rule_id | Field | Rule | Severity |
| --- | --- | --- | --- |
| `FM-101` | `title` | required, string, 3-100 chars | error |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error |
| `FM-106` | `created_at` | required, ISO 8601 with timezone | error |
| `FM-107` | `last_updated_at` | required, ISO 8601 with timezone, ≥ created_at | error |
| `FM-110` | `prd_status` | one of: draft, in_review, approved, superseded | error |
| `FM-111` | `project_brief_ref` | required; resolves to a real `project_brief@1` markdown OR memory_id | error |
| `FM-112` | `target_release` | SemVer / quarter / `unspecified` | error |
| `FM-113` | `client_visible` | boolean | error |
| `FM-114` | `client_id` | required when `client_visible: true` | error |
| `FM-115` | `eu_ai_act_risk_class` | one of: not_ai, minimal, limited, high (`unacceptable` rejected) | error |
| `FM-116` | `confidentiality` | one of: public, internal, client_confidential, regulated | error |
| `FM-117` | `prd_iteration` | integer ≥ 1 | error |
| `FM-118` | `superseded_by` | required when `prd_status: superseded`; resolves to a real PRD | error |

## §15.3 Always-required H2 sections

| rule_id | Heading | Severity |
| --- | --- | --- |
| `SEC-001` | `## Background` | error |
| `SEC-002` | `## Goals` | error |
| `SEC-003` | `## Non-goals` | error |
| `SEC-004` | `## User Stories` | error |
| `SEC-005` | `## Quality Bars` | error |
| `SEC-006` | `## Open Questions` | error |
| `SEC-007` | `## EU AI Act Considerations` | error |
| `SEC-008` | `## Compliance and Privacy` | error |
| `SEC-009` | `## Rough Sizing` | error |
| `SEC-010` | `## Success Definition` | error |
| `SEC-011` | `## Research Signals` | error |
| `SEC-012` | Each required H2 is non-empty | error |

## §15.4 Conditionally-required H2 sections

| rule_id | Trigger | Required | Severity |
| --- | --- | --- | --- |
| `COND-001` | `client_visible: true` | `## Client Context` H2 | error |
| `COND-002` | `eu_ai_act_risk_class: high` | `## High-Risk AI Risk Assessment` H2 with H3s `### Annex III mapping`, `### Oversight mechanism`, `### Transparency obligations`, `### Post-market monitoring` | error |
| `COND-003` | `confidentiality ∈ {client_confidential, regulated}` | `## Compliance Implementation Plan` H2 | error |
| `COND-004` | `prd_status: approved` | `## Approval Record` H2 with table of approver / role / ISO ts / version-hash | error |
| `COND-005` | `eu_ai_act_risk_class ∈ {not_ai, minimal}` | `## EU AI Act Considerations` contains the explicit "Not in scope of EU AI Act" statement | error |

## §15.5 Authority markers (NEW vs fr-audit)

| rule_id | Anti-pattern | Detection | Severity |
| --- | --- | --- | --- |
| `AUTH-001` | Goal lacks authority marker | Any line in `## Goals` matching `^\d+\. ` not preceded by `<!-- authority: ... -->` | error → needs_human (`authority_marker_missing`) |
| `AUTH-002` | Goal carries `llm-implicit` | Any Goal item with `<!-- authority: llm-implicit -->` | error → needs_human (`authority_too_weak`) |
| `AUTH-003` | User Story acceptance criterion lacks authority marker | Any `- ` bullet under `### Story N` acceptance subsection without preceding marker | warning |
| `AUTH-004` | Quality Bar metric lacks authority marker | Any `## Quality Bars` bullet without authority marker AND with strong claim language ("MUST", "WILL", numeric target) | warning |

## §15.6 Quality heuristics (mostly warning per Q4)

| rule_id | Anti-pattern | Detection | Severity |
| --- | --- | --- | --- |
| `QA-001` | Article 5 / prohibited practice (mirrors fr-audit) | Body describes social scoring, untargeted face scraping, workplace/education emotion inference, real-time biometric ID for law enforcement, subliminal manipulation | error → needs_human (`legal_compliance`) |
| `QA-002` | High-risk indicator without `high` (mirrors fr-audit) | Body mentions Annex III domain while class < high | error → needs_human (`ai_act_classification_drift`) |
| `QA-003` | Vague success metric | `## Success Definition` claim missing baseline OR target OR deadline | warning |
| `QA-004` | Vanity metric | Metric uses signups / views / followers without engagement / retention / outcome context | warning |
| `QA-005` | Vague Non-goals | `## Non-goals` is empty OR has only one bullet | warning |
| `QA-006` | Unsourced numeric target | Quality Bar metric uses target value not derivable from brief / chat / BRAIN cite | warning → needs_human (`unverifiable_research_signal`) |
| `QA-007` | Empty research signals | `## Research Signals` empty OR contains only "founder intuition" without surrounding reasoning | warning |
| `QA-008` | Confidentiality loosening | PRD `confidentiality` < brief `confidentiality` | error (per `prd-author` INV-007; redundant safety) |
| `QA-009` | Approval record stale | `prd_status: approved` AND `last_updated_at` > approval timestamp + 7 days | warning |

## §15.7 Untrusted-content safety (mirrors fr-audit)

| rule_id | Check | Severity |
| --- | --- | --- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (mirrors fr-audit's set) | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor | warning |

## §15.8 Cross-skill rules (when chained from prd-author)

| rule_id | Check | Severity |
| --- | --- | --- |
| `STALE-001` | PRD's on-disk sha256 differs from prd-author's manifest at audit time (when `upstream_context.from_skill == cuo/cpo/prd-author`) | error → needs_human (`stale_prd_disposition`) |

## §15.9 Severity → exit-code mapping

- `0` = pass (no error, no warning)
- `1` = errors present
- `2` = warnings only
- `3` = needs_human verdicts present (HITL_PAUSE)

## §15.10 Confidence-band reporting

Mechanical-rule majority (`confidence ≥ 0.95`): FM-001..118, SEC-001..012, COND-001..005, SAFE-001..004, STALE-001, QA-008, AUTH-001/002, QA-005, QA-007.

LLM-judgement minority (caps at `0.7`): QA-001 (Article 5 detection — needs natural-language judgement), QA-002 (high-risk indicator — same), QA-003 (vague metric — needs to interpret "vague"), QA-004 (vanity — same), QA-006 (unsourced target — needs reasoning about where the target could derive from), AUTH-003/004 (authority on stories/quality-bars — natural-language strength check).

The split mirrors fr-audit/RUBRIC.md §15.9. Mechanical rules are reproducible; LLM-judgement rules are band-reproducible.
