# `audit_rubric@2.0` — machine-checkable Feature Request rubric

> Sourced from `cyberos/skill/contracts/feature-request/CONTRACT.md` (the FR contract body) and `cyberos/docs/Software Development Process.md` §2(b) Requirements. Rubric version `2.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in audit reports so reports are diffable across iterations and operators.

This rubric is a port of the proven rule set from the legacy `cuo/cpo/feature-request-audit` skill (audit_rubric@2.0, locked since 2026-02). It is preserved here verbatim because it has been battle-tested against 50+ FRs in the cyberos `docs/feature-requests/` catalog. Bumping to 3.0 requires governance sign-off.

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
| `FM-104` | `status` | required, one of: `draft`, `ready_to_implement`, `implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing`, `done`, `on_hold`, `closed` (per `docs/feature-requests/STATUS-REFERENCE.md` §1). Freeform tags like `[BLOCKED: ...]` / `[FAILED: ...]` are NO LONGER valid status values — failures route back to `ready_to_implement` (STATUS-REFERENCE §1.3). | error | false |
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

## §7  Cross-skill rules (when chained from `feature-request-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The FR's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The FR's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |
| `XCHAIN-003` | If the FR was created via fr-with-tasks chain, the linked impl-plan path resolves | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff to operator. | warning → needs_human (`stale_artefact_disposition`) |

## §9  Spec-vs-implementation traceability  *(applies to cyberos-style §1/§4/§5 FR template only)*

These rules apply to FRs that use the cyberos template (numbered §1 normative clauses · §4 acceptance criteria · §5 verification/tests), per `AUTHORING_DISCIPLINE.md` §1. Added 2026-05-18 (session 21) after the audit-fix loop on FR-AUTH-001 + FR-AUTH-006 surfaced 13 §1↔§4 / §4↔§5 traceability gaps in two "shipped" FRs — see memory feedback `feedback_fr_author_clause_to_test_traceability.md`. The upstream fix: refuse to score 10/10 if any §1 clause lacks a downstream test, so future FRs can't ship code that passes §5 tests while missing §1 clauses.

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `TRACE-001` | Every §1 numbered clause with a BCP-14 keyword (MUST · MUST NOT · SHOULD · SHOULD NOT · MAY) is cited by at least one §4 AC. Citation form: `§1 #N` or `§1.N` inside the AC's rationale or in the AC's `traces_to:` frontmatter field. Clauses explicitly tagged `(deferred to slice N)` in §1 are exempt. | error → needs_human (`spec_clause_without_ac`) | skeleton (insert AC stub with TODO marker linked to §1 #N) |
| `TRACE-002` | Every §4 AC cites at least one §5 verification entry — typically a named test function (e.g. `services/<crate>/tests/<file>.rs::<test_fn>`) OR a manual verification step with a rationale (manual is acceptable only for ops/UI flows that can't be automated, and must justify why). | error → needs_human (`ac_without_test`) | skeleton (insert §5 test-name placeholder) |
| `TRACE-003` | Every §5 test path is either listed in `frontmatter.new_files` (test file will be authored as part of this FR's implementation) OR resolves to an existing file on disk. Dangling test references (test name with no file) → fail. | error | false |
| `TRACE-004` | If `status: done`, every §1 clause's cited test is `passed` in the most recent `implementation_audit.coverage_report` (§10.3 audit-fix log). Tests in `implementing`/`ready_to_review`/`reviewing`/`ready_to_test`/`testing`/`draft`/`ready_to_implement` FRs are exempt (coverage is enforced separately by `coverage-gate-audit` during the `testing → done` transition). | error → needs_human (`done_with_untested_clause`) | false |
| `TRACE-005` | When an FR uses the deferred-slice pattern (e.g. "§1 #8 — deferred to slice 2"), §10.7 of the `.audit.md` MUST enumerate the deferred clauses with a scope estimate per the FR-AUTH-006 slice-2 precedent. Missing §10.7 with deferred clauses → fail. | warning | false |

**Rationale:** the audit-fix loop on FR-AUTH-001 surfaced 7 spec-vs-code gaps where §1 MUST clauses had no §4 AC or no §5 test backing them — the implementer passed all declared tests while quietly missing 7 normative clauses. TRACE-001..005 close that gap structurally: an FR can't score 10/10 (and thus can't move from `draft` → `ready_to_implement` → ... → `done`) if any of its §1 clauses lacks a downstream test. **The audit becomes the source of truth for "what's actually shipped" instead of `BACKLOG.md` status alone.**

**Phase ownership.** This skill (`feature-request-audit`) is the **spec correctness gate** — it drives the `draft → ready_to_implement` transition by verifying frontmatter, structure, traceability (TRACE-001..005), and quality heuristics on the spec itself. It does NOT enforce test coverage; that is the job of `coverage-gate-audit` during the `testing → done` transition. The two gates are deliberately separated so spec correctness can be verified before any implementation work begins (cheap early failure), and coverage can be verified independently once tests have run (expensive late failure).

**Worked example** (FR-AUTH-001's §1 #14 — `slug == "root"` defence-in-depth reject):
- §1 #14 says: `MUST NOT create a tenant with slug "root"`
- §4 AC #11 says: `Reserved-slug validator returns 400 with structured body before DB transaction (traces_to: §1 #14)`
- §5 test entry says: `services/auth/tests/admin_tenant_create_test.rs::create_tenant_rejects_reserved_root_slug` (covers ECM-008)
- §5 test file is in `frontmatter.new_files: [services/auth/tests/admin_tenant_create_test.rs, ...]`
- Pre-G-001 the file didn't exist on disk → TRACE-003 would have failed
- Post-G-001 the file + test exist → TRACE-003 + TRACE-001 + TRACE-002 all pass

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
