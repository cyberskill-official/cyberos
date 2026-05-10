# CHAIN_ORCHESTRATOR.md — agent-side runbook for fully automated chain execution

> **Audience: the AGENT** (Claude Sonnet 4.6 / Opus 4.7 / equivalent reasoning model). When the human user invokes this orchestrator, you become the supervisor for the full Requirements → Planning chain. The user's job shrinks to: (1) provide an initial pitch, (2) answer HITL questions you raise. Everything else — reading SKILL.md files, conducting interviews, writing artefacts, running audit-fix loops, executing brain_writer.py, routing between skills — is YOUR job.

> **Audience: the HUMAN.** Pin the trigger phrase below. Once invoked, you only need to answer questions the agent asks you. The agent drives every other step.

---

## Trigger phrases (the human pins one of these)

The user invokes you with one of:

```
Drive the CyberOS chain on this project. Read cyberos/docs/skills/CHAIN_ORCHESTRATOR.md and follow it.

Pitch: <one paragraph describing the project>
Project repo: <absolute path to the new project's directory>
Output dir: <where to save artefacts; default ./planning/<YYYY-MM-DD>-<slug>/>
Caller: human:<their-id>
Profile preference: <auto | lean | standard | full> (default: auto — let chain-selector decide)
```

Or shorter:

```
/cyberos-chain
Pitch: <paragraph>
```

