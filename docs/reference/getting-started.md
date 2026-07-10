---
title: CyberOS — Getting Started
source: website/docs/reference/getting-started.html
migrated: FR-DOCS-002
---

[Home](<../index.html>)› [Reference](<../index.html#navigate>)› Getting Started

Reference * Guide

# Getting Started

CyberSkill's AI-native internal operations platform. Three production modules plus a documentation surface and utility folders. This guide covers the repository layout, quick start, versioning, and production deployment for Wave 1. 

## Repository Layout

cyberos/ +-- modules/ <\-- all production modules | +-- cuo/ <\-- persona-aware orchestration (47 personas + 194 workflows) | +-- skill/ <\-- agent Skills catalog (104 author+audit pairs) | +-- memory/ <\-- the memory protocol + reference implementation | +-- plugin/ <\-- packaging + distribution to Claude Code / Cursor / Cowork / Codex CLI | +-- docs/ <\-- canonical project docs | +-- README.md | +-- feature-requests/ <\-- FR catalog across 26 domains (~556 FRs) | +-- strategy/ <\-- strategic positioning + ecosystem playbook +-- website/ <\-- multi-page documentation site (Liquid Glass) +-- docs/sessions/ <\-- per-session log archives +-- services/ <\-- service descriptors | +-- .cyberos/memory/store/ <\-- memory store (gitignored) +-- .github/ <\-- CI workflows +-- README.md <\-- THIS FILE +-- CHANGELOG.md <\-- repo-level umbrella changelog +-- AGENTS.md <\-- Layer-1 memory protocol spec (normative) +-- CLAUDE.md <\-- loads AGENTS.md as project instructions

### modules/ vs services/

Folder| What it is| Why it's separate  
---|---|---  
`modules/` | Catalog + protocol spec + reference Python implementation. Each module is _the deliverable itself_. | Module bundles travel as one folder. Spec + reference impl + CHANGELOG + AGENTS.md all live together so a `cp -r modules/<name>/ <other-repo>/` keeps the unit intact.  
`services/` | Rust production binaries -- the boxed runtime that consumes the protocols defined in `modules/`. | Production services need a Cargo workspace for `Cargo.lock` cohesion + shared crates. Each binary ships as its own Docker image; workspace coupling is build-time only.  
  
## Quick Start

### Memory module
    
    
    # Memory module -- the memory
    cd modules/memory
    pip install -e .
    cyberos --store ../../.cyberos/memory/store doctor          # -> READY 15/15 invariants

### CUO module
    
    
    # CUO module -- persona-aware orchestration
    cd ../cuo
    pip install -e .
    cyberos-cuo list-personas                              # -> 47 active + 1 extinct
    cyberos-cuo route "Architect a new payment system"     # -> chief-technology-officer/architect-new-system
    cyberos-cuo execute chief-technology-officer/adr-quick-capture \
        --output-dir /tmp/run-1 \
        --invoker mock \
        --memory-emit \
        --actor stephen

### Skill module
    
    
    # Skill module -- agentic Skills catalog
    cd ../skill
    ls -1 | grep -E -- '-author$' | wc -l                  # -> 104 author skills

Each module's documentation lives on the [docs site](<https://cyberos-wiki.cyberskill.world/>) \-- install, audit, fine-tune, and deploy instructions are in the per-module appendices and changelogs. 

## Versioning & Release

**Single source of truth:** the `VERSION` file at repo root. All modules share one version.

### Bump a version
    
    
    scripts/release.sh minor          # 0.1.0 -> 0.2.0
    scripts/release.sh patch          # 0.2.0 -> 0.2.1
    scripts/release.sh major          # 0.2.1 -> 1.0.0
    scripts/release.sh 2.5.0          # explicit version
    
    scripts/release.sh --dry-run minor      # preview only
    scripts/release.sh --no-commit patch    # skip git commit + tag

The script reads `VERSION`, bumps it, propagates to all `pyproject.toml` \+ `__init__.py` files, commits, and tags `vX.Y.Z`.

### Version file map

File| Role  
---|---  
`VERSION` (repo root)| Canonical version -- edited by `scripts/release.sh`  
`modules/*/pyproject.toml`| Package metadata -- auto-updated on release  
`modules/*/cyberos*/__init__.py`| `__version__` \-- auto-updated on release  
`.cyberos/memory/store/manifest.json`| Per-store `cyberos_version` \-- updated by `cyberos self-update`  
`.cyberos/memory/store/AGENTS.md`| Protocol file -- re-copied from source on `self-update`  
  
## Install Memory in Another Project
    
    
    # 1. Install the engine (one-time, per machine)
    cd modules/memory && pip install -e .
    
    # 2. Bootstrap a target project
    modules/memory/scripts/install.sh /path/to/your/project

The install script:

  * Creates `.cyberos/memory/store/` with full directory skeleton + `HEAD`
  * Copies `AGENTS.md` into the store (self-contained)
  * Symlinks `AGENTS.md` \+ `CLAUDE.md` from project root into `.cyberos/memory/store/`
  * Adds `.cyberos/memory/store/` to `.gitignore`
  * Runs `cyberos doctor` to verify



## Sync After Version Bump
    
    
    # From the target project directory
    cyberos self-update
    cyberos self-update --force    # re-copy AGENTS.md even if version matches

Every `cyberos` command prints a hint when the store version is stale:  
`hint: cyberos store version (0.0.1) differs from installed (0.1.0). run `cyberos self-update` to sync.`

## Modules

Module| Role| Status| Docs  
---|---|---|---  
P0 `modules/memory/` | Memory -- append-only audit-chained personal memory store | shipped 255 green tests; all 12 audit proposals shipped | [Docs](<https://cyberos-wiki.cyberskill.world/modules/memory/>) * [AGENTS](<https://cyberos-wiki.cyberskill.world/modules/memory/appendices.html>)  
P0 `modules/skill/` | Agent Skills catalog + Rust host + Bun toolchain | shipped 104 author+audit pairs (208 bundles); 108 contracts | [Docs](<https://cyberos-wiki.cyberskill.world/modules/skill/>)  
P0 `modules/cuo/` | Persona-aware orchestration (Chief Universal Officer) | shipped 47 personas + 194 workflows; supervisor Phase 1-3 shipped (21/22 tests pass) | [Docs](<https://cyberos-wiki.cyberskill.world/modules/cuo/>)  
P0 `modules/plugin/` | Packaging + distribution -- exposes CUO + memory + SKILL as installable `.plugin` artefacts for Claude Code / Cursor / Cowork / Codex CLI | scaffold 8 FRs at 10/10; runtime at `services/plugin-host/` planned | [Docs](<https://cyberos-wiki.cyberskill.world/modules/plugin/>)  
  
### Repository Status

Layer| Status  
---|---  
Memory protocol (Layer-1) + reference implementation| Shipped -- 255 tests, 30 CLI commands, P2 Stage 3  
SKILL catalog| 104 pairs / 208 bundles / 108 contracts; zero `planned:` gaps after Session H  
CUO catalog| 47 active personas + 194 workflows; zero gaps after Session N  
CUO supervisor (Python)| Phase 1 (catalog + router + dry-run), Phase 2 (Invoker + execute_chain), Phase 3 (LLMInvoker + memory emission) -- all shipped 2026-05-18  
Docs site (`website/`)| 32 pages, 226 diagrams, 341 FRs, 100 NFRs  
Design system (sibling repo `../design-system/`)| Liquid Glass v1.1.0 -- L3 enterprise tier  
  
## Production Deploy -- Wave 1: MEMORY + AUTH + SKILL

The canonical runbook for taking the first three modules to production. Run in order; each phase is gated by the previous phase's smoke test.

### Prerequisites

Component| Version| Why  
---|---|---  
PostgreSQL| 16.x| Base store for AUTH (RLS) + memory (Layer-2 ingest + audit chain)  
PostgreSQL extensions| `pgvector` 0.7+, `apache_age` 1.5+| Embeddings + graph (memory Layer-2)  
Redis| 7.x| Rate-limit + session cache (AUTH) + event-bus draft  
Rust toolchain| 1.88 stable| Matches `services/Cargo.toml` `rust-version`  
Python| 3.10+| `modules/memory/` reference impl + `modules/cuo/` supervisor + sidecars  
sqlx-cli| 0.8| `cargo install sqlx-cli --no-default-features --features rustls,postgres`  
AWS account| \--| Fargate (services) + ECR (images) + RDS (Postgres) + ElastiCache (Redis)  
Domain + DNS| \--| `cyberos.cyberskill.world` (wiki) * `auth.cyberskill.world` * `memory.cyberskill.world`  
  
### Bootstrap Order
    
    
      +-----------+    +--------+    +--------+    +---------+    +--------------+
      |  MEMORY   | -> |  AUTH  | -> |  CHAT  | -> | PROJECT | -> |  CUO + SKILL |
      | (memory)  |    |        |    |        |    | (PROJ)  |    |              |
      +-----------+    +--------+    +--------+    +---------+    +--------------+
           wave 1        wave 2        wave 3        wave 4        wave 5

### MEMORY Deploy

**What ships:** Layer-1 protocol (file-only memory per `AGENTS.md`) PLUS Layer-2 Rust service (`services/memory/`) for ingest + AGE graph + search REST.
    
    
    # 3.1 -- Build the Rust memory service binary
    cd services
    cargo build --release -p cyberos-memory
    
    # 3.2 -- Apply migrations to the production database
    export DATABASE_URL="postgres://cyberos_admin:$PG_PASSWORD@prod-postgres.cyberskill.world:5432/cyberos"
    sqlx migrate run --source services/memory/migrations
    
    # 3.3 -- Seed the AGE graph
    psql "$DATABASE_URL" -f services/memory/seed/age_init.sql
    
    # 3.4 -- Initialise the local Layer-1 memory (the protocol root)
    cd ../modules/memory
    pip install -e .
    cyberos --store /var/lib/cyberos-memory doctor       # expect: READY 15/15 invariants
    cyberos --store /var/lib/cyberos-memory bootstrap    # writes HEAD=00 + audit segment + index
    
    # 3.5 -- Boot the memory HTTP server
    cd ../../services
    ./target/release/cyberos-memory serve \
        --listen 0.0.0.0:8081 \
        --database-url "$DATABASE_URL" \
        --layer-1-store /var/lib/cyberos-memory \
        --otel-endpoint http://otel-collector.cyberskill.world:4317
    
    # 3.6 -- Smoke tests
    curl -fsS http://localhost:8081/healthz                          # -> {"status":"ready"}
    curl -fsS http://localhost:8081/v1/audit/chain | jq '.head_seq'  # -> 0 (or current head)
    cyberos --store /var/lib/cyberos-memory invariants               # -> 15/15 PASS
    
    # 3.7 -- Deploy to Fargate
    aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com
    docker build -t cyberos-memory:$(git rev-parse --short HEAD) services/memory/
    docker tag cyberos-memory:$(git rev-parse --short HEAD) <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-memory:latest
    docker push <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-memory:latest
    aws ecs update-service --cluster cyberos-prod --service memory --force-new-deployment

**Health endpoints:** `/healthz` (ready/not-ready), `/v1/audit/chain` (current head + tip hash), `/metrics` (Prometheus).

**Rollback:** `aws ecs update-service --cluster cyberos-prod --service memory --task-definition cyberos-memory:N-1 --force-new-deployment`

### AUTH Deploy

**What ships:** Rust AUTH service (`services/auth/`) with RLS, JWT/JWKS issuance, MFA (TOTP/WebAuthn/Passkey), SAML/OIDC SSO, 22-role RBAC catalogue.
    
    
    # 4.1 -- Build
    cd services
    cargo build --release -p cyberos-auth
    
    # 4.2 -- Apply migrations (20 ordered SQL files; idempotent post-2026-05-19 fix)
    export DATABASE_URL="postgres://cyberos_admin:$PG_PASSWORD@prod-postgres.cyberskill.world:5432/cyberos"
    sqlx migrate run --source services/auth/migrations
    
    # 4.3 -- Generate signing keys (one-time per environment)
    ./target/release/cyberos-auth keygen \
        --algorithm RS256 \
        --kid auth-prod-2026-05-1 \
        --output /var/lib/cyberos/keys/auth-prod-2026-05-1.jwk
    
    # 4.4 -- Bootstrap the root tenant + first admin subject
    ./target/release/cyberos-auth bootstrap \
        --root-tenant-name "CyberSkill" \
        --admin-email stephen@cyberskill.world \
        --otp-secret-out /tmp/admin-totp.txt
    
    # 4.5 -- Boot AUTH HTTP server
    ./target/release/cyberos-auth serve \
        --listen 0.0.0.0:8080 \
        --database-url "$DATABASE_URL" \
        --redis-url "redis://prod-redis.cyberskill.world:6379/0" \
        --jwk-path /var/lib/cyberos/keys/auth-prod-2026-05-1.jwk \
        --memory-base-url http://memory.cyberskill.world \
        --otel-endpoint http://otel-collector.cyberskill.world:4317
    
    # 4.6 -- Smoke tests
    curl -fsS http://localhost:8080/healthz                                      # -> {"status":"ready"}
    curl -fsS http://localhost:8080/.well-known/jwks.json | jq '.keys | length' # -> 1
    curl -fsS http://localhost:8080/.well-known/openid-configuration | jq .     # -> discovery doc
    
    # 4.7 -- Deploy to Fargate
    docker build -t cyberos-auth:$(git rev-parse --short HEAD) services/auth/
    docker push <ecr-id>.dkr.ecr.ap-southeast-1.amazonaws.com/cyberos-auth:latest
    aws ecs update-service --cluster cyberos-prod --service auth --force-new-deployment

**Health endpoints:** `/healthz`, `/.well-known/jwks.json`, `/.well-known/openid-configuration`, `/metrics`.

**Rollback:** ECS task-def rollback. Rolling back ACROSS a migration requires `sqlx migrate revert` against the database FIRST -- migrations are forward-only by default.

### SKILL Deploy

**What ships today:** the catalog itself (`modules/skill/`) -- 104 author+audit pairs + 108 contracts. The Rust skill-broker (`services/skill-broker/`) is not yet built (FR-SKILL-103). For Wave 1 production, the catalog is consumed by the CUO supervisor which routes user queries to the right skill.
    
    
    # 5.1 -- Validate the catalog
    cd modules/skill
    ls -1 | grep -E -- '-author$' | wc -l        # -> 104 (expected)
    ls -1 | grep -E -- '-audit$' | wc -l         # -> 104 (expected)
    
    # 5.2 -- Run the SKB-* validators
    cd ../cuo
    PYTHONPATH=. python3 -m cuo.trigger_tests --catalog ../skill
    PYTHONPATH=. python3 -m cuo.baseline --catalog ../skill
    PYTHONPATH=. python3 -m cuo.placeholder_check --catalog ../skill
    
    # 5.3 -- Verify chain integrity (every skill's contract dependencies resolve)
    PYTHONPATH=. python3 -m cuo.cli validate-chains
    # -> 104 chains pass; 0 broken
    
    # 5.4 -- Smoke: end-to-end chain
    python3 -c "
    from cuo.core.router import route
    decision = route('Turn this PRD into a backlog of FRs')
    print(f'Routes to: {decision.persona_slug}/{decision.workflow_slug}')
    print(f'Confidence: {decision.confidence:.2f}')
    "

**Health endpoints:** none today (catalog is markdown, not a service). When `services/skill-broker/` ships: `/healthz`, `/v1/skills`, `/v1/skills/<id>/validate`.

**Rollback:** `git revert` on the catalog. No DB schema changes to worry about today.

### End-to-End Smoke

After all three modules are deployed independently, prove they talk to each other:
    
    
    # Get an AUTH token
    TOKEN=$(curl -fsS -X POST http://auth.cyberskill.world/v1/auth/token \
        -d "grant_type=password&username;=stephen@cyberskill.world&password;=$PW&totp;=$(oathtool --totp --base32 $TOTP_SECRET)" | jq -r .access_token)
    
    # Write a memory audit row via memory service (AUTH-stamped)
    curl -fsS -X POST http://memory.cyberskill.world/v1/audit/append \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"kind":"put","path":"memories/projects/cyberos/smoke-test.md","body":"hello"}' \
        | jq '.audit_id'
    
    # Route a query through CUO that touches both AUTH + memory
    python3 -m cuo.core.supervisor execute \
        --actor "human:stephen-cheng" \
        --query "Audit FR-AUTH-003 for completeness" \
        --memory-emit \
        --output-dir /tmp/e2e-smoke
    
    curl -fsS http://memory.cyberskill.world/v1/audit/chain | jq '.head_seq'  # incremented by N

**Pass criteria:** AUTH issues token, memory accepts authenticated audit-append, CUO routes correctly, memory chain head advances.

### Day-2 Operations

Concern| Where it lives  
---|---  
Health monitoring| Grafana dashboards `cyberos-memory` \+ `cyberos-auth`  
Alerting| PagerDuty service `cyberos-prod-on-call`; routes via Alertmanager per FR-OBS-007  
Backup (Postgres)| RDS automated snapshots -- 7-day retention. Cross-region replication to ap-southeast-2  
Backup (memory local store)| `cyberos export <tenant_id> --target s3://cyberos-backups/<env>/<date>.zip` (nightly via Lambda)  
Audit ledger integrity| Tamper detector per AGENTS.md; alerts on chain breaks  
Cost| AWS Cost Explorer tag `Project=cyberos` \+ budget alarm at 80% of $535/mo envelope  
  
## Documentation

  * **Docs site** (single source of truth): [cyberos-wiki.cyberskill.world](<https://cyberos-wiki.cyberskill.world/>)
  * **Module docs** : per-module pages with appendices, changelogs, and technical deep-dives
  * **Feature requests** : `docs/feature-requests/` (~268 FRs across 26 domains)
  * **Strategic playbook** : `strategy/`



## Sibling Projects

Sibling| Where| Role  
---|---|---  
**design-system**| `../design-system/`| CyberSkill brand + design doctrine. Liquid Glass v1.1.0  
**landing-page**| `../landing-page/`| `cyberskill.world` landing page source  
**sale-noti**| `../sale-noti/`| Sales notification subsystem  
**tamagochi**| `../tamagochi/`| Virtual-pet game + PetOS B2B (53 FRs at 10/10)  
  
Siblings stay separate because they have their own git history, release cadence, and audit cycles.

**License:** MIT throughout. **Maintainer:** CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam. Founder: Stephen Cheng (Trinh Thai Anh) * [info@cyberskill.world](<mailto:info@cyberskill.world>)
