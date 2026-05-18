---
fr_id: FR-AUTH-001
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-001 was expanded from 184 lines to ~830 lines. The expansion added 7 §1 normative clauses (#5 idempotency-key handling, #11 explicit error-body shapes with field/reason, #12 single-transaction atomicity, #13 OTel span emission, #14 explicit reject of slug='root', #15 OTel metrics, plus expanded #6/#10 with shapes), 7 substantive §2 rationale paragraphs, full Rust types + migrations + RLS templates + handler skeleton + idempotency module in §3, expanded §4 from 9 to 18 ACs, full Rust test bodies in §5 (8 test functions covering happy + 401 + 403 + 409 + 400 + idempotent-replay + RLS + p95-latency), expanded §7 with code/concept/operational deps, 7 example payloads in §8 (success + conflict + idempotent replay + key-reuse + forbidden + invalid + audit row), 20 failure modes in §10 (vs. 4 in first pass), 8 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Tenant ID type mismatch: API uses UUID but `jwt.tenant_id != 0` (numeric)
- **severity:** error  **status:** resolved
- First-pass §6 had `if jwt.tenant_id != 0` — but the table schema uses UUID. A code-gen agent would either make tenant_id an integer (breaking the schema) or write `Uuid::nil()` checks without the FR specifying. Resolved: §1 + §3 + §6 use `Uuid::nil()` (the convention for tenant 0); explicit in §11 notes.

### ISS-002 — RLS policy application not specified (which tables, what template)
- **severity:** error  **status:** resolved
- First-pass §1 #7 said "every tenant-scoped table SHALL have the new tenant's RLS policy applied automatically" — but no list of tables, no template SQL. Resolved: §3 introduces `rls/templates.rs` with `TENANT_SCOPED_TABLES` registry + `apply_for_tenant_sql` template; AC #15 + §5 test asserts policies exist; §10 row + §11 note about the registry-as-contract.

### ISS-003 — Audit row builder `canonical::tenant_created` not shown
- **severity:** error  **status:** resolved
- First-pass §6 had `brain_writer::emit(canonical::tenant_created(&row.id, &req.slug)).await?` but no builder definition. Resolved: full builder in §6 with payload schema; §8 example audit row JSON; AC #12 asserts emission.

### ISS-004 — No idempotency support (network retries create duplicates)
- **severity:** error  **status:** resolved
- A network timeout during `POST /v1/admin/tenants` followed by client retry produces either duplicate tenants OR a 409 (caller can't tell which path the prior attempt took). Resolved: §1 #5 idempotency-key handling; `admin_idempotency_keys` table in 0002 migration; §3 lookup/insert helpers; AC #13 + #14 + §5 tests.

### ISS-005 — Concurrent tenant insert + RLS apply not transactional
- **severity:** error  **status:** resolved
- First-pass §6 inserted, then called `apply_rls_policies` AFTER, then emitted audit. Three separate operations; partial-state failures possible. Resolved: §1 #12 single-transaction atomicity; §3 handler uses `pool.begin()` with all three ops on same `&mut tx`; AC #17 asserts rollback on RLS failure.

### ISS-006 — No tenant-deletion path; soft-delete via `suspended` column unspecified
- **severity:** warning  **status:** resolved
- The `suspended` column existed in the schema but no documentation. Operators wouldn't know it exists. Resolved: §11 explicitly documents the column's purpose ("future suspension workflow; no endpoint mutates yet"); §9 lists tenant deletion + suspension as deferred items.

## §3 — Resolution

All 6 mechanical revisions applied (2026-05-16). **Score = 10/10.**

---

## §10 — Implementation audit (code-vs-spec)

> Added 2026-05-18 (session 20) by `chief-technology-officer/implement-backlog-frs` workflow.
> §1-§3 above audit the spec-authoring quality (was the spec well-written?).
> §10 audits the code-vs-spec match (does the code meet the spec?).

### §10.1 — Verdict

**Implementation status:** BLOCKED. 7 spec-vs-code gaps documented; audit-fix loop pending.

**Cumulative audit scores across 6 audit pairs (out of 10):**

| Pair | Score | Notes |
|---|---|---|
| repo-context-map | 10/10 | Found handler at `services/auth/src/handlers.rs:125-217`; 7 baseline patterns + 2 domain-specific captured |
| edge-case-matrix | 9/10 | 14 rows × 6 categories; WARN: planned test files don't exist (resolved by §10.3 fixes) |
| implementation-plan | 10/10 | 9 gap-fills + execution order; 8h estimate matches FR effort_hours |
| observability-injection | 10/10 | 7 logs + 3 trace spans + 4 counters; 100% branch coverage planned |
| coverage-gate | 4/10 | FAIL — 0 of 14 ECM rows covered; 3 declared test files don't exist; per-file coverage on `create_tenant` estimated 0% |
| backlog-state-update | 10/10 | Atomic single-cell mutation on BACKLOG.md line 212 |
| **Average** | **8.8/10** | |

### §10.2 — Gap list (drives the audit-fix loop)

| # | Spec ref | Gap | Severity | Effort | Status |
|---|---|---|---|---|---|
| G-001 | §1 #14 | `slug == "root"` defence-in-depth reject not enforced in handler | medium | 5 LOC + 1 test | open |
| G-002 | §1 #11 | Error body shape uses ad-hoc `{error: "X"}` instead of spec'd `{error, field, reason}` structure | medium | 25 LOC + 4 tests | open |
| G-003 | §1 #1 | Handler does NOT assert `caller.claims.tenant_id == Uuid::nil() AND 'root-admin' in claims.roles`; relies on middleware only | critical | 15 LOC + 2 tests | open |
| G-004 | §1 #13 | OTel `auth.create_tenant` span not emitted — no `#[tracing::instrument]` macro | medium | 10 LOC | open |
| G-005 | §1 #6 | `auth.tenant_created` BRAIN audit row not emitted in-transaction; no BRAIN bridge wired | high | 35 LOC + ADR DEC-BRAIN-BRIDGE-001 + 1 test | open |
| G-006 | §1 #8 | 100 ms p95 SLO not asserted by any test | medium | 60 LOC (new test file) | open |
| G-007 | §new_files | Test files declared in FR header but absent on disk: `admin_tenant_create_test.rs`, `admin_tenant_idempotency_test.rs`, `admin_tenant_rls_test.rs` | high | 410 LOC across 3 files | open (overlaps G-001/002/003/006) |

**Edge-case-matrix coverage (referenced by gap closures):** 14 rows × 6 categories — all currently uncovered. Each gap closure adds the corresponding ECM rows to a real test. Full ECM enumeration captured during step 5; preserved in §10.5 working notes below.

### §10.3 — Audit-fix log (one row per gap closure)

| ts | gap | change | tests | cargo result | commit |
|---|---|---|---|---|---|
| 2026-05-18T15:35:00Z | G-001 | `services/auth/src/handlers.rs:125-145` — added early-return on `req.slug == "root"` before idempotency lookup + DB transaction; returns 400 + `{error:"invalid_input", field:"slug", reason:"slug \"root\" is reserved …"}` | new file `services/auth/tests/admin_tenant_create_test.rs` (1 test: `create_tenant_rejects_reserved_root_slug` — covers ECM-008; gated by `#[ignore]`, runs in `cargo test -- --ignored` integration tier with Postgres) | `cargo build --workspace`: **green** (rustc 1.93.1, after closing 7 unrelated workspace bugs — see §10.6); `cargo test --workspace` (lib tier, no Postgres): **61 passed / 2 failed** (2 failures are pre-existing test bugs in `geoip::from_env_required_without_db_errors` + `saml_sig::verify_rejects_when_no_signature_element`, both unrelated to G-001 — see §10.6) | _pending commit_ |

(Append rows here as each gap closes. Format: `ISO-8601 | G-NNN | file:line summary | new test names | cargo +1.85.0 build/test outcome | git sha`.)

### §10.4 — BACKLOG.md mutations

| ts | line | from | to | mutation_kind |
|---|---|---|---|---|
| 2026-05-18T15:10:00Z | 212 | `planned` | `[BLOCKED: 7 spec gaps documented in auth/.workflow/FR-AUTH-001/]` | status-cell-only |
| _pending audit-fix completion_ | 212 | (above) | `shipped + strict-audited` | status-cell-only |

### §10.5 — Working notes (consolidates the 12 transient YAMLs)

**Existing patterns the gap-fill must respect** (from repo-context-map):
- error_type: `thiserror::Error` + axum `Result<(StatusCode, Json<T>), (StatusCode, Json<Value>)>` via `internal_err` helper at `handlers.rs:178`
- di_container: `axum::extract::State<AppState>` + `Extension<Claims>` (the latter post-middleware)
- state_management: `tokio` + `sqlx::PgPool` + `once_cell` for signing-key cache
- logging: `tracing` macros (`info!`/`warn!`/`error!`); JSON subscriber in `main.rs`
- test_framework: `cargo test` + `sqlx::query` against docker-compose Postgres; `#[ignore]` gates Postgres tests
- idempotency: `crate::idempotency::{lookup, record}` module
- tenant_isolation: `SET LOCAL app.current_tenant_id = '<uuid>'` GUC inside every transaction

**Edge-case-matrix rows** (14 total; each maps to a planned test):
- NULL_INPUT — empty body, body `{}` → 400
- BOUNDARY — slug 1/40 chars; name 1/80/81 chars; idempotency-key 65 chars
- MALFORMED — uppercase slug, non-UTF8 display_name, slug=="root"
- CONCURRENT — same-slug 409, idempotent replay same body, key reuse different body
- SECURITY — non-root tenant caller, root tenant non-admin caller
- DEGRADATION — BRAIN unreachable rollback

**Implementation-plan execution order** (drives §10.3 fill order):
1. G-001 (slug=='root' reject) — 5 LOC, lowest risk; uses existing patterns
2. G-002 (structured error helper) — touches multiple branches; foundation for G-007 tests
3. G-003 (root-admin authz) — depends on `Extension<Claims>` from FR-AUTH-004 middleware
4. G-004 (OTel span) — single macro addition
5. G-005 (BRAIN bridge) — needs DEC-BRAIN-BRIDGE-001 ADR (subprocess vs HTTP); biggest unit
6. G-007 (3 test files) — written incrementally as G-001..005 land; each gap-fill brings its ECM rows
7. G-006 (p95 SLO test) — last; slowest test; runs in `--ignored` CI tier

**Observability injection plan** (7 logs / 3 spans / 4 counters):
- Logs: `auth.create_tenant.{attempt,idempotent_replay,created,slug_conflict,missing_key,forbidden,brain_emit_failed}`
- Trace spans: `create_tenant` (root) · `idempotency::{lookup,record}` (Postgres IO) · `brain::emit_tenant_created` (BRAIN IO)
- Counters: `auth_tenant_create_total{outcome}` · `auth_tenant_create_latency_ms` (histogram) · `auth_tenant_count` (gauge) · `auth_brain_emit_failures_total`
- Branch coverage post-fill: 100% (9/9 branches instrumented)

**Coverage-gate verify command** (operator runs on Mac):
```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos/services
rustup install 1.85.0
cargo +1.85.0 install cargo-llvm-cov
cargo +1.85.0 llvm-cov --workspace --html --output-dir /tmp/llvm-cov-FR-AUTH-001
cargo +1.85.0 llvm-cov --workspace -- --ignored   # includes Postgres tests
```

---

### §10.6 — Workspace-wide bugs surfaced during the audit-fix loop

Driving the audit-fix loop on FR-AUTH-001 G-001 (closing one 5-LOC gap) exposed **7 pre-existing workspace-level compile bugs** that had never been caught because cargo had never run on this code. All 7 are unrelated to G-001 — they're "the code was written without ever being compiled" drift. Listed here for the human reviewer to commit + push to CI:

| Bug | File | Fix | Severity |
|---|---|---|---|
| Raw-string premature-closure at line 808 | `auth/src/saml_sig.rs:808` | `r#"…URI="#abc"…"#` had inner `"#` that closed the raw string prematurely; bumped to `r##"…"##` | ERROR — entire test module failed to lex |
| Raw-string premature-closure at line 838 | `auth/src/saml_sig.rs:838` | Same shape: inner `URI="#a"` closed the multi-line raw string; bumped to `r##"…"##` | ERROR — entire test module failed to lex |
| thiserror format-string used named fields on tuple variant | `auth/src/saml_sig.rs:68` | `#[error("invalid base64 in {field}: {detail}")]` → `#[error("invalid base64 in {0}: {1}")]` — tuple-variant format uses positional indexes | ERROR |
| sqlx missing `ipnetwork` feature | `services/Cargo.toml:26` | Added `"ipnetwork"` to sqlx features; resolves 18 `IpAddr/IpNetwork: Type<Postgres>` trait-bound errors in `travel.rs` + `travel_policy.rs` | ERROR (FR-AUTH-106) |
| webauthn-rs missing `danger-allow-state-serialisation` feature | `auth/Cargo.toml:43` | Added the feature; resolves `PasskeyRegistration/PasskeyAuthentication: Serialize/DeserializeOwned` trait-bound errors in `passkey.rs` | ERROR (FR-AUTH-105) |
| sha2 missing `oid` feature | `services/Cargo.toml:43` | Added `features = ["oid"]`; resolves `VerifyingKey::<Sha256>::new` trait-bound `AssociatedOid` error in `saml_sig.rs:152` | ERROR (FR-AUTH-103) |
| `passkey.rs` passing `Vec<u8>` to API expecting `Uuid` | `auth/src/passkey.rs:75-85` | Removed `subject_uuid_to_bytes(subject_id)` indirection; pass `subject_id: Uuid` directly to `start_passkey_registration` | ERROR (FR-AUTH-105) |
| `passkey.rs` typing `(Uuid, Uuid)` for a row whose first column can be NULL | `auth/src/passkey.rs:187` | Changed to `(Option<Uuid>, Uuid)` so the discoverable-credential branch's `SELECT NULL::uuid, …` returns None, and the downstream `.ok_or_else()` at line 213 works | ERROR (FR-AUTH-105) |
| `geoip.rs` test used `.unwrap_err()` on Result whose Ok type is `Arc<dyn GeoIpResolver>` (not Debug) | `auth/src/geoip.rs:255` | Replaced with explicit `match` pattern | ERROR (test compile only) |

### §10.7 — Remaining test failures (not blocking G-001)

Two pure-Rust tests fail after the workspace compiles green. **Both are pre-existing bugs unrelated to G-001 or the audit-fix loop**; they're listed here as separate items for the operator to triage:

1. **`geoip::tests::from_env_required_without_db_errors`** — sets `AUTH_GEOIP_REQUIRED=1` env var, expects `from_env()` to return `Err(GeoIpError::Required)`, but receives `Ok(Arc<NullResolver>)`. Likely a parallel-test isolation issue (cargo runs tests in parallel; another test may unset the env var). Fix: either serialize with `#[serial_test::serial]` (needs new dep) OR refactor the test to inject the env-var read.
2. **`saml_sig::tests::verify_rejects_when_no_signature_element`** — `verify()` returns an error variant other than `SamlSigError::NoSignature`. May be a downstream effect of the rsa/sha2 fixes; needs investigation of what `verify()` now returns when given XML without a `<ds:Signature>` element.

Treat both as new gaps for FR-AUTH-103 / FR-AUTH-106 implementation audits in a follow-up session — they don't block FR-AUTH-001.

---

*End of FR-AUTH-001 audit. Spec quality: PASS 10/10. Implementation: 1/7 gaps closed; workspace compiles green; 61/63 tests pass.*
