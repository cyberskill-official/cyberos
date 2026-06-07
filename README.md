# CyberOS

CyberSkill's AI-native internal operations platform. 25 federated modules, 104 agent Skills, 223 CUO workflows, 48 C-roles.

## Documentation

**All documentation lives on the [docs site](https://cyberos-wiki.cyberskill.world/).**

| Page | What's there |
|---|---|
| [Getting Started](https://cyberos-wiki.cyberskill.world/reference/getting-started.html) | Repo layout, quick start, versioning, install, deploy runbook |
| [Modules](https://cyberos-wiki.cyberskill.world/) | Per-module pages (25 modules) with appendices, changelogs, deep-dives |
| [FR Catalog](https://cyberos-wiki.cyberskill.world/reference/fr-catalog.html) | ~268 Feature Requests across 26 domains |
| [NFR Catalog](https://cyberos-wiki.cyberskill.world/reference/nfr-catalog.html) | ~157 Non-Functional Requirements across 10 categories |
| [Changelog](https://cyberos-wiki.cyberskill.world/reference/changelog.html) | All significant changes across modules and services |
| [Strategy](https://cyberos-wiki.cyberskill.world/architecture/strategy.html) | Ecosystem landscape, competitive analysis, EaaS roadmap |
| [Roadmap](https://cyberos-wiki.cyberskill.world/architecture/milestones.html) | P0 → P4 phased milestones |

## Repository layout

```
cyberos/
├── modules/          ← production modules (cuo, skill, memory, plugin)
├── services/         ← Rust production binaries (auth, memory, ai-gateway, ...)
├── docs/             ← FR/NFR specs (workflow deliverables → website build pipeline)
├── website/          ← documentation site (Liquid Glass)
├── AGENTS.md         ← Layer-1 memory protocol spec (normative)
├── CLAUDE.md         ← loads AGENTS.md as project instructions
└── VERSION           ← single source of truth for versioning
```

## Quick start

```bash
# Memory module
cd modules/memory && pip install -e .
cyberos --store ../../.cyberos-memory doctor    # → READY

# CUO module
cd ../cuo && pip install -e .
cyberos-cuo list-personas                        # → 47 active personas
```

For the local test path across SKILL, MEMORY, CUO, AUTH, CHAT, and PROJ, use
[`docs/local-live-test.md`](docs/local-live-test.md) and
`scripts/local-live-test.sh`.

## Versioning

```bash
scripts/release.sh minor    # bumps VERSION, propagates to all pyproject.toml + __init__.py
```

## License

MIT

## Maintainer

CyberSkill Software Solutions Consultancy and Development JSC (DUNS 673219568), Ho Chi Minh City, Vietnam.
Founder: Stephen Cheng (Trịnh Thái Anh) · [info@cyberskill.world](mailto:info@cyberskill.world)
