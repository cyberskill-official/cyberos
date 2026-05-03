# frontends/

Module Federation remotes — one per module's UI. These are loaded into the host shells in `apps/`:

- `apps/shell-internal` loads all internal remotes (`*-views`).
- `apps/shell-portal` loads ONLY `auth-views`, `portal-views`, and `design-system` — never internal remotes (FR-PORTAL-001 architectural rule).

## Catalog

| Remote | FR | Target shell | Notes |
|---|---|---|---|
| `design-system` | FR-DESIGN-001 | both | Design tokens + component library; foundational |
| `auth-views` | FR-AUTH-001..003 | both | Sign-in, sign-up, passkey, step-up |
| `proj-views` | FR-PROJ-005 | internal | Board / list / timeline |
| `crm-views` | FR-CRM-002 | internal | Pipeline kanban + account 360 |
| `kb-views` | FR-KB-003 | internal | Notion-style block editor + permissions |
| `time-views` | FR-TIME-001..003 | internal | Time entries + leave + expense |
| `email-views` | FR-EMAIL-002 | internal | Missive-style shared inbox |
| `chat-views` | FR-CHAT-001 | internal | Mattermost fork webapp |
| `hr-views` | FR-HR-003 | internal | Directory + org chart |
| `rew-views` | FR-REW-005 | internal | Read-only payslip narrator (CUO/CHRO never compute) |
| `learn-views` | FR-LEARN-004 | internal | Member + manager + Council + admin |
| `inv-views` | FR-INV-004 | internal | AR/AP dashboards + dunning queue |
| `esop-views` | FR-ESOP-003 | internal | Equity dashboard + grant management |
| `res-views` | FR-RES-003 | internal | Allocation Gantt + heatmap |
| `okr-views` | FR-OKR-003 | internal | Cascade tree + heatmap |
| `doc-views` | FR-DOC-002 | internal | Contract redline review |
| `obs-views` | FR-OBS-001..005 | internal | Single-pane dashboards + gates |
| `admin-views` | FR-TEN-005 | internal | Tenant admin console (7 panes) |
| `portal-views` | FR-PORTAL-001..003 | portal only | External client surface + CXO chat |

## Stack (target)

- React 19 + TypeScript
- Vite 6 + Module Federation Plugin (or Rspack 1 with Module Federation v2)
- Tanstack Router + Tanstack Query
- Apollo Client (for federated supergraph reads via `services/gateway`)
- Tailwind CSS using `@cyberos/design-tokens`
- i18n via `@cyberos/i18n` (vi-VN + en-US)

## Per-remote shape (target)

```
frontends/<module>-views/
├── README.md
├── package.json             # @cyberos/<module>-views
├── vite.config.ts           # exposes the remote bundle + module-federation manifest
├── tsconfig.json
├── src/
│   ├── routes/              # Tanstack Router routes
│   ├── components/          # React components
│   ├── queries/             # Apollo queries
│   ├── i18n/                # remote-specific strings
│   └── index.ts             # remote entry point
└── test/
    ├── unit/
    └── e2e/                 # Playwright
```

## Status

`stub` — created 2026-05-03.
