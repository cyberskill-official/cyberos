# CyberOS

AI-native internal operations platform for CyberSkill (Vietnam-based software consultancy). Memory layer + skills layer + an opinionated chain that turns natural-language pitches into addressable, assignable tasks.

> "Turn Your Will Into Real" — the CyberSkill slogan and the design principle for this codebase.

## Top-level layout

```text
cyberos/
├── README.md              ← you are here
├── AGENTS.md              ← symlink → docs/memory/AGENTS.md (single source of truth)
├── CLAUDE.md              ← @-reference → docs/memory/AGENTS.md
├── CONTRIBUTING.md        ← how to land changes
├── docs/                  ← ALL documentation (each subfolder has its own README.md)
│   ├── memory/            ← AGENTS protocol — single source of truth
│   ├── skills/            ← skills layer manual (CPO/CTO chain)
│   ├── contracts/         ← versioned artefact schemas (feature_request@1, task@1, …)
│   ├── prd/               ← Product Requirements Doc (PRD.docx + CHANGELOG.md)
│   ├── srs/               ← System Requirements Spec (SRS.docx + CHANGELOG.md)
│   └── tours/             ← 10 guided walkthroughs (.tour files)
├── runtime/               ← ALL code (each subfolder has its own README.md)
│   ├── tools/             ← cyberos CLI + per-subcommand modules (63+ subcommands)
│   ├── skill_runners/     ← LLM-driven skill runners (BaseSkillRunner framework)
│   ├── mcp/               ← read-only MCP server for the BRAIN
│   ├── hooks/             ← pre/post-write hooks
│   ├── completions/       ← shell tab-completion
│   ├── lib/               ← shared scripts (brain_writer.py, apply-bundle-Q.sh, cleanup-host.sh)
│   ├── starter/           ← bootstrap scaffolds for new projects
│   ├── migrations/        ← BRAIN schema migration scripts
│   └── tests/             ← integration tests + skill fixtures
├── planning/              ← per-project FRs (auto-generated project-index.md per project)
└── .cyberos-memory/       ← THE BRAIN (gitignored — local tenant state)
                              includes cache/, staging/, refinements/ — generated state lives here
                              so a single ignore rule covers everything
```

**Four top-level folders + four files.** No `outputs/`, no `var/`, no `migrations/`, no `tours/`, no `AGENTS-CORE.md` — all consolidated. Every functional folder has exactly one `README.md` entry point.

## Where to start

- **Reading the protocol** — start with [`docs/memory/README.md`](docs/memory/README.md) Parts 1–12 (32-part operator manual).
- **Running the chain** — `cyberos chain run --pitch "your idea here" --profile solo` (writes `planning/<slug>/FR-001-*.md`).
- **Authoring a new memory** — `cyberos add <TYPE>` (delegates to `runtime/lib/brain_writer.py`).
- **Daily health check** — `cyberos verify` + `cyberos doctor`.
- **Browsing what's in the BRAIN** — `cyberos status --weekly`, or open the audit dashboard at `.cyberos-memory/cache/audit-site/index.html` (regenerate with `cyberos audit publish`).

## The three layers

```mermaid
flowchart TD
    A[Memory layer<br/>docs/memory/AGENTS.md] -.-> B[Skills layer<br/>docs/skills/README.md]
    B -.-> C[Runtime<br/>runtime/tools/cyberos]
    C --> D[Artefacts<br/>planning/<project>/FR-*.md]
    A -.-> D
```

1. **Memory layer (`docs/memory/`)** — the AGENTS protocol. Defines what a memory is, how the BRAIN is structured, the §x.y rules every tool must respect, source-tier system, audit ledger, sync-class model.
2. **Skills layer (`docs/skills/`)** — single-doc operator manual covering the 11 chain skills (CPO/CTO personas), the `cyberos chain` umbrella, host adapters, and the chain orchestrator.
3. **Runtime (`runtime/`)** — Python tools that implement the protocols. The umbrella binary is [`runtime/tools/cyberos`](runtime/tools/cyberos) with 63+ subcommands.

## The chain in one diagram

