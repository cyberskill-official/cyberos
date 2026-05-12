# Changelog — CyberOS-PRD.docx

All notable changes to **CyberOS-PRD.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

---

## 2026-05-12 — Layer-1 catalog 100 % shipped + doc consolidation (no PRD §-level changes)

### Summary

Batches 4–10 of the Layer-1 operator surface landed today. Total: 33 `cyberos` subcommands (up from 18 at session start), 16 new runtime tools, 3 pluggable validators, 24 mutation tests (0 SURVIVED), read-only MCP server, multi-machine sync scaffolding, council mode for ambiguous REFs, persona-defined defaults, cold-storage tier, advisory-lock module, streaming session-start loader (74.9× speedup measured). **100 % of `workbench/cyberos-layer1-deep-improvements.md` is closed.**

Documentation consolidated: the standalone `CyberOS-LAYER-1-MANUAL.md` was merged into `CyberOS-AGENTS.README.md` as Parts 25–31 (architecture overview, per-aspect detail, CLI reference, workflows, troubleshooting, deferred items, file map). The README is now the single home for mental model + operator manual.

### PRD-side impact (Batches 4–10)

**Zero PRD §-level changes.** Every new tool implements existing PRD commitments:
- PRD §5.3.1 "Concept and storage layout" — operator surface now has 33 subcommands; README Parts 25–27 document each
- PRD §5.3.2 "Six file operations" — `cyberos_lock.py` (Aspect 5.7) adds `.lock.shared` / `.lock.exclusive` coordination helpers
- PRD §5.3.3 "Sync classes" — `cyberos_sync.py` (Aspect 6.x) ships deterministic export + 3-way merge import + conflict resolver UX (Aspect 6.5)
- PRD §5.3.5 "Auto Dream consolidation" — `cyberos refinements` (Aspect 11.4) surfaces drift + council-pending + rejected items
- PRD §6.3 "CUO routing logic" — `cyberos skill` registry (Aspect 12.5) stages the chain-of-skills metadata that CUO P0+ will consume
- PRD §8 "MCP Gateway" — Aspect 12.7 ships a read-only MCP server today (`brain_search`, `brain_show`, `brain_get`, `brain_stats`) ahead of the full MCP Gateway

### Cross-reference updates pending in PRD.docx (next .docx editing session)

- §5.3.1 — point at **README Parts 25–27** (not the deprecated standalone manual)
- §5.3.2 — link `runtime/tools/cyberos_lock.py` as the §4.4 atomic-write advisory-lock helper
- §5.3.3 — link `runtime/tools/cyberos_sync.py` as the §17 sync-class enforcement implementation
- §5.3.5 — link `cyberos_refinements.py` + `cyberos_prune.py` (Aspects 1.1 + 9.7) as Auto-Dream operator-side companions
- §5.6 — `cyberos sync conflicts --resolve` is the interactive resolver UX (Aspect 6.5 — shipped, was previously deferred)
- §6.3 — `cyberos skill chain` is the static-analysis precursor; the live CUO router builds on this
- §8 — MCP Gateway pre-P0 status: read-only server shipped; write-enabled gateway remains P0+

### Verification (post-Batch 10)

- `cyberos verify` → CRITICAL: 0 / WARN: 11 / INFO: 1
- `cyberos mutation-test` → 24 mutations × 0 SURVIVED
- Audit chain intact; chain head `sha256:b30dc197b713f168…`
- 2 items remain blocked-with-rationale: 10.3 differential testing (blocked — only one impl), 13.8 repo split (deferred — architectural)

---

## 2026-05-12 — Layer-1 operator surface lands (no PRD §-level changes) [earlier batches]

### Summary

A batch of Layer-1 improvements landed (Aspects 1.1, 2.1, 3.1, 3.4, 3.5, 4.1, 4.3-4.6, 5.1, 5.5, 7.2, 7.3, 7.4, 8.1, 11.1, 11.2, 13.4, 13.10) covering operator UX, refinement-detection hooks, memory templates, voice + consistency CI, onboard wizard, local analytics, tour files, and emergency-stop. **None of this changes the PRD §5.3 Layer-1 architecture** — all additions sit alongside existing rules.

