# `docs/` — All documentation

CyberOS keeps every kind of documentation under this single tree. One README per folder, one folder per concern.

| Folder | Purpose | Entry point |
| --- | --- | --- |
| [`memory/`](memory/) | AGENTS protocol — the memory layer rules, schema, source-tier system, audit ledger | [`memory/README.md`](memory/README.md) |
| [`skills/`](skills/) | Skills layer — CPO/CTO chain skills + chain orchestrator + host adapters | [`skills/README.md`](skills/README.md) |
| [`contracts/`](contracts/) | Versioned artefact schemas (`feature_request@1`, `task@1`, `prd@1`, `srs@1`, …) | [`contracts/README.md`](contracts/README.md) |
| [`prd/`](prd/) | Product Requirements Document for CyberOS itself (markdown is source of truth; `make docx` regenerates `PRD.docx`) | [`prd/README.md`](prd/README.md) |
| [`srs/`](srs/) | System Requirements Specification (markdown is source of truth; `make docx` regenerates `SRS.docx`) | [`srs/README.md`](srs/README.md) |
| [`tours/`](tours/) | 10 guided walkthroughs (`.tour` files) for common workflows | [`tours/README.md`](tours/README.md) |

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

- The single source of truth for the AGENTS protocol is [`memory/AGENTS.md`](memory/AGENTS.md) (v2.0.0). The compact "AGENTS-CORE" variant was removed in Batch 27 (2026-05-12); the v1 document is frozen at [`memory/AGENTS.v1.md`](memory/AGENTS.v1.md) as the rollback target.
- The v2 CLI is `python -m cyberos`; legacy umbrella at `../runtime/tools/cyberos` still works via the schema-version shim ([`../runtime/lib/brain_writer_shim.py`](../runtime/lib/brain_writer_shim.py)).
- Per-batch history (1–27 and the 2026-05 rebuild) lives in [`memory/CHANGELOG.md`](memory/CHANGELOG.md).

## Docx ↔ markdown round-trip

`PRD.md` and `SRS.md` are the source of truth; `.docx` outputs are regenerated via pandoc:

```bash
cd docs
make docx     # md → docx (after editing the markdown)
make md       # docx → md (after a Word user applied changes)
```

Requires `pandoc` (`brew install pandoc` / `apt-get install pandoc`). See [`Makefile`](Makefile) for the exact commands.
