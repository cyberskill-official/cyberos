# MANUAL_WORKFLOW.md — running the chain by hand, today

> **Audience.** Stephen, on his next project, before the runtime exists. **Scope.** Phase A (Requirements Discovery) → Phase G (Implementation Plan) — the Requirements + Planning halves of the daily workflow. Execute + QA come later. **Trigger model.** Stephen is the supervisor: he loads SKILL.md content into a fresh chat session, follows it, reviews output, and runs the next skill. **Time budget.** A full chain on a small project: ~2-4 hours. Lean profile: ~45 min.

> **Why this doc exists.** `cyberos/docs/skills/` is design-complete but the runtime is gated until `runtime_v0_3_0`. This RUNBOOK is the bridge — it tells you exactly which file to paste, what to expect, what to do when the audit fails, and how to handle a refinement proposal. Pin this doc to your workspace.

---

## Two modes — pick one

**Automated mode (★ recommended)** — you give a pitch + answer HITL questions. The agent reads every SKILL.md, drives every interview, writes every artefact, runs every audit-fix loop, executes brain_writer.py, and routes between skills. **You never copy-paste a SKILL.md or run a command yourself.**

→ See **[CHAIN_ORCHESTRATOR.md](./CHAIN_ORCHESTRATOR.md)**. Pin the trigger phrase below.

```
Drive the CyberOS chain on this project. Read cyberos/docs/skills/CHAIN_ORCHESTRATOR.md and follow it.

Pitch: <one paragraph describing the project>
Project repo: <absolute path to the new project's directory>
Output dir: <where to save artefacts; default ./planning/<YYYY-MM-DD>-<slug>/>
Caller: human:<your-id>
Profile preference: <auto | lean | standard | full>   (default: auto)
```

The agent then announces phase transitions in chat, asks you HITL questions when needed (triage gating, section approvals, audit decisions, refinement proposals), and produces the full artefact tree at the output dir. Total user effort: the trigger + ~10-30 HITL answers. Total user effort *saved*: copy-pasting ~12 SKILL.md files + running ~30 brain_writer.py commands by hand.

**Manual mode** — you drive every step yourself. Useful when:
- The agent host doesn't auto-load AGENTS.md and you want full visibility.
- You're learning the chain and want to see each SKILL.md as you go.
- An audit fails in a way the orchestrator can't auto-recover from and you want to take over.
- You're on a degraded host (Claude.ai web / ChatGPT) where auto-orchestration isn't viable.

For manual mode, the 6-line version is:

```
1. Open your agent host on the new project's repo (Cowork / Claude Code / Cursor / Codex / Gemini CLI / OpenCode — see HOST_ADAPTERS.md).
2. Ensure AGENTS.md (or AGENTS-CORE.md) is loaded into the agent's context.
3. Paste cuo/cpo/requirements-discovery/STANDALONE_INTERVIEW.md → run the 20-question interview → save project_brief@1.md.
4. Paste cuo/cpo/chain-selector/SKILL.md → confirm the chain_profile (lean / standard / full).
5. For each skill the chain selected: paste its SKILL.md → run → review → save artefact → run the paired audit skill → fix HITL issues → next.
6. Stop after spec-to-impl-plan. You now have impl_plan@1 with vertical-slice issues.
```

Everything below in this doc is for **manual mode**. For automated mode, the orchestrator is the only doc you need to point the agent at.

---

## Host compatibility — what runs where

The workflow is **fully host-agnostic**. Any agent that can read text files, write markdown to disk, and (ideally) run a Python script can drive it. Capability requirements:

| Capability | Required for | Fallback if missing |
|---|---|---|
| **Read user-project files** | Loading SKILL.md / CONTEXT.md / artefacts | Manual paste of file contents |
| **Write markdown to disk** | Saving artefacts (`project-brief.md`, `prd-*.md`, etc.) | Agent emits markdown in chat; you copy-paste into local files |
| **Run shell / Python** | `brain_writer.py` audit-chain commands | Run brain_writer.py yourself in a separate terminal after each skill |
| **MCP tool calls** | `proj.create_issue`, `chat.review_request` (optional) | Manual ticket creation in Linear/Jira/GitHub; HITL questions answered in chat |
| **Subagent dispatch** | Running multiple skills in parallel (optional) | Sequential single-agent runs |

