# Wave 4 - hardening (IMP-031..045)

Goal: architecture and security debt paid down continuously alongside the waves. Order within this file is flexible; respect `depends_on` and pick by priority. Report sections: 3.1-3.3.

---

### IMP-031: unified error envelope crate

`refs: R1 | prio: p1 | effort: m | deps: - | area: arch`

Context: at least three error response shapes exist across services; clients and agents pay for the inconsistency.

Scope: define `shared/cyberos-http-error` (error code enum, message, request id, optional details; IntoResponse impl; helpers for common cases); migrate auth and chat first, remaining services follow one crate per commit; document the envelope in `docs/adrs/`.

Acceptance:
- [ ] Crate with unit tests; auth + chat migrated with unchanged-or-better client behavior (web client verified).
- [ ] Envelope ADR merged; remaining services tracked as checklist in this block.
- [ ] Gate green per touched crate.

Touches: `services/shared/`, `services/{auth,chat}/`, later the rest.

---

### IMP-032: extract cyberos-service-kit

`refs: R2 | prio: p1 | effort: l | deps: IMP-031 | area: arch`

Context: every service re-implements bootstrap (config, DB pool, JWKS verify middleware, CORS toggle, health route) by copy; drift multiplies review surface.

Scope: `shared/cyberos-service-kit` providing config loader, pool init, tracing init (hosting IMP-017's OTLP layer), JWT/JWKS verification middleware, dev-CORS toggle (hosting IMP-002's prod refusal), `/healthz`, and `/metrics` (hosting IMP-018). Migrate services one per commit; each migration deletes the local copy.

Acceptance:
- [ ] Kit crate tested; at least 4 services migrated with net-negative LOC.
- [ ] IMP-002/017/018 behaviors preserved by kit tests.
- [ ] Migration checklist for remaining services recorded here.

Touches: `services/shared/`, each service.

---

### IMP-033: wire cloud router adapters in ai-gateway

`refs: R3 | prio: p1 | effort: m | deps: - | area: gateway`

Context: `services/ai-gateway/src/router/{openai,anthropic,bedrock}.rs` each stop at a line-27 TODO; local models cap the quality available to evolution work.

