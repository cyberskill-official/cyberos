# `cyberos/docs/skills/` — The CyberSkill Skill Wiki

> **One document. Everything you need to know about CyberOS skills.** Mental model · anatomy · five contracts · dual-mode invocation · exposability · auto-refinement · manual fine-tune · skills↔contracts split · host-adapter strategy · build-a-skill walkthroughs · runtime architecture · security model · performance & observability · per-persona quickstart · cookbook · FAQ · glossary. Read top-to-bottom on day 1 as your onboarding curriculum; come back to specific Parts as reference.

> **🚀 Running the chain TODAY (before runtime ships)?** Two modes:
>
> - **★ Automated** — give a pitch + answer HITL questions; the agent does everything else. Trigger phrase + agent runbook in [**CHAIN_ORCHESTRATOR.md**](./CHAIN_ORCHESTRATOR.md).
> - **Manual** — drive every step yourself. Procedure in [**MANUAL_WORKFLOW.md**](./MANUAL_WORKFLOW.md).
>
> Both modes are **host-agnostic** — Cowork (★ for automated), Claude Code, Cursor, Codex CLI, Gemini CLI, OpenCode, Windsurf, Copilot CLI, etc. Per-host setup in [**HOST_ADAPTERS.md**](./HOST_ADAPTERS.md). Pin all three docs when starting a new project.

> **📋 What's planned next?** → [**CHANGELOG.md**](./CHANGELOG.md) v0.2.10 entry (2026-05-11) lists three TIER-1 modifications (`.out-of-scope/` registry, `domain-context@1` contract, vertical-slice rule) and three TIER-2 additions (`lifecycle_state` field, `zoom-out` meta-skill, `caveman` operational_mode) lifted from mattpocock-skills, ECC, superpowers, and AGENTS.md protocol synthesis. Backed by the multi-phase plan in `<workbench>/.cyberos-memory/project/skills-evolution/cyberos-skills-evolution-plan.md`.

---

## Table of contents