**Recommended hosts** (full-capability — workflow runs end-to-end without leaving the chat):

- **Claude Cowork (this app)** — file tools + bash + connected folders + MCP. Best fit for solo / small-team manual mode today.
- **Claude Code (CLI)** — file tools + bash + auto-loads AGENTS.md from project root + native hooks.
- **Cursor** — file tools + terminal + `.cursor/rules/` auto-load + MCP.
- **Codex CLI (OpenAI)** — file tools + shell + AGENTS.md auto-load.
- **Gemini CLI / OpenCode** — file tools + shell.

**Degraded hosts** (workflow runs but parts go manual):

- **Claude.ai web app** — sandboxed; agent emits markdown; you save files manually; you run brain_writer.py in terminal.
- **ChatGPT (with Code Interpreter)** — same shape as Claude.ai web; the Python sandbox can't reach your local filesystem.
- **Claude in Chrome / browser-only agents** — interview + artefact generation work; persistence is manual.

For per-host setup recipes (symlinks, plugin install, AGENTS.md loading, brain_writer access, MCP wiring), see **[HOST_ADAPTERS.md](./HOST_ADAPTERS.md)**.

---

## Prerequisites (one-time per project, ~10 minutes)

The exact commands depend on your host — see [HOST_ADAPTERS.md](./HOST_ADAPTERS.md) for per-host setup. The *abstract* steps are:

