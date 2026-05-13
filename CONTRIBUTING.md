# Contributing to CyberOS

> Welcome. This file is the new-contributor smoke test. If you follow it end-to-end and any step fails, that's a bug — file an issue or send a PR.

## What CyberOS is (3-sentence orientation)

CyberOS is CyberSkill's AI-native internal operations platform, multi-tenant from day 1, with three architectural bets: agent parity (every human task is MCP-callable under same RBAC), CUO as named brand (10 C-suite skills in one persona), BRAIN as universal memory (three layers: filesystem `.cyberos-memory/` at the edge, vector+graph in the centre, S3 archival cold). CyberSkill is the only tenant through Phase 3; external launch is Phase 4. Layer 1 (`.cyberos-memory/`) ships today; Layers 2 & 3 land with the BRAIN module at P0+.

Read [`memory/README.md`](memory/README.md) for the module quick start, then [`memory/docs/AGENTS.md`](memory/docs/AGENTS.md) for the protocol RFC.

## Setup (5 minutes)

```bash
git clone <repo-url> cyberos
cd cyberos

# Install Python deps + register the `cyberos` console script
cd memory
pip install -e .

# Smoke-test the CLI
cyberos --store ../.cyberos-memory state
cyberos --store ../.cyberos-memory doctor
```

If `cyberos doctor` reports zero errors you're set.

## Adding your first memory

```bash
# Interactive wizard — fills templates from .cyberos-memory/meta/templates/
python3 runtime/tools/cyberos_add.py FACT
# answer prompts: slug, classification, authority, tags, source_ref, ...

# Or skip the wizard:
python3 runtime/tools/cyberos_add.py FACT --slug my-fact \
    --classification public --authority human-confirmed \
    --tags fact,first-contribution --prov-source-ref "my notes"
```

The wizard:
1. Reads `.cyberos-memory/meta/templates/FACT.md` (Aspect 4.1 templates)
2. Fills variables (UUID7, ts, subject ID, next NNN per bucket monotonic)
3. Stages to `outputs/staged-memories/`
4. Asks for confirmation
5. Invokes `outputs/brain_writer.py write` to commit with audit row

## Proposing a §0.4 protocol refinement

Per AGENTS.md §0.4 standing rule: every memory issue MUST trigger a refinement proposal in the same response.

Template flow (`docs/CyberOS-AGENTS.README.md` Part 8 covers the full propose-adopt-record cycle):

1. **Observe** the friction — what was rejected, what looped, what got drift-flagged?
2. **Propose** — `cyberos_add.py REF --slug <descriptive>` with:
   - Trigger (what observation)
   - Tier (1/2/3) — see template
   - AGENTS.md section to amend
   - Exact prose to insert
   - Capability eval (what new behavior?)
   - Regression eval (what doesn't break?)
3. **Adopt** — user says *"approve protocol upgrade to sha256:<X>"* in chat
4. **Record** — `brain_writer.py protocol-upgrade` archives prior AGENTS.md, updates pin, appends CHANGELOG + README + REF entries per §0.6

Auto-detected candidates land in `.cyberos-memory/memories/drift/<date>-refinement-candidate-*.md` from the `runtime/hooks/refinement_candidates.py` Stop-hook (Aspect 3.1). Review weekly.

## Voice standard (gstack /codex)

For all new prose in `memory/docs/*.md`, `memories/decisions/`, `memories/refinements/`, `memories/facts/`:

- **No em dashes (—) or en dashes (–)** — use commas, parens, or sentence rewrite
- **No AI vocabulary:** delve, crucial, robust, comprehensive, nuanced, multifaceted, furthermore, moreover, additionally, pivotal, landscape, tapestry, underscore, foster, showcase, intricate, vibrant, fundamental, significant
- **Lead with the point.** Name files, line numbers, commands, outputs.
- **Builder-to-builder tone**, not consultant-to-client.

Lint: `python3 memory/tools/voice_check.py --summary` (or via CI: `.github/workflows/voice-and-consistency.yml`).

CHANGELOG is exempt (descriptive — may quote LLM outputs verbatim).

## Running tests

```bash
# Full memory-module pytest suite
cd memory && python -m pytest tests/ -q

# Schema-drift check (regenerate if it fails)
cd memory && python tools/cyberos_generate_schema.py --check --out docs/memory.schema.json

# Doctor (every invariant must pass on the BRAIN store)
cd memory && python -m cyberos --store ../.cyberos-memory doctor
```

CI runs voice + schema-drift + the full pytest suite on every PR touching `memory/**`.

## Reporting issues

- **Validator false positive** — file with the rejected input + expected outcome
- **Validator false negative** — file with the input that should have been rejected
- **Drift** — `cyberos status` flags it; share the dashboard output
- **Protocol confusion** — quote AGENTS.md verbatim with line numbers

## Project layout

```
.cyberos-memory/                # the BRAIN (gitignored — local tenant state)
├── manifest.json               # root pointer (§6)
├── audit/<NNN>.binlog          # append-only audit ledger (v2)
├── company/, module/, member/, client/, project/, persona/
├── memories/{decisions,refinements,facts,people,projects,preferences,drift}/
├── meta/{templates,protocol-history,health}/
├── conflicts/, exports/, index/

memory/                         # the memory module
├── README.md                   # quick start
├── pyproject.toml              # registers `cyberos` console script
├── cyberos/                    # Python package
│   ├── core/                   # writer, walker, reader, lock, ops, ...
│   └── __main__.py             # single CLI entrypoint
├── docs/
│   ├── AGENTS.md               # protocol RFC (canonical source of truth)
│   ├── memory.schema.json      # generated schema (don't hand-edit)
│   ├── memory.invariants.yaml  # invariants enforced by cyberos doctor
│   ├── PROPOSAL.md, EVOLUTION.md, INTEROP.md, CHANGELOG.md, README.md
├── tools/                      # schema generator, voice linter, encrypt, benchmark
├── tests/                      # pytest suite
├── bench/                      # throughput / cold-CLI / determinism benchmarks
└── scripts/                    # install.sh, automation, pre-commit hook

docs/                           # project-level docs only
├── prd/                        # PRD.md + PRD.docx + CHANGELOG.md
└── srs/                        # SRS.md + SRS.docx + CHANGELOG.md

runtime/                        # non-memory runtime (skill_runners, hooks, mcp, ...)

.github/workflows/
└── voice-and-consistency.yml   # CI gate: voice + schema-drift + pytest + doctor
```

## Code style

Python 3.10+. PEP 8 with these exceptions:
- 100-char line limit (not 79)
- Type hints on public functions only
- Docstrings: one-line summary + optional details

Per `coding-standards` skill: avoid AI vocabulary in docstrings + comments too.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` new feature (Layer-1 aspect)
- `fix:` bug fix
- `docs:` docs only
- `chore:` housekeeping (tests, ci, infra)
- `refactor:` no behavior change

Bundle naming (post-Bundle-Q): `bundle <letter>: <theme>`. See CHANGELOG for examples.

## Questions

For protocol questions: read `docs/CyberOS-AGENTS.README.md` Part 1-4 first. For implementation questions: check `tours/onboarding.tour` then ask in chat. For refinement proposals: follow `tours/refinement-loop.tour`.

## License

[Add license here once decided.]
