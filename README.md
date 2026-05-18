# CyberOS

CyberSkill's AI-native internal operations platform. Three production modules + a documentation/strategy surface + utility folders.

## Repository layout (post-2026-05-18 refactor)

```
cyberos/
├── modules/                ← all three production modules
│   ├── cuo/                ← persona-aware orchestration (47 personas + 194 workflows)
│   ├── skill/              ← agent Skills catalog (104 author+audit pairs)
│   └── memory/             ← the BRAIN protocol + reference implementation
│
├── docs/                   ← canonical project docs
│   ├── README.md
│   ├── Software Development Process.md   ← SDP 13 stages (normative)
│   ├── The C-Suite Reference.md          ← 48-persona atlas (normative)
│   └── feature-requests/   ← FR catalog across 26 domains (~556 FRs)
│
├── tours/                  ← CodeTour walkthroughs (incident response, BRAIN repair, security audit)
├── strategy/               ← strategic positioning + ecosystem playbook
├── website/                ← multi-page documentation site (Liquid Glass, Pagefind)
├── docs/sessions/         ← per-session log archives (new — see docs/sessions/2026-05-18-wave-1-2.md)
├── services/               ← service descriptors
├── pagefind/               ← static search index for website
│
├── .cyberos-memory/        ← BRAIN store (gitignored)
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

- `modules/memory/` = the BRAIN protocol spec + Python reference writer (`AGENTS.md` + `memory.schema.json` + `memory.invariants.yaml` + `cyberos/` package). `services/brain/` = the Rust production BRAIN service that implements that protocol.
- `modules/skill/` = the SKILL catalog (markdown bundles, RUBRIC, AUTHORING discipline). The future Rust skill-broker will live in `services/skill-broker/` when FR-SKILL-103 ships.
- `modules/cuo/` = the CUO Python supervisor; no Rust port planned (it stays Python).
- `services/auth/` = the Rust AUTH service; **has no `modules/auth/`** because AUTH has no spec/Python-reference-impl split. Its spec lives in `docs/feature-requests/auth/` directly.

**The split is intentional and stable.** Both layers preserve module isolation: `modules/<name>/` clones cleanly because it's a folder; `services/<name>/` clones cleanly because Cargo lets you build a single workspace member with `cargo build -p <name>`. The workspace's shared `Cargo.lock` is a build-time cache, not a coupling — production deploys ship each binary independently.

## Quick start

```bash
# Memory module — the BRAIN
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
    --brain-emit \
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
| [`modules/memory/`](modules/memory/) | BRAIN — append-only audit-chained personal memory store | 255 green tests; all 12 audit proposals shipped | [README](modules/memory/README.md) · [AGENTS](modules/memory/AGENTS.md) |
| [`modules/skill/`](modules/skill/) | Agent Skills catalog + Rust host + Bun toolchain | 104 author+audit pairs (208 bundles); 108 contracts; catalog-complete post-Session H | [README](modules/skill/README.md) |
| [`modules/cuo/`](modules/cuo/) | Persona-aware orchestration (Chief Universal Officer) | 47 personas + 194 workflows; supervisor Phase 1–3 shipped (21/22 tests pass) | [README](modules/cuo/README.md) |

## Status

| Layer | Status |
|---|---|
| BRAIN protocol (Layer-1) + reference implementation | shipped — 255 tests, 30 CLI commands, P2 Stage 3 |
| SKILL catalog | 104 pairs / 208 bundles / 108 contracts; zero `planned:` gaps after Session H |
| CUO catalog | 47 active personas + 194 workflows; zero gaps after Session N |
| CUO supervisor (Python) | Phase 1 (catalog + router + dry-run), Phase 2 (Invoker + execute_chain), Phase 3 (LLMInvoker + BRAIN emission) — all shipped 2026-05-18 |
| Docs site (`website/`) | 32 pages, 226 diagrams, 341 FRs, 100 NFRs, Pagefind search |
| Design system (sibling repo `../design-system/`) | Liquid Glass v1.1.0 — L3 enterprise tier |

