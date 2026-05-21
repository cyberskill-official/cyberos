# Changelog — AUTH

## 2026-05-18 — Wave-1+2 impl: FR-AUTH-103 SAML XML-DSig + FR-AUTH-106 GeoIP + policy

**FR-AUTH-103 SAML XML-DSig (slice-2) + xml-c14n hardening.** `services/auth/src/saml_sig.rs` (~520 lines): ds:Signature discovery, strict algorithm allowlist (RSA-SHA256 + SHA-256 + exc-c14n), enveloped-signature stripping, reference-by-ID resolution, RSA-PKCS1-v1.5 verify, hand-rolled X.509 → SPKI TLV walk. Migration 0017 adds per-IdP `allow_unsigned BOOLEAN DEFAULT FALSE`. `exc_c14n` rewritten as a proper tokeniser. 14 canonicaliser tests + 7 X.509/PEM tests.

**FR-AUTH-106 GeoIP + policy + CIDR + sticky-suppression (slices 2 + 3).** New `services/auth/src/geoip.rs` with `GeoIpResolver` trait, `MaxMindResolver`, `NullResolver` fallback. Activates `cross_continent_velocity` and `geo_velocity_exceeded` detectors. Migration 0018 ships `travel_policy`, `travel_cidr_allowlist`, `travel_policy_audit`. New `travel_policy.rs` — 60s `PolicyCache`, bounded-50k `StickySuppress` LRU. New `assess_login` wraps the detector chain. New `travel_admin.rs` exposes 5 routes. **All four login flows** now go through `assess_login`.

---

## 2026-05-19 — [AUTH] FR-AUTH-005 drained 17/17 + rustc floor bumped 1.83→1.88

**FR-AUTH-005** (admin REST list_tenants + list_subjects + revoke + unrevoke + cursor + jti deny-list) drained end-to-end in one Cowork session. 17 spec-vs-code gaps closed across 5 slices; ≈1,300 LOC src + tests. BACKLOG line 224 mutated `planned` → `[BLOCKED: 17 gaps]` → `shipped + strict-audited`. **All 6 Wave-1 MUST AUTH FRs (001/002/003/004/005/006) are now shipped + strict-audited** — wave-1-2 deploy table-stakes are drain-complete.

New modules: `services/auth/src/{cursor,deny_list,sessions}.rs`. New migration: `migrations/0021_sessions.sql` (relocated per DEC-MIGRATION-SLOT-001; slot 0007 was taken). New memory_bridge emitters: `emit_subject_revoked` + `emit_subject_unrevoked`. New test files: `admin_list_test.rs` + `admin_revoke_test.rs` + `admin_cursor_pagination_test.rs` + `admin_deny_list_test.rs`. OTel `#[tracing::instrument]` on all 4 admin handlers.

Architecture decisions logged in FR-AUTH-005-admin-rest.audit.md §10.5: **DEC-DENY-LIST-001** (in-memory slice-1; Redis lift = FR-AUTH-110), **DEC-CURSOR-SIGN-001** (HMAC-SHA256 via HKDF from `AUTH_CURSOR_SIGNING_SECRET` env), **DEC-MIGRATION-SLOT-001** (0007→0021 relocation). Structural G-012 enforcement: `DenyList` exposes no `remove()` API — unrevoke literally cannot clear the deny-list at the compile-time level.

Deferred follow-ups: **FR-AUTH-110** (Redis-backed deny-list lift for wave-2 horizontal scale) + **FR-AUTH-111** (closed revoke reason taxonomy enum: compromised / terminated / policy-violation / operator-error / other).

**`services/Cargo.toml` `rust-version` bumped 1.83 → 1.88** — `webauthn-rs 0.5.5`, `time 0.3.47`, `icu_* 2.2.0`, `base64urlsafedata 0.5.5`, `home 0.5.12` all now require ≥1.86/1.88. One-time operator step: `rustup toolchain install 1.88.0`. README §1 prerequisites table updated.

**Cascading build fixes after the bump** — `cargo +1.88.0 build -p cyberos-auth` surfaced 2 errors that were either pre-existing (borrow-checker stricter on this NLL path) or shaken loose by the sqlx 0.8 + ipnetwork 0.20 trait shuffle:

