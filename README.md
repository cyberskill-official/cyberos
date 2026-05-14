# CyberOS

CyberSkill's AI-native internal operations platform. Three modules shipped; nineteen designed.

## Quick start

```bash
# Memory module
cd memory && pip install -e . && cyberos doctor

# Skill module
cd skill && cargo run -p cyberos-skill-cli -- list

# CUO router
cd cuo && PYTHONPATH=. python3 -m cuo catalog
```

## Layout

| Folder | Role |
|---|---|
| `memory/` | The BRAIN — append-only audit-chained personal memory store |
| `skill/` | Agent Skills catalog + Rust host + Bun toolchain |
| `cuo/` | The router — natural-language to skill chain to memory record |
| `website/docs/` | Multi-page documentation site (32 pages, Liquid Glass, Pagefind) |
| `strategy/` | Strategic positioning + ecosystem-as-a-service playbook |
| `public-skills/` | Public-repo scaffold for the cyberskill-vn skill collection |
| `docs/prd/` | Product Requirements Document (Markdown source) |
| `docs/srs/` | System Requirements Specification (Markdown source) |
| `.cyberos-memory/` | The user's BRAIN store — gitignored |

### Sibling projects (separate git repos, not under cyberos/)

| Sibling | Where | Role |
|---|---|---|
| **design-system** | `../design-system/` | CyberSkill brand + design doctrine. Liquid Glass v1.1.0. Pull into the docs site as a submodule when needed. |
| **landing-page** | `../landing-page/` | `cyberskill.world` landing page source. Marketing surface. |

Sibling projects intentionally stay separate because they have their own git history, their own release cadence, and their own audit cycles. The docs site cross-references them via relative paths or external URLs once published.

## Status

| Module | Status |
|---|---|
| memory | shipped (245 tests, 30 CLI commands, P2 Stage 3 done) |
| skill | shipped (20 SKILL.md bundles, all 7 audit phases, 6 VN skills) |
| cuo | shipped (Phase 1 rule-based, 15/15 routing fixtures) |
| docs site | shipped (32 pages, 226 diagrams, 341 FRs, 100 NFRs, Pagefind search) |
| design-system (sibling) | doctrine v1.1.0 — Liquid Glass default, L3 enterprise tier |

19 remaining modules (AUTH, AI, MCP, OBS, CHAT, EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN) — scaffolded in docs, not built.

## Documentation

- **Multi-page interactive docs site**: open `website/docs/index.html`
- **Markdown PRD**: `docs/prd/PRD.md`
- **Markdown SRS**: `docs/srs/SRS.md`
- **Strategic playbook**: `strategy/CYBEROS_STRATEGY.md`
- **Design system** (sibling repo): `../design-system/DESIGN.md`
- **Landing page** (sibling repo): `../landing-page/`

## License

Apache 2.0 throughout.

## Maintainer

CyberSkill Software Solutions Consultancy and Development JSC, Ho Chi Minh City, Vietnam.
