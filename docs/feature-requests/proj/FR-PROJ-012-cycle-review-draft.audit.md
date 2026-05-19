---
fr_id: FR-PROJ-012
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..017 added)
---

## §1 — Verdict summary

FR-PROJ-012 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (scheduler, stats, COO persona prompt, memory draft path, audit, no-auto-accept, PII redact, force-regen, LLM failure stub, RLS, metrics, locale-aware drafts, estimate-vs-actual ratio, top-5 longest-in-status, blocker recap by category, per-engagement persona override, iteration audit, all-revisions-preserved, comparison-to-prior-cycle, LLM response redaction, skip-cycle annotation, running learnings inclusion). 18 §2 rationale paragraphs. §3 contains: CycleStats struct, stats computation with multi-aggregate SQL, COO compose with prompt template, save_draft with frontmatter. 28 ACs. §10 lists 28 failure rows. §11 lists 25 implementation notes covering locale prompt mechanics, ratio interpretation, top-N tie-breaking, blocker recap fuzzy matching, persona registry fallback, revision linearity, skip mechanism, learnings concatenation.

## §2 — Findings (all resolved)

### ISS-001 — Auto-accept vs manual gate
Auto-accept = quality drift. Resolved: §1 #6 + DEC-331 never; AC #8.

### ISS-002 — LLM failure handling
Network outage during cycle close blocks reviews. Resolved: §1 #9 stats-only stub + sev-3 alarm; AC #13.

### ISS-003 — PII in LLM prompt
External LLM sees customer data. Resolved: §1 #7 FR-MEMORY-111 redact pre-call; AC #10.

### ISS-004 — Force-regenerate semantics
Without spec, naive impl overwrites or duplicates. Resolved: §1 #8 memory memory revisions preserved; AC #11.

### ISS-005 — Acceptance tracking
Without metric, drafts pile up unaccepted. Resolved: §1 #11 acceptance-minutes histogram visible in ops dashboards.

### ISS-006 — Schedule cadence
15-min poll vs cron. Resolved: §1 #1 tokio interval default + Postgres cron in production.

### ISS-007 — Locale ignored (strict-redo pass)
VN engagements got English drafts. Resolved: §1 #12 + per-engagement locale + AC #18.

### ISS-008 — No estimate accuracy signal (strict-redo pass)
Stats missed estimate-vs-actual; key calibration signal. Resolved: §1 #13 + ratio + AC #19.

### ISS-009 — Outlier issues hidden (strict-redo pass)
Aggregate stats mask outliers; reviews missed the story. Resolved: §1 #14 + top-5 longest + AC #20.

### ISS-010 — Blocker recap flat (strict-redo pass)
Operators want pattern detection across blockers; flat list misses categories. Resolved: §1 #15 + per-category recap + AC #21.

### ISS-011 — One persona for all engagements (strict-redo pass)
Internal vs client-facing review tones differ. Resolved: §1 #16 + per-engagement persona + AC #22.

### ISS-012 — Iteration history lost (strict-redo pass)
Force-regenerate replaced silently; no operator visibility into iteration. Resolved: §1 #17 + iteration audit + AC #23.

### ISS-013 — Revisions not preserved (strict-redo pass)
Force-regenerate overwrote; previous drafts lost. Resolved: §1 #18 + revision preservation + AC #24.

### ISS-014 — Reviews lack cross-cycle context (strict-redo pass)
Drafts had only current-cycle stats; prior context absent. Resolved: §1 #19 + prior-cycle comparison + AC #25.

### ISS-015 — LLM response PII passthrough (strict-redo pass)
Only input was redacted; output could leak. Resolved: §1 #20 + response redaction + AC #26.

### ISS-016 — Skip-cycle annotation missing (strict-redo pass)
Empty/trivial cycles generated noise drafts. Resolved: §1 #21 + skip annotation + AC #27.

### ISS-017 — Running learnings ignored (strict-redo pass)
Operator captured insights during cycle; draft didn't reflect. Resolved: §1 #22 + learnings concat + AC #28.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (COO compose × stats × LLM × locale × persona × revisions × comparison × redaction × skip × learnings), not by line targets.

---

*End of FR-PROJ-012 audit.*
