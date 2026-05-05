# CHANGELOG — `cyberos/docs/skills/` registry

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the registry level: MAJOR breaks the layout or the SKILL.md frontmatter contract; MINOR adds a new persona namespace or new contract sections; PATCH is editorial / typo fixes.

---

## v0.2.1 — 2026-05-06 (README expansion + diagrams to assets + bigger infographic)

### Changed

- **`README.md`** — substantially expanded from 27 to 27+ Parts with new content covering runtime architecture (LangGraph + action_log + NATS), security model deep-dive, performance & observability, localization & i18n, anti-patterns, per-persona quickstart, migration paths from non-CyberOS skills, and an end-to-end worked example chaining fr-create → fr-audit. **Removed Part 0 (CyberSkill Design System)** — not skill-related; the design system is applied silently to visual artefacts but isn't a skill-wiki concern. Reorganised TOC to 27 Parts.
- **All embedded Mermaid diagrams extracted to standalone SVG files** under `assets/diagrams/`. README now references each diagram via `![alt](./assets/diagrams/NN-name.svg)` rather than inlining Mermaid blocks. Cleaner rendering across viewers; no more in-page diagram bugs; each diagram is independently printable. Eleven diagrams total: skill-folder-anatomy, frontmatter-field-families, five-contracts, dual-mode-invocation, exposability-surfaces, auto-refinement-loop, manual-fine-tune-7-step, host-adapter-pipeline, validation-pyramid, skill-lifecycle-state, fr-create-fr-audit-chain-sequence.
- **All prose paragraphs rewritten as single unbroken lines** (no manual hard-wraps mid-sentence). Hard-wrapping at column 80 was producing visually-fragmented text in some Markdown viewers where the last word or two of a sentence ended up alone on a wrapped line, looking like orphan list items. Fixed across the entire README.
- **`assets/skills-anatomy-infographic.svg`** — remade as one connected master infographic. Old version was 1600×3200 with 8 stacked sections that didn't visually link. New version is 2400×4800 with 8 numbered bands (① INPUT → ② SKILL + 5 contracts → ③ DUAL-MODE → ④ EXPOSABILITY → ⑤ AUTO-REFINEMENT → ⑥ MANUAL FINE-TUNE → ⑦ HOST-ADAPTER PIPELINE → ⑧ DESTINATIONS) with explicit connecting arrows showing data flow end-to-end. Larger type, more breathing room, printable at poster size.

### Added

- `assets/diagrams/` — eleven standalone SVG diagrams (one per major concept). Each carries its own filename caption at the bottom for traceability when extracted.
- README Part 11 — worked example end-to-end: fr-create → fr-audit. Narrated walk-through plus the sequence diagram and the action_log SQL trace.
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
- **Promoted** — `cuo/_shared/feature-request-template/` → `cyberos/docs/contracts/feature-request/v1/`. `SKILL.md` renamed to `CONTRACT.md` with a smaller, contract-only frontmatter (drops `allowed_mcp_tools`, `expects/produces`, `audit`, etc.). Body of `template.md` is byte-identical.
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

- **`cuo/cpo/fr-create`** — v0.1.0 → v0.2.0. Adopts every new frontmatter block. Adds `STANDALONE_INTERVIEW.md`, `HUMAN_SUMMARY.md`, `INVARIANTS.md` (8 invariants: citation completeness, manifest↔disk parity, ingestion coverage, FR-ID uniqueness, fabrication boundary, scope discipline, EU AI Act non-degradation, confidence reporting). Output envelope shape unchanged.
- **`cuo/cpo/fr-audit`** — v0.1.0 → v0.2.0. Same v0.2.0 frontmatter expansion. `INVARIANTS.md` adds INV-001 (verdict determinism) as a sev-0 invariant — fr-audit's reproducibility is its highest-value contract.
- **`cuo/cpo/SKILL.md`** persona-card — v0.1.0 → v0.2.0. Adopts the persona-strict subset of new fields (no pipeline interface, no contract dependencies). Owned-workflow table updated to v0.2.0.
- **`cuo/README.md`** — `_shared/` index updated. The former `feature-request-template` row marked as "promoted to contract" with a pointer to the new location.
- **All cross-references to `cuo/_shared/feature-request-template/`** updated across `cuo/cpo/fr-create/`, `cuo/cpo/fr-audit/`, `cuo/cpo/AUDIT_TRACE_EXAMPLE.md`, `cuo/cpo/fr-create/PIPELINE.md`, reference docs, and the registry README. Old path 100% retired outside of historical CHANGELOG entries (v0.1.0 entries preserved intact as history).

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
- **Output envelope shapes** — fr-create and fr-audit envelope shapes unchanged. v0.2.0 additions all sit under new top-level keys.

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
- `cyberos/docs/skills/cuo/cpo/fr-create/` — port of the create-and-audit prompt's create half (sections §0–§14 + §18 of v2.0.0). Standalone trigger: PRD → backlog → FR markdowns. Produces FR files + a `fr-manifest@2` state file.
- `cyberos/docs/skills/cuo/cpo/fr-audit/` — port of the create-and-audit prompt's audit half (sections §15–§17 of v2.0.0, plus shared §7 HITL + §12 untrusted-content). Standalone trigger: existing FR markdowns → sibling audit reports. Chains naturally after `fr-create`.

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

- Wire the registry into the CyberOS-PRD/SRS source-of-truth (a one-line reference from PRD Part 6 + SRS Part 6.2 pointing here). Parked because PRD/SRS are .docx and must be edited in Word; raised as a separate feature request once `fr-create` is operational and can self-host the request.
- Migrate the existing `feature-request/FR_CREATE_AND_AUDIT.md` repo into this registry as a soft-deprecation: leave the prompt in place, point its README to `cyberos/docs/skills/cuo/cpo/fr-create/` + `fr-audit/`. Bump that prompt's CHANGELOG to v2.1.0 with a "MOVED" note.
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