**Roadmap:** CUO Phase 4 (5 special-case workflow handlers); CUO depth additions (per-persona workflow expansion 4 → 8–12); 19 remaining modules (AUTH, AI, MCP, OBS, CHAT, EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN) — scaffolded in docs, not built.

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
| PostgreSQL | 16.x | base store for AUTH (RLS) + BRAIN (Layer-2 ingest + audit chain) |
| PostgreSQL extensions | `pgvector` 0.7+, `apache_age` 1.5+ | embeddings + graph (BRAIN Layer-2) |
| Redis | 7.x | rate-limit + session cache (AUTH) + event-bus draft (will become NATS at scale) |
| Rust toolchain | 1.81 stable | matches `services/Cargo.toml` `rust-version` |
| Python | 3.10+ | `modules/memory/` reference impl + `modules/cuo/` supervisor + sidecars |
| sqlx-cli | 0.8 | `cargo install sqlx-cli --no-default-features --features rustls,postgres` |
| AWS account | — | Fargate (services) + ECR (images) + RDS (Postgres) + ElastiCache (Redis) |
| Domain + DNS | — | `cyberos.cyberskill.world` (wiki) · `auth.cyberskill.world` (AUTH endpoint) · `brain.cyberskill.world` (BRAIN endpoint) |

### §2 — Bootstrap order (deploy roadmap)

Per [`docs/feature-requests/BACKLOG.md` §0.6](docs/feature-requests/BACKLOG.md), the user-locked production order is:

```
  ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────┐    ┌──────────────┐
  │  MEMORY  │ ─▶ │  AUTH  │ ─▶ │  CHAT  │ ─▶ │ PROJECT │ ─▶ │  CUO + SKILL │
  │ (BRAIN)  │    │        │    │        │    │ (PROJ)  │    │              │
  └──────────┘    └────────┘    └────────┘    └─────────┘    └──────────────┘
        wave 1         wave 2         wave 3         wave 4         wave 5
```

Wave 1 (this runbook) covers MEMORY + AUTH + SKILL. The next sections walk each.

### §3 — MEMORY deploy

**What ships:** Layer-1 protocol (file-only BRAIN per [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md)) PLUS Layer-2 Rust service ([`services/brain/`](services/brain/)) for ingest + AGE graph + search REST.

```bash
# 3.1 — Build the Rust BRAIN service binary
cd services
cargo build --release -p cyberos-brain
ls -lh target/release/cyberos-brain   # → single statically-linked binary

# 3.2 — Apply migrations to the production database
export DATABASE_URL="postgres://cyberos_admin:$PG_PASSWORD@prod-postgres.cyberskill.world:5432/cyberos"
sqlx migrate run --source services/brain/migrations

# 3.3 — Seed the AGE graph
psql "$DATABASE_URL" -f services/brain/seed/age_init.sql

# 3.4 — Initialise the local Layer-1 BRAIN (the protocol root)
cd ../modules/memory
pip install -e .
cyberos --store /var/lib/cyberos-memory doctor       # expect: READY ✓ 15/15 invariants
cyberos --store /var/lib/cyberos-memory bootstrap    # writes HEAD=00 + audit segment + index

# 3.5 — Boot the BRAIN HTTP server
cd ../../services
./target/release/cyberos-brain serve \
    --listen 0.0.0.0:8081 \
    --database-url "$DATABASE_URL" \
    --layer-1-store /var/lib/cyberos-memory \
    --otel-endpoint http://otel-collector.cyberskill.world:4317

# 3.6 — Smoke tests
curl -fsS http://localhost:8081/healthz                          # → {"status":"ready"}
curl -fsS http://localhost:8081/v1/audit/chain | jq '.head_seq'  # → 0 (or current head)
cyberos --store /var/lib/cyberos-memory invariants               # → 15/15 PASS

# 3.7 — Deploy to Fargate (per docs/feature-requests/brain/FR-BRAIN-104)
aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com
docker build -t cyberos-brain:$(git rev-parse --short HEAD) services/brain/
docker tag cyberos-brain:$(git rev-parse --short HEAD) <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-brain:latest
docker push <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-brain:latest
aws ecs update-service --cluster cyberos-prod --service brain --force-new-deployment
```

