# CyberOS website

Multi-page documentation site — 45+ pages, 22 module pages, Liquid Glass design system.

## Structure

```
website/
├── docs/                     ← the deployed site (open index.html in browser)
│   ├── index.html            ← overview / landing
│   ├── modules/              ← per-module pages (22 modules, sub-pages for cuo/memory/skill/plugin)
│   ├── architecture/         ← infrastructure, compliance, tech stack, milestones, strategy
│   ├── reference/            ← FR catalog, NFR catalog, changelog, glossary, risk register, getting-started
│   └── assets/               ← shared nav, JS, CSS, Tailwind, tokens
├── build/                    ← build pipeline (generates pages from source)
│   ├── build.sh              ← one-command rebuild: ./build/build.sh
│   ├── data-extract.mjs      ← walks docs/feature-requests/**/FR-*.md → frs.json
│   ├── render-fr-catalog.mjs ← frs.json → reference/fr-catalog.html
│   ├── nfr-extract.mjs       ← walks docs/non-functional-requirements/**/NFR-*.md → nfrs.json
│   ├── render-nfr-catalog.mjs← nfrs.json → reference/nfr-catalog.html
│   ├── render-changelog.mjs  ← CHANGELOG.md → reference/changelog.html
│   ├── render-module-changelog.mjs ← modules/<slug>/CHANGELOG.md → modules/<slug>/changelog.html
│   └── data/                 ← generated intermediates (frs.json, nfrs.json; deterministic, checked in)
└── README.md                 ← this file
```

## How the build pipeline works

The website's **FR Catalog**, **NFR Catalog**, **Changelog**, and **Per-module Changelogs** are generated from source markdown:

```
docs/feature-requests/**/FR-*.md        →  data-extract.mjs   →  data/frs.json   →  render-fr-catalog.mjs   →  reference/fr-catalog.html
docs/non-functional-requirements/**/NFR-*.md  →  nfr-extract.mjs  →  data/nfrs.json  →  render-nfr-catalog.mjs  →  reference/nfr-catalog.html
CHANGELOG.md                                            →  render-changelog.mjs  →  reference/changelog.html
modules/<slug>/CHANGELOG.md                            →  render-module-changelog.mjs  →  modules/<slug>/changelog.html
```

**When you create or modify source files**, regenerate the catalog:

```bash
./website/build/build.sh              # full build (FR + NFR + changelog + per-module)
./website/build/build.sh --fr         # FR catalog only
./website/build/build.sh --nfr        # NFR catalog only
./website/build/build.sh --changelog  # changelog only (consolidated + per-module)
```

The build is **deterministic** (FR-DOCS-001 §1 #3) — same input produces byte-identical output.

## Deployment

Deployed to **https://cyberos-wiki.cyberskill.world** via **Vercel** (manual deploy). The `cyberos-docs` project lives on the Stephen Cheng's projects team.

```bash
vercel deploy --prod
```

Deploys are operator-controlled — no `vercel.json` in the repo intentionally.

## Sibling projects

| Project | Location | Role |
|---|---|---|
| landing-page | `../landing-page/` | `cyberskill.world` marketing site |
| design-system | `../design-system/` | Liquid Glass + Umber/Ochre tokens (consumed via `docs/assets/tokens.css`) |
