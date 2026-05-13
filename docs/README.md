# `docs/` — Project-level documentation

| Folder | Purpose | Entry point |
| --- | --- | --- |
| [`prd/`](prd/) | Product Requirements Document for CyberOS itself (markdown is source of truth; `make docx` regenerates `PRD.docx`) | [`prd/README.md`](prd/README.md) |
| [`srs/`](srs/) | System Requirements Specification (markdown is source of truth; `make docx` regenerates `SRS.docx`) | [`srs/README.md`](srs/README.md) |
| [`skills/`](skills/) | Skills layer — CPO/CTO chain skills + chain orchestrator + host adapters (will move into a `skills/` module folder next pass) | [`skills/README.md`](skills/README.md) |
| [`contracts/`](contracts/) | Versioned artefact schemas (`feature_request@1`, `task@1`, `prd@1`, `srs@1`, …) — pending skill module move | [`contracts/README.md`](contracts/README.md) |
| [`tours/`](tours/) | Guided walkthroughs (`.tour` files) for common workflows — pending skill module move | [`tours/README.md`](tours/README.md) |

The memory protocol now lives in [`../memory/docs/`](../memory/docs/) (relocated 2026-05-13 during the memory-module restructure). The schema, invariants, AGENTS.md, EVOLUTION.md, INTEROP.md, PROPOSAL.md, and CHANGELOG.md are all there.

## How the layers relate

```text
PRD  ─authority──►  SRS  ─authority──►  AGENTS protocol
                                              │
                                              ▼
                                       Skills layer
                                              │
                                              ▼
                                    contracts (artefact schemas)
                                              │
                                              ▼
                                     runtime/ implements everything
```

- **PRD** says *what we're building and why*.
- **SRS** says *how it's structured technically*.
- **AGENTS protocol** says *how a memory works, how the BRAIN operates*.
- **Skills** say *what work an agent can do*.
- **Contracts** say *what shape the inputs/outputs of skills must conform to*.
- **Runtime** under `../runtime/` implements all of it as Python tools.

## Naming convention

- Top-level folder entry point: **`README.md`**.
- Skill folder entry point: **`SKILL.md`** (established convention; tools look up skills by this filename).
- Contract folder entry point: **`CONTRACT.md`** (deliberate signal that "this is a schema, not a skill").
- Daily history: **`CHANGELOG.md`**.

## Cross-references

- The single source of truth for the memory protocol is [`../memory/docs/AGENTS.md`](../memory/docs/AGENTS.md) (RFC, v2).
- The CLI is `python -m cyberos` (or `cyberos` after `cd memory && pip install -e .`).
- Per-batch history lives in [`../memory/docs/CHANGELOG.md`](../memory/docs/CHANGELOG.md).

## Docx ↔ markdown round-trip

`PRD.md` and `SRS.md` are the source of truth; `.docx` outputs are regenerated via pandoc:

```bash
cd docs
make docx     # md → docx (after editing the markdown)
make md       # docx → md (after a Word user applied changes)
```

Requires `pandoc` (`brew install pandoc` / `apt-get install pandoc`). See [`Makefile`](Makefile) for the exact commands.
