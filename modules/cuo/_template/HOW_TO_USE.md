# `cuo/_template/` — canonical scaffolds for new personas + workflows

This directory ships the reusable templates every new CUO persona and workflow copies from. Two scaffolds:

| Scaffold | Path | Purpose |
|---|---|---|
| Persona | `_template/persona/README.md` | The 9-block schema (per C-Suite Reference §4) that every new persona's `cuo/<slug>/README.md` SHALL render. |
| Workflow | `_template/workflow/<workflow-template>.md` | The skill-chain frontmatter + operator-facing body that every new `cuo/<slug>/workflows/<workflow-name>.md` SHALL follow. |

## Step-by-step: add a new persona

### 1. Pick the slug

Use the disambiguation matrix in `MODULE.md` §2. For colliding acronyms, suffix with the meaning:

- `cro-revenue` (Chief Revenue Officer) vs `cro-risk` (Chief Risk Officer) vs `cro-restructuring`.
- `cco-commercial` / `cco-compliance` / `cco-customer` / `cco-communications`.
- `cdo-data` / `cdo-digital` / `cdo-diversity`.
- `cso-strategy` / `cso-security` / `cso-sustainability` / `cso-sales`.
- `cpo-people` / `cpo-product` / `cpo-privacy` / `cpo-procurement`.
- `cao-admin` / `cao-accounting`.
- `cio-information` / `cio-investment`.
- `clo-legal` / `clo-learning`.

Unambiguous roles use the bare acronym (`ceo`, `cto`, `cfo`, `cmo`, `chro`, `caio`, `ciso`, `cgo`, `cxo`).

For roles where the acronym isn't standard, use the full name in kebab-case (`chief-of-staff`, `chief-architect`, `chief-medical-officer`, etc.).

### 2. Copy the scaffold

```bash
cd cuo
cp -r _template/persona <persona-slug>
```

### 3. Fill in the 9 blocks

Open `cuo/<persona-slug>/README.md`. The scaffold has every block as a header with placeholder bullets. For each block, cross-reference the role's profile in `../docs/The C-Suite Reference.md` §5.

The nine blocks (per §4):

1. **Identity & scope** — Full disambiguated title; reports-to / reports-in; stage prevalence (per §3 matrix); one-sentence scope statement.
2. **Information inputs** — Dashboards, reports, market intel, customer signals.
3. **Stakeholder inputs** — CEO / board mandates, peer-C-suite asks, regulator signals.
4. **Resource inputs** — Budget envelope, headcount, tooling.
5. **Outputs** — Strategic, operational, communication, team.
6. **Cadence** — Daily / weekly / monthly / quarterly / annual rhythms.
7. **KPIs** — 3-5 quantitative metrics with target ranges.
8. **Audit criteria** — Quantitative gates, qualitative rubric, failure modes (universal 6 from §6 + role-specific).
9. **Tools & stack** — Categories + named exemplars.

### 4. Add the persona to `MODULE.md` §4 status table

Flip its row from `planned` to `shipped` and record the workflow count.

### 5. Build workflows (next section)

A persona with zero workflows is non-actionable. Build at least one workflow per major output type from block 5.

## Step-by-step: add a new workflow

### 1. Copy the workflow scaffold

```bash
cd cuo/<persona-slug>/workflows/    # create the folder if it doesn't exist
cp ../../_template/workflow/<workflow-template>.md <workflow-slug>.md
```

### 2. Fill the frontmatter

The frontmatter declares:

- `workflow_id` — `<persona-slug>/<workflow-slug>`.
- `purpose` — one-line.
- `cadence` — when this workflow runs.
- `inputs` — list of `{name, source, format}`.
- `outputs` — list of `{name, format, recipient}`.
- `skill_chain` — ordered list of `{step, skill, inputs_from, outputs_to}`.
- `escalates_to` — other personas this workflow escalates to and when.
- `consults` — other personas this workflow consults and when.
- `audit_hooks` — what gets logged to the BRAIN audit chain.

### 3. Identify any planned skills

For each step in `skill_chain`, check if the skill exists in `skill/MODULE.md` §4. If not, mark the step as `skill: planned:<skill-name>` and add a row to `cuo/docs/NEEDED_SKILLS.md`.

### 4. Fill the body

The body documents the workflow for operators: when to invoke, how to invoke (the CUO routing trigger language), expected duration, failure modes per step, operator-side decisions (where the chain pauses for human input).

### 5. Increment workflow count in `MODULE.md` §4

For the parent persona's row, bump the workflows count.

## Anti-patterns — do not do these

- **Do not** create a persona without the 9-block schema fully populated. Empty blocks signal a half-baked persona.
- **Do not** create a workflow that doesn't chain SKILL module skills. Workflows orchestrate skills; they don't reimplement them.
- **Do not** invent new skills inline in a workflow. New skills go through `skill/_template/HOW_TO_USE.md`. CUO surfaces gaps via `cuo/docs/NEEDED_SKILLS.md`.
- **Do not** stash workflows outside `cuo/<persona-slug>/workflows/`. The flat-with-workflows-subfolder convention is what the runtime orchestrator (when shipped) scans.
- **Do not** edit `_template/` for persona-specific needs. If a future persona class needs a different scaffold, propose a new template directory (e.g. `_template-collegium/` for multi-persona collaborative roles).

## Cross-references

- `../MODULE.md` — the canonical persona catalog this template feeds into.
- `../docs/AGENTS.md` — protocol normativity.
- `../docs/SPEC.md` — contract summary.
- `../docs/ROUTING.md` — how the runtime orchestrator selects persona → workflow → skill chain.
- `../../docs/The C-Suite Reference.md` — the source atlas. §4 is the 9-block schema; §5 is the role profiles.
- `../../skill/MODULE.md` — the source of skill names referenced in `skill_chain:`.
