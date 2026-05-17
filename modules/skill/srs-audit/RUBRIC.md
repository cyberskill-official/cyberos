# `srs_rubric@1.0` — machine-checkable Software Requirements Specification rubric

> Sourced from `cyberos/skill/contracts/srs/CONTRACT.md`, IEEE 830-1998 (Recommended Practice for Software Requirements Specifications), and ISO/IEC 25010:2023 (the nine quality characteristics). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `srs@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–120 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `product` | required, string | error | false |
| `FM-104` | `srs_version` | required, SemVer | error | true |
| `FM-105` | `created_at` | required, ISO 8601 with timezone | error | true |
| `FM-106` | `status` | required, one of: draft, in_review, approved, archived | error | false |
| `FM-107` | `linked_prd` | optional but recommended; if present, must resolve to a PRD path | warning | false |
| `FM-108` | `provenance.source_path` | required, path exists, readable | error | false |
| `FM-109` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |
| `FM-110` | `ieee_830_compliance` | required, boolean. If `true`, additional `IEEE-NNN` rules in §9 apply. | error | false |
| `FM-111` | `iso_25010_quality_chars_covered` | required, array of `{name, status}` pairs covering ALL nine ISO/IEC 25010:2023 characteristics. `status` ∈ `covered | waived | not_applicable` | error | false |

## §3  Always-required sections (IEEE 830 §5 outline)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Introduction` with H3s `### 1.1 Purpose`, `### 1.2 Scope`, `### 1.3 Definitions, Acronyms, Abbreviations`, `### 1.4 References`, `### 1.5 Overview` | error |
| `SEC-002` | `## 2. Overall Description` with H3s `### 2.1 Product Perspective`, `### 2.2 Product Functions`, `### 2.3 User Characteristics`, `### 2.4 Constraints`, `### 2.5 Assumptions and Dependencies` | error |
| `SEC-003` | `## 3. Specific Requirements` | error |
| `SEC-004` | `### 3.1 Functional Requirements` under §3 | error |
| `SEC-005` | `### 3.2 Non-Functional Requirements` under §3 (the ISO/IEC 25010 mapping block) | error |
| `SEC-006` | `### 3.3 External Interface Requirements` under §3 (user / hardware / software / communications) | error |
| `SEC-007` | `### 3.4 Performance Requirements` under §3 | error |
| `SEC-008` | `### 3.5 Design Constraints` under §3 | error |
| `SEC-009` | `## 4. Appendices` (glossary, models, mock-ups — at least one populated) | warning |
| `SEC-901` | Each required section is non-empty (≥1 non-blank line of body) | error |
| `SEC-902` | Section + sub-section ordering matches the IEEE 830 outline | warning |
| `SEC-903` | Each REQ-ID is unique within the SRS | error |
| `SEC-904` | Each REQ-ID follows the pattern `^REQ-[A-Z]{1,4}-\d{3,}$` (e.g. `REQ-AUTH-001`) | error |
| `SEC-905` | Each requirement has a `priority:` of `must | should | could | wont` (MoSCoW) | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `product` is AI-driven | `### 3.6 AI Risk and Compliance Requirements` under §3 enumerating EU AI Act risk class + mitigation requirements | error |
| `COND-002` | Product handles personal data | `### 3.7 Privacy & Data-Protection Requirements` under §3 referencing applicable regulations | error |
| `COND-003` | Product is multi-tenant or hosted in multiple regions | `### 3.8 Tenancy & Residency Requirements` under §3 | error |
| `COND-004` | Product replaces an existing system | `### 3.9 Migration & Backwards-Compatibility Requirements` under §3 | error |
| `COND-005` | Product is safety-critical (medical / aerospace / automotive / nuclear) | `### 3.10 Safety Requirements` under §3 enumerating hazards + mitigations + V&V plan | error |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | Any paragraph in a non-boilerplate section lacks a `source_ref` marker | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Any paragraph in a non-boilerplate section lacks an `authority` marker | error |
| `QA-REQ-001` | Ambiguous "should" / "may" / "could" in §3.1 Functional Requirements (only `must` is allowed for functional shalls) | grep for `\\b(should\\|may\\|could\\|might)\\b` in functional REQ blocks | warning |
| `QA-REQ-002` | Requirement without acceptance criteria | A `REQ-*` block lacks a `Given/When/Then` or equivalent acceptance criterion | error |
| `QA-REQ-003` | Untestable requirement | A `REQ-*` lacks a `verification_method:` of `inspection | analysis | demonstration | test` (per IEEE 830 §4.3.8) | warning |
| `QA-NFR-001` | Missing ISO/IEC 25010 characteristic | Any of the 9 characteristics in `iso_25010_quality_chars_covered` lacks at least one corresponding REQ block in §3.2 (unless `waived` or `not_applicable` with reason) | error → needs_human (`nfr_coverage`) |
| `QA-NFR-002` | NFR without numeric target | A non-functional REQ lacks a measurable target (percentile, ms, MB, error rate, etc.) | warning |
| `QA-NUM-001` | Unsourced numeric target | Any quantitative target not derivable from inputs | error → needs_human (`success_metric_targets`) |
| `QA-CONFLICT-001` | Conflicting requirements | Two REQ blocks with mutually exclusive targets in overlapping scope (heuristic: same actor, same action, contradictory rate/value) | warning → needs_human (`scope_decomposition`) |
| `QA-TRACE-001` | Untraceable requirement | A REQ block lacks a `linked_prd_section:` or `linked_sow_section:` reference | warning |
| `QA-DEP-001` | Cross-team dependency without owner | An external dependency in §2.5 names another team without ticket/owner/commitment | warning → needs_human (`cross_team_dependency`) |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | Quoted passage not wrapped | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` | warning (open until human resolves) |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (per `references/UNTRUSTED_CONTENT.md` §3) | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor | warning |

## §7  Cross-skill rules (when chained from `prd-audit` or `srs-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The SRS's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The SRS's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |
| `XCHAIN-003` | `linked_prd` (if present) resolves to a PRD that passed prd-audit at 10/10 | warning |
| `XCHAIN-004` | Every REQ-ID in §3 maps back to at least one PRD use case (when `linked_prd` is set) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source PRD/SOW/brief hash differs from SRS's `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff. | warning → needs_human (`stale_artefact_disposition`) |

