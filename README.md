# CyberOS

CyberSkill's AI-native internal operations platform. Three production modules + a documentation/strategy surface + utility folders.

## Repository layout (post-2026-05-18 refactor)

```
cyberos/
├── modules/                ← all production modules
│   ├── cuo/                ← persona-aware orchestration (47 personas + 194 workflows)
│   ├── skill/              ← agent Skills catalog (104 author+audit pairs)
│   ├── memory/             ← the memory protocol + reference implementation
│   └── plugin/             ← packaging + distribution to Claude Code / Cursor / Cowork / Codex CLI (new 2026-05-19)
│
├── docs/                   ← canonical project docs
│   ├── README.md
│   └── feature-requests/   ← FR catalog across 26 domains (~556 FRs)
│
├── tours/                  ← CodeTour walkthroughs (incident response, memory repair, security audit)
├── strategy/               ← strategic positioning + ecosystem playbook
├── website/                ← multi-page documentation site (Liquid Glass, Pagefind)
├── docs/sessions/         ← per-session log archives (new — see docs/sessions/2026-05-18-wave-1-2.md)
├── services/               ← service descriptors
├── pagefind/               ← static search index for website
│
├── .cyberos-memory/        ← memory store (gitignored)
├── .github/                ← CI workflows
├── README.md               ← THIS FILE
├── CHANGELOG.md            ← repo-level umbrella changelog
├── AGENTS.md → modules/memory/AGENTS.md  ← symlink (Layer-1 spec target)
└── CLAUDE.md → modules/memory/AGENTS.md  ← symlink (same target)
```

### What changed

**Before (legacy flat):** modules sat at repo root alongside utility folders; each module had a `docs/` subfolder with 5–11 .md files; `docs/prd/`, `docs/srs/`, `docs/tours/` mixed product-spec + operational tours.

**After (modules/ refactor):** all three production modules collected under `modules/`; each module has a single comprehensive `README.md` at module root (with protocol artefacts as siblings — `AGENTS.md`, `*.schema.json`, `*.invariants.yaml`); operational tours promoted to repo-root `tours/`; outdated `docs/prd/` + `docs/srs/` removed (frozen 2026-05-15, superseded by feature-requests/).

Isolation is preserved — each module is still self-contained (own `pyproject.toml` / `Cargo.toml` / `README.md` / `AGENTS.md` / `CHANGELOG.md`) and can be cloned independently. The `modules/` parent is just a tidy collection.

### Repo layout doctrine — `modules/` vs `services/`

Two top-level homes for code, with **different semantics**:

| Folder | What it is | Why it's separate |
|---|---|---|
| **`modules/`** | Catalog + protocol spec + reference Python implementation. Each module is *the deliverable itself* (e.g. `modules/skill/` IS the 104-pair catalog; the catalog is what ships). | Module bundles travel as one folder. Spec + reference impl + CHANGELOG + AGENTS.md all live together so a `cp -r modules/<name>/ <other-repo>/` keeps the unit intact. |
| **`services/`** | Rust production binaries — the boxed runtime that consumes the protocols defined in `modules/`. | Production services need a Cargo workspace for `Cargo.lock` cohesion + shared crates (`shared/cyberos-cli-exit`, `shared/cyberos-types`). Each binary still ships as its own Docker image; workspace coupling is build-time only, not deploy-time. |

Concretely:

- `modules/memory/` = the memory protocol spec + Python reference writer (`AGENTS.md` + `memory.schema.json` + `memory.invariants.yaml` + `cyberos/` package). `services/memory/` = the Rust production memory service that implements that protocol.
- `modules/skill/` = the SKILL catalog (markdown bundles, RUBRIC, AUTHORING discipline). The future Rust skill-broker will live in `services/skill-broker/` when FR-SKILL-103 ships.
- `modules/cuo/` = the CUO Python supervisor; no Rust port planned (it stays Python).
- `services/auth/` = the Rust AUTH service; **has no `modules/auth/`** because AUTH has no spec/Python-reference-impl split. Its spec lives in `docs/feature-requests/auth/` directly.

**The split is intentional and stable.** Both layers preserve module isolation: `modules/<name>/` clones cleanly because it's a folder; `services/<name>/` clones cleanly because Cargo lets you build a single workspace member with `cargo build -p <name>`. The workspace's shared `Cargo.lock` is a build-time cache, not a coupling — production deploys ship each binary independently.

## Quick start

```bash
# Memory module — the memory
cd modules/memory
pip install -e .
cyberos --store ../../.cyberos-memory doctor          # → READY ✓ 15/15 invariants

# CUO module — persona-aware orchestration
cd ../cuo
pip install -e .
cyberos-cuo list-personas                              # → 47 active + 1 extinct
cyberos-cuo route "Architect a new payment system"     # → chief-technology-officer/architect-new-system
cyberos-cuo execute chief-technology-officer/adr-quick-capture \
    --output-dir /tmp/run-1 \
    --invoker mock \
    --memory-emit \
    --actor stephen

# Skill module — agentic Skills catalog
cd ../skill
ls -1 | grep -E -- '-author$' | wc -l                  # → 104 author skills
# Rust host (when activated):
# cargo run -p cyberos-skill-cli -- list
```

Each module's `README.md` has full install / audit / fine-tune / deploy instructions.

## Modules

