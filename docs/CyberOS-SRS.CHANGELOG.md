# Changelog — CyberOS-SRS.docx

All notable changes to **CyberOS-SRS.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

---

## 2026-05-06 — Registry v0.2.4 absorbed (chain entry point; MINOR within scope of §6.13/6.14/6.15/6.16)

### No .docx edits this round

v0.2.4 added 2 skills + 2 artefact_schema contracts upstream of fr-author: `cuo/cpo/requirements-discovery` + `cuo/cpo/prd-author`, consuming/producing `project_brief@1` + `prd@1`. The runtime mechanisms in §6.13–§6.16 are all reusable (skills↔contracts split, dual-mode invocation, self-audit, manual fine-tune, host-adapter pipeline) — no new SRS section needed.

The contracts layout simplified from `<contract-id>/v<n>/` to flat `<contract-id>/`; the SRS's contract-kind taxonomy (artefact_schema | envelope_schema | wire_protocol) is unchanged. Per REF-018 in BRAIN, the simplification was deferred-complexity-recovery, not contract-semantics change.

cpo + cto persona-cards bumped (cpo 0.2.0→0.3.0, cto 0.1.0→0.2.0) for scope-ceiling expansion. SRS §6.4 scope-contract semantics are unchanged; the persona-cards now declare a wider read-ceiling that the new workflows (requirements-discovery, prd-author, future srs-author/srs-audit) consume.

---

## 2026-05-06 — Registry v0.2.2 absorbed (Tier-2/3 follow-up; PATCH within scope of §6.13/6.14/6.15/6.16)

### No .docx edits this round

The registry-level changes recorded under `cyberos/docs/skills/CHANGELOG.md` v0.2.2 — including the new `cyberos/docs/contracts/nats-subjects/` wire-protocol contract, the per-skill divergence note in the two `references/README.md` files, the confidence-band documentation in `RUBRIC.md` §15.9, the deterministic-input rule in `AUDIT_LOOP.md`, and the INV-006 severity demotion — are PATCH-level operationalisation of existing SRS sections §6.13 (skills↔contracts split), §6.14 (dual-mode + exposability), §6.15 (self-audit + auto-refinement), §6.16 (manual fine-tune). They do NOT alter the four locked decisions DEC-090..093 and do NOT introduce new runtime mechanisms.

### Why this is recorded here at all

Same reason as the PRD: traceability. A future reader auditing "what does the SRS currently say about runtime mechanisms?" should see v0.2.2 happened, that it was Tier-2/3 absorption, and that no SRS body content moved.

### What absorbed (mapping to existing SRS sections)

