# Anthropic *Complete Guide to Building Skills for Claude* — digest vs. CyberOS SKILL module

> **Status:** findings doc, 2026-05-19 · **Owner:** Stephen Cheng (CEO) · **Source:** Anthropic, *The Complete Guide to Building Skills for Claude* (33 pages, undated 2025/2026 — references "January 2026" in Chapter 4, and the Anthropic-blog "Engineering Agents for the Real World") · **Module under audit:** `cyberos/modules/skill/` v2.0.0 (104 author+audit pairs / 108 contracts as of Session H, 2026-05-18).
>
> Companion FR drafts: [`docs/feature-requests/skill/FR-SKILL-111-trigger-description-enrichment.md`](../../docs/feature-requests/skill/FR-SKILL-111-trigger-description-enrichment.md) · [`docs/feature-requests/skill/FR-SKILL-112-trigger-tests-fixtures.md`](../../docs/feature-requests/skill/FR-SKILL-112-trigger-tests-fixtures.md). A third FR sketched in §6 of this doc (XML-tag-free frontmatter) is **not** authored here — it carries enough breakage risk on the existing 104 pairs that it deserves operator sign-off before authoring.

This document digests Anthropic's 33-page guide, lines each principle up against the current CyberOS SKILL module (v2.0.0, README at [`modules/skill/README.md`](README.md), authoring discipline at [`modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`](feature-request-audit/AUTHORING_DISCIPLINE.md)), and proposes a ranked adaptation package. Cross-references throughout cite the Anthropic guide by chapter and the CyberOS README by Part number, so a reader can verify every claim without re-reading either source.

---

## Table of contents

- **§1** — Executive summary (1-page TL;DR)
- **§2** — Anthropic guide structural digest (6 chapters condensed)
- **§3** — Side-by-side principle-vs-CyberOS table
- **§4** — Where CyberOS already exceeds the guide (validation that the heavy stack pays off)
- **§5** — Where the guide reveals real gaps in CyberOS (3 confirmed, 5 nuance-only)
- **§6** — Ranked adaptation proposals (high / medium / low value)
- **§7** — Candidate FRs (with proposed IDs, scope, effort estimates)
- **§8** — What NOT to adopt (Anthropic patterns that would regress CyberOS)
- **§9** — Open questions deferred to operator
- **§10** — Citations

---

## §1 — Executive summary

The Anthropic guide is a fundamentals doc aimed at first-time skill authors, not a competing architecture. It defines the lowest-common-denominator Anthropic Agent Skills format — a folder containing `SKILL.md` (YAML frontmatter + Markdown body), optional `scripts/`, `references/`, `assets/` — and walks through planning, testing, distribution, and a five-pattern troubleshooting catalogue.

CyberOS SKILL module v2.0.0 already implements every structural principle in the guide. Most of them are implemented *more strictly*: the guide's `description` is freeform; CyberOS's frontmatter has 33 fields organised into 11 governance blocks. The guide recommends a "before-upload checklist"; CyberOS has Part 24.1 self-test plus the 40-rule AUTHORING_DISCIPLINE plus the 8-step audit loop plus `INVARIANTS.md` runtime checks. The guide's iteration advice is "watch under/over-triggering and refine the description"; CyberOS has auto-refinement (Part 6), manual fine-tune (Part 7), drift signals, and acceptance auto-pause at <40% (DEC-055).

The guide does, however, surface **three concrete things CyberOS is missing** when (not if) skills ship to non-CyberOS hosts (Claude.ai, Claude Code, Codex, Cursor, vanilla MCP). All three are about the *port surface* — the frontmatter contract that a flat-host loader actually sees:

1. **The `description:` field carries no trigger phrases today.** CyberOS puts trigger phrases in the body's `## When to invoke this skill` section. A flat-host loader reads only frontmatter at level 1 of progressive disclosure (per the guide's Chapter 1 "first level (YAML frontmatter): always loaded in Claude's system prompt"). Ported skills will under-trigger because the host can't see the body when deciding whether to load. **→ FR-SKILL-111.**
2. **No formal "triggering tests" — only functional acceptance fixtures.** The guide's Chapter 3 recommends three test layers: triggering, functional, performance comparison. CyberOS has the middle one (`acceptance/golden-input.json` + `golden-output*.md`) and the third via OBS production telemetry (Part 13.4). The first is missing: there's no `acceptance/TRIGGER_TESTS.md` listing positive + negative phrases the skill MUST or MUST NOT load on. **→ FR-SKILL-112.**
3. **XML angle brackets in frontmatter values block portability.** CyberOS uses `wrap_in: <untrusted_content/>` as a sentinel literal in frontmatter (per `_template/author/SKILL.md` line 97). The guide's Chapter 1, Reference B, lists `< >` as **forbidden in frontmatter** for security reasons (system-prompt injection). Ported skills will fail the host loader's frontmatter validator. **→ Sketched in §6, not yet FR — see §6.3 for why.**

Beyond these three, the guide adds a few smaller-value patterns CyberOS can absorb cheaply (the "iterate on one task before expanding" pro-tip; the `BASELINE.md` artefact for promoted skills) and confirms many CyberOS conventions already in production. The headline take: **the guide is mostly *validation* that the v0.2.0 frontmatter contract and the audit-fix-audit discipline are correct**; the residual delta is a port-surface hygiene pass.

Adoption cost estimate: **~14 engineering-hours** for FR-SKILL-111 (description sweep across 104 pairs + RUBRIC rule) + **~12 hours** for FR-SKILL-112 (trigger-tests convention + template + auditor rule) + **~10 hours** if the operator decides to author the XML-free sketch as FR-SKILL-113 (with cascading 104-pair sweep + RUBRIC rule + per-host transpiler test). All three are net-positive for portability and cost zero behavioural change to existing chains.

---

## §2 — Anthropic guide structural digest

The guide is 33 pages, 6 chapters, 3 reference appendices. Compressed by chapter:

### Chapter 1 — Fundamentals (pp. 4–6)

- **A skill is a folder containing `SKILL.md` (required), `scripts/`, `references/`, `assets/` (all optional).**
- **Three design principles:**
  - **Progressive disclosure** in three levels: frontmatter (always loaded) → body (loaded when relevant) → linked files (loaded on demand).
  - **Composability** — multiple skills load simultaneously; design assuming others exist.
  - **Portability** — same skill works across Claude.ai, Claude Code, API, provided dependencies are present.
