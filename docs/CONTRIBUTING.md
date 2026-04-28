# Contributing to CyberOS

This is the operational handbook for everyone who files, reviews, or implements work in this repository. Read [README.md](./README.md) first for the lay of the land.

---

## 1. Working agreements

- **English is canonical, Vietnamese is required for client-visible artifacts.** Frontmatter, code, identifiers, and architectural prose are English. Customer-facing copy carries a `_VN:` gloss; the only place a full Vietnamese paragraph is required is the **Bilingual Sales/CS Summary** of feature requests where `client_visible: true`.
- **Spec is law, code is consequence.** If reality and the SRS disagree, the change goes into the SRS first via the change-control process in PRD §14 — never silently into code.
- **IDs are durable.** `FR-{MOD}-{NNN}`, `NFR-{CAT}-{NNN}`, `DEC-{NNN}` — once assigned, never reused. A retired FR is set to `Deprecated`, not deleted.
- **No new top-level docs.** Everything lives under `docs/`. New runbooks become sections of an existing doc or a new file inside `docs/`.

---

## 2. Filing a feature request

The PRD/SRS already enumerate every FR for v1.0. New work in v1.0 lifecycle is mostly:

- (a) **A clarification or sub-task of an existing FR.** Open a child FR file `FR-{MOD}-{NNN}-a.md` or expand the parent's body — do not invent a new top-level FR ID.
- (b) **A genuine new requirement** (rare in v1.0). Bump the SRS in a PR that adds the FR, allocate the next free `{NNN}` for the module, then run the generator.

Either way:

1. Read [`templates/feature-request/README.md`](./templates/feature-request/README.md) for the canonical field reference, section reference, and a fully-filled example. Vietnamese version: [`README_VI.md`](./templates/feature-request/README_VI.md).
2. Copy [`templates/feature-request/FEATURE_REQUEST.md`](./templates/feature-request/FEATURE_REQUEST.md) into `docs/feature-requests/{phase}/{module}/FR-{MOD}-{NNN}.md`, or — preferred — add the entry to [`roadmap/tasks.yaml`](./roadmap/tasks.yaml) and run `pnpm gen:features` so the file is generated from the same data the validator and roadmap consume.
3. Run `pnpm validate:fr` (local) or `pnpm validate:templates` (canonical cyberskill CLI) before opening the PR. Exit code `0` = pass, `1` = errors, `2` = warnings only.

> **Important:** the canonical schema has `additionalProperties: false`. Do **not** add bookkeeping fields (`fr_id`, `module`, `phase`, etc.) to the artifact frontmatter — they live in `tasks.yaml` and the on-disk path. The body is English-only; Vietnamese copy goes in `templates/*/README_VI.md`, never inline.

### EU AI Act bucket selection

Pick `eu_ai_act_risk_class` honestly:

| If the feature… | Bucket | Triggers |
|---|---|---|
| Has no AI involvement at all | `not_ai` | — |
| Calls a model in a non-customer-facing way (search, summarisation that the user clearly initiated) | `minimal` | — |
| Emits AI-generated content visible to a natural person, or affects user-visible behaviour through a model | `limited` | Article 50 transparency obligation; **AI Risk Assessment** required |
| Is in employment / vocational training / promotion / dismissal / payroll / equity (REW, LEARN, ESOP) | `high` | Annex III §4; **AI Risk Assessment** required + EU AI Act Conformity Pack |
| Is on the prohibited list (social scoring, real-time biometric ID, etc.) | — | The schema rejects this — these features must not be filed |

Full mapping in [`compliance/eu-ai-act-risk-classes.md`](./compliance/eu-ai-act-risk-classes.md).

### Required-when rules (validator-enforced)

- `eu_ai_act_risk_class` ∈ {`limited`, `high`} ⇒ `## AI Risk Assessment` H2 with three subsections (`### Data Sources`, `### Human Oversight`, `### Failure Modes`).
- `client_visible: true` ⇒ `## Customer Quotes` and `## Sales/CS Summary` H2 sections present.
- `ai_authorship` ≠ `none` ⇒ `## AI Authorship Disclosure` H2 with three bullets (`Tools used`, `Scope`, `Human review`).

---

## 3. Filing a bug report

Use [`templates/bug-report/BUG_REPORT.md`](./templates/bug-report/BUG_REPORT.md). The field reference is at [`templates/bug-report/README.md`](./templates/bug-report/README.md). For a one-line tweak that has no user-visible behaviour, an issue comment is fine.

If you suspect a Vietnam PDPL personal-data breach, set `pdpl_breach_suspected: true` and `discovered_at: <ISO timestamp>` — the validator then requires the `## Breach Containment` and `## Notification Plan` H2 sections, and the 72-hour notification clock starts at `discovered_at` (PDPL Article 23).

## 4. Pull requests

