# EVAL module - task index

The `eval` module is CyberOS's people-evaluation surface: it measures work recorded on the platform against CyberSkill's three signed employment documents (Labor Contract; NDA / Non-compete / IP; Total Rewards & Career Path Appendix) and produces evidence-linked, human-reviewed assessments. It is the top of the BRAIN/EVAL workstream - the MEMORY brain (TASK-MEMORY-121..123) captures and recalls the evidence; `eval` turns that evidence into a governed, access-restricted evaluation. See [`../../strategy/cyberos-brain-evaluation-plan.md`](../../strategy/cyberos-brain-evaluation-plan.md) and BACKLOG.md §2.5.

Governing principles (founder decisions, 2026-06-29): wide day-1 capture; access-restricted (founder + manager-of-report + self) and contract-disclosed (not covert collection); auto-scoring AND a mandatory human-in-the-loop gate for anything affecting pay, progression, or employment; governance and capture first. Operating mode (2026-06-30): the product runs quiet / in-product-silent - no employee-facing monitoring or evaluation surface by default, access founder + managers only, employee self-view off by default (served on request via HR). The lawful-basis floor that is kept is the signed clause in [`../../legal/data-monitoring-and-evaluation-notice.md`](../../legal/data-monitoring-and-evaluation-notice.md) (EN + VN, for counsel review); fully-covert collection with no notice is out of scope.

## FRs

| FR | Priority | Step | Title |
|---|---|---|---|
| [TASK-EVAL-001](TASK-EVAL-001-governance-consent-access-retention/spec.md) | MUST | 0 governance | Governance, consent, access-control + retention - the Phase-0 gate every capture and evaluation depends on |
| [TASK-EVAL-002](TASK-EVAL-002-rubric-from-signed-documents/spec.md) | MUST | 3 rubric | Rubric from the three signed documents - versioned, bilingual VN/EN, each item cites its source clause |
| [TASK-EVAL-003](TASK-EVAL-003-evaluation-engine-autoscore-hitl/spec.md) | MUST | 4 engine | Evaluation engine - GENIE evidence-linked auto-scoring + mandatory HITL gate; results audit-chained |
| [TASK-EVAL-004](TASK-EVAL-004-manager-employee-views/spec.md) | MUST | 5 views | Manager + employee views - access-restricted console panel; auto-score shown as DRAFT pending human approval |

## Cross-module dependencies

- **AUTH**: TASK-EVAL-001 -> TASK-AUTH-003 (per-tenant RLS, roles); access grants extend the manager-of relationship.
- **MEMORY**: TASK-EVAL-003 -> TASK-MEMORY-123 (access-scoped recall with provenance); the whole module reads evidence captured via TASK-MEMORY-121/122 and chained into `l1_audit_log`.
- **AI / CUO**: TASK-EVAL-003 runs GENIE (Lumi) over the ai-gateway with spend caps, residency, and ZDR.
- **APP**: TASK-EVAL-004 -> TASK-APP-001 (the operator console shell it extends).

## Governance note

Not legal advice. The monitoring notice, lawful basis, retention terms, and the covert/disclosed boundary must be reviewed by Vietnamese counsel before go-live (Vietnam PDPD Decree 13/2023/ND-CP + Labor Code 45/2019/QH14 + Decree 145/2020).
