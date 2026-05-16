# `docs/` — Project-level documentation

This folder holds CyberOS's canonical specifications, the FR backlog, design references, and the document-conversion toolchain.

| Folder / File | Purpose | Entry point |
|---|---|---|
| [`prd/`](prd/) | Product Requirements Document — canonical product brief (Markdown is source of truth; `make docx` regenerates `PRD.docx`) | [`prd/README.md`](prd/README.md) |
| [`srs/`](srs/) | System Requirements Specification — technical spec derived from PRD | [`srs/README.md`](srs/README.md) |
| [`feature-requests/`](feature-requests/) | Living FR backlog, organised by module. Each FR is a markdown file authored via the `fr-author` skill | [`BACKLOG.md`](feature-requests/BACKLOG.md) (index, when present) |
| [`tours/`](tours/) | VS Code CodeTour walkthroughs for common operator workflows | [`tours/README.md`](tours/README.md) |
| [`BRAIN_AUTOSYNC_DESIGN.md`](BRAIN_AUTOSYNC_DESIGN.md) | Locked design for the universal-personal-and-Lumi BRAIN auto-sync system (referenced from every module page) | — |
| [`FR_AUTHORING_WORKFLOW.md`](FR_AUTHORING_WORKFLOW.md) | Canonical playbook for authoring new FRs via the `fr-author` skill | — |
| [`archive/`](archive/) | Historical / one-time documents superseded by later work (kept for traceability) | — |
| [`Makefile`](Makefile) | pandoc round-trip for PRD.md ↔ PRD.docx, SRS.md ↔ SRS.docx | — |

The memory protocol's source-of-truth lives in [`../memory/docs/`](../memory/docs/) (AGENTS.md, EVOLUTION.md, INTEROP.md, PROPOSAL.md, schema, invariants).

The Skill layer lives in [`../skill/`](../skill/) (Rust host + Bun runtime + per-skill `SKILL.md` files).

The CUO orchestrator lives in [`../cuo/`](../cuo/).

## How the layers relate

```
PRD ── authority ──▶ SRS ── authority ──▶ AGENTS protocol (memory module)
                                                │
                                                ▼
                                          Skill layer
                                                │
                                                ▼
                                            CUO orchestrator
                                                │
                                                ▼
                                        Implementation modules
                                        (ai-gateway, auth, mcp, chat, …)
```

- **PRD** says *what we're building and why*.
- **SRS** says *how it's structured technically*.
- **AGENTS protocol** specifies the memory store + audit chain semantics.
- **Skills** are agentic capabilities (CPO / CTO / etc.) with frontmatter + bodies.
- **CUO** orchestrates skill invocations and writes BRAIN audit rows.
- **Implementation modules** are the runtime services (services/auth, services/ai-gateway, etc.) that satisfy the FRs.

## Naming convention

| Convention | Filename | Purpose |
|---|---|---|
| Top-level folder index | `README.md` | What's in this folder |
| Skill index | `SKILL.md` | Skill manifest + body (canonical name; tools look up skills by this filename) |
| Contract index | `CONTRACT.md` | Versioned artefact schema (e.g. `feature_request@1`) |
| FR markdown | `FR-{MOD}-{NNN}-{slug}.md` | One FR per file |
| Per-folder log | `CHANGELOG.md` | Newest-first history |

## Roadmap reference

CyberOS ships in five gated phases — P0 (Foundation) → P1 (Productivity) → P2 (Operations) → P3 (SaaS-ready) → P4 (Client-facing GA). The phase-by-phase milestone arc is documented on the docs site at [`website/docs/architecture/milestones.html`](../website/docs/architecture/milestones.html). All work in `feature-requests/` is tagged by phase.

## Docx ↔ markdown round-trip

`PRD.md` and `SRS.md` are the working source; `.docx` outputs are regenerated via pandoc:

```bash
cd docs
make docx     # md → docx (after editing the markdown)
make md       # docx → md (after a Word user applied changes)
```

Requires `pandoc` (`brew install pandoc` / `apt-get install pandoc`). See [`Makefile`](Makefile) for the exact commands.
