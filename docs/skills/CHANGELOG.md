# CHANGELOG — `cyberos/docs/skills/` registry

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the registry level: MAJOR breaks the layout or the SKILL.md frontmatter contract; MINOR adds a new persona namespace or new contract sections; PATCH is editorial / typo fixes.

---

## v0.2.12 — 2026-05-11 (CHAIN_ORCHESTRATOR — fully automated mode; MINOR — doc-only)

### Added

- **NEW**: [`CHAIN_ORCHESTRATOR.md`](./CHAIN_ORCHESTRATOR.md) — agent-side runbook for fully automated chain execution. The user provides a pitch + answers HITL questions; the agent reads every SKILL.md, drives every interview, writes every artefact, runs every audit-fix loop, executes brain_writer.py, and routes between skills. **The user never copy-pastes a SKILL.md or runs a command by hand.**

### Changed

- **`MANUAL_WORKFLOW.md`** — Quickstart restructured into "Two modes" (Automated ★ recommended; Manual). Automated mode points to CHAIN_ORCHESTRATOR.md with the canonical trigger phrase. Manual mode keeps the previous 6-line procedure. Body of the doc unchanged — it's the manual-mode reference.
- **`README.md`** — header banner updated to reflect the two modes; CHAIN_ORCHESTRATOR.md added to the pinned-docs list.

### Why this is MINOR not PATCH

CHAIN_ORCHESTRATOR.md is a new doc. v0.2.10 (MANUAL_WORKFLOW) was MINOR; this follows the same convention.

### Driver

User asked: *"run manually i mean i just need to give first inputs and do HITL during workflow (if any), not mean i have to copy paste skills content and run all command by myself, can you solve that?"* Yes — the orchestrator is the answer. Trigger phrase + agent runbook give the user a single-message kickoff for the entire chain.

### Trigger phrase (copy this; pin it for the next project)

```
Drive the CyberOS chain on this project. Read cyberos/docs/skills/CHAIN_ORCHESTRATOR.md and follow it.

Pitch: <one paragraph describing the project>
Project repo: <absolute path to the new project's directory>
Output dir: <default: ./planning/<YYYY-MM-DD>-<slug>/>
Caller: human:<your-id>
Profile preference: <auto | lean | standard | full>   (default: auto)
```

Total user effort per chain run: trigger phrase + ~10-30 HITL answers. Total agent effort: read ~12 SKILL.md files + drive ~9 phases + ~9 audit loops + ~30 audit-row appends + 1 final summary.

### Backwards compatibility

Pure addition + clarification. v0.2.11's MANUAL_WORKFLOW.md remains valid as the manual-mode reference; the new orchestrator is purely additive.

### Verification

- ✅ CHAIN_ORCHESTRATOR.md created (length: ~480 lines / ~30 KB)
- ✅ MANUAL_WORKFLOW.md Quickstart restructured into two modes
- ✅ README.md banner updated
- ✅ Trigger phrase consistent across all three docs

---

## v0.2.11 — 2026-05-11 (HOST_ADAPTERS + host-neutral MANUAL_WORKFLOW; PATCH — doc-only)

### Added

