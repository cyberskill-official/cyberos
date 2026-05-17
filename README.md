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
├── runtime/                ← Rust runtime artefacts (separate from skill host)
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
