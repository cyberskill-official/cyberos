---
id: FR-AUTH-108
title: "AUTH Lumi tenant-identity JWT shape — agent_persona + tenant_residency + lumi_org_tenant claims + persona-version stamping + cross-tenant sync identity"
module: AUTH
priority: MUST
status: done
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-AUTH-004, FR-AUTH-101, FR-AI-014, FR-AI-016, FR-AI-003, FR-MEMORY-101, FR-CUO-101, FR-AUTH-109]
depends_on: [FR-AUTH-101]
blocks: []

source_pages:
  - website/docs/modules/auth.html#lumi-jwt
  - website/docs/modules/cuo.html#lumi-tenant-identity
source_decisions:
  - DEC-420 (Lumi JWT shape is additive on FR-AUTH-004 + FR-AUTH-101 — adds `agent_persona`, `tenant_residency`, `lumi_org_tenant`, `persona_version`, `sync_class_allowed` claims; existing claims preserved unchanged)
  - DEC-421 (`agent_persona` claim format: `cuo-<persona-key>@<semver>` matching FR-AI-014 stamping; persona-key ∈ FR-AUTH-101 closed enum {agent-persona, founder, cfo, cto, coo, chro, cmo, cpo, cso, cseco, clo, cdo, dpo, caio})
  - DEC-422 (`tenant_residency` claim is the per-tenant residency code from FR-AI-016 closed enum: vn-1 | sg-1 | eu-1 | us-1; mismatch with route's residency → 451 unavailable for legal reasons)
  - DEC-423 (`lumi_org_tenant` claim is the SLUG of the org-tenant that this Lumi persona represents — when present, indicates this is a cross-tenant Lumi sync identity, NOT a regular tenant-user identity)
  - DEC-424 (`persona_version` claim is a semver string matching FR-AI-014's pinned version; verifiers may reject tokens with persona_version older than 2 minor versions behind current)
  - DEC-425 (`sync_class_allowed` claim is a closed-set array of strings from {private, shareable, publishable, shared, client-visible} per AGENTS.md §15; gates what a Lumi sync token may publish across tenants)
  - DEC-426 (Lumi tokens have a DISTINCT issuer `https://lumi.cyberos.world` separate from per-tenant `https://auth.<tenant>.cyberos.world`; verifiers MUST check issuer against tenant_id mapping)
  - DEC-427 (Lumi tokens have audience `aud: https://memory.cyberos.world/sync` — bounded to memory sync endpoints only; not a general-purpose tenant token)
  - DEC-428 (memory audit kinds: auth.lumi_token_issued, auth.lumi_token_verified, auth.lumi_token_rejected, auth.lumi_persona_version_stale)
  - DEC-429 (Lumi token TTL = 1 hour at slice 1; refresh via FR-AUTH-004 refresh path; rotation on persona_version bump invalidates prior tokens within 1 hour)
  - DEC-430 (REVOKE UPDATE, DELETE on lumi_token_issuance_log from cyberos_app — append-only at SQL grant)
  - DEC-431 (Lumi JWT alg pinned at RS256 per FR-AUTH-004 default; alg=none rejected always; alg=HS256 rejected always per JWT-confusion attack defense)
  - DEC-432 (only `cuo-*` agent_persona values may issue Lumi tokens at slice 1; human subjects with agent-persona role cannot impersonate Lumi without explicit FR-AUTH-2xx delegation flow)
  - DEC-433 (cross-tenant chain anchor — Lumi tokens carry an `anchor_chain_hash` claim that the memory sync endpoint validates against the source tenant's chain head; mismatch → 409 chain_diverged)
  - PDPL Art. 13 (data minimisation — JWT claims include only what's needed for sync gating)
  - EU AI Act Art. 13 (transparency — agent_persona stamp = explicit AI-actor identifier)
  - EU AI Act Art. 14 (human oversight — Lumi sync via agent persona is an AI-initiated action; sync operations subject to per-tenant deny-by-default policy)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0013_lumi_token_issuance_log.sql
  - services/auth/src/lumi/mod.rs                              # public API
  - services/auth/src/lumi/issuer.rs                           # token issuance (extends FR-AUTH-004 issuer)
  - services/auth/src/lumi/verifier.rs                         # token verification with Lumi-specific checks
  - services/auth/src/lumi/claims.rs                           # LumiClaims struct (extends Claims)
  - services/auth/src/lumi/persona_version.rs                  # version-stale-check (consumes FR-AI-014)
  - services/auth/src/lumi/sync_class.rs                       # SyncClass enum + closed-set validator
  - services/auth/src/lumi/repo.rs                             # issuance log writer
  - services/auth/src/lumi/audit.rs                            # 4 memory row builders
  - services/auth/src/handlers/lumi.rs                         # POST /v1/auth/lumi/issue + GET /v1/auth/lumi/verify
  - services/auth/tests/admin_list_test.rs
  - services/auth/tests/admin_list_test.rs
  - services/auth/tests/middleware_test.rs
  - services/auth/tests/lumi_persona_version_stale_test.rs
  - services/auth/tests/admin_deny_list_test.rs
  - services/auth/tests/lumi_alg_confusion_test.rs
  - services/auth/tests/lumi_sync_class_closed_test.rs
  - services/auth/tests/lumi_human_cannot_issue_test.rs
  - services/auth/tests/admin_deny_list_test.rs
  - services/auth/tests/rls_isolation_test.rs
modified_files:
  - services/auth/src/jwt.rs                                   # add Lumi-claim deserialisation paths
  - services/auth/src/lib.rs                                   # pub mod lumi

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test lumi

disallowed_tools:
  - allow alg=none or alg=HS256 (per DEC-431 — JWT-confusion attack defense)
  - allow human subjects to issue Lumi tokens at slice 1 (per DEC-432)
  - allow Lumi tokens for non-sync audiences (per DEC-427)
  - skip persona_version staleness check (per DEC-424)
  - log raw JWT to file (chain holds scrubbed)

effort_hours: 6
sub_tasks:
  - "0.4h: 0013_lumi_token_issuance_log.sql — append-only log + REVOKE writes"
  - "0.5h: claims.rs — LumiClaims with 5 new fields"
  - "0.6h: issuer.rs — token issuance + persona-version stamping"
  - "0.7h: verifier.rs — verification with issuer/aud/alg + residency check"
  - "0.5h: persona_version.rs — staleness check (2-minor-version tolerance)"
  - "0.4h: sync_class.rs — closed enum + array validator"
  - "0.4h: repo.rs — issuance log writer"
  - "0.3h: audit.rs — 4 row builders"
  - "0.4h: handlers/lumi.rs — REST surface"
  - "1.8h: tests — 10 test files"

risk_if_skipped: "Lumi is the cross-tenant synthesis identity — the persona that owns memory sync push, cross-team synthesis, and tenant-aware policy enforcement at P3+. Without DEC-420's distinct JWT shape, the same FR-AUTH-004 token would be used for both per-tenant operations AND cross-tenant Lumi sync — the distinction is forensically critical (which subject performed which class of operation?). Without DEC-426's distinct issuer, an attacker who compromises a per-tenant AUTH could forge Lumi tokens. Without DEC-431's alg-confusion defense, the classic JWT-alg-substitution attack succeeds (HS256 + JWKS public key = forged token). Without DEC-432's human-cannot-issue restriction, a tenant-admin could escalate to cross-tenant sync privileges. Without DEC-433's anchor_chain_hash, Lumi sync replays succeed (attacker captures + replays Lumi token to write at a stale chain head). The 6h effort lands the cross-tenant identity primitive with the security gates Lumi depends on."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** extend FR-AUTH-004 JWT shape with Lumi-specific claims for cross-tenant sync identity. Each requirement:

1. **MUST** add the following 5 claims to JWTs issued via the `/v1/auth/lumi/issue` endpoint (per DEC-420):
    - `agent_persona: TEXT` — format `cuo-<persona-key>@<semver>` per DEC-421.
    - `tenant_residency: TEXT` — one of `vn-1 | sg-1 | eu-1 | us-1` per DEC-422.
    - `lumi_org_tenant: TEXT` — slug of the org-tenant this Lumi persona represents (per DEC-423).
    - `persona_version: TEXT` — semver matching FR-AI-014 pinned version (per DEC-424).
    - `sync_class_allowed: TEXT[]` — closed-set array per DEC-425.
   Existing FR-AUTH-004 claims (`sub`, `tid`, `iss`, `iat`, `exp`, `nbf`, `roles`, `rbac_v`) preserved unchanged.

2. **MUST** issue Lumi tokens with distinct `iss: https://lumi.cyberos.world` and `aud: https://memory.cyberos.world/sync` (per DEC-426 + DEC-427). Per-tenant tokens (FR-AUTH-004) continue using `iss: https://auth.<tenant>.cyberos.world` and tenant-specific audiences. Mismatch on either → 401 + `auth.lumi_token_rejected` memory row.

3. **MUST** enforce that `agent_persona` claim's persona-key ∈ FR-AUTH-101 agent-persona-role-family enum (per DEC-421). Verifier parses prefix `cuo-` + key + `@` + semver; invalid format → 401 `agent_persona_malformed`; unknown persona-key → 401 `agent_persona_unknown`.

4. **MUST** verify `persona_version` against the live version registry from FR-AI-014 (per DEC-424 + DEC-429). Verifier compares token's `persona_version` against `cuo-<persona-key>@<live-semver>` from the registry; if token's version is more than 2 minor versions behind → 401 `persona_version_stale` + emit `auth.lumi_persona_version_stale` memory row (sev-2 — signals refresh-token reuse beyond rotation window).

5. **MUST** enforce `sync_class_allowed` is a closed-set array (per DEC-425). Allowed values: `'private'`, `'shareable'`, `'publishable'`, `'shared'`, `'client-visible'`. Unknown values → 401 `sync_class_unknown`. Empty array → 401 `sync_class_empty` (token must permit at least one class).

6. **MUST** verify `tenant_residency` claim matches the routing residency of the request (per DEC-422). The memory sync endpoint's host determines the expected residency (e.g. `memory.vn-1.cyberos.world` expects `tenant_residency=vn-1`); mismatch → 451 `unavailable_for_legal_reasons` + emit `auth.lumi_token_rejected` memory row with `reason='residency_mismatch'`.

7. **MUST** pin Lumi JWT alg at RS256 (per DEC-431). Reject alg=none ALWAYS. Reject alg=HS256 ALWAYS (JWT-confusion attack: attacker uses JWKS public key as HS256 secret). Unknown alg → 401 `unsupported_alg`.

8. **MUST** restrict Lumi token issuance to subjects whose `roles` claim contains `agent-persona` per FR-AUTH-101 (per DEC-432). Human subjects (those without `agent-persona` role) attempting issuance → 403 `human_cannot_issue_lumi`. Delegation flow (a human authorising a Lumi issuance on their behalf) deferred to FR-AUTH-2xx.

9. **MUST** include `anchor_chain_hash: TEXT` claim (per DEC-433) — the source tenant's memory chain head at token issuance time. The memory sync endpoint validates this against the current chain head; if the source chain has diverged → 409 `chain_diverged` + emit `auth.lumi_token_rejected` with `reason='chain_diverged'`.

10. **MUST** define `lumi_token_issuance_log` table: `(id BIGSERIAL PRIMARY KEY, issuer_subject_id UUID NOT NULL REFERENCES auth.subjects(id), agent_persona TEXT NOT NULL, persona_version TEXT NOT NULL, source_tenant_id UUID NOT NULL, lumi_org_tenant TEXT NOT NULL, residency TEXT NOT NULL, sync_class_allowed TEXT[] NOT NULL, anchor_chain_hash TEXT NOT NULL, ttl_seconds INT NOT NULL DEFAULT 3600, issued_at TIMESTAMPTZ NOT NULL DEFAULT now(), token_jti TEXT NOT NULL UNIQUE)`. `REVOKE UPDATE, DELETE FROM cyberos_app` (per DEC-430).

11. **MUST** enforce RLS with both `USING` and `WITH CHECK` on `lumi_token_issuance_log`. Policy: `source_tenant_id = current_setting('auth.tenant_id')::uuid OR current_setting('auth.is_root_admin', true) = 'true'` (root-admin can see all for audit).

12. **MUST** ship `POST /v1/auth/lumi/issue` handler. Body: `{agent_persona: "<key>", persona_version: "<semver>", lumi_org_tenant: "<slug>", sync_class_allowed: [...], anchor_chain_hash: "<hex>"}`. Caller MUST have role `agent-persona` per FR-AUTH-101. Validates all fields against closed sets + persona_version against registry. Returns `{token: "<jwt>", expires_at: <iso8601>}`. Emits `auth.lumi_token_issued` memory row.

13. **MUST** ship `GET /v1/auth/lumi/verify` handler used by memory sync + downstream consumers. Header: `Authorization: Bearer <token>`. Returns `{valid: bool, claims: <object>, reason: <text?>}`. Internal-use only (not exposed externally); the verifier library is the canonical path.

14. **MUST** emit 4 memory audit row kinds (per DEC-428):
    - `auth.lumi_token_issued` — every successful issuance; carries persona, version, residency, sync_class, ttl, jti.
    - `auth.lumi_token_verified` — sampled at 1% for cost reasons (high-volume); 100% on outcome != success.
    - `auth.lumi_token_rejected` — every verification failure; carries reason (residency_mismatch | chain_diverged | persona_version_stale | unsupported_alg | aud_mismatch | iss_mismatch | sync_class_unknown | agent_persona_unknown).
    - `auth.lumi_persona_version_stale` — sev-2 alarm trigger; emitted when verifier sees a persona_version > 2 minor versions behind.

15. **MUST** PII-scrub `lumi_org_tenant` slug via FR-MEMORY-111 before chain commit (treated as PII at the cross-tenant boundary).

16. **MUST** complete Lumi token issuance in ≤ 100 ms p95 (signing key + JWKS already in process). Verification in ≤ 50 ms p95 (signature verify + claim parse + persona-version lookup from in-memory registry).

17. **MUST** emit OTel span `auth.lumi.{issue,verify}` with attributes: `tenant_id`, `agent_persona`, `persona_version`, `lumi_org_tenant`, `outcome` (success | residency_mismatch | chain_diverged | persona_version_stale | aud_mismatch | iss_mismatch | unsupported_alg | sync_class_unknown | agent_persona_unknown | human_cannot_issue).

18. **MUST** emit OTel metrics:
    - `auth_lumi_issuance_total{tenant_id, agent_persona, outcome}` (counter).
    - `auth_lumi_verification_total{outcome, reason}` (counter).
    - `auth_lumi_persona_version_stale_total{persona, stale_versions_behind}` (counter; sev-2 alarm at sustained > 5/h).
    - `auth_lumi_active_tokens{tenant_id, persona}` (gauge — periodic count of unexpired tokens from log).
    - `auth_lumi_token_latency_ms{op}` (histogram; op ∈ {issue, verify}).

19. **MUST** sign Lumi JWT with the SAME RS256 keypair used by FR-AUTH-004 (issuer-distinct via the `iss` claim; key-distinct deferred to slice 2). The kid is the same; the iss differs — verifiers must check both.

20. **MUST** include `jti: TEXT` claim (JWT ID — RFC 7519) as a UUIDv4. Stored in `lumi_token_issuance_log.token_jti` for revocation lookup. Revocation API ships in FR-AUTH-2xx; this FR ships the column.

21. **MUST** ship the `LumiClaims` Rust struct with all 5 new claims + existing 8 FR-AUTH-004 claims. Total 13 claims. Serialisation order: existing 8 first (preserves backward compat for any FR-AUTH-004 verifier inspecting only the standard claims), then Lumi additions.

22. **MUST** validate that `lumi_org_tenant` slug matches the pattern `^[a-z][a-z0-9-]{2,40}[a-z0-9]$` per FR-TEN-001's slug regex. Malformed → 401 `lumi_org_tenant_malformed`.

23. **MUST** support TTL configurable via env `LUMI_TOKEN_TTL_SECONDS` with default 3600 (per DEC-429). Operators MAY shorten for high-security tenants; values < 300 (5min) → reject env config at startup.

24. **MUST** include `anchor_chain_hash` validation flow: the memory sync endpoint (out of scope here, ships in FR-MEMORY-2xx) reads the claim + compares against the current chain head; this FR ships the claim emission + verifier-side parsing.

25. **MUST** validate `sync_class_allowed` ⊆ what the source tenant's policy permits. The per-tenant policy YAML (FR-AI-005) declares per-tenant `lumi_sync_max_classes`. If requested array exceeds policy → 403 `sync_class_exceeds_policy` + emit `auth.lumi_token_rejected` with reason.

---

## §2 — Why this design (rationale for humans)

**Why distinct iss for Lumi tokens (DEC-426)?** Per-tenant AUTH compromise should not let attackers forge cross-tenant Lumi tokens. Distinct iss means even if a tenant's AUTH cluster is breached, the attacker still needs to forge tokens with `iss=https://lumi.cyberos.world` — that's a separate trust anchor. Verifiers check iss + tenant_id mapping; mismatch fails closed.

**Why distinct aud (DEC-427)?** Audience-bound tokens (RFC 7519 §4.1.3) prevent token misuse: a Lumi token meant for `memory.cyberos.world/sync` cannot be replayed against `api.cyberos.world/v1/...`. Bounding audience to sync endpoints contains the blast radius of any compromise.

**Why alg=HS256 rejected (DEC-431, §1 #7)?** Classic JWT-confusion attack: attacker takes a JWKS public key, signs a token using HS256 with the public key as the HMAC secret, and the verifier (if it accepts HS256) validates because the "secret" matches. The defense is to pin alg=RS256 — rejecting HS256 always means the attack cannot succeed regardless of verifier bugs.

**Why human subjects can't issue Lumi tokens at slice 1 (DEC-432, §1 #8)?** Lumi is an AI persona; tokens represent AI-initiated cross-tenant operations. A human tenant-admin authenticating + then issuing a Lumi token would be claiming to be an AI agent — defeats the EU AI Act Art. 13 transparency requirement (which clearly distinguishes AI actors from human actors). Restricting issuance to `agent-persona` role subjects keeps the boundary clean. Delegation (human authorises a specific Lumi sync on their behalf) is a slice 3 ADR-gated flow.

**Why persona_version staleness check at 2 minor versions (DEC-424, §1 #4)?** Persona prompts evolve; a 2-minor-version-stale token suggests the verifier's persona registry has updated but the token-issuing process hasn't refreshed. Tolerance of 2 minors covers normal rotation lag (~hours); beyond that signals operational issue. Sev-2 alarm prompts ops investigation.

**Why sync_class_allowed closed enum (DEC-425, §1 #5)?** AGENTS.md §15 defines the closed sync-class set. Allowing arbitrary strings would let tokens claim privileges that don't exist (silently treated as "no privilege" or worse). Closed-set validation at issuance time catches typos at the boundary.

**Why anchor_chain_hash claim (DEC-433, §1 #9)?** Lumi sync writes to memory. If an attacker captures a Lumi token and replays it after the source tenant's chain has advanced (e.g. with newer rows), the replay would write at a stale head — silently corrupting the chain. The anchor_chain_hash binds the token to a specific chain state; mismatch rejects the replay.

**Why same signing key as FR-AUTH-004 at slice 1 (§1 #19)?** Operationally simpler; one key rotation flow. Cryptographically the distinct iss + aud claims provide the security boundary (an attacker forging an FR-AUTH-004 token can't get the iss accepted by the Lumi verifier). Distinct key per identity-class is a slice 2 hardening.

**Why TTL 1 hour configurable (DEC-429, §1 #23)?** Lumi sync is a periodic activity (typically every 5-15 minutes). 1-hour TTL gives reasonable refresh cadence without making compromised tokens long-lived. Operators with high-security needs can tighten via env; lower bound of 5 minutes prevents pathological config.

**Why jti claim with log row (§1 #20)?** Revocation requires looking up "is this jti revoked?". Storing every issued jti in the log enables this. The revocation API (FR-AUTH-2xx) consults the log. Slice 1 ships the column + log; revocation lookup is slice 2.

**Why tenant_residency check at verifier (§1 #6, DEC-422)?** Decree 53/2022 + GDPR + DORA require data residency enforcement at every operation. If a Lumi token issued in vn-1 hits a sync endpoint in eu-1, that's a cross-residency operation — must be rejected even if the token is otherwise valid. The 451 status code (RFC 7725) is the spec-correct response for legal-reason rejection.

**Why `lumi_org_tenant` is a separate claim from `tid` (DEC-423, §1 #1)?** `tid` is the regular tenant id of the issuing AUTH; `lumi_org_tenant` is the slug of the org-tenant this Lumi persona represents. These can differ: a Lumi sync from a customer tenant pushing to the CyberSkill org-tenant has `tid=<customer-uuid>` and `lumi_org_tenant=cyberskill`. Two-field design makes the cross-tenant relationship explicit.

**Why `auth.lumi_token_verified` sampled at 1% (§1 #14)?** High-volume hot path; 100% sampling would flood memory. 1% gives statistical coverage; 100% on `outcome != success` ensures every failure is captured for debugging.

**Why `sync_class_allowed` validated against per-tenant policy (§1 #25)?** A tenant may restrict their data's max sync class (e.g. "we never publish externally — sync_class cap = 'shared'"). The issuance handler enforces; tokens cannot grant more than policy permits. Defense in depth — the memory sync endpoint also checks at write time, but rejecting at issuance time saves the round-trip.

**Why slug pattern validation for `lumi_org_tenant` (§1 #22)?** Slug is used in cross-tenant routing + memory audit paths; malformed values cause hard-to-debug failures downstream. Pattern validation at JWT issuance time + JWT verification time catches typos at the boundary.

**Why total 13 claims (§1 #21)?** Extending FR-AUTH-004's 8 with 5 Lumi-specific claims is the minimum complete set. Adding fewer would force consumers to infer; adding more bloats the token. The serialisation order preserves FR-AUTH-004-compatible read paths.

**Why same kid for FR-AUTH-004 + Lumi at slice 1 (§1 #19)?** Simplification — JWKS verification just needs to find the kid; iss + aud provide the boundary. Per-identity-class kid (slice 2) provides defense-in-depth at the cost of dual-key rotation procedures.

**Why sev-2 alarm on persona_version_stale > 5/h (§1 #14, §1 #18)?** Normal rotation produces 0-1 stale tokens/hour; > 5/h sustained signals (a) version-registry update not propagated to issuer, (b) compromised token reuse beyond refresh window, (c) misconfigured caching. Sev-2 prompts operator investigation.

**Why no Lumi token revocation handler at slice 1 (§1 #20)?** Revocation is a separable concern — needs a CRL endpoint + propagation mechanism + cache invalidation. Slice 1 ships the jti column; FR-AUTH-2xx ships the revocation flow. Until then, short TTL (1h) is the mitigation.

---

## §3 — API contract

### 3.1 — Migration 0013

```sql
-- services/auth/migrations/0013_lumi_token_issuance_log.sql

BEGIN;

CREATE TABLE lumi_token_issuance_log (
    id                       BIGSERIAL    PRIMARY KEY,
    issuer_subject_id        UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    agent_persona            TEXT         NOT NULL CHECK (agent_persona ~ '^cuo-[a-z][a-z0-9-]*@[0-9]+\.[0-9]+\.[0-9]+$'),
    persona_version          TEXT         NOT NULL,
    source_tenant_id         UUID         NOT NULL,
    lumi_org_tenant          TEXT         NOT NULL CHECK (lumi_org_tenant ~ '^[a-z][a-z0-9-]{2,40}[a-z0-9]$'),
    residency                TEXT         NOT NULL CHECK (residency IN ('vn-1','sg-1','eu-1','us-1')),
    sync_class_allowed       TEXT[]       NOT NULL CHECK (array_length(sync_class_allowed, 1) >= 1),
    anchor_chain_hash        TEXT         NOT NULL CHECK (anchor_chain_hash ~ '^[0-9a-f]{64}$'),
    ttl_seconds              INT          NOT NULL DEFAULT 3600 CHECK (ttl_seconds BETWEEN 300 AND 86400),
    issued_at                TIMESTAMPTZ  NOT NULL DEFAULT now(),
    token_jti                TEXT         NOT NULL UNIQUE
);

CREATE INDEX lumi_log_tenant_issued_idx ON lumi_token_issuance_log (source_tenant_id, issued_at DESC);
CREATE INDEX lumi_log_persona_idx ON lumi_token_issuance_log (agent_persona, persona_version);

ALTER TABLE lumi_token_issuance_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY lumi_log_tenant_iso ON lumi_token_issuance_log
    USING (source_tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (source_tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON lumi_token_issuance_log FROM cyberos_app;

COMMIT;
```

### 3.2 — Lumi claims struct

```rust
// services/auth/src/lumi/claims.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::lumi::sync_class::SyncClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumiClaims {
    // FR-AUTH-004 baseline claims
    pub sub: Uuid,
    pub tid: Uuid,
    pub iss: String,        // always "https://lumi.cyberos.world" for Lumi tokens
    pub aud: String,        // always "https://memory.cyberos.world/sync"
    pub iat: i64,
    pub exp: i64,
    pub nbf: i64,
    pub roles: Vec<String>,
    pub rbac_v: u32,
    pub jti: String,        // RFC 7519 JWT ID (UUIDv4)

    // Lumi-specific claims (DEC-420)
    pub agent_persona: String,         // "cuo-<key>@<semver>"
    pub tenant_residency: String,      // vn-1 | sg-1 | eu-1 | us-1
    pub lumi_org_tenant: String,       // slug
    pub persona_version: String,       // semver
    pub sync_class_allowed: Vec<SyncClass>,
    pub anchor_chain_hash: String,     // 64-hex
}

impl LumiClaims {
    pub fn validate_shape(&self) -> Result<(), LumiClaimError> {
        if self.iss != "https://lumi.cyberos.world" {
            return Err(LumiClaimError::IssMismatch);
        }
        if self.aud != "https://memory.cyberos.world/sync" {
            return Err(LumiClaimError::AudMismatch);
        }
        crate::lumi::persona_version::validate_format(&self.agent_persona)?;
        crate::lumi::sync_class::validate_array(&self.sync_class_allowed)?;
        if !slug_re().is_match(&self.lumi_org_tenant) {
            return Err(LumiClaimError::LumiOrgTenantMalformed);
        }
        if !chain_hash_re().is_match(&self.anchor_chain_hash) {
            return Err(LumiClaimError::AnchorChainHashMalformed);
        }
        Ok(())
    }
}
```

### 3.3 — Sync class enum

```rust
// services/auth/src/lumi/sync_class.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncClass {
    Private,
    Shareable,
    Publishable,
    Shared,
    ClientVisible,
}

impl SyncClass {
    pub const ALL: &'static [SyncClass] = &[
        SyncClass::Private, SyncClass::Shareable, SyncClass::Publishable,
        SyncClass::Shared, SyncClass::ClientVisible,
    ];
}

pub fn validate_array(arr: &[SyncClass]) -> Result<(), LumiClaimError> {
    if arr.is_empty() {
        return Err(LumiClaimError::SyncClassEmpty);
    }
    Ok(())
}
```

### 3.4 — Verifier

```rust
// services/auth/src/lumi/verifier.rs
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header, decode};
use crate::lumi::claims::LumiClaims;
use crate::lumi::persona_version::PersonaVersionRegistry;

const LUMI_ISS: &str = "https://lumi.cyberos.world";
const LUMI_AUD: &str = "https://memory.cyberos.world/sync";
const PERSONA_VERSION_STALE_THRESHOLD: u32 = 2; // minor versions behind

pub fn verify_lumi_token(
    token: &str,
    jwks: &crate::oidc::jwks::JwksCache,
    persona_registry: &PersonaVersionRegistry,
    expected_residency: &str,
) -> Result<LumiClaims, LumiVerifyError> {
    // 1. Header check — alg pinned at RS256 (DEC-431)
    let header = decode_header(token).map_err(|_| LumiVerifyError::TokenMalformed)?;
    if header.alg != Algorithm::RS256 {
        return Err(LumiVerifyError::UnsupportedAlg(format!("{:?}", header.alg)));
    }
    let kid = header.kid.as_deref().ok_or(LumiVerifyError::KidMissing)?;
    let jwk = jwks.get(kid).ok_or(LumiVerifyError::UnknownKid)?;

    // 2. Build decoder
    let key = DecodingKey::from_rsa_components(
        jwk.n.as_deref().ok_or(LumiVerifyError::JwkMalformed)?,
        jwk.e.as_deref().ok_or(LumiVerifyError::JwkMalformed)?,
    ).map_err(|_| LumiVerifyError::JwkMalformed)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[LUMI_ISS]);
    validation.set_audience(&[LUMI_AUD]);
    validation.leeway = 60;

    // 3. Decode + validate signature/iss/aud/exp/nbf
    let decoded = decode::<LumiClaims>(token, &key, &validation)
        .map_err(|e| LumiVerifyError::TokenValidation(format!("{e:?}")))?;

    let claims = decoded.claims;
    claims.validate_shape().map_err(LumiVerifyError::Shape)?;

    // 4. Residency check (DEC-422)
    if claims.tenant_residency != expected_residency {
        return Err(LumiVerifyError::ResidencyMismatch {
            expected: expected_residency.into(), got: claims.tenant_residency.clone(),
        });
    }

    // 5. Persona-version staleness check (DEC-424)
    let live_version = persona_registry
        .get_live_version(&claims.agent_persona)
        .ok_or_else(|| LumiVerifyError::PersonaUnknown(claims.agent_persona.clone()))?;
    let behind = persona_registry.minor_versions_behind(&claims.persona_version, &live_version);
    if behind > PERSONA_VERSION_STALE_THRESHOLD {
        return Err(LumiVerifyError::PersonaVersionStale {
            token_version: claims.persona_version.clone(),
            live_version,
            minor_versions_behind: behind,
        });
    }

    Ok(claims)
}
```

### 3.5 — Issuer handler

```rust
// services/auth/src/handlers/lumi.rs
use axum::{Json, extract::State, http::StatusCode};
use cyberos_auth::rbac::Role;
use crate::lumi::{claims::LumiClaims, sync_class::SyncClass, repo, audit};

#[derive(Deserialize)]
pub struct IssueRequest {
    pub agent_persona: String,
    pub persona_version: String,
    pub lumi_org_tenant: String,
    pub sync_class_allowed: Vec<SyncClass>,
    pub anchor_chain_hash: String,
}

#[derive(Serialize)]
pub struct IssueResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub jti: String,
}

pub async fn issue_lumi_token(
    State(state): State<AppState>,
    claims: crate::jwt::Claims,
    Json(req): Json<IssueRequest>,
) -> Result<(StatusCode, Json<IssueResponse>), LumiIssueError> {
    // (1) Caller must have agent-persona role (DEC-432)
    if !claims.roles().contains(&Role::AgentPersona) {
        return Err(LumiIssueError::HumanCannotIssue);
    }

    // (2) Validate persona_version against registry
    state.persona_registry.validate_known(&req.agent_persona, &req.persona_version)?;

    // (3) Validate sync_class_allowed against per-tenant policy (FR-AI-005)
    let policy = state.tenant_policy.load(claims.tenant_id()).await?;
    state.lumi.validate_sync_class_against_policy(&req.sync_class_allowed, &policy)?;

    // (4) Resolve residency
    let residency = state.residency.resolve(claims.tenant_id()).await?;

    // (5) Issue token
    let jti = uuid::Uuid::new_v4().to_string();
    let ttl = state.config.lumi_token_ttl_seconds;
    let exp = chrono::Utc::now() + chrono::Duration::seconds(ttl as i64);
    let lumi_claims = LumiClaims {
        sub: claims.subject_id(), tid: claims.tenant_id(),
        iss: "https://lumi.cyberos.world".into(),
        aud: "https://memory.cyberos.world/sync".into(),
        iat: chrono::Utc::now().timestamp(),
        exp: exp.timestamp(),
        nbf: chrono::Utc::now().timestamp(),
        roles: vec!["agent-persona".into()],
        rbac_v: state.rbac_matrix.snapshot().version,
        jti: jti.clone(),
        agent_persona: req.agent_persona.clone(),
        tenant_residency: residency.code.clone(),
        lumi_org_tenant: req.lumi_org_tenant.clone(),
        persona_version: req.persona_version.clone(),
        sync_class_allowed: req.sync_class_allowed.clone(),
        anchor_chain_hash: req.anchor_chain_hash.clone(),
    };
    let token = state.signer.sign_rs256(&lumi_claims).await?;

    // (6) Persist to log + emit memory audit
    let mut tx = state.db.begin().await?;
    repo::insert_issuance_log(&mut tx, &lumi_claims, claims.subject_id(), &jti, ttl).await?;
    audit::emit_lumi_token_issued(&mut tx, &lumi_claims, claims.subject_id()).await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(IssueResponse { token, expires_at: exp, jti })))
}
```

---

## §4 — Acceptance criteria

1. **iss locked at https://lumi.cyberos.world** — token with different iss → 401 iss_mismatch.
2. **aud locked at https://memory.cyberos.world/sync** — different aud → 401 aud_mismatch.
3. **alg=RS256 enforced** — HS256 or none → 401 unsupported_alg.
4. **agent_persona format `cuo-<key>@<semver>`** — malformed → 401 agent_persona_malformed.
5. **agent_persona unknown key** → 401 agent_persona_unknown.
6. **tenant_residency in closed enum** — outside set → 451 residency_mismatch.
7. **tenant_residency mismatch with route** — 451 unavailable_for_legal_reasons.
8. **sync_class_allowed closed enum** — unknown value → 401 sync_class_unknown.
9. **sync_class_allowed empty** → 401 sync_class_empty.
10. **persona_version 2 minor versions behind** → 401 persona_version_stale + sev-2 audit.
11. **persona_version > 2 behind** → 401; counter increments.
12. **Human cannot issue Lumi token** — caller without agent-persona role → 403 human_cannot_issue.
13. **anchor_chain_hash 64-hex format** — malformed → 401 anchor_chain_hash_malformed.
14. **lumi_org_tenant slug pattern** — malformed → 401 lumi_org_tenant_malformed.
15. **POST issue happy path** — agent-persona role + valid request → 201 + token + log row + memory row.
16. **lumi_token_issuance_log append-only** — UPDATE/DELETE blocked from cyberos_app.
17. **jti uniqueness enforced** — duplicate jti at insert → 23505.
18. **TTL default 3600** — exp = iat + 3600.
19. **TTL configurable via env** — LUMI_TOKEN_TTL_SECONDS=600 → token expires at 600s.
20. **TTL < 300 rejected at startup** — service refuses to start.
21. **sync_class_allowed exceeds policy** → 403 sync_class_exceeds_policy.
22. **OTel span emitted** — auth.lumi.issue / verify with outcome attr.
23. **Counter `auth_lumi_issuance_total{outcome=success}` increments** per successful issuance.
24. **Counter `auth_lumi_persona_version_stale_total` increments** on stale-version rejection.
25. **Perf budget < 100ms p95 issue / 50ms p95 verify** — perf tests assert.
26. **`auth.lumi_token_issued` memory row carries jti + persona + version + residency + sync_class + ttl**.
27. **`auth.lumi_token_rejected` memory row carries reason** — for every failure path.

---

## §5 — Verification

```rust
// services/auth/tests/lumi_alg_confusion_test.rs
#[test]
fn hs256_token_rejected() {
    let token = mock_hs256_token();   // attacker forges using public key as HMAC secret
    let result = cyberos_auth::lumi::verifier::verify_lumi_token(
        &token, &mock_jwks(), &mock_registry(), "vn-1",
    );
    assert!(matches!(result, Err(LumiVerifyError::UnsupportedAlg(_))));
}

#[test]
fn alg_none_rejected() {
    let token = mock_none_alg_token();
    let result = cyberos_auth::lumi::verifier::verify_lumi_token(
        &token, &mock_jwks(), &mock_registry(), "vn-1",
    );
    assert!(matches!(result, Err(LumiVerifyError::UnsupportedAlg(_))));
}
```

```rust
// services/auth/tests/lumi_human_cannot_issue_test.rs
#[tokio::test]
async fn human_subject_cannot_issue_lumi(ctx: TestCtx) {
    let human_token = ctx.issue_tenant_admin_token().await;   // role=tenant-admin, no agent-persona
    let resp = ctx.post("/v1/auth/lumi/issue", &valid_body())
        .header("Authorization", format!("Bearer {human_token}"))
        .await;
    assert_eq!(resp.status(), 403);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "human_cannot_issue_lumi");
}

#[tokio::test]
async fn agent_persona_can_issue_lumi(ctx: TestCtx) {
    let agent_token = ctx.issue_agent_persona_token().await;
    let resp = ctx.post("/v1/auth/lumi/issue", &valid_body())
        .header("Authorization", format!("Bearer {agent_token}"))
        .await;
    assert_eq!(resp.status(), 201);
}
```

```rust
// services/auth/tests/lumi_persona_version_stale_test.rs
#[test]
fn version_3_minors_behind_rejected() {
    let live_version = "1.5.0";
    let token_version = "1.2.0";   // 3 minors behind
    let result = cyberos_auth::lumi::verifier::verify_lumi_token(
        &mock_token_with_version(token_version), &mock_jwks(),
        &mock_registry_with_live(live_version), "vn-1",
    );
    assert!(matches!(result, Err(LumiVerifyError::PersonaVersionStale { .. })));
}

#[test]
fn version_2_minors_behind_accepted() {
    let live_version = "1.5.0";
    let token_version = "1.3.0";   // 2 minors behind — at threshold
    let result = cyberos_auth::lumi::verifier::verify_lumi_token(
        &mock_token_with_version(token_version), &mock_jwks(),
        &mock_registry_with_live(live_version), "vn-1",
    );
    assert!(result.is_ok());
}
```

```rust
// services/auth/tests/admin_deny_list_test.rs
#[test]
fn vn1_token_at_eu1_endpoint_rejected() {
    let token = mock_token_with_residency("vn-1");
    let result = cyberos_auth::lumi::verifier::verify_lumi_token(
        &token, &mock_jwks(), &mock_registry(), "eu-1",
    );
    assert!(matches!(result, Err(LumiVerifyError::ResidencyMismatch { .. })));
}
```

```rust
// services/auth/tests/lumi_sync_class_closed_test.rs
#[test]
fn empty_sync_class_rejected() {
    let result = cyberos_auth::lumi::sync_class::validate_array(&[]);
    assert!(matches!(result, Err(LumiClaimError::SyncClassEmpty)));
}

#[test]
fn unknown_sync_class_rejected_at_deserialise() {
    let json = r#"["private", "made-up-class"]"#;
    let r: Result<Vec<cyberos_auth::lumi::sync_class::SyncClass>, _> = serde_json::from_str(json);
    assert!(r.is_err());
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 4 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-101** — RBAC catalogue (agent-persona role + role enum).

**Downstream:** none at slice 1 (this FR provides the JWT shape that downstream memory sync + FR-AUTH-2xx revocation consume).

**Cross-module:**
- **FR-AUTH-004** — JWT baseline + signing key + JWKS.
- **FR-AI-014** — persona-version registry (consumed by staleness check).
- **FR-AI-016** — residency policy.
- **FR-AI-005** — per-tenant policy YAML (consumed for sync_class policy gate).
- **FR-AI-003** — memory audit bridge.
- **FR-MEMORY-111** — PII scrubbing of lumi_org_tenant.

---

## §8 — Example payloads

### 8.1 — POST /v1/auth/lumi/issue request

```json
{
  "agent_persona": "cuo-cpo@0.4.1",
  "persona_version": "0.4.1",
  "lumi_org_tenant": "cyberskill",
  "sync_class_allowed": ["shareable", "publishable"],
  "anchor_chain_hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
}
```

### 8.2 — 201 CREATED response

```json
{
  "token": "eyJhbGciOiJSUzI1NiIsImtpZCI6IjFhYjJjM2Q0IiwidHlwIjoiSldUIn0...",
  "expires_at": "2026-05-16T11:00:00Z",
  "jti": "01HG7V8B0K8M4Z8Z8M8M8M8M8M"
}
```

### 8.3 — Decoded Lumi JWT claims

```json
{
  "sub": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "tid": "5e8f1d2a-...",
  "iss": "https://lumi.cyberos.world",
  "aud": "https://memory.cyberos.world/sync",
  "iat": 1747920731,
  "exp": 1747924331,
  "nbf": 1747920731,
  "roles": ["agent-persona"],
  "rbac_v": 2,
  "jti": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "agent_persona": "cuo-cpo@0.4.1",
  "tenant_residency": "vn-1",
  "lumi_org_tenant": "cyberskill",
  "persona_version": "0.4.1",
  "sync_class_allowed": ["shareable", "publishable"],
  "anchor_chain_hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
}
```

### 8.4 — auth.lumi_token_issued memory row

```json
{
  "kind": "auth.lumi_token_issued",
  "tenant_id": "5e8f1d2a-...",
  "issuer_subject_id_hash16": "9b1deb4d3b7d4bad",
  "agent_persona": "cuo-cpo@0.4.1",
  "persona_version": "0.4.1",
  "lumi_org_tenant_hash16": "abc123def4567890",
  "residency": "vn-1",
  "sync_class_allowed": ["shareable", "publishable"],
  "ttl_seconds": 3600,
  "token_jti": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — auth.lumi_persona_version_stale memory row (sev-2)

```json
{
  "kind": "auth.lumi_persona_version_stale",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "agent_persona": "cuo-cpo",
  "token_version": "0.2.0",
  "live_version": "0.4.1",
  "minor_versions_behind": 2,
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Lumi token revocation API + CRL endpoint** — FR-AUTH-2xx.
- **Distinct kid for Lumi vs FR-AUTH-004** — slice 2.
- **Human delegation flow (human authorises specific Lumi sync)** — slice 3 ADR.
- **Cross-Lumi token chain (Lumi A → Lumi B)** — FR-MEMORY-2xx.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| HS256 alg in token | header check | 401 unsupported_alg | Designed |
| alg=none in token | header check | 401 unsupported_alg | Designed |
| iss mismatch | validation | 401 iss_mismatch | Designed |
| aud mismatch | validation | 401 aud_mismatch | Designed |
| Unknown kid | JWKS lookup | 401 unknown_kid | Designed |
| Token expired | exp + leeway | 401 token_expired | Re-issue |
| persona_version > 2 behind | registry compare | 401 persona_version_stale + sev-2 | Refresh token |
| Unknown agent_persona key | registry lookup | 401 agent_persona_unknown | Designed |
| Residency mismatch | endpoint vs claim | 451 unavailable_for_legal_reasons | Use correct region |
| Empty sync_class_allowed | validator | 401 sync_class_empty | Designed |
| Unknown sync_class value | serde + validator | 401 sync_class_unknown | Designed |
| Human subject issues | role check | 403 human_cannot_issue_lumi | Use agent-persona subject |
| Slug malformed | regex | 401 lumi_org_tenant_malformed | Fix slug |
| anchor_chain_hash malformed | regex | 401 anchor_chain_hash_malformed | Fix hash |
| sync_class exceeds policy | policy lookup | 403 sync_class_exceeds_policy | Reduce array |
| jti collision | UNIQUE | 23505 | Should never happen (UUIDv4) |
| TTL < 300 in env | startup check | service refuses | Fix env |
| persona registry unreachable | registry error | 500 + sev-2 | Investigate |
| KMS signing key disabled | signer error | 500 + sev-1 | Rotate key |
| Append-only log UPDATE from app | SQL grant | permission denied | Designed |
| RLS bypass | USING | 0 rows | Designed |
| Cross-tenant log read | RLS | 0 rows unless root-admin | Designed |
| Token forwarded with different residency in URL | endpoint check | 451 | Designed |
| Refresh path missing | FR-AUTH-004 dependency | Existing path | Designed |
| > 5/h stale-version sustained | OBS rule | sev-2 alarm | Investigate |
| jsonwebtoken crate version drift | tests pin | CI fails | Pin version |
| concurrent issuance with same jti | UUID v4 | Collision astronomically improbable | None |
| Policy YAML unreachable | tenant config | 500 + sev-3 | Restore config |
| Subject deleted while log refs | FK RESTRICT | DELETE fails | Soft-delete subject first |
| chain_hash regex too permissive | unit test | CI fails | Tighten regex |
| sub_claim leaked in memory | PII scrub | Pre-commit | Designed |

---

## §11 — Implementation notes

- **Lumi JWT additive on FR-AUTH-004** — 5 new claims, 8 existing preserved; 13 total.
- **Distinct iss + aud** — security boundary that survives per-tenant AUTH compromise.
- **alg=RS256 pinned** — JWT-confusion attack defense.
- **persona_version staleness 2-minor tolerance** — covers normal rotation lag.
- **sync_class_allowed closed enum** — AGENTS.md §15 set.
- **anchor_chain_hash claim** — binds token to specific chain state; replay defense.
- **Same signing key as FR-AUTH-004 at slice 1** — operational simplicity; distinct iss provides boundary.
- **jti UUIDv4 + UNIQUE constraint** — revocation lookup target.
- **TTL 1h default, configurable 5min-24h** — short-lived; mitigates lack of revocation at slice 1.
- **Human cannot issue at slice 1** — EU AI Act Art. 13 clean transparency.
- **Residency check at verifier** — Decree 53 + GDPR + DORA enforcement.
- **Per-tenant sync_class policy enforced at issuance** — defense in depth with memory sync write-time check.
- **Append-only log via SQL grant** — forensic record.
- **4 memory audit kinds** — issued / verified / rejected / version_stale.
- **PII scrub lumi_org_tenant** — cross-tenant boundary.
- **OTel sampling 1% verify** — high-volume hot path; 100% on non-success.
- **Sev-2 alarm on stale-version > 5/h** — operator investigation prompt.
- **Distinct kid for Lumi** — slice 2 hardening.
- **Revocation API** — FR-AUTH-2xx; jti column ships here.

---

*End of FR-AUTH-108.*
