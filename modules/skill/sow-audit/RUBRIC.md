# `sow_rubric@1.0` — machine-checkable audit rubric

> Sourced from `cyberos/docs/Software Development Process.md` §4.9 (the SOW skeleton) and §6 (consultancy considerations: IP, confidentiality, AI-use policy). Rubric version `1.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in the audit report so reports are diffable across iterations and operators.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `sow@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–120 chars after trimming | error | skeleton |
| `FM-102` | `client_name` | required, string, length 1–120 chars | error | false |
| `FM-103` | `client_legal_entity` | required, string (full legal name + jurisdiction) | error | false |
| `FM-104` | `engagement_model` | required, one of: fixed_price, time_and_materials, dedicated_team, staff_augmentation, managed_services | error | false |
| `FM-105` | `effective_date` | required, ISO 8601 date | error | true |
| `FM-106` | `target_close_date` | required, ISO 8601 date | error | false |
| `FM-107` | `sow_version` | required, SemVer | error | true |
| `FM-108` | `provenance.source_path` | required, path exists, readable | error | false |
| `FM-109` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |
| `FM-110` | `cs_signer`, `em_signer`, `cyberskill_signer` | required handles for each role | error | false |
| `FM-111` | `governing_law` | required, free string (e.g. "Vietnam", "Delaware, USA") | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Objectives and Success Criteria` | error |
| `SEC-002` | `## 2. Scope` | error |
| `SEC-003` | `## 3. Deliverables` | error |
| `SEC-004` | `## 4. Assumptions and Constraints` | error |
| `SEC-005` | `## 5. Engagement Model` | error |
| `SEC-006` | `## 6. Team and Roles` | error |
| `SEC-007` | `## 7. Schedule and Milestones` | error |
| `SEC-008` | `## 8. Pricing and Invoicing` | error |
| `SEC-009` | `## 9. Acceptance Criteria` | error |
| `SEC-010` | `## 10. IP and Confidentiality` | error |
| `SEC-011` | `## 11. Change Control` | error |
| `SEC-012` | `## 12. Warranty, Support, and Governance Cadence` | error |
| `SEC-901` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-902` | Section ordering matches SEC-001..012 exactly | error |
| `SEC-903` | A `[WAIVED]` placeholder is accompanied by a `reason:` line explaining why | error |
| `SEC-904` | Heading hierarchy well-formed (no H2→H4 jumps; one H1 only — the SOW title) | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `engagement_model = fixed_price` | `### Fixed-Price Terms` subsection under §5 with milestone-tied invoice schedule | error |
| `COND-002` | `engagement_model = time_and_materials` | `### Rate Card` subsection under §5 + `### Weekly Timesheet Policy` under §8 | error |
| `COND-003` | `engagement_model = dedicated_team` | `### Team Composition` under §5 with named roles + named individuals (or hiring window) | error |
| `COND-004` | `engagement_model = staff_augmentation` | `### Performance Management` under §6 stating CyberSkill retains performance oversight | error |
| `COND-005` | `engagement_model = managed_services` | `### SLA Definitions` under §12 with availability + MTTR commitments | error |
| `COND-006` | Source brief mentions personal data | `### Data Processing Addendum` reference under §10 + sub-processor list | error |
| `COND-007` | Client based in / data residency includes EU | `### GDPR Addendum` reference under §10 | error |
| `COND-008` | Client based in / data residency includes Vietnam | `### Vietnam Compliance` block under §10 (Decree 13/2023 PDPD + Decree 53/2022 cybersecurity) | error |
| `COND-009` | Source brief mentions PHI / health data | `### HIPAA-aligned Controls` under §10 + escalation to cuo-cseco | error |
| `COND-010` | `engagement_model ∈ {dedicated_team, staff_augmentation, managed_services}` | `### On-call Coverage` under §12 (if applicable) | warning |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | Any paragraph in a non-boilerplate section lacks a `source_ref` marker | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Any paragraph in a non-boilerplate section lacks an `authority` marker | error |
| `QA-NUM-001` | Unsourced price/timeline | Numeric target in §7 / §8 without a citation to the source brief or operator reply | error → needs_human (`pricing_terms` or `acceptance_criteria`) |
| `QA-IP-001` | Vague IP language | §10 contains "TBD", "to be determined", or "standard terms apply" without referencing CyberSkill's IP boilerplate id | error → needs_human (`ip_assignment`) |
| `QA-SCOPE-001` | Vague scope boundaries | §2 lacks a `### Out of Scope` subsection, or §2.Out-of-Scope contains <2 bullets | warning |
| `QA-AI-001` | Missing AI-use disclosure | §10 lacks a paragraph naming permitted AI tools and data-perimeter rules per SDP §5 | error |
| `QA-RACI-001` | Incomplete RACI | §6 lacks roles for CS, EM, PO, TL, AR, DEV, QA, DO, SEC (or doesn't justify omission per engagement model) | error |
| `QA-MILE-001` | Milestone without acceptance gate | A milestone in §7 lacks an `acceptance_gate:` reference | warning |
| `QA-DPA-001` | Data-processing addendum referenced but no sub-processor list | §10 mentions DPA but no enumeration of sub-processors | error |
| `QA-WAIVE-001` | Waiver without operator handle | A `[WAIVED]` section has no `waived_by:` operator handle | error |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | Quoted passage from source not wrapped | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` from a skeleton stub | warning (open until human resolves) |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (per `references/UNTRUSTED_CONTENT.md` §3) | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor | warning |

## §7  Cross-skill rules (when chained from `sow-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The SOW's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The SOW's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source brief hash differs from SOW's `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff to operator. | warning → needs_human (`stale_artefact_disposition`) |

## §9  Compliance-aware rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `COMP-GDPR-001` | If `COND-007` fires, the GDPR addendum reference resolves to an actual document path in this engagement folder OR to a versioned BRAIN memory_id | error |
| `COMP-VN-001` | If `COND-008` fires, the Vietnam compliance block names both Decree 13/2023 PDPD and Decree 53/2022 cybersecurity | error |
| `COMP-AI-001` | The §10 AI-use disclosure paragraph specifies (a) permitted tools, (b) data-perimeter rules, (c) AI-assisted PR labelling commitment per SDP §5 | error |

---

## Rule auto-fix behaviour catalogue

| auto-fixable value | Audit behaviour |
| ------------------ | --------------- |
| `true` | Minimal textual change; mark `fixed`. |
| `false` | Leave `open` or mark `needs_human` per severity. |
| `skeleton` | Insert TODO marker; mark `open` with `todo_inserted: true`. |

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the 8-step algorithm that walks this rubric.
- `cyberos/skill/docs/RUBRIC_FORMAT.md` — the rubric column format every audit skill follows.
- `REPORT_FORMAT.md` (sibling file) — the on-disk shape of `.audit.md` reports.
- `INVARIANTS.md` (sibling file) — invariant catalog that includes the `deterministic_drift` check.
- `cyberos/docs/Software Development Process.md` §4.9 — the SOW skeleton source.
- `cyberos/docs/Software Development Process.md` §6 — IP, confidentiality, AI-use policy source.
