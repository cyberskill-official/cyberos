# `audit_rubric@2.0` — machine-checkable Feature Request rubric

> Sourced from `cyberos/skill/contracts/feature-request/CONTRACT.md` (the FR contract body) and `cyberos/docs/Software Development Process.md` §2(b) Requirements. Rubric version `2.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in audit reports so reports are diffable across iterations and operators.

This rubric is a port of the proven rule set from the legacy `cuo/cpo/fr-audit` skill (audit_rubric@2.0, locked since 2026-02). It is preserved here verbatim because it has been battle-tested against 50+ FRs in the cyberos `docs/feature-requests/` catalog. Bumping to 3.0 requires governance sign-off.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` (lowercase ASCII letters, digits, underscores; no leading digit) | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `feature_request@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–72 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `department` | required, one of: engineering, design, product, sales, operations, hr, client_success | error | false |
| `FM-104` | `status` | required, one of: draft, in_review, approved, in_progress, shipped, closed | error | false |
| `FM-105` | `priority` | required, one of: p0, p1, p2, p3 | error | false |
| `FM-106` | `created_at` | required, ISO 8601 with timezone | error | true |
| `FM-107` | `ai_authorship` | required, one of: none, assisted, co_authored, generated_then_reviewed | error | false |
| `FM-108` | `feature_type` | required, one of: user_facing, internal_tooling, integration, infrastructure | error | false |
| `FM-109` | `eu_ai_act_risk_class` | required, one of: not_ai, minimal, limited, high. `unacceptable` MUST be rejected (per Article 5) | error | false |
| `FM-110` | `target_release` | optional; if present, SemVer `^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$` OR quarter `^\d{4}-Q[1-4]$` | error | false |
| `FM-111` | `client_visible` | required, boolean (YAML true/false, not strings, not yes/no) | error | true |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## Summary` | error |
| `SEC-002` | `## Problem` | error |
| `SEC-003` | `## Proposed Solution` | error |
| `SEC-004` | `## Alternatives Considered` | error |
| `SEC-005` | `## Success Metrics` | error |
| `SEC-006` | `## Scope` | error |
| `SEC-007` | `## Dependencies` | error |
| `SEC-008` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-009` | Heading hierarchy well-formed (no H2→H4 jumps; one or zero H1s) | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `client_visible: true` | `## Customer Quotes` with ≥1 quote in `<untrusted_content>`, attribution outside | error |
| `COND-002` | `client_visible: true` | `## Sales/CS Summary` in plain English (no jargon — see QA-009) | error |
| `COND-003` | `eu_ai_act_risk_class ∈ {limited, high}` | `## AI Risk Assessment` with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` in that order | error |
| `COND-004` | `ai_authorship != none` | `## AI Authorship Disclosure` with three bullets labeled `Tools used:`, `Scope:`, `Human review:` | error |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-001` | Dodged risk class | `eu_ai_act_risk_class ∈ {minimal, not_ai}` AND body contains AI-generation cues + `client_visible: true` OR `feature_type: user_facing` | error → needs_human (`ai_act_risk_boundary`) |
| `QA-002` | High-risk indicator without `high` | Body mentions Annex III domain (biometrics / hiring / credit / education grading / emergency triage / law enforcement / migration / critical infra) while class < high | error → needs_human |
| `QA-003` | Article 5 / prohibited practice | Body describes social scoring, untargeted face scraping, workplace/education emotion inference, real-time biometric ID for law enforcement, subliminal manipulation | error → needs_human (`legal_compliance`) |
| `QA-004` | Vanity metric | Metric without baseline + target + deadline; or only signups/views/followers without definition | warning |
| `QA-005` | Vague Alternatives | <2 distinct alternatives; or filler-only ("considered other options") | warning |
| `QA-006` | Vague scope boundaries | `## Scope` lacks `### Out of scope` / `### Non-Goals`, or contains only one bullet | warning |
| `QA-007` | Unsourced numeric target | Metric uses a target value not derivable from inputs | error → needs_human (`success_metric_targets`) |
| `QA-008` | Cross-team dependency claim | `## Dependencies` names another team/module without ticket/owner/commitment | warning → needs_human (`cross_team_dependency`) |
| `QA-009` | Engineering jargon in Sales/CS Summary | Words detected: API, endpoint, schema, webhook, latency, payload, RBAC, JWT, idempotent, migration, raw HTTP verbs, file paths, regex | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` from a §16.5 stub | warning (open until human resolves) |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (case-insensitive, NFC-normalised, zero-width stripped, confusables folded). Markers: `ignore previous`, `ignore all prior`, `disregard the above`, `system prompt`, `you are now`, `developer mode`, `DAN`, `jailbreak`, `<\|im_start\|>`, `<\|im_end\|>`, `[INST]`, `</s>`, `assistant:` at line start, `BEGIN SYSTEM`, `print your instructions`, `reveal your`, base64 blobs ≥80 chars with no surrounding prose | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor (`do this`, `output X`) | warning |

## §7  Cross-skill rules (when chained from `fr-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The FR's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The FR's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |
| `XCHAIN-003` | If the FR was created via fr-with-tasks chain, the linked impl-plan path resolves | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff to operator. | warning → needs_human (`stale_artefact_disposition`) |

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
- `cyberos/skill/contracts/feature-request/CONTRACT.md` — the FR template this rubric audits.
- `cyberos/skill/contracts/feature-request/template.md` — the FR body skeleton.
