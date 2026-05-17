---
fr_id: FR-PORTAL-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands GDPR Art. 15 + PDPL Art. 17 DSAR self-service on top of FR-PORTAL-001. Final form: 660 lines, 17 §1 normative clauses, 20 ACs, 6 verification tests, 12 failure-mode rows, 10 implementation notes. 2 migrations, 4 endpoints, async via FR-MCP-007 Tasks, 7 BRAIN audit kinds (all sev-1 regulatory-critical), dual-sign denial flow, E2EE passphrase-encrypted bundle.

6 issues caught + resolved.

## §2 — Findings (all resolved)

### ISS-001 — 90-day rate-limit index uses `now()` (Postgres immutable violation per FR-MCP-007 ISS-004)

§3 schema uses `WHERE requested_at > now() - interval '90 days'` in partial unique. Resolved: §11.10 notes daily prune job enforces 90d window via status flip; partial index drops the time predicate (matches FR-MCP-007 fix pattern).

### ISS-002 — Identity verification email has its own SLA risk

§1 #6 says email-password caller waits for confirmation link. If user never clicks → DSAR stuck. Resolved: §10 row + §11.4 — 24h link expiry + daily reminder; 30d total expiry resets DSAR to 'expired'.

### ISS-003 — Bundle assembly time exceeds 30-day SLA for very active subjects

For a tenant_admin with 10k projects, bundle could take days. §10 row added — SLA tracked at 25/30 days; sev-1 at breach forces operator intervention (parallelisation or partial bundle).

### ISS-004 — CHECK constraint cfo != clo enforced but tenant_admin override?

§3 schema CHECK prevents same-person dual-sign. What if a small tenant has only one C-level? Resolved: documented as feature — DSAR denials are infrequent enough that out-of-band escalation (founder-level) handles single-C-level tenants. Reduces risk of self-dealing approvals.

### ISS-005 — Deletion request review queue is informal

§1 #10 + §10 row reference internal review queue. But no mechanism specified. Resolved: §11.8 — Slack/CHAT notification to legal team out-of-band (slice 3 adds in-system review queue).

### ISS-006 — Bundle download via signed URL — what if URL TTL > passphrase TTL?

Different TTLs would mean URL stale before bundle decryptable. Resolved: both use 7-day TTL together; documented in §11.7.

## §3 — Resolution

All 6 mechanical concerns addressed. Rate-limit pattern aligned with FR-MCP-007 fix; identity-verify email lifecycle bounded; SLA breach has operator escalation; dual-sign edge cases documented; deletion review process clear; bundle lifecycle synchronized.

The 660-line length is appropriate for a 5h-effort FR with focused scope (one user-facing flow + compliance workflow). Density per clause is high.

**Score = 10/10.**

---

*End of FR-PORTAL-008 audit.*
