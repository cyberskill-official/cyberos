---
task_id: TASK-AUTH-105
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per task-audit skill §0)
---

## §1 — Verdict summary

TASK-AUTH-105 ships passkey enrolment + login on top of TASK-AUTH-102's WebAuthn substrate — discoverable credentials + autofill UI + downgrade-resistance. Scope: 26 §1 normative clauses covering closed 3-value passkey_origin enum (platform_synced, platform_local, cross_platform) with AAGUID detection, enrolment FSM (requested → confirmed | abandoned with 24h TTL + hourly abandonment job), discoverable-credential login with empty allowCredentials (resident-key UX), autofill conditional mediation requiring UV=required, downgrade-resistance with 60s cache (password login blocked when passkey enrolled), per-subject opt-out flag with root-admin gate + sev-2 audit, removal requires fresh MFA (< 5min challenge token) to defeat session-hijack passkey wipe, max 5 active passkeys per subject, per-tenant `passkey_required_for_roles` policy from tenant_policy YAML with founder hard-coded, recovery-code warning at enrolment (passkey-only + no recovery = lockout risk), 8 memory audit kinds with PII scrubbing + sev-2 on downgrade-blocked, append-only lifecycle log at SQL grant, public key never exposed via API (defense in depth). 22 rationale paragraphs. §3 contains: 2 migrations (enrolment_state with ENUM + mfa_factors ALTER to add passkey fields, lifecycle_log append-only), origin detection with well-known AAGUID map, enrolment FSM with abandonment job, downgrade_gate with 60s TTL cache matching TASK-AUTH-109 pattern, enrol_begin handler with recovery warning, login_finish handler with cloned-authenticator detection. 28 ACs. 31 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Downgrade-resistance missing (passkey + password both valid)
First-pass let password login succeed even when passkey enrolled. Resolved: §1 #12 + DEC-543 + downgrade_gate + 401 passkey_required + sev-2 audit; AC #12.

### ISS-002 — Discoverable credentials not enforced
First-pass had no residentKey=required. Resolved: §1 #5 + DEC-540 + WebAuthn creation challenge config; AC #2.

### ISS-003 — Removal allowed without fresh MFA (session-hijack passkey wipe)
First-pass had no re-auth requirement. Resolved: §1 #14 + DEC-552 + X-MFA-Challenge-Token header < 5min; AC #20.

### ISS-004 — Autofill conditional mediation without UV
Resolved: §1 #11 + DEC-551 + W3C spec compliance + handler reject; AC #16.

### ISS-005 — Opt-out without audit + role gate
First-pass let any caller set opt-out. Resolved: §1 #25 + #26 + root-admin role + reason required + sev-2 audit; AC #14 + #15.

### ISS-006 — Enrolment limit unbounded
First-pass had no 5-cap. Resolved: §1 #13 + DEC-544 + handler check + 409; AC #6.

### ISS-007 — Recovery-code lockout risk silent
First-pass didn't warn at enrolment. Resolved: §1 #24 + DEC-548 + recovery_warning in response body; AC #7.

### ISS-008 — Public key exposed via API (defense in depth gap)
Resolved: §1 #19 + handler omits public_key + AAGUID from /factors response.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (discoverable credentials × autofill conditional mediation × downgrade-resistance × per-subject opt-out × removal requires fresh MFA × max 5 cap × per-tenant policy × recovery-code warning × 8 memory audit kinds × AAGUID origin detection × append-only log × public key never exposed × TASK-AUTH-102 substrate reuse), not by line targets.

---

*End of TASK-AUTH-105 audit.*