| Module | Role | Status | Read |
|---|---|---|---|
| [`modules/memory/`](modules/memory/) | memory — append-only audit-chained personal memory store | 255 green tests; all 12 audit proposals shipped | [README](modules/memory/README.md) · [AGENTS](modules/memory/AGENTS.md) |
| [`modules/skill/`](modules/skill/) | Agent Skills catalog + Rust host + Bun toolchain | 104 author+audit pairs (208 bundles); 108 contracts; catalog-complete post-Session H | [README](modules/skill/README.md) |
| [`modules/cuo/`](modules/cuo/) | Persona-aware orchestration (Chief Universal Officer) | 47 personas + 194 workflows; supervisor Phase 1–3 shipped (21/22 tests pass) | [README](modules/cuo/README.md) |
| [`modules/plugin/`](modules/plugin/) | Packaging + distribution — exposes CUO + memory + SKILL as installable `.plugin` artefacts for Claude Code / Cursor / Cowork / Codex CLI | scaffold + 8 FRs at 10/10 (FR-PLUGIN-001..008); runtime at `services/plugin-host/` planned | [README](modules/plugin/README.md) · [INTEROP](modules/plugin/INTEROP.md) |

## Status

| Layer | Status |
|---|---|
| memory protocol (Layer-1) + reference implementation | shipped — 255 tests, 30 CLI commands, P2 Stage 3 |
| SKILL catalog | 104 pairs / 208 bundles / 108 contracts; zero `planned:` gaps after Session H |
| CUO catalog | 47 active personas + 194 workflows; zero gaps after Session N |
| CUO supervisor (Python) | Phase 1 (catalog + router + dry-run), Phase 2 (Invoker + execute_chain), Phase 3 (LLMInvoker + memory emission) — all shipped 2026-05-18 |
| Docs site (`website/`) | 32 pages, 226 diagrams, 341 FRs, 100 NFRs, Pagefind search |
| Design system (sibling repo `../design-system/`) | Liquid Glass v1.1.0 — L3 enterprise tier |

**Roadmap:** CUO Phase 4 (5 special-case workflow handlers); CUO depth additions (per-persona workflow expansion 4 → 8–12); PLUGIN module runtime build-out (`services/plugin-host/` per FR-PLUGIN-001..008 — manifest packer, MCP bridge, OAuth-PKCE, memory audit emission, multi-runtime adapters, marketplace publish); 19 remaining modules (AUTH, AI, MCP, OBS, CHAT, EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN) — scaffolded in docs, not built.

## Sibling projects (separate git repos)

| Sibling | Where | Role |
|---|---|---|
| **design-system** | `../design-system/` | CyberSkill brand + design doctrine. Liquid Glass v1.1.0 |
| **landing-page** | `../landing-page/` | `cyberskill.world` landing page source |
| **sale-noti** | `../sale-noti/` | Sales notification subsystem |
| **tamagochi** | `../tamagochi/` | Virtual-pet game + PetOS B2B (53 FRs at 10/10) |

Siblings stay separate because they have their own git history, release cadence, and audit cycles.

## Production Deploy — Wave 1: MEMORY + AUTH + SKILL

This section is the canonical runbook for taking the first three modules to production. Run in order; each phase is gated by the previous phase's smoke test.

### §1 — Prerequisites

| Component | Version | Why |
|---|---|---|
| PostgreSQL | 16.x | base store for AUTH (RLS) + memory (Layer-2 ingest + audit chain) |
| PostgreSQL extensions | `pgvector` 0.7+, `apache_age` 1.5+ | embeddings + graph (memory Layer-2) |
| Redis | 7.x | rate-limit + session cache (AUTH) + event-bus draft (will become NATS at scale) |
| Rust toolchain | 1.88 stable | matches `services/Cargo.toml` `rust-version` (bumped 1.83→1.88 on 2026-05-19 — webauthn-rs/time/icu/base64urlsafedata transitively require ≥1.86/1.88) |
| Python | 3.10+ | `modules/memory/` reference impl + `modules/cuo/` supervisor + sidecars |
| sqlx-cli | 0.8 | `cargo install sqlx-cli --no-default-features --features rustls,postgres` |
| AWS account | — | Fargate (services) + ECR (images) + RDS (Postgres) + ElastiCache (Redis) |
| Domain + DNS | — | `cyberos.cyberskill.world` (wiki) · `auth.cyberskill.world` (AUTH endpoint) · `memory.cyberskill.world` (memory endpoint) |

### §2 — Bootstrap order (deploy roadmap)

Per [`docs/feature-requests/BACKLOG.md` §0.6](docs/feature-requests/BACKLOG.md), the user-locked production order is:

```
  ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────┐    ┌──────────────┐
  │  MEMORY  │ ─▶ │  AUTH  │ ─▶ │  CHAT  │ ─▶ │ PROJECT │ ─▶ │  CUO + SKILL │
  │ (memory)  │    │        │    │        │    │ (PROJ)  │    │              │
  └──────────┘    └────────┘    └────────┘    └─────────┘    └──────────────┘
        wave 1         wave 2         wave 3         wave 4         wave 5
```

Wave 1 (this runbook) covers MEMORY + AUTH + SKILL. The next sections walk each.

### §3 — MEMORY deploy

**What ships:** Layer-1 protocol (file-only memory per [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md)) PLUS Layer-2 Rust service ([`services/memory/`](services/memory/)) for ingest + AGE graph + search REST.

