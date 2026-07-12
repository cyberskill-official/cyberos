# `project_plan_rubric@1.0` — machine-checkable project plan rubric

> Sourced from `../../../modules/cuo/docs/module.md` §2(c) Feasibility and project planning; PMBOK 8th Edition (performance domains, May 2026 release); PRINCE2 7th Edition (five integrated elements: principles, people, practices, processes, project context). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---`; closing `---` exists; YAML parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `project-plan@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` (typically the EM) | error | false |
| `FM-103` | `project` | required, string | error | false |
| `FM-104` | `linked_srs` | recommended; if present, must resolve to an SRS path | warning | false |
| `FM-105` | `linked_sow` | recommended; if present, must resolve to a SOW path | warning | false |
| `FM-106` | `engagement_model` | required, one of: fixed_price, time_and_materials, dedicated_team, staff_augmentation, managed_services | error | false |
| `FM-107` | `plan_version` | required, SemVer | error | true |
| `FM-108` | `effective_date` | required, ISO 8601 | error | true |
| `FM-109` | `target_close_date` | required, ISO 8601 | error | false |
| `FM-110` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-111` | `governance_framework` | required, one of: pmbok_8, prince2_7, hybrid_lite, none | error | false |

## §3  Always-required sections (per SDP §2(c))

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Technical Feasibility Memo` (tech spikes summary, build-vs-buy, key risks) | error |
| `SEC-002` | `## 2. Cost/Benefit Analysis` | error |
| `SEC-003` | `## 3. Schedule and Milestones` (with target dates + dependencies) | error |
| `SEC-004` | `## 4. RAID Log` (Risks, Assumptions, Issues, Dependencies) | error |
| `SEC-005` | `## 5. Communication Plan` (cadence, channels, recipients) | error |
| `SEC-006` | `## 6. Resourcing Plan` (team composition + capacity assumptions) | error |
| `SEC-007` | `## 7. Quality Plan` (test approach summary + acceptance gate strategy) | error |
| `SEC-008` | `## 8. Definition of Ready / Done` (or pointer to project's DoR/DoD doc) | error |
| `SEC-009` | `## 9. Change-Control Process` | error |
| `SEC-010` | `## 10. Approval and Sign-off` (named approvers with `role:`) | error |
| `SEC-901` | Each required section is non-empty | error |
| `SEC-902` | Section ordering matches SEC-001..010 | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `governance_framework = pmbok_8` | `## 11. PMBOK Performance Domains Mapping` covering all eight domains (stakeholders, team, development approach, planning, project work, delivery, measurement, uncertainty) | error |
| `COND-002` | `governance_framework = prince2_7` | `## 11. PRINCE2 Elements Mapping` covering the five integrated elements (principles, people, practices, processes, project context); the new "Issues" practice replaces the legacy "Change" theme | error |
| `COND-003` | `engagement_model = fixed_price` | `## 12. Stage-Gate Plan` enumerating each gate's entry + exit criteria | error |
| `COND-004` | Project is regulated (GDPR / HIPAA / SOX / Vietnam Decree 13/2023 PDPD / etc.) | `## 13. Regulatory Compliance Plan` enumerating each rule + responsible party | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph without `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-NUM-001` | Unsourced cost/timeline number | numeric target in §2/§3 without citation | error → needs_human (`success_metric_targets`) |
| `QA-RAID-001` | RAID entry without owner | a Risk / Issue / Dependency lacks `owner:` and `due:` | error |
| `QA-RAID-002` | RAID entry without severity | a Risk lacks `likelihood:` (1-5) + `impact:` (1-5) | error |
| `QA-MILE-001` | Milestone without acceptance gate | §3 milestone lacks `acceptance_gate:` | warning |
| `QA-CAP-001` | Capacity not declared | §6 lacks per-person FTE percent + PTO assumptions | warning |
| `QA-COMMS-001` | Communication cadence vague | §5 lacks at least: daily standup, weekly client status, monthly steering (per SDP §6 governance cadence) | warning |
| `QA-FEAS-001` | Feasibility memo lacks build-vs-buy | §1 doesn't address build-vs-buy or explicitly note "all custom build" | warning |
| `QA-TODO` | Skeleton TODO marker remaining | literal `TODO:` | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Injection-marker scan inside `<untrusted_content>` | warning (error if ≥3) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches author manifest at write time | error |
| `XCHAIN-003` | `linked_sow` resolves to a SOW that passed statement-of-work-audit at 10/10 | warning |
| `XCHAIN-004` | `linked_srs` resolves to an SRS that passed software-requirements-specification-audit at 10/10 | warning |
| `XCHAIN-005` | `engagement_model` in this plan matches the linked SOW's `engagement_model` | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source SRS/SOW hash differs | Reset open + needs_human to open | warning → needs_human (`stale_artefact_disposition`) |
| `STALE-002` | `effective_date` is >90 days old | Suggest revision cycle | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `REPORT_FORMAT.md`, `INVARIANTS.md`
- `../../../modules/cuo/docs/module.md` §2(c) — Feasibility & project planning source
- PMBOK 8th Edition — May 2026 release
- PRINCE2 7th Edition
