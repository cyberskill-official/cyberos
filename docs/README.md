# `docs/` — All documentation

CyberOS keeps every kind of documentation under this single tree. One README per folder, one folder per concern.

| Folder | Purpose | Entry point |
| --- | --- | --- |
| [`memory/`](memory/) | AGENTS protocol — the memory layer rules, schema, source-tier system, audit ledger | [`memory/README.md`](memory/README.md) |
| [`skills/`](skills/) | Skills layer — CPO/CTO chain skills + chain orchestrator + host adapters | [`skills/README.md`](skills/README.md) |
| [`contracts/`](contracts/) | Versioned artefact schemas (`feature_request@1`, `task@1`, `prd@1`, `srs@1`, …) | [`contracts/README.md`](contracts/README.md) |
| [`prd/`](prd/) | Product Requirements Document for CyberOS itself | [`prd/README.md`](prd/README.md) |
| [`srs/`](srs/) | System Requirements Specification | [`srs/README.md`](srs/README.md) |
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

- The single source of truth for the AGENTS protocol is [`memory/AGENTS.md`](memory/AGENTS.md). The compact "AGENTS-CORE" variant was removed in Batch 27 (2026-05-12).
- The umbrella CLI is at `../runtime/tools/cyberos`; subcommand reference lives in [`memory/README.md` Part 27](memory/README.md).
- Per-batch history (1–27) lives in [`memory/CHANGELOG.md`](memory/CHANGELOG.md).