Use [`templates/pull-request/PULL_REQUEST_TEMPLATE.md`](./templates/pull-request/PULL_REQUEST_TEMPLATE.md). Required-when rules: breaking changes need a migration plan; SOC 2 emergency changes need post-merge review. Field reference at [`templates/pull-request/README.md`](./templates/pull-request/README.md).

---

## 5. Roadmap discipline

[`ROADMAP.md`](./ROADMAP.md) is generated from the same `tasks.yaml`. Don't hand-edit the FR-level checklists — change `tasks.yaml` and re-run `pnpm gen:roadmap` (the generator's roadmap mode) so the human and machine views stay in sync.

When you add an FR to `tasks.yaml`:

```yaml
- id: FR-AUTH-017
  module: AUTH
  phase: P0
  moscow: MUST
  priority: p0
  feature_type: infrastructure
  eu_ai_act_risk_class: not_ai
  client_visible: false
  title: "Issue refresh tokens with replay-detection"
  summary: |
    Issue rotating refresh tokens with replay detection. On replay attempt, revoke
    the entire refresh-token family for the user and force re-auth.
  depends_on: [FR-AUTH-003]
  tags: [security, jwt]
```

The generator handles defaults (created_at, author, template) and emits valid frontmatter.

---

## 6. Pull-request checklist

- [ ] PRD/SRS unchanged, **or** the change is described in the PR body and reviewed by a §14 governance signer
- [ ] If a new FR was added: `tasks.yaml` updated, `pnpm gen:features` run, validator passes
- [ ] If the change touches a module: the relevant module's exit criteria in PRD §8 / SRS §9.2 are still achievable
- [ ] No PII or compensation values committed (CI grep enforces this — see `.github/workflows/ci.yml` once enabled)
- [ ] Vietnamese gloss present on any `client_visible: true` artifact

---

## 7. Module conventions (the structure is enforced by code)

Every module under `apps/{module}/` has the **same 14-file shape**, stamped by `pnpm gen:module` from `apps/_template/`. The shape is non-negotiable; if you need a deviation, raise it in `#cyberos-eng` and we'll update the template (which propagates to all 21 modules in one re-run).

```
apps/{module}/
├── package.json                 # @cyberos/{module}, depends on the workspace packages
├── tsconfig.json                # extends ../../tsconfig.base.json
├── vitest.config.ts
├── Dockerfile
├── README.md                    # links to FRs + dependency contracts
└── src/
    ├── index.ts                 # subgraph entry — boots through @cyberos/subgraph-kit
    ├── graphql/
    │   ├── schema.ts            # federated SDL (uses graphql-tag)
    │   └── resolvers/           # one file per top-level type
    ├── db/
    │   └── schema.prisma        # module-scoped Postgres schema
    ├── mcp/
    │   └── tools.ts             # MCP tools — `{namespace}.{action}`
    ├── events/
    │   ├── publishers.ts        # events this module emits
    │   └── subscribers.ts       # events this module consumes
    └── services/                # business logic — thin resolvers, fat services
```

### Naming rules

| What | Pattern | Example |
|---|---|---|
| Module code | `[A-Z]+` (3–5 chars) | `AUTH`, `REW`, `PROJ` |
| Workspace package | `@cyberos/{lowercase code}` | `@cyberos/auth` |
| Folder | `apps/{lowercase code}/` | `apps/auth/` |
| Port (deterministic) | locked in `modules.yaml`, never reused | `4001`–`4021` |
| GraphQL namespace | lowercase code | `auth`, `rew` |
| GraphQL type prefix | TitleCase code | `Auth`, `Rew` |
| Postgres schema | lowercase code | `auth`, `rew` |
| MCP tool | `{namespace}.{verb}` (snake_case) | `auth.create_session` |
| Event | `{namespace}.{verb}.{noun}` | `rew.payslip.issued` |
| FR id (durable) | `FR-{CODE}-{NNN}` | `FR-AUTH-001` |

### Adding a new module

1. Append an entry to [`modules.yaml`](../modules.yaml) — pick the next free port (current free starts at 4022), set the deps, set the EU AI Act default if it touches employment data.
2. Run `pnpm gen:module --module {CODE}` to stamp the folder.
3. Run `pnpm validate:modules` to confirm no port collisions or broken deps.
4. Open a PR; CI runs the same validators.

### Cross-module communication

- **Read:** GraphQL federation. Never `import { PrismaClient }` from another module.
- **Write:** the module that owns the data is the only one that writes to it. Other modules call its mutation or send an event.
- **Async signal:** NATS event named `{namespace}.{verb}.{noun}` (SRS §5.4). At-least-once; consumers must dedupe on `event.idempotency_key`.
- **AI agent surface:** MCP tools, gated by the same scopes as GraphQL.

## 8. Future-state: GitHub integration

Once the GitHub integration ships (CP module / P4), CyberOS itself will read `tasks.yaml` and call `gh issue create` for each FR — the same data the generator already emits. Keep `tasks.yaml` clean today and the migration is a no-op.
