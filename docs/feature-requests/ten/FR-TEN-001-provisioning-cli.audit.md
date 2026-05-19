---
fr_id: FR-TEN-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-TEN-001 ships the `cyberos-ten provision` CLI — the ops-driven tenant lifecycle primitive. Scope: 26 §1 normative clauses covering 2 closed Postgres enums (TenantStatus 5, ResidencyCode 4), 8-step transactional orchestration (validate → tenant row → schema → NATS → S3 → AUTH bootstrap → status flip → memory audit), per-tenant Postgres schema namespace, NATS subject namespace, S3 marker prefix, root-admin password printed-once + zeroised, append-only tenants + status history at SQL grant, idempotency on slug, exit-code mapping per cyberos-cli-exit, operator root-admin role gate, `--dry-run` mode for FR-TEN-101 preflight, `--json` mode with password-to-stderr, tenant_residency_map for FR-DOC-001/FR-EMAIL-001/FR-AI-016 consumption, memory chain anchor, OTel emission. 22 rationale paragraphs. §3 contains: 3 migrations (tenants + status history + residency map with cyberos_provisioner role), Rust types, orchestrator with 8-step flow + compensating actions, CLI subcommand with exit codes. 30 ACs. 32 failure-mode rows. 24 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Self-serve signup conflated with ops-driven CLI
First-pass tried to ship both. Resolved: §1 split + DEC-320 — ops CLI at slice 1; FR-TEN-101 ships self-serve at P3.

### ISS-002 — RLS-only isolation (no namespace defense in depth)
First-pass relied solely on RLS. Resolved: §1 #6 + DEC-321 + DEC-322 + DEC-323 — per-tenant Postgres schema + NATS namespace + S3 prefix; AC #22 + #23 + #24.

### ISS-003 — Root admin password leaked into logs / memory
First-pass logged password. Resolved: §1 #7 + DEC-331 + zeroize crate + AC #16 + #17.

### ISS-004 — Provisioning partial failures left orphan state
First-pass had no compensating actions. Resolved: §1 #6 + compensating rollback per step; AC #26.

### ISS-005 — `cyberos_app` role could mutate tenants
First-pass had no SQL role split. Resolved: §1 #10 + #11 + cyberos_provisioner role distinct from cyberos_app; AC #11 + #12.

### ISS-006 — Slug not validated against multi-system constraints
First-pass had loose regex. Resolved: §1 #1 + `^[a-z][a-z0-9-]{2,40}[a-z0-9]$` — safe for Postgres schema + NATS subject + S3 prefix + URL; AC #6.

### ISS-007 — Idempotency semantics ambiguous
First-pass returned exit 0 on idempotent match (indistinguishable from new). Resolved: §1 #5 + DEC-329 + DEC-330 — exit 1 idempotent_match distinct from exit 0; AC #4.

### ISS-008 — Operator role check missing
First-pass let any caller provision. Resolved: §1 #14 + root-admin role gate; AC #9.

### ISS-009 — Closed enum cardinality drift
First-pass had no test asserting Rust + SQL enum size match. Resolved: AC #1 + #2 + closed-enum cardinality tests in §5.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (CLI × 8-step orchestration × per-tenant namespace × append-only history × root-admin password printed-once + zeroised × idempotency × operator role gate × dry-run × residency map × memory chain anchor), not by line targets.

---

*End of FR-TEN-001 audit.*