1. **Make AGENTS.md available to the agent.** The agent must load `cyberos/docs/CyberOS-AGENTS-CORE.md` at session start. Three options:
   - **Symlink** (Claude Code, Codex CLI auto-load): `ln -s <abs>/cyberos/docs/CyberOS-AGENTS-CORE.md AGENTS.md` at the new project's root.
   - **Plugin / rule file** (Cursor: `.cursor/rules/cyberos-memory.mdc`; Windsurf: `.windsurfrules`; Copilot: `.github/copilot-instructions.md`).
   - **Manual paste** (Claude.ai / ChatGPT): paste the contents at the start of each session.

   CORE.md is sufficient for the manual chain (full AGENTS.md is only needed for §0.5 protocol-upgrade flows, which won't fire here).

2. **Bootstrap the project's BRAIN.** First agent session detects `PRISTINE` state per §13.0 and silently auto-bootstraps `.cyberos-memory/` IF the host can write to disk. If it doesn't, paste *"bootstrap and continue"*. Hosts without filesystem access: run `python3 <path>/.brain_writer.py session-start <actor>` in your terminal manually and let the agent know the BRAIN is initialised.

3. **Initialise CONTEXT.md** (per Plan v1.1 / M2). Create at the new project's repo root:

   ```markdown
   # <Project name>

   ## Language

   <will fill in during Requirements Discovery>

   ## Relationships

   <will fill in during Requirements Discovery>

   ## Flagged ambiguities

   <will fill in during Requirements Discovery>
   ```

   This file is the project's shared vocabulary. Every chain skill will read and update it.

4. **Initialise `docs/adr/`** (per mattpocock's `grill-with-docs` pattern). Empty directory; ADRs land here only when the three conditions are met (hard-to-reverse + surprising-without-context + result-of-real-trade-off).

5. **(Optional) Initialise `.out-of-scope/`** (per Plan v1.1 / M1). Empty directory at the new project's repo root. When you Reject a refinement proposal, you'll write a file here. Anti-re-litigation.

---

## The chain end-to-end at a glance

```
human chat + BRAIN
  → A. requirements-discovery → project_brief@1
  → B. chain-selector → chain_plan (lean / standard / full)
  → C. prd-author → prd@1
  → D. [if standard|full] prd-audit → audited prd@1
  → E. [if full] srs-author → srs@1 → srs-audit → audited srs@1
  → F. fr-author → FR markdowns
  → G. fr-audit → audited FRs
  → H. [if standard|full] fr-to-tech-spec → tech_spec@1
  → I. spec-to-impl-plan → impl_plan@1 + (optionally) tickets in PROJ MCP
```

For lean profile, you skip D, E, H. For standard, you skip E. For full, you do everything.

---

## How to run a single skill manually (the meta-procedure)

Every skill follows the same shape. Memorise this; the per-step sections below just tell you which file to load and what to expect.

### Step ⓪ — Open a fresh agent session

A fresh session avoids context-pollution from prior skills. The §14 memory-update block at the previous skill's end is what carries state between sessions, plus the artefact you saved to disk.

### Step ① — Paste the skill's SKILL.md content

The skill's frontmatter declares `expects.standalone_interview_ref` (the question script for the human-driven flow) and `produces.human_summary_ref` (what to expect at the end). Read both files before starting; paste the SKILL.md body as the agent's instructions.

### Step ② — Provide the input envelope (manual form)

Skills expect an envelope per `expects.schema_ref`. In manual mode, you skip the envelope wire format and just answer the questions. Required fields you'll typically need:

- `output_dir` — where to save the artefact (e.g., `./planning/2026-05-12-<project-slug>/`)
- `caller_persona` — set to `human:stephen-cheng` for manual-mode runs
- `trace_id` — any unique ID; a UUID or `<date>-<slug>-<n>`

Optional fields the skill might ask for:

- `initial_prompt` (requirements-discovery only) — paste your draft requirements / pitch / scoping doc
- `client_id` (if commissioned work) — sets `client_visible: true`
- `target_release` — when does this need to ship
- `chain_to` — the next skill in the chain (auto-set per chain-selector output)

### Step ③ — Run the interview / generation

The skill will either ask you questions one at a time (interview mode) or generate an artefact directly (generation mode). Watch for:

- **HITL pauses.** The skill will say something like *"I need a human decision on X — option (a), (b), or (c)?"* Answer in chat. Do NOT skip with "use your best guess" — the audit-fix loop relies on `needs_human` being a real human answer.
- **Refinement proposals.** If the skill detects an anomaly (e.g., "you've now amended this brief 5 times — the discovery skill may be misaligned"), it will propose a refinement. Read it; either Accept, Accept-with-edits, Defer, or Reject. If Reject, write a `.out-of-scope/<topic>.md` entry per Plan v1.1 / M1.

### Step ④ — Review the output artefact

The skill writes a markdown file under your `output_dir`. Open it. Check:

- Frontmatter populated (the contract fields are non-empty).
- Body sections complete (no `TODO:` markers unless explicitly flagged).
- Cross-references resolve (any path mentioned actually exists).

### Step ⑤ — Run the paired audit skill (sev-0)

Every workflow skill `<name>-author` has a paired `<name>-audit`. The audit skill reads the artefact and runs the 8-step `AUDIT_LOOP.md` algorithm. Outcomes:

- **PASS** — no open issues. Save the audit report. Move to next chain step.
- **HITL_PAUSE** — at least one issue requires human judgement. Answer the questions in chat; the audit re-runs.
- **EXHAUSTED** / **NO_PROGRESS** — the audit can't reach pass even with retries. Read the audit report; either fix the artefact manually OR demote to lean profile and skip the audit OR file a `.out-of-scope/<rule>.md` if the rule itself is wrong.

### Step ⑥ — Update CONTEXT.md (per M2)

If the skill resolved any new domain terms during its run, append to the project's `CONTEXT.md`. The `cuo/cpo/requirements-discovery` skill will do this automatically; later skills should add terms only when the artefact introduces a new canonical name.

### Step ⑦ — Append the §14 memory-update block

The agent will end its response with a §14.1 compact block. The audit row goes to `.cyberos-memory/audit/<YYYY-MM>.jsonl`. **Do not edit the audit ledger by hand.** If the agent didn't append, run `python3 .cyberos-memory/.brain_writer.py session-end <actor>` before closing the tab.

---

## Phase A — Requirements Discovery (start here)

**Skill**: `cuo/cpo/requirements-discovery`
**Files to load**: `SKILL.md` (283 lines / 17 KB) + `STANDALONE_INTERVIEW.md` (156 lines / 7.5 KB)
**Input**: free-text pitch / draft requirements / commissioning email
**Output**: `project_brief@1` markdown — the structured intake artefact every downstream skill consumes

### Procedure

1. Open a fresh agent session in the new project's repo.
2. Paste:

   ```
   I want to run cuo/cpo/requirements-discovery on a new project.

   Initial pitch: <paste your draft requirements here>

   Output dir: ./planning/<YYYY-MM-DD>-<project-slug>/
   Caller: human:stephen-cheng
   Trace ID: <YYYY-MM-DD>-<project-slug>-discovery

   Load the skill at /Users/stephencheng/Projects/CyberSkill/cyberos/docs/skills/cuo/cpo/requirements-discovery/SKILL.md
   And the interview at .../requirements-discovery/STANDALONE_INTERVIEW.md
   Then start at Q0 (or skip Q0 if I provided an initial_prompt above).
   ```

3. Answer 20 questions: **Q0 (initial pitch)** + **Q1-Q5 (triage gating: strategic fit / capacity / runway / customer signal / reversibility)** + **Q6-Q20 (discovery: objectives / users / metrics / constraints / risks / scope / etc.)**. The interview is project-kind-agnostic — works for software, marketing, hiring, partnerships, research.
4. **Triage verdicts**: after Q1-Q5, the skill computes one of `proceed | revise | reject`. If `revise`, the skill routes to a different persona (e.g., `cuo-clo` for locked-decision conflicts) — pause the chain until that's resolved.
5. **CONTEXT.md is built inline.** When you use a term that needs a canonical definition, the skill asks "what should we call this?" and appends to your project's `CONTEXT.md`. Don't fight this — it's the language scaffolding for every later skill.
6. **Output**: `./planning/<date>-<slug>/project-brief.md` (~3-8 KB) with the 14-field `project_brief@1` frontmatter populated.
7. **Audit**: `requirements-discovery` doesn't have a paired audit skill (the interview script + invariants are the audit). The triage-verdict computation IS the gate.

### Common HITL pauses you'll hit

- **Q1 (strategic fit)** — "this conflicts with locked decision DEC-NNN; revisit it?" → only you can answer.
- **Q2 (capacity)** — "current team has 3 engineer-weeks; project needs 12 — hire / scope-down / reject?" → only you can answer.
- **Q4 (customer signal)** — "I see <2 prior signals in `memories/projects/`; do you have additional ones not in BRAIN?" → answer based on memory.
- **Triage verdict `revise`** — if you'd rather override and proceed anyway, say *"override triage; proceed despite revise"* and the skill records the override with `provenance.confidence: 0.5` (downgraded authority).

### Validation before moving on

- ✅ `project-brief.md` exists with all 14 frontmatter fields populated
- ✅ `CONTEXT.md` has at least the 3-5 core domain terms defined
- ✅ Triage verdict is `proceed` (or you've explicitly overridden)
- ✅ §14 block confirms the audit row was appended

---

## Phase B — Chain Selection

**Skill**: `cuo/cpo/chain-selector`
**Files to load**: `SKILL.md` (185 lines / 7 KB)
**Input**: `project-brief.md` from Phase A
**Output**: `chain_plan` — a list of skill IDs the supervisor (you) will route through

### Procedure

1. New session.
2. Paste:

   ```
   Run cuo/cpo/chain-selector on the brief at ./planning/<date>-<slug>/project-brief.md.

   Caller: human:stephen-cheng
   Trace ID: <date>-<slug>-chain

   Load: /Users/stephencheng/Projects/CyberSkill/cyberos/docs/skills/cuo/cpo/chain-selector/SKILL.md
   Output dir: ./planning/<date>-<slug>/
   ```

3. The skill reads the brief's `project_kind`, `eu_ai_act_risk_class`, `confidentiality`, `budget_band`, `target_release` → recommends a `chain_profile`:

   - **lean** — small / experimental / non-customer-facing. Skips PRD-audit, SRS-author/audit, fr-to-tech-spec. Just: brief → fr-author → fr-audit → spec-to-impl-plan. Use for prototypes, internal tools.
   - **standard** — typical customer-facing feature. Skips SRS-author/audit. Use for most projects.
   - **full** — regulated / high-risk / multi-team. Everything including SRS. Use for EU AI Act high-risk, healthcare/finance verticals, anything with compliance review.

4. **Override at brief-completion time** if the auto-selection feels wrong. Just say *"override to <profile>"*.

5. **Output**: `./planning/<date>-<slug>/chain-plan.md` listing the skill IDs you'll run.

### Validation

- ✅ Profile feels right for the project's risk + scale
- ✅ Skill list matches the chain end-to-end overview above for that profile

---

## Phase C — PRD Authoring

**Skill**: `cuo/cpo/prd-author`
**Files to load**: `SKILL.md` (247 lines / 13 KB)
**Input**: `project-brief.md` + `CONTEXT.md`
**Output**: `prd@1` markdown — the PRD that engineering will eventually consume

### Procedure

1. New session.
2. Paste the standard envelope-as-chat format:

   ```
   Run cuo/cpo/prd-author on:
   - Brief: ./planning/<date>-<slug>/project-brief.md
   - Context: ./CONTEXT.md
   - Output: ./planning/<date>-<slug>/prd-<feature-slug>.md
   - Caller: human:stephen-cheng
   - Trace ID: <date>-<slug>-prd-<feature-slug>

   Load: /Users/stephencheng/Projects/CyberSkill/cyberos/docs/skills/cuo/cpo/prd-author/SKILL.md
   ```

3. The skill reads the brief, the context, and any prior `memories/decisions/` + `memories/refinements/`. It generates the PRD body section-by-section, asking you to confirm each section before proceeding. **Don't rush** — the PRD's quality determines how clean the audit will be.
4. **Use only canonical terms from CONTEXT.md** (per Plan v1.1 / M2 / `INV-CONTEXT-CONSISTENCY-001`). If you find yourself wanting a new term, pause and update CONTEXT.md first.
5. **Output**: `./planning/<date>-<slug>/prd-<feature-slug>.md` with the 24-field `prd@1` frontmatter + 8 required H2 sections (Goals / Non-goals / User stories / Success metrics / Constraints / Risks / Open questions / Acceptance criteria).

### Validation

- ✅ All 24 frontmatter fields populated
- ✅ All 8 required H2 sections present
- ✅ Every term used has a CONTEXT.md entry
- ✅ Open questions section is honest (don't paper over uncertainty)

---

## Phase D — PRD Audit (standard / full only)

**Skill**: `cuo/cpo/prd-audit`
**Files to load**: `SKILL.md` (194 lines / 7.6 KB) + `AUDIT_LOOP.md` (87 lines / 4 KB) + `RUBRIC.md` (112 lines / 7.6 KB)
**Input**: `prd@1` markdown from Phase C
**Output**: `prd-<feature>.audit.md` audit report + (optionally) edits to the PRD

### Procedure

1. New session.
2. Paste:

   ```
   Run cuo/cpo/prd-audit on ./planning/<date>-<slug>/prd-<feature-slug>.md.

   Caller: human:stephen-cheng
   Trace ID: <date>-<slug>-prd-audit-<feature-slug>

   Load: /Users/stephencheng/Projects/CyberSkill/cyberos/docs/skills/cuo/cpo/prd-audit/SKILL.md
   Plus AUDIT_LOOP.md and RUBRIC.md from the same folder.
   ```

3. The skill executes the **8-step audit-fix loop**:

   ```
   1. Locate prd_path; init audit_path = prd_path with .audit.md extension
   2. Hash (UTF-8, LF, BOM strip, trailing-WS trim, ≥3-blank-line collapse, terminating LF, sha256)
   3. Load or initialise audit report
   4. Run rubric (every rule in RUBRIC.md §15.1-§15.8)
   5. Attempt fixes (auto-fix / inferable-skeleton / HITL-only / ambiguous)
   6. Re-audit
   7. Termination check (PASS / HITL_PAUSE / EXHAUSTED / NO_PROGRESS)
   8. Write audit report
   ```

4. **HITL_PAUSE handling**: the audit will list questions like:

   ```
   ## ISS-003 — `eu_ai_act_risk_class` may be wrong [needs_human]
   The PRD declares `minimal` but mentions "automated decision-making affecting users" in section 4.
   This may push to `limited` or `high`. Which is it?
   ```

   Answer in chat: *"It's `limited` — the system suggests but does not auto-execute."* The audit re-enters Step 4 and reconciles.

5. **EXHAUSTED handling**: rare; means the audit hit max_iterations without converging. Read the audit report's open issues. Either:
   - Edit the PRD manually to address them.
   - File a refinement proposal: *"the rubric rule SEC-008 fires on every PRD I write — propose relaxing it."* This becomes a `REF-NNN` entry.
   - Demote to lean profile and skip future PRD-audits for this project.

6. **NO_PROGRESS handling**: same `(rule_id, location)` set on consecutive iterations means the audit is stuck. Same options as EXHAUSTED.

7. **Output**: `./planning/<date>-<slug>/prd-<feature-slug>.audit.md` with `overall_status: pass | needs_human | fail`. Only proceed when `pass`.

### Validation

- ✅ `overall_status: pass`
- ✅ Audit iteration count ≤ max (typically <5)
- ✅ All `needs_human` issues have non-null `resolution`

---

## Phase E — SRS Authoring + Audit (full only)

Same shape as Phase C + D but with `cuo/cto/srs-author` (175 lines / 5 KB) and `cuo/cto/srs-audit` (157 lines / 4.6 KB). The SRS adds the technical-spec layer above the PRD's product-spec layer. **Skip this phase entirely** for lean and standard profiles.

The SRS-audit's RUBRIC focuses on:

- Architectural-decision references (every architectural choice → ADR in `docs/adr/`)
- NFR completeness (performance / security / accessibility / observability / etc.)
- Cross-system integrations explicit
- Failure modes documented

---

## Phase F — Feature-Request Authoring

**Skill**: `cuo/cpo/fr-author`
**Files to load**: `SKILL.md` (364 lines / 20 KB) — largest in the chain; budget read-time accordingly
**Input**: audited `prd@1` (+ audited `srs@1` if full profile)
**Output**: a folder of `feature_request@1` markdown files — one per feature, each ≤2 weeks of work

### Procedure

1. New session.
2. Paste the standard run prompt with paths to the audited PRD (and SRS if full).
3. The skill **decomposes the PRD into feature requests**. Each FR is a unit of work that:
   - Has a single dominant risk
   - Has a single dominant invariant
   - Is independently completable
   - Has clear acceptance criteria

4. **Output**: `./planning/<date>-<slug>/fr/FR-001-<slug>.md`, `FR-002-<slug>.md`, … One file per feature.

### Validation

- ✅ Each FR has all 18 frontmatter fields populated
- ✅ Each FR's acceptance criteria are testable (could be expressed as a `test should X` statement)
- ✅ FR scopes don't overlap (each piece of work belongs to exactly one FR)

---

## Phase G — Feature-Request Audit

**Skill**: `cuo/cpo/fr-audit`
**Files to load**: `SKILL.md` (316 lines / 16 KB) + `RUBRIC.md` + `AUDIT_LOOP.md`
**Input**: folder of FR markdowns
**Output**: `<FR>.audit.md` per FR + `AUDIT_BATCH_SUMMARY.md` aggregate

### Procedure

Same shape as PRD-audit but **runs per-FR sequentially**. The skill emits an `AUDIT_BATCH_SUMMARY` after all FRs, plus a single `HITL_BATCH_REQUEST` if any FR has `needs_human`. Answer all HITL questions in one batch; the skill resumes per-FR.

### Validation

- ✅ Every FR has `overall_status: pass`
- ✅ Batch summary shows zero open / zero needs_human
- ✅ FR cross-refs (`depends_on: [FR-NNN]`) all resolve

---

## Phase H — Tech Spec (standard / full only)

**Skill**: `cuo/cto/fr-to-tech-spec`
**Files to load**: `SKILL.md` (270 lines / 14 KB)
**Input**: audited FR markdowns
**Output**: `tech_spec@1` markdown — implementation-shaped spec engineering will execute against

For lean profile, **skip this phase**; spec-to-impl-plan consumes audited FRs directly.

The tech-spec adds:

- Concrete data shapes (tables / API schemas / message formats)
- Component decomposition (which files, which classes, which modules)
- Sequence diagrams for cross-component interactions
- Specific library / framework / language choices (with ADR refs)

This is where mattpocock's `/grill-with-docs` discipline matters most: **every term in the tech-spec must already be in CONTEXT.md** (or be added to it during this phase). Don't introduce new vocabulary at the implementation layer.

---

## Phase I — Implementation Plan (the chain's final step)

**Skill**: `cuo/cto/spec-to-impl-plan`
**Files to load**: `SKILL.md` (217 lines / 9 KB)
**Input**: audited `tech_spec@1` (standard / full) OR audited FR (lean)
**Output**: `impl_plan@1` markdown + (optionally) tickets in PROJ MCP

### Procedure

1. New session.
2. Paste the standard run prompt.
3. **Vertical-slice rule (per Plan v1.1 / M3 / INV-VERTICAL-SLICE-001)**: every issue in `impl_plan@1` MUST be:
   - Independently completable
   - Independently testable (each issue's acceptance includes one failing test that the issue makes pass)
   - 2-15 minutes of focused work for an enthusiastic-but-clueless junior engineer (the Superpowers heuristic)

   The skill rejects horizontal-slicing patterns ("build all schemas first → build all handlers"). If you see one, push back: *"this looks horizontal — restructure as per-feature vertical slices."*

4. **HALT_BEFORE_CREATE** (per existing INV-002): even with `create_tickets: true`, the runtime forces a final HALT prompt before creating tickets in PROJ MCP. Manually mode: just don't run the create-tickets step until you've reviewed the plan in markdown form.

5. **Output**: `./planning/<date>-<slug>/impl-plan.md` with a list of issues, each with: title / acceptance test / file paths / dependencies / estimate.

### Validation before handing to Execute

- ✅ Every issue passes the vertical-slice test
- ✅ Total estimate sums to a believable number for your team
- ✅ Critical-path dependencies don't form a cycle
- ✅ No issue has more than 2 unresolved open questions

---

## Handling refinement proposals (the human loop)

When a skill detects an anomaly during a run, it emits a `refinement_proposal` envelope. You'll see something like:

```
## REFINEMENT PROPOSAL — REF-NNN-<slug>

**Signal**: triage_reject_streak (3 consecutive rejects in window=10)

**Observation**: The discovery skill's triage rubric rejected the last 3 projects.
This may indicate the rubric is too strict for the current project mix
(e.g., requiring "≥3 independent customer signals" when most projects this quarter
are exploratory internal tools that won't have external signals).

**Proposed change**: Soften Q4 (customer signal strength) to allow `internal_tooling`
project_kind to bypass the signal-count gate when capacity check (Q2) passes.

**Scope of change**: cuo/cpo/requirements-discovery/STANDALONE_INTERVIEW.md §Q4
**Decision**: ✅ Accept / ✏ Accept-with-edits / ⏸ Defer / ❌ Reject
```

### Your four options

- **Accept** — the skill version bumps; a `memories/refinements/REF-NNN-<slug>.md` entry lands.
- **Accept-with-edits** — you tweak the proposed change; reply with the edited version; same outcome.
- **Defer** — the proposal stays open; the rolling window resets; revisit next time.
- **Reject** — write a `.out-of-scope/<slug>.md` entry per Plan v1.1 / M1. Three sections: what's out of scope, why (the criteria for what WOULD be in scope), prior requests (REF-NNN audit_id).

### When to favour each

| Signal type | Default lean |
|---|---|
| `triage_reject_streak` 3+ | Investigate — usually rubric is too strict OR your project pipeline shifted |
| `interview_truncation_rate` >30% | Accept — questions are too long or in wrong order |
| `brain_read_zero_results_rate` >50% | Accept — wrong scopes |
| `same_artefact_rewritten_more_than_5x` | Defer or Reject — usually the artefact's owner needs more clarity, not the skill |

---

## When something breaks

| Symptom | Likely cause | Fix |
|---|---|---|
| Skill asks for a field you don't have | Envelope schema mismatch | Open the skill's `expects.schema_ref` JSON; add a default value to your prompt |
| Audit keeps cycling on the same issue | NO_PROGRESS termination | Edit the artefact manually OR file a refinement proposal against the rule |
| `chat.review_request` MCP not available | Runtime not built | Manual mode: just answer the HITL question in chat directly |
| `proj.create_issue` MCP not available | Runtime not built | Manual mode: copy issues from `impl_plan@1` into Linear/Jira/GitHub by hand |
| §14 block doesn't appear | Agent forgot | Run `python3 .cyberos-memory/.brain_writer.py session-end <actor>` before closing |
| BRAIN classified `INCOMPATIBLE:protocol-sha256-mismatch` | AGENTS.md changed since last pin | Run §0.5 protocol upgrade flow OR revert AGENTS.md to the pinned SHA |

---

## What to save at the end of a chain run

Per project, the deliverables you keep:

```
./planning/<YYYY-MM-DD>-<project-slug>/
├── project-brief.md            # Phase A
├── chain-plan.md               # Phase B
├── prd-<feature>.md            # Phase C (one per feature)
├── prd-<feature>.audit.md      # Phase D (standard/full)
├── srs-<feature>.md            # Phase E (full only)
├── srs-<feature>.audit.md      # Phase E
├── fr/
│   ├── FR-001-<slug>.md        # Phase F
│   ├── FR-001-<slug>.audit.md  # Phase G
│   └── ...
├── tech-spec-<feature>.md      # Phase H (standard/full)
└── impl-plan.md                # Phase I — hand to engineering
```

Plus:

```
./CONTEXT.md                    # the project's shared vocabulary
./docs/adr/                     # ADRs land here as the chain runs (sparingly)
./.out-of-scope/                # rejected refinement proposals (only if any)
./.cyberos-memory/              # the BRAIN — keep, this is your replay-able audit ledger
```

---

## How long does this take, realistically

| Phase | Lean | Standard | Full |
|---|---|---|---|
| A — Requirements Discovery | 30 min | 30 min | 45 min |
| B — Chain Selection | 5 min | 5 min | 5 min |
| C — PRD Author | — | 30 min | 45 min |
| D — PRD Audit | — | 15 min | 20 min |
| E — SRS Author + Audit | — | — | 60 min |
| F — FR Author | 20 min | 30 min | 45 min |
| G — FR Audit | 15 min | 20 min | 30 min |
| H — Tech Spec | — | 30 min | 60 min |
| I — Impl Plan | 15 min | 20 min | 30 min |
| **Total** | **~85 min** | **~3 hours** | **~5-6 hours** |

These are the happy-path numbers. Add 30-50% for HITL pauses, refinement proposals, and re-runs after audit fails. For your first run on the next project, expect **~6-8 hours** (you'll be learning the loop too).

---

## Anti-patterns (don't do these)

- **Skipping the audit-fix loop on standard / full profile** — the rubric exists for a reason; bypassing it is how subtle bugs land in the impl_plan.
- **Editing audit/<YYYY-MM>.jsonl by hand** — sev-0 forbidden per AGENTS.md §7.4. Use `op:"corrects"` referencing the prior `audit_id` instead.
- **Skipping CONTEXT.md updates** — the cost compounds; later skills will use inconsistent terms; the eventual codebase will be hard to navigate.
- **Accepting a refinement proposal without reading the proposed change carefully** — once accepted, the skill version bumps and the change applies to every future run.
- **Mixing chain runs across projects in the same `./planning/` folder** — one folder per project per date, always.
- **Running multiple skills in the same agent session** — context-pollution risk. Fresh session per skill is cheap regardless of host.
- **Falling for "this is too simple to need a PRD"** — superpowers' anti-rationalization rule applies here. Even tiny features benefit from the brief → chain-selector → at-least-FR pipeline.

---

## Where to go next

After you've run this on one project end-to-end:

1. **File observations** as `memories/refinements/REF-NNN-<slug>.md` candidates. The runtime will use these as training data for Phase 2.
2. **Update `cyberos/docs/skills/CHANGELOG.md`** with anything you learned that the skill set didn't cover (gaps for `cuo/cto/code-author`, `qa-runner`, `qa-triage` — the Execute + QA halves of the workflow not yet designed).
3. **Read the plan** at `.cyberos-memory/project/skills-evolution/cyberos-skills-evolution-plan.md` to see where this fits in the multi-phase roadmap.

## History

- 2026-05-11 — Initial creation. Author: Claude Opus 4-7 in CyberSkill workbench session 6. The bridge from "design-complete chain" to "manually-runnable today" pending Phase 1 runtime.