## §9  IEEE 830 conformance rules (fire only when `ieee_830_compliance: true`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `IEEE-001` | §1.2 Scope identifies the product by name + primary benefits/objectives | error |
| `IEEE-002` | §1.3 Definitions block has ≥1 entry per non-obvious term used in the SRS | warning |
| `IEEE-003` | §2.1 Product Perspective enumerates external interfaces (other systems, hardware, users) | error |
| `IEEE-004` | §2.4 Constraints lists at least: regulatory, hardware, language/standard | warning |
| `IEEE-005` | §3 Specific Requirements uses one of the IEEE 830 organisation styles (by mode / by user class / by object / by feature / by stimulus / by response / by functional hierarchy / mixed). Declared via `srs_organisation_style:` frontmatter field. | error |
| `IEEE-006` | Each REQ block satisfies the IEEE 830 §4.3 quality criteria: correct, unambiguous, complete, consistent, ranked, verifiable, modifiable, traceable | error → needs_human |

## §10  ISO/IEC 25010:2023 quality-characteristic mapping

For each of the 9 characteristics listed in `iso_25010_quality_chars_covered`, the audit checks for at least one REQ-NFR-* block in §3.2 (unless waived). The 9 characteristics are:

| char | example sub-characteristics |
| ---- | --------------------------- |
| functional_suitability | functional completeness, correctness, appropriateness |
| performance_efficiency | time behaviour, resource utilisation, capacity |
| compatibility | co-existence, interoperability |
| interaction_capability | appropriateness recognisability, learnability, operability, user error protection, user engagement, **inclusivity** (new in 2023), self-descriptiveness (new in 2023), accessibility |
| reliability | faultlessness (renamed from maturity in 2023), availability, fault tolerance, recoverability |
| security | confidentiality, integrity, non-repudiation, accountability, authenticity, **resistance** (new in 2023) |
| maintainability | modularity, reusability, analysability, modifiability, testability |
| flexibility | adaptability, scalability (new in 2023), installability, replaceability |
| safety | operational constraint, risk identification, fail safe, hazard warning, safe integration |

Rule `QA-NFR-001` (§5) is what enforces this mapping at audit time.

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
- `REPORT_FORMAT.md` — `.audit.md` shape.
- `INVARIANTS.md` — invariant catalog.
- `cyberos/skill/contracts/srs/CONTRACT.md` — the SRS template.
- IEEE 830-1998 §5 (SRS outline) — §3 sources.
- ISO/IEC 25010:2023 — §10 source.
