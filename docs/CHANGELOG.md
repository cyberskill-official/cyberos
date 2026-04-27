# Changelog

All notable changes to CyberOS are documented here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versions follow [Semantic Versioning](https://semver.org/) per module (`@cyberos/{module}@MAJOR.MINOR.PATCH`).  
Monorepo releases managed by [Changesets](https://github.com/changesets/changesets).

---

## [Unreleased]

### Added
- Monorepo scaffold: pnpm workspaces + Turborepo + Changesets
- `packages/config-tsconfig` — shared TypeScript base config
- `packages/config-eslint` — shared ESLint flat config
- `packages/graphql-shared` — shared scalars (DateTime, EmailAddress, URL, JSON), PageInfo, error codes
- `packages/db-shared` — Prisma RLS helpers, `setTenantContext`, soft-delete mixin
- `apps/shell` — Module Federation host skeleton
- `modules/auth` — canonical module reference: Federation v2 SDL, tenant middleware, MCP tools template
- All 22 module directories scaffolded (P0–P4)
- `infra/docker/docker-compose.yml` — Postgres 17+pgvector, Redis 7, NATS 2.10
- `infra/docker/init-extensions.sql` — uuid-ossp, pgvector, pg_jsonschema
- `.github/workflows/ci.yml` — Turborepo affected-only CI + `rover subgraph check`
- `docs/` — all project documentation consolidated (PRD, SRS, TASKS, README, CHANGELOG)
- `docs/PRD.md` — Product Requirements Document v0.1
- `docs/SRS.md` — Software Requirements Specification v0.1 (37 locked DECs, full FR/NFR catalog)
- `docs/TASKS.md` — Full P0–P4 build checklist (~170 tasks across engineering + compliance)

---

<!-- Template for future entries:

## [@cyberos/auth@0.1.0] — YYYY-MM-DD

### Added
- JWT RS256 issuance + OIDC discovery endpoint
- Argon2id password hashing
- Google OAuth 2.0 login
- TOTP MFA mandatory flow
- `auth.whoami` MCP tool

### Changed
- ...

### Fixed
- ...

### Security
- ...

-->

[Unreleased]: https://github.com/cyberskill-official/cyberos/compare/HEAD
