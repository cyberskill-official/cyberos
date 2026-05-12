# docs/memory/ — BRAIN protocol + operator manual

Everything about the CyberOS **memory layer** (the `.cyberos-memory/` BRAIN, its protocol, its operator surface) lives in this folder. The skills layer (CPO/CTO chain, `cyberos chain`, fr-with-tasks, etc.) is documented in `docs/skills/`.

## Reading order

| File | Purpose | When to read |
| --- | --- | --- |
| **[README.md](README.md)** | On-ramp + 32-part operator manual + skills cross-reference | First read; recurring reference |
| **[AGENTS.md](AGENTS.md)** | The protocol itself (1,241 lines, ~108 KB) | Authoritative reference when implementing a rule |
| **[AGENTS-CORE.md](AGENTS-CORE.md)** | 42 KB compact version of AGENTS.md (regenerable) | Per-session load via symlink |
| **[CHANGELOG.md](CHANGELOG.md)** | Daily landing log; every batch (1–23) recorded line-by-line | Audit trail; "what changed today" |
| **[PRD.CHANGELOG.md](PRD.CHANGELOG.md)** | Notes on PRD-side impact per batch | When you next edit PRD.docx |
| **[SRS.CHANGELOG.md](SRS.CHANGELOG.md)** | Notes on SRS-side impact per batch | When you next edit SRS.docx |

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

- **2026-05-12**: docs reorganisation. Memory-related docs moved from `docs/CyberOS-*.md` into this folder. Originals at `docs/CyberOS-*.md` are now redirect stubs (sandbox can't unlink; remove with `rm docs/CyberOS-*.md` on the host filesystem when convenient).
- Skills-layer docs deliberately separate, under `docs/skills/`.

## Cross-references

- **Skills layer:** `../skills/README.md` (single-doc manual after the 2026-05-12 skills-folder consolidation)
- **Contracts:** `../contracts/` (task@1, chain_manifest@1, feature_request@1, prd@1, srs@1, impl_plan@1, project_brief@1, nats-subjects)
- **Runtime tools:** `../../runtime/tools/cyberos` (umbrella binary, 63 subcommands)
- **The BRAIN itself:** `../../.cyberos-memory/`