### PRD-side impact

**Zero PRD §-level changes.** The new tooling implements existing PRD commitments:
- PRD §5.3.1 "Concept and storage layout" — gets a `cyberos` operator surface (was implicit; now explicit)
- PRD §5.3.5 "Auto Dream nightly consolidation" — gets companion Stop-hook for §0.4 refinement-candidate detection
- PRD §1.4 Operating Principle 6 "Universal memory, three layers" — Layer 1 now has CI gates (voice + consistency + validator) per `dashboard-builder` pattern
- PRD §6.3 "CUO routing logic" — gateguard PreToolUse hook is a CUO precursor (denies first-attempt writes, forces investigation)

### Cross-reference updates needed in PRD.docx (next editing session)

- §5.3.1 — add reference to `runtime/tools/cyberos` umbrella binary
- §5.3.2 — link to `runtime/hooks/gateguard.py` as enforcement layer for the six file operations
- §5.3.5 — reference `runtime/hooks/refinement_candidates.py` as Auto-Dream Phase-6 companion
- §5.8 — link to `runtime/tests/denylist/test_denylist.py` as denylist regression coverage
- §6.3 — note that gateguard is the local-edge precursor to the eventual P0+ CUO routing
- New §13 entry: list `cyberos` operator surface + 5 tour files

### No new DEC entries

This bundle is tooling + scaffolding, not decisions. Operator UX improvements are below the §0.4-refinement threshold (no protocol mechanism added).

### Driver

User asked for full implementation of Layer-1 improvements catalog. All non-§0.5-requiring work landed in one batch. See `CyberOS-AGENTS.CHANGELOG.md` 2026-05-12 entry for the canonical record.

---

## 2026-05-10 — Bundle M absorbed (functional-zero refinement pass; no DEC entry)

### Not yet applied to CyberOS-PRD.docx

Bundle M is a functional-zero refinement of AGENTS.md (Changes A–D applied; E + F deferred to Bundle N). No new ops, no schema changes, no PRD-level surface change. The PRD-side work is one cross-reference update: §5.10 references to §4.11 should change to §4.10.2 at the next .docx editing session.

No new DEC entry — Bundle M is documentation cleanup, not a decision.

### Real-world trigger

Same as `CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 Bundle M entry).

---

## 2026-05-10 — Stage 5 protocol upgrade absorbed (DEC-108 pending; .docx update deferred)

### Not yet applied to CyberOS-PRD.docx

The Stage 5 protocol upgrade landed six additive amendments to AGENTS.md (§5.6 at-rest encryption envelope, §6 encryption_policy + shamir_fragments fields, §7.1 +8 new ops, §4.6 encrypted-tombstone semantics, §9.3 denylist clarification, §17.6 cross-link refresh). Full text in `docs/CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 Stage 5 entry) and `docs/proposals/STAGE-5-PROTOCOL-UPGRADE.md`.

The PRD-level surface is **§5.8 BRAIN data classification** + **§9.6 Security NFR (SEC)** in Part 11 NFRs. A new sub-section §5.8.1 (`At-rest encryption envelope`) documenting the XChaCha20-Poly1305-IETF + Shamir 3-of-5 escrow design will land in the PRD .docx at the next .docx editing session, alongside Part 13 entry **DEC-108**.

### Pending DEC entry

- **DEC-108** Stage 5: At-rest encryption + Shamir 3-of-5 escrow. Status: Adopted (AGENTS.md §0.5 upgraded to `sha256:d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0`); .docx record pending. Deciders: Founder. Reference: AGENTS.md §5.6, §6, §7.1, §4.6, §9.3, §17.6 + `docs/proposals/STAGE-5-PROTOCOL-UPGRADE.md` + `docs/proposals/STAGE-5-OPEN-QUESTIONS.md` (decisions baseline: Q1=c, Q2=c, Q3=3-of-5 wizard, Q4=a body-only, Q5=c user-paced).

