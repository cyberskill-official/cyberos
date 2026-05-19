# `docs/` — Project-level documentation

This folder holds CyberOS's canonical specifications (SDP + C-Suite Reference) and the FR backlog. Module-specific docs live in each module's `README.md` (see [`../modules/`](../modules/)). Operational tours live at [`../tours/`](../tours/).

| Folder / File | Purpose | Entry point |
|---|---|---|
| [`Software Development Process.md`](Software%20Development%20Process.md) | **Normative.** The 13-stage SDP (SOW → SRS → FRs → ADR → SDD → impl → review → test → deploy → release → runbook → retro → decomm). Every skill chains a sub-set of these stages | — |
| [`The C-Suite Reference.md`](The%20C-Suite%20Reference.md) | **Normative.** 48-persona atlas (47 active + 1 EXTINCT cautionary tale). Source for every `modules/cuo/<persona-slug>/README.md`. Sections §2 (acronym matrix), §4 (9-block schema), §5 (per-persona profiles), §7 (CyberSkill priority order), §8 (commercial baselines) | — |
| [`feature-requests/`](feature-requests/) | **Living.** ~556 FRs organised across 26 domains (ai, auth, memory, chat, crm, cuo, doc, docs, email, esop, hr, inv, kb, learn, mcp, obs, okr, portal, proj, res, rew, skill, ten, time). Each FR authored via the `feature-request-author` skill | [`feature-requests/BACKLOG.md`](feature-requests/BACKLOG.md) (index) + [`../modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`](../modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md) (discipline — moved 2026-05-18 to live with the `feature-request-audit` skill) |
| [`Makefile`](Makefile) | pandoc round-trip utility for any md ↔ docx work | — |

### What's gone (deleted 2026-05-18)

- **`docs/prd/`** — frozen 2026-05-15. The PRD was a 434 KB Markdown brief plus docx export. Superseded by the FR catalog under `feature-requests/` (FR-level granularity beats document-level granularity for AI-assisted work).
- **`docs/srs/`** — frozen 2026-05-15. Same logic — the SRS is now expressed as FRs + per-module `README.md` files.

### What moved (2026-05-18)

- **`docs/tours/` → [`../tours/`](../tours/)** — promoted out of `docs/`. CodeTour walkthroughs are operational runbooks, not project documentation.

### Where module docs live now

Each module has a single comprehensive `README.md` at module root with sections for install / audit / fine-tune / deploy. The per-module `docs/` subfolders are gone.

| Module | Read |
|---|---|
| memory protocol + reference impl | [`../modules/memory/README.md`](../modules/memory/README.md) + [`../modules/memory/AGENTS.md`](../modules/memory/AGENTS.md) (Layer-1 spec) + [`../modules/memory/INTEROP.md`](../modules/memory/INTEROP.md) (non-ledger subset) |
| Agent Skills catalog | [`../modules/skill/README.md`](../modules/skill/README.md) (single 4,100-line guide consolidating AUDIT, AUDIT_LOOP, FINE_TUNE, RUBRIC_FORMAT, PUBLISH, SPEC, Phase-5/7 runbooks) |
| Persona-aware orchestration | [`../modules/cuo/README.md`](../modules/cuo/README.md) + [`../modules/cuo/MODULE.md`](../modules/cuo/MODULE.md) (persona catalog) |

## How the layers relate

```
SDP ── normative ──▶ FRs ── authority ──▶ Skill catalog ── compose ──▶ CUO workflows
                                                                            │
                                                                            ▼
                                                                      memory (memory module)
                                                                            │
                                                                            ▼
                                                                   Implementation modules
                                                                   (ai-gateway, auth, mcp, …)
```

- **SDP** (`Software Development Process.md`) defines the 13 stages every deliverable flows through.
- **C-Suite Reference** (`The C-Suite Reference.md`) defines the 48 personas + the 9-block schema each persona spec must render.
- **FRs** (`feature-requests/`) capture every concrete change request, tagged by phase + module.
- **Skill catalog** (`modules/skill/`) ships 104 author+audit pairs that materialise SDP stages into agentic Skills.
- **CUO workflows** (`modules/cuo/`) chain Skills into persona-owned deliverables (194 workflows live).
- **memory** (`modules/memory/`) records every chain decision in an append-only audit chain.
- **Implementation modules** (planned in FRs) are the runtime services (services/auth, services/ai-gateway, etc.) that satisfy the FRs.

## Naming convention

| Convention | Filename | Purpose |
|---|---|---|
| Top-level folder index | `README.md` | What's in this folder |
| Module spec (Layer-1) | `AGENTS.md` | Normative protocol — currently only the memory module has one |
| Skill manifest | `SKILL.md` | Skill body + frontmatter (Anthropic Agent Skills standard) |
| Audit rubric | `RUBRIC.md` | Per-skill rubric for the audit-loop |
| Contract schema | `CONTRACT.md` | Versioned artefact schema (e.g. `feature_request@1`) |
| FR markdown | `FR-{MOD}-{NNN}-{slug}.md` | One FR per file (e.g. `FR-AUTH-001-magic-link.md`) |
| Per-folder log | `CHANGELOG.md` | Newest-first release history |

## Roadmap reference

CyberOS ships in five gated phases — P0 (Foundation) → P1 (Productivity) → P2 (Operations) → P3 (SaaS-ready) → P4 (Client-facing GA). The phase-by-phase milestone arc is documented on the docs site at [`../website/docs/architecture/milestones.html`](../website/docs/architecture/milestones.html). All work in `feature-requests/` is tagged by phase.

## Docx ↔ markdown utility

If you need to convert any future Markdown to/from docx (e.g. for sharing with non-technical reviewers):

```bash
cd docs
make docx INPUT=path/to/file.md
make md   INPUT=path/to/file.docx
```

Requires `pandoc` (`brew install pandoc` / `apt-get install pandoc`). See [`Makefile`](Makefile) for the exact commands.