- **handlers.rs:1871** — `traceparent: Option<String>` was moved into `svc.issue(..., traceparent, ...)` then borrowed via `.as_deref()` for the memory audit emit at :1896. Fixed with `traceparent.clone()` at the move site per the compiler's own suggestion.
- **travel.rs:213 + travel_admin.rs:170 + travel_admin.rs:212/244 + travel_policy.rs:134** — `ipnetwork::IpNetwork` no longer satisfies `sqlx::Encode<'_, Postgres>` / `Decode<'_, Postgres>` (likely a version-coherence pitfall: sqlx-postgres pulls its own ipnetwork copy distinct from the auth crate's). Fixed by binding/reading at the DB boundary as `String` (Postgres INET accepts the textual CIDR; `::text` cast on read). The struct field `TravelPolicy::allowlist: Vec<IpNetwork>` is preserved — parsed from `String` at the read boundary via `filter_map(|(c,)| c.parse().ok())`.
- 4 unused-import warnings cleaned (`body::Body` in middleware.rs, `Redirect` in oidc.rs, `Redirect`+`Serialize` in saml.rs).

Compile-verify on macOS: `cd services && cargo +1.88.0 build -p cyberos-auth && cargo +1.88.0 test -p cyberos-auth` (was `+1.85.0`).

---

## 2026-05-14 — AUTH module page rewritten to Gold (P0 · slice 2 stub vs P3 full + Lumi tenant identity + RFC open Qs resolved)

Rewrote `website/docs/modules/auth.html` from 1169 → 1442 lines (+273 lines, +23%). Encodes the research review §2.4 reorder (AI Gateway BEFORE AUTH) and AUTH's distinct roles as P0 · slice 2 stub vs P3 full. Targeted Edit operations preserved every gold-quality detail of the prior content while adding 4 new strategic sections + risk/KPI extensions.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "P0 · slice 2 stub → P3 full · Lumi tenant identity · Agent-equal".
- **Hero tagline + lede** — explicit P0 · slice 2 stub vs P3 full distinction · cites reordered P0 sequence (AI Gateway @ P0 · slice 1 → AUTH @ P0 · slice 2 → MCP Gateway @ P0 · slice 3 → CHAT/CUO @ P0 · exit) · references RFC.md + sign-in mockup + MEMORY_AUTOSYNC_DESIGN.md.
- **Hero fact-grid** — split status into "P0 · slice 2 stub designed" + "P3 full designed", LoC into 1,500 stub + 7,000 full, RBAC into 5 stub + 22 full, dependencies + Lumi enablement.
- **NEW §0 "The bigger picture — three strategic moves"** — 3-card layout (Move 1 P0 · slice 2 stub / Move 2 P3 full / Move 3 Lumi tenant identity). Gantt chart Mermaid showing the reordered P0 build sequence end-to-end. Rationale for reorder cited from reviewer.
- **TOC** — added bigger-picture · stub-vs-full · rbac-catalogue · lumi-integration · open-questions entries.
- **NEW §2.5 "P0 · slice 2 stub vs P3 full"** — 12-row capability-contrast table covering login mechanism · MFA · RBAC catalogue · JWT signing · tenant isolation · audit-chain emission · admin surfaces · cost · LoC · tests · Lumi integration · SOC 2 evidence. Plus "Migration discipline" + "What stub doesn't compromise on" prose.
- **NEW §2.6 "22-role RBAC catalogue"** — full 22-row table with scope summary, stub-eligibility, and slice when each role lands. The 5 stub roles (root-admin · tenant-admin · tenant-member · service-account · agent-persona) are explicitly the first 5; the remaining 17 land across slices 3–5. Role-addition policy: ADR-gated, no code-only changes.
- **NEW §2.7 "AUTH ↔ Lumi's memory"** — full JWT claim shape (15 fields incl. tenant_id, tenant_residency, agent_persona, scope_grants) · sequence diagram of Lumi's memory verifying a sync push · 5-bullet contract requirements list (tenant_id non-removable, JWKS reachability, refresh-token reuse detection, agent-persona claims preserve agent-equal, residency pinning flows through).
- **NEW §2.8 "RFC open questions resolved"** — table addressing all 5 open Qs from RFC §6 with proposed defaults + rationale: Q1 workspace = new repo-root Cargo workspace · Q2 memory bridge = subprocess slice 4 → PyO3 slice 5 · Q3 tenant-0 bootstrap = `cyberos-auth bootstrap` CLI subcommand · Q4 HIBP = default-on with per-tenant opt-out · Q5 OBS = slice 1 stdout → slice 5 OTLP. Each becomes an ADR once Stephen signs off.
- **§12 Risks** — added 7 new (R-AUTH-011..017): stub stays past P3 · reorder regret · Lumi tenant-id spoofing · cross-shard JWT replay · sub-process audit-bridge bottleneck · tenant-0 bootstrap leak · PDPL Art. 38 SME grace lapse.
- **§13 KPIs** — added 7 new: stub-to-full migration coverage (≥95% T2+ subjects passkey-enrolled by P1 · exit) · mock-AUTH retirement · Lumi tenant-id verification rate · cross-shard rejection · audit-bridge p99 · SME-grace lapsed tenants · 22-role catalogue stability.
- **§17 References** — replaced PRD/SRS section refs (stripped) with services/auth/RFC.md, sign-in mockup, MEMORY_AUTOSYNC_DESIGN.md §6, RESEARCH_REVIEW §2.4 (cited verbatim), AUDIT_AND_PLAN, feature-request-audit skill, AGENTS.md §3.6+§11.

Verified:
- 1442 lines parses cleanly
- 23 top-level sections (was 18) including 4 strategic new ones
- Mermaid gantt chart documents the reordered P0 sequence
- All 5 RFC §6 open questions now have proposed defaults visible on the page

The AUTH page now reads as the complete answer to: (1) why AUTH is not P0 #1 (research review §2.4), (2) what the P0 · slice 2 stub actually contains vs the P3 full target, (3) how AUTH enables Lumi's memory tenant isolation, (4) what the 5 open RFC questions resolve to. A new engineer reading this page cold can pick up RFC.md and start slice 1.

---

## 2026-05-14 — AUTH module RFC + sign-in mockup

- Added `services/auth/RFC.md` — implementation RFC with 5-slice ship plan, audit-chain integration design, and 5 open questions blocking slice 1.
- Added `services/auth/mockups/sign-in.html` — first AUTH UI mockup applying design-system Part 21 Liquid Glass defaults, Umber + Ochre anchors, Be Vietnam Pro first, passkey-first flow with password fallback, MFA chips, memory audit-chain trust footnote.
- Verification pass against shipped modules:
  - memory: 222 tests pass + 1 skip (numpy + jsonschema needed for full green). Real bug found AND fixed: `check_manifest_validates` was skipping parseability when jsonschema absent → `cyberos state` returned READY on a broken manifest. Patched to always parse `manifest.json` first (regardless of jsonschema availability) and report `False` on `JSONDecodeError`; the optional schema-validation layer still skips cleanly when jsonschema is absent. Verified: all 4 `tests/test_state.py` tests pass, full suite 238 pass / 1 skip / 0 fail. Also verified by simulating absent jsonschema via import hook — good manifest still returns True with "parseability OK, schema skip"; bad manifest returns False with "manifest.json unparseable: ...".
  - skill: 20 SKILL.md bundles structurally verified, 4 crates, 8 inline Rust tests. `cargo build` not run (sandbox-only limitation).
  - cuo: 15/15 pytest + 15/15 routing fixtures pass. Catalog discovers all 20 skills correctly.
- Stale-claim drift surfaced (none are blockers, all are doc-only):
  - Memory tests: bootstrap says 245, README says 255, actual is 238 collected.
  - Doctor invariants: bootstrap says 16, README says 15, actual is 13 on a fresh store.
  - Docs pages: bootstrap says 32, strategy says 31, actual is 33 HTML files (32 user-facing + nav include).
  - Strategy §3 Tier-1 #2 and §5 Session-1 #1 list "wire Pagefind" as a to-do; Pagefind is already built and serving (v1.5.2, 32 pages indexed).
  - DEPLOYMENT.md is at `website/docs/DEPLOYMENT.md` (bootstrap implies it lives at `website/`).
- Docs site deploy-prep findings:
  - 6 real broken internal links to 2 missing architecture pages: `architecture/services.html` (5 refs from LEARN/HR/INV/ESOP/REW) and `architecture/runtime.html` (1 ref from CHAT). These are demand-gen blockers — fix before public deploy or convert the link targets.

