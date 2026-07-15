---
task_id: TASK-AUTH-108
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per task-audit skill §0)
---

## §1 — Verdict summary

TASK-AUTH-108 ships the Lumi tenant-identity JWT shape — extension of TASK-AUTH-004 with 5 Lumi-specific claims (agent_persona, tenant_residency, lumi_org_tenant, persona_version, sync_class_allowed, anchor_chain_hash, jti). Scope: 25 §1 normative clauses covering distinct iss `https://lumi.cyberos.world` + aud `https://memory.cyberos.world/sync` for cross-tenant security boundary, alg=RS256 pinned (HS256 + none rejected for JWT-confusion defense), persona-version staleness check with 2-minor-version tolerance, sync_class closed enum (5 values per AGENTS.md §15), tenant_residency enforcement (Decree 53 + GDPR + DORA compliance), human-cannot-issue restriction (EU AI Act Art. 13), anchor_chain_hash for replay defense, per-tenant sync_class policy gate, append-only issuance log at SQL grant, 4 memory audit kinds with sev-2 staleness alarm, RLS with root-admin escape clause, RFC 7519 jti for future revocation lookup, TTL 1h default (5min-24h configurable), slug regex validation. 19 rationale paragraphs. §3 contains: migration 0013 (lumi_token_issuance_log with append-only + RLS), LumiClaims struct with full shape validation, SyncClass closed enum, verifier with RS256-pinned signature + iss/aud/exp/nbf + residency + persona-version staleness checks, issuer handler with role gate + policy gate. 27 ACs. 32 failure-mode rows. 19 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — JWT-confusion attack (HS256 with public key)
First-pass allowed alg=HS256. Resolved: §1 #7 + DEC-431 + alg pinned RS256 + HS256/none always rejected; AC #3.

### ISS-002 — Same iss as TASK-AUTH-004 (per-tenant AUTH compromise = Lumi forgery)
First-pass shared iss. Resolved: §1 #2 + DEC-426 + distinct `https://lumi.cyberos.world` + distinct aud; AC #1 + #2.

### ISS-003 — Persona-version drift unbounded
First-pass had no staleness check. Resolved: §1 #4 + DEC-424 + 2-minor-version tolerance + sev-2 alarm; AC #10 + #11.

### ISS-004 — Residency unenforced (cross-region token replay)
First-pass omitted check. Resolved: §1 #6 + DEC-422 + 451 unavailable_for_legal_reasons; AC #6 + #7.

### ISS-005 — Human subjects could issue Lumi (EU AI Act Art. 13 violation)
First-pass had no role gate. Resolved: §1 #8 + DEC-432 + agent-persona role required; AC #12.

### ISS-006 — sync_class_allowed open-ended
First-pass accepted any string. Resolved: §1 #5 + DEC-425 + closed enum (5 values per AGENTS.md §15) + empty array rejected; AC #8 + #9.

### ISS-007 — Replay attack against stale chain head
First-pass omitted anchor_chain_hash. Resolved: §1 #9 + DEC-433 + 64-hex required claim + memory sync validates against current head.

### ISS-008 — Token revocation impossible (no jti)
First-pass omitted jti. Resolved: §1 #20 + RFC 7519 jti + UNIQUE constraint + task-AUTH-2xx revocation API consumes.

### ISS-009 — sync_class_allowed could exceed tenant policy
First-pass had no policy gate. Resolved: §1 #25 + per-tenant policy check at issuance; AC #21.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (distinct iss + aud × alg-pinned RS256 × persona-version staleness × residency enforcement × human-issuance restriction × sync_class closed enum × anchor_chain_hash replay defense × per-tenant policy gate × jti + UNIQUE × 4 memory audit kinds × append-only log × RLS with root-admin escape), not by line targets.

---

*End of TASK-AUTH-108 audit.*