Scope: implement the three adapters (reqwest/SDK calls, streaming where the dispatch supports it, error mapping into the router's error type, spend estimation feeding IMP-009); keys via env only, absent key = adapter cleanly disabled; per-alias provider selection stays in the existing model map; integration tests behind an env-gated feature (mock server for CI, real-key smoke documented for the Mac).

Acceptance:
- [ ] Mock-server tests green in CI for all three; real-key smoke for at least one provider run by operator and recorded.
- [ ] Missing-key path returns the existing clean "not configured" error.
- [ ] Ledger rows (IMP-009) written with provider cost estimates.

Touches: `services/ai-gateway/src/router/`, tests, `deploy/vps/.env.p0.example`.

---

### IMP-034: chat realtime fanout seam

`refs: R4 | prio: p1 | effort: m | deps: - | area: chat`

Context: the realtime hub is in-process, so chat is single-instance by construction; the seam should exist before scale forces it.

Scope: introduce a `Fanout` trait over the current in-process hub; add a Postgres LISTEN/NOTIFY implementation selected by env (`CHAT_FANOUT=local|pg`); message publish goes through the trait; document the Redis upgrade path. Move no infrastructure yet - P0 keeps `local`.

Acceptance:
- [ ] Two chat instances against one Postgres in the dev stack deliver cross-instance messages and notify events with `pg` fanout.
- [ ] `local` mode byte-identical behavior (existing tests green).
- [ ] Latency delta of `pg` mode measured and recorded.

Touches: `services/chat/src/` (hub, notify), dev stack scripts.

---

### IMP-035: unwrap/expect burn-down and panic removal

`refs: R5, R6 | prio: p1 | effort: m | deps: - | area: quality`

Context: rough grep (tests included) counts unwrap/expect at auth 99, ai-gateway 158, mcp-gateway 113; ~16 panic sites sit in obs-proxy and mcp-gateway paths; a panicking proxy amplifies outages.

Scope: enable `clippy::unwrap_used` + `expect_used` as deny per crate (test modules allowed) starting with mcp-gateway, ai-gateway, auth; convert hits to typed errors through the IMP-031 envelope where user-facing; replace hot-path panics in obs-proxy/mcp-gateway with error returns + counters; one crate per commit.

Acceptance:
- [ ] Three worst crates clippy-clean under the new lints.
- [ ] Zero panic!/unwrap in obs-proxy request paths (grep evidence).
- [ ] No behavior change beyond error responses (tests green).

Touches: `services/{mcp-gateway,ai-gateway,auth,obs-proxy}/`.

---

### IMP-036: finish and property-test audit-chain crate

`refs: R7 | prio: p1 | effort: m | deps: - | area: security`

Context: `shared/cyberos-audit-chain` (~129 LOC) does not yet cover the full AGENTS.md chain spec; it is the trust anchor for memory, eval and BRAIN and should be the best-tested code in the repo.

Scope: implement the complete spec (hash-over record||prev_chain, framing, verification API); proptest suite (arbitrary payload sequences round-trip, any single-byte mutation detected, prefix consistency); fuzz target optional; migrate services that hand-roll chain logic onto the crate.

Acceptance:
- [ ] Property tests green (documented case count); mutation detection proven.
- [ ] At least memory-writer path and eval use the crate (no duplicate chain code left - grep evidence).
- [ ] Public API documented in the crate root.

Touches: `services/shared/cyberos-audit-chain/`, callers.

---

### IMP-037: OpenAPI generation per service

`refs: R8 | prio: p2 | effort: m | deps: - | area: arch`

Context: no machine-readable API specs; typed clients, contract gates and the wiki all want them.

Scope: utoipa annotations starting with auth and chat; spec served at `/v1/openapi.json` (internal); CI artifact uploads the specs; wiki build ingests them; drift gate compares committed spec snapshots to generated ones.

Acceptance:
- [ ] auth + chat specs generated, committed snapshots, drift gate red on a seeded route change.
- [ ] Remaining services tracked as a checklist here.

Touches: `services/{auth,chat}/`, `.github/workflows/`, `website/` or docs pipeline.

---

### IMP-038: extend RLS property gate, cross-tenant probe

`refs: R15 | prio: p1 | effort: m | deps: IMP-016 | area: security`

Context: rls-property-gate.yml exists and 49 migration files carry RLS, but chat/memory tables are not fully covered and isolation is asserted, never continuously probed.

Scope: extend the property gate generators to chat (channels, messages, attachments, mentions) and memory tables; add a scheduled staging probe: synthetic tenant A attempts reads/writes against tenant B across every /v1 surface, expecting 100% denial, alerting otherwise.

Acceptance:
- [ ] Property gate covers the new tables (seeded leak turns it red once, on scratch).
- [ ] Probe runs green on schedule in staging; alert path proven.

Touches: `.github/workflows/rls-property-gate.yml`, `services/{chat,memory}/tests/`, probe script + schedule.

---

### IMP-039: load and soak test suite

`refs: R17 | prio: p2 | effort: m | deps: IMP-016 | area: testing`

Context: no recorded performance baselines; regressions surface in production or not at all.

Scope: k6 scripts for chat ws fanout (connect storm + sustained message rate), message post, auth token issuance; run against staging weekly and on demand; baselines committed under `docs/verification/perf-baselines/`; failure = regression beyond threshold vs baseline.

Acceptance:
- [ ] Three scenarios scripted with committed baselines from two clean runs.
- [ ] Weekly schedule live; regression detection proven once with an artificial slowdown.

Touches: `tools/` or `scripts/perf/`, `.github/workflows/`, docs.

---

### IMP-040: mutation testing pilot on shared crates

`refs: R18 | prio: p2 | effort: s | deps: - | area: testing`

Context: the evolution premise is "gates catch bad changes"; mutation score is the honest measure of that claim.

Scope: run cargo-mutants on `shared/cyberos-audit-chain` and `shared/cyberos-types`; triage survivors (add tests or annotate why acceptable); record scores in `docs/verification/mutation-scores.md`; decide (one paragraph) whether to extend to service crates.

Acceptance:
- [ ] Scores recorded; audit-chain survivors zero or individually justified.
- [ ] Follow-up decision written.

Touches: `docs/verification/`, shared crate tests.

---

### IMP-041: secrets inventory and rotation runbook

`refs: R22 | prio: p1 | effort: s | deps: - | area: security`

Context: secrets exist across GH Actions, the VPS env, Supabase, GHCR, OAuth and signing keys with no single inventory or rotation drill.

Scope: `docs/deploy/secrets-inventory.md` table: secret, system, owner, storage location, rotation procedure, blast radius, last-rotated; rotate-on-leak runbook (ordered steps per secret class); quarterly review reminder wired like other rituals.

Acceptance:
- [ ] Every env var in `deploy/vps/.env.p0.example` and every Actions secret named in workflows appears in the table.
- [ ] One low-risk secret actually rotated as the drill, with the operator, recorded in the ledger.

Touches: `docs/deploy/`.

---

### IMP-042: rate limits beyond login

`refs: R24 | prio: p1 | effort: s | deps: - | area: security`

Context: login has per-IP/per-account limits; message post, uploads (50 MB raw-body route), search and the MCP endpoint have none.

Scope: apply the existing limiter pattern (tower layer) to chat message post, uploads, search, and mcp-gateway entry; sensible defaults documented in env examples; 429 with retry-after through the IMP-031 envelope; keep state behind a small trait so the future shared store is a swap.

Acceptance:
- [ ] Limits enforced with tests (burst then 429, recovery after window).
- [ ] Defaults documented; web client handles 429 gracefully (banner or retry).

Touches: `services/{chat,mcp-gateway}/`, `apps/web/` (429 handling), env examples.

---

### IMP-043: supply-chain hardening

`refs: R25 | prio: p2 | effort: m | deps: - | area: security`

Context: the VPS pulls GHCR images on every push, so image provenance is the deploy trust boundary; Actions are tag-pinned, images unsigned, no SBOM.

Scope: pin all GitHub Actions by commit SHA; generate SBOM (syft) as a build artifact per image; sign images with cosign (keyless via OIDC) and verify signature in the deploy step before rolling.

Acceptance:
- [ ] All workflows SHA-pinned (grep evidence).
- [ ] Deploy refuses an unsigned image (proven once on staging).
- [ ] SBOM artifacts attached to image builds.

Touches: `.github/workflows/*`, `deploy/` (verify step).

---

### IMP-044: automated dependency updates

`refs: R26 | prio: p2 | effort: s | deps: IMP-001 | area: ci`

Context: dependency freshness currently depends on operator attention.

Scope: Renovate (or Dependabot) config grouping minor/patch updates weekly, majors as separate PRs; awh/services gates are the merge condition; cargo, npm, GitHub Actions and Docker base images covered.

Acceptance:
- [ ] First automated PR opened and merged through the normal gate.
- [ ] Grouping rules documented in the config comments.

Touches: `renovate.json` or `.github/dependabot.yml`.

---

### IMP-045: session and token security validation

`refs: R28 | prio: p1 | effort: m | deps: - | area: security`

Context: refresh rotation, reuse detection, revocation latency and break-glass auditability are documented intentions; tests should prove them.

Scope: integration tests in auth for: refresh-token rotation invalidates the predecessor; reuse of a rotated token revokes the family and writes an audit row; revocation propagates within the documented TTL (chat/mcp verifiers reject); break-glass admin actions emit distinct audit rows. Fix whatever the tests reveal; document actual TTLs in `docs/deploy/auth-google-sso-runbook.md`.

Acceptance:
- [ ] All four properties covered by green tests (or fixed then green).
- [ ] TTL table merged into the runbook.

Touches: `services/auth/`, verifier crates/tests, docs.