- **Skills + MCP** — "kitchen analogy": MCP is the kitchen (tools), skills are the recipes (knowledge). Without skills, users prompt MCP-only integrations inconsistently. With skills, workflows activate automatically; consistent results.

### Chapter 2 — Planning and design (pp. 7–13)

- **Start with 2-3 concrete use cases.** Define trigger + steps + result before writing.
- **Three use-case categories observed at Anthropic:**
  - **Category 1: Document & Asset Creation** — `frontend-design`, `docx`, `pptx`, `xlsx`, `pdf`.
  - **Category 2: Workflow Automation** — `skill-creator` walks users through skill authoring with validation gates.
  - **Category 3: MCP Enhancement** — `sentry-code-review` coordinates Sentry MCP calls for PR triage.
- **Define success criteria.** Quantitative: 90% trigger-on-relevant, X tool-calls-to-complete, 0 failed API calls. Qualitative: no redirection, consistent across sessions. *Vibes-based; aspirational, not contractual.*
- **Technical requirements (the rules CyberOS must respect for portability):**
  - `SKILL.md` exact case; **no variations accepted**.
  - Skill folder **kebab-case**, no spaces, no capitals, no underscores.
  - **No `README.md` inside the skill folder.**
  - `description` (required) MUST include WHAT + WHEN; **≤ 1024 chars**; **no XML tags**; **specific trigger phrases**; **mention file types if relevant**.
  - Skill names containing `claude` or `anthropic` are **reserved**.
- **YAML frontmatter fields** Anthropic supports out of the box: `name`, `description`, `license` (optional), `compatibility` (optional, 1-500 chars), `allowed-tools` (optional; restricts tool access; example value: `"Bash(python:*) Bash(npm:*) WebFetch"`), `metadata` (optional, custom key-value object).
- **Body structure recommendation:** `# Skill Name` → `## Instructions` (numbered steps) → `## Examples` → `## Troubleshooting` (Error / Cause / Solution).
- **Best practices for instructions:** be specific and actionable, include error handling, reference bundled resources clearly, use progressive disclosure (`references/` for detail).

### Chapter 3 — Testing and iteration (pp. 14–17)

- **Three rigor levels:** manual (Claude.ai), scripted (Claude Code), programmatic (skills API for evaluation suites). Pick by deployment surface.
- **Pro tip:** iterate on a single task end-to-end first, then extract the winning approach into a skill. Faster signal than broad multi-task testing.
- **Three test areas:**
  - **Triggering tests** — listed positive phrases that should load, negative phrases that should not.
  - **Functional tests** — given input X, when skill runs, then Y assertions hold.
  - **Performance comparison** — with-skill vs without-skill: tool-call count, token count, failed-call count.
- **`skill-creator` skill** generates / reviews / iteratively improves skills. Anthropic's official authoring helper.
- **Iteration triage:**
  - **Undertriggering** → add detail and nuance to description; include technical-term keywords.
  - **Overtriggering** → add negative triggers (`Do NOT use for...`), be more specific.
  - **Execution issues** → improve instructions, add error handling.

### Chapter 4 — Distribution and sharing (pp. 18–20)

- **Current model (Jan 2026):** download folder → zip → upload to Claude.ai (Settings > Capabilities > Skills) **or** drop into Claude Code skills directory.
- **Org-level skills** (shipped Dec 18, 2025) — admins deploy workspace-wide with auto-update.
- **Open standard** — Agent Skills published. Skills should be portable across tools/platforms; `compatibility` field notes platform-specific requirements.
- **Skills via API** — `/v1/skills` endpoint + `container.skills` on Messages API + Claude Console version management + Agent SDK integration. Requires Code Execution Tool beta.
- **Recommended distribution:** GitHub public repo + clear README (for humans, *outside* the skill folder) + installation guide + screenshots.
- **Positioning:** focus on outcomes, not features. Highlight the MCP + skills story.

### Chapter 5 — Patterns and troubleshooting (pp. 21–27)

- **Choosing approach:** problem-first vs tool-first framing.
- **Five patterns:**
  - **Pattern 1: Sequential workflow orchestration** — explicit step ordering, dependencies, validation at each stage, rollback instructions.
  - **Pattern 2: Multi-MCP coordination** — phases with clear separation, data passing between MCPs, centralized error handling.
  - **Pattern 3: Iterative refinement** — quality criteria + iteration + validation scripts + termination rule.
  - **Pattern 4: Context-aware tool selection** — decision tree on input characteristics, fallback options, transparency.
  - **Pattern 5: Domain-specific intelligence** — embedded compliance / governance / audit logging.
- **Troubleshooting catalogue:**
  - Won't upload → SKILL.md naming, YAML frontmatter validity, kebab-case.
  - Doesn't trigger → revise description, add trigger phrases, add file-type mentions.
  - Triggers too often → add negative triggers, narrow scope.
  - MCP connection issues → verify connection, auth, tool names.
  - Instructions not followed → keep concise, put critical at top, use precise language, **advanced: bundle deterministic scripts for critical validations**.
  - Large context → keep SKILL.md under 5000 words, move detail to references/, reduce simultaneously-enabled skills (cap suggested at 20-50).

### Chapter 6 — Resources and references (pp. 28–29)

- Anthropic docs, blog posts, example skills (`anthropics/skills` GitHub), Claude Developers Discord.
- `skill-creator` skill (built into Claude.ai, downloadable for Claude Code).

### Reference A — Quick checklist (p. 30)

Four-stage checklist: before you start (2-3 use cases / tools / planned folder structure) → during development (12 hygiene items) → before upload (6 testing items) → after upload (5 monitoring items including "update version in metadata").

### Reference B — YAML frontmatter (p. 31)

- **Required:** `name`, `description`.
- **All optional** in one example: `license`, `allowed-tools`, `metadata` (with `author`, `version`, `mcp-server`, `category`, `tags`, `documentation`, `support`).
- **Security:** allows standard YAML, custom metadata fields, descriptions up to 1024 chars. **Forbids:** XML angle brackets (`<` `>`), code execution in YAML (safe parsing only), names with `claude` or `anthropic` prefix.