```bash
# 3.1 — Build the Rust memory service binary
cd services
cargo build --release -p cyberos-memory
ls -lh target/release/cyberos-memory   # → single statically-linked binary

# 3.2 — Apply migrations to the production database
export DATABASE_URL="postgres://cyberos_admin:$PG_PASSWORD@prod-postgres.cyberskill.world:5432/cyberos"
sqlx migrate run --source services/memory/migrations

# 3.3 — Seed the AGE graph
psql "$DATABASE_URL" -f services/memory/seed/age_init.sql

# 3.4 — Initialise the local Layer-1 memory (the protocol root)
cd ../modules/memory
pip install -e .
cyberos --store /var/lib/cyberos-memory doctor       # expect: READY ✓ 15/15 invariants
cyberos --store /var/lib/cyberos-memory bootstrap    # writes HEAD=00 + audit segment + index

# 3.5 — Boot the memory HTTP server
cd ../../services
./target/release/cyberos-memory serve \
    --listen 0.0.0.0:8081 \
    --database-url "$DATABASE_URL" \
    --layer-1-store /var/lib/cyberos-memory \
    --otel-endpoint http://otel-collector.cyberskill.world:4317

# 3.6 — Smoke tests
curl -fsS http://localhost:8081/healthz                          # → {"status":"ready"}
curl -fsS http://localhost:8081/v1/audit/chain | jq '.head_seq'  # → 0 (or current head)
cyberos --store /var/lib/cyberos-memory invariants               # → 15/15 PASS

# 3.7 — Deploy to Fargate (per docs/feature-requests/memory/FR-MEMORY-104)
aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com
docker build -t cyberos-memory:$(git rev-parse --short HEAD) services/memory/
docker tag cyberos-memory:$(git rev-parse --short HEAD) <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-memory:latest
docker push <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-memory:latest
aws ecs update-service --cluster cyberos-prod --service memory --force-new-deployment
```

**Health endpoints:** `/healthz` (ready/not-ready), `/v1/audit/chain` (current head + tip hash), `/metrics` (Prometheus exposition).

**Rollback:** `aws ecs update-service --cluster cyberos-prod --service memory --task-definition cyberos-memory:N-1 --force-new-deployment` (replace `N-1` with the previous task-def revision).

**Observability:** Grafana dashboard `cyberos-memory` (panels for HEAD lag, audit-row rate, AGE query p95, embedding sidecar latency). Datadog tag `service:cyberos-memory`.

**Secrets:** Postgres password lives in AWS Secrets Manager at `arn:aws:secretsmanager:ap-southeast-1:<acct>:secret:cyberos/prod/memory/db-password`. OTel collector token at `…/otel-token`.

### §4 — AUTH deploy

**What ships:** Rust AUTH service ([`services/auth/`](services/auth/)) with RLS, JWT/JWKS issuance, MFA (TOTP/WebAuthn/Passkey), SAML/OIDC SSO, 22-role RBAC catalogue.

```bash
# 4.1 — Build
cd services
cargo build --release -p cyberos-auth

# 4.2 — Apply migrations (20 ordered SQL files; idempotent post-2026-05-19 fix)
export DATABASE_URL="postgres://cyberos_admin:$PG_PASSWORD@prod-postgres.cyberskill.world:5432/cyberos"
sqlx migrate run --source services/auth/migrations

# Sanity check: migrations leave database in known state
psql "$DATABASE_URL" -c "SELECT version, description FROM _sqlx_migrations ORDER BY version;"
# Expect: 20 rows (0001_tenants through 0020_cyberos_ops_role)

# 4.3 — Generate signing keys (one-time per environment)
./target/release/cyberos-auth keygen \
    --algorithm RS256 \
    --kid auth-prod-2026-05-1 \
    --output /var/lib/cyberos/keys/auth-prod-2026-05-1.jwk
# Result: JWK is loaded by AUTH at boot; PUBLIC half is served at /.well-known/jwks.json

# 4.4 — Bootstrap the root tenant + first admin subject
./target/release/cyberos-auth bootstrap \
    --root-tenant-name "CyberSkill" \
    --admin-email stephen@cyberskill.world \
    --otp-secret-out /tmp/admin-totp.txt
# Save the TOTP seed to a password manager; the file is wiped on first reboot.

# 4.5 — Boot AUTH HTTP server
./target/release/cyberos-auth serve \
    --listen 0.0.0.0:8080 \
    --database-url "$DATABASE_URL" \
    --redis-url "redis://prod-redis.cyberskill.world:6379/0" \
    --jwk-path /var/lib/cyberos/keys/auth-prod-2026-05-1.jwk \
    --memory-base-url http://memory.cyberskill.world \
    --otel-endpoint http://otel-collector.cyberskill.world:4317

# 4.6 — Smoke tests
curl -fsS http://localhost:8080/healthz                                      # → {"status":"ready"}
curl -fsS http://localhost:8080/.well-known/jwks.json | jq '.keys | length' # → 1 (the just-generated key)
curl -fsS http://localhost:8080/.well-known/openid-configuration | jq .     # → discovery doc

# 4.7 — Authenticate the bootstrap admin (TOTP flow)
TOTP=$(oathtool --totp --base32 "$(cat /tmp/admin-totp.txt)")
TOKEN=$(curl -fsS -X POST http://localhost:8080/v1/auth/token \
    -d "grant_type=password&username=stephen@cyberskill.world&password=$BOOTSTRAP_PASSWORD&totp=$TOTP" | jq -r .access_token)
curl -fsS http://localhost:8080/v1/me -H "Authorization: Bearer $TOKEN"     # → {"subject_id":"...", "tenant_id":"..."}

# 4.8 — Deploy to Fargate
docker build -t cyberos-auth:$(git rev-parse --short HEAD) services/auth/
docker push <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-auth:latest
aws ecs update-service --cluster cyberos-prod --service auth --force-new-deployment
```

**Health endpoints:** `/healthz`, `/.well-known/jwks.json`, `/.well-known/openid-configuration`, `/metrics`.

**Rollback:** ECS task-def rollback (same pattern as memory). **Critical:** rolling back ACROSS a migration requires `sqlx migrate revert` against the database FIRST — migrations are forward-only by default.

**Observability:** Grafana dashboard `cyberos-auth` (login success/failure rate, MFA challenge rate, JWT issuance latency, RLS rejection rate per tenant).

