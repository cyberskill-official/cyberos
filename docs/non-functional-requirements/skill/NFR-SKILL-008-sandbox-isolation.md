---
id: NFR-SKILL-008
title: "SKILL sandbox isolation — skill execution MUST NOT escape declared capabilities"
module: SKILL
category: security
priority: MUST
verification: T
phase: P0
slo: "0 cap-bypass incidents in 90 days; 100% of attempted escapes blocked + audited"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-104, FR-SKILL-101]
---

## §1 — Statement (BCP-14 normative)

1. A skill **MUST NOT** access any platform resource (DB, network, filesystem, secrets) outside the capabilities declared in its manifest and granted by the broker.
2. The runtime **MUST** enforce isolation at the OS level (per-skill seccomp/sandbox profile) — runtime-level checks alone are not sufficient.
3. Network egress **MUST** be denied by default; skills requesting `cap:net.egress.<host>` get a per-skill egress allowlist; wildcard egress (`*`) is reserved for system skills only and requires explicit broker admin approval.
4. Any attempted escape (denied syscall, denied egress) **MUST** be logged to BRAIN with `kind=skill.cap_bypass_attempt` carrying the denied call + skill identity.
5. Three cap-bypass attempts from the same skill version in 24h **MUST** auto-quarantine that skill version (pulled from registry, running invocations completed but not refreshed).

## §2 — Why this constraint

Skills run third-party logic from the catalog. Without OS-level isolation, a malicious or buggy skill could exfiltrate tenant data via undeclared HTTP, read another tenant's DB rows, or use platform credentials to attack adjacent services. The default-deny + explicit-allowlist + auto-quarantine combination converts "skill misbehavior" from "we hope it doesn't happen" into "we catch + isolate it automatically." The OS-level enforcement matters because runtime-only checks can be bypassed by an exploit that disables them.

## §3 — Measurement

- Counter `skill_cap_bypass_attempt_total{skill, denied_call, syscall_or_host}`.
- Counter `skill_quarantine_event_total{skill, version, trigger=auto|manual}`.
- Audit row count for `kind=skill.cap_bypass_attempt` must equal counter exactly (no audit drops).

## §4 — Verification

- Integration test `modules/skill/tests/sandbox_escape_test.py` (T) — runs a deliberately-malicious test skill; asserts every escape attempt is denied + audited + (after 3) the skill is quarantined.
- Quarterly red-team exercise (T) — submit hostile skills to broker; verify all escape paths fail.
- CI gate per skill: declared capabilities subset-check against actual syscall/egress trace from a recorded invocation.

## §5 — Failure handling

- Single escape attempt → audit row; skill continues.
- Three attempts same skill/version 24h → auto-quarantine; sev-3; catalog owner notified.
- Successful escape (declared cap was insufficient + skill accessed unauthorised resource) → sev-1; immediate platform-wide quarantine of that skill; CISO + CTO postmortem within 24h.

---

*End of NFR-SKILL-008.*