### Real-world trigger

Same as `CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 Stage 5 entry).

---

## 2026-05-10 — Stage 6 protocol upgrade absorbed (DEC-107 pending; .docx update deferred)

### Not yet applied to CyberOS-PRD.docx

The Stage 6 protocol upgrade landed five additive amendments to AGENTS.md (§4.9.1 `.lock.shared`, §7.6 Merkle checkpoints, §7.7 ledger compaction, §8.7 phase 4 Merkle verification, §8.9 ledger compaction phase). Full text in `docs/CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 Stage 6 entry) and `docs/proposals/STAGE-6-PROTOCOL-UPGRADE.md`.

The PRD-level surface is **§5.3.5 Auto Dream consolidation** + **§5.10 Ingestion-side discipline**. A new sub-section §5.3.6 (`Merkle checkpoints + ledger compaction`) and Part 13 entry **DEC-107** will land in the PRD .docx at the next .docx editing session.

### Pending DEC entry

- **DEC-107** Stage 6: Merkle checkpoints + ledger compaction + .lock.shared. Status: Adopted (AGENTS.md §0.5 upgraded to `sha256:77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa`); .docx record pending. Deciders: Founder. Reference: AGENTS.md §4.9.1, §7.6, §7.7, §8.7, §8.9 + `docs/proposals/STAGE-6-PROTOCOL-UPGRADE.md`.

### Real-world trigger

Same as `CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 Stage 6 entry).

---

## 2026-05-10 — Stage 1 protocol upgrade absorbed (DEC-106 pending; .docx update deferred)

### Not yet applied to CyberOS-PRD.docx

The Stage 1 protocol upgrade landed five additive amendments to AGENTS.md (§5.1 frontmatter compactness, §6 reconciliation_checkpoint + read_profile, §4.7 incremental reconciliation, §8.7 phase 4 stale-checkpoint check). Full text in `docs/CyberOS-AGENTS.CHANGELOG.md` (2026-05-10 entry) and `docs/proposals/STAGE-1-PROTOCOL-UPGRADE.md`.

The PRD-level surface for this upgrade is **§5.10 Ingestion-side discipline** — adjacent territory to DEC-076..088. A new sub-section §5.10.13 (`Reconciliation checkpoint + lazy-load profile + frontmatter compactness`) and Part 13 entry **DEC-106** will land in the PRD .docx at the next .docx editing session, programmatically via python-docx in the same pattern as prior entries (DEC-087/DEC-088 evening of 2026-05-04).

### Why this is recorded here at all

Same reason as every PRD CHANGELOG entry: traceability. A future reader auditing "what did the PRD say about session-start performance?" should see at a glance that the protocol upgrade landed 2026-05-10, that DEC-106 is pending, and that the canonical contract record is in `CyberOS-AGENTS.CHANGELOG.md`.

### Real-world trigger

User-driven local-optimization design (2026-05-09 evening) — Stephen explicitly scoped the work to "perform optimal with local files (.cyberos-memory folder)" given CyberOS-the-product is still pre-build. The local-optimization plan (`docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`) Stage 1 was approved via §0.5 chat-turn phrase. PRD §5.3 (BRAIN Layer 1) gains a footnote at next .docx update referencing the new §6 manifest fields and §4.7 incremental reconciliation.

### Pending DEC entry

- **DEC-106** Stage 1: Reconciliation checkpoint + lazy-load profile + frontmatter compactness. Status: Adopted (AGENTS.md §0.5 upgraded to `sha256:576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a`); .docx record pending. Deciders: Founder. Reference: AGENTS.md §4.7, §5.1, §6, §8.7 + `docs/proposals/STAGE-1-PROTOCOL-UPGRADE.md`.

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
