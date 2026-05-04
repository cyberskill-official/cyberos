# Changelog — CyberOS-PRD.docx

All notable changes to **CyberOS-PRD.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

---

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