- **NEW**: [`HOST_ADAPTERS.md`](./HOST_ADAPTERS.md) — per-host setup recipes. Capability matrix covering 12+ hosts (Claude Cowork, Claude Code, Cursor, Codex CLI, Windsurf, Copilot CLI, Gemini CLI, OpenCode, Aider, Continue, Trae, Kiro, plus degraded-mode Claude.ai web / ChatGPT / Claude in Chrome). Adapter sections for each recommended host with setup commands, per-step shape, and quirks. Decision tree for picking a host. Notes on switching hosts mid-project (BRAIN ledger + on-disk artefacts are host-agnostic; just don't run two hosts concurrently against the same `.cyberos-memory/`).

### Changed

- **`MANUAL_WORKFLOW.md`** — host-neutral throughout. "Open Claude Code" → "Open your agent host"; "first Claude Code session" → "first agent session"; etc. Added a **Host Compatibility** section (between Quickstart and Prerequisites) listing capability requirements + recommended/degraded host classes. Prerequisites updated to abstract away from CLI-specific symlink commands; concrete commands moved to HOST_ADAPTERS.md.

### Why this is PATCH not MINOR

No SKILL.md changed. No contract changed. No new behaviour. Pure clarification: the manual workflow was always host-neutral; the doc just had Claude-Code-flavoured framing. Stephen flagged it ("I don't want fixed Claude Code solution").

### Driver

User asked: *"is it possible to run manual workflow using Claude Cowork or other agents? I don't want fixed Claude Code solution"*. Answer: **yes, fully host-agnostic** — the chain's core (load SKILL.md → follow it → write artefacts → run audit-fix loop → append to BRAIN ledger) needs only file-read + file-write + (ideally) shell access. Claude Code has no special privileges here.

### Recommendation for solo / small-team manual mode today

**Claude Cowork** is the smoothest fit because it has connected folders + sandboxed bash + MCP + file tools all in one chat surface. The BRAIN at `~/Projects/CyberSkill/workbench/.cyberos-memory/` is already wired up; running the chain in Cowork against `~/Projects/CyberSkill/cyberos/docs/skills/` requires no additional setup beyond approving the folder-connection prompts.

### Backwards compatibility

Pure addition + clarification. v0.2.10's MANUAL_WORKFLOW.md content is preserved; only Claude-Code-specific phrasing was generalised.

### Verification

- ✅ HOST_ADAPTERS.md created at registry root
- ✅ MANUAL_WORKFLOW.md `grep -c "Claude Code session" → 0`
- ✅ MANUAL_WORKFLOW.md remaining "Claude Code" mentions are now in lists of supported hosts (legitimate uses)
- ✅ README.md banner to be updated with HOST_ADAPTERS.md pointer (next commit)

---

## v0.2.10 — 2026-05-11 (MANUAL_WORKFLOW + 6 planned improvements; MINOR — doc-only)

> **Naming note**: `v0.3.0` is reserved per the v0.3.0-design entry below — it ships when the runtime's Phase J acceptance harness goes green. This release is `v0.2.10` because it's a doc-only registry update that doesn't change any SKILL.md or contract.

### Added

- **NEW**: [`MANUAL_WORKFLOW.md`](./MANUAL_WORKFLOW.md) — step-by-step procedure for running the chain by hand, today, before the runtime ships. Phase A (Requirements Discovery) → Phase I (Implementation Plan), with per-skill prompts, audit-fix loop walkthroughs, HITL handling, refinement-proposal handling, time budgets per chain_profile (~85 min lean / ~3 h standard / ~5-6 h full). Pin this doc when running on a new project.

### Planned (TIER 1 — fold into Phase 1 of the multi-phase plan)

The companion plan at `<workbench>/.cyberos-memory/project/skills-evolution/cyberos-skills-evolution-plan.md` (v2) — synthesised across mattpocock-skills + everything-claude-code + superpowers + Anthropic patterns/agents + Anthropic Agent SDK + AGENTS.md protocol — calls out three TIER-1 modifications to the existing skill set:

- **M1. `.out-of-scope/<topic>.md` rejection registry** in the refine-suggest mechanism. When a `REF-NNN` proposal is rejected, runtime writes a 3-section markdown file (what / why / prior-requests) under each skill's folder. Anomaly-watcher checks it before re-emitting; matches within Levenshtein-3 → `op:"warn"` instead of `op:"refinement_proposed"`. **Anti-re-litigation by construction.** Pattern lifted verbatim from mattpocock-skills.
- **M2. `domain-context@1` contract** under `cyberos/docs/contracts/domain-context/v1`. Adds a per-project `CONTEXT.md` artefact emitted by `cuo/cpo/requirements-discovery` and consumed by every downstream workflow skill. Format: `## Language` (canonical-term + definition + avoid-list) / `## Relationships` / `## Flagged ambiguities`. New invariant `INV-CONTEXT-CONSISTENCY-001` (sev-1) on every consumer skill: non-canonical term used where a canonical exists → `op:"warn"`. Closes the gap between scope contract (access control) and language contract (vocabulary). Pattern lifted from mattpocock-skills (`grill-with-docs` discipline).
- **M3. `INV-VERTICAL-SLICE-001`** (sev-1) on `cuo/cto/spec-to-impl-plan`. Every issue in `impl_plan@1` MUST be independently completable AND independently testable. Audit explicitly rejects horizontal-slicing patterns ("build all schemas first → build all handlers"). Anti-rationalization framing — name the failure mode. Pattern lifted from mattpocock-skills `tdd/SKILL.md`.

### Planned (TIER 2 — fold into Phase 2 of the multi-phase plan)

Three additions deferred to runtime-bring-up:

- **A1. `lifecycle_state` 29th frontmatter field** (`draft | proposed | active | deprecated`) — requires §0.5 protocol upgrade per the closed-set rule. Marketplace publishes only `active` skills. New audit ops: `skill_promoted`, `skill_deprecated`. Adds bucket-promotion lifecycle from mattpocock-skills.
- **A2. `cuo/_shared/zoom-out` meta-skill** — agent reads CONTEXT.md + ADRs + module BRAIN scope before working in unfamiliar territory. Maps mattpocock's `/zoom-out` skill onto the AGENTS.md §10 read protocol but applied to user-project artefacts.
- **A3. `operational_mode: caveman`** — extend manifest's `operational_mode` enum to include `caveman` for ~75% token reduction on routine runs in established projects. Lifted from mattpocock-skills `caveman/SKILL.md`. §14 block compresses to a one-line status when active.

### Tension noted (not a change, a stance)

mattpocock-skills is **deliberately opposed** to "process-owning frameworks" (their words) — the chain (`requirements-discovery → chain-selector → prd-author → ...`) IS process-owning by design. Resolution: **`chain_profile: lean`** is the mattpocock-stance on-ramp for solo-engineer / small-team users. CyberOS doesn't pick a side; it gives users the dial. Standard/full profiles serve regulated / multi-tenant / agency-style work where process-owning is the value proposition.

### Real-world trigger

User asked for a manual-runnable workflow guide ("focus on refine workflow, includes Requirement discovery then Planning, guide me step by step how to do it manually") and modifications to related docs to capture the plan for future reading. Plan synthesis was triggered earlier by the broader question of "build a comprehensive plan for improvements/refinements/enhancements" against the digested external reference repos.

### Backwards compatibility

Pure addition. v0.3.0 is MINOR per the registry SemVer policy:
- MANUAL_WORKFLOW.md is a new doc (no existing skill changed).
- M1/M2/M3 are PLANNED changes; they don't ship in this version's SKILL.md files. The CHANGELOG entry exists so future readers know what's pending.
- A1/A2/A3 require §0.5 protocol upgrade or runtime support before they can ship.

The 13 existing SKILL.md files remain valid v0.2.9 contracts. v0.3.0 changes the registry-level documentation, not the per-skill contracts.

### Verification

- ✅ MANUAL_WORKFLOW.md created at registry root
- ✅ CHANGELOG.md (this file) updated with v0.3.0 entry
- ✅ README.md to be updated with pointer to MANUAL_WORKFLOW.md (next commit)
- ✅ No SKILL.md frontmatter changes — contract unchanged

---

## v0.2.9 — 2026-05-06 (Stage closing: spec-to-impl-plan + impl_plan@1 contract; MINOR)

### Added

- **NEW contract: `impl_plan@1`** under `cyberos/docs/contracts/impl-plan/`. Stewarded by `cuo-cto`. The shadow record of engineering tickets created in PROJ MCP — markdown lives in repo, actual tickets live in Linear/Jira/GitHub. 12 frontmatter fields + 5 required H2 sections + 2 conditional sections.
- **NEW skill: `cuo/cto/spec-to-impl-plan/`** v0.1.0 — the LAST skill in the chain. Consumes either `tech_spec@1` (standard/full chain_profile) OR audited `feature_request@1` (lean chain_profile, no tech-spec exists). Emits `impl_plan@1` markdown + optionally creates tickets via `proj.create_issue`. INV-001 (refuse non-pass input) sev-0; INV-002 (never auto-create tickets without explicit human approval — even with `create_tickets: true`, runtime forces final HALT_BEFORE_CREATE prompt) sev-0.

### Chain end-to-end now covered

```
human chat + BRAIN
  → requirements-discovery → project_brief@1
  → chain-selector → chain_plan
  → prd-author → prd@1
  → [if standard|full] prd-audit → audited prd@1
  → [if full] srs-author → srs@1 → srs-audit → audited srs@1
  → fr-author → FR markdowns
  → fr-audit → audited FRs
  → [if standard|full] fr-to-tech-spec → tech_spec@1
  → spec-to-impl-plan → impl_plan@1 + tickets in PROJ MCP
```

### Driver

User said "implement spec-to-impl-plan" — the missing last step. Without spec-to-impl-plan, the chain ended at "tech-spec markdown sitting in a folder" — engineering still had to manually create tickets. v0.2.9 closes that loop. Tickets land in PROJ MCP (Linear/Jira/GitHub) only after explicit human approval per INV-002.

### Backwards compatibility

Pure addition. New contract + new skill. Both gated until runtime (`gated_until_phase: runtime_v0_3_0`). The `impl_plan@1` markdown is the SHADOW RECORD — markdown is permanent, tickets are mutable in the external system.

---

## v0.2.8 — 2026-05-06 (chain_profile field + chain-selector skill; MINOR)

### Added

- **`chain_profile` field** added to `project_brief@1` (FM-121) + `prd@1` (FM-118) frontmatter. Enum: `lean` / `standard` / `full`. Brief sets it (via chain-selector); PRD inherits and CANNOT override.
- **NEW skill: `cuo/cpo/chain-selector/`** v0.1.0 — reads brief frontmatter (project_kind, eu_ai_act_risk_class, confidentiality, budget_band, target_release, client_visible) → picks chain_profile via 3-tier first-match-wins rules → emits chain_plan (list of skill_ids). User can override with reasoning. Auto-invoked by supervisor at brief-completion time; chained-only invocation mode (no standalone interview).
- 4 self-audit invariants. INV-001 (deterministic selection from frontmatter) sev-0; INV-003 (warn before skipping prd-audit when client_visible) sev-1.
- `project-brief@1` CONTRACT.md gains a `## Chain profile` section documenting the 3 profiles + skill-list-per-profile + per-project-kind defaults.

### Lean / Standard / Full profiles

| Profile | Default for | Chain |
| --- | --- | --- |
| `lean` (4 skills) | internal_tooling, research_spike, projects under ~2 engineer-weeks | prd-author → fr-author → fr-audit → spec-to-impl-plan |
| `standard` (6 skills, default) | software_product, software_consulting_engagement, projects 2-12 engineer-weeks | prd-author → prd-audit → fr-author → fr-audit → fr-to-tech-spec → spec-to-impl-plan |
| `full` (8 skills) | confidentiality: regulated, eu_ai_act_risk_class: high, multi-year projects | + srs-author → srs-audit |

### Driver

User said "B: yes — chain-selector skill" in registry v0.2.7 design conversation. Closes the gap between "every project goes through the full chain" (overkill for small projects) and "no chain at all" (loses the audit gates). The chain-selector skill IS the rule engine; selection rules are documented in its SKILL.md and gated by `human_fine_tune.on_selection_rule_changed: true`.

### Backwards compatibility

- Existing briefs without `chain_profile` field → schema validation will fail under v0.2.8. Mitigation: chain-selector treats missing `chain_profile` as `standard` and writes the field on its first invocation. Pre-v0.2.8 briefs auto-upgrade.
- `prd@1` field addition is purely additive — existing PRDs get `chain_profile: standard` written on first audit pass.

---

## v0.2.7 — 2026-05-06 (rename fr-create → fr-author for naming consistency; PATCH)

### Changed

- **`cuo/cpo/fr-create/` → `cuo/cpo/fr-author/`** — folder renamed. All artefact-emitting skills now use the "author" verb consistently (prd-author, srs-author, fr-author). The "audit" suffix stays for audit skills. `requirements-discovery` keeps its name (the central activity is interview, not the artefact emission).
- All references swept across the registry: skill_id paths, NATS subject names (`cuo.fr_create.* → cuo.fr_author.*`), prompt_revision (`fr_create@* → fr_author@*`), envelope file names (`fr-create.input.json → fr-author.input.json`), persona-card owned-workflows table, contract consumer lists, README indexes, runtime/ docs, SVG diagram labels. ~74 files / ~633 string replacements.
- Renamed asset: `assets/diagrams/11-fr-create-fr-audit-chain-sequence.svg` → `11-fr-author-fr-audit-chain-sequence.svg`. README link updated.
- Historical references to the SOURCE prompt name `fr_create_and_audit@2.0.0` (in CHANGELOG entries describing the v0.1.0 port history) preserved verbatim — those describe what the skills were ported FROM, not what they're called NOW.

### Driver

User-driven naming-consistency cleanup (Q1 of registry v0.2.7 design conversation). Three artefact-emitting skills (`prd-author`, `srs-author`, `fr-create`) used inconsistent verbs. Rename `fr-create → fr-author` aligns the convention to "author" (every artefact has an `author:` frontmatter field; matching the verb to the field is cleaner). Mechanical rename, no semantic changes.

### Backwards compatibility

PATCH-level mechanical change. No contract changes. No envelope shape changes. No body semantics changes. The skill emits the same outputs against the same inputs. Existing `*.audit.md` reports and `fr-manifest@2` files remain valid — they don't carry the skill name in their content. The only break is for any downstream consumer that hard-coded the path `cuo/cpo/fr-create/` instead of using `depends_on_contracts:` — those need a one-line path update.

---

## v0.3.0-design — 2026-05-06 (Stage D: runtime build plan; design-only, NO skills change)

### Added

- **NEW folder: `cyberos/runtime/`** — the engineering hand-off for building the runtime. Three documents:
  - `PLAN.md` — what the runtime does, 15 phases (A-O), critical-path mapping, ~17 engineer-weeks single-eng / 6-8 weeks 3-eng parallel estimate.
  - `INTERFACES.md` — public surfaces every skill sees regardless of host (`runtime.brain` / `.audit` / `.invariants` / `.envelope` / `.untrusted` / `.nats` / peripheral MCPs).
  - `BUILD_ORDER.md` — concrete sequence with definition-of-done per phase. Recommended sequence for single-engineer + parallel-engineer ordering.
  - `README.md` — read-order pointer.
- This is a **design-only** registry release. No skills changed. No contracts changed. No CHANGELOG bump for any skill. The `gated_until_phase: runtime_v0_3_0` in every scaffolded skill's frontmatter remains in force until Phase J (acceptance harness) turns green.

### Driver

User said "do all stages" — Stage D: turn the design into an engineering hand-off. The user is the founder; they have employees who can pick up the build. This folder is what they hand to engineering.

### Why this is `v0.3.0-design`, not `v0.3.0`

True `v0.3.0` ships when Phase J (acceptance harness) is green AND at least one skill has run end-to-end through the runtime AND `gated_until_phase: runtime_v0_3_0` is removed from skill frontmatter en masse. That's a multi-week engineering effort, not a doc release. `v0.3.0-design` is the bridge: the registry says "here's what the runtime must satisfy", engineering says "here's how we'll build it."

### Backwards compatibility

Pure addition. No registry contract changes. No skill changes. Future v0.3.0 (the real one) will retroactively reference this design doc.

---

## v0.2.6 — 2026-05-06 (Stage C: srs-author + srs-audit + srs@1 contract; MINOR)

### Added

- **NEW contract: `srs@1`** under `cyberos/docs/contracts/srs/`. Stewarded by `cuo-cto`. Documents the system in technical detail (architecture, data model, API surface, data flows, NFRs, failure modes, security posture, telemetry); distinct from `prd@1` (product spec). 12 frontmatter fields + 10 required H2 sections + 3 conditional sections.
- **NEW skill: `cuo/cto/srs-author/`** v0.1.0 — consumes audited `prd@1` + 5-7 architectural-review questions + `module:*` BRAIN reads → emits `srs@1`. INV-001 refuses non-pass PRDs (sev-0); INV-002 forbids llm-implicit on Architecture (sev-0).
- **NEW skill: `cuo/cto/srs-audit/`** v0.1.0 — quality gate on SRSs. Mirrors prd-audit's advisory-leaning approach (most rules warning). `srs_rubric@1.0` with 6 rule families (FM/SEC/COND/AUTH/QA/SAFE + STALE).

### Changed

- `cuo/cto/SKILL.md` owned-workflows table extended: srs-author + srs-audit added.
- `cyberos/docs/contracts/README.md` index extended with `srs@1` row + extended `prd@1` consumers list.
- `cyberos/docs/skills/README.md` Part 23.1 + 23.2 indexes extended.

### Driver

User said "do all stages" after registry v0.2.4 ship. Stage C: srs-author closes the upstream side of the engineering-handoff seam (PRD → SRS); srs-audit gates the SRS before tech-spec authoring.

### Backwards compatibility

All additions are additive; both new skills carry `gated_until_phase: runtime_v0_3_0`.

---

## v0.2.5 — 2026-05-06 (Stage B: prd-audit; MINOR)

### Added

- **NEW skill: `cuo/cpo/prd-audit/`** v0.1.0 — quality gate on PRDs. Advisory-leaning per Q4 of registry v0.2.4 design (most rules warning, structural rules error). `prd_rubric@1.0` with 7 rule families (FM/SEC/COND/AUTH/QA/SAFE/STALE) — AUTH-001..004 is NEW vs fr-audit and enforces per-claim authority markers per AGENTS.md §5.3.
- 6 self-audit invariants. INV-001 (verdict reproducibility on mechanical rules) is sev-0; LLM-judgement rules are explicitly band-reproducible only.
- Full scaffold: SKILL.md + RUBRIC.md + INVARIANTS.md + AUDIT_LOOP.md + REPORT_FORMAT.md + STANDALONE_INTERVIEW.md + HUMAN_SUMMARY.md + envelopes + acceptance.

### Changed

- `cuo/cpo/SKILL.md` owned-workflows table extended: prd-audit added.

### Driver

Stage B: closes the quality gate between `prd-author` and downstream consumers (`fr-author` once it migrates to consume `prd@1` at v0.3.0+; `srs-author` already consumes audited PRD via INV-001 in this release).

### Backwards compatibility

Pure addition; gated_until_phase: runtime_v0_3_0.

---

## v0.2.4 — 2026-05-06 (chain entry point: requirements-discovery + prd-author + project-brief@1 + prd@1 contracts; contracts layout simplified; MINOR)

### Layout

- **Contracts layout simplified** (per REF-018): `<contract-id>/v<n>/` collapsed to `<contract-id>/`. The major version stays in CONTRACT.md frontmatter (`contract_version: v1`); the v<n>/ folder structure was over-engineered for current scale (no parallel-version maintenance need yet). When a contract MAJOR-bumps to v2, the preferred path is "extend the existing folder" (CONTRACT.md documents both versions; template-v2.md added; single CHANGELOG continues). Reviving v<n>/ folders is option B if parallel maintenance becomes burdensome. Mechanical migration: 4 folders moved, 6 SKILL.md `pin_path` declarations updated, 2 README layout diagrams updated, ~93 string replacements across 24 files. Zero contract-semantics changes.
- **NEW contract: `project-brief@1`** registered under `cyberos/docs/contracts/project-brief/`. `artefact_schema` kind; stewarded by `cuo-cpo`. The structured-intake artefact emitted by `requirements-discovery` and consumed by `prd-author`. 16 frontmatter fields + 9 required H2 sections + 4 conditional sections + per-Goal authority markers per AGENTS.md §5.3.
- **NEW contract: `prd@1`** registered under `cyberos/docs/contracts/prd/`. `artefact_schema` kind; stewarded by `cuo-cpo`. The Product Requirements Document artefact emitted by `prd-author`; consumed by future `prd-audit` (v0.2.5) + future `fr-author` v0.3.0+ (when fr-author migrates from generic "PRD/spec docs" to `prd@1`). 15 frontmatter fields + 11 required H2 sections + 4 conditional sections.
- **NEW skill: `cuo/cpo/requirements-discovery/`** scaffolded at v0.1.0. The chain ENTRY POINT for new projects. Reads BRAIN (`company:locked-decisions`, `company:values`, `memories:projects`, `memories:decisions`, `member:*` excluding `private/`, `client:*` when commissioned) AND conducts a 20-question interview (5 triage gates + 15 discovery questions) AND folds in project-triage gating, then synthesises a `project_brief@1`. Project-kind-agnostic per Q2 of the design conversation (handles software, marketing, hiring, partnerships, research, etc.).
- **NEW skill: `cuo/cpo/prd-author/`** scaffolded at v0.1.0. Consumes a `project_brief@1` + 3-5 follow-up questions (feature-flag strategy, telemetry, approval workflow, rollback triggers) + targeted BRAIN reads; emits a `prd@1` draft. Refuses (INV-001) any brief with `triage_verdict: reject`. Refuses (INV-003) `triage_verdict: revise` unless the input envelope sets `proceed_despite_revise: true`. Enforces (INV-002) zero `llm-implicit` authority on Goals.

### Added

Contracts:

- `cyberos/docs/contracts/project-brief/` — CONTRACT.md, template.md, CHANGELOG.md.
- `cyberos/docs/contracts/prd/` — CONTRACT.md, template.md, CHANGELOG.md.

Skills:

- `cuo/cpo/requirements-discovery/` — SKILL.md (full v0.2.0 frontmatter), CHANGELOG.md, INVARIANTS.md (6 invariants; INV-001 BRAIN-must-be-reachable is sev-0), STANDALONE_INTERVIEW.md (20-question script: 5 triage + 15 discovery), HUMAN_SUMMARY.md, envelopes/input.json + output.json, acceptance/README.md (12 priority scenarios).
- `cuo/cpo/prd-author/` — SKILL.md (full v0.2.0 frontmatter), CHANGELOG.md, INVARIANTS.md (7 invariants; INV-001 refuse-rejected-briefs + INV-002 no-llm-implicit-on-Goals are sev-0), STANDALONE_INTERVIEW.md (3-5 follow-up questions + Q5 authority-elevation pass), HUMAN_SUMMARY.md, envelopes/input.json + output.json (6 outcome enums including REFUSED_REJECTED_BRIEF and REFUSED_REVISE_NEEDS_OVERRIDE), acceptance/README.md (12 priority scenarios).

### Changed

- `cyberos/docs/contracts/README.md` — Layout section rewritten to reflect flat folder structure; "How to add a new contract" recipe simplified (no v<n>/ folder); index extended with `project-brief`, `prd`, plus `fr-to-tech-spec` added as consumer of `feature-request`.
- `cyberos/docs/skills/README.md` Part 8.1 table — Folder location row updated (`<contract-id>/` not `<contract-id>/v<n>/`); Versioned-how row clarifies `contract_version` lives in frontmatter, layout is flat per registry v0.2.4.
- `cyberos/docs/skills/cuo/cpo/SKILL.md` owned-workflows table extended: `requirements-discovery` v0.1.0 (scaffold) + `prd-author` v0.1.0 (scaffold) added as the upstream chain entries; existing fr-author + fr-audit rows preserved.
- All `pin_path` declarations in 3 existing SKILL.md files (fr-author, fr-audit, fr-to-tech-spec) updated from `/v1/` to flat. ~93 string replacements across 24 files completed via sed sweep + verification grep returned clean.
- **`cuo/cpo/SKILL.md` bumped 0.2.0 → 0.3.0 (MAJOR):** scope-ceiling expansion. Added read scopes `company:values`, `memories:refinements`, `member:*`, `client:*`; added `read_excluded: member:*/private/`. Required by the new chain-entry-point skills (requirements-discovery + prd-author) which would otherwise have violated the workflows-must-be-subsets rule. Audit-fix-audit on v0.2.4 surfaced the gap.
- **`cuo/cto/SKILL.md` bumped 0.1.0 → 0.2.0 (MAJOR):** same scope expansion as cpo, applied pre-emptively for srs-author + srs-audit landing in v0.2.6 (Stage C).
- README Part 23.1 + 23.2 indexes extended with the 4 new entries (2 skills, 2 contracts).
- Stale `<contract-id>/v<n>/` references in `feature-request/CONTRACT.md` body, README Recipe 7 + Recipe 13, and fr-to-tech-spec forward-references all updated to flat layout.

### Driver

User-driven design conversation: "the first inputs should be the BRAIN info itself, because i'll create new project and begin interact with it: so BRAIN + human inputs => PRD/SRS/other specs.... => cuo/cpo/fr-author". Identified the chain's missing entry point. Six HITL design questions answered:

- **Q1 naming** — `requirements-discovery` (chosen over `project-discovery`, `intake`, `kickoff`).
- **Q2 project-kind taxonomy** — fr-author stays universal; no kind-based routing.
- **Q3 triage** — fold into requirements-discovery; no separate `project-triage` skill.
- **Q4 PRD audit severity** — PRDs are judgement-heavy; prd-audit (v0.2.5) will be more advisory than fr-audit.
- **Q5 iteration** — amendment-batch protocol (mirror fr-author's).
- **Q6 BRAIN scopes** — defaults applied: `company:locked-decisions`, `company:values`, `memories:projects`, `memories:decisions`, `member:*` (excluding `private/`), `client:*` (when commissioned).

User's bonus question on contracts layout (`<contract-id>/v<n>/` vs flat) — answered as "over-engineered for current scale; simplify now". The simplification was applied as part of v0.2.4.

### MINOR vs PATCH classification

This is a **MINOR** registry bump (not PATCH) for two reasons:

1. New skills added (`requirements-discovery`, `prd-author`) — registry layout grows.
2. New contracts added (`project-brief@1`, `prd@1`).

The contracts-layout simplification (the v<n>/ collapse) is, on its own, a PATCH-level structural cleanup with no semantic changes. But it's bundled into v0.2.4 because the new contracts get authored under the new layout from the start; doing them in two separate releases would introduce inconsistency.

### Backwards compatibility

- Existing skill SKILL.md files keep working — the `pin_path` updates are mechanical; the resolved files at the new paths are byte-identical to the v0.2.3 files (the v1/ folder was just removed; contents unchanged).
- `feature-request@v1` and `nats-subjects@v1` contracts: byte-identical at the new flat path.
- `fr-author`, `fr-audit`, `fr-to-tech-spec`, `cpo`, `cto` SKILL.md files: only `pin_path` lines + body cross-reference paths changed; all other content preserved.
- New skills + contracts are purely additive.
- `requirements-discovery` and `prd-author` carry `gated_until_phase: runtime_v0_3_0` per REF-017; the supervisor MUST NOT route to them until the runtime ships.

---

## v0.2.3 — 2026-05-06 (post-v0.2.2 follow-up: README update + cto persona scaffold + sample PRD; MINOR)

### Layout

- **NEW persona** — `cuo/cto/` registered as the second sub-persona under CUO (after `cuo/cpo/`). Persona-card + CHANGELOG.md authored at v0.1.0. Steward of the technical-artefact lifecycle (tech specs, ADRs, runtime stewardship). Stewards the new `nats-subjects@v1` wire-protocol contract introduced in v0.2.2.
- **NEW skill** — `cuo/cto/fr-to-tech-spec/` scaffolded at v0.1.0. The next downstream skill in the chain after `cuo/cpo/fr-audit`. Consumes audited FRs (pass-verdict only per its INV-001) and emits tech specs. Carries `gated_until_phase: runtime_v0_3_0` — the scaffold ships now (full v0.2.0 frontmatter contract; INVARIANTS.md; envelopes; STANDALONE_INTERVIEW.md; HUMAN_SUMMARY.md; acceptance/README.md), the executable runtime ships in v0.3.0.

### Added

- `cuo/cto/SKILL.md` (v0.1.0 persona-card) — modeled directly on `cuo/cpo/SKILL.md` v0.2.0 with audience-appropriate voice deltas (implementation-feasibility-first; cite the action_log row + metric + trace; dependency direction matters; production-ready ≠ production-deployed).
- `cuo/cto/CHANGELOG.md` — v0.1.0 entry.
- `cuo/cto/fr-to-tech-spec/SKILL.md` — full v0.2.0 frontmatter (33 fields), `depends_on_contracts:` declares both `feature-request@v1` and `nats-subjects@v1`.
- `cuo/cto/fr-to-tech-spec/CHANGELOG.md` — v0.1.0 entry with explicit "what this version DOESN'T do (intentionally)" section.
- `cuo/cto/fr-to-tech-spec/INVARIANTS.md` — 6 invariants. INV-001 (pass-verdict-only ingestion) is sev-0 and is the central seam between "audited FR" and "engineering work".
- `cuo/cto/fr-to-tech-spec/STANDALONE_INTERVIEW.md` — chat-mode entry script (5 questions, validates each answer).
- `cuo/cto/fr-to-tech-spec/HUMAN_SUMMARY.md` — chat-rendered batch-completion template with status emoji mapping + localisation note.
- `cuo/cto/fr-to-tech-spec/envelopes/fr-to-tech-spec.input.json` — JSON Schema (2 required, 6 optional).
- `cuo/cto/fr-to-tech-spec/envelopes/fr-to-tech-spec.output.json` — JSON Schema with HITL_PAUSE / EXHAUSTED / REFUSED branches.
- `cuo/cto/fr-to-tech-spec/acceptance/README.md` — 10 priority scenarios pending v0.3.0 harness (5 sev-0 / 4 sev-1 / 1 sev-2).
- `cuo/cpo/fr-author/acceptance/sample-prd.md` — worked-example PRD ("Saved Searches & Saved Filters"). Realistically-shaped input that demonstrates what fr-author consumes; useful as a manual-walkthrough example until the harness lands.

### Changed

- `README.md` Part 3 (5 inherited contracts table) — `wire_protocol` row's example updated from "the genie.action_log row format itself, when it lands as a contract" (stale) to "`nats-subjects@v1` (subject names + payload shapes for every NATS subject CyberOS skills emit; first concrete wire_protocol contract, registered v0.2.2)".
- `README.md` Part 18 (Anti-patterns) — new entry "Don't over-specify a new contract beyond what consumers actually do" citing the v0.2.2 audit-fix-audit catch + REF-016.
- `README.md` Part 19 (Cookbook) — bumped from "12 recipes" to "13 recipes"; added Recipe 13 "Register a new contract with the audit-fix-audit discipline" (7-step procedure).
- `README.md` Part 23.1 (Skills index) — versions bumped: fr-author v0.2.0 → v0.2.2; fr-audit v0.2.0 → v0.2.2; new row for `cuo/cto/fr-to-tech-spec` v0.1.0 (scaffold).
- `README.md` Part 23.2 (Contracts index) — new row for `nats-subjects@v1`; existing `feature-request@v1` row updated to include `cuo/cto/fr-to-tech-spec` v0.1.0+ as a consumer.
- `README.md` table of contents — Part 19 entry updated to "Cookbook: 13 recipes".

### Driver

User-driven follow-up after registry v0.2.2 audit-fix-audit loop completed. Direct quote: "Q1: apply all you can — Q2: apply all you can — yes, yes." Q1 was README updates (5 changes); Q2 was next-step actions for fr-author + fr-audit (scaffold cto + worked-example PRD); the two "yes"es confirmed both. Acts on the next-step plan from the post-audit recommendations; nothing here is novel design, it's all execution of plans documented earlier in the conversation.

### Backwards compatibility

- README is documentation-only; readers see the updated text on next load.
- `cuo/cto/` persona is additive; no existing skill or contract changes meaning.
- `fr-to-tech-spec` is gated (`gated_until_phase: runtime_v0_3_0`) — the supervisor MUST NOT route to it until the runtime ships. Until then, the skill folder is documentation that the future runtime will satisfy.
- Sample PRD under `acceptance/` is additive; existing acceptance/README.md still describes the priority scenarios pending the harness.

### MINOR vs PATCH classification

This is a **MINOR** registry bump (not PATCH) because a new persona namespace was added (`cuo/cto/`), which extends the registry layout per the SemVer-at-registry-level rules at the top of this CHANGELOG. PATCH would have been the right choice for any combination of (a) README updates only, (b) per-skill version bumps, (c) docs cleanup. New persona = MINOR.

---

## v0.2.2 — 2026-05-06 (fr-author + fr-audit pre-deployment audit + Tier-2/3 absorption; PATCH)

### Layout

- **NEW contract** — `cyberos/docs/contracts/nats-subjects/` registered. Wire-protocol contract documenting every NATS subject emitted or subscribed by a CyberOS skill (subject naming convention, payload schemas, QoS levels, durability tiers, operational protocol). Stewarded by `cuo-cto`. First consumers: `cuo/cpo/fr-author` + `cuo/cpo/fr-audit` v0.2.2. Three files: `CONTRACT.md` + `schema.json` + `protocol.md` + `CHANGELOG.md`. Resolves the gap that both fr-author and fr-audit emitted NATS subjects without a declared contract — risked future skills colliding on subject names without a single source of truth.

### Changed

- `cuo/cpo/fr-author` v0.2.0 → v0.2.1 → v0.2.2:
  - **v0.2.1 (Tier-1)** — dead links to `references/HASHING.md` + `references/OUTPUT_FORMATS.md` resolved to actual files; input envelope schema's `required` array aligned with SKILL.md `expects.required_fields` (6 → 3); `chain_to` documented in `optional_fields`; `acceptance/README.md` stub added with 9 priority scenarios.
  - **v0.2.2 (Tier-2/3)** — `depends_on_contracts:` extended with `nats-subjects/v1`; `references/README.md` added (index + per-skill divergence note explaining why HITL_PROTOCOL/UNTRUSTED_CONTENT/ANTI_FABRICATION/EU_AI_ACT_DECISION_TREE differ between fr-author and fr-audit by SHA-256, deferred consolidation to v0.3.0).
- `cuo/cpo/fr-audit` v0.2.0 → v0.2.1 → v0.2.2:
  - **v0.2.1 (Tier-1)** — missing `stale_fr_disposition` added to CONTRACT_ECHO `hitl_categories` (STALE-001 maps to it but it was undeclared); stale `skill_version: 0.1.0` example fixed in output-envelope JSON; input envelope schema's `required` trimmed (3 → 1) and `rubric_version` field added; `caller_persona` + `max_iterations_per_fr` documented in `optional_fields`; `acceptance/README.md` stub added with 10 priority scenarios including INV-001 (verdict determinism) as sev-0.
  - **v0.2.2 (Tier-2/3)** — `depends_on_contracts:` extended with `nats-subjects/v1`; `references/README.md` added; `RUBRIC.md` §15.9 (`## Confidence-band reporting`) added — documents per-rule confidence bands (mechanical-rule majority ≥0.95; LLM-judgement minority QA-007 / QA-009 capped at 0.7); `AUDIT_LOOP.md` §"Deterministic-input rule" added — defines the closed input set for verdict computation, makes INV-001's auto-refinement template's anchor target resolve cleanly; INV-006 severity demoted from `error` to `info` (schema validation already enforces presence + range; runtime invariant was redundant).
- `cuo/cpo/SKILL.md` owned-workflows table updated to v0.2.2 / v0.2.2.
- `cyberos/docs/contracts/README.md` index extended with the `nats-subjects` row.

### Driver

User-driven request to "audit and refine fr-author and fr-audit", followed by HITL approval to absorb Tier-2/3 follow-ups ("HITL decisions, do as your suggestions"). Ran the manual-fine-tune playbook (registry README Part 7) in pre-deployment mode. Applied the README Part 24.1 self-test checklist + Part 18 anti-pattern scan + cross-skill consistency check. Six Tier-1 findings absorbed first (v0.2.1); five Tier-2/3 findings absorbed second (v0.2.2): B1 (per-skill divergence — documented as intentional), B2 (NATS subjects undocumented — promoted to wire-protocol contract), B3 (confidence bands per rule — documented), B4 (INV-006 redundancy — demoted), C3 (deterministic-input rule referenced but never defined — added). Two Tier-3 items deferred: C1 (batch_size soft-cap — already in schema description), and the four-way reference-doc consolidation (deferred to v0.3.0 when consolidation pain is shown to outweigh per-skill clarity).

### Backwards compatibility

Pure PATCH cleanup at the registry level. No frontmatter contract changes. No envelope shape changes. No rule changes (rubric IDs + severities + verdicts unchanged). No audit row format changes. Both skills remain at v0.2.0 frontmatter contract; v0.2.2 just brings their schemas + bodies + dead links + cross-references into alignment AND introduces the new wire-protocol contract additively. The new `nats-subjects` contract is additive; skills that don't yet declare it have no contract to reference (de-facto behaviour preserved). Existing v0.2.0 manifests resume cleanly.

---

## v0.2.1 — 2026-05-06 (README expansion + diagrams to assets + bigger infographic)

### Changed

- **`README.md`** — substantially expanded from 27 to 27+ Parts with new content covering runtime architecture (LangGraph + action_log + NATS), security model deep-dive, performance & observability, localization & i18n, anti-patterns, per-persona quickstart, migration paths from non-CyberOS skills, and an end-to-end worked example chaining fr-author → fr-audit. **Removed Part 0 (CyberSkill Design System)** — not skill-related; the design system is applied silently to visual artefacts but isn't a skill-wiki concern. Reorganised TOC to 27 Parts.
- **All embedded Mermaid diagrams extracted to standalone SVG files** under `assets/diagrams/`. README now references each diagram via `![alt](./assets/diagrams/NN-name.svg)` rather than inlining Mermaid blocks. Cleaner rendering across viewers; no more in-page diagram bugs; each diagram is independently printable. Eleven diagrams total: skill-folder-anatomy, frontmatter-field-families, five-contracts, dual-mode-invocation, exposability-surfaces, auto-refinement-loop, manual-fine-tune-7-step, host-adapter-pipeline, validation-pyramid, skill-lifecycle-state, fr-author-fr-audit-chain-sequence.
- **All prose paragraphs rewritten as single unbroken lines** (no manual hard-wraps mid-sentence). Hard-wrapping at column 80 was producing visually-fragmented text in some Markdown viewers where the last word or two of a sentence ended up alone on a wrapped line, looking like orphan list items. Fixed across the entire README.
- **`assets/skills-anatomy-infographic.svg`** — remade as one connected master infographic. Old version was 1600×3200 with 8 stacked sections that didn't visually link. New version is 2400×4800 with 8 numbered bands (① INPUT → ② SKILL + 5 contracts → ③ DUAL-MODE → ④ EXPOSABILITY → ⑤ AUTO-REFINEMENT → ⑥ MANUAL FINE-TUNE → ⑦ HOST-ADAPTER PIPELINE → ⑧ DESTINATIONS) with explicit connecting arrows showing data flow end-to-end. Larger type, more breathing room, printable at poster size.

### Added

- `assets/diagrams/` — eleven standalone SVG diagrams (one per major concept). Each carries its own filename caption at the bottom for traceability when extracted.
- README Part 11 — worked example end-to-end: fr-author → fr-audit. Narrated walk-through plus the sequence diagram and the action_log SQL trace.
- README Part 12 — runtime architecture deep-dive: LangGraph supervisor + genie.action_log + NATS event bus + crash recovery semantics.
- README Part 15 — security model deep-dive: scope contract, untrusted-content discipline, denylist, EU AI Act compliance, hash-chain integrity.
- README Part 16 — performance & observability: latency budgets per layer, OBS metrics, logging conventions, tracing.
- README Part 17 — localization & i18n: manifest-level / body-level / artefact-level language handling for the Vietnam-English dual operation.
- README Part 18 — anti-patterns: twelve specific things NOT to do, with reasoning.
- README Part 19 — cookbook expanded from 7 to 12 recipes (added: acceptance fixtures, INVARIANTS.md authoring, refinement_proposal that humans approve, plan a v0.x → v1.0 promotion, run a fine-tune cycle).
- README Part 21 — per-persona quickstart: pointers and considerations for each of the 14 personas as they come online.
- README Part 22 — migration from non-CyberOS skills: from flat Anthropic SKILL.md, from Claude Code plugin, from vanilla MCP tool, from a freeform LLM prompt.

### Removed

- README Part 0 (CyberSkill Design System) — content was off-topic for a skill wiki. The palette + typography rules still apply silently to visual artefacts (infographic, diagrams) but no longer occupy a Part of the wiki.

### Driver

User feedback after v0.2.0 release (2026-05-06): (1) "the embed visualize materials/workflows in README got many UI bugged, I suggest to move them to assets as images for better display" (2) "sentences was cutoff by newline, check and fix all" (3) "no need to mention about design system as it's not related to skills" (4) "double check if README includes all necessary aspects and comprehensive enough — feeling it still short & limited" (5) "the infographic was cut off at the bottom section, remake to make it even better and more informative, everything need connect to make a overall big picture, don't limit image size & ratio".

### Backwards compatibility

- No frontmatter contract change. v0.2.0 SKILL.md files remain valid.
- No file move. The diagram extraction adds new files under `assets/diagrams/` without removing or renaming existing files.
- Bumped registry from v0.2.0 to v0.2.1 (PATCH — pure documentation refinement).

---

## v0.2.0 — 2026-05-06 (contract expansion: dual-mode + self-audit + manual fine-tune + host portability + skills↔contracts split)

### Layout

- **NEW namespace** — `cyberos/docs/contracts/` is now a sibling of `cyberos/docs/skills/`. Holds versioned schema artefacts (artefact schemas, envelope schemas, wire protocols) consumed by skills via `depends_on_contracts:`. Documented in `cyberos/docs/contracts/README.md`.
- **Promoted** — `cuo/_shared/feature-request-template/` → `cyberos/docs/contracts/feature-request/`. `SKILL.md` renamed to `CONTRACT.md` with a smaller, contract-only frontmatter (drops `allowed_mcp_tools`, `expects/produces`, `audit`, etc.). Body of `template.md` is byte-identical.
- **Consolidated** — `GETTING_STARTED.md` retired. All operational content absorbed into `README.md` as Parts 1–17. The README is now the single comprehensive wiki for the registry; per the v0.2.0 brief, no new top-level docs.

### Added

- **Frontmatter contract: 27 → 33 fields.**
  - `invocation_modes` — `[standalone, chained]` for workflows; `[persona_routing_only]` for persona-cards. (DEC-091)
  - `expects.optional_fields` and `expects.standalone_interview_ref` — enable chat-mode entry without a pre-built input envelope.
  - `produces.human_summary_ref` — chat-rendered summary template.
  - `depends_on_contracts:` — list pinning consumed contracts to a specific path + major version. Validators reject skill bodies that reference contracts not declared here. (DEC-090)
  - `exposable_as` — 4-bool block declaring which surfaces the skill ships through (`internal`, `agent_plugin`, `mcp_tool`, `partner_connector`). (DEC-091)
  - `self_audit` — block carrying `invariants_ref`, `check_at`, `anomaly_signals`, `on_breach`. Runtime checks invariants at declared checkpoints; breaches emit `refinement_proposal` envelopes and pause the pipeline. (DEC-092)
  - `human_fine_tune` — block carrying `fine_tuner_role`, `review_required`, `signals_to_initiate`, `procedure_ref`, `required_artifacts`, `blackout_windows`. The structured manual fine-tune playbook lives in README Part 7. (DEC-093)
- **New `produces.output_kind` enum value** — `refinement_proposal` (alongside notify / question / review / act / artefact).
- **New `audit.row_kind` enum value** — `self_refinement_proposal`.
- **Per-skill files** — `INVARIANTS.md`, `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md` are now first-class citizens of every Tier-2 skill. Required by the README Part 16.1 self-test checklist.
- **README Part 0 — CyberSkill Design System** — voice rules, palette (Cyber Indigo + Will Amber), typography, layout grid, status emoji vocabulary, slogan placement. Applied to every artefact going forward (Mermaid diagrams, infographic, audit reports, HUMAN_SUMMARY templates).
- **README Parts 4–9** — comprehensive treatment of dual-mode invocation, exposability, auto-refinement, manual fine-tune, the skills↔contracts split, and the host-adapter strategy (CCSM → transpilers → host shim → equivalence test matrix).
- **README Part 13 — Cookbook** — 7 recipes including Recipe 7 (promoting a `_shared/` skill to a contract), the canonical example of which is `feature-request@v1`.

### Changed

- **`cuo/cpo/fr-author`** — v0.1.0 → v0.2.0. Adopts every new frontmatter block. Adds `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md`, `INVARIANTS.md` (8 invariants: citation completeness, manifest↔disk parity, ingestion coverage, FR-ID uniqueness, fabrication boundary, scope discipline, EU AI Act non-degradation, confidence reporting). Output envelope shape unchanged.
- **`cuo/cpo/fr-audit`** — v0.1.0 → v0.2.0. Same v0.2.0 frontmatter expansion. `INVARIANTS.md` adds INV-001 (verdict determinism) as a sev-0 invariant — fr-audit's reproducibility is its highest-value contract.
- **`cuo/cpo/SKILL.md`** persona-card — v0.1.0 → v0.2.0. Adopts the persona-strict subset of new fields (no pipeline interface, no contract dependencies). Owned-workflow table updated to v0.2.0.
- **`cuo/README.md`** — `_shared/` index updated. The former `feature-request-template` row marked as "promoted to contract" with a pointer to the new location.
- **All cross-references to `cuo/_shared/feature-request-template/`** updated across `cuo/cpo/fr-author/`, `cuo/cpo/fr-audit/`, `cuo/cpo/AUDIT_TRACE_EXAMPLE.md`, `cuo/cpo/fr-author/PIPELINE.md`, reference docs, and the registry README. Old path 100% retired outside of historical CHANGELOG entries (v0.1.0 entries preserved intact as history).

### Removed

- `cyberos/docs/skills/GETTING_STARTED.md` — content fully absorbed into `README.md` Parts 1–17. The registry no longer carries two parallel docs.
- `cyberos/docs/skills/cuo/_shared/feature-request-template/` — promoted to a contract; old folder deleted in the same commit that lands the v1.1.0 contract entry. Body byte-preserved at the new location.

### Driver

User-explicit requirements (2026-05-06):

1. *"Every single skill can adapt — work both as standalone or pipeline/chaining."* → DEC-091 invocation_modes + standalone interview + human summary.
2. *"Used to build plugins/connectors/MCPs that CyberOS will expose for partners."* → DEC-091 exposable_as.
3. *"Audit itself to detect issues/abnormal behaviour at runtime and proactively suggest self refinements, so controllers can do HITL to apply necessary changes."* → DEC-092 self-audit + refinement_proposal.
4. *"Manual fine-tune strategy which can be done by human."* → DEC-093 human_fine_tune + README Part 7 7-step playbook.
5. *"Suggest comprehensive step-by-step strategy to build adapters … port/map/convert the skills to serve multiple hosts (Antigravity, Codex, …)."* → README Part 9 phased plan (CCSM → transpilers → host shim → equivalence matrix).
6. *"For skills I don't want too many documents, let's combine all into README.md inside skills folder."* → GETTING_STARTED retired; single comprehensive README.
7. *"Have to cover and give comprehensive step-by-step guidelines for all possible cases relate to skills … with simple/practical examples and visualize materials … as a detailed wiki so CyberSkill's employees can easily learn/digest & improve it."* → 19 Parts, 7+ Mermaid diagrams, 7 recipes, FAQ, glossary.

Plus three §0.4 refinement candidates surfaced in conversation (continuing the BRAIN's REF sequence — REF-001..011 already exist):

- **REF-012** — split frontmatter contract by audience (portable Anthropic-skill fields vs. CyberOS runtime extensions vs. v0.2.0 governance). Adopted as README Part 2.2.
- **REF-013** — declare cross-skill dependencies in frontmatter. Adopted as `depends_on_contracts:` (DEC-090).
- **REF-014** — promote AGENTS.md §0.4 from protocol-level to skill-level. Adopted as `self_audit:` + `INVARIANTS.md` + `refinement_proposal` envelope (DEC-092).

### Backwards compatibility

- **Registry layout** — adds `cyberos/docs/contracts/`. Existing `cyberos/docs/skills/` tree shape is unchanged; only one folder relocated (`feature-request-template` → contracts).
- **Frontmatter contract** — pure additions. v0.1.x SKILL.md files remain valid; the validator gates "passes v0.2.0 self-test checklist" but does NOT reject v0.1.x files outright. Skills can promote to v0.2.0 at their own cadence per Recipe 4 (README Part 13).
- **Audit row schema** — extended additively (`refinement_proposal`, `self_refinement_proposal` are new enum values). Existing rows still parse cleanly.
- **Output envelope shapes** — fr-author and fr-audit envelope shapes unchanged. v0.2.0 additions all sit under new top-level keys.

### Migration notes for existing skills

To bring a v0.1.x skill to v0.2.0:

1. Add the 6 new frontmatter blocks per README Part 2.1.
2. Author `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md`, `INVARIANTS.md` (Recipe 4).
3. Bump `skill_version` 0.1.x → 0.2.0; add CHANGELOG entry citing registry v0.2.0.
4. If the skill consumes the FR template, replace any reference to `cuo/_shared/feature-request-template/` with the new contract path AND add a `depends_on_contracts:` entry.
5. Run the README Part 16.1 self-test checklist before committing.

### Known follow-ups (tracked outside this CHANGELOG)

- Build the transpilers + host shim per README Part 9 phases B–D (target: v0.3.0).
- Author the onboarding infographic (target: paired with this release; tracked separately).
- Build the partner-connector pipeline per README Part 9 Phase E (target: v0.4.0; gated on partner-exposure DEC).
- Migrate any future `_shared/` schema-shaped skills into the contracts namespace per Recipe 7.

---

## v0.1.2 — 2026-05-05 (comprehensive guide + hello-world skill)

### Added

- `cuo/_shared/hello-world/` — the simplest possible CyberOS skill, authored as a teaching example. Carries the full 27-field frontmatter contract with the most trivial body (read a name → write a greeting markdown). Includes `acceptance/` golden-input + golden-output + golden-envelope fixtures (`greeting_sha256`: `ddd394ab7eaa5950ce5ab2ea9f7eb37199fd0d5d42a37be9fdf56ec490d39805`). Used as Example 1 throughout `GETTING_STARTED.md`.

### Changed

- `GETTING_STARTED.md` — substantially expanded into a comprehensive basic→advanced guide. Now organised into three tiers (🌱 Beginner, 🌿 Intermediate, 🌳 Advanced) with 20 numbered sections, 6 embedded Mermaid diagrams (skill-as-folder, three trigger paths, frontmatter anatomy, chain sequence, validation pyramid, fine-tuning loop, skill lifecycle state diagram), 5 cookbook recipes (build / chain / debug / retire / add-persona), an FAQ section covering 8 common confusions, and a glossary of 22 terms.
- README.md and registry CHANGELOG entry for v0.1.1 unchanged but now point at the much more comprehensive guide.

### Driver

User feedback after v0.1.1: "comprehensive as possible, basic → advanced; simple examples for newbies; visualisations help more than text." The previous v0.1.1 GETTING_STARTED.md was a quick on-ramp; this v0.1.2 expansion turns it into the canonical learning curriculum.

### Backwards compatibility

Pure additions. The hello-world skill is deliberately at v1.0.0 (not v0.1.0) because its purpose — a teaching example — is locked. Future v2.0.0 would mean a different skill entirely; bumping the existing one is forbidden.

---

## v0.1.1 — 2026-05-05 (operational guide)

### Added

- `cyberos/docs/skills/GETTING_STARTED.md` — the operational view of the registry: 30-second mental model, the two unrelated meanings of "audit" (action_log row vs. fr-audit skill), the three trigger paths (direct / supervisor-routed / chained), a 5-command worked example for building a tiny new skill (`fr-priority-rebalance`), the three layers of skill validation (mechanical / functional / operational), the fine-tuning lifecycle (tightening, prompt refinement, acceptance-set growth, drift-signal feedback, replacement vs revision), a "what doesn't exist yet" section, and a TL;DR cookbook table.
- `acceptance/` folder convention referenced. Skills SHOULD ship golden-input + golden-output pairs for regression testing; the runner is not yet built.
- README.md updated to point at GETTING_STARTED.md as the entry point.

### Driver

User feedback after v0.1.0: "the structure is complicated, and after all I still have no idea step by step about how to build a skill, trigger it standalone/chained, audit it, validate it worked, fine-tune it." The architecture docs answered "what" and "why" but not "how do I do this on Tuesday afternoon." GETTING_STARTED.md is the missing operational on-ramp.

### Backwards compatibility

Pure additions; no existing skill needs to change. Existing reference docs continue to be authoritative; GETTING_STARTED.md cross-references them in its "Map: when to read which architecture doc" section rather than duplicating them.

---

## v0.1.0 — 2026-05-05 (initial registry bootstrap)

### Added

- `cyberos/docs/skills/README.md` — registry contract: layout (Option B, persona-grouped + nested workflow skills), SKILL.md frontmatter contract, the five inherited contracts (audit / chain / plug-in / versioning / trust), routing rules, and citations to the authoritative PRD/SRS/AGENTS.md sections.
- `cyberos/docs/skills/cuo/README.md` — CUO persona namespace index. Lists the 14 sub-personas (10 canonical + 4 emergent) per DEC-052; marks per-phase availability.
- `cyberos/docs/skills/cuo/cpo/SKILL.md` — first persona-card (Chief Product Officer). Owns FR backlog management.
- `cyberos/docs/skills/cuo/_shared/feature-request-template/` — first cross-persona shared skill: holds the canonical `feature_request@1` template (sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18).
- `cyberos/docs/skills/cuo/cpo/fr-author/` — port of the create-and-audit prompt's create half (sections §0–§14 + §18 of v2.0.0). Standalone trigger: PRD → backlog → FR markdowns. Produces FR files + a `fr-manifest@2` state file.
- `cyberos/docs/skills/cuo/cpo/fr-audit/` — port of the create-and-audit prompt's audit half (sections §15–§17 of v2.0.0, plus shared §7 HITL + §12 untrusted-content). Standalone trigger: existing FR markdowns → sibling audit reports. Chains naturally after `fr-author`.

### Layout decision (Option B trade-off)

Three layouts were considered (full diagram retained in the conversation log of 2026-05-05). Option B was selected because:

1. It is the only layout that keeps each workflow as a standalone-trigger atom AND preserves persona grouping in the filesystem AND honours PRD §3.2's `cuo/<role>/` mandate AND DEC-061's reusable-skill clause (via `_shared/`).
2. The audit row schema in SRS §6.7 (`persona_id`, `skill_id`, `skill_version`, `row_kind`) maps 1:1 to the workflow leaf without requiring a sub-skill field.
3. Plug-in extraction works at three granularities (workflow / persona / whole-CUO) without restructuring.

### Skill self-test checklist (run before committing any new SKILL.md)

A skill is registry-valid when ALL of:

- [ ] Folder name is kebab-case and matches `name:` in frontmatter.
- [ ] `SKILL.md` parses as Markdown with one YAML frontmatter block, no mid-file `---` outside fenced code spans (AGENTS.md §4.3 + DEC-087).
- [ ] All 27 frontmatter fields from `cyberos/docs/skills/README.md` §3 are present (or explicitly `null` where allowed).
- [ ] `expects:` and `produces:` reference real JSON schemas reachable from this folder or `_shared/`.
- [ ] `allowed_brain_scopes.write` is empty UNLESS the skill is explicitly authorised to mutate BRAIN (separate decision per skill, recorded in CHANGELOG).
- [ ] `allowed_mcp_tools` is exhaustive — gateway will reject unlisted tools at call time.
- [ ] `audit.row_kind` matches the `produces.output_kind` enum.
- [ ] At least one `references/` doc OR a clear note that none are needed.
- [ ] `CHANGELOG.md` exists in the skill folder, with at least a v0.1.0 entry.
- [ ] Adding the skill to `cyberos/docs/skills/README.md` §7 index does not duplicate an existing `(persona, name)` pair.

### Known follow-ups (tracked outside this CHANGELOG)

- Wire the registry into the CyberOS-PRD/SRS source-of-truth (a one-line reference from PRD Part 6 + SRS Part 6.2 pointing here). Parked because PRD/SRS are .docx and must be edited in Word; raised as a separate feature request once `fr-author` is operational and can self-host the request.
- Migrate the existing `feature-request/FR_CREATE_AND_AUDIT.md` repo into this registry as a soft-deprecation: leave the prompt in place, point its README to `cyberos/docs/skills/cuo/cpo/fr-author/` + `fr-audit/`. Bump that prompt's CHANGELOG to v2.1.0 with a "MOVED" note.
- Define `_shared/` for additional cross-persona skills as they emerge (e.g., `draft-payslip-explanation` from DEC-061's worked example, owned by neither CFO nor CHRO exclusively).

---

## How to add a future entry

For a new release, prepend a new `## vX.Y.Z — <ISO date> (<one-line summary>)` block above v0.1.0. Standard sub-sections:

- **Added** — new skills, new personas, new shared assets, new contracts.
- **Changed** — semantics changes that don't break the layout or frontmatter contract.
- **Deprecated** — skills moving to `superseded_by:` in their frontmatter.
- **Removed** — soft-deletions only; skill folders move to `cuo/<role>/_archive/<skill-id>/` with a tombstone CHANGELOG entry. The folder body is preserved for audit (per AGENTS.md §4.6).
- **Layout** — only on MAJOR bumps; describes the new tree shape.
- **Backwards compatibility** — what existing skills still validate, what needs migration.
