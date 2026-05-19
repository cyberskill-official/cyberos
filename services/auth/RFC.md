# AUTH module — Implementation RFC

**Status:** draft, 2026-05-14
**Author:** Stephen Cheng (CyberSkill)
**Spec:** `../../website/docs/modules/auth.html` (1,218 lines, 5W1H2C5M + 12 internal components + 20 FR-AUTH + RBAC catalogue)
**Lives at:** `services/auth/` once approved
**Depends on:** memory module (audit chain), OBS module (telemetry — not built; deferred dep)

---

## 1. Decision summary

| Question | Decision | Why |
|---|---|---|
| Language | **Rust** | Spec is unambiguous (axum + sqlx); aligns with skill module's existing Rust workspace |
| Web framework | axum 0.7 | Per spec; lowest-friction Tokio-native HTTP |
| DB | PostgreSQL 16 + Redis 7 | Per spec; RLS by `tenant_id`, Redis caches the hot-path RBAC view |
| JWT scheme | RS256, 15 min access + 30 d rotating refresh | Per spec FR-AUTH-003 / FR-AUTH-004 |
| MFA | WebAuthn (mandatory at T2+) + TOTP (member tier) | webauthn-rs + totp-rs |
| Signing-key custody | AWS KMS wrap; JWKS published | KMS rotation; isolate signer from verifier |
| RBAC model | Closed catalogue, 22 roles per PRD §8.6.1 | No ABAC at P0; ship narrow, expand later |
| Audit integration | Every auth decision → memory module audit chain | Via canonical Writer (memory module's `cyberos.core.writer`), NOT flat files |
| Multi-tenancy | Per-row RLS on every identity table | `tenant_id` claim in JWT; predicate on every cross-module call |

The spec is detailed enough that the RFC's job is **sequencing**, not re-architecting.

---

## 2. Module layout (mirrors spec §"Internal components")

```
services/auth/
├── Cargo.toml                  member of the workspace at repo root
├── README.md                   status table + place in CyberOS
├── RFC.md                      this file
├── migrations/                 sqlx migrations; RLS-enabled tables
│   ├── 0001_tenants.sql
│   ├── 0002_subjects.sql       (humans + service accounts + agent personas)
│   ├── 0003_credentials.sql    (password / webauthn / totp / api-token)
│   ├── 0004_sessions.sql
│   ├── 0005_roles_permissions.sql
│   ├── 0006_subject_role.sql
│   ├── 0007_audit_decisions.sql
│   └── 0008_effective_view.sql (materialised, Redis-cached)
├── src/
│   ├── lib.rs
│   ├── main.rs                 binary; loads config; spawns axum server
│   ├── oidc.rs                 OIDC discovery + JWKS
│   ├── oauth.rs                OAuth 2.1 token endpoint (PKCE mandatory)
│   ├── webauthn.rs             credential create + assertion verify
│   ├── totp.rs                 enrol + verify; replay window
│   ├── rbac.rs                 predicate evaluator; hot-path cache
│   ├── scope.rs                Scope Contract Grant resolver (reads memory persona sheets)
│   ├── session.rs              issue / validate / revoke
│   ├── jwt.rs                  RS256 via KMS; JWKS publication
│   ├── impossible_travel.rs    geographic-velocity check
│   ├── password.rs             Argon2id + HIBP k-anonymity
│   ├── device.rs               device fingerprinting + new-device email
│   ├── admin.rs                admin REST: tenant + role CRUD
│   ├── mcp.rs                  MCP tool surface (token issue / RBAC predicate)
│   ├── audit.rs                bridge → memory module's canonical Writer
│   └── types.rs                shared types
└── tests/
    ├── oauth_conformance.rs    RFC 6749 + RFC 7636 conformance
    ├── webauthn_flow.rs        register + authenticate happy + edge paths
    ├── rbac_predicate.rs       all 22 roles × representative actions
    └── audit_chain.rs          decisions land on memory chain (Writer integration)
```

---

## 3. Slice plan — five mergeable PRs

The 7,000 LoC + 120 tests estimate is realistic for ~5–6 weeks of focused work. Slicing for parallel review:

### Slice 1 — Scaffold + tenant + subject CRUD (week 1)

- Cargo crate, axum router skeleton, sqlx setup, RLS-enabled `tenants` + `subjects` tables.
- Admin REST: `POST /tenants`, `POST /tenants/:id/subjects`, `GET /subjects/:id`.
- 15 tests: tenant isolation, RLS enforcement, schema migrations.
- **Mergeable when:** `cargo test -p cyberos-auth` green; integration test creates two tenants and proves cross-tenant read fails.

### Slice 2 — Password + session + JWT (week 2)

- `password.rs` (Argon2id + HIBP), `session.rs` (issue/revoke), `jwt.rs` (RS256 with **dev** keys; KMS deferred to slice 5).
- Public REST: `POST /auth/login` (password), `POST /auth/refresh`, `POST /auth/logout`.
- 25 tests: password hashing edge cases, refresh rotation + reuse detection, JWT claim shape.
- **Mergeable when:** full username/password login → access+refresh token round-trip works; refresh-reuse invalidates session.

### Slice 3 — WebAuthn + TOTP (week 3)

- `webauthn.rs` (passkey enrol + assertion), `totp.rs` (RFC 6238).
- Public REST: `POST /auth/webauthn/register`, `/auth/webauthn/authenticate`, `/auth/totp/enrol`, `/auth/totp/verify`.
- 25 tests: WebAuthn happy path, multi-credential, TOTP replay window, T2+ MFA enforcement.
- **Mergeable when:** end-to-end passkey login works against a real authenticator emulator.

### Slice 4 — RBAC + Scope Contract + audit-chain bridge (week 4)

- `rbac.rs` (predicate eval + Redis cache), `scope.rs` (memory-backed agent persona scopes), `audit.rs` (memory module bridge).
- 22 PRD §8.6.1 roles loaded as seed data; gRPC `RBAC.Check` internal API.
- 35 tests: predicate matrix (22 roles × ~20 actions), Redis-cache invalidation, audit-row schema, cross-module call sequence.
- **Mergeable when:** another module can call `RBAC.Check(subject, action, resource)` over gRPC and the decision appears as a memory record on the memory audit chain.

### Slice 5 — KMS + impossible-travel + device + OIDC + MCP (week 5–6)

- `jwt.rs` switches dev-keys → AWS KMS wrap + JWKS rotation endpoint.
- `impossible_travel.rs`, `device.rs`, `oidc.rs` (discovery + ID-token), `mcp.rs` (MCP tool surface for agents).
- 20 tests: KMS rotation roundtrip, geographic-velocity edges, OIDC discovery doc matches spec, MCP tool I/O.
- **Mergeable when:** production-ready; passes OWASP Gen AI Top-10 mitigations checklist (NFR-SEC-001..012).

---

## 4. Audit-chain integration — non-negotiable detail

Every auth decision (login attempt, MFA challenge, token issue, RBAC predicate evaluation, session revocation) MUST land on the memory module's audit chain. Implementation:

- `services/auth/src/audit.rs` calls into the memory module via a thin shim. For the Rust↔Python boundary, two options:
  1. **Subprocess shim** (simplest, slice-4-acceptable): `cyberos --store $memory put ...` via stdin. Slow but correct.
  2. **PyO3 binding** (preferred, slice-5 target): link `cyberos.core.writer` as a Python module embedded in the Rust binary. Requires the memory module to expose a stable C-ABI or PyO3 surface — TBD with memory module owner.

Slice 4 ships option 1; slice 5 evaluates option 2 with the memory module owner.

The audit record schema for AUTH decisions:

```jsonc
{
  "op": "put",
  "path": "memories/decisions/auth-{decision_id}.md",
  "actor": "service:auth@{version}",
  "extra": {
    "tenant_id": "...",
    "subject_id": "...",
    "decision": "allow" | "deny" | "challenge",
    "action": "login.password" | "token.issue" | "rbac.check.{action}",
    "resource": "...",
    "reason": "...",
    "client_ip": "redacted-per-policy",
    "device_fingerprint_sha256": "..."
  }
}
```

Per AGENTS.md §11, the *fact* of an auth decision is itself a leaf on the memory chain. Per §3.6, redaction is allowed but the audit row of the redaction is itself unerasable.

---

## 5. Risks worth pre-empting

| # | Risk | Mitigation |
|---|---|---|
| 1 | webauthn-rs major version churn (the lib is pre-1.0) | Pin to a known-good minor; vendor the dep if necessary at slice 3 |
| 2 | sqlx + axum + tokio combinatorial test flakiness | Use `cargo nextest`; isolate Postgres + Redis containers per integration test |
| 3 | Memory-module bridge (Rust ↔ Python) is slow | Slice 4 accepts subprocess overhead (<50ms p99 for `put`); slice 5 evaluates PyO3 |
| 4 | KMS signing-key custody adds ~$10/mo even at dev scale | Acceptable; KMS-backed JWT is non-negotiable for prod per spec §"Token lifetime budget" |
| 5 | Impossible-travel false positives during VPN use | Configurable per-tenant; default off until slice-5 user research |
| 6 | The 22-role catalogue may drift from PRD §8.6.1 as other modules ship | Roles loaded from `migrations/0005_roles_permissions.sql`; require ADR for any addition |
| 7 | Audit-row volume (every RBAC check is a row) blows up memory size | Sampling not allowed (per spec); but memory consolidation §7 archives sealed segments — measure at slice 4, decide on Redis aggregation if needed |

---

## 6. Open questions for Stephen

These are decisions I want explicit answers on **before slice 1 lands**:

1. **Workspace membership** — should `services/auth/` be in the same Cargo workspace as `skill/`, or its own workspace? Skill's workspace is at `skill/Cargo.toml`; if AUTH joins, we'd lift the workspace root to repo top-level. Recommend: **new repo-root workspace**, with `skill/` and `services/auth/` as members.

2. **Memory bridge timing** — slice 4 subprocess-shim or insist on PyO3 in slice 4? Subprocess is mergeable in 1 week; PyO3 adds ~1 week. Recommend: **ship subprocess in slice 4, PyO3 in slice 5**.

3. **First tenant for dev** — how do we bootstrap tenant 0 ("CyberSkill itself") in slice 1 without an admin token? Recommend: `cyberos-auth bootstrap` CLI subcommand that runs as root and seeds tenant 0 + the first admin subject with a one-time enrolment URL.

4. **HIBP integration toggle** — slice 2 calls HIBP for breach-list check. Outbound HTTPS from the auth service in P0. OK or defer behind a feature flag? Recommend: **enabled by default; per-tenant opt-out**.

5. **OBS not yet built** — telemetry sinks are TBD. Slice-1 emits structured tracing logs to stdout; slice 5 switches to OTLP once OBS lands. OK?

---

## 7. Definition of done

AUTH ships when all of the following are green:

- [ ] All 5 slices merged to `main`
- [ ] `cargo test --workspace -p cyberos-auth` ≥ 120 tests pass
- [ ] OAuth 2.1 conformance suite (oauth2-conformance-suite Docker image) clean
- [ ] WebAuthn L3 conformance vendor-agnostic test (FIDO MDS metadata) clean
- [ ] OWASP Gen AI Top-10 checklist annotated, each mitigation linked to a test
- [ ] Memory-bridge integration test proves an auth decision lands on the memory audit chain and `cyberos --store $memory verify` stays clean
- [ ] AUTH module README + CHANGELOG written
- [ ] `cyberos doctor` invariants extended with one auth-related check (proposal: `auth-jwks-reachable`)
- [ ] Spec page `website/docs/modules/auth.html` updated to "shipped"
- [ ] Strategy §6 12-month markers updated to reflect AUTH ship date

---

*End of RFC.*