**Secrets:** Postgres password + Redis password + JWK private key all in AWS Secrets Manager. Apple/Google/Microsoft OIDC client secrets (when SSO ships per FR-AUTH-101+) also there.

### §5 — SKILL deploy

**What ships today:** the catalog itself (`modules/skill/`) — 104 author+audit pairs + 108 contracts. **The Rust skill-broker (`services/skill-broker/`) is not yet built** — that's FR-SKILL-103, deferred per [`modules/skill/FR_111_115_COMPLETION_PLAN.md`](modules/skill/FR_111_115_COMPLETION_PLAN.md) Session 3.

For Wave 1 production: the catalog is consumed by the **CUO supervisor** (`modules/cuo/`, Python) which routes user queries to the right skill. No skill runtime exists yet; skills are interpreted by the LangGraph supervisor.

```bash
# 5.1 — Validate the catalog
cd modules/skill
ls -1 | grep -E -- '-author$' | wc -l        # → 104 (expected)
ls -1 | grep -E -- '-audit$' | wc -l         # → 104 (expected)

# 5.2 — Run the new SKB-* validators (post-FR-SKILL-111..115)
cd ../cuo
PYTHONPATH=. python3 -m cuo.trigger_tests --catalog ../skill        # routes test
PYTHONPATH=. python3 -m cuo.baseline --catalog ../skill             # BASELINE.md check
PYTHONPATH=. python3 -m cuo.placeholder_check --catalog ../skill    # SKB-030 (FR-115)
# Today (pre-FR-115 sweep): placeholder_check exits 1 because 163 skills
# still carry stale <placeholder> syntax. Sweep ETA: 8-10h per FR-SKILL-115.

# 5.3 — Verify chain integrity (every skill's contract dependencies resolve)
PYTHONPATH=. python3 -m cuo.cli validate-chains
# → ✓ 104 chains pass; ✗ 0 broken

# 5.4 — Bundle for distribution (OCI registry per FR-SKILL-102, when shipped)
# Today: catalog is shipped as the cyberos repo itself. Future: each skill
# becomes a signed `.skill` bundle in an OCI registry; CUO supervisor pulls
# on demand.

# 5.5 — Smoke: end-to-end chain
python3 -c "
from cuo.core.router import route
decision = route('Turn this PRD into a backlog of FRs')
print(f'Routes to: {decision.persona_slug}/{decision.workflow_slug}')
print(f'Confidence: {decision.confidence:.2f}')
"
# Expected: chief-product-officer/<workflow that contains feature-request-author>
```

**Health endpoints:** none today (catalog is markdown, not a service). When `services/skill-broker/` ships: `/healthz`, `/v1/skills` (list), `/v1/skills/<id>/validate` (per-skill validation).

**Rollback:** `git revert` on the catalog. No DB schema changes to worry about today (FR-SKILL-102 OCI-registry deploy adds them later).

**Observability:** today routed via CUO supervisor → CUO emits OTel spans `cuo.classify_act` + `cuo.execute_chain`. Future broker will emit `skill.broker.validate`, `skill.broker.invoke`, etc.

### §6 — End-to-end smoke (MEMORY + AUTH + SKILL all live)

After §3-§5 succeed independently, prove the three modules talk to each other:

```bash
# Get an AUTH token (uses MFA per §4.7)
TOKEN=$(curl -fsS -X POST http://auth.cyberskill.world/v1/auth/token \
    -d "grant_type=password&username=stephen@cyberskill.world&password=$PW&totp=$(oathtool --totp --base32 $TOTP_SECRET)" | jq -r .access_token)

# Write a memory audit row via memory service (AUTH-stamped)
curl -fsS -X POST http://memory.cyberskill.world/v1/audit/append \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"kind":"put","path":"memories/projects/cyberos/smoke-test.md","body":"hello"}' \
    | jq '.audit_id'

# Route a query through CUO that touches both AUTH (for identity) + memory (for memory read)
python3 -m cuo.core.supervisor execute \
    --actor "human:stephen-cheng" \
    --query "Audit FR-AUTH-003 for completeness" \
    --memory-emit \
    --output-dir /tmp/e2e-smoke
ls /tmp/e2e-smoke/                                     # → step output JSONs
curl -fsS http://memory.cyberskill.world/v1/audit/chain | jq '.head_seq'  # incremented by N
```

**Pass criteria:** AUTH issues token, memory accepts authenticated audit-append, CUO routes correctly, memory chain head advances. If any step fails, the rollback for that service per §3.6/§4.8 fires.

### §7 — Day-2 operations