```text
spec (pitch / --spec-file / --prd + --srs)
   ↓
cyberos chain run --profile solo --with-llm
   ↓
fr-with-tasks (collapsed FR + impl-plan)
   ↓
fr-audit (14 INVARIANT checks)
   ↓
planning/<slug>/
  ├── FR-001-*.md       (one user-story = one FR file)
  │   ├── frontmatter   (registry + task index — slim, 25 lines)
  │   └── body          (Problem / Users / Success metrics / Scope / Risks
  │                      / per-task H2 sections with task-meta YAML fences)
  ├── project-index.md  (auto-generated dashboard)
  └── chain-manifest.json  (state for resume / status)
```

Each task has `id` = `FR-NNN-T-MM`, optional subtasks `FR-NNN-T-MM-ST-XX`, sizing (S/M/L/XL), `assignable_to` (human / ai-agent / either), and a concrete `acceptance_test` (shell command OR assertion).

## Key commands

| Goal | Command |
| --- | --- |
| Start a new project from pitch | `cyberos chain run --pitch "..." --profile solo` |
| Start with separate PRD + SRS | `cyberos chain run --pitch "..." --prd p.md --srs s.md` |
| List all FRs | `cyberos fr list` |
| Render task DAG | `cyberos fr task-graph FR-001` |
| Migrate legacy FR to new shape | `cyberos fr-migrate path/to/FR.md --in-place` |
| Regenerate project dashboard | `cyberos project-index planning/<slug>/` |
| BRAIN health | `cyberos verify && cyberos doctor` |
| Find conflicts | `cyberos conflicts` |
| See recent activity | `cyberos status --weekly` |

## Recent shape changes (2026-05-12 sprint)

- **Batch A** — `feature_request@1` reshaped: slim frontmatter + body H2 task sections + fenced `task-meta` YAML. Much more readable than the legacy single-YAML form.
- **Batch B** — optional `subtasks` for `task@1`: `FR-NNN-T-MM-ST-XX` IDs, rendered as sub-nodes in `cyberos fr task-graph`.
- **Batch C** — `cyberos chain run` accepts `--prd` and `--srs` as separate inputs (alongside `--spec-file`).
- **Batch D** — chain auto-generates `project-index.md` (project dashboard) in each `planning/<slug>/` folder; preserves a `<!-- BEGIN human-edited -->` block across regenerations.

- **Batch 25** — folder cleanup, part 1: PRD/SRS docs into dedicated `docs/prd/` + `docs/srs/`; memory protocol consolidated under `docs/memory/`.
- **Batch 26** — folder cleanup, part 2: `outputs/` split into `runtime/lib/` + `runtime/starter/` + `var/`. `migrations/` → `runtime/migrations/`. `tours/` → `docs/tours/`.
- **Batch 27** — single-source-of-truth pass: `AGENTS-CORE.md` removed (one canonical `AGENTS.md`). `var/` removed (generated state moves into the BRAIN cache, gitignored). Every functional folder now has exactly one `README.md` entry point.

See [`docs/memory/CHANGELOG.md`](docs/memory/CHANGELOG.md) for the full batch history (27 batches, 2026-05-04 onward).

## Identifier conventions

| Pattern | Meaning |
| --- | --- |
| `FR-NNN` | Feature Request (user story) |
| `FR-NNN-T-MM` | Task within an FR (ticket) |
| `FR-NNN-T-MM-ST-XX` | Subtask within a task |
| `DEC-NNN` | Decision recorded in `memories/decisions/` |
| `FACT-NNN` | Locked fact recorded in `memories/facts/` |
| `PREF-NNN` | Operator preference in `memories/preferences/` |
| `PERSON-NNN` | Person profile in `memories/people/` |

## Cross-reference cheat sheet

- **Protocol authority:** `docs/memory/AGENTS.md` (full) or `AGENTS.md` symlink (compact, per-session).
- **Skill catalog:** `docs/skills/README.md` — Parts 1–30 cover authoring, runtime, host adapters, chain orchestrator, manual workflow.
- **Contracts:** `docs/contracts/<id>/CONTRACT.md` + `template.md` per contract.
- **CLI reference:** `docs/memory/README.md` Part 27.
- **Per-aspect manual:** `docs/memory/README.md` Part 26 (88 aspects + tier amplifiers).

## License + ownership

Internal to CyberSkill (CYBERSKILL SOFTWARE SOLUTIONS CONSULTANCY AND DEVELOPMENT JOINT STOCK COMPANY). Founder: Stephen Cheng (Trịnh Thái Anh, zintaen@gmail.com).
