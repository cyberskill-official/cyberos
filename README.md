# CyberOS

CyberSkill's AI-native internal operations platform. 26 federated modules, 244 agent Skills, 224 CUO workflows, 48 C-roles, 577 tasks.

## Documentation

**All documentation lives on the [docs site](https://os.cyberskill.world/docs/).**

| Page | What's there |
|---|---|
| [Getting Started](https://os.cyberskill.world/docs/reference/getting-started.html) | Repo layout, quick start, versioning, install, deploy runbook |
| [Modules](https://os.cyberskill.world/docs/) | Per-module pages (26 modules) with appendices, changelogs, deep-dives |
| [task Catalog](https://os.cyberskill.world/docs/reference/task-catalog.html) | 577 Tasks across 29 domains |
| [NFR Catalog](https://os.cyberskill.world/docs/reference/nfr-catalog.html) | ~157 Non-Functional Requirements across 10 categories |
| [Changelog](https://os.cyberskill.world/docs/reference/changelog.html) | All significant changes across modules and services |
| [Strategy](https://os.cyberskill.world/docs/architecture/strategy.html) | Ecosystem landscape, competitive analysis, EaaS roadmap |
| [Roadmap](https://os.cyberskill.world/docs/architecture/milestones.html) | P0 -> P4 phased milestones |

## Repository layout

```
cyberos/
├── modules/          <- federated modules (cuo, skill, memory, ...), each owning its docs/
├── services/         <- Rust production binaries (auth, chat, memory, ai-gateway, ...), each owning its docs/
├── apps/             <- the one client (web) + thin desktop/console wrappers
├── docs/             <- global docs sources: task/NFR specs, architecture, deploy runbooks
├── tools/            <- install (the distributable payload) + docs-site (website generator)
├── scripts/          <- gates, local_verify.sh (CI-equivalent), release.sh
├── deploy/           <- VPS compose + Caddyfile
├── dist/             <- build outputs (payload, website); gitignored, never committed
├── AGENTS.md         <- Layer-1 memory protocol spec (normative)
├── CLAUDE.md         <- loads AGENTS.md as project instructions
└── VERSION           <- the single platform version
```

The website is generated (`bash tools/docs-site/build.sh` -> `dist/website`); there is no hand-authored HTML in the repo. Consumer repos install CyberOS from the payload (`bash tools/install/build.sh`, then `dist/cyberos/install.sh <repo>`) - see the day-one guide on the docs site (cuo module -> Guides).

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

CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam. Founder: Stephen Cheng (Trịnh Thái Anh) · [info@cyberskill.world](mailto:info@cyberskill.world)
