# CyberOS

CyberSkill's AI-native internal operations platform. 25 federated modules, 104 agent Skills, 223 CUO workflows, 48 C-roles.

## Documentation

**All documentation lives on the [docs site](https://cyberos-wiki.cyberskill.world/).**

| Page | What's there |
|---|---|
| [Getting Started](https://cyberos-wiki.cyberskill.world/reference/getting-started.html) | Repo layout, quick start, versioning, install, deploy runbook |
| [Modules](https://cyberos-wiki.cyberskill.world/) | Per-module pages (25 modules) with appendices, changelogs, deep-dives |
| [FR Catalog](https://cyberos-wiki.cyberskill.world/reference/fr-catalog.html) | 489 Feature Requests across 29 domains |
| [NFR Catalog](https://cyberos-wiki.cyberskill.world/reference/nfr-catalog.html) | ~157 Non-Functional Requirements across 10 categories |
| [Changelog](https://cyberos-wiki.cyberskill.world/reference/changelog.html) | All significant changes across modules and services |
| [Strategy](https://cyberos-wiki.cyberskill.world/architecture/strategy.html) | Ecosystem landscape, competitive analysis, EaaS roadmap |
| [Roadmap](https://cyberos-wiki.cyberskill.world/architecture/milestones.html) | P0 -> P4 phased milestones |

## Repository layout

```
cyberos/
├── modules/          <- federated modules (cuo, skill, memory, ...), each owning its docs/
├── services/         <- Rust production binaries (auth, chat, memory, ai-gateway, ...), each owning its docs/
├── apps/             <- the one client (web) + thin desktop/console wrappers
├── docs/             <- global docs sources: FR/NFR specs, architecture, deploy runbooks
├── tools/            <- cyberos-init (the distributable payload) + docs-site (website generator)
├── scripts/          <- gates, local_verify.sh (CI-equivalent), release.sh
├── deploy/           <- VPS compose + Caddyfile
├── dist/             <- build outputs (payload, website); gitignored, never committed
├── AGENTS.md         <- Layer-1 memory protocol spec (normative)
├── CLAUDE.md         <- loads AGENTS.md as project instructions
└── VERSION           <- the single platform version
```

The website is generated (`bash tools/docs-site/build.sh` -> `dist/website`); there is no hand-authored HTML in the repo. Consumer repos install CyberOS from the payload (`bash tools/cyberos-init/build.sh`, then `dist/cyberos/init.sh <repo>`) - see the day-one guide on the docs site (cuo module -> Guides).

## Quick start

```bash
# Memory module
cd modules/memory && pip install -e .
cyberos --store ../../.cyberos/memory/store doctor    # → READY

# CUO module
cd ../cuo && pip install -e .
cyberos-cuo list-personas                        # → 47 active personas
```

## Versioning

```bash
scripts/release.sh minor    # bumps VERSION, propagates to all pyproject.toml + __init__.py
```

## License

MIT

## Maintainer

CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam.
Founder: Stephen Cheng (Trịnh Thái Anh) · [info@cyberskill.world](mailto:info@cyberskill.world)
