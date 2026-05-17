# `postmortem_rubric@1.0` — machine-checkable Post-mortem rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(j) Operations incidents; Google SRE Book (blameless post-mortem culture). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `postmortem@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `incident_id` | required, string (Linear / Jira / PagerDuty incident ref) | error | false |
| `FM-103` | `severity` | required, one of: sev1, sev2, sev3, sev4, sev5 | error | false |
| `FM-104` | `started_at`, `detected_at`, `resolved_at` | required, ISO 8601 (all three) | error | false |
| `FM-105` | `duration_minutes` | required, integer (computed from `started_at` → `resolved_at`) | error | true |
| `FM-106` | `mttd_minutes` | required, integer (`detected_at` − `started_at`) | error | true |
| `FM-107` | `mttr_minutes` | required, integer (`resolved_at` − `detected_at`) | error | true |
| `FM-108` | `services_affected` | required, array of service names | error | false |
| `FM-109` | `customer_impact` | required, string (concrete: # of customers / requests / dollars) | error | false |
| `FM-110` | `provenance.source_path`, `provenance.source_hash` | required (incident timeline export) | error | false |
| `FM-111` | `facilitator` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-112` | `participants` | required, array of `^@[A-Za-z0-9_.-]{1,38}$` (>=3) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Incident Summary` (1-2 paragraphs: what happened, impact, resolution) | error |
| `SEC-002` | `## 2. Timeline` (per-minute or per-event log; minutes ≥ from incident start) | error |
| `SEC-003` | `## 3. Customer Impact` (numbers — affected users / failed requests / revenue loss) | error |
| `SEC-004` | `## 4. Detection` (how we found out; MTTD analysis; could we have detected earlier) | error |
| `SEC-005` | `## 5. Response` (what we did; what worked; what didn't) | error |
| `SEC-006` | `## 6. Contributing Factors` (Five-Whys analysis; technical + process + organizational) | error |
| `SEC-007` | `## 7. What Went Well` | error |
| `SEC-008` | `## 8. What Went Wrong` | error |
| `SEC-009` | `## 9. Where We Got Lucky` | warning |
| `SEC-010` | `## 10. Action Items` (table: title, owner, due_date, linked_ticket, severity) | error |
| `SEC-011` | `## 11. Lessons Learned` | error |
| `SEC-012` | `## 12. SLO/SLA Impact` (error-budget burn calculation; per-SLO impact) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `severity ∈ {sev1, sev2}` | `## 13. Public Communication Log` (status-page entries, customer emails, post-incident comms) | error |
| `COND-002` | Incident involved personal-data exposure | `## 14. Data Breach Assessment` (GDPR Art. 33 / Vietnam Decree 13/2023 timeline + notifications) | error → needs_human (`legal_compliance`) |
| `COND-003` | Incident involved security exploit | `## 15. Security Disclosure` (CVE plan if applicable, customer advisory plan) | error → needs_human (`legal_compliance`) |
| `COND-004` | `severity = sev1` | `## 16. Executive Brief` (≤200 words for leadership) | error |
| `COND-005` | Incident touched AI/ML system | `## 17. Model Behaviour Analysis` (drift / regression / fall-back trigger) | warning |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-BLAME-001` | Blameful language | Body contains: "operator error", "X should have known", "user fault", or names with negative attribution | error → needs_human (`scope_decomposition`) — recompose blamelessly |
| `QA-TIMELINE-001` | Timeline gaps >15 min during active incident | error |
| `QA-TIMELINE-002` | Timeline entry without source | A row in §2 lacks `source:` (log query / Slack permalink / PagerDuty event) | warning |
| `QA-5WHY-001` | Contributing factors lacks Five-Whys depth | §6 has <3 "why" levels | warning |
| `QA-ACTION-001` | Action item without owner | A row in §10 lacks `owner:` | error |
| `QA-ACTION-002` | Action item without due date | A row in §10 lacks `due_date:` | error |
| `QA-ACTION-003` | Action item without linked ticket | A row in §10 lacks `linked_ticket:` (or `ticket: TBD` warning) | warning |
| `QA-MTTR-001` | MTTR > SLO without rationale | `mttr_minutes` > runbook-declared SLO for this service AND no `mttr_explanation:` block | warning |
| `QA-IMPACT-001` | Customer impact vague | §3 has only "some customers affected" without numbers | error |
| `QA-PUBLIC-001` | sev1/sev2 without public-comms log | warning → needs_human |
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
| `XCHAIN-003` | Every `services_affected` has a runbook artefact (else flag for runbook creation) | warning |
| `XCHAIN-004` | Every action item with `due_date` <30 days lands in the linked project tracker (verified via tool link if available) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source incident timeline updated since post-mortem `provenance.source_hash` | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | Action items in §10 past `due_date` and not closed | warning (track separately in a follow-up audit) |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/docs/Software Development Process.md` §2(j) — Operations source
- Google SRE Book — Blameless Post-mortem culture
- GDPR Art. 33 — 72-hour data-breach notification
- Vietnam Decree 13/2023 PDPD — data-protection regime