- **Part 1** — [What is a skill, in 90 seconds](#part-1--what-is-a-skill-in-90-seconds)
- **Part 2** — [Anatomy: the 33-field SKILL.md contract](#part-2--anatomy-the-33-field-skillmd-contract)
- **Part 3** — [The 5 inherited contracts](#part-3--the-5-inherited-contracts)
- **Part 4** — [Dual-mode invocation: standalone OR chained](#part-4--dual-mode-invocation-standalone-or-chained)
- **Part 5** — [Exposability: plugin / MCP / connector](#part-5--exposability-plugin--mcp--connector)
- **Part 6** — [Auto-refinement: the self-audit loop](#part-6--auto-refinement-the-self-audit-loop)
- **Part 7** — [Manual fine-tune: the human loop](#part-7--manual-fine-tune-the-human-loop)
- **Part 8** — [Skills vs. contracts: the v0.2.0 split](#part-8--skills-vs-contracts-the-v020-split)
- **Part 9** — [Host-adapter strategy: CCSM → Anthropic / Antigravity / Codex / MCP](#part-9--host-adapter-strategy)
- **Part 10** — [Build a skill: step-by-step](#part-10--build-a-skill-step-by-step)
- **Part 11** — [Worked example end-to-end: fr-author → fr-audit](#part-11--worked-example-end-to-end-fr-author--fr-audit)
- **Part 12** — [Runtime architecture: LangGraph + action_log + NATS](#part-12--runtime-architecture-langgraph--action_log--nats)
- **Part 13** — [Validate & debug](#part-13--validate--debug)
- **Part 14** — [The skill lifecycle](#part-14--the-skill-lifecycle)
- **Part 15** — [Security model deep-dive](#part-15--security-model-deep-dive)
- **Part 16** — [Performance & observability](#part-16--performance--observability)
- **Part 17** — [Localization & i18n](#part-17--localization--i18n)
- **Part 18** — [Anti-patterns: what NOT to do](#part-18--anti-patterns-what-not-to-do)
- **Part 19** — [Cookbook: 13 recipes](#part-19--cookbook-13-recipes)
- **Part 20** — [Routing: how CUO picks a skill](#part-20--routing-how-cuo-picks-a-skill)
- **Part 21** — [Per-persona quickstart](#part-21--per-persona-quickstart)
- **Part 22** — [Migration from non-CyberOS skills](#part-22--migration-from-non-cyberos-skills)
- **Part 23** — [Index of skills + contracts](#part-23--index-of-skills--contracts)
- **Part 24** — [How to add a new skill](#part-24--how-to-add-a-new-skill)
- **Part 25** — [FAQ + glossary](#part-25--faq--glossary)
- **Part 26** — [What doesn't exist yet](#part-26--what-doesnt-exist-yet)
- **Part 27** — [Citations](#part-27--citations)

---

## Part 1 — What is a skill, in 90 seconds

A skill is a **folder** containing a single mandatory file: `SKILL.md`. That's it. Everything else is optional.

![Skill folder anatomy](./assets/diagrams/01-skill-folder-anatomy.svg)

The mental model in five lines: a **skill** is a folder with a `SKILL.md` (the atomic unit of versioning, audit, routing, and plug-in distribution). A **persona** is a folder of skills (e.g., `cuo/cpo/` is the Chief Product Officer; 14 personas total per DEC-052). A **trigger** is anything that hands an envelope to a skill — three paths, direct, supervisor-routed, or chained. A **chain** is when one skill's output envelope's `next_skill_recommendation` causes the supervisor to invoke another skill. A **contract** is a versioned schema (NOT a skill) under `cyberos/docs/contracts/`, declared via `depends_on_contracts:` in any consumer skill's frontmatter.

That's the whole architecture. Everything below is documentation about this five-line model. The shape is deliberate: a folder with a `SKILL.md` is the lowest common denominator across every modern agent host (Claude Code, Antigravity, Codex, Cursor, plus MCP tool registries), so the same source compiles to plugin, MCP tool, connector, Antigravity skill, or plain prompt without rewriting. The transpiler pipeline that does this compilation is documented in [Part 9](#part-9--host-adapter-strategy).

---

## Part 2 — Anatomy: the 33-field SKILL.md contract

Every workflow `SKILL.md` MUST carry the v0.2.0 frontmatter contract. Persona-cards (`cuo/<role>/SKILL.md`) carry a strict subset (no pipeline interface, no contract dependencies). Contracts (under `cyberos/docs/contracts/`) carry a smaller, contract-specific frontmatter — see [Part 8](#part-8--skills-vs-contracts-the-v020-split).

### 2.1 The full v0.2.0 frontmatter

```yaml
---
# ── Identity ─────────────────────────────────────────────────────────
name:               <kebab-case skill id; matches folder name>
description:        <one sentence; ≤140 chars; what + when CUO should invoke>
skill_version:      <SemVer; bumped on every CHANGELOG entry>
persona:            <cuo | cuo-<role> | cuo-_shared>
owner_role:         <role enum from §20.1 | _shared>

# ── Scope contract (SRS §6.4) ────────────────────────────────────────
allowed_brain_scopes:
  read:  [<scope-glob>, …]              # e.g. project:*, member:self
  write: [<scope-glob>, …]              # default: empty (read-only skill)
allowed_mcp_tools:  [<tool-name>, …]    # exhaustive; gateway enforces
escalation:
  to_persona_on_legal:    <persona-id | null>      # e.g. cuo-clo
  to_persona_on_security: <persona-id | null>
  to_persona_on_compliance: <persona-id | null>
  to_human_on_irreversible: true                    # default true

# ── Invocation modes (NEW v0.2.0 / DEC-091) ──────────────────────────
invocation_modes:   [standalone, chained]   # one or both; persona cards: [persona_routing_only]

# ── Pipeline interface (chaining contract) ───────────────────────────
expects:
  schema_ref:                   <relative path to JSON schema>
  required_fields:              [<field>, …]
  optional_fields:              [<field>, …]               # NEW v0.2.0
  standalone_interview_ref:     <relative path | null>     # NEW v0.2.0
produces:
  schema_ref:                   <relative path>
  output_kind:                  notify | question | review | act | artefact | refinement_proposal
  human_summary_ref:            <relative path | null>     # NEW v0.2.0

# ── Contract dependencies (NEW v0.2.0 / DEC-090) ─────────────────────
depends_on_contracts:
  - id:        <contract folder name>
    version:   v<n>
    purpose:   <why this skill needs it>
    pin_path:  <full path under cyberos/docs/contracts/>

# ── Exposability (NEW v0.2.0 / DEC-091) ──────────────────────────────
exposable_as:
  internal:           <bool>     # CUO supervisor can route here
  agent_plugin:       <bool>     # ships in plugin bundles
  mcp_tool:           <bool>     # auto-emit MCP tool descriptor
  partner_connector:  <bool>     # gated; needs partner-exposure DEC

# ── Audit hook (SRS §6.7) ────────────────────────────────────────────
audit:
  emit_to:                 genie.action_log     # always
  row_kind:                <one or more enum values>
  payload_hash_field:      <which produced field gets sha256'd>
  explanation_pane:        required

# ── Trust calibration (PRD §6.4) ─────────────────────────────────────
confidence_band:
  default:                 <0.0–1.0>
  defer_below:             0.5
  cite_sources:            required

# ── Untrusted-content discipline (DEC-050; AGENTS.md §4.2) ───────────
untrusted_inputs:
  wrap_in:                 <untrusted_content/>
  injection_scan:          required
  on_marker_hit:           surface_to_human

# ── Self-audit + auto-refinement (NEW v0.2.0 / DEC-092) ──────────────
self_audit:
  invariants_ref:          <relative path to INVARIANTS.md | null>
  check_at:                [on_node_boundary, {on_audit_row_count: 25}, on_completion]
  anomaly_signals:
    confidence_low_streak:     {threshold: <int>, window: <int>}
    user_correction_streak:    {threshold: <int>, window: <int>}
    denylist_near_miss_streak: {threshold: <int>, window: <int>}
    scope_rejection_streak:    {threshold: <int>, window: <int>}
    # plus skill-specific signals
  on_breach:
    emit:                  refinement_proposal
    pause_pipeline:        <bool>
    resume_token_field:    <field name>

# ── Manual fine-tune (NEW v0.2.0 / DEC-093) ──────────────────────────
human_fine_tune:
  fine_tuner_role:         <role | _shared | engineering | any>
  review_required:
    on_minor_bump:         <bool>
    on_major_bump:         <bool>      # always true for safety/scope changes
    on_safety_change:      <bool>
  signals_to_initiate:
    - acceptance_rate_below: <float>
    - hitl_pause_rate_above: <float>
    - drift_signal_count_above: <int>
    - <skill-specific signals>
  procedure_ref:           <path | null>      # null = use Part 7 default playbook
  required_artifacts:
    - changelog_entry
    - acceptance_test_added
    - memory_refinement_entry
  blackout_windows:        []        # ISO date ranges where edits are frozen

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible:            <bool>
  fixity_notes:            <e.g. "canonical JSON, sorted keys, no time fields">

# ── Source-tier emitted (AGENTS.md §5.1, §6, §9.1) ───────────────────
emitted_source_freshness_tier: <int ≥ 1 | null>   # null → tier 99 default
gated_until_phase:         <P0|P1|P2|P3|P4 | null>
---
```

### 2.2 Field families, by audience

The 33 fields split into three groups by who actually reads them at load time. Group 1 is portable across every modern agent host. Group 2 needs the CyberOS runtime to enforce; non-CyberOS hosts silently ignore these. Group 3 is the v0.2.0 governance and portability surface — declared in the SKILL.md but interpreted by transpilers and the host shim, not by every skill loader.

![Frontmatter field families](./assets/diagrams/02-frontmatter-field-families.svg)

This split is the answer to "if I copy this skill folder into Antigravity, will it work?" — the §3.1 fields will load and route. The §3.2 fields will silently fall back to filesystem-local enforcement via the host shim (Part 9.3). The §3.3 fields are read by the build pipeline at compile time and don't need runtime support.

### 2.3 The 3-tier progression

Most skills don't need all 33 fields filled out from day one.

| Tier | Fields | Outcome |
| --- | --- | --- |
| **Tier 0 — sketch** | `name` + `description` + body | Loads in any Anthropic-style host. Won't pass CyberOS registry validation. Useful for thinking. |
| **Tier 1 — production-routable** | + persona, owner_role, allowed_brain_scopes, allowed_mcp_tools, expects, produces, audit, untrusted_inputs | Passes minimum validation. Routable by the CUO supervisor. Sensible defaults fill the rest. |
| **Tier 2 — fully-specified** | All 33 fields | What you ship to production. Use `cuo/_shared/hello-world/SKILL.md` as your template — copy and edit. |

Promote step-by-step. Don't try to write Tier 2 first.

### 2.4 Body structure

After the frontmatter, the Markdown body MUST contain (in order): `# H1 title` (the skill's display name in human prose), `## When to invoke this skill` (natural-language phrases CUO should route here — this is what the classifier reads at routing time), the body proper (instructions to the LLM, MUST sections, MUST NOT sections, SHOULD sections), `## Failure modes` (link to `references/FAILURE_MODES.md` if it exists), and `## Citations` (every external source — PRD/SRS/AGENTS.md/DEC-NNN references — listed). Everything else is optional and progressive-disclosure.

---

## Part 3 — The 5 inherited contracts

Every skill inherits five contracts from this registry. Three were present in v0.1.0; **two are net-new in v0.2.0** (self-audit, host portability). They compose: a skill that violates any one of them is not a CyberOS skill.

![The 5 inherited contracts](./assets/diagrams/03-five-contracts.svg)

### 3.1 Audit-hook contract — SRS §6.7

Every concrete output (Notify / Question / Review / Act / artefact write / refinement_proposal) produces exactly one row in `genie.action_log`. Row schema: `(persona_id, skill_id, skill_version, row_kind, target, payload_sha256, explanation_pane_ref, confidence, hash_chain_prev, hash_chain_self)`. The hash chain mirrors AGENTS.md §7.2 canonical-JSON rules. Skipping the row is a contract violation surfaced by the CP module's tamper detector (SRS §10.4.6).

### 3.2 Chain contract

Skills compose via the `expects:` ↔ `produces:` envelopes. A LangGraph edge from `skill_A` to `skill_B` is legal when `skill_A.produces.schema_ref` validates against `skill_B.expects.schema_ref` (subset or identity). The CUO supervisor plans the chain at runtime; an example chain (`fr-author` → `fr-audit`) is documented in `cuo/cpo/fr-author/PIPELINE.md`. State between nodes is checkpointed to `genie.graph_checkpoint` per SRS §6.1.1 — chains are crash-safe and resumable.

### 3.3 Plug-in contract — AGENTS.md §11

A skill folder is a self-contained portable unit. Three granularities: one skill (`cp -r cuo/cpo/fr-author/ <other>/skills/cuo/cpo/` plus its declared contracts via `depends_on_contracts:`), one persona (`cp -r cuo/cpo/ <other>/skills/cuo/`), or the whole CUO bundle (`cp -r cuo/ <other>/skills/`). Export = deterministic zip per AGENTS.md §11.2 (sorted entries, fixed mtime, zero uid/gid). Import = unpack with the AGENTS.md §4.1 path-traversal guard + one `op:"import"` audit row.

### 3.4 Versioning + drift contract

Every skill carries `CHANGELOG.md` (Keep-a-Changelog 1.1.0). SemVer rules are explicit: **MAJOR** breaks `expects:`/`produces:` schema, removes a SKILL.md field, or removes a self-audit invariant. **MINOR** adds a backwards-compatible field, a new optional behaviour, a new invariant, or a new optional reference doc. **PATCH** is editorial. The `persona_version` stamp on every output (DEC-054) includes both the persona ID and the active skill version. Drift detection runs in OBS (SRS §6.12); acceptance rate <40% over 7 days auto-pauses the skill for the affected member (DEC-055).

### 3.5 Trust + safety contract — PRD §6.4

Every skill MUST carry a confidence band on every output, cite BRAIN sources for every factual claim (RAG-mandatory; no free-form recall), defer to a human via the Question primitive on irreversible actions / cross-tenant data / legal-or-compliance assertions / confidence below `defer_below` / conflicting BRAIN signals (AGENTS.md §9.1) / REW or LEARN or ESOP writes (PRD §6.4.1) / scope-contract refusal, wrap every external byte in `<untrusted_content>` before reasoning over it (DEC-050 CaMeL pattern), and stamp `emitted_source_freshness_tier` on every BRAIN write so downstream conflict resolution (AGENTS.md §9.1) ranks correctly.

### 3.6 Self-audit + auto-refinement contract — NEW v0.2.0 / DEC-092

Every skill carries `INVARIANTS.md` declaring runtime truths it enforces about its own behaviour. The runtime checks invariants at declared `check_at` checkpoints. Breach → emit `refinement_proposal` envelope (new `output_kind`), pause the pipeline, wait for human review. Detail in [Part 6](#part-6--auto-refinement-the-self-audit-loop).

### 3.7 Host portability contract — NEW v0.2.0 / DEC-091

The SKILL.md is the **Canonical CyberSkill Skill Manifest (CCSM)** — authoritative source of truth. Per-host artefacts under `dist/<host>/` are *generated* by transpilers, never hand-edited. Every skill is shippable to multiple agent hosts (Claude Code, Antigravity, Codex, Cursor, vanilla MCP) without behavioral drift. Detail in [Part 9](#part-9--host-adapter-strategy).

---

## Part 4 — Dual-mode invocation: standalone OR chained

A v0.2.0 skill is **one function body** with **two front doors**.

![Dual-mode invocation](./assets/diagrams/04-dual-mode-invocation.svg)

### 4.1 Mode detection (deterministic)

The runtime puts the skill in standalone mode when ALL of: no `pipeline_run_id` is present in the call context; the caller is the CHAT primitive (a human typing in a thread), not another skill; the input envelope is empty or `{}`. Otherwise: chained mode.

### 4.2 What changes between modes

| Aspect | Standalone | Chained |
| --- | --- | --- |
| `required_fields` source | the user, via `STANDALONE_INTERVIEW.md` | the upstream envelope |
| `optional_fields` source | defaults, unless user volunteers | upstream envelope, falling back to defaults |
| User-facing summary | `HUMAN_SUMMARY.md` rendered to chat | none — supervisor folds into upstream summary |
| Resume on partial input | re-enter the interview at the missing field | error: `MALFORMED_ENVELOPE` |
| `audit.row_kind` count | one row per `question` (interview Q) + one row per concrete output | one row per concrete output |

### 4.3 Why this matters

Without dual-mode, every skill needs two implementations: one for chat, one for pipelines. With dual-mode, the body is written once. The runtime picks the door. Adding a new skill becomes: write the function logic, write the input envelope schema, write `STANDALONE_INTERVIEW.md` (3-5 questions, max), write `HUMAN_SUMMARY.md` (one Markdown template). Done. The skill works in both modes.

### 4.4 Worked example — fr-author

In **standalone** mode, a user in CHAT says "turn this PRD into a backlog, here's the path". The supervisor routes to `fr-author`. The interview fires: Q1 `requirements_files` → answered, Q2 `output_dir` → defaulted, Q3 `manifest_path` → defaulted. The function runs. `HUMAN_SUMMARY.md` renders 3 FR lines + amendment list + trace ID. In **chained** mode, an upstream `cuo/cpo/prd-import` skill (hypothetical) emits `{requirements_files, output_dir, manifest_path, batch_size}` in its output envelope and sets `next_skill_recommendation: cuo/cpo/fr-author`. The supervisor invokes; the function runs; the output envelope feeds whatever's after. Same body. Different door.

---

## Part 5 — Exposability: plugin / MCP / connector

Every v0.2.0 skill declares `exposable_as:` listing which surfaces it can be shipped through. The build pipeline reads this and emits the right artefact per surface.

![Exposability surfaces](./assets/diagrams/05-exposability-surfaces.svg)

### 5.1 The four surfaces

| Surface | What it produces | Audience | Gating |
| --- | --- | --- | --- |
| `internal` | A SKILL.md the CUO supervisor can route to via the `classify_act` node. | CyberSkill employees. | None — default true. |
| `agent_plugin` | A `.plugin/` bundle (skills + commands + .mcp.json + manifest.json) per Claude-Code-style plugin spec. Ships in marketplace bundles. | Claude Code / Antigravity / Codex / Cursor users. | None — default true unless skill carries internal-only data. |
| `mcp_tool` | An MCP tool descriptor (`tool.json`) with auto-derived `inputSchema`/`outputSchema` from `expects`/`produces`. Plus a stdio runner stub. | Any MCP-compatible agent (any vendor). | Default false unless trivially safe; must be explicitly opted in. |
| `partner_connector` | A hosted MCP server + OAuth handshake + tenancy isolation + billing hooks + rate limits. | External partners on a marketplace. | **Always gated.** Requires a separate DEC per skill before flag flips to true. |

### 5.2 What the build pipeline does (today / soon)

Today: the flags exist; transpilers are scoped for the v0.3.0 milestone. See [Part 9](#part-9--host-adapter-strategy) for the phased plan and [Part 26](#part-26--what-doesnt-exist-yet) for the honest inventory of what's actually built.

When the build pipeline lands, it will work like this:

```bash
cyberos build cuo/cpo/fr-author
# emits:
#   dist/anthropic/cuo/cpo/fr-author/SKILL.md
#   dist/mcp-tool/fr-author/{tool.json, server.py}
#   dist/claude-plugin/cuo-fr/.plugin/...
#   dist/antigravity/cuo/cpo/fr-author/...
```

One source. Many surfaces. Per-host transpilers strip the irrelevant frontmatter (e.g., Antigravity ignores `audit:`).

### 5.3 The "exposability ↔ trust" link

A skill with `partner_connector: true` MUST also have `confidence_band.cite_sources: required`, `untrusted_inputs.injection_scan: required`, `self_audit.invariants_ref` populated with at least three invariants including INV-`scope-discipline` and INV-`fabrication-boundary`, and `human_fine_tune.review_required.on_safety_change: true`. The validator enforces these as a precondition for the partner flag.

---

## Part 6 — Auto-refinement: the self-audit loop

Every v0.2.0 skill ships with `INVARIANTS.md` listing declarative runtime truths. The runtime checks them at declared `self_audit.check_at` checkpoints. Breach → the skill emits a `refinement_proposal` envelope, the LangGraph supervisor pauses the pipeline, the human reviews, and the chain resumes from a checkpoint.

![Auto-refinement loop](./assets/diagrams/06-auto-refinement-loop.svg)

### 6.1 Anatomy of a refinement_proposal

```yaml
kind: refinement_proposal
skill_id: cuo/cpo/fr-author
skill_version: 0.2.0
trigger: "INV-003 breach: PRD digest mem_… has coverage 0.81 (<0.99) without intentional_summary flag"
observation: |
  Read 4 chunks of {source_path}; processed 81% of source lines.
  Coverage stat in BRAIN write was 0.81 / 1.00.
proposed_amendments:
  - tier: 1
    target_doc: cuo/cpo/fr-author/SKILL.md
    section: §"PLAN phase" step 1
    diff: |
      +  Paginate PRD reads sequentially (offset/limit) instead of
      +  sampling. Track high-water mark. Verify coverage ≥0.99 before
      +  writing the digest, OR set intentional_summary: true with
      +  summary_reason populated.
minimum_viable: "Add coverage check at end of PLAN phase."
pipeline_paused: true
resume_token: refinement_run_id_abc123
```

### 6.2 Anomaly signals (early warning)

Beyond invariant breaches, the runtime tracks streaks. `confidence_low_streak` triggers when N of last K outputs had `confidence < defer_below`. `user_correction_streak` triggers on N user corrections within K turns ("no, not that — do X instead"). `denylist_near_miss_streak` fires when N times in K writes the skill almost wrote denylisted content. `scope_rejection_streak` triggers when the gateway rejected N writes for scope violations. Skill-specific signals stack on top — for example, `fr-audit`'s `deterministic_drift` is sev-0 because the auditor's reproducibility is its core promise. When a signal threshold trips, the skill emits a `refinement_proposal` even without an invariant breach.

### 6.3 What auto-refinement is NOT

It is NOT auto-edit — the skill never modifies its own SKILL.md without human approval. It is NOT silent — every proposal is a Question primitive surfaced to a human. It is NOT a substitute for testing — acceptance fixtures (`acceptance/golden-input.json` + golden-output) still gate every skill_version bump in CI.

### 6.4 When auto-refinement escalates to manual fine-tune

If `self_audit_refinement_proposal_count_above` is exceeded (default: 2 proposals on the same theme within one batch), the auto-refinement loop hands off to the manual fine-tune flow. Reason: the skill has flagged a problem the auto-refinement loop can't resolve and a human investigation is needed. See [Part 7](#part-7--manual-fine-tune-the-human-loop).

---

## Part 7 — Manual fine-tune: the human loop

Auto-refinement (Part 6) catches mechanical drift. Manual fine-tune is for the harder cases: a human notices a pattern, decides the skill needs improvement, and runs a structured edit cycle.

![Manual fine-tune playbook](./assets/diagrams/07-manual-fine-tune-7-step.svg)

### 7.1 When to manually fine-tune

Manual fine-tune runs when ANY of: auto-refinement hit its escalation threshold (≥2 proposals on the same theme), acceptance rate has dropped below the `human_fine_tune.signals_to_initiate.acceptance_rate_below` threshold (default 0.6 for production skills) over the last 7 days, HITL pause rate is above `signals_to_initiate.hitl_pause_rate_above` (default 0.4 — the skill is asking too often), a regulator or a customer raised a specific concern that requires rule changes, or the persona owner schedules a routine fine-tune cycle (typically every 2 weeks during the first month, monthly thereafter).

### 7.2 The 7-step playbook

1. **Pause** — set skill to maintenance mode. Routes return `E_SKILL_MAINTENANCE`.
2. **Diagnose** — read `genie.action_log` for the affected window. Cluster failures by mode.
3. **Add regression** — write golden-input + golden-output for each distinct failure mode. Land under `acceptance/`.
4. **Edit** — `SKILL.md` body / `INVARIANTS.md` / `RUBRIC.md` (for auditor skills). Aim: pass new regressions without breaking old ones.
5. **Re-run acceptance suite** — old + new golden cases. ALL must pass.
6. **Bump + log** — PATCH if no behaviour change; MINOR if new invariant; MAJOR if envelope changes. CHANGELOG entry with `### Driver` citing the trigger.
7. **Resume** — set skill to active. Monitor acceptance over next 24-48h.

### 7.3 Who can fine-tune which skills

Per `human_fine_tune.fine_tuner_role`:

| `fine_tuner_role` | Who has commit access |
| --- | --- |
| `<role>` (e.g., `cpo`) | The persona owner + the registry maintainer. |
| `_shared` | Any persona owner whose persona consumes the skill. |
| `engineering` | Members of the engineering team (DEC-052 §3 §"engineering" annotation). |
| `any` | Any CyberSkill employee — used for low-risk skills like `hello-world`. |

`review_required` adds approval gates. MINOR bumps default to `false` (override per skill). MAJOR bumps are always `true` — `fine_tuner_role` plus the registry maintainer must both sign off. Safety changes (denylist tweak, scope widen, new MCP tool) are always `true` and add `cuo-cseco` plus `cuo-clo` to the reviewer set. For auditor skills only, rubric rule add/remove also requires `cuo-clo` if the rule touches legal/EU AI Act.

### 7.4 Required artefacts on every fine-tune

A fine-tune is incomplete without three artefacts. **`changelog_entry`** — Keep-a-Changelog 1.1.0, with `### Driver` citing the trigger. **`acceptance_test_added`** — at least one new regression in `acceptance/regression-<YYYY-MM-DD>-<slug>.md`. **`memory_refinement_entry`** — append to BRAIN `memories/refinements/REF-NNN-<slug>.md` describing what was learned and what changed (per AGENTS.md §0.4 + §10 step 6, future agents will read this). Optional but recommended: `rubric_rule_diff` (auditor skills only — the rule-by-rule diff at the top of the CHANGELOG entry), `drift_record` (when a drift signal triggered the cycle, append to BRAIN `memories/drift/<YYYY-MM-DD>-<source-slug>.md`).

### 7.5 Blackout windows

Any ISO date range listed in `human_fine_tune.blackout_windows` freezes edits to the skill. Useful for audit weeks (no skill edits during external compliance review), production launches (no skill edits in the 48 hours bracketing a customer-visible launch), and holiday weeks (no skill edits when the persona owner is offline). The validator rejects any commit touching the skill during a blackout window with `op:rejected reason:blackout-window-active`.

### 7.6 Why this is structured (not freeform)

The temptation with skills is "they're just markdown — anyone can edit them". That's true *technically*, false *operationally*. A skill in production has acceptance fixtures, drift telemetry, downstream consumers, partner exposures. Editing it without the playbook breaks trust calibration and audit chain. The 7-step playbook is the minimum discipline to keep skills evolvable without becoming brittle.

---

## Part 8 — Skills vs. contracts: the v0.2.0 split

Before v0.2.0, the registry conflated two things that are architecturally distinct.

### 8.1 The distinction

| | **Skill** | **Contract** |
| --- | --- | --- |
| What it does | *acts*: takes input, produces output, writes audit row | *constrains*: declares the shape of an artefact, envelope, or wire protocol |
| Folder location | `cyberos/docs/skills/cuo/<role>/<skill-id>/` | `cyberos/docs/contracts/<contract-id>/` |
| Entry file | `SKILL.md` | `CONTRACT.md` |
| Frontmatter | 33 fields (Part 2) | ~10 fields (much smaller) |
| Has `expects/produces`? | yes | **no** |
| Has `allowed_mcp_tools`? | yes | **no** |
| Has `confidence_band`? | yes | **no** |
| Has `audit.row_kind`? | yes | **no** (a contract isn't an action) |
| Versioned how? | SKILL.md `skill_version` | CONTRACT.md `contract_version` (frontmatter field; layout is flat per registry v0.2.4) |
| Bumped how often? | every CHANGELOG entry | rarely; bumps cascade to every consumer |

### 8.2 How a skill consumes a contract

```yaml
# In the skill's SKILL.md frontmatter:
depends_on_contracts:
  - id:        feature-request          # contract folder name
    version:   v1                        # locks to this major version
    purpose:   generation_skeleton       # human-readable: why this skill needs it
    pin_path:  cyberos/docs/contracts/feature-request/
```

The validator confirms three things: the path resolves to a real `CONTRACT.md`; the skill body's references to that contract use the declared path; on contract MAJOR bumps, every declared consumer is updated (or explicitly opts in to staying on the older version with a CHANGELOG entry).

### 8.3 Three contract kinds

| `contract_kind` | Schema body lives in | Example |
| --- | --- | --- |
| `artefact_schema` | `template.md` (Markdown skeleton) | `feature-request@v1` (the FR template) |
| `envelope_schema` | `schema.json` (JSON Schema) | a hypothetical `pipeline-checkpoint@v1` envelope contract |
| `wire_protocol` | `schema.json` + `protocol.md` | `nats-subjects@v1` (subject names + payload shapes for every NATS subject CyberOS skills emit; first concrete wire_protocol contract, registered v0.2.2) |

### 8.4 Why the split matters for portability

When a skill ships to Antigravity / Codex / a partner connector, the build pipeline needs to package skill + its contract dependencies as one bundle. `depends_on_contracts:` makes that machine-readable. Without it, you'd need to grep skill bodies for path strings. The `_contracts/` namespace turns "what does this skill need to function" from documentation into a build-time artefact.

---

## Part 9 — Host-adapter strategy

The CCSM (this SKILL.md) is the source of truth. Per-host artefacts are generated by transpilers. One source compiles to many surfaces without behavioural drift.

![Host adapter pipeline](./assets/diagrams/08-host-adapter-pipeline.svg)

### 9.1 Phased delivery

| Phase | Milestone | Effort | Status |
| --- | --- | --- | --- |
| **A — CCSM lock** | Frontmatter contract finalised; `dist/` is generated, never hand-edited. | 1 week | ✅ done in v0.2.0 (this README is the source of truth). |
| **B — Transpilers** | One transpiler per output target (anthropic / mcp / plugin / antigravity / codex / cursor). Each is a pure function `CCSM → host-artefact-tree`. | 2–3 weeks | 🔵 planned for v0.3.0. |
| **C — Host shim** | A library (`cyberos-skill-runtime` Python + `@cyberos/skill-runtime` Node) every transpiled skill links against. Provides `brain.*`, `audit.*`, `invariants.*` semantics regardless of host. | 1–2 weeks | 🔵 planned for v0.3.0. |
| **D — Equivalence test matrix** | Golden input/output runs across every target. CI gate. | 1 week | 🔵 planned for v0.3.0. |
| **E — Partner connector pipeline** | Hosted MCP + tenancy + OAuth + billing for `partner_connector: true` skills. | 4+ weeks | 🟣 planned for v0.4.0. |

Realistic critical path: ~5 weeks of focused work to get fr-author + fr-audit running on Anthropic + MCP + Antigravity. Each additional host costs days, not weeks, after the shim ships.

### 9.2 What the shim provides (uniform semantics across hosts)

The shim exposes a uniform API: `cyberos.runtime.brain` reads/writes `.cyberos-memory/` (host-agnostic; just FS), `cyberos.runtime.audit` appends to `genie.action_log` OR a local JSONL fallback, `cyberos.runtime.invariants` runs `INVARIANTS.md` checks at declared checkpoints, `cyberos.runtime.envelope` validates expects/produces against schema, `cyberos.runtime.untrusted` applies the AGENTS.md §4.2 marker scan, and `cyberos.runtime.scope` enforces SRS §6.4 scope contract. Inside CyberOS the shim talks to real `kb.*`, `brain.*`, `audit.*` MCP servers. Outside CyberOS the shim falls back to filesystem-local `.cyberos-memory/` and a local JSONL — degraded but functional.

### 9.3 Adapter-strategy summary table

| Host | Discovery path | Frontmatter dialect | Memory fallback | Adapter status |
| --- | --- | --- | --- | --- |
| Claude Code | `~/.claude/skills/` | Anthropic SKILL.md | filesystem `.cyberos-memory/` | 🔵 v0.3.0 |
| Antigravity | `.gemini/antigravity/skills/` | (investigate; likely SKILL.md-compatible) | filesystem `.cyberos-memory/` | 🔵 v0.3.0 |
| Codex | `~/.codex/instructions/` | Codex agent format | filesystem `.cyberos-memory/` | 🔵 v0.3.0 |
| Cursor | `.cursorrules` | concat-style | (none — read-only) | 🔵 v0.3.0 |
| Vanilla MCP | served via stdio/HTTP | MCP `tool.json` | partner-side | 🔵 v0.3.0 |
| CyberOS native | `cyberos/docs/skills/` | full v0.2.0 SKILL.md | full BRAIN | ✅ today |

---

## Part 10 — Build a skill: step-by-step

### 10.1 Path 1: copy hello-world (10 minutes, Tier 1)

```bash
SKILL_NAME=daily-headline
PERSONA=cpo

# 1. Copy hello-world as scaffold
cp -r cyberos/docs/skills/cuo/_shared/hello-world \
      cyberos/docs/skills/cuo/$PERSONA/$SKILL_NAME

# 2. Update frontmatter — name, owner_role, body
cd cyberos/docs/skills/cuo/$PERSONA/$SKILL_NAME
sed -i '' "s/name: hello-world/name: $SKILL_NAME/"      SKILL.md
sed -i '' "s/owner_role: _shared/owner_role: $PERSONA/" SKILL.md

# 3. Replace body with what your skill actually does
$EDITOR SKILL.md

# 4. Update envelopes
$EDITOR envelopes/input.json
$EDITOR envelopes/output.json

# 5. CHANGELOG.md v0.1.0 entry
cat > CHANGELOG.md <<EOF
# CHANGELOG — \`cuo/$PERSONA/$SKILL_NAME\`

## v0.1.0 — $(date +%Y-%m-%d) (initial)

### Added
- \`SKILL.md\` — <one-line summary>.
EOF

# 6. Register in two places:
#    - cyberos/docs/skills/cuo/$PERSONA/SKILL.md "Owned workflow skills" table
#    - cyberos/docs/skills/README.md Part 23 index
```

That's a Tier 1 skill. Routable, auditable, chainable.

### 10.2 Path 2: promote to Tier 2 (production-ready)

After your Tier 1 skill has run for a week, do six things. Add `INVARIANTS.md` with at least 3 invariants the skill should never violate. Add `STANDALONE_INTERVIEW.md` with the 3-5 questions the supervisor should ask in chat-mode entry. Add `HUMAN_SUMMARY.md` with the chat-rendered summary template. Fill the v0.2.0 frontmatter blocks: `invocation_modes`, `exposable_as`, `self_audit`, `human_fine_tune`, and `depends_on_contracts` if the skill consumes any contracts. Bump to v0.2.0 with a CHANGELOG entry citing the registry v0.2.0 contract expansion as the driver. Add 2-3 acceptance fixtures under `acceptance/` (golden input + golden output).

### 10.3 Path 3: build from scratch (no scaffold)

Use `cuo/_shared/hello-world/` as the **reference** but author the new skill from blank. Useful when the new skill doesn't resemble hello-world's shape (e.g., it's chained-only, or it produces multiple artefacts per invocation).

### 10.4 Body skeleton — recommended structure

```markdown
# <skill-name>

> One-paragraph summary. What this skill does, when it should be invoked, and what it produces. Mention chaining if relevant.

`prompt_revision: <skill-name>@<MAJOR>.<MINOR>.<PATCH>`

## When to invoke this skill

CUO routes here when the user wants to:

- "<natural language phrase 1>"
- "<natural language phrase 2>"
- "<natural language phrase 3>"

If the user wants <related skill>, route to <related skill> instead.

## Self-test preamble — emit BEFORE any file action

Begin every invocation with a single fenced `CONTRACT_ECHO` block.

\`\`\`
CONTRACT_ECHO
skill_id:        cuo/<role>/<skill-name>
skill_version:   <SemVer>
phase:           <whatever phases the skill has>
inputs:          <listing>
\`\`\`

## Pipeline interface

[Document expects/produces envelope shapes with examples.]

## Phase 1 — <PHASE NAME>

[Numbered steps the LLM follows.]

## MUST / MUST NOT / SHOULD

[Hard rules.]

## Failure modes

See `references/FAILURE_MODES.md`.

## Citations

- Source artefact → ...
- Persona inheritance → `cuo/<role>/SKILL.md`.
- BRAIN scope contract → SRS §6.4.
```

---

## Part 11 — Worked example end-to-end: fr-author → fr-audit

The canonical chain. Walk through it once and you understand the whole architecture.

![fr-author → fr-audit chain sequence](./assets/diagrams/11-fr-author-fr-audit-chain-sequence.svg)

### 11.1 What happens, narrated

A user types in CHAT: *"Turn this PRD into a backlog and audit it."* The supervisor's `classify_act` node returns `{persona_id: cuo-cpo, skill_id: cuo/cpo/fr-author, confidence: 0.93}`. The supervisor synthesises the input envelope (it's chat-mode entry; `STANDALONE_INTERVIEW.md` runs to fill `requirements_files`; the rest defaults). It invokes `fr-author`. The skill enters PLAN phase: reads the PRD with sequential pagination (per AGENTS.md §4.10), enumerates feature requests, runs INV-003 (ingestion-coverage check). PLAN appends one `row_kind: question` row to `genie.action_log` and emits the proposed FR backlog as a Question primitive. The supervisor halts, surfaces the backlog to the user via `HUMAN_SUMMARY.md`. The user replies "APPROVE."

The supervisor resumes from the LangGraph checkpoint. `fr-author` enters WORKER phase: writes FR-001, FR-002, FR-003 to disk, computing each FR's hash and appending three `row_kind: artefact_write` rows to action_log. Output envelope sets `next_skill_recommendation: cuo/cpo/fr-audit`. The supervisor's conditional edge fires; it invokes `fr-audit` with `{fr_paths: [...]}` and the upstream context. `fr-audit` runs its 8-step audit loop against `audit_rubric@2.0`, checking INV-001 (verdict determinism — sev-0). All 3 FRs PASS; three `row_kind: artefact_write` rows are appended for the audit reports. The chain closes; `HUMAN_SUMMARY` renders to chat: *"Audit complete — 3/3 PASS. Reports at FR-001.audit.md, FR-002.audit.md, FR-003.audit.md. Trace: <uuid>."*

### 11.2 Why this example is the canonical one

It exercises every contract: dual-mode (standalone entry via interview), chain (fr-author → fr-audit), audit-hook (7 action_log rows correlated by trace_id), self-audit (INV-003 in fr-author, INV-001 in fr-audit), pipeline interface (envelope handoff), human-in-the-loop (PLAN approval gate), and persona scope (both skills under cuo/cpo, sharing the persona's escalation graph). If you can read this trace and explain every row, you understand CyberOS skills.

### 11.3 What the action_log looks like

```sql
SELECT audit_id, ts, persona, op, skill_id, row_kind, LEFT(reason, 60) AS reason
FROM genie.action_log
WHERE trace_id = 'a1b2c3d4-...'
ORDER BY ts;
```

Returns 7 rows for this run: one `question` (PLAN approval), three `artefact_write` (FR-001..003 from fr-author), three `artefact_write` (audit reports from fr-audit). Every row's `chain` field equals `sha256(canonical_json(row) + prev_row.chain)` per AGENTS.md §7.2 — tampering breaks the chain.

---

## Part 12 — Runtime architecture: LangGraph + action_log + NATS

### 12.1 The three runtime layers

The CyberOS runtime is three layers stacked. **Layer 1 — the LangGraph supervisor** (per SRS §6.1.1, DEC-027) runs an Observe-Decide-Act loop. The `classify_act` node calls a Haiku-class router (PRD §6.3) that returns `{persona_id, skill_id, confidence}`. Conditional edges route between skill nodes based on the previous output's `next_skill_recommendation` field. State (envelopes, in-flight FR hashes, HITL pause tokens) is checkpointed to `genie.graph_checkpoint` after every node — chains are crash-safe and resumable.

**Layer 2 — `genie.action_log`** (per SRS §6.7) is the append-only Postgres table where every skill output gets a row. Schema: `(audit_id, ts, persona_id, skill_id, skill_version, row_kind, target, payload_sha256, explanation_pane_ref, confidence, hash_chain_prev, hash_chain_self, trace_id, cc_personas, correction_to)`. The hash chain is canonical-JSON over the row minus the chain field, prepended to the previous row's chain. The CP module's tamper detector (SRS §10.4.6) runs continuously and surfaces any chain break as a Notify primitive routed to the security oncall.

**Layer 3 — NATS event bus** (DEC-029) carries fire-and-forget events between skills that don't need direct chaining. Subjects follow the pattern `cuo.<skill-id>.<event-name>` (e.g., `cuo.fr_author.fr_written`). Subscribers (other skills, OBS metrics, downstream pipelines) consume the event without coupling to the producer's invocation lifecycle. NATS is **not** a substitute for LangGraph chaining — it complements it. Use NATS for "tell me when X happened"; use LangGraph for "now run Y."

### 12.2 How a skill invocation flows

A skill invocation has six stages. **Pre-invocation:** the supervisor validates the input envelope against `expects.schema_ref` (Layer 1 mechanical validation). The scope contract is enforced — `allowed_mcp_tools` and `allowed_brain_scopes` are intersected with the caller persona's ceiling. **Invocation:** the supervisor pushes the LangGraph state, invokes the skill's body. The skill's MCP tool calls go through the gateway, which enforces the per-skill `allowed_mcp_tools` allowlist. Every BRAIN read/write goes through the BRAIN MCP server, which enforces `allowed_brain_scopes`. **In-flight checks:** at every node boundary, the runtime runs the skill's `INVARIANTS.md` (per `self_audit.check_at`). Anomaly streaks update; threshold trips emit `refinement_proposal`. **Post-invocation:** the output envelope is validated against `produces.schema_ref`. Each concrete output (artefact write, Question, Review, Notify) gets one `genie.action_log` row appended atomically with the side-effect. **Chaining:** if `next_skill_recommendation` is set, the supervisor's conditional edge fires, routing to the next skill with the output envelope as input. **Closure:** the supervisor pops the LangGraph state, releases the checkpoint, and emits the final `HUMAN_SUMMARY` to chat (standalone mode) or rolls into the parent chain's summary (chained mode).

### 12.3 Crash recovery

A skill run can crash at three points: between node boundaries (nothing committed; the next session start re-enters at the last checkpoint), mid-write (the AGENTS.md §4.4 two-phase atomic write means the file either lands fully or not at all; crash = stale `.tmp.*.part` file the next session start unlinks), or mid-action_log append (the database transaction either commits or rolls back; partial writes are impossible). Reconciliation per AGENTS.md §4.7 runs at session start: walk audit rows newer than the last `consolidation_run`, verify file existence + hash match, freeze writes against any path with a hash mismatch.

---

## Part 13 — Validate & debug

### 13.1 The three-layer validation pyramid

Stack from cheapest to most thorough.

![Validation pyramid](./assets/diagrams/09-validation-pyramid.svg)

### 13.2 Layer 1 — mechanical (run on every output)

```bash
ajv validate \
  -s cuo/cpo/my-skill/envelopes/output.json \
  -d ./skill-output-from-test-run.json
```

If this fails, the skill produced something structurally invalid. Fast, deterministic, no LLM judgement.

### 13.3 Layer 2 — functional (CI regression tests)

Every skill ships an `acceptance/` folder with golden input/output pairs. For deterministic skills (`determinism.reproducible: true`), the comparison is byte-equal: `diff <(./run-skill cuo/_shared/hello-world < golden-input.json) golden-output-stephen.md`. Empty diff = pass. For LLM-judgement skills (most production skills), use a fuzzy similarity threshold — embedding cosine ≥0.95 is the default; tune per skill based on observed false-positive vs. false-negative tradeoff.

### 13.4 Layer 3 — operational (production telemetry)

```sql
SELECT
  COUNT(*)                                                AS invocations,
  AVG(CASE WHEN reaction = 'accepted' THEN 1.0 ELSE 0.0 END)
                                                          AS acceptance_rate,
  COUNT(*) FILTER (WHERE row_kind = 'question')           AS hitl_pauses,
  COUNT(*) FILTER (WHERE row_kind = 'self_refinement_proposal')
                                                          AS auto_refinements,
  AVG(audit_iteration_count)                              AS avg_iterations
FROM genie.action_log
WHERE skill_id = 'cuo/cpo/my-skill'
  AND ts > now() - interval '7 days';
```

Healthy thresholds: acceptance rate ≥80% (concerning at 40-80%, auto-pause at <40% per DEC-055), HITL frequency <20% (concerning at 20-40%, refine prompt at >40%), auto-refinement 0-1 per week (concerning at 2/week, escalate to manual fine-tune at ≥3/week per Part 7), average iterations ≤2 (concerning at 2-4, slow convergence at >4).

### 13.5 Three debug queries to memorise

**"What did this trace_id actually do?"**

```sql
SELECT audit_id, ts, persona, op, skill_id, row_kind, path,
       LEFT(reason, 100) AS reason
FROM genie.action_log
WHERE trace_id = 'a1b2c3d4-…'
ORDER BY ts;
```

**"Was the chain tampered with?"**

```sql
SELECT audit_id, prev_chain, chain,
       LAG(chain) OVER (ORDER BY ts) = prev_chain AS chain_intact
FROM genie.action_log
WHERE trace_id = 'a1b2c3d4-…'
ORDER BY ts;
```

Any `chain_intact = false` → broken chain → tampering or bug. SRS §10.4.6.

**"Why did this skill emit a refinement_proposal?"**

```sql
SELECT audit_id, ts, skill_id, payload_data->'trigger',
       payload_data->'observation', payload_data->'proposed_amendments'
FROM genie.action_log
WHERE row_kind = 'self_refinement_proposal'
  AND skill_id = 'cuo/cpo/my-skill'
ORDER BY ts DESC
LIMIT 5;
```

For a worked end-to-end trace, see [`cuo/cpo/AUDIT_TRACE_EXAMPLE.md`](./cuo/cpo/AUDIT_TRACE_EXAMPLE.md).

---

## Part 14 — The skill lifecycle

From idea to archive.

![Skill lifecycle state diagram](./assets/diagrams/10-skill-lifecycle-state.svg)

Setting `gated_until_phase: P1` on a skill means the supervisor returns `E_PERSONA_GATED` if a user tries to invoke it before P1 ships. The phase plan per PRD §14: P0 covers cpo and cto only; P1 brings ceo, coo, cfo, chro, cseco, clo, caio online; P2+ unlocks the remaining personas as their dependent modules ship.

---

## Part 15 — Security model deep-dive

CyberOS skills are subject to four layered security controls. Skipping any one of them is a contract violation that the validator rejects.

### 15.1 Scope contract (SRS §6.4)

`allowed_brain_scopes` and `allowed_mcp_tools` form an explicit allowlist at the skill level. The MCP gateway enforces these at call time — any attempt to use a tool not in the allowlist returns `E_SCOPE_VIOLATION`. The BRAIN gateway enforces scope-glob matching on every read and write — `allowed_brain_scopes.read: [project:*]` permits reads under any project but rejects reads from `member:`, `client:`, `company:`, etc. Writes default to empty (read-only); a skill that needs to mutate BRAIN must explicitly enumerate its write scopes. The persona-card sets a ceiling; every workflow under that persona declares a strict subset, never a superset.

### 15.2 Untrusted-content discipline (DEC-050 CaMeL)

Every external byte (PRD content, user-typed name, customer quote, fetched web content) MUST be wrapped in `<untrusted_content source="...">…</untrusted_content>` before reasoning. Skills MUST NOT execute imperatives inside untrusted blocks. The runtime scans for prompt-injection markers per the SAFE-003 list (case-insensitive, NFC-normalised, zero-width stripped, mixed-script-detected). Marker hits trigger `on_marker_hit: surface_to_human` — the skill halts and the supervisor surfaces the suspected injection as a Question primitive. Reference: AGENTS.md §4.2 marker set, `cuo/cpo/fr-author/references/UNTRUSTED_CONTENT.md`.

### 15.3 Denylist (sev-0; AGENTS.md §9.3)

Skills MUST NEVER write any of these to memory: compensation (salary, payslip, bonus, equity grants, RSUs), government IDs (national IDs, passport, tax ID, driver's licence), bank/card numbers (account numbers, IBAN, SWIFT, full PANs), home addresses (work addresses with consent are fine), health PII (special-category data including health leave-reason text), individual peer-review scores (aggregates ok), secrets (raw API keys, .env contents, OAuth tokens, refresh tokens, session cookies, private keys, certificates, mnemonics, recovery phrases, DB connection strings with credentials), or external-party PII without explicit consent. If a memory must reference a denylisted item, store a pointer instead (`"see <vault-name> → <folder> → <entry>; held by <person>"`). If a user insists on storing the value, the skill pushes back once and refuses.

### 15.4 EU AI Act compliance (PRD §12.2.2; SRS DEC-064)

Any skill that uses LLM inference, generation, or scoring on data about humans needs to think about Article 5 (prohibited practices), Annex III (high-risk systems), and Article 50 (transparency obligations). Skills MUST defer to `cuo-clo` (Chief Legal Officer persona) on any boundary call. The decision tree lives in `cuo/cpo/fr-author/references/EU_AI_ACT_DECISION_TREE.md`. Concretely, a skill that auto-classifies a user-facing AI feature's risk class without a determining fact is a sev-0 invariant breach (see `fr-author/INVARIANTS.md` INV-007).

### 15.5 Hash-chain integrity (SRS §10.4.6)

Every skill's audit row participates in the `genie.action_log` hash chain. Tampering — modifying a row, deleting a row, reordering rows — breaks the chain and is detected by the CP module's continuous tamper detector. A broken chain emits a sev-0 Notify to the security oncall. The hash chain is what makes CyberOS auditable in the EU AI Act Article 12 sense (logging requirement). It is non-negotiable.

---

## Part 16 — Performance & observability

### 16.1 Performance budget per layer

A skill invocation has a typical latency budget. **Pre-invocation** (envelope validation + scope check) takes <50ms. **Body execution** is dominated by LLM inference — Haiku-class for routing and judgement is ~500ms per call; Sonnet/Opus for heavier work is 2-10s; deterministic skills with no inference are <100ms. **Invariants check** at each node boundary is ~30ms for 8 invariants (proportional to invariant count × cost-per-check). **Audit row append** is <10ms (Postgres single-row insert with hash compute). **Post-invocation** (envelope validation + chain dispatch) is <20ms.

Expect a typical chat-mode `fr-author` PLAN-phase run to take 3-8 seconds end-to-end (dominated by Sonnet/Opus inference reading the PRD and enumerating FRs). A WORKER-phase FR generation is ~5-15s per FR. An `fr-audit` run is ~2-5s per FR (mostly mechanical rule checks; only a few rules need LLM judgement).

### 16.2 Observability — what to monitor

Per skill, OBS (the observability module per SRS §6.12) tracks five primary metrics. **`acceptance_rate`** — fraction of outputs the user accepted (versus corrected, ignored, or rejected). Drops below 40% over 7 days auto-pause the skill (DEC-055). **`hitl_pause_rate`** — fraction of invocations that emitted a Question primitive. Above 40% indicates the skill is asking too often; refine the prompt. **`avg_iteration_count`** — for skills that loop (e.g., `fr-audit`'s per-FR audit loop), how many iterations to convergence. Above 4 indicates slow convergence. **`refinement_proposal_rate`** — auto-refinement frequency. ≥2 per week per skill triggers manual fine-tune escalation. **`drift_signal_count`** — anomaly signals (confidence-low streaks, user-correction streaks, etc.) firing per 7 days. ≥3 triggers a Notify.

### 16.3 Logging conventions

Every skill output produces exactly one `genie.action_log` row — that's the canonical log. Skills SHOULD NOT write parallel log streams; instead, populate the row's `payload_data` and `reason` fields richly. The `reason` field is ≤200 chars present-tense citing the source (e.g., "fr-author wrote FR-007 from PRD §4.2 lines 110-145; coverage 0.99"). The `payload_data` field is the full JSON of the produced artefact (truncated to 64 KB; longer artefacts get a hash-only row).

### 16.4 Tracing

Every chained invocation carries a `trace_id` (UUIDv7) through every action_log row. Reconstructing a chain is `SELECT * FROM genie.action_log WHERE trace_id = '...' ORDER BY ts`. The `cc_personas` field (DEC-052) annotates rows where the active persona's action implicates other personas (e.g., a CHRO action that touches comp gets CFO and CLO listed). The CC is informational; it doesn't change who acted.

---

## Part 17 — Localization & i18n

CyberSkill operates in Vietnam with English-default deliverables. Skills support multilingual operation in three layers.

### 17.1 Language at the manifest level

`manifest.languages: [en, vi]` declares supported languages. `manifest.language_routing_default: en` is the fallback. The CHAT primitive detects the user's language (per their browser locale or the language of their first message) and the supervisor passes `caller_language` in the input envelope. Skills SHOULD branch their `HUMAN_SUMMARY.md` rendering on `caller_language` — render Vietnamese summaries to Vietnamese-speaking users.

### 17.2 Language at the body level

Skill bodies are written in English (the engineering lingua franca). The interview Q&A in `STANDALONE_INTERVIEW.md` SHOULD include Vietnamese translations as parenthetical or bilingual. The `HUMAN_SUMMARY.md` template SHOULD include both English and Vietnamese rendering paths. The audit_log `reason` field is always English (it's machine-readable; humans translate at display time).

### 17.3 Artefact language

When a skill produces an artefact (an FR, a tech spec, a report), its language matches the input language. fr-author reads a Vietnamese PRD and writes Vietnamese FR markdowns. The audit rubric's mechanical rules (FM-001..111, SEC-001..009) are language-neutral; the LLM-judgement rules (QA-009 plain-English check) need a Vietnamese-equivalent rule (QA-009-vi) when auditing Vietnamese FRs. This is a known gap; the rubric expansion to Vietnamese is a v0.3.0 follow-up.

---

## Part 18 — Anti-patterns: what NOT to do

Patterns that look reasonable but break CyberOS contracts.

**Don't write skills that call other skills directly.** All skill-to-skill handoffs go through the supervisor (which writes the action_log row, applies the scope contract, validates the envelope schemas). Direct calls break audit and chain-of-custody. If you need shared logic, put it in `scripts/` inside the skill folder.

**Don't conflate "skill" with "schema."** A skill *acts*; a contract *constrains*. If your "skill" has empty `allowed_mcp_tools: []`, `expects: null`, and `confidence_band: 1.0`, it's a contract wearing a skill costume. Promote it to `cyberos/docs/contracts/` per Part 8.4 + Recipe 7.

**Don't hard-code paths to other skills or contracts in the body.** Use `depends_on_contracts:` for contract dependencies. Use `next_skill_recommendation` for chain targets. Hard-coded paths break extraction and bundling.

**Don't suppress the action_log row.** Every concrete output must produce exactly one row. "Skipping" the row to "make it cleaner" breaks tamper detection and is a sev-0 contract violation.

**Don't write a 500-line SKILL.md body.** Use progressive disclosure. The body is the system prompt; reference docs go in `references/`. A 300-line body is the soft cap.

**Don't promote an LLM-inferred fact to `confidence: 1.0`.** AGENTS.md §5.2 caps LLM-inferred at 0.7. Authority is human-edited > human-confirmed > llm-explicit > llm-implicit; never promote.

**Don't auto-set `eu_ai_act_risk_class: minimal` without a determining fact.** When in doubt, escalate to `cuo-clo`. INV-007 in `fr-author/INVARIANTS.md` makes this an enforced invariant.

**Don't write to `.cyberos-memory/` outside the BRAIN MCP gateway.** Direct file writes bypass the AGENTS.md §4.1 path-traversal guard, the §4.2 content gate, and the §4.4 two-phase atomic write. Always go through `brain.write_memory`.

**Don't change RUBRIC.md mid-batch.** fr-audit's INV-007 is sev-0. The runtime hashes the rubric at batch start and verifies before each FR audit. A change mid-batch aborts with `RUBRIC_CHANGED_MID_BATCH`.

**Don't set `partner_connector: true` without a separate DEC.** The validator enforces the trust↔exposability link (Part 5.3) plus a per-skill DEC. Partner exposure has SLA, billing, and tenancy implications that need explicit governance.

**Don't paste full SHA-256 hashes in chat.** First 12 hex chars + ellipsis. Full hashes go in audit rows and machine-readable contexts only.

**Don't bypass `STANDALONE_INTERVIEW.md` to "save time."** Skills that hard-code defaults and skip the interview break user expectation that they can override defaults. The interview pattern is what makes dual-mode work.

**Don't over-specify a new contract beyond what consumers actually do.** When you register a contract to capture a previously-undocumented convention (e.g. NATS subject names that skills already emit), the temptation is to add structural rules that "sound right" — sub-persona namespacing, field-naming hierarchies, payload-versioning schemes the skills don't actually produce. The first draft of `nats-subjects@1` (registry v0.2.2) made this mistake: contract said `<sub-persona>.<skill>.<event>` (e.g. `cuo_cpo.fr_author.fr_written`); reality has always been `<top-level-persona>.<skill>.<event>` (e.g. `cuo.fr_author.fr_written`). The audit-fix-audit loop caught the drift before merge. **Rule:** when documenting a pre-existing convention, grep the consuming skill bodies for the exact form before writing the contract; reality wins. See REF-016 in BRAIN. The audit-fix-audit discipline (audit → fix → re-audit until clean) is mandatory after every new contract registration; see Recipe 13.

---

## Part 19 — Cookbook: 13 recipes

### Recipe 1 — Build my first skill in 10 minutes

See [§10.1](#101-path-1-copy-hello-world-10-minutes-tier-1).

### Recipe 2 — Chain skill A into skill B

In skill A's output envelope, set `"next_skill_recommendation": "cuo/cpo/skill-b"`. In skill A's `envelopes/output.json`, document the field with `default: "cuo/cpo/skill-b"`. The supervisor's LangGraph conditional edge does the routing. Verify by running skill A and observing two action_log rows with the same `trace_id`.

### Recipe 3 — Debug a wrong-output skill

```bash
psql -c "SELECT trace_id, audit_id, ts, payload_data
         FROM genie.action_log
         WHERE skill_id = 'cuo/cpo/my-skill'
           AND ts > now() - interval '1 day'
           AND row_kind = 'artefact_write'
         ORDER BY ts DESC LIMIT 5;"
```

Read the offending payload to identify the failure mode. Add a regression case under `acceptance/`. Edit SKILL.md body to handle the case (or add an INVARIANTS.md entry). Bump version + CHANGELOG entry. Re-run; confirm regression case now passes.

### Recipe 4 — Promote a skill from v0.1.x to v0.2.0

Add the v0.2.0 frontmatter blocks per Part 2.1: `invocation_modes`, `depends_on_contracts` (if any), `exposable_as`, `self_audit`, `human_fine_tune`. Add subfields: `expects.optional_fields`, `expects.standalone_interview_ref`, `produces.human_summary_ref`. Author the three new files: `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md`, `INVARIANTS.md` (≥3 invariants). Bump `skill_version` 0.1.x → 0.2.0. Add CHANGELOG entry citing registry v0.2.0 as the driver.

### Recipe 5 — Retire an old skill

Build the replacement under a new name (e.g., `my-skill-v2`). Mark the old skill superseded with `superseded_by: cuo/cpo/my-skill-v2` in its frontmatter. Run them in parallel for one phase. When v2's acceptance ≥ v1's, retire v1 to `_archive/` via `git mv cyberos/docs/skills/cuo/cpo/my-skill cyberos/docs/skills/cuo/cpo/_archive/my-skill`. Document in the persona CHANGELOG. The body is preserved per AGENTS.md §4.6 (soft-delete). Audit history remains in `genie.action_log`.

### Recipe 6 — Add a new sub-persona

```bash
mkdir -p cyberos/docs/skills/cuo/clo
cp cyberos/docs/skills/cuo/cpo/SKILL.md cyberos/docs/skills/cuo/clo/SKILL.md
$EDITOR cyberos/docs/skills/cuo/clo/SKILL.md
# Edit: name, owner_role, voice deltas, escalation, gated_until_phase: P1
```

Add a CHANGELOG entry. Update `cuo/README.md` index. Add the first workflow under `clo/` when the persona is ready to operate.

### Recipe 7 — Promote a `_shared/` skill to a contract

Use case: a "skill" has empty `allowed_mcp_tools`, `expects: null`, `confidence_band: 1.0` — it's a schema, not a skill.

```bash
mkdir cyberos/docs/contracts/<id>/
git mv cyberos/docs/skills/cuo/_shared/<skill-id>/template.md \
       cyberos/docs/contracts/<id>/template.md
git mv cyberos/docs/skills/cuo/_shared/<skill-id>/SKILL.md \
       cyberos/docs/contracts/<id>/CONTRACT.md
# Trim CONTRACT.md frontmatter — drop skill-only fields, add contract-only.
# In CONTRACT.md frontmatter, set: contract_version: v1
git rm -r cyberos/docs/skills/cuo/_shared/<skill-id>
```

Update every consumer skill: add `depends_on_contracts:` + update body refs. Update `cyberos/docs/contracts/README.md` index. The `feature-request` contract was promoted exactly this way in registry v0.2.0 — see `cyberos/docs/contracts/feature-request/CHANGELOG.md` for the canonical example.

### Recipe 8 — Set up acceptance fixtures for a new skill

Create `cuo/<role>/<skill>/acceptance/` and add three files. `golden-input.json` — a known input envelope. `golden-output-<scenario>.md` — the expected artefact (or `golden-envelope-<scenario>.json` for envelope outputs). `README.md` — explains each fixture's scenario. Run `ajv validate -s envelopes/output.json -d golden-envelope-<scenario>.json` to sanity-check the fixture itself. Add 1-3 fixtures covering happy path + 1-2 edge cases. Wire into CI when the test harness lands.

### Recipe 9 — Write an INVARIANTS.md

Identify 3-8 truths the skill enforces about its own behaviour. Each invariant has ID + Statement + Check + Severity + Refinement template. Start with the most universal: `INV-confidence-band-reporting` (every output's `confidence` is in [0.0, 1.0]) and `INV-scope-discipline` (no write outside declared `allowed_brain_scopes`). Add skill-specific invariants the skill's contract makes salient — e.g., `fr-audit`'s `INV-001 verdict-determinism` is the auditor's reproducibility promise. Severity = `error` for sev-0; `warning` for advisory; `info` for telemetry-only.

### Recipe 10 — Write a refinement_proposal that humans actually approve

Make the proposal actionable. Cite the exact section to amend. Propose the exact prose change as a unified diff. Include the observation as facts (numbers, file paths, line numbers) — not interpretation. State the `minimum_viable: <one-line MVA recommendation>` so the human can choose between full adoption and minimal patch. Vague proposals get rejected; specific proposals get approved 80%+ of the time.

### Recipe 11 — Plan a skill promotion (v0.x → v1.0)

The Mature → v1.0 transition needs four checks. Acceptance ≥80% over 4 consecutive weeks. Zero open auto-refinement proposals on the same theme (the skill has stabilised). Acceptance fixtures cover the happy path + ≥3 edge cases. CHANGELOG has a clear ### Driver section explaining the maturity claim. Once green: bump from 0.x to 1.0.0 with a "promoted to mature" entry. The skill is now eligible for partner exposure (subject to the per-skill `partner_connector` DEC).

### Recipe 12 — Run a fine-tune cycle (the 7-step playbook)

See [Part 7.2](#72-the-7-step-playbook). Expected duration: 2-4 hours for a focused cycle on one skill. The diagnose step (clustering action_log failures by mode) is usually the slowest — budget 30-60 minutes for that alone.

### Recipe 13 — Register a new contract with the audit-fix-audit discipline

Mandatory after every new contract registration. Running this loop on `nats-subjects@1` in registry v0.2.2 caught a real contract-vs-reality drift before merge — the cost of running the loop (~5 minutes) is much smaller than the cost of shipping a contract that diverges from what consumers actually do.

**Step 1 — Author the first draft.** Create `cyberos/docs/contracts/<id>/CONTRACT.md` (with `contract_version: v1` in frontmatter), `schema.json` (or `template.md`), `protocol.md` (wire_protocol only), `CHANGELOG.md`. Pick the convention the contract documents (subject names, payload shapes, frontmatter fields, etc.).

**Step 2 — Audit pass 1: grep consumer skill bodies for the convention as the contract describes it.** Use the contract's exact form in the grep. If the grep returns nothing, or returns the wrong form, the contract has drifted from reality and needs correction. Real example from v0.2.2: contract said `cuo_cpo.fr_author.fr_written`; grep against fr-author's body returned `cuo.fr_author.fr_written`. Reality wins. Update the contract.

**Step 3 — Fix.** Update the contract files (CONTRACT.md inventory + naming convention prose, schema.json descriptions, protocol.md prose, CHANGELOG.md historical claims). Be exhaustive — include the description fields in JSON Schema, not just the inventory tables. Strings appear in surprising places.

**Step 4 — Audit pass 2.** Re-grep with the new form. Look for residual references to the old form. Look for cross-document inconsistency (e.g., CONTRACT.md table updated but CHANGELOG.md historical narrative still uses the old form). Look for anchor-target mismatches if any document references another by header anchor.

**Step 5 — Fix any residuals from pass 2.**

**Step 6 — Audit pass 3 (verification).** This pass should be clean. If it isn't, return to step 5.

**Step 7 — Capture the lesson.** Write `memories/refinements/REF-NNN-<slug>.md` in BRAIN describing what the loop caught and the rule that prevents it next time. Append BRAIN audit rows + manifest update per AGENTS.md §4 + §7. Update the registry CHANGELOG entry's `### Driver` section to cite the audit-fix-audit rounds.

Expected duration: 5-15 minutes per contract. Budget more if the contract has many consumers or long inventory tables. The discipline scales sub-linearly: a contract with 20 subjects takes maybe 2× longer to audit than one with 9.

## Part 20 — Routing: how CUO picks a skill

### 20.1 The 14 sub-personas (locked: DEC-052)

| ID | Role | Phase available |
| --- | --- | --- |
| `ceo`  | Chief Executive Officer            | P1+ |
| `coo`  | Chief Operating Officer            | P1+ |
| `cfo`  | Chief Financial Officer            | P1+ |
| `cmo`  | Chief Marketing Officer            | P2+ |
| `cto`  | Chief Technology Officer           | P0  |
| `chro` | Chief Human Resources Officer      | P1+ |
| `cseco`| Chief Security Officer             | P1+ |
| `clo`  | Chief Legal Officer                | P1+ |
| `cdo`  | Chief Data Officer                 | P2+ |
| **`cpo`** | **Chief Product Officer**       | **P0** |
| `caio` | Chief AI Officer                   | P1+ |
| `cxo`  | Chief Experience Officer           | P2+ |
| `cro`  | Chief Revenue Officer              | P2+ |
| `cso-sustainability` | Chief Sustainability Officer | P3+ |

### 20.2 Routing rules

Per SRS §6.1.1, a request enters CUO's LangGraph and hits the `classify_act` node. The classifier returns `{persona_id, skill_id, confidence}`. Disambiguation rules: if the user names a persona explicitly ("ask the CFO…"), confidence override = 1.0; if the action implies a regulated domain (REW / LEARN / ESOP / compliance / legal), an automatic CC to the matching persona is added — the audit row's `cc_personas:` field; if multiple personas could plausibly own the request, escalate via the Question primitive; below `defer_below` confidence, surface "I'm not sure which workflow you mean — here are the candidates."

### 20.3 Eligibility filters

A skill is eligible for routing when ALL of: caller persona's `allowed_mcp_tools` ⊇ skill's `allowed_mcp_tools`, caller persona's `allowed_brain_scopes` ⊇ skill's `allowed_brain_scopes`, skill not in a paused state for this member, skill's `gated_until_phase` ≤ current phase. A failed classification escalates to the Question primitive — CUO asks the human which workflow they want.

---

## Part 21 — Per-persona quickstart

When each persona comes online, it brings its own scope contract + skill set + escalation graph. Quickstart pointers per persona:

**`cpo` (P0, today)** — owns FR backlog management. Two skills: fr-author, fr-audit. See `cuo/cpo/SKILL.md` for voice deltas (user outcomes over feature counts; one primary metric + one guardrail; out-of-scope is a feature; never auto-set EU AI Act risk class to minimal).

**`cto` (P0, today)** — owns tech-spec drafting and architecture review. First workflow: `fr-to-tech-spec` (planned, consumes `fr-author`'s output). See PRD §6.5 for voice.

**`cfo` (P1)** — owns cashflow projection, payroll narration, budget variance. Defers to `cuo-clo` on REW (right-to-erasure) writes per PRD §6.4.1. Defers to `cuo-cseco` on financial-data security boundaries.

**`chro` (P1)** — owns onboarding plans, performance-cycle prep, leave summaries. The denylist (Part 15.3) is *especially* relevant here — comp data, gov IDs, health PII are all forbidden. Skills under chro that need comp must use the pointer pattern.

**`clo` (P1)** — owns EU AI Act conformity, contract redline summaries. Receives every escalation from other personas on legal/compliance ambiguity. The most-CC'd persona in the audit log.

**`cseco` (P1)** — owns threat modelling, breach response. Receives every escalation on security boundaries. Skills under cseco have widest `allowed_brain_scopes.read` (security needs context) but tightest `allowed_brain_scopes.write`.

**`caio` (P1)** — owns model-card drafting, EU AI Act Annex IV packs, model-eval reviews. Heavy collaboration with clo on AI Act compliance.

**`cmo` (P2)** — owns campaign briefs, content calendars, attribution reviews. Skills here will be the first to expose `partner_connector: true` (marketing tooling integrations).

**`cdo` (P2)** — owns data-quality digests, lineage explainers, schema migrations. Heavy BRAIN write surface; cseco reviewer on every MAJOR.

**`cxo` (P2)** — owns NPS digests, journey-friction surfacing. Customer-facing artefacts (`client_visible: true` heavy).

**`cro` (P2)** — owns pipeline reviews, win/loss synthesis. Skills here often touch CRM data; client-scope writes are common.

**`cso-sustainability` (P3+)** — owns ESG roll-ups, scope-3 emissions narratives. Distant horizon; placeholder folder only today.

`ceo` and `coo` are P1 but largely write narrative artefacts (strategy memos, OKR roll-ups, weekly ops reviews). Their skills are mostly summarisation + framing on top of other personas' output.

---

## Part 22 — Migration from non-CyberOS skills

### 22.1 From an Anthropic-style flat SKILL.md

Take the existing flat `SKILL.md` (just `name` + `description` + body). Decide the owner persona — pick the closest of the 14 in Part 20.1 (or `_shared` if cross-persona). Create the folder `cyberos/docs/skills/cuo/<role>/<skill-id>/`. Move the SKILL.md into it. Promote frontmatter to Tier 1 (Part 2.3) — add `skill_version`, `persona`, `owner_role`, `allowed_brain_scopes`, `allowed_mcp_tools`, `escalation`, `expects`, `produces`, `audit`, `untrusted_inputs`. Add `envelopes/{input,output}.json`. Author CHANGELOG with a v0.1.0 entry citing the migration. The body stays intact — no need to rewrite.

### 22.2 From a Claude Code plugin

Plugins are bundles (skills + commands + agents + .mcp.json + manifest). Each skill in the plugin's `skills/` folder migrates as in §22.1. The `.mcp.json` server definitions become `allowed_mcp_tools` on the migrated skills. The plugin's `commands/` (slash-commands) become standalone skills under the same persona. The plugin's `agents/` (subagents) need separate consideration — most should be inlined into a single skill body, since CyberOS skills are atomic units of routing.

### 22.3 From a vanilla MCP tool

A vanilla MCP tool exposes `{name, description, inputSchema, outputSchema}`. Migrate by creating a CyberOS skill with `expects.schema_ref` pointing to the tool's `inputSchema` and `produces.schema_ref` pointing to the `outputSchema`. The tool's implementation either (a) becomes a `script/` inside the skill folder, called from the body, or (b) remains an external MCP server and the skill body issues `allowed_mcp_tools` calls to it. Set `exposable_as.mcp_tool: true` so the same skill can be re-emitted as an MCP tool descriptor when needed.

### 22.4 From a freeform LLM prompt

The hardest case. Identify what the prompt *does* (action) versus what it *constrains* (schema). The action becomes a skill body. The schema, if the prompt has hard expectations on input/output shape, becomes envelope schemas. The hard rules (MUST / MUST NOT) become invariants. The soft preferences become SHOULD bullets in the body. Promote to Tier 1 first, then Tier 2 once acceptance fixtures exist.

---

## Part 23 — Index of skills + contracts

### 23.1 Skills

| Persona / shared | Skill | Status | Owner-role | Pipeline links |
| --- | --- | --- | --- | --- |
| `cuo/_shared/` | `hello-world` | v1.0.0 | shared | teaching example; no chains |
| `cuo/cpo/`     | `requirements-discovery` | v0.1.0 (scaffold) | cpo | chain entry point: BRAIN + 20-q interview → `project_brief@1` |
| `cuo/cpo/`     | `prd-author` | v0.1.0 (scaffold) | cpo | consumes `project_brief@1` + 3-5 follow-ups → `prd@1` |
| `cuo/cpo/`     | `fr-author`   | v0.2.2 | cpo    | consumes PRD/spec docs → FR markdowns → `fr-audit` |
| `cuo/cpo/`     | `fr-audit`    | v0.2.2 | cpo    | consumes FR markdowns from `fr-author` or any source |
| `cuo/cpo/`     | `prd-audit`   | v0.1.0 (scaffold) | cpo | quality gate on PRDs (advisory-leaning per Q4) |
| `cuo/cto/`     | `fr-to-tech-spec` | v0.1.0 (scaffold) | cto | consumes audited FR markdowns → emits tech specs (gated on runtime) |
| `cuo/cto/`     | `srs-author`  | v0.1.0 (scaffold) | cto | consumes audited PRD → emits `srs@1` markdown |
| `cuo/cto/`     | `srs-audit`   | v0.1.0 (scaffold) | cto | quality gate on SRSs (advisory-leaning) |
| `cuo/cto/`     | `spec-to-impl-plan` | v0.1.0 (scaffold) | cto | tech-spec OR audited FR → impl-plan + tickets in PROJ MCP |
| `cuo/cpo/`     | `chain-selector` | v0.1.0 (scaffold) | cpo | reads brief → picks lean/standard/full → emits chain plan |

### 23.2 Contracts

| Contract | Latest version | Kind | Stewarded by | Consumed by |
| --- | --- | --- | --- | --- |
| `feature-request` | v1 (`feature_request@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/fr-author` v0.2.0+, `cuo/cpo/fr-audit` v0.2.0+, `cuo/cto/fr-to-tech-spec` v0.1.0+, `cuo/cto/spec-to-impl-plan` v0.1.0+ (lean) |
| `nats-subjects` | v1 (`nats_subjects@1`) | wire_protocol | `cuo-cto` | all skills v0.2.2+, the supervisor |
| `project-brief` | v1 (`project_brief@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/requirements-discovery` v0.1.0+, `cuo/cpo/prd-author` v0.1.0+, `cuo/cpo/chain-selector` v0.1.0+ |
| `prd` | v1 (`prd@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/prd-author` v0.1.0+, `cuo/cpo/prd-audit` v0.1.0+, `cuo/cto/srs-author` v0.1.0+ (input), `cuo/cpo/fr-author` v0.3.0+ (planned) |
| `srs` | v1 (`srs@1`) | artefact_schema | `cuo-cto` | `cuo/cto/srs-author` v0.1.0+, `cuo/cto/srs-audit` v0.1.0+, `cuo/cto/fr-to-tech-spec` v0.2.0+ (input context) |
| `impl-plan` | v1 (`impl_plan@1`) | artefact_schema | `cuo-cto` | `cuo/cto/spec-to-impl-plan` v0.1.0+ |

(Indexes grow as skills land. Maintained by hand; CI consolidation script is a v0.3.0 follow-up.)

---

## Part 24 — How to add a new skill

Decide the owner role — one of the 14, or `_shared/` if reusable. `mkdir cuo/<role>/<skill-id>/` with a kebab-case id. `touch SKILL.md CHANGELOG.md` and write a v0.1.0 frontmatter block per [Part 2.1](#21-the-full-v020-frontmatter), starting at Tier 1. Implement progressive disclosure — minimal SKILL.md body (≤500 lines, ideal ≤300), reference docs in `references/`, executables in `scripts/`. Wire `expects:` / `produces:` to existing schemas in `_shared/` if reusable, or new ones under `envelopes/`. Declare contract dependencies — if your skill uses an artefact schema, add a `depends_on_contracts:` entry pointing into `cyberos/docs/contracts/`. Add a row to [Part 23.1](#231-skills) above. Append a v0.1.0 entry to the skill's CHANGELOG and to `cyberos/docs/skills/CHANGELOG.md`.

### 24.1 Self-test checklist (run before committing any new SKILL.md)

A skill is registry-valid when ALL of:

- [ ] Folder name is kebab-case and matches `name:` in frontmatter.
- [ ] `SKILL.md` parses as Markdown with one YAML frontmatter block, no mid-file `---` outside fenced code spans (AGENTS.md §4.3 + DEC-087).
- [ ] All 33 frontmatter fields ([Part 2.1](#21-the-full-v020-frontmatter)) are present (or explicitly `null` where allowed).
- [ ] `expects:` and `produces:` reference real JSON schemas reachable from this folder or `_shared/`.
- [ ] `allowed_brain_scopes.write` is empty UNLESS the skill is explicitly authorised to mutate BRAIN.
- [ ] `allowed_mcp_tools` is exhaustive — gateway will reject unlisted tools at call time.
- [ ] `audit.row_kind` matches the `produces.output_kind` enum.
- [ ] `invocation_modes` declared (workflows: `[standalone, chained]` or `[chained]` only; persona cards: `[persona_routing_only]`).
- [ ] `self_audit.invariants_ref` populated and the file exists with ≥3 invariants for production skills.
- [ ] `human_fine_tune.fine_tuner_role` set to a valid value.
- [ ] At least one `references/` doc OR a clear note that none are needed.
- [ ] `CHANGELOG.md` exists with at least a v0.1.0 entry.
- [ ] Adding the skill to [Part 23.1](#231-skills) does not duplicate an existing `(persona, name)` pair.

---

## Part 25 — FAQ + glossary

### 25.1 FAQ

**Q. "Standalone vs chained — does the skill know which mode it's in?"** A. Yes. The runtime sets a flag based on §4.1 mode detection. The body can branch on it (`if standalone: render HUMAN_SUMMARY.md`).

**Q. "When does auto-refinement (Part 6) become manual fine-tune (Part 7)?"** A. When `self_audit_refinement_proposal_count_above` is exceeded — default 2 proposals on the same theme within one batch. Auto-refinement caught a problem auto-refinement can't solve; a human takes over.

**Q. "Two skills both want to be triggered by the same user phrase. How does the supervisor pick?"** A. The classifier returns `{skill_id, confidence}`. If multiple skills match above the floor, escalate via Question: "I'm not sure which workflow you mean — A or B?"

**Q. "Should fr-author and fr-audit be one skill or two?"** A. Two. CyberOS skills are atomic: each is standalone AND chainable. The split lets you audit-only without regenerating, regenerate without re-auditing, or chain both. See `cuo/cpo/fr-author/CHANGELOG.md` v0.1.0 for the trade-off.

**Q. "Can a skill call another skill directly, without the supervisor?"** A. No. Every skill-to-skill handoff goes through the supervisor's LangGraph (which writes the action_log row, applies the scope contract, validates envelope schemas). Direct calls would break audit and chain-of-custody. If you need a "library" of helper functions, those go in `scripts/` inside the skill folder.

**Q. "When do I make a skill vs. write a regular Python script?"** A. Use a skill when ANY of: the work involves LLM inference, you want auditability through `genie.action_log`, you want it composable with other skills, you want CUO to invoke it from natural language. Use a script for purely deterministic computation outside the supervisor's loop.

**Q. "What if I want to copy fr-author to Antigravity / Codex / Cursor?"** A. See [Part 9](#part-9--host-adapter-strategy). Today: copy the folder + the `_contracts/feature-request/` folder; the body works but auto-refinement, audit ledger, and scope enforcement are degraded to filesystem fallbacks. Soon (v0.3.0): the build pipeline emits host-native artefacts via transpilers + a host shim, so equivalence is preserved.

**Q. "How do I test a skill before the runtime exists?"** A. Three ways: (1) read it as a human — does the body make sense as a prompt? (2) Run it manually — paste the SKILL.md body into Claude.ai with the input envelope as the user message; compare output against `acceptance/golden-output*.md`. (3) Validate envelopes with `ajv`. The skill is a contract, not code — most validation happens by reading.

**Q. "Why do skills use Markdown frontmatter instead of a structured config format?"** A. Markdown frontmatter is the lowest common denominator. Anthropic skills, Claude Code, Antigravity, Codex, Cursor, MCP server descriptors all read SKILL.md-style files. JSON or TOML would lock us into a different ecosystem. The choice was deliberate: portability over purity.

**Q. "How does versioning interact with chained skills?"** A. Each skill's `skill_version` is independent. A chain of `fr-author v0.2.0 → fr-audit v0.2.0` works because their envelope schemas are compatible. If `fr-audit` MAJOR-bumps to v1.0.0 with breaking schema changes, `fr-author` stays at v0.2.0 unless its own contract changes. The CI matrix verifies envelope compatibility on every PR.

**Q. "Can a single skill produce multiple artefacts in one invocation?"** A. Yes. fr-author writes 3 FRs in one batch, producing 3 `artefact_write` rows. The output envelope's `frs_written` array carries all 3. Multi-artefact skills are common; they're not multiple invocations.

### 25.2 Glossary

| Term | Definition |
| --- | --- |
| **skill** | A folder with a `SKILL.md`. Atomic unit of CyberOS automation. |
| **contract** | A versioned schema under `cyberos/docs/contracts/`. NOT a skill — declares the shape that skills produce/consume. |
| **persona** | A folder of skills representing one C-level role (e.g., `cuo/cpo/`). 14 personas total per DEC-052. |
| **CUO** | Chief Universal Officer. The outer persona surface; the 14 sub-personas (CEO, CFO, CPO, etc.) are CUO's specialists. |
| **trigger** | An invocation of a skill. Three paths: direct, supervisor-routed, chained. |
| **chain** | A pipeline. Skill A's output envelope's `next_skill_recommendation` causes the supervisor to invoke Skill B. |
| **envelope** | A JSON object validated against a schema. Inputs (`expects`) and outputs (`produces`) of a skill. |
| **`genie.action_log`** | The append-only Postgres table where every skill output gets a row. The audit trail. |
| **action_log row** | One audit log entry. Carries `(persona_id, skill_id, skill_version, row_kind, trace_id, payload_hash, hash_chain)`. |
| **hash chain** | Each row's `chain` field = sha256(canonical_json(row) + prev_row.chain). Tampering breaks the chain. |
| **trace_id** | A UUID flowing through every action_log row in one chained invocation. |
| **HITL** | Human in the loop. When a skill needs a human decision, it emits a Question primitive (SRS §6.6.2) and pauses. |
| **scope contract** | The frontmatter fields `allowed_brain_scopes` + `allowed_mcp_tools` + `escalation`. Enforced by the runtime per SRS §6.4. |
| **persona-card** | A `SKILL.md` at the persona level (e.g., `cuo/cpo/SKILL.md`) declaring voice, scope ceiling, escalation graph, owned workflows. |
| **acceptance/** | Folder of golden input/output pairs. Layer 2 validation. |
| **drift signal** | OBS-detected metric (acceptance rate <40% / 7 days) that triggers auto-pause per DEC-055. |
| **invocation_modes** | NEW v0.2.0. List declaring whether the skill accepts standalone (chat) entry, chained (envelope) entry, or both. |
| **CCSM** | Canonical CyberSkill Skill Manifest. The SKILL.md as source of truth; per-host artefacts are generated. |
| **self-audit** | NEW v0.2.0. Runtime invariant checks declared in `INVARIANTS.md`; breaches emit `refinement_proposal`. |
| **refinement_proposal** | NEW v0.2.0 output_kind. Structured envelope the skill emits when an invariant breaks. Supervisor pauses, human reviews. |
| **manual fine-tune** | NEW v0.2.0. The 7-step playbook (Part 7) for human-driven skill improvement. |
| **exposable_as** | NEW v0.2.0. Frontmatter block declaring which surfaces the skill ships through (internal / plugin / MCP / connector). |
| **depends_on_contracts** | NEW v0.2.0. Frontmatter list pinning the contract versions the skill consumes. |
| **NATS** | The event bus for fire-and-forget pub-sub between skills (DEC-029). Subjects: `cuo.<skill>.<event>`. |
| **LangGraph** | The supervisor framework (DEC-027). Observe-Decide-Act loop with checkpointed state. |
| **SRS / PRD** | Source-of-truth design documents. SRS = Software Requirements Spec; PRD = Product Requirements Doc. Both at `cyberos/docs/`. |
| **AGENTS.md** | The CyberOS Universal Agent Memory Protocol. `cyberos/docs/CyberOS-AGENTS.md`. |
| **BRAIN** | The `.cyberos-memory/` directory + the Postgres mirror. Three-layer memory store. PRD Part 5; AGENTS.md §0.3. |
| **DEC-NNN** | A locked decision in SRS Part 13 + Appendix G. Cited throughout. |
| **CaMeL** | Google DeepMind's dual-LLM defence pattern against indirect prompt injection. DEC-050. |
| **MCP** | Model Context Protocol. The cross-vendor tool registry standard. DEC-048. |
| **OBS** | The observability module (SRS §6.12). Tracks per-skill acceptance, drift, HITL rate. |
| **REW / LEARN / ESOP** | Three regulated domains: Right-to-Erasure-Writes (REW), Learning Records (LEARN), Equity/Stock Plan (ESOP). PRD §6.4.1 forbids most personas from auto-writing these. |

---

## Part 26 — What doesn't exist yet

Honest inventory of contracts-only-no-runtime: the `cyberos run` CLI (contract specified by every skill's `expects:` envelope schema; implementation pending), the CUO LangGraph supervisor (topology specified in SRS §6.1.1, code pending), the `genie.action_log` Postgres table + tamper detector (schema in SRS §6.7 + §10.4, migration not authored), the auto-refinement runtime (`INVARIANTS.md` is read; breaches declared; the engine that runs them at LangGraph node boundaries is pending), the acceptance-test harness (folder convention documented; the runner script is not), the drift-signal feedback loop (OBS module's per-skill acceptance-rate metric per DEC-055 needs to wire into a Notify generator that auto-pauses skills), the plug-in installer + transpilers (Part 9 — Phase A done, Phases B–E pending), and the host shim library `cyberos-skill-runtime` (interface specified in §9.2; library pending).

The registry is the **source-of-truth that all of those will read**. None of them need to exist for the skill folders to be valuable today — the skills *are* the contracts. When the runtime is built, every behaviour it needs is documented in some `SKILL.md` or `references/*.md` already.

---

## Part 27 — Citations

This document deliberately cites rather than duplicates. Authoritative sources: persona model + 14-persona registry → CyberOS-PRD.docx Part 6 + Part 3.2; SRS Part 6.3 + DEC-052. Anthropic Skills format mandate → SRS §6.2 + DEC-061. Audit ledger schema → SRS §6.7 + §10.4 + AGENTS.md §7. Scope contract enforcement → SRS §6.4. Notify/Question/Review primitives → SRS §6.6 + PRD §6.5. Trust calibration + defer triggers → PRD §6.4 + §6.4.1. Anti-prompt-injection (CaMeL) → DEC-050 + AGENTS.md §4.2. LangGraph runtime → DEC-027 + SRS §6.1. NATS event bus → DEC-029. Drift / acceptance auto-pause → DEC-055 + SRS §6.12. v0.2.0 contracts split (skills vs. contracts) → DEC-090. v0.2.0 dual-mode + exposability → DEC-091. v0.2.0 self-audit + auto-refinement → DEC-092. v0.2.0 manual fine-tune playbook → DEC-093. Memory protocol (the BRAIN this all writes to) → CyberOS-AGENTS.md (entire document).

If any rule above conflicts with one of those source documents, the source document wins; raise a §0.4 protocol-refinement candidate against this README.