**Health endpoints:** `/healthz` (ready/not-ready), `/v1/audit/chain` (current head + tip hash), `/metrics` (Prometheus exposition).

**Rollback:** `aws ecs update-service --cluster cyberos-prod --service brain --task-definition cyberos-brain:N-1 --force-new-deployment` (replace `N-1` with the previous task-def revision).

**Observability:** Grafana dashboard `cyberos-brain` (panels for HEAD lag, audit-row rate, AGE query p95, embedding sidecar latency). Datadog tag `service:cyberos-brain`.

**Secrets:** Postgres password lives in AWS Secrets Manager at `arn:aws:secretsmanager:ap-southeast-1:<acct>:secret:cyberos/prod/brain/db-password`. OTel collector token at `…/otel-token`.

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
    --brain-base-url http://brain.cyberskill.world \
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

**Rollback:** ECS task-def rollback (same pattern as BRAIN). **Critical:** rolling back ACROSS a migration requires `sqlx migrate revert` against the database FIRST — migrations are forward-only by default.

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

# Write a BRAIN audit row via BRAIN service (AUTH-stamped)
curl -fsS -X POST http://brain.cyberskill.world/v1/audit/append \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"kind":"put","path":"memories/projects/cyberos/smoke-test.md","body":"hello"}' \
    | jq '.audit_id'

# Route a query through CUO that touches both AUTH (for identity) + BRAIN (for memory read)
python3 -m cuo.core.supervisor execute \
    --actor "human:stephen-cheng" \
    --query "Audit FR-AUTH-003 for completeness" \
    --brain-emit \
    --output-dir /tmp/e2e-smoke
ls /tmp/e2e-smoke/                                     # → step output JSONs
curl -fsS http://brain.cyberskill.world/v1/audit/chain | jq '.head_seq'  # incremented by N
```

**Pass criteria:** AUTH issues token, BRAIN accepts authenticated audit-append, CUO routes correctly, BRAIN chain head advances. If any step fails, the rollback for that service per §3.6/§4.8 fires.

### §7 — Day-2 operations

| Concern | Where it lives |
|---|---|
| Health monitoring | Grafana dashboards `cyberos-brain` + `cyberos-auth` (live via [Datadog](https://app.datadoghq.eu/) when configured) |
| Alerting | PagerDuty service `cyberos-prod-on-call`; routes via Alertmanager per FR-OBS-007 |
| Backup (Postgres) | RDS automated snapshots — 7-day retention. Cross-region replication to ap-southeast-2 (per FR-AUTH-005 compliance) |
| Backup (BRAIN local store) | `cyberos export <tenant_id> --target s3://cyberos-backups/<env>/<date>.zip` runs nightly via Lambda |
| Audit ledger integrity | Tamper detector per AGENTS.md §10 + SRS §10.4.6 runs continuously; alerts on chain breaks |
| Cost | AWS Cost Explorer tag `Project=cyberos` + budget alarm at 80% of $535/mo envelope (per `docs/feature-requests/BACKLOG.md` §architecture/tech-stack) |

## Documentation

- **Multi-page interactive docs site**: open `website/docs/index.html`
- **SDP (Software Development Process)**: `docs/Software Development Process.md` (13 stages, normative)
- **C-Suite Reference**: `docs/The C-Suite Reference.md` (48-persona atlas, normative)
- **Feature requests**: `docs/feature-requests/` (~556 FRs across 26 domains; see `BACKLOG.md`)
- **Operational tours**: `tours/` (CodeTour walkthroughs — open with VS Code CodeTour extension)
- **Per-module READMEs**: each `modules/*/README.md` is comprehensive (install / audit / fine-tune / deploy)
- **Strategic playbook**: `strategy/`

## License

MIT throughout (was Apache 2.0 in earlier docs — modules ship MIT per their `pyproject.toml` / `Cargo.toml`).

## Maintainer

CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam.
Founder: Stephen Cheng (Trịnh Thái Anh) · [info@cyberskill.world](mailto:info@cyberskill.world)