### Reference C — Complete skill examples (p. 32)

Pointers to Anthropic-maintained skills (PDF, DOCX, PPTX, XLSX, plus Asana / Atlassian / Canva / Figma / Sentry / Zapier partner skills). "Clone them, modify them for your use case."

---

## §3 — Side-by-side principle-vs-CyberOS table

| # | Anthropic principle | CyberOS state today | Verdict |
|---:|---|---|---|
| 1 | `SKILL.md` (exact case) is the required entry | Same; `_template/author/SKILL.md` enforces | **Aligned** |
| 2 | Folder kebab-case, no spaces/capitals/underscores | Enforced everywhere; e.g. `feature-request-author/`, `product-requirements-document-author/` | **Aligned** |
| 3 | No `README.md` inside the skill folder | Compliant: no skill-folder `README.md`s found. README.md lives at module root, `tools/`, `toolchain/`, `runners/` (all utility folders, not skill folders) | **Aligned** |
| 4 | Three-level progressive disclosure (FM → body → linked files) | Implemented across all 104 pairs with `references/` (UNTRUSTED_CONTENT, ANTI_FABRICATION, HITL_PROTOCOL, FAILURE_MODES, MANIFEST_SCHEMA) | **Aligned** (CyberOS goes further with 5+ reference files vs the guide's vaguer "as needed") |
| 5 | Composability — multiple skills load simultaneously | Native: LangGraph supervisor + `expects:`/`produces:` envelopes + 20-50 concurrent skill ceiling per the guide ≈ CyberOS's 104-pair catalog routed via classifier (only the matched skill body loads at runtime) | **Aligned** |
| 6 | Portability — same skill across Claude.ai / Claude Code / API | Part 9 host-adapter strategy locks the CCSM source-of-truth; Phases B-E (transpilers + shim + equivalence tests) are 🔵 planned for v0.3.0 | **Partial — port surface vulnerable** (see §5 #1) |
| 7 | `description` MUST include WHAT + WHEN + trigger phrases; ≤1024 chars | CyberOS uses multi-line YAML descriptions (≥3 lines typical, often >200 chars but still ≤1024); trigger phrases live in body's `## When to invoke this skill`, not in `description:`. Part 2 of README says "description: one sentence; ≤140 chars" but the actual templates use much longer freeform descriptions | **Gap** — description carries WHAT but not WHEN trigger phrases (see §5 #1, FR-SKILL-111) |
| 8 | No XML tags `< >` in frontmatter | CyberOS templates use `wrap_in: <untrusted_content/>` as a sentinel literal — this is in frontmatter | **Gap** — port-blocking on Anthropic loader (see §5 #3) |
| 9 | Skill names with `claude` / `anthropic` prefix reserved | No CyberOS skill uses these prefixes | **Aligned** (trivially — CyberOS persona-prefixed namespace `cuo/<role>/...`) |
| 10 | Use cases — start with 2-3 concrete | `STANDALONE_INTERVIEW.md` defines required + optional fields per skill; `## When to invoke this skill` lists 3 natural-language phrases | **Aligned** |
| 11 | Define success criteria — quantitative + qualitative | OBS module tracks acceptance_rate, hitl_pause_rate, avg_iteration_count, refinement_proposal_rate, drift_signal_count per Part 16.2; thresholds explicit (≥80% accept, <20% HITL, etc.) | **Aligned** (CyberOS more rigorous — operational telemetry vs the guide's "vibes-based assessment") |
| 12 | Triggering tests — positive + negative phrases | **Not present** — no `acceptance/TRIGGER_TESTS.md` convention; trigger discovery happens in production via OBS rather than design-time | **Gap** (see §5 #2, FR-SKILL-112) |
| 13 | Functional tests — golden input/output | `acceptance/golden-*-input.json` + `golden-*-output*.md` present on every production skill; `feature-request-audit` is reproducible (byte-stable reports) | **Aligned** (CyberOS more rigorous — deterministic byte-stable + Layer 1 mechanical via `ajv validate`) |
| 14 | Performance comparison — with vs without skill | Not formalised as an artefact; OBS provides per-skill latency / token / acceptance metrics post-deploy. **No `BASELINE.md` design-time comparison.** | **Partial — minor gap** (see §6.4) |
| 15 | Sequential workflow orchestration (Pattern 1) | Native: feature-request-author PLAN→WORKER→RESUME phases, manifest-state-driven phase computation | **Aligned** (more rigorous — manifest hash + re-entrancy guarantee) |
| 16 | Multi-MCP coordination (Pattern 2) | Native: `allowed_mcp_tools:` + chain envelopes; CUO supervisor walks LangGraph edges | **Aligned** |
| 17 | Iterative refinement (Pattern 3) | Native: 8-step audit loop with PASS / NEEDS_HUMAN / FAIL / EXHAUSTED / NO_PROGRESS termination; auto-refinement Part 6 | **Aligned** (CyberOS far more rigorous — `INVARIANTS.md` + anomaly signals + `refinement_proposal` envelope) |
| 18 | Context-aware tool selection (Pattern 4) | Native: `chain-selector` skill, persona escalation graphs, `confidence_band.defer_below` triggers | **Aligned** |
| 19 | Domain-specific intelligence (Pattern 5) | Native: persona-card pattern, `audit.row_kind`, EU AI Act discipline, denylist (Part 15.3), hash-chain audit ledger | **Aligned** (CyberOS more rigorous — sev-0 invariants, denylist enforcement, regulatory mapping) |
| 20 | `allowed-tools` frontmatter field (Anthropic format `"Bash(python:*) Bash(npm:*) WebFetch"`) | CyberOS uses `allowed_mcp_tools:` (snake_case, list-shaped). Different name, equivalent semantic. Anthropic-format transpilers will need to translate one → other | **Aligned — minor namespace drift** (Phase B transpiler concern) |
| 21 | `compatibility` field (optional) | CyberOS `public/vietnam-bank-transfer/SKILL.md` uses it; not in the 33-field v0.2.0 spec. Vietnamese-locale skills sometimes carry it | **Partial — adoption could be more uniform** |
| 22 | `metadata` block (free-form custom kv) | CyberOS v0.2.0 uses `metadata.{version, module, stage, cyberos-template, cyberos-rubric-target}`; uniform across the 104 pairs | **Aligned + extended** |
| 23 | Body structure (`# Title` → `## Instructions` → `## Examples` → `## Troubleshooting`) | CyberOS body: `# skill-name` → `## When to invoke` → `## Self-test preamble` → `## §N pipeline / phase / operating principles / failure modes / reference docs / how-to-use`. Different but more structured | **Aligned + extended** |
| 24 | "Be specific and actionable" instruction principle | Enforced via §"MUST" / §"MUST NOT" / §"SHOULD" sections in every workflow body | **Aligned** |
| 25 | "Include error handling" | `references/FAILURE_MODES.md` per skill catalogues BOOT codes + STALE handling + EXHAUSTED + drift codes | **Aligned + extended** |
| 26 | "Reference bundled resources clearly" | `## §N Reference docs (progressive disclosure)` body section enumerates every linked file | **Aligned** |
| 27 | "Use progressive disclosure — SKILL.md focused on core instructions" | Body ≤500 lines (Part 24 of README), detail in `references/`. Anthropic suggests ≤5000 words (~14000 chars) — CyberOS cap is tighter | **Aligned + stricter** |
| 28 | Three test rigor levels (manual / scripted / programmatic) | Layer 1 mechanical (`ajv validate`) + Layer 2 functional (`acceptance/`) + Layer 3 operational (OBS dashboard) per Part 13.1 | **Aligned** |
| 29 | Pro tip — iterate on a single task before expanding | Not formalised. `_template/author/acceptance/README.md` mentions adding 1-3 fixtures but doesn't say "start with one and iterate" | **Partial — minor gap** (see §6.5) |
| 30 | Distribution — GitHub repo + README (outside skill folder) + screenshots | CyberOS repo is monorepo; per-skill README absence is correct per Anthropic; module-level `README.md` + `CHANGELOG.md` consolidated. Phase B host-adapter strategy will emit per-host artefact bundles | **Aligned — Phase B pending** |
| 31 | Skills as open standard | CCSM (Part 3.7) treats SKILL.md as authoritative; transpilers emit per-host artefacts | **Aligned in spec, Phase B pending in build** |
| 32 | Iteration based on feedback (under/over/execution) | Auto-refinement (Part 6) + manual fine-tune (Part 7) + drift auto-pause (DEC-055) — far beyond the guide's three bullets | **Aligned + dramatically extended** |
| 33 | Anti-patterns catalogue | Part 18 of README has 12 anti-patterns (vs the guide's ~6); covers things the guide doesn't (don't write skills that call other skills directly, don't promote LLM-inferred to confidence: 1.0, don't auto-set EU AI Act risk class, don't change RUBRIC.md mid-batch) | **Aligned + extended** |
| 34 | Body soft cap on size (5000 words) | Part 24 prescribes "≤500 lines, ideal ≤300" — tighter | **Aligned + stricter** |
| 35 | 20-50 simultaneously enabled skills ceiling | Not a constraint in CyberOS: progressive disclosure + supervisor routing means only the matched skill loads | **Architecturally moot — not applicable** |
| 36 | "Advanced technique: bundle scripts for critical validations" | `scripts/` folder convention with `generate_qr.py`, `brain_writer.py` etc.; not heavily emphasised in the template | **Aligned — could be emphasised more in `_template/author/SKILL.md`** |
| 37 | "Model laziness" — explicit `## Performance Notes` in user prompts | Not present in CyberOS; the equivalent is the CONTRACT_ECHO discipline (forces the model to declare phase + bounds before any action) | **Aligned via different mechanism** (CONTRACT_ECHO is arguably stronger) |
| 38 | Quick checklist Reference A — before/during/before-upload/after-upload | Part 24.1 self-test checklist covers most "during development"; "before upload" + "after upload" partial — OBS handles "monitor for under/over-triggering" but not codified as a checklist | **Partial — minor doc gap** (see §6.6) |
| 39 | Reference B YAML frontmatter — name, description, license, allowed-tools, metadata | 33-field v0.2.0 frontmatter (Part 2.1) is a strict superset | **Aligned + extended** |
| 40 | Reference C — Anthropic-maintained example skills | CyberOS has `_template/author/SKILL.md` + `_template/audit/SKILL.md` + `hello-world` (v1.0.0) as the in-house references | **Aligned** |

---

## §4 — Where CyberOS already exceeds the guide

Worth documenting because future fine-tunes are tempted to "simplify" toward Anthropic's surface — most of those simplifications would be regressions for CyberOS production needs:

1. **33-field frontmatter contract** (Part 2.1) vs Anthropic's 5 (`name`, `description`, `license`, `allowed-tools`, `metadata`). The extra 28 govern audit, escalation, dual-mode, exposability, self-audit, manual fine-tune, determinism, source-tier, scope contract, and contract dependencies. Each one closes a class of incident; none should be cut.
2. **`expects:` / `produces:` envelope schemas** with explicit `required_fields` / `optional_fields` / `standalone_interview_ref` / `human_summary_ref`. The guide has no concept of a typed envelope between skills.
3. **`depends_on_contracts:`** (DEC-090) — skills declare the contract versions they consume; the build pipeline can ship skill+contract as one bundle. Anthropic skills hard-code paths in body prose.
4. **`exposable_as:` block** — `internal` / `agent_plugin` / `mcp_tool` / `partner_connector` with the trust-↔-exposability linkage (Part 5.3) — `partner_connector: true` is gated on having ≥3 invariants including INV-scope-discipline + INV-fabrication-boundary. The guide treats distribution as "zip + upload"; CyberOS treats it as a versioned trust contract.
5. **`audit:` block** — every concrete output writes exactly one row to `genie.action_log`, hash-chained per AGENTS.md §7.2. The guide has no audit story.
6. **`untrusted_inputs:` block + CaMeL pattern** (DEC-050) — every external byte wrapped in `<untrusted_content>`, marker scan, surface-to-human on hit. The guide acknowledges "instructions not followed" troubleshooting but has no prompt-injection defence.
7. **`self_audit:` block with `INVARIANTS.md`** — Part 6's auto-refinement loop. Runtime checks invariants at every node boundary; breach → `refinement_proposal` envelope → pipeline pause → human review. The guide's equivalent is "iterate based on feedback"; CyberOS makes it a contract.
8. **`human_fine_tune:` block + 7-step playbook** (Part 7) — pause + diagnose + add regression + edit + re-run + bump + resume. Reviewer roles, blackout windows, required artefacts (changelog_entry, acceptance_test_added, memory_refinement_entry). The guide has no equivalent.
9. **Audit-fix-audit discipline** (Part 18 + Recipe 13) — mandatory after every new contract registration. Caught real drift on `nats-subjects@1` registry v0.2.2 (contract said `cuo_cpo.fr_author.fr_written`; reality was `cuo.fr_author.fr_written`). The guide has nothing like it.
10. **Anomaly signals** — `confidence_low_streak`, `user_correction_streak`, `denylist_near_miss_streak`, `scope_rejection_streak`, `citation_missing_streak`, `deterministic_drift`, `rule_reversal_streak`, `needs_human_rate_above`. Tunable thresholds + windows. The guide has none.
11. **40-rule AUTHORING_DISCIPLINE.md** + the master rule ("after creating one FR, loop audit rounds on it until it reaches perfect — before starting the next FR"). The guide has Reference A's 4-stage checklist; CyberOS has 40 specific rules each tied to a prior rework moment.
12. **Persona-card pattern** — `cuo/<role>/SKILL.md` declares voice / scope ceiling / escalation graph; workflow skills inherit. The guide has no role-scoped namespacing.
13. **Contract-vs-skill split** (Part 8, DEC-090) — contracts under `contracts/<id>/CONTRACT.md` constrain shape; skills under `<skill-name>/SKILL.md` act. The guide conflates the two.
14. **Hash-chain audit ledger + tamper detector** (SRS §10.4.6) — every action_log row's `chain = sha256(canonical_json(row) + prev.chain)`. The guide has no audit ledger.
15. **CONTRACT_ECHO discipline** — every workflow body emits a fenced `CONTRACT_ECHO` block before any file action, declaring skill_id / version / phase / inputs. Forces the model to state intent. The guide has nothing equivalent (its "Performance Notes" trick is weaker).
16. **HUMAN_SUMMARY.md template** for chat-mode rendering — chained mode rolls into parent summary, standalone mode renders separately. The guide doesn't address this duality.
17. **PIPELINE.md** per skill documenting the chain edges in/out. The guide's chain pattern is prose, not a contract.
18. **`emitted_source_freshness_tier`** field on every skill — declares the authority tier of what the skill produces (BRAIN conflict resolution). The guide has no concept of source tiering.

---

## §5 — Where the guide reveals real gaps in CyberOS

Three confirmed gaps where the guide's structural advice exposes a CyberOS weakness. Five nuance-only items where the guide's framing could improve CyberOS docs but doesn't reveal an actual defect.

### §5.1 — Confirmed gaps

#### Gap 1 — `description:` carries no trigger phrases (port-blocking)

**Where it surfaces:** Anthropic guide Chapter 1 "first level (YAML frontmatter): always loaded in Claude's system prompt. Provides just enough information for Claude to know when each skill should be used without loading all of it into context" (p. 5) + Chapter 2 "MUST include BOTH: What the skill does, When to use it (trigger conditions)" + Chapter 2 examples "Use when user uploads .fig files, asks for 'design specs', 'component documentation', or 'design-to-code handoff'" (p. 11).

**Current CyberOS state:** Inspecting `_template/author/SKILL.md` lines 4-9 and `feature-request-author/SKILL.md` lines 4-9 — `description:` is a multi-line WHAT block. The trigger phrases live in the body's `## When to invoke this skill` section (lines 161-168 of `_template/author/SKILL.md`, lines 161-168 of `feature-request-author/SKILL.md`). Inside CyberOS this works because the supervisor's `classify_act` node reads the body. **On any non-CyberOS host** (Claude.ai, Codex, Cursor, vanilla MCP) the loader sees only frontmatter and will fail to trigger on user phrasings the body anticipates.

**Concrete failure mode:** A user on Claude.ai installs the (eventually-shipped-via-Phase-B) `feature-request-author` plugin. They type "Turn this PRD into a backlog." The loader's relevance classifier reads frontmatter `description:`: *"Generate a versioned, audited Feature Request backlog from one or more PRD/spec/SRS documents. Halts at PLAN approval, HITL gates, and amendment batches..."*. The classifier sees "backlog" and "Feature Request" but doesn't see "Turn this into a backlog" or "PRD" or any other natural trigger. Result: skill is **not loaded**, user is told "I don't have a tool for that."

**Fix:** extend `description:` to mandate `[What] + [When-trigger-phrases] + [Key value]` per Anthropic format. Raise the budget from CyberOS's nominal 140 chars to ≤1024 chars (Anthropic's max). Add an auditor rule (e.g. `FM-112 description-format`) that checks the description contains at least 2 distinct trigger-phrase forms (e.g. `Use when user asks to "..."`, `Triggers on "..."`).

**Scope of change:** 104 SKILL.md files in `modules/skill/`, plus `_template/author/SKILL.md`, plus `_template/audit/SKILL.md`, plus `feature-request-audit/RUBRIC.md` adds one rule, plus README Part 2.4 updates the body-section ordering hint (since `## When to invoke this skill` becomes optional / illustrative once triggers are in frontmatter), plus AUTHORING_DISCIPLINE.md §3.13 mentions the new rule. Sweep across 104 pairs is the heavy lift — about 8-12 hours if done by an FR-driven fine-tune cycle.

**FR proposal:** **FR-SKILL-111** (authored in this session — see `docs/feature-requests/skill/FR-SKILL-111-trigger-description-enrichment.md`).

#### Gap 2 — No triggering tests (only functional)

**Where it surfaces:** Anthropic guide Chapter 3 "Recommended Testing Approach — 1. Triggering tests / 2. Functional tests / 3. Performance comparison" (pp. 15-16).

**Current CyberOS state:** Inspecting `feature-request-author/acceptance/` shows `golden-happy-path-input.json` — a functional fixture. There is no list of trigger phrases the skill should match and shouldn't match. The supervisor's classifier is trained / configured separately; per-skill triggering is currently a runtime concern surfaced only in OBS post-deploy.

**Concrete failure mode:** A fine-tune cycle widens the skill's `description:` to catch a missed user phrasing. The fine-tuner doesn't realise the wider phrasing now overlaps with `feature-request-audit`'s description — the supervisor's classifier picks the wrong skill 30% of the time. The drift only surfaces a week later when OBS shows `feature-request-audit.acceptance_rate` drop below 60%. A triggering test that asserted *"Generate v2 of the FR" → feature-request-author* and *"Has FR-007 changed since the last audit?" → feature-request-audit* would have caught the regression at edit time.

**Fix:** define an `acceptance/TRIGGER_TESTS.md` convention. ≥3 positive phrases the skill MUST match (the supervisor's classifier returns this skill with confidence ≥ `defer_below`). ≥3 negative phrases the skill MUST NOT match (the classifier returns a different skill, or `none`, or confidence < `defer_below`). Auditor rule `FM-113 trigger-tests-present` enforces the file exists on production skills (v0.2.0+). CI gate (when CUO supervisor v3.x routing is wired) runs `TRIGGER_TESTS.md` against the classifier in a smoke pass.

**Scope of change:** `_template/author/acceptance/` + `_template/audit/acceptance/` get the new file scaffold. RUBRIC.md adds one rule. AUTHORING_DISCIPLINE.md §3 adds a rule. Existing 104 pairs get backfilled lazily during the next fine-tune of each skill — the rule fires only on production skills (`status: accepted` or higher).

**FR proposal:** **FR-SKILL-112** (authored in this session — see `docs/feature-requests/skill/FR-SKILL-112-trigger-tests-fixtures.md`).

#### Gap 3 — XML angle brackets in frontmatter (port-blocking)

**Where it surfaces:** Anthropic guide Reference B "Forbidden: XML angle brackets (`<` `>`) - security restriction" (p. 31) + Chapter 1 "Security restrictions — Forbidden in frontmatter: XML angle brackets (< >)" (p. 11). Rationale: "Frontmatter appears in Claude's system prompt. Malicious content could inject instructions."

**Current CyberOS state:** `_template/author/SKILL.md` line 97: `wrap_in: <untrusted_content/>` — a literal value containing `<`, `>`. Every one of the 104 production skills carries this sentinel.

**Concrete failure mode:** A Phase-B Anthropic transpiler emits `dist/anthropic/<skill>/SKILL.md` from the CCSM. The transpiler must either (a) strip the field (losing the semantic), (b) replace it with a string sentinel (silent contract change), or (c) fail (no Anthropic build). Today the bug is masked because Phase B isn't shipped; once it ships, this is a hard blocker.

**Fix sketch:** replace `wrap_in: <untrusted_content/>` with a string-form sentinel. Two options:

- **Option A — sentinel rename.** Change to `wrap_in_marker: "untrusted_content"`. The actual XML in the body (`<untrusted_content source="...">…</untrusted_content>`) is untouched — it lives in markdown prose, not frontmatter. Auditor rule `FM-014 marker-string-form` enforces.
- **Option B — boolean flag.** Change to `untrusted_wrapping: required` (which already exists as a separate frontmatter field — see `_template/author/SKILL.md` line 148 `untrusted_content_wrapping: required`). The redundant `wrap_in: <untrusted_content/>` field can simply be dropped, since the marker is implicit and the body's XML form is what actually wraps content.

**Why this is not authored as a third FR in this session:** the fix touches every existing audit file's reciprocity check, every contract that references the field, and every host-adapter transpiler's input shape. The author template `_template/audit/SKILL.md` line 91 also uses `wrap_in: <untrusted_content/>` — both templates plus all 104 pairs plus the contracts that mention it (e.g. `_template/author/references/UNTRUSTED_CONTENT.md`) need a coordinated sweep. A naive replace-all could break `_template/author/references/UNTRUSTED_CONTENT.md` which legitimately documents the `<untrusted_content>` XML form *as body content*. The change deserves operator sign-off on which option (A vs B) before authoring an FR. Sketched in §6.3.

**Recommended next step:** operator picks A or B, authors FR-SKILL-113, sweeps 104 pairs in one session. Estimated 10-14 hours.

### §5.2 — Nuance-only items (the guide's framing improves docs but no defect exists)

#### Nuance 1 — "Iterate on a single task before expanding"

Anthropic's Chapter 3 pro-tip (p. 15): "The most effective skill creators iterate on a single challenging task until Claude succeeds, then extract the winning approach into a skill." CyberOS's `_template/author/acceptance/README.md` mentions adding 1-3 fixtures but doesn't surface this discipline. A one-paragraph addition to AUTHORING_DISCIPLINE.md §3.10 ("Verification rules") or Recipe 8 ("Set up acceptance fixtures") would close the framing gap. **Effort: trivial.** Not a defect; absorption costs ~15 min.

#### Nuance 2 — `BASELINE.md` performance comparison at skill promotion

Anthropic Chapter 3 (p. 16) suggests baselining tool-calls / tokens / failures with-vs-without the skill at design time. CyberOS measures these in production via OBS but doesn't formalise a design-time baseline artefact. Adding an optional `BASELINE.md` to the skill folder at v0.x→v1.0 promotion (Recipe 11) would create a fossilised "why this skill earns its context cost" record. Useful for partner connector gating (Part 5.3). **Effort: small** — define the artefact schema, add to Recipe 11.

#### Nuance 3 — "Negative triggers" pattern (`Do NOT use for...`)

Anthropic Chapter 5 (p. 25) shows `description: ...Use for statistical modeling. Do NOT use for simple data exploration (use data-viz skill instead).` CyberOS bodies do this in `## When to invoke this skill` ("If the user asks to *audit an existing FR*, route to `feature-request-audit` instead") but the disambiguation is in the body, not the description. **Same root cause as Gap 1** — addressed by FR-SKILL-111's mandated description format (which can include the negative-trigger clause).

#### Nuance 4 — `compatibility` field uniform adoption

CyberOS public/Vietnam skills (e.g. `public/vietnam-bank-transfer/SKILL.md` line 11-15) use `compatibility:` to document environment requirements. The 104 internal pairs don't carry it. Anthropic Reference B (p. 31) treats it as optional but useful. **Effort: small** — add `compatibility:` as a Tier-2 optional field in Part 2.1; require for any skill with non-trivial dependencies (e.g. Python script bundles, wasm runtime).

#### Nuance 5 — `## Performance Notes` user-prompt coaching pattern

Anthropic Chapter 5 (p. 26) notes "Adding this to user prompts is more effective than in SKILL.md" — meaning the `Take your time, quality > speed` framing belongs in the supervisor's pre-invocation prompt, not the skill body. CyberOS already does this via CONTRACT_ECHO (forces phase declaration) + `confidence_band` (forces self-assessment). Equivalent function via different mechanism; no change needed.

---

## §6 — Ranked adaptation proposals

Sorted by value × ease.

### §6.1 — HIGH value: FR-SKILL-111 — description trigger enrichment

- **Trigger gap closes:** §5.1 Gap 1
- **Touches:** `_template/author/SKILL.md`, `_template/audit/SKILL.md`, 104 production SKILL.md files, `feature-request-audit/RUBRIC.md` (adds rule FM-112), AUTHORING_DISCIPLINE.md §3.13, README.md Part 2 + Part 18 (anti-pattern: "Don't put triggers only in the body")
- **Effort:** 12-14 hours (FR authoring + 104-pair sweep + RUBRIC rule + auditor regression fixture)
- **Risk:** low — change is additive (existing descriptions remain valid; new rule adds requirement)
- **Authored:** yes, in this session

### §6.2 — HIGH value: FR-SKILL-112 — triggering test fixtures

- **Trigger gap closes:** §5.1 Gap 2
- **Touches:** `_template/author/acceptance/`, `_template/audit/acceptance/`, RUBRIC.md (adds rule FM-113), AUTHORING_DISCIPLINE.md §3.10, README.md Part 13.2 (validation pyramid grows a new tier)
- **Effort:** 10-12 hours (FR authoring + template scaffold + RUBRIC rule + 3 backfill exemplars on `feature-request-author` / `feature-request-audit` / `prd-author`)
- **Risk:** low — new convention; existing skills don't break, only fail the new rule until backfilled (which the rule allows on `status: draft` skills)
- **Authored:** yes, in this session

### §6.3 — HIGH value: FR-SKILL-113 — XML-free frontmatter (NOT authored — needs operator decision)

- **Trigger gap closes:** §5.1 Gap 3
- **Touches:** every SKILL.md (104 pairs), `_template/author/SKILL.md` + `_template/audit/SKILL.md`, the v0.2.0 frontmatter contract spec in README Part 2.1, RUBRIC.md (rule FM-014 or rename existing rule), every `references/UNTRUSTED_CONTENT.md` (to ensure XML form is documented as body-only)
- **Effort:** 10-14 hours
- **Risk:** medium — coordinated sweep across 104 pairs; any miss breaks the audit-fix-audit reproducibility invariant; needs the operator to pick option A (rename to `wrap_in_marker:`) or option B (drop the field — rely on `untrusted_content_wrapping: required`)
- **Authored:** **no** — see §5.1 Gap 3 rationale. Operator decision needed before authoring.

### §6.4 — MEDIUM value: `BASELINE.md` artefact at promotion

- **Trigger gap closes:** §5.2 Nuance 2
- **Touches:** Recipe 11 in README ("Plan a skill promotion (v0.x → v1.0)"), `_template/author/` scaffold optionally
- **Effort:** 2-3 hours (recipe doc + template stub)
- **Risk:** zero — purely additive
- **Authored:** no — small enough to be a future recipe addendum, not a full FR

### §6.5 — MEDIUM value: "Iterate on one task before expanding" methodology

- **Trigger gap closes:** §5.2 Nuance 1
- **Touches:** AUTHORING_DISCIPLINE.md §3.10 adds a rule; Recipe 8 expands one paragraph
- **Effort:** 1 hour
- **Risk:** zero
- **Authored:** no — fold into the next AUTHORING_DISCIPLINE revision

### §6.6 — MEDIUM value: "After-upload" / post-deploy operator checklist

- **Trigger gap closes:** §3 #38 (Reference A's 4-stage checklist)
- **Touches:** README Part 24.1 self-test checklist; add "after-deploy" section pointing at OBS dashboards + drift signals + acceptance auto-pause thresholds. Currently OBS Part 13.4 covers the metrics; this would surface them in the operator's authoring path
- **Effort:** 2-3 hours (docs only)
- **Risk:** zero
- **Authored:** no — fold into the next README revision

### §6.7 — LOW value: `compatibility:` field uniform adoption

- **Trigger gap closes:** §5.2 Nuance 4
- **Touches:** the 33-field frontmatter spec gains an explicit row; existing skills add it lazily
- **Effort:** 4-6 hours (most of it is sweep / fill-in)
- **Risk:** low
- **Authored:** no — only matters when Phase B transpilers ship

---

## §7 — Candidate FRs

| FR id | Title | Status | Module | Priority | Phase | Effort hrs | Authored in this session? |
|---|---|---|---|---|---|---:|---|
| **FR-SKILL-111** | Trigger-phrase enrichment of `description:` for host portability | draft | SKILL | SHOULD | P1 | 12 | ✅ yes |
| **FR-SKILL-112** | `acceptance/TRIGGER_TESTS.md` convention — positive + negative triggers per skill | draft | SKILL | SHOULD | P1 | 10 | ✅ yes |
| FR-SKILL-113 | XML-tag-free frontmatter — replace `wrap_in: <untrusted_content/>` sentinel | sketch only | SKILL | SHOULD | P1 | 12 | ❌ — needs operator decision (option A vs B per §6.3) |
| FR-SKILL-114 | `BASELINE.md` artefact at promotion (recipe addendum, not full FR) | sketch only | SKILL | MAY | P2 | 3 | ❌ — small enough for a recipe doc edit |
| AUTHORING-revision-1 | "Iterate on one task before expanding" rule | sketch only | doc | MAY | any | 1 | ❌ — fold into next AUTHORING revision |
| README-revision-1 | After-upload checklist (Reference A absorption) | sketch only | doc | MAY | any | 2 | ❌ — fold into next README revision |

The two authored FRs honour the §0 master rule of AUTHORING_DISCIPLINE.md (10/10 loop) and §3.12 rules (≥6 ISS findings in the audit sibling). Each is ~500-600 lines, 11-section compliant, with .audit.md siblings. They are independent of each other (FR-SKILL-112 doesn't depend on FR-SKILL-111 — and vice versa).

---

## §8 — What NOT to adopt

A few Anthropic patterns are wrong for CyberOS. Calling them out so future fine-tunes don't accidentally absorb them:

1. **The `description:` ≤140-char convention CyberOS doesn't actually enforce.** Part 2.1 of the README says "≤140 chars" but the templates use multi-line YAML descriptions that often run 200-1000 chars. **Keep the looser de-facto limit.** Tightening to 140 would lose the WHAT detail. FR-SKILL-111 raises the formal cap to Anthropic's 1024 to match reality.
2. **"Reduce enabled skills to 20-50 simultaneously" (guide p. 27).** Anthropic's concern is context bloat from non-progressive skills. CyberOS's progressive disclosure + supervisor routing means only the matched skill loads — the 104-pair catalog is not a runtime cost. **Don't trim the catalog to match the guide.**
3. **"Skill folder name should match `name:` frontmatter field" (guide p. 10).** CyberOS sometimes uses persona-namespaced paths (`cuo/cpo/feature-request-author/`) while `name:` is just `feature-request-author`. **Keep the persona prefix in the path** — it carries scope-contract inheritance and routing semantics. The folder-name-matches-frontmatter rule is for flat Anthropic skill folders; CyberOS layout (post-Session N: `chief-product-officer/` etc.) is a strict superset.
4. **"No README.md inside the skill folder" but free-form prose docs elsewhere (guide p. 10).** CyberOS replaces README.md with structured artefacts: `CHANGELOG.md`, `INVARIANTS.md`, `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md`, `PIPELINE.md`. **Don't merge any of these back into a freeform README** — each is read by a different audit rule.
5. **`skill-creator` skill as authoring tool (guide p. 16).** CyberOS uses the chain orchestrator (README Part 28) plus AUTHORING_DISCIPLINE.md's 40 rules. `skill-creator` is single-author; CyberOS's flow is author-then-audit-loop-to-10/10. **Don't import `skill-creator` semantics** — the audit-loop discipline is stronger.
6. **"`license: MIT`" as the default (guide p. 11).** CyberOS uses `Apache-2.0` (per `_template/author/SKILL.md` line 10). **Keep Apache-2.0** — it carries patent grants and is the company policy.
7. **The guide's "Performance Notes" trick (guide p. 26).** It's user-prompt coaching ("Take your time, quality > speed"). CyberOS has CONTRACT_ECHO + `confidence_band` which are stronger mechanisms. **Don't add `## Performance Notes` to skill bodies** — would dilute the more rigorous controls.

---

## §9 — Open questions deferred to operator

1. **FR-SKILL-113 option A vs option B** (XML-tag-free frontmatter). See §5.1 Gap 3. Recommendation in this doc: lean **option B** (drop `wrap_in:` entirely; rely on the separate `untrusted_content_wrapping: required` field) because it's simpler, but operator may prefer **option A** (rename to `wrap_in_marker: "untrusted_content"`) for forward-compat with hypothetical future marker types. Either choice is sound.
2. **Description budget cap** for FR-SKILL-111: 1024 chars (Anthropic max) vs a CyberOS-specific intermediate (e.g. 512). 1024 matches the port-surface contract directly; 512 forces tighter discipline. Recommendation: **1024 with a soft target of ≤512**, audited via warning rule.
3. **TRIGGER_TESTS.md required count** (FR-SKILL-112): ≥3 positive + ≥3 negative? Or scale by tier (Tier 1 = 3+3, Tier 2 = 5+5)? Recommendation: **3+3 as the floor, scale up at the fine-tuner's discretion**.
4. **Sweep timing.** FR-SKILL-111 + FR-SKILL-112 require backfilling 104 production skills with new fields. Should this be (a) one batch sweep across all 104 (~2-3 days), (b) lazy backfill during each skill's next fine-tune, or (c) gated by phase — fill on `gated_until_phase` activation? Recommendation: **(b) lazy** — the new RUBRIC rule fires at `status: accepted` or higher, so production skills must comply at promotion; scaffold-status skills get a grace window.
5. **Whether to absorb the `BASELINE.md` recipe in this round** or defer. Recommendation: **defer** — it's a v1.0-promotion artefact and no skill has promoted to 1.0 yet (hello-world is the only 1.0 and it's a teaching example).

---

## §10 — Citations

Authoritative sources for every claim above:

- **Anthropic, *The Complete Guide to Building Skills for Claude*** — the PDF under audit. 33 pages. Cited by chapter + page number throughout.
- **CyberOS SKILL module README** — [`modules/skill/README.md`](README.md). Cited by Part number (Parts 1-28).
- **CyberOS FR authoring discipline** — [`modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`](feature-request-audit/AUTHORING_DISCIPLINE.md). The 40-rule normative spec.
- **CyberOS author template** — [`modules/skill/_template/author/SKILL.md`](_template/author/SKILL.md). The 149-line skeleton every workflow author skill inherits from.
- **CyberOS audit template** — [`modules/skill/_template/audit/SKILL.md`](_template/audit/SKILL.md). The 144-line skeleton every auditor skill inherits from.
- **CyberOS feature-request-author** — [`modules/skill/feature-request-author/SKILL.md`](feature-request-author/SKILL.md). The canonical v0.2.2 production example (Part 11 of README).
- **CyberOS contract example** — [`modules/skill/contracts/feature-request/CONTRACT.md`](contracts/feature-request/CONTRACT.md). The `feature_request@1` artefact contract.
- **CyberOS public skill example** — [`modules/skill/public/vietnam-bank-transfer/SKILL.md`](public/vietnam-bank-transfer/SKILL.md). The Tier-1 flat-form example with `compatibility:` field.
- **CyberOS BACKLOG** — [`docs/feature-requests/BACKLOG.md`](../../docs/feature-requests/BACKLOG.md). Confirms FR-SKILL-101 through FR-SKILL-110 are taken; FR-SKILL-111 + FR-SKILL-112 are the next free slots.

If any rule above conflicts with one of those source documents, the source document wins; raise an AGENTS.md §0.4 protocol-refinement candidate against this digest.

---

*End of ANTHROPIC_GUIDE_DIGEST.md.*
