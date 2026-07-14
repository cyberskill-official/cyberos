---
task_id: TASK-CHAT-013
audited: 2026-06-29
verdict: PASS
score: 9.5/10
issues_resolved: 6
issues_open: 0
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (config/integration FR; supersedes TASK-CHAT-002; mirrors TASK-AUTH-110 structure)
---

## §1 - Verdict summary

TASK-CHAT-013 wires the Mattermost fork to the TASK-AUTH-110 CyberOS OIDC provider through Mattermost's own native OAuth/OpenID connector, replacing the closed TASK-CHAT-002 AuthBridge plugin. Scope: 11 normative clauses covering the native-connector requirement (no plugin), the free-edition GitLab-connector-repurposed method plus the Enterprise OpenID alternative, real JIT user provisioning from the OIDC claims, RP registration with a byte-exact redirect_uri and an uncommitted one-time secret, disabling builtin password sign-up via config (not a patch), the honest kick boundary (revoke blocks re-login; instant session kill is back-channel logout / SCIM), HTTPS end to end for the __Host- cookie, single-tenant team mapping, optional role mapping, and owner-run live-sign-in verification. It is a small, configuration-weight FR because the heavy lifting is TASK-AUTH-110, which is built and green. Verdict PASS.

## §2 - Findings (all resolved)

### ISS-001 - The closed approach could be silently re-attempted (RESOLVED)
A reader might try the plugin again. Resolved: DEC-2500 + §1 #1 + disallowed_tools forbid any plugin-based login interception, and the TASK-CHAT-002 banner explains why a plugin cannot replace the login route.

### ISS-002 - Free-edition connector reality (RESOLVED)
The generic OpenID connector is Enterprise-gated in the open-source build. Resolved: DEC-2501 + §3.2 use the GitLab connector with overridden endpoint URLs (the standard free-edition method), with the Enterprise OpenID path in §3.3 as the alternative - the FR does not assume a connector the free build lacks.

### ISS-003 - JIT must create a real user, not a simulation (RESOLVED)
The closed FR's provisioner was in-memory. Resolved: DEC-2502 + §1 #3 - Mattermost's native sign-in creates a real user row + session from sub/email/preferred_username; the FR explicitly contrasts this with the simulation.

### ISS-004 - The kick was the TASK-CHAT-002 overclaim (RESOLVED)
Resolved: DEC-2503 + §1 #6 + §9 - revoke blocks re-login immediately (provider authorize refuses); an open Mattermost session dies via back-channel logout (TASK-AUTH-110 slice 2) or the session TTL; SCIM (TASK-PORTAL-004) is instant cross-app. Stated plainly, no overclaim.

### ISS-005 - Secret handling + HTTPS (RESOLVED)
Resolved: DEC-2508 + §1 #4 (one-time secret, env-only, never committed, byte-exact redirect_uri) and DEC-2504 + §1 #7 (HTTPS end to end; the __Host- cookie needs TLS; local uses a tunnel or the curl path).

### ISS-006 - Verification is not unit-testable (RESOLVED)
A browser SSO handshake cannot be a unit test. Resolved: DEC-2509 + §5 - an owner-run live sign-in is the proof (the provider side is already proven by the round-trip runbook), and the connector config is checked into deploy config. This matches how the Google flow is owner-verified.

## §3 - Resolution

6 of 6 concerns resolved. The supersede linkage (TASK-CHAT-002 closed; this FR's `supersedes` + the banner) is clean, the kick boundary is honest, and the free-vs-enterprise connector reality is handled rather than assumed. The FR's depth is bounded by its real surface (one provider, one connector, config + an owner-run sign-in), not by a line target.

**Score = 9.5/10.** The half point is the inherent residual that a browser SSO handshake is only ever owner-verified, not gated by an automated test - a property of the integration, not a defect of the spec.

---

*End of TASK-CHAT-013 audit.*
