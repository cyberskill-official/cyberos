# Changelog — CyberOS-PRD.docx

All notable changes to **CyberOS-PRD.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

---

## 2026-05-06 — Registry v0.2.4 absorbed (chain entry point; MINOR within scope of DEC-090..093)

### No .docx edits this round

Registry v0.2.4 added the upstream chain entry point — `cuo/cpo/requirements-discovery` (BRAIN + 20-q interview → `project_brief@1`) and `cuo/cpo/prd-author` (brief + 3-5 follow-ups → `prd@1`) — plus 2 new artefact contracts (`project-brief@1`, `prd@1`). The registry-level changes are operationalisation of existing DEC-090..093 surfaces; no PRD body content moves. The chain entry point fills a gap (PRDs were previously assumed-as-input to fr-author; v0.2.4 makes the upstream "BRAIN + human → PRD" path explicit).

The cpo persona-card MAJOR bump (v0.2.0 → v0.3.0) and cto MAJOR bump (v0.1.0 → v0.2.0) — both for scope-ceiling expansion to support the new skills — are persona-internal SemVer movements; PRD §6.3 (14-persona registry) is unchanged.

### What absorbed

- Chain entry-point design — answers the user's "BRAIN + human inputs => PRD/SRS/other specs" framing without requiring new DEC entries (DEC-090..093 already cover the contract + skill machinery).
- Contracts layout simplification (flat folders) — REF-018 in BRAIN; PATCH-level structural cleanup, no semantic change.

---

## 2026-05-06 — Registry v0.2.2 absorbed (Tier-2/3 follow-up; PATCH within scope of DEC-090..093)

### No .docx edits this round

The registry-level changes recorded under `cyberos/docs/skills/CHANGELOG.md` v0.2.2 — including the new `cyberos/docs/contracts/nats-subjects/` wire-protocol contract, the per-skill divergence note in `references/README.md`, the confidence-band documentation in `RUBRIC.md` §15.9, the deterministic-input rule in `AUDIT_LOOP.md`, and the INV-006 severity demotion — are PATCH-level operationalisation of the existing DEC-090..093 family. They do NOT alter any of the four locked decisions and do NOT introduce new product-level surfaces. Per the registry's manual-fine-tune playbook (registry README Part 7), PATCH-level absorptions of audit findings are recorded in the registry CHANGELOG and the per-skill CHANGELOGs but do NOT require a new PRD `§5.11` paragraph or a new DEC entry.

### Why this is recorded here at all

For the same reason every PRD changelog entry exists: traceability. A future reader auditing "what does the PRD currently say about the skill registry?" should be able to see at a glance that v0.2.2 happened, that it was Tier-2/3 absorption, and that no PRD body content moved. If they need to dive deeper, the path is `cyberos/docs/skills/CHANGELOG.md` v0.2.2 → the per-skill CHANGELOGs → the actual files.

### What absorbed

- **B1** (per-skill reference doc divergence) — documented as intentional in `references/README.md` files; deferred consolidation to v0.3.0.
- **B2** (NATS subjects undocumented) — promoted to a wire-protocol contract under `cyberos/docs/contracts/nats-subjects/`. New contract, no PRD-level surface change.
- **B3** (per-rule confidence bands) — documented in `RUBRIC.md` §15.9. No PRD-level surface change.
- **B4** (INV-006 redundancy with schema) — severity demoted from `error` to `info`. No PRD-level surface change.
- **C3** (deterministic-input rule referenced but never defined) — added to `AUDIT_LOOP.md`. No PRD-level surface change.

---

## 2026-05-06 — §5.11 Skill-registry v0.2.0 (dual-mode + exposability + self-audit + manual fine-tune + skills↔contracts split + host portability) + DEC-090 / DEC-091 / DEC-092 / DEC-093

### Applied to CyberOS-PRD.docx (programmatically via python-docx, 2026-05-06 evening)

