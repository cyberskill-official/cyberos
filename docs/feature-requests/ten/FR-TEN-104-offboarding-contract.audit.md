---
fr_id: FR-TEN-104
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

FR-TEN-104 ships the 90-day offboarding contract — closed 4-state FSM + scheduled hourly advance + read-only freeze + dead-letter recovery + CSO+CLO dual-signoff irreversible wipe. Scope: 26 §1 normative clauses covering closed `offboarding_state` enum (active, terminating_a, terminating_b, terminated), FSM transition matrix with 6 valid transitions (5 forbidden), hourly scheduled advance job with SKIP LOCKED concurrency + DOES-NOT-auto-terminate-B-to-Terminated invariant, read-only freeze via FR-AUTH-004 JWT issuer hook + handler-side 423 with `ten.read_only_write_attempted` memory row, dead-letter wipe to S3 Object-Lock COMPLIANCE bucket (30-90 day per-tenant override), CSO+CLO dual-signoff with self-co-sign rejection + confirmation string match for terminate + dead-letter-restore, max 2 extensions per cycle via DB CHECK + CLO-only handler, FR-TEN-202 hostile fast-track support via active → terminating_b direct transition, terminated_at immutability via trigger, force-advance sev-1 audit for operator override, append-only log at SQL grant, 8 memory audit kinds with PII scrubbing, per-tenant total_grace_days [30,180] + dead_letter_retention_days [30,90], 60s cache on read-only gate matching FR-AUTH-109 pattern. 20 rationale paragraphs. §3 contains: 2 migrations (state with FSM trigger + provisioner grants + seed, log append-only), FSM validator with closed transition matrix, scheduler with SKIP LOCKED + dead-letter trigger, read-only gate with 60s TTL cache, initiate handler with grace-proportion split, finalize-termination handler with full dual-signoff + confirmation flow. 28 ACs. 31 failure-mode rows. 21 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Scheduler auto-terminating from terminating_b
First-pass had scheduled job auto-firing terminated transition. Resolved: §1 #6 + DEC-506 + notification-only + manual dual-signoff handler.

### ISS-002 — Single-signer terminate
First-pass had no dual-signoff. Resolved: §1 #11 + DEC-506 + CSO+CLO + self-co-sign rejection + confirmation string; AC #10-#13.

### ISS-003 — Read-only freeze not enforced
First-pass had no JWT-issuance hook. Resolved: §1 #7 + DEC-509 + read_only_gate cache + 423 + memory row; AC #17 + #18.

### ISS-004 — Cancellation in terminating_b silently succeeded
Resolved: §1 #9 + DEC-502 + handler state check + AC #5 + #6.

### ISS-005 — Extensions unbounded
First-pass had no cap. Resolved: §1 #18 + DEC-512 + DB CHECK extension_count <= 2 + AC #16.

### ISS-006 — terminated_at mutable post-set
Resolved: §1 #17 + trigger `terminated_at_immutable` + AC #21.

### ISS-007 — Append-only log writable from app role
Resolved: §1 #3 + DEC-508 + REVOKE UPDATE, DELETE; cyberos_provisioner role split for state UPDATE; AC #22 + #23.

### ISS-008 — Object-Lock mode unspecified (Governance vs Compliance)
Resolved: §1 #12 + DEC-505 + DEC-506 + COMPLIANCE mode hard-coded — root account can't delete dead letter; matches FR-DOC-001 pattern.

### ISS-009 — Dead-letter restore single-signer
First-pass had no dual-signoff on restore. Resolved: §1 #13 + DEC-513 + same CSO+CLO + sev-1 audit; AC #20.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (closed 4-state FSM × scheduled advance never-auto-terminate × CSO+CLO dual-signoff × read-only freeze via JWT issuer hook × dead-letter Object-Lock COMPLIANCE × per-tenant grace + retention overrides × extension cap via DB CHECK × terminated_at immutability × force-advance sev-1 × 8 memory audit kinds × append-only via SQL grant), not by line targets.

---

*End of FR-TEN-104 audit.*
