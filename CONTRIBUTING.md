# Contributing to CyberOS

> Welcome. This file is the new-contributor smoke test. If you follow it end-to-end and any step fails, that's a bug — file an issue or send a PR.

## What CyberOS is (3-sentence orientation)

CyberOS is CyberSkill's AI-native internal operations platform, multi-tenant from day 1, with three architectural bets: agent parity (every human task is MCP-callable under same RBAC), CUO as named brand (10 C-suite skills in one persona), BRAIN as universal memory (three layers: filesystem `.cyberos-memory/` at the edge, vector+graph in the centre, S3 archival cold). CyberSkill is the only tenant through Phase 3; external launch is Phase 4. Layer 1 (`.cyberos-memory/`) ships today; Layers 2 & 3 land with the BRAIN module at P0+.

Read [`docs/CyberOS-AGENTS.README.md`](docs/CyberOS-AGENTS.README.md) first (the friendly on-ramp). Read [`docs/CyberOS-AGENTS.md`](docs/CyberOS-AGENTS.md) when you need the exact wording.

## Setup (5 minutes)

```bash
git clone <repo-url> cyberos
cd cyberos

# Install Python deps
pip install pyyaml rfc8785 --break-system-packages

# Smoke-test the operator CLI
python3 runtime/tools/cyberos --version
python3 runtime/tools/cyberos status
python3 runtime/tools/cyberos help

# Verify the BRAIN is healthy
python3 runtime/tools/cyberos verify
```

If `cyberos status` reports `0 CRITICAL / 0 WARN` you're set. Otherwise, run `cyberos doctor` and follow the surfaced findings.

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

For all new prose in `docs/CyberOS-AGENTS*.md`, `memories/decisions/`, `memories/refinements/`, `memories/facts/`:

- **No em dashes (—) or en dashes (–)** — use commas, parens, or sentence rewrite
- **No AI vocabulary:** delve, crucial, robust, comprehensive, nuanced, multifaceted, furthermore, moreover, additionally, pivotal, landscape, tapestry, underscore, foster, showcase, intricate, vibrant, fundamental, significant
- **Lead with the point.** Name files, line numbers, commands, outputs.
- **Builder-to-builder tone**, not consultant-to-client.

Lint: `python3 runtime/tools/cyberos voice --summary` (or via CI: `.github/workflows/voice-and-consistency.yml`).

CHANGELOG is exempt (descriptive — may quote LLM outputs verbatim).

## Running tests

```bash
# Validator self-test (16 fixtures)
python3 runtime/tools/cyberos_validate.py --self-test

# Denylist regression suite (24 fixtures)
python3 runtime/tests/denylist/test_denylist.py

# Cross-doc consistency (§-refs + DEC-refs)
python3 runtime/tools/cyberos doc-consistency

# Chain integrity (Merkle LINK check)
python3 outputs/brain_writer.py verify
```

CI runs `voice + doc-consistency + validator` on every PR touching `docs/CyberOS-AGENTS*.md` or `runtime/tools/`.

## Adding a new validator

1. Add the check function to `runtime/tools/cyberos_validate.py`
2. Add a fixture under `runtime/tools/tests/vectors/` that exercises it (positive + negative case)
3. Run `cyberos verify --self-test` — your fixture must pass
4. If the new check rejects previously-valid input → must go through §0.5 protocol upgrade + 3-release deprecation cycle per README Part 8 "additive only" rule

## Reporting issues

- **Validator false positive** — file with the rejected input + expected outcome
- **Validator false negative** — file with the input that should have been rejected
- **Drift** — `cyberos status` flags it; share the dashboard output
- **Protocol confusion** — quote AGENTS.md verbatim with line numbers

## Project layout

```
.cyberos-memory/                # the BRAIN (Layer 1)
├── manifest.json               # root pointer (§6)
├── company/                    # locked-decisions, values, glossary
├── module/<name>/              # per-capability/module memories
├── member/<id>/                # per-person; <id>/private/ is subject-only
├── client/<id>/
├── project/                    # this project's working memory
├── persona/<role>.md
├── memories/{decisions,refinements,facts,people,projects,preferences,drift}/
├── meta/
│   ├── templates/              # Aspect 4.1 — DEC.md, REF.md, FACT.md, ...
│   ├── protocol-history/       # verbatim AGENTS.md archives (§0.5)
│   │   └── INDEX.md            # Aspect 13.4 — bundle → SHA mapping
│   └── health/                 # §8.7 self-audit reports
├── audit/<YYYY-MM>.jsonl       # append-only Merkle ledger
├── conflicts/
├── exports/
└── index/                      # regenerable search index

docs/
├── CyberOS-AGENTS.md           # canonical protocol (load on demand)
├── CyberOS-AGENTS-CORE.md      # 10K-token normative subset (load every session)
├── CyberOS-AGENTS.README.md    # this guide's full version
├── CyberOS-AGENTS.CHANGELOG.md # day-by-day record of protocol changes
├── CyberOS-PRD.docx            # product requirements doc (binary)
├── CyberOS-PRD.CHANGELOG.md
├── CyberOS-SRS.docx            # system requirements spec (binary)
└── CyberOS-SRS.CHANGELOG.md

runtime/
├── tools/
│   ├── cyberos                 # operator umbrella binary (Aspect 1.1)
│   ├── cyberos_validate.py     # validator + self-test
│   ├── cyberos_doctor.py       # recovery + MAINTENANCE-mode repair
│   ├── cyberos_index.py        # SQLite search index (Stage 3)
│   ├── cyberos_export.py       # deterministic export (Stage 4)
│   ├── cyberos_encrypt.py      # at-rest encryption + Shamir (Stage 5)
│   ├── canonical_sha.py        # §0.5 canonical SHA computation
│   ├── voice_check.py          # em-dash + AI-vocab linter (Aspect 7.2)
│   ├── cyberos_show.py         # memory browser
│   ├── cyberos_onboard.py      # new-contributor wizard (Aspect 8.1)
│   ├── cyberos_analytics.py    # local-only usage analytics (Aspect 11.1)
│   ├── cyberos_add.py          # interactive memory wizard (Aspect 1.2)
│   ├── extract_agents_core.py  # AGENTS-CORE.md regen
│   └── tests/                  # 16 validator fixtures
├── hooks/
│   ├── gateguard.py            # PreToolUse 3-stage DENY/FORCE/ALLOW (Aspect 5.1)
│   └── refinement_candidates.py # Stop-hook §0.4 auto-detection (Aspect 3.1)
└── tests/
    ├── denylist/               # §9.3 regression suite (Aspect 5.5)
    ├── fuzz/                   # content-gate fuzz tests (Aspect 5.6 — placeholder)
    └── refinements/            # per-REF capability+regression evals (Aspect 3.2)

tours/                          # CodeTour walkthroughs (Aspect 7.4)
├── onboarding.tour
├── refinement-loop.tour
├── incident-response.tour
├── protocol-upgrade.tour
└── security-audit.tour

outputs/
├── brain_writer.py             # canonical writer — append audit + atomic-write per §4.4
├── staged-memories/            # cyberos add stages here before commit
├── doctor/                     # repair op logs
└── refinements/                # in-progress REF drafts before promotion

.github/workflows/
└── voice-and-consistency.yml   # CI gate: voice + doc-consistency + validator
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

For protocol questions: read `docs/CyberOS-AGENTS.README.md` Part 1-4 first.
For implementation questions: check `tours/onboarding.tour` then ask in chat.
For refinement proposals: follow `tours/refinement-loop.tour`.

## License

[Add license here once decided.]
