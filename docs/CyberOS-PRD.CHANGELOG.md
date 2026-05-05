# Changelog — CyberOS-PRD.docx

All notable changes to **CyberOS-PRD.docx** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

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