If shorter form is used, ask for missing fields via AskUserQuestion (or chat questions if AskUserQuestion isn't available).

---

## Operating contract (read these once, then internalise)

### What you do

1. **Read every SKILL.md the chain requires** — yourself, via the Read tool. Don't ask the user to paste them.
2. **Conduct interviews in chat** — ask questions one at a time when the SKILL.md / STANDALONE_INTERVIEW.md prescribes them. Use the AskUserQuestion tool when the question has a clear set of options (≤4 choices); use plain chat questions for free-text answers.
3. **Generate artefacts** — write to disk via the Write tool. Save to `<output_dir>/<artefact-name>.md`.
4. **Run the audit-fix loop autonomously** — execute the 8-step algorithm from each `<skill>-audit/AUDIT_LOOP.md`. The loop runs in your head + via tool calls; the user only sees HITL pauses.
5. **Execute brain_writer.py via bash** — append audit rows. The user shouldn't see this; just do it after every artefact write.
6. **Update `CONTEXT.md`** when domain terms are resolved during interviews.
7. **Write `.out-of-scope/<topic>.md`** when the user rejects a refinement proposal.
8. **End every substantive reply with a §14 compact memory-update block** (per AGENTS.md §14.1).

### What you ask the user for

- The initial pitch (if not provided in the trigger).
- Triage gating answers (Q1-Q5 of requirements-discovery): strategic fit / capacity / runway / customer signal / reversibility.
- Discovery answers (Q6-Q20): objectives / users / metrics / constraints / risks / scope / etc.
- Profile override at brief-completion (if auto-selection feels wrong).
- Section-by-section approval during PRD/SRS/FR generation (one bulk approval per section is fine).
- HITL answers when an audit pauses with `needs_human` issues.
- Decisions on refinement proposals (Accept / Accept-with-edits / Defer / Reject).
- Any explicit `override` calls if you suggested one.

### What you DO NOT ask the user

- "Should I read this SKILL.md?" → just read it.
- "Should I save this artefact?" → just save it.
- "Should I run brain_writer?" → just run it.
- "Should I move to the next skill?" → just move; report transitions in the §14 block.
- "What's the path to AGENTS.md?" → resolve it yourself from the standard layout.

### Pacing rule

- **Default**: announce phase transitions in chat; ask HITL questions; keep moving.
- **If the user types `pause`**: stop, summarise current state, wait.
- **If the user types `resume`**: continue from last state.
- **If the user types `abort`**: write a `<output_dir>/ABORTED.md` with current state; exit.
- **If the user types `status`**: emit a §14 compact block + the chain-position summary.

---

## Phase ⓪ — Pre-flight (silent unless something's broken)

Run these in order; the user shouldn't see most of this unless something fails.

1. **Verify project root.** Resolve `<project repo>` to an absolute path. Confirm it's a real folder (not a sandbox path forbidden by AGENTS.md §0.1). If forbidden → halt; ask user to grant access.
2. **Verify AGENTS.md is loaded.** If the conversation context doesn't already contain the protocol, read `cyberos/docs/CyberOS-AGENTS-CORE.md` now. Acknowledge `Loaded agent memory protocol`.
3. **Bootstrap BRAIN if needed.** Check for `<project repo>/.cyberos-memory/manifest.json`. If absent → run `python3 <cyberos>/docs/skills/scripts/bootstrap-brain.sh <project-repo>` (if exists) OR perform §13.1 manually using the Write tool. If present → check `READY` state per §13.0.
4. **Create output dir.** `mkdir -p <output_dir>`.
5. **Initialise CONTEXT.md** at `<project repo>/CONTEXT.md` if absent (skeleton: 3 H2 sections — Language / Relationships / Flagged ambiguities).
6. **Initialise `docs/adr/` and `.out-of-scope/`** as empty directories.
7. **Append `op:"session.start"`** via `python3 outputs/brain_writer.py session-start agent:claude-opus-4-7` (run from project repo root).
8. **Resolve the chain.** Default chain (will be refined by Phase B):

   ```
   A. requirements-discovery   →  project_brief@1
   B. chain-selector           →  chain_plan
   C. prd-author               →  prd@1
   D. prd-audit                →  audited prd@1            (skipped on lean)
   E. srs-author + srs-audit   →  audited srs@1            (full only)
   F. fr-author                →  feature_request@1 ×N
   G. fr-audit                 →  audited fr@1 ×N
   H. fr-to-tech-spec          →  tech_spec@1              (skipped on lean)
   I. spec-to-impl-plan        →  impl_plan@1
   ```

9. **Announce readiness in chat**: *"Pre-flight complete. Starting Phase A: Requirements Discovery."* Move to Phase A.

---

## Phase A — Requirements Discovery (the longest phase; ~30-45 min)

### Your steps

1. **Read** `cyberos/docs/skills/cuo/cpo/requirements-discovery/SKILL.md` (283 lines) and `STANDALONE_INTERVIEW.md` (156 lines). Internalise the 5 triage gating questions + 15 discovery questions.
2. **Read BRAIN scopes** the SKILL.md declares (`company:locked-decisions`, `company:values`, `memories:projects`, `memories:decisions`, `memories:refinements`, `member:*` excluding `private/`, `client:*` if commissioned). Use this context to ask questions intelligently — e.g., for Q1 (strategic fit), surface the 3 most-relevant locked decisions before asking the question.
3. **Q0 (initial pitch)**: skip if pitch was provided in the trigger; else ask: *"What's the project? One paragraph is fine — what would you build, ship, or commission, and what would success feel like?"*
4. **Classify project_kind** silently: software_product / software_consulting_engagement / internal_tooling / marketing_campaign / hiring_plan / partnership / research_spike / other. If ambiguous, ask: *"This sounds like both X and Y. Which is the dominant frame?"* (use AskUserQuestion).
5. **Q1-Q5 triage gating** (one at a time, each via AskUserQuestion when options are clear):

   - Q1 strategic fit (use AskUserQuestion: aligns / partial / requires-revisit)
   - Q2 capacity (use AskUserQuestion: realistic / hire-needed / scope-down / reject)
   - Q3 runway (use AskUserQuestion for budget band: under_5k / 5k_to_25k / 25k_to_100k / over_100k / undisclosed; chat for ship-by date)
   - Q4 customer signal (chat — context-dependent)
   - Q5 reversibility (use AskUserQuestion: trivial / modest / meaningful / severe)

6. **Compute triage_verdict** silently from Q1-Q5 per the rubric in SKILL.md. If `revise` or `reject`:
   - Tell user: *"Triage verdict is `<v>` because <reasons>. Options: (1) override and proceed, (2) escalate to <persona>, (3) abort."*
   - Use AskUserQuestion. Honour the choice.
7. **Q6-Q20 discovery** — ask one at a time. For each answer, watch for **new domain terms**; when one appears, pause briefly and resolve into `CONTEXT.md`:
   - *"You said 'X' — should this be a canonical term in your project's vocabulary? If so, how should I define it?"*
   - Append to `<project>/CONTEXT.md` `## Language` section using the format in MANUAL_WORKFLOW.md.
8. **Synthesise `project_brief@1`** in markdown with the 14-field frontmatter populated. Save to `<output_dir>/project-brief.md`.
9. **Append audit row** via `python3 outputs/brain_writer.py write agent:claude-opus-4-7 project/<slug>/project-brief.md <abs path to artefact>`.
10. **Announce**: *"Phase A complete. Brief saved to <path>. Moving to Phase B (chain selection)."*

### HITL templates

When asking the user to approve a triage verdict:

```
The triage rubric flagged this project for `revise` because:
  - Q1: This conflicts with locked decision DEC-NNN (<title>).
  - Q4: Only 1 customer signal in BRAIN (rubric requires ≥3).

Options:
  1. Override and proceed (the brief will record `provenance.confidence: 0.5` to mark the override).
  2. Escalate to cuo-clo for locked-decision review.
  3. Abort and rethink.

Which would you like?
```

Use AskUserQuestion with these 3 options.

When resolving a domain term:

```
You used "account". In <other context> this could mean Customer or User.
  - Customer = a paying entity (organisation or individual).
  - User = a person with login credentials.

Which one do you mean here? Or is "account" the right canonical term for both, and I should add it to CONTEXT.md as a parent concept?
```

---

## Phase B — Chain Selection (~5 min)

### Your steps

1. **Read** `cyberos/docs/skills/cuo/cpo/chain-selector/SKILL.md`.
2. **Compute recommended profile** (`lean` / `standard` / `full`) from brief's `project_kind` + `eu_ai_act_risk_class` + `confidentiality` + `budget_band` + `target_release` per the SKILL.md rubric.
3. **Honour the user's profile preference** if set in the trigger (`auto` = use computed; otherwise override).
4. **Confirm with user** if `profile_preference == auto`:

   ```
   Recommended chain_profile: <profile>
   Reasoning: <one paragraph>
   Skill list this profile will run: <list>
   Override? (y/n, or specify another profile)
   ```

   Use AskUserQuestion: lean / standard / full / accept-recommended.

5. **Save `chain_plan`** to `<output_dir>/chain-plan.md`.
6. **Append audit row.**
7. **Announce**: *"Phase B complete. Profile: `<profile>`. Moving to Phase C (PRD authoring)."*

---

## Phase C — PRD Authoring (~30-45 min)

### Your steps

1. **Read** `cuo/cpo/prd-author/SKILL.md` + the project-brief @ `<output_dir>/project-brief.md` + `<project repo>/CONTEXT.md` + relevant `memories/decisions/` + `memories/refinements/`.
2. **Generate PRD body section by section** (8 required H2 sections: Goals / Non-goals / User stories / Success metrics / Constraints / Risks / Open questions / Acceptance criteria). After each section:
   - Show it to the user.
   - Ask: *"Section '<name>' looks like this. Approve / amend / skip?"* (AskUserQuestion).
   - On amend → take user's feedback, regenerate, re-confirm.
3. **Use only canonical CONTEXT.md terms.** If you find yourself wanting a new term, pause and resolve to CONTEXT.md first.
4. **Save `prd-<feature-slug>.md`** to `<output_dir>/`.
5. **Append audit row.**
6. **Announce**: *"Phase C complete. PRD: <path>. Running PRD-audit."*

### Pacing tip

- For lean profile, you can offer "approve all sections in one shot — I trust you" as a third option in the per-section question.
- For standard/full profile, per-section confirmation is recommended (cheap insurance against the audit pass kicking back issues).

---

## Phase D — PRD Audit (skipped on lean; ~15-20 min)

### Your steps

1. **Read** `cuo/cpo/prd-audit/SKILL.md` + `AUDIT_LOOP.md` + `RUBRIC.md`.
2. **Execute the 8-step audit loop** autonomously:
   - Step 1 — Locate `prd_path`; init `audit_path`.
   - Step 2 — Hash (UTF-8 / LF / BOM strip / trailing-WS / ≥3-blank-line collapse / terminating LF / sha256).
   - Step 3 — Load existing audit report or initialise.
   - Step 4 — Run every rule in RUBRIC.md §15.1-§15.8.
   - Step 5 — Attempt fixes (auto-fixable / inferable-skeleton / HITL-only / ambiguous).
   - Step 6 — Re-audit.
   - Step 7 — Termination check (PASS / HITL_PAUSE / EXHAUSTED / NO_PROGRESS).
   - Step 8 — Write audit report.
3. **Auto-fix what you can.** Don't ask the user about formatting fixes, missing-but-inferrable fields, etc.
4. **HITL_PAUSE handling**: collect ALL `needs_human` issues; ask them as a batch via AskUserQuestion or numbered chat questions. Resume the loop with answers.
5. **EXHAUSTED / NO_PROGRESS handling**: report to user:

   ```
   PRD audit hit <termination_reason> after <N> iterations.
   Open issues:
     - ISS-NNN: <summary>
     - ISS-MMM: <summary>

   Options:
     1. Edit the PRD manually and re-run audit.
     2. File a refinement proposal against rule <rule_id> ("rule may be too strict for this project type").
     3. Demote to lean profile and skip PRD-audit for this project.

   Which?
   ```

   Use AskUserQuestion.
6. **On PASS**: save the audit report; **append audit row** with op `consolidation_run`-shaped reasoning; announce: *"Phase D complete. PRD audit pass. Moving to Phase F (FR authoring)."* (Skip Phase E unless full profile.)

### HITL_PAUSE template

When the audit accumulates `needs_human` issues:

```
PRD audit paused — I need decisions on <N> issues:

1. ISS-003 — `eu_ai_act_risk_class` may be wrong.
   The PRD declares `minimal` but mentions automated decision-making.
   Options: minimal / limited / high.

2. ISS-007 — Open question Q4 ("how do we measure stickiness?") has no proposed answer.
   Suggested options: weekly active users / session length / feature breadth.

3. ISS-011 — `acceptance criteria` for "user can checkout" lacks a measurable threshold.
   Suggested: ≥98% of valid carts result in successful checkout in <3 seconds.

Please answer 1-2-3 in any format. I'll reconcile and re-audit.
```

---

## Phase E — SRS Author + Audit (full only; ~60 min)

Same shape as Phase C + D but driven by `cuo/cto/srs-author/SKILL.md` and `cuo/cto/srs-audit/SKILL.md`. Skip entirely on lean and standard.

The SRS audit's rubric leans advisory — most rules emit warnings, not blocking issues. Most runs PASS on first try; HITL is rare.

---

## Phase F — FR Authoring (~20-45 min)

### Your steps

1. **Read** `cuo/cpo/fr-author/SKILL.md` (364 lines — the largest skill).
2. **Decompose the audited PRD (+ SRS if full) into feature requests.** Each FR ≤2 weeks of work, single dominant risk, single dominant invariant, independently completable.
3. **Generate one FR markdown per feature** under `<output_dir>/fr/FR-001-<slug>.md`, FR-002-..., etc.
4. **Show the user the FR list** with one-line summaries:

   ```
   Decomposed into <N> feature requests:
     FR-001 <slug>: <one-line>
     FR-002 <slug>: <one-line>
     ...

   Approve / amend / decompose differently?
   ```

   Use AskUserQuestion: approve / amend / re-decompose.
5. **On approve**: save all FRs; append audit rows.
6. **On amend**: take user's feedback (which FR to merge / split / rename); regenerate; re-confirm.
7. **Announce Phase G start.**

---

## Phase G — FR Audit (~15-30 min)

### Your steps

1. **Read** `cuo/cpo/fr-audit/SKILL.md` + its RUBRIC + AUDIT_LOOP.
2. **Run the 8-step loop per-FR sequentially.** Don't parallelise — concurrent writes to the BRAIN ledger contend on `.lock`.
3. **Aggregate HITL questions across all FRs** before pausing. Better UX than asking once per FR.
4. **Emit `AUDIT_BATCH_SUMMARY.md`** at `<output_dir>/fr/`.
5. **On all-PASS**: announce; move to Phase H or I.
6. **On any FR not PASS**: list the FRs that need attention; ask user how to proceed (edit FR / drop the FR / file refinement / accept-with-warnings).

---

## Phase H — Tech Spec (skipped on lean; ~30-60 min)

### Your steps

1. **Read** `cuo/cto/fr-to-tech-spec/SKILL.md`.
2. **Generate `tech_spec@1` markdown** consuming all audited FRs. The tech-spec adds: data shapes / component decomposition / sequence diagrams / library + framework + language choices (with ADR refs in `<project>/docs/adr/` if architectural).
3. **Offer ADRs sparingly** — only when ALL THREE: hard-to-reverse + surprising-without-context + result-of-real-trade-off (per mattpocock's grill-with-docs rule).
4. **Confirm tech-spec with user** in chunks (Architecture / Data shapes / Components / Cross-system integrations / Failure modes).
5. **Save `tech-spec-<slug>.md`** to `<output_dir>/`.
6. **Announce Phase I start.**

---

## Phase I — Implementation Plan (~15-30 min — the chain's final step)

### Your steps

1. **Read** `cuo/cto/spec-to-impl-plan/SKILL.md`.
2. **Generate `impl_plan@1`** with vertical-slice issues (per Plan v1.1 / M3 / `INV-VERTICAL-SLICE-001`):
   - Each issue independently completable AND independently testable.
   - Each issue's acceptance includes one failing test that the issue makes pass.
   - 2-15 minutes of focused work per issue (the Superpowers heuristic).
   - Reject horizontal-slicing patterns explicitly.
3. **Show issue list** with summaries; confirm with user.
4. **HALT_BEFORE_CREATE** (per existing INV-002): even if `create_tickets: true` is set, force a final approval prompt before creating tickets in PROJ MCP. Default mode is markdown-only.
5. **Save `impl-plan.md`** to `<output_dir>/`.
6. **Append final audit rows.** Run `op:"consolidation_run"` then `op:"session.end"`.

---

## End-of-chain summary (template)

After Phase I, emit a final summary in chat:

```
✅ CyberOS chain complete — <project name>

Profile: <profile>
Total skills run: <N>
Total artefacts: <N>
Total audit rows appended: <N>
Total HITL pauses: <N> (you answered <M> questions)

Artefacts saved to <output_dir>:
- project-brief.md           (Phase A)
- chain-plan.md              (Phase B)
- prd-<slug>.md              (Phase C)
- prd-<slug>.audit.md        (Phase D)        ← if standard/full
- srs-<slug>.md              (Phase E)        ← if full
- srs-<slug>.audit.md        (Phase E)        ← if full
- fr/FR-001-<slug>.md, ...   (Phase F)
- fr/FR-NNN-<slug>.audit.md  (Phase G)
- tech-spec-<slug>.md        (Phase H)        ← if standard/full
- impl-plan.md               (Phase I)        ← hand to engineering

Project artefacts:
- <project repo>/CONTEXT.md   (<N> domain terms resolved)
- <project repo>/docs/adr/    (<M> ADRs created)
- <project repo>/.out-of-scope/ (<K> rejection records, if any)

BRAIN ledger:
- audit_chain_head: sha256:<hash>
- memory_count: <N>
- mode: normal

Time elapsed: <real time>
Next steps:
  - Review impl-plan.md before handing to engineering.
  - Create tickets in PROJ MCP if you didn't already (run /cyberos-tickets).
  - Schedule the next chain run for the next feature batch.

📝 .cyberos-memory updated
[§14 compact block]
```

---

## Resume contract (if a session ends mid-chain)

If your session ends (token limit, network drop, user quit) mid-chain:

1. The artefacts already on disk + the audit ledger + CONTEXT.md are durable.
2. Next session: the user runs the trigger phrase again. You:
   - Re-read `<project repo>/.cyberos-memory/manifest.json`.
   - Walk audit ledger to find the last `op:"create"` or `op:"str_replace"` against an artefact path.
   - Determine the chain position from that.
   - Announce: *"Resuming from Phase <X>. Last artefact: <path>. Continuing with skill <name>."*
   - Continue from that step.

Do not re-run completed phases. Do not re-ask the user questions you already have answers for.

---

## Failure modes you'll hit + the right reflex

| Symptom | Reflex |
|---|---|
| Skill SKILL.md not found at expected path | Ask user for the cyberos repo path; cache in this session's context |
| `python3 outputs/brain_writer.py` returns non-zero | Read stderr; if `frontmatter-validation` → fix the artefact; if `audit-corrupt` → halt and surface to user |
| User pastes an answer that doesn't fit the question's options | Reframe the question; don't punish the user for free-text — extract the option |
| Audit-fix loop doesn't terminate after 5 iterations | Treat as EXHAUSTED; surface to user per Phase D template |
| User says `pause` or `abort` | Honour immediately; write `<output_dir>/PAUSED.md` or `ABORTED.md` with current state |
| BRAIN classification returns `INCOMPATIBLE:protocol-sha256-mismatch` | Halt the chain; tell the user; offer §0.5 protocol-upgrade flow |
| Disk write fails (permission / quota) | Retry once; if it fails again, halt + surface the error |
| User says they want to switch to a different host mid-chain | Confirm + write `<output_dir>/HOST_SWITCH.md` recording current state; let them resume in the new host (the BRAIN + artefacts are host-agnostic) |

---

## What this orchestrator is NOT

- Not a skill in the v0.2.0 33-field SKILL.md sense — it's a **runbook for the agent**, not an envelope-driven workflow node.
- Not the runtime — the runtime is gated until `runtime_v0_3_0` per AGENTS.md. This is the manual bridge.
- Not a substitute for reading the underlying SKILL.md files — you must read each one before invoking it. The orchestrator just tells you in what order and how to glue them.
- Not Cowork-specific — it works in any host that has file tools + bash. But Cowork is the smoothest fit.

---

## Capability self-check (run silently before starting Phase A)

Before announcing Phase A, verify:

- [ ] Read tool available
- [ ] Write tool available
- [ ] Bash / shell tool available (for brain_writer.py)
- [ ] AskUserQuestion tool available (or willingness to use plain chat questions as fallback)
- [ ] Connected access to both `<cyberos>` (read-only OK) and `<project repo>` (read+write)

If any are missing, surface to the user before starting:

```
I'm missing <capability>. Options:
  1. Continue in degraded mode (you'll do <X> manually).
  2. Switch to a host that has <capability> (see cyberos/docs/skills/HOST_ADAPTERS.md).
  3. Abort.
```

---

## History

- 2026-05-11 — Initial creation. Author: Claude Opus 4-7 in Cowork session 8. Solves the "user wants to give first inputs + answer HITL only" requirement. Companion to MANUAL_WORKFLOW.md (which is the human-readable reference); this doc is the agent-readable runbook.
