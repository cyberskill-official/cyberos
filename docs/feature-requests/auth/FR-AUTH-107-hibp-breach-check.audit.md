---
fr_id: FR-AUTH-107
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-AUTH-107 ships HIBP k-anonymity breach check at signup + rotation + admin-set + password-reset (NOT login) with per-tenant threshold + per-tenant unreachable policy + air-gapped local dump fallback. Scope: 25 §1 normative clauses covering closed 3-value `hibp_decision` enum (allowed, rejected, unreachable), closed 2-value `hibp_unreachable_policy` (fail_open, fail_closed), closed 4-value `hibp_code_path` (signup, rotation, admin_set, password_reset), 5-char SHA-1 prefix k-anonymous range API (plaintext + full hash never leave process), per-tenant `hibp_block_threshold` [1, 10000] default 5 with security_admin role gate + reason + sev-2 audit, in-process LRU cache 10000 entries × 1h TTL, 2s timeout + 1 connect-retry + treat 429/5xx as unreachable, 429-escalation (3 in 5min → sev-1), 10-permit concurrency semaphore, append-only `hibp_audit_log` table at SQL grant (REVOKE UPDATE/DELETE) with RLS + audit row carries prefix + count + decision only (NO plaintext, NO full hash, NO PII), local k-anon dump fallback (`CYBEROS_HIBP_LOCAL_DUMP_PATH` env) with fail-loud startup when dir empty, dev whitelist env `CYBEROS_HIBP_DEV_WHITELIST` honored ONLY in development + sev-1 startup audit if set in prod, HIBP UA `cyberos-auth/<version>` + Add-Padding NOT enabled (cache hides traffic patterns), HIBP check runs BEFORE Argon2 (DEC + ordering test) so rejected passwords don't burn 100ms of CPU, login NEVER invokes HIBP (verified by zero-call test), live-canary CI test against `21BD1` prefix asserts "password" count > 1M (nightly only), dry_run preview returns effective config + affected subject count + no DB mutation + no audit row, 6 memory audit kinds (hibp_check_passed sev-3, hibp_check_rejected sev-2, hibp_unreachable_fail_open sev-3, hibp_unreachable_fail_closed sev-2, hibp_rate_limited sev-2 escalating sev-1, hibp_policy_changed sev-2), memory_chain_hash dual-link to Postgres row, per-tenant policy cache 60s TTL with pg_notify invalidation. 22 rationale paragraphs. §3 contains: migration with all 3 closed enums + per-tenant policy columns + RLS + audit table grants, HibpClient implementation with cache + semaphore + local-dump + retry, check() function with policy enforcement + audit emission. 32 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Full hash or plaintext could leak to network
First-pass had no k-anonymity. Resolved: §1 #2 + DEC-720 + 5-char prefix only + GET /range/{prefix} + CI network-capture test asserts request body ≤ 5 hex chars; AC #10 + #27.

### ISS-002 — Argon2 invoked before HIBP (CPU burn on credential-stuffing)
Resolved: §1 #13 + ordering test + handler structure: validate → HIBP → Argon2 → persist; AC #32.

### ISS-003 — Login could invoke HIBP (latency burn + redundant)
First-pass auto-applied to all password paths. Resolved: §1 #1 + DEC-728 + login NEVER invokes HIBP + zero-call test; AC #9.

### ISS-004 — Network unreachable defaulted to fail-closed (locks out tenants)
Resolved: §1 #8 + DEC-725 + DEC-726 + default fail_open + sev-3 audit + per-tenant override with security_admin + sev-2; AC #17 + #18.

### ISS-005 — Threshold unbounded (could be 0 disabling check, or 10^9)
Resolved: §1 #10 + DEC-722 + CHECK [1, 10000] + 400 on out-of-range; AC #20.

### ISS-006 — Audit log mutable (operator could rewrite breach decisions)
Resolved: §1 #12 + REVOKE UPDATE, DELETE FROM cyberos_app + auth_writer role grant + memory_chain_hash dual-link; AC #25.

### ISS-007 — Air-gapped tenants couldn't use HIBP at all
Resolved: §1 #9 + DEC-723 + CYBEROS_HIBP_LOCAL_DUMP_PATH env + per-prefix file lookup + fail-loud startup on empty dir; AC #21 + #22.

### ISS-008 — Dev whitelist could leak to prod
Resolved: §1 #23 + CYBEROS_ENV gate + sev-1 startup audit on prod-with-whitelist + ignore in non-dev; AC #24.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (k-anonymity 5-char prefix × in-process LRU cache 10000×1h × 2s timeout+retry × 10-permit semaphore × per-tenant threshold [1,10000] × per-tenant unreachable policy × air-gapped local dump × dev whitelist env-gated × HIBP-before-Argon2 ordering × append-only audit log SQL grant × RLS × 6 closed memory kinds × 429-escalation sliding window × NO login invocation × NO plaintext/full-hash logging × live-canary nightly CI × dry-run preview × memory_chain_hash dual-link × FR-MEMORY-111 PII scrubbing), not by line targets.

---

*End of FR-AUTH-107 audit.*
