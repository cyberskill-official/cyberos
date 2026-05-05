# Changelog — CyberOS-SRS.docx

All notable changes to **CyberOS-SRS.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

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