The following sub-sections have landed in the .docx (16 paragraphs inserted between §5.10's tail and Part 6, matching existing Heading 2 / Heading 3 / default styles):

- **§5.11** new section "Skill-registry v0.2.0 — making every skill standalone-or-pipeline, plugin-shippable, self-auditing, manually fine-tunable, and host-portable" with five sub-sections:
  - **§5.11.1** Skills↔contracts split: schemas (`feature_request@1`, future envelope contracts) move from `cuo/_shared/` to a sibling `cyberos/docs/contracts/` namespace. Skills declare consumption via `depends_on_contracts:`. (DEC-090)
  - **§5.11.2** Dual-mode invocation: every workflow skill works standalone (chat-mode entry via `STANDALONE_INTERVIEW.md`) AND chained (envelope from upstream skill). One function body, two front doors. Plus the `exposable_as` frontmatter declaring which surfaces the skill ships through (`internal`, `agent_plugin`, `mcp_tool`, `partner_connector`). (DEC-091)
  - **§5.11.3** Self-audit + auto-refinement: every Tier-2 skill carries `INVARIANTS.md` declaring runtime truths. Breaches emit a `refinement_proposal` envelope (new output_kind), pause the pipeline, surface as a Question primitive for human review. (DEC-092)
  - **§5.11.4** Manual fine-tune playbook: the 7-step structured cycle for human-driven skill improvement (pause → diagnose → add regression → edit → re-run suite → bump+log → resume). Frontmatter `human_fine_tune` declares fine_tuner_role, review_required gates, signals_to_initiate, required_artifacts, blackout_windows. (DEC-093)
  - **§5.11.5** Host-adapter strategy: SKILL.md as the Canonical CyberSkill Skill Manifest (CCSM); per-host artefacts under `dist/<host>/` are GENERATED by transpilers (`ccsm-to-anthropic`, `ccsm-to-mcp-tool`, `ccsm-to-claude-plugin`, `ccsm-to-antigravity`, `ccsm-to-codex`, `ccsm-to-cursor`); a host shim library (`cyberos-skill-runtime`) provides uniform `brain.* / audit.* / invariants.*` semantics regardless of host. (DEC-091 carrier; full plan in registry README Part 9)
- **§5.9 (decision log)** 4 new locked decisions:
  - **DEC-090** Skills↔contracts split — promote schemas out of skill folders into `cyberos/docs/contracts/` with `depends_on_contracts:` as the explicit dependency declaration.
  - **DEC-091** Dual-mode invocation + exposability — every workflow skill declares `invocation_modes: [standalone, chained]` and `exposable_as: {internal, agent_plugin, mcp_tool, partner_connector}`.
  - **DEC-092** Self-audit + auto-refinement — every Tier-2 skill carries `INVARIANTS.md`; breaches emit `refinement_proposal` envelopes; the supervisor pauses + surfaces for human review; new audit row kind `self_refinement_proposal`.
  - **DEC-093** Manual fine-tune playbook — 7-step structured cycle declared via `human_fine_tune:` frontmatter block with review-required gates, signals to initiate, required artefacts, and blackout windows.

### Real-world trigger

User-driven design conversation (2026-05-06) on whether CyberOS skills could (a) work both standalone and as pipeline atoms, (b) ship as plugins / connectors / MCP tools, (c) self-audit and propose refinements at runtime, (d) be manually fine-tunable by humans, and (e) port host-agnostically to Antigravity / Codex / Cursor without rewriting. Three §0.4 refinement candidates surfaced (REF-012, REF-013, REF-014 in the BRAIN) and were adopted as the four DECs above.

### Numbering note

PRD's DEC-087 = "Fenced-code-block exemption" (created 2026-05-04 evening) and DEC-088 = "Datetime-instance acceptance" (created 2026-05-04 evening). The BRAIN's `memories/decisions/DEC-087-...` slot is filled by an earlier "skill-registry-layout" decision (BRAIN created 2026-05-05). DEC-089 is unused in both. The v0.2.0 work starts at DEC-090 to avoid colliding with either namespace. Future reconciliation candidate: align the BRAIN's DEC-087 with PRD's by either renumbering the BRAIN entry or adding an alias.

---

## 2026-05-04 (evening, follow-up) — §5.10.11/12 validator discipline + DEC-087/DEC-088

### Added
- **§5.10.11** new sub-section "Fenced-code-block exemption in §4.3 multi-frontmatter check (AGENTS.md §4.3)" — narrative summary of the §4.3 amendment.
- **§5.10.12** new sub-section "Datetime-instance acceptance in §5.2 timestamp validator (AGENTS.md §5.2)" — narrative summary of the §5.2 amendment.
- **§5.9 (decision log)** 2 new locked decisions:
  - **DEC-087** Fenced-code-block exemption in §4.3 multi-frontmatter check (AGENTS.md §4.3).
  - **DEC-088** Datetime-instance acceptance in §5.2 timestamp validator (AGENTS.md §5.2).

### Real-world trigger
Surfaced during the workbench/.cyberos-memory bootstrap session (2026-05-04 evening) ingesting the agentskills + skills + claude-cookbooks/skills repos into a 12-file skills-knowledge module digest. Both failures hit on the very first memory file write: §4.3 rejected `spec.md` because the body legitimately contained `---`-delimited example SKILL.md frontmatter inside ```` ``` ```` fences; §5.2 rejected its own valid output because PyYAML auto-coerced ISO-8601 timestamps into `datetime.datetime`, and `str(dt)` rendered with a space separator that failed the validator's regex. Both proposed as TIER-1 refinements per §0.4 in the same response and adopted. The full reference-implementation patches landed in the session's local `.brain_writer.py`; SRS §5.12.8 captures the implementation specification.

## 2026-05-04 — §5.10 Ingestion-side discipline + DEC-076..DEC-085

### Added
- **§5.10** new section "Ingestion-side discipline + standing rule on refinements" with 10 sub-sections (§5.10.1 through §5.10.10) summarising each AGENTS.md amendment.
- **§5.9 (decision log)** 10 new locked decisions:
  - **DEC-076** Standing rule: protocol refinement on every memory issue (AGENTS.md §0.4).
  - **DEC-077** Verify-before-respond on user completeness challenge (AGENTS.md §1.10).
  - **DEC-078** Ingestion completeness for multi-section sources (AGENTS.md §4.10).
  - **DEC-079** Token-budget transparency on >500-line sources (AGENTS.md §4.11).
  - **DEC-080** Source freshness tier as conflict-resolution Step 0 (AGENTS.md §5.1, §6, §9.1).
  - **DEC-081** Source-coverage validator as Auto-Dream Phase 6 (AGENTS.md §8.6).
  - **DEC-083** Audit row `correction_to` field (AGENTS.md §7.1).
  - **DEC-084** Drift and refinement first-class memory buckets (AGENTS.md §3, §10).
  - **DEC-085** End-of-response coverage stat mandatory on ingestion ops (AGENTS.md §14).

### Real-world trigger
Same as `CyberOS-AGENTS.CHANGELOG.md` — corrective Miguel-DM re-ingestion. PRD changes summarise the AGENTS.md amendments at product/decision level; SRS captures the implementation specification.

## 2026-05-04 (afternoon revisions)

### Removed
- **§5.10.7** Sharpened credential denylist — never store AND never use. Reverted same-day: rule is already covered by host-platform safety ("Never authorize password-based access on the user's behalf") + the original §9.3 storage rule. Adding it as a separate §9.3 bullet duplicated higher-precedence rules.
- **DEC-082** entry from §5.9. Tombstoned in BRAIN with reason "rule subsumed by host-platform safety + original §9.3 storage rule."

### Changed
- **DEC-072 (Bootstrap state classifier)** — `INCOMPATIBLE:<schema_version>` replaced with `INCOMPATIBLE:<unknown-manifest-field>` (field-presence tripwire). The discrete-version-number model is incompatible with day-by-day protocol evolution; field-presence detection achieves the same forward-compat protection without the noise. Reference: CyberOS-AGENTS.md §13.0 + DEC-086.
- **§5.3.1** forward-compat sentence updated to use field-presence detection rather than `manifest.schema_version`.

## 2026-05-04 (afternoon revisions, follow-up)

### Changed
- **source_tiers description** — stripped Styx-specific example patterns (whatsapp-*-dm / notion-*); replaced with generic schema language clarifying the field is universal protocol but values are per-project. Each project's manifest.json carries its own patterns matching its actual scope graph.