- **B1** (per-skill reference doc divergence) → §6.13 still describes contracts as the unification mechanism for byte-identical schemas; the four reference docs documented as intentionally divergent (per skill's lifecycle phase) under `references/README.md` files.
- **B2** (NATS subjects undocumented) → §6.13 (skills↔contracts) and §6.14 (`depends_on_contracts:`) acquire a concrete second consumer beyond `feature-request@1` — `nats_subjects@1` (wire_protocol kind). The SRS's contract-kind taxonomy (`artefact_schema | envelope_schema | wire_protocol`) was already complete; v0.2.2 fills the wire_protocol slot for the first time.
- **B3** (per-rule confidence bands) → §6.7 audit row schema's `confidence` field gains documented per-rule expectations in `RUBRIC.md`. SRS-level confidence-band semantics (LLM ≤ 0.7 cap) unchanged.
- **B4** (INV-006 redundancy) → §6.15 self-audit invariants still mandated; one specific invariant demoted to `info` because schema validation already covers it. Pattern (demote when schema covers) is implicit in the existing self-audit guidance.
- **C3** (deterministic-input rule) → §6.15's invariants get sharper definitions in their target documents. `INV-001`'s anchor target (`AUDIT_LOOP.md` §"Deterministic-input rule") now resolves cleanly.

---

## 2026-05-06 — §6.13/6.14/6.15/6.16 + Part 13 DEC-090..093 (skill registry v0.2.0)

### Applied to CyberOS-SRS.docx (programmatically via python-docx, 2026-05-06 evening)

The following sub-sections have landed in the .docx (28 paragraphs inserted before Part 7 covering §6.13/6.14/6.15/6.16; 4 DEC entries inserted before Part 14 in the §13.3 decision log). All inserted content matches existing Heading 2 / Heading 3 / default styles by deep-copying template paragraphs and replacing run text. The skill registry's `cyberos/docs/skills/` and `cyberos/docs/contracts/` directories carry the canonical artefact state; SRS sections below document the runtime mechanisms they imply.

- **§6.2 Anthropic Skills format** — frontmatter contract grows from 27 to 33 fields. New blocks: `invocation_modes`, `expects.optional_fields` + `expects.standalone_interview_ref`, `produces.human_summary_ref`, `depends_on_contracts`, `exposable_as`, `self_audit`, `human_fine_tune`. Validator must accept both v0.1.x (legacy) and v0.2.0 SKILL.md files; v0.2.0 self-test checklist is the gate, not a hard rejection of v0.1.x.
- **§6.4 Scope contract** — extended to recognise the `_contracts/` namespace. Skills' `depends_on_contracts:` entries are validated against actual contract paths at build time.
- **§6.7 Audit ledger (`genie.action_log`)** — `op` enum extended with `self_refinement_proposal`. `row_kind` enum extended to match. `produces.output_kind` enum extended with `refinement_proposal`. Hash-chain semantics unchanged.
- **§6.13** new section "Self-audit invariants engine" with five sub-sections:
  - **§6.13.1** `INVARIANTS.md` schema: ID + Statement + Check (deterministic test, often SQL against `genie.action_log` or a Python predicate against manifest state) + Severity + Refinement template.
  - **§6.13.2** Runtime: invariants engine runs at declared `self_audit.check_at` checkpoints (`on_node_boundary`, `on_audit_row_count: N`, `on_completion`). Implementation provided by the host shim library (§6.16).
  - **§6.13.3** Anomaly signals: `confidence_low_streak`, `user_correction_streak`, `denylist_near_miss_streak`, `scope_rejection_streak`, plus skill-specific signals (e.g. fr-audit's `deterministic_drift` is sev-0).
  - **§6.13.4** Breach handling: emit `refinement_proposal` envelope; LangGraph supervisor checkpoints state, pauses pipeline, surfaces Question primitive for human review; APPROVE / REVISE / REJECT routes.
  - **§6.13.5** Auto-refinement → manual-fine-tune escalation: when `self_audit_refinement_proposal_count_above` is exceeded (default 2 proposals on the same theme within one batch), the runtime hands off to the manual fine-tune flow (§6.14).
- **§6.14** new section "Manual fine-tune playbook" — the 7-step cycle (pause → diagnose → regression → edit → re-run → bump → resume)
  + the `human_fine_tune` frontmatter block + review-required gates + blackout-window enforcement.
- **§6.15** new section "Skills↔contracts split + `depends_on_contracts` resolver" — `cyberos/docs/contracts/<id>/v<n>/CONTRACT.md` schema; resolver walks declared dependencies on every build; CI matrix blocks merge if any consumer references a contract not declared.
- **§6.16** new section "Host-adapter pipeline" with five sub-sections:
  - **§6.16.1** Phase A — CCSM (the SKILL.md) is source of truth; `dist/<host>/` is generated.
  - **§6.16.2** Phase B — transpilers (one per output target): `ccsm-to-anthropic-skill`, `ccsm-to-mcp-tool`, `ccsm-to-claude-plugin`, `ccsm-to-antigravity`, `ccsm-to-codex`, `ccsm-to-cursor`. Pure functions `CCSM → host-artefact-tree`.
  - **§6.16.3** Phase C — host shim library (`cyberos-skill-runtime` Python + `@cyberos/skill-runtime` Node) providing uniform `runtime.brain` / `runtime.audit` / `runtime.invariants` / `runtime.envelope` / `runtime.untrusted` semantics. Falls back to filesystem-local BRAIN + JSONL audit log when CyberOS MCP servers are unreachable (degraded but functional).
  - **§6.16.4** Phase D — equivalence test matrix: every skill ships `acceptance/` golden fixtures; CI runs each fixture across every transpiled target and asserts behavioural equivalence (modulo declared host-specific fields).
  - **§6.16.5** Phase E — partner connector pipeline (gated; needs per-skill DEC for `partner_connector: true`).
- **Part 13 decisions log:** 4 new entries DEC-090, DEC-091, DEC-092, DEC-093 with implementation cross-refs (full text in PRD §5.11.1–§5.11.5).

### Reference implementations

- `INVARIANTS.md` worked examples in `cyberos/docs/skills/cuo/cpo/fr-author/INVARIANTS.md` (8 invariants including INV-003 ingestion-coverage that mirrors AGENTS.md §4.10 at the skill level) and `cyberos/docs/skills/cuo/cpo/fr-audit/INVARIANTS.md` (8 invariants including INV-001 verdict-determinism, the auditor's highest-value contract).
- `STANDALONE_INTERVIEW.md` and `HUMAN_SUMMARY.md` worked examples in both fr-author/ and fr-audit/.
- The promoted contract at `cyberos/docs/contracts/feature-request/CONTRACT.md` is the canonical example of the smaller, contract-only frontmatter (drops `allowed_mcp_tools`, `expects/produces`, `audit`, `confidence_band`, `untrusted_inputs`, `gated_until_phase`; adds `contract_id`, `contract_version`, `contract_kind`, `template_literal`, `steward_persona`, `escalation_on_breach`, `moved_from`).

### Performance impact analysis (estimates pending real measurement)

- Invariants engine: O(N) per checkpoint where N = number of invariants × cost-per-check. For fr-author's 8 invariants, ~30ms per node boundary on a typical batch.
- Transpiler: one-shot at build time; ~50ms per target per skill; embarrassingly parallel.
- Shim library overhead: ~1-2ms per `runtime.*` call (FS-fallback mode); negligible (<0.1ms) when MCP servers are reachable.

### Real-world trigger

Same as `CyberOS-PRD.CHANGELOG.md` (2026-05-06). User-driven design conversation on host portability + plugin/connector exposure + self-audit + manual fine-tune. Three §0.4 refinement candidates adopted as four DECs (REF-012/013/014 in the BRAIN; DEC-090/091/092/093 in the PRD/SRS).

---

## 2026-05-04 (evening, follow-up) — §5.12.8 validator discipline implementation + DEC-087/DEC-088

### Added
- **§5.12.8** new sub-section "Validator discipline — fenced-code-block exemption + datetime-instance acceptance" with reference Python implementations:
  - `brain.frontmatter.split(text)` — pre-process body by stripping fenced spans (regex `(?ms)^(```|~~~).*?^\1\s*$`) before scanning for a secondary `\n---\n`. Opening-block check unchanged. Performance: O(n), ~0.5ms per 30 KB memory.
  - `brain.validators.timestamp(field, value)` — early-branch on `isinstance(value, datetime.datetime)` before any string coercion; reject naive (tzinfo-less) datetimes as `naive-ts:<field>`. Migration note: a naive port that adds the datetime branch without early-returning still hits the original bug because `str(dt)` is computed downstream.
  - Test fixtures specified for both: ISO string accept, tz-aware datetime accept, naive datetime reject, PyYAML-parsed datetime accept (regression for the original failing case).
- **Part 13 decisions log:** 2 new entries DEC-087 and DEC-088 with implementation cross-refs (full text in PRD §5.10.11–§5.10.12).

### Real-world trigger
Same as `CyberOS-AGENTS.CHANGELOG.md` (evening, follow-up) and `CyberOS-PRD.CHANGELOG.md` (evening, follow-up) — workbench/.cyberos-memory bootstrap session, two TIER-1 validator amendments adopted.

## 2026-05-04 — §5.12 Ingestion-side discipline implementation + DEC-076..DEC-085

### Added
- **§5.12** new section "Ingestion-side discipline — implementation specification" with 7 sub-sections:
  - **§5.12.1** Frontmatter schema additions (`brain.memory_file` table: +`source_freshness_tier`, +`ingestion_coverage` JSONB, with `intentional_summary:true` + `summary_reason:"pre-rule ingestion; coverage retroactively unverified"` backfill so consolidation does not flag legacy memories as shallow).
  - **§5.12.2** Manifest `source_tiers` table + glob-resolution rules; `brain.tier.resolve(scope)` MCP tool.
  - **§5.12.3** Audit row `correction_to` column on `brain.memory_event` (foreign key to `audit_id`); retrieval surfaces correction chain in explanation pane (§6.8); default `recency_penalty` of 0.5× on corrected rows.
  - **§5.12.4** Source-coverage validator added to `brain.dream()` pipeline as Phase 6 (after manifest update).
  - **§5.12.5** Conflict-resolution Step 0 in `brain.conflict.resolve()` — `source_freshness_tier` gap ≥ 1 + neither side `personnel`/`client` ⇒ lower-tier wins; logged in `dream_journal`.
  - **§5.12.6** §14 end-of-response block contract integrated into CHAT module's reply-rendering pipeline; structured §14 block validated via JSON Schema before delivery.
  - **§5.12.7** Performance impact analysis (Phase 6: ~250ms per dream cycle for 1K memories @ 50KB avg; tier resolution: O(log K) per read; Step 0: O(1) ahead of existing tree).
- **Part 13 decisions log:** 10 new entries DEC-076 through DEC-085 (full text in PRD §5.10.1–§5.10.10).

### Real-world trigger
Same as `CyberOS-AGENTS.CHANGELOG.md` — corrective Miguel-DM re-ingestion.

## 2026-05-04 (afternoon revisions)

### Removed
- **DEC-082** entry from Part 13 Decisions Log. Reverted same-day: rule is already covered by host-platform safety + original §9.3 storage rule. Tombstoned in BRAIN.

### Changed
- **DEC-072 (Bootstrap state classifier)** in Part 13 — `INCOMPATIBLE:<schema_version>` replaced with `INCOMPATIBLE:<unknown-manifest-field>`. Field-presence tripwire replaces discrete-version-number model for compatibility with day-by-day protocol evolution. Reference: CyberOS-AGENTS.md §13.0 + DEC-086.

## 2026-05-04 (afternoon revisions, follow-up)

### Changed
- **source_tiers description** — stripped Styx-specific example patterns (whatsapp-*-dm / notion-*); replaced with generic schema language clarifying the field is universal protocol but values are per-project. Each project's manifest.json carries its own patterns matching its actual scope graph.