| Concern | Where it lives |
|---|---|
| Health monitoring | Grafana dashboards `cyberos-memory` + `cyberos-auth` (live via [Datadog](https://app.datadoghq.eu/) when configured) |
| Alerting | PagerDuty service `cyberos-prod-on-call`; routes via Alertmanager per FR-OBS-007 |
| Backup (Postgres) | RDS automated snapshots — 7-day retention. Cross-region replication to ap-southeast-2 (per FR-AUTH-005 compliance) |
| Backup (memory local store) | `cyberos export <tenant_id> --target s3://cyberos-backups/<env>/<date>.zip` runs nightly via Lambda |
| Audit ledger integrity | Tamper detector per AGENTS.md §10 + SRS §10.4.6 runs continuously; alerts on chain breaks |
| Cost | AWS Cost Explorer tag `Project=cyberos` + budget alarm at 80% of $535/mo envelope (per `docs/feature-requests/BACKLOG.md` §architecture/tech-stack) |

## Documentation

- **Multi-page interactive docs site**: open `website/docs/index.html`
- **SDP (Software Development Process)**: `modules/cuo/README.md` (14 stages, normative)
- **C-Suite Reference**: `modules/cuo/README.md` (48-persona atlas, normative)
- **Feature requests**: `docs/feature-requests/` (~556 FRs across 26 domains; see `BACKLOG.md`)
- **Operational tours**: `tours/` (CodeTour walkthroughs — open with VS Code CodeTour extension)
- **Per-module READMEs**: each `modules/*/README.md` is comprehensive (install / audit / fine-tune / deploy)
- **Strategic playbook**: `strategy/`

## License

MIT throughout (was Apache 2.0 in earlier docs — modules ship MIT per their `pyproject.toml` / `Cargo.toml`).

## Maintainer

CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam.
Founder: Stephen Cheng (Trịnh Thái Anh) · [info@cyberskill.world](mailto:info@cyberskill.world)


# CyberOS — Strategic Push Forward

*Where CyberOS sits today, where world-class ecosystems are heading, and how to turn CyberOS into the internal-then-commercial Ecosystem-as-a-Service moat for CyberSkill.*

---

## 1 — Where CyberOS sits today (May 2026)

Three modules shipped, twenty more in plan. The architectural foundation is real:

- **Memory module** — local-first, audit-chained, cryptographically verifiable personal memory store. 245 tests green.
- **Skill module** — Anthropic Agent Skills open-standard compliant. 20 SKILL.md bundles indexed, 6 Vietnamese-market skills shipped, Rust+Wasmtime+Bun toolchain. All 7 audit phases done.
- **CUO module** — rule-based router. 15/15 routing fixtures + 15/15 pytest tests. Phase 1 (rule-based) shipped, Phases 2–4 (LLM, multi-skill chains, persona switching) designed.
- **Documentation site** — 31 pages, 226 Mermaid diagrams, 341 FRs, 100 NFRs, 199 glossary terms, 42 risks. Multi-page Path C.

The remaining 19 modules (AUTH, AI Gateway, MCP Gateway, OBS, CHAT, EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN) are scaffolded in the docs but not built.

## 2 — World-class ecosystem landscape (2026 snapshot)

Here is what CyberOS is competing with, organized by where they sit on the **closed ↔ open** + **horizontal ↔ vertical** axes:

### The horizontal-closed giants

| Player | Revenue | What they own | Their agentic move |
|---|---|---|---|
| **Microsoft 365** | ~$80B/yr | Office + Teams + SharePoint + Outlook + Power Platform | Copilot baked into every product; closed agent layer |
| **Google Workspace** | ~$30B/yr | Gmail + Docs + Drive + Meet + Calendar | Gemini baked in; closed agent layer |
| **Salesforce** | ~$35B/yr | Sales Cloud + Service Cloud + Marketing Cloud + Slack | Einstein/Agentforce; closed agent layer |
| **Atlassian** | ~$5B/yr | Jira + Confluence + Bitbucket + Loom + Browser Co (Arc/Dia, Oct 2025 $610M acq) | Rovo; building agentic dev platform |

These are entrenched. CyberOS cannot win horizontally against any of them. The only viable plays are: (a) interop via MCP, (b) regional vertical wedge (Vietnam), (c) net-new categories (agentic-native ops, where they're playing catch-up).

### The vertical-open challengers

| Player | Bet | Why they matter to CyberOS |
|---|---|---|
| **Notion** | Knowledge ops + AI ($10B val) | Closest analog for KB + ad-hoc workflow; lacks agentic substrate |
| **Linear** | PM done right (~$1B val) | Performance bar for PROJ; modern stack reference |
| **Plane.so** | Open-source Linear | Open-source playbook |
| **Retool** | Internal tools builder | Vertical CyberOS could absorb |
| **HuggingFace** | Open AI hub | The "agent registry" CyberOS skills could one day publish to |

### The agent-spec wars

| Player | Status | CyberOS posture |
|---|---|---|
| **Anthropic Agent Skills** (Dec 2025 spec release) | Open standard; 26+ clients adopting | **CyberOS is a citizen.** Skills compatible with Claude, Codex, Cursor, Goose, Amp, etc. |
| **OpenAI Apps SDK + ChatGPT Custom GPTs** | Proprietary; ecosystem-locked | OpenAI itself adopting SKILL.md format inside Codex CLI (Dec 2025) — the standard won |
| **MCP (Model Context Protocol)** | LF-donated Dec 2025; 10K+ public servers | **CyberOS speaks MCP natively.** MCP Gateway is a P0 module |
| **Sigstore Rekor / transparency logs** | Open; growing | CyberOS audit chain anchors here long-term |

### The ecosystem-as-a-service playbooks (2024–2026 lessons)

| Platform | Marketplace size | Key lesson for CyberOS |
|---|---|---|
| **Salesforce AppExchange** | ~7,000 apps | Vertical packs unlock enterprise sales |
| **Microsoft AppSource** | ~50,000 apps | Compliance certifications drive adoption |
| **Shopify App Store** | ~10,000 apps | Revenue share + dev-friendly tooling matters |
| **Atlassian Marketplace** | ~5,000 apps | Forge platform took 5 years; trust + sandbox is hard |
| **Notion Templates Gallery** | ~30,000 templates | Free templates = top-of-funnel for paid product |
| **agentskills.io directory** | ~500 skills (still scaling) | **CyberOS publishes Vietnamese pack here = early-mover advantage** |

### What's new in 2026 worth tracking

- **Atlassian Rovo** — agentic teammates baked across Jira/Confluence/Bitbucket. Threat to PROJ + KB modules.
- **Sierra (Bret Taylor)** — vertical AI agents for customer service. Threat to CHAT-as-support.
- **Lindy AI / Cognosys** — agent builders for ops automation. Indirect competitors.
- **Devin AI (Cognition)** — autonomous coding agent. Adjacent — could integrate via MCP.
- **Browser Co (Dia, Arc 2.0)** — Atlassian-owned, becoming a browser-native agent OS. Direct threat to the desktop-shell layer.
- **Anthropic Claude Code + MCP** — the model of agentic-CLI-meets-codebase. Worth deep study; CyberOS CLI surface should match its DX.

## 3 — Specific recommendations to push CyberOS-docs deeper

The docs site at `cyberos/website/docs/` is now structurally complete (31 pages, 22 modules, FR/NFR catalogs). Here is the **next-tier feature list** to turn it from "comprehensive reference" into "the canonical CyberOS wiki + roadmap tracker".

### Tier 1 — Critical (next 1–2 sessions)

| # | Feature | Why | Implementation |
|---|---|---|---|
| 1 | **Live module-status dashboard** | Each module shows real-time pass/fail status, test count, last-deploy date | Per-module status JSON file, Alpine fetches + renders |
| 2 | **Site-wide search** (Lunr.js or Pagefind) | 30,400 lines of docs — Cmd+F per page doesn't scale | Pagefind (Rust, fast, no backend); build at deploy time |
| 3 | **Decision log / ADRs** | Track every architectural decision with date + author + rationale | New `reference/decisions.html` with chronological list + per-ADR pages |
| 4 | **Public changelog** with RSS | Every module ships changes; one place to subscribe | Aggregate per-module CHANGELOGs into `reference/changelog.html` + `feed.xml` |
| 5 | **Cross-link tightening (per-FR anchors)** | Currently FR refs point at module sections; should anchor per-FR | Add `id="FR-{MOD}-{NNN}"` to each FR card in `fr-catalog.html` |

### Tier 2 — Substantial (3–5 sessions)

| # | Feature | Why |
|---|---|---|
| 6 | **Interactive dependency graph** | D3 force-directed graph of all 22 modules + their relationships; click a node to drill in |
| 7 | **API playground** (Stripe-docs-style) | For modules with GraphQL/MCP surface, try requests inline with sample data |
| 8 | **Comparison matrices** | "CyberOS PROJ vs Linear vs Jira" feature-by-feature; ditto for every module |
| 9 | **Migration guides** | "From Slack to CyberOS CHAT" / "From Notion to CyberOS KB" / "From Jira to PROJ" |
| 10 | **Pricing calculator** | Interactive: select modules + seats + tenant size → estimated monthly cost. Includes infra cost ($380/mo internal / $2.2k for 50-tenant) |
| 11 | **Customer stories / case studies** | Start with CyberSkill-itself; add Vietnamese partner case studies as deals close |
| 12 | **Roadmap kanban view** | Drag-drop FR cards across "Backlog / This Sprint / In Progress / Done"; per-phase swimlanes |
| 13 | **SLA/SLO dashboards** | Per-module reliability targets + actuals (when production data exists) |

### Tier 3 — Long-horizon (multi-month)

| # | Feature | Why |
|---|---|---|
| 14 | **Vietnamese full-language version** | Every page bilingual; `Be Vietnam Pro` already in the font stack |
| 15 | **Versioned docs** | v0.1 / v0.2 / v1.0; diff view; deprecated banners |
| 16 | **i18n for English variants** | US English vs International English (date formats, currency) |
| 17 | **Embeddable widgets** | "Embed CyberOS module status on your blog/dashboard" via iframe |
| 18 | **PDF generation** | One-click PDF export per page or for whole site (use Paged.js) |
| 19 | **Video walkthroughs** | One 60-second video per module embedded above the fold |
| 20 | **Interactive tutorials** | Web-based "Try CyberOS" with a sandboxed BRAIN + skills (use Pyodide for the Python tier) |

### Tier 4 — Wiki-style depth

Atlassian-Confluence-class depth requires:

| # | Feature |
|---|---|
| 21 | **Hierarchical TOCs per module** — drill several levels deep without losing context |
| 22 | **Inline annotations / comments** — like Google Docs sidebars; reviewers can leave notes |
| 23 | **"Edit this page" links** — every page links to its source markdown for direct edits |
| 24 | **AI Q&A over the whole docs corpus** — like Notion AI; uses BGE-M3 over the entire site |
| 25 | **Notebook-style live examples** — embed runnable Python/JS in skill explanations |
| 26 | **Glossary popovers** — hover any glossary term anywhere in the docs to see its definition |
| 27 | **Cross-page breadcrumb trails** — show how related pages link together |
| 28 | **Reading time + difficulty markers** — "This page: 8 min read · Intermediate" |

## 4 — Ecosystem-as-a-Service strategy

The big idea: **CyberOS isn't just a product; it's the substrate for OTHER products.** Same playbook as Salesforce (not just CRM, but a platform for ISVs), Notion (not just notes, but a platform for templates), Shopify (not just storefronts, but a platform for apps).

Five levels of ecosystem productization, in order:

### Level 0 — Internal (today)

CyberSkill uses CyberOS for everything internally. Dogfooding. Bet 4 from the PRD. Status: **shipped for memory/skill/cuo**, in progress for the rest.

### Level 1 — Open-source distribution (next 6 months)

CyberOS is on GitHub. Anyone can clone, run their own instance, contribute modules. Tactics:

- Apache 2.0 license throughout
- One-command install (`curl …/install.sh | bash`)
- Public agentskills.io presence for the cyberskill-vn collection
- Public docs site (cyberos/website/docs/ deployed to docs.cyberskill.world)
- Open RFC process for protocol changes
- Public weekly office hours / community calls
- Public ROADMAP.md updated weekly

This is the credibility play. Without OSS distribution, no developer takes CyberOS seriously as a platform.

### Level 2 — Hosted SaaS (months 6–18)

CyberSkill runs CyberOS for paying tenants. Each tenant gets isolated infra (tenant_id RLS Postgres, tenant-scoped NATS, tenant S3 prefix). Pricing tiers:

- **Free** — 5 seats, 100 MB BRAIN, 50K AI tokens/mo, community support
- **Pro** ($29/seat/mo) — unlimited seats, 5 GB BRAIN, 5M tokens/mo, email support, all P0+P1 modules
- **Enterprise** ($99/seat/mo + setup) — bring-your-own-LLM-keys, dedicated tenant, SSO, audit log retention, SLA, all 22 modules including ESOP+DOC

This unlocks ARR. Vietnam-market launch first (HCMC tech scene, then HN), then SEA expansion (Singapore, Indonesia, Thailand, Philippines).

### Level 3 — Marketplace (months 12–24)

3rd parties publish skills + module integrations to the CyberSkill marketplace. Tactics:

- Skill publish workflow: `cyberos-skill publish` pushes to `agentskills.io/cyberskill/<author>/<skill>`
- Revenue share (70% to skill author, 30% to CyberSkill) for paid skills
- Marketplace UI in the docs site at `marketplace.cyberskill.world`
- Curated "Vetted by CyberSkill" badge for security-reviewed skills
- "Built on CyberOS" co-marketing

The marketplace converts CyberOS from a product into a **platform**. This is what Salesforce did in 2005 with AppExchange — and 21 years later it's still the moat.

### Level 4 — Vertical packs (months 18–36)

Beyond Vietnamese skills, build complete vertical packs:

- **cyberskill-vn** (already shipping) — VN compliance, e-invoice, banking, identity, legal
- **cyberskill-sg** — Singapore tax (IRAS), local bank APIs, PDPA, ACRA filings
- **cyberskill-id** — Indonesia (BPJS, NPWP, OJK compliance)
- **cyberskill-th** — Thailand (RD VAT, PDPA-Thailand)
- **cyberskill-eu** — EU compliance (GDPR-native, eIDAS DOC integration, EU AI Act helpers)
- **cyberskill-us** — US compliance (SOC 2 reports, HIPAA helpers, state tax)
- **cyberskill-hr** — HR-specific (US W-2, EU contracts, VN BHXH)
- **cyberskill-legal** — Legal practice (contract review, litigation tracking, billable hours)
- **cyberskill-accounting** — Accounting (GAAP/IFRS reports, audit trail, year-end close)

Each vertical pack is a saleable product on top of the base CyberOS. Margins: 70%+ since the base is open-source.

### Level 5 — Ecosystem-as-a-Service (months 24+)

The endgame: **sell the CyberOS framework itself to enterprises** who want their own branded internal-ops platform.

- "Acme Corp Operating System, powered by CyberOS"
- Enterprise pays CyberSkill to deploy, customize, and operate a private-cloud or on-prem CyberOS instance
- White-label everything (logo, colors via design system, custom modules)
- ISVs publish into the enterprise's private marketplace, not the public one
- Margins: 80%+ on multi-year contracts; recurring services revenue stacks

This is the Confluent / Databricks / Snowflake playbook applied to agentic ops. CyberSkill becomes the consultancy AND the platform — exactly the position your 2020 charter aimed for.

### Comparative positioning

| Dimension | Microsoft 365 / Google Workspace | Salesforce | Notion | Linear | **CyberOS** |
|---|---|---|---|---|---|
| Horizontal vs vertical | Horizontal | Vertical (CRM-first) | Horizontal (KB) | Vertical (PM) | **Horizontal (ops) + vertical packs** |
| Closed vs open | Closed | Semi-closed (AppExchange) | Closed | Closed | **Open standard + Apache 2.0 base** |
| AI-native | Bolted on (Copilot) | Bolted on (Einstein) | Bolted on (Notion AI) | Native-ish | **Agentic substrate from day one** |
| Regional moat | None | Localized regions | None | None | **Vietnamese-first, then SEA** |
| Marketplace | Yes (50K apps) | Yes (7K apps) | Yes (30K templates) | No | **Planned (agentskills.io citizen + own marketplace)** |
| Open audit chain | No | No | No | No | **Yes (MMR + STH on every action)** |

CyberOS's defensible position: **the only platform that's agentic-native + open-standard + audit-chained + regionally-localized**. None of the giants have all four.

## 5 — Concrete next-session priorities

Three actionable next steps, in order:

### Session 1 — Push the docs site to public-ready

1. Wire site-wide search (Pagefind — 30 minutes; Rust-fast; builds at deploy time)
2. Add per-FR anchors in fr-catalog (cross-link tightening)
3. Add decision log + RSS-able changelog page
4. Polish remaining Tailwind utility colors to match Umber/Ochre tokens
5. Deploy to `cyberskill.world/docs` (Cloudflare Pages or GitHub Pages, either works)
6. Announce on LinkedIn + Vietnam dev communities

### Session 2 — Begin the AUTH module

AUTH is the keystone for everything else. Building it unlocks: AI Gateway, MCP Gateway, OBS, every P1 module. The docs already specify the design. Build it:

- Postgres-backed identity service (Rust or Python, your call)
- JWT RS256 with tenant_id claim
- OAuth 2.1 + RFC 7636 PKCE
- WebAuthn L3 for MFA
- RBAC with role catalogue per PRD §8.6.1
- Audit log integration (every auth decision → memory audit chain)

### Session 3 — Comparison matrices + migration guides

The fastest demand-generation play:
- "CyberOS PROJ vs Linear" — feature table + migration script
- "CyberOS CHAT vs Slack" — feature table + import tool
- "CyberOS KB vs Notion" — feature table + import tool

These pages bring search traffic (everyone Googling "Linear vs alternative", "Notion alternative", etc. lands on CyberOS docs).

## 6 — What success looks like

12-month markers if this strategy works:

- **3 months**: docs site live publicly; agentskills.io listing live; LinkedIn/Vietnam tech community awareness; first 100 OSS users
- **6 months**: AUTH + AI Gateway + MCP Gateway + OBS + CHAT shipped; 10 OSS contributors; 1,000+ docs site weekly visitors
- **9 months**: PROJ + TIME + CRM + KB + HR shipped; SaaS tier launched (Free + Pro); first 50 paying tenants in Vietnam
- **12 months**: REW + LEARN + EMAIL shipped; 500+ paying tenants; ARR ≥ $500K; first enterprise customer signed
- **18 months**: 22-module catalog complete; ARR ≥ $1.5M (HoldCo flip trigger per PRD §1.3); marketplace launched with 50+ third-party skills
- **24 months**: First white-label enterprise deal (Level 5 — ecosystem-as-a-service); CyberSkill team grown to 20–30; SEA market expansion underway

This is the ambition. The architectural substrate (memory/skill/cuo + docs site) is in place. The remaining 19 modules are designed. What's left is execution discipline + distribution.

## 7 — Risks worth pre-empting

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Anthropic deprecates or restructures Agent Skills spec** | Tracks the open agentskills.io spec; contribute upstream to have voice in governance |
| 2 | **OpenAI / Microsoft / Google build a competing "agentic OS"** | Differentiate on: open + regional + audit-chained + multi-vendor. They cannot copy all four. |
| 3 | **CyberSkill team can't ship 19 more modules in 18 months** | Modular ownership (Bet 6); each module is one owner; hiring pace per PRD §1.3 (10→12→14→16→20 over 18 months) |
| 4 | **Vietnamese market too small to justify the investment** | Vietnam is the wedge; full TAM is global. Vertical packs unlock global pricing on local content. |
| 5 | **Open-source contributors fork CyberOS away from CyberSkill** | Standard OSS playbook: trademark "CyberOS" name; CyberSkill keeps consultancy + hosted SaaS + private marketplace as commercial moat |
| 6 | **EU AI Act compliance becomes more onerous than expected** | REW + LEARN designed for Annex III §4 from day one; head-start vs competitors retro-fitting |
| 7 | **AGI accelerates faster than CyberOS can ship** | The substrate stays valuable regardless of model capability; memory + audit + capability sandbox + Vietnamese localization don't go away |

## 8 — Closing summary

CyberOS is at an unusual moment. The architectural substrate is real. The Vietnamese-market wedge is shipping. The Anthropic Agent Skills open standard is settling. The competition is bolt-on AI; CyberOS is agent-native from day one.

The next 12 months are about **distribution, not architecture**. Ship the docs publicly. Ship AUTH so the rest of the modules can land. Ship vertical packs. Build the marketplace. Start the Level 5 enterprise conversations early — they take 6–9 months to close.

CyberSkill the consultancy becomes CyberSkill the platform. The Vietnamese tech scene gets an internationally-credible product company headquartered in HCMC. CyberOS becomes the substrate other Vietnamese (then SEA, then global) businesses run their agentic ops on.

That is the bet.

---

*Strategic recommendations prepared 2026-05-14 alongside the CyberOS docs site Wave 1 upgrade. Companion document to `../website/docs/index.html`. For roadmap detail see `../website/docs/architecture/milestones.html`.*


# `tours/` — guided walkthroughs

`.tour` files are step-by-step walkthroughs for common CyberOS workflows. Open them with the **CodeTour** VS Code extension or read as plain JSON. Each tour points at specific files + line numbers + commands to run in order.

| Tour | When to use it |
| --- | --- |
| [`onboarding.tour`](onboarding.tour) | First-time operator setup: install, init the BRAIN, write first memory. |
| [`incident-response.tour`](incident-response.tour) | Production incident playbook: capture, diagnose, recover, postmortem memory. |
| [`protocol-upgrade.tour`](protocol-upgrade.tour) | Upgrading the AGENTS protocol: §0.5 procedure + canonical-SHA pin update. |
| [`refinement-loop.tour`](refinement-loop.tour) | Acting on `cyberos refinement dashboard` candidates. |
| [`security-audit.tour`](security-audit.tour) | Pre-audit cluster security review (Aspect 13.5). |
| [`repair-audit-chain.tour`](repair-audit-chain.tour) | Recover from corrupt audit ledger. |
| [`repair-fix-frontmatter.tour`](repair-fix-frontmatter.tour) | Fix memories with invalid §5.1 frontmatter. |
| [`repair-manual-rollback.tour`](repair-manual-rollback.tour) | Manual rollback when `cyberos rollback` can't auto-recover. |
| [`repair-stuck-conflict.tour`](repair-stuck-conflict.tour) | Resolve a stuck sync conflict. |
| [`repair-tombstone-orphan.tour`](repair-tombstone-orphan.tour) | Clean up orphaned tombstones from `cyberos prune`. |

## How tours work

Each `.tour` file is JSON describing N steps:

```json
{
  "title": "onboarding",
  "steps": [
    { "file": "runtime/tools/cyberos", "line": 1, "description": "Start here…" },
    { "directory": "docs/memory/", "description": "Read AGENTS.md before proceeding." },
    ...
  ]
}
```

Install the **CodeTour** VS Code extension, open this folder, and tours appear in the activity-bar list. Or read them as plain JSON in your editor.

## Adding a new tour

1. Pick a workflow that takes >3 steps and is run rarely (so its steps blur over time).
2. Write a tour pointing at the relevant files + commands.
3. Add a row to the table above so future-you can find it.
