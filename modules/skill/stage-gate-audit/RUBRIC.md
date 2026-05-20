# `stage_gate_rubric@1.0` — machine-checkable stage-gate sign-off rubric

> Sourced from `../../../modules/cuo/README.md` Template §4.3 (Stage-gate sign-off one-page). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---`; closing `---` exists; YAML parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `stage-gate@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `project` | required, string | error | false |
| `FM-103` | `stage_name` | required, one of the 13 SDP §2 stage labels (a..m) or a custom value with `stage_custom: true` | error | false |
| `FM-104` | `gate_date` | required, ISO 8601 | error | true |
| `FM-105` | `gate_version` | required, SemVer | error | true |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-107` | `decision` | required, one of: go, go_with_conditions, no_go, deferred | error | false |
| `FM-108` | `decision_recorded_at` | required, ISO 8601 (matches gate meeting timestamp) | error | true |
| `FM-109` | `signers` | required, array of `{handle, role, signed_at}` covering at minimum EM, TL, Client_Sponsor | error | false |
| `FM-110` | `linked_project_plan` | required, must resolve to a project-plan path | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Stage` (the stage being closed; cite SDP §2 letter) | error |
| `SEC-002` | `## 2. Entry Criteria — Met?` (Y/N table with evidence link per criterion) | error |
| `SEC-003` | `## 3. Exit Criteria — Met?` (Y/N table with evidence link per criterion) | error |
| `SEC-004` | `## 4. Open Risks and Issues` (rows from the RAID log that remain open at this gate) | error |
| `SEC-005` | `## 5. Decision` (go / go_with_conditions / no_go / deferred + rationale) | error |
| `SEC-006` | `## 6. Conditions` (only required if decision = go_with_conditions; lists explicit conditions with owners + due dates) | error |
| `SEC-007` | `## 7. Signatures` (table mapping signer handle → role → signed_at + checkbox/initial) | error |
| `SEC-901` | Each required section is non-empty | error |
| `SEC-902` | Section ordering matches SEC-001..007 | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `decision = go_with_conditions` | §6 must enumerate ≥1 specific condition with `owner:` and `due:` | error |
| `COND-002` | `decision = no_go` | `## 8. Remediation Plan` block with corrective actions and a re-gate date | error |
| `COND-003` | `decision = deferred` | `## 8. Deferral Reason and Re-gate Date` block | error |
| `COND-004` | Stage = (i) Deployment | §3 Exit must include "DORA metrics baseline captured for this release" | warning |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-EVIDENCE-001` | Y in §2/§3 without evidence link | A Y row lacks an `evidence:` URL or relative path | error |
| `QA-EVIDENCE-002` | Y but evidence link doesn't resolve | The link is broken at audit time | warning |
| `QA-SIGN-001` | Missing required signer | §7 lacks one of EM, TL, Client Sponsor (the three minimum signers per Template §4.3) | error |
| `QA-COND-001` | Condition without owner or due date | §6 row lacks `owner:` or `due:` | error |
| `QA-DECISION-001` | Decision rationale missing | §5 has a decision but no rationale paragraph | error |
| `QA-RAID-001` | Open critical risk with no mitigation | §4 lists a Risk with `severity >= 4` (out of 5) and no `mitigation:` | error → needs_human (`regulatory_compliance` if risk is regulatory) |
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
| `XCHAIN-002` | `provenance.source_hash` matches author manifest at write time | error |
| `XCHAIN-003` | `linked_project_plan` resolves to a project-plan that passed project-plan-audit at 10/10 | warning |
| `XCHAIN-004` | The entry/exit criteria in §2/§3 match the DoR/DoD declarations on file for this project (if dor-dod artefact exists) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source plan hash differs | Reset open + needs_human to open | warning → needs_human |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `REPORT_FORMAT.md`, `INVARIANTS.md`
- `../../../modules/cuo/README.md` Template §4.3 — Stage-gate sign-off source
