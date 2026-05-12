# docs/memory/ — BRAIN protocol + operator manual

Everything about the CyberOS **memory layer** (the `.cyberos-memory/` BRAIN, its protocol, its operator surface) lives in this folder. The skills layer (CPO/CTO chain, `cyberos chain`, fr-with-tasks, etc.) is documented in `docs/skills/`.

## Reading order

| File | Purpose | When to read |
| --- | --- | --- |
| **[README.md](README.md)** | On-ramp + 32-part operator manual + skills cross-reference | First read; recurring reference |
| **[AGENTS.md](AGENTS.md)** | The protocol itself (1,241 lines, ~108 KB) | Authoritative reference when implementing a rule |
| **[AGENTS-CORE.md](AGENTS-CORE.md)** | 42 KB compact version of AGENTS.md (regenerable) | Per-session load via symlink |
| **[CHANGELOG.md](CHANGELOG.md)** | Daily landing log; every batch (1–25) recorded line-by-line | Audit trail; "what changed today" |

## Sister folders under `docs/`

The PRD and SRS each got their own folder in the 2026-05-12 cleanup — they are design docs, not memory-protocol docs, so they sit alongside this folder rather than inside it:

| Folder | What's there |
| --- | --- |
| [`../prd/`](../prd/) | `PRD.docx` + `CHANGELOG.md` + a small `README.md`. CyberOS Product Requirements. |
| [`../srs/`](../srs/) | `SRS.docx` + `CHANGELOG.md` + a small `README.md`. CyberOS System Requirements. |
| [`../skills/`](../skills/) | Single-doc operator manual for the skills layer (Parts 1–30). |
| [`../contracts/`](../contracts/) | Versioned artefact schemas: `feature_request@1`, `task@1`, `project_brief@1`, `prd@1`, `srs@1`. |

## How to use this folder

- **Day one with the BRAIN:** read README.md Parts 1–12.
- **Adding a new memory:** consult AGENTS.md §5.1 (schema) and run `cyberos add <TYPE>`.
- **Daily verify:** README.md Part 28 → "Daily verify" workflow.
- **Operator surface (33+ subcommands):** README.md Part 27 — full CLI reference.
- **Per-aspect detail (88 + tier amplifiers):** README.md Part 26.
- **Skills layer cross-reference:** README.md Part 32.

## Symlink recipe (per CyberOS PRD §1.4)

For a new project that consumes this BRAIN protocol:

```bash
cd /path/to/your-project
ln -s /path/to/cyberos/docs/memory/AGENTS-CORE.md AGENTS.md
ln -s /path/to/cyberos/docs/memory/AGENTS-CORE.md CLAUDE.md
```

Symlink to `AGENTS-CORE.md` (compact) for per-session load. Agents that need the full reference (validator, doctor, §0.5 upgrades) follow the in-doc pointer to `AGENTS.md`.

## Folder history

- **2026-05-12 (Batch 24)** — memory-protocol docs moved from `docs/CyberOS-*.md` into this folder.
- **2026-05-12 (Batch 25)** — PRD + SRS docs (and their CHANGELOGs) moved out of `docs/` top-level into dedicated `docs/prd/` and `docs/srs/` folders so each design doc travels with its own changelog. Skills-layer docs were also consolidated into a single anchor — see `../skills/README.md`.
- Legacy redirect stubs at `docs/CyberOS-AGENTS*.md` remain (sandbox couldn't unlink); remove with `rm docs/CyberOS-*.md` on the host filesystem when convenient.

## Cross-references

- **Skills layer:** `../skills/README.md` (single-doc manual after the 2026-05-12 skills-folder consolidation)
- **Contracts:** `../contracts/` (task@1, chain_manifest@1, feature_request@1, prd@1, srs@1, impl_plan@1, project_brief@1, nats-subjects)
- **Runtime tools:** `../../runtime/tools/cyberos` (umbrella binary, 63 subcommands)
- **The BRAIN itself:** `../../.cyberos-memory/`
