# `closure_rubric@1.0` — machine-checkable Project Closure rubric

> Sourced from `../../../modules/cuo/README.md` §2(l) Project closure and §6 (consultancy considerations: offboarding pack). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `closure@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `project` | required, string | error | false |
| `FM-103` | `client` | required, string | error | false |
| `FM-104` | `closure_date` | required, ISO 8601 | error | true |
| `FM-105` | `closure_version` | required, SemVer | error | true |
| `FM-106` | `linked_sow` | required, resolves to a SOW that passed statement-of-work-audit | error | false |
| `FM-107` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-108` | `client_nps` | required, integer 0-10 | error | false |
| `FM-109` | `signers` | required, array of `{handle, role, signed_at}` covering Client_Sponsor, EM, TL | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Sign-Off Certificate` (the formal client + CyberSkill acceptance text + signature table) | error |
| `SEC-002` | `## 2. Deliverables Accepted` (list of each SOW deliverable + acceptance date + acceptor) | error |
| `SEC-003` | `## 3. Lessons Learned` (compiled from per-iteration retros) | error |
| `SEC-004` | `## 4. Knowledge Transfer` (what was handed over; to whom; in what form) | error |
| `SEC-005` | `## 5. Source-Code Handover` (repo / branch / tag / access transferred) | error |
| `SEC-006` | `## 6. Runbook and Operations Handover` (runbook artefact references + on-call transition plan) | error |
| `SEC-007` | `## 7. Credentials Rotation` (which credentials were rotated; who holds them now) | error |
| `SEC-008` | `## 8. Asset Handover` (designs, contracts, third-party licenses, vendor accounts) | error |
| `SEC-009` | `## 9. Closure Metrics` (on-time delivery %, on-budget %, defect leakage, DORA at closure) | error |
| `SEC-010` | `## 10. Client NPS and Verbatim Feedback` | error |
| `SEC-011` | `## 11. Surviving Obligations` (warranty, support, NDA, IP, audit-rights) | error |
| `SEC-012` | `## 12. Next-Steps Proposal` (renewal / phase-2 / referenceability discussion) | warning |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Engagement model was `dedicated_team` or `staff_augmentation` | `## 13. People Offboarding` (re-deployment plan, retention check-ins, alumni network membership) | error |
| `COND-002` | Engagement included personal-data processing | `## 14. Data Disposition` (deletion / return / continued processing per DPA terms) | error → needs_human (`legal_compliance`) |
| `COND-003` | Engagement reaches end of contracted warranty period | `## 15. Warranty Expiry Notice` | warning |
| `COND-004` | Engagement was `managed_services` | `## 16. Service Disengagement Plan` (alternative provider hand-off, knowledge embed) | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-DELIVERABLE-001` | Deliverable in SOW missing from §2 | A SOW §3 deliverable lacks an acceptance row | error |
| `QA-DELIVERABLE-002` | Deliverable accepted without acceptor handle | A row in §2 lacks `accepted_by:` | error |
| `QA-CRED-001` | Credentials rotation incomplete | §7 lists creds without `rotated_at:` and `new_holder:` for each | error |
| `QA-RUNBOOK-001` | Runbook handover but linked runbook is not at 10/10 audit | warning |
| `QA-METRIC-001` | Closure metric without numeric value | §9 row lacks number | error |
| `QA-NPS-001` | NPS out of range | `client_nps` not in 0-10 | error |
| `QA-NPS-002` | NPS ≤6 (detractor) without explanation | A detractor NPS with no §10 verbatim narrative | warning → needs_human |
| `QA-SIGN-001` | Missing signer | §1 lacks one of the three required signer roles | error |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | `linked_sow` resolves to a SOW that passed statement-of-work-audit at 10/10 | error |
| `XCHAIN-004` | Every deliverable in linked SOW is enumerated in §2 | error |
| `XCHAIN-005` | Runbook references in §6 are at 10/10 audit | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Linked SOW hash differs | Reset open + needs_human | warning → needs_human |
| `STALE-002` | `closure_date` is in the future | warning (acceptable pre-announce) |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/README.md` §2(l) — Closure source
- `../../../modules/cuo/README.md` §6 — Offboarding pack source
