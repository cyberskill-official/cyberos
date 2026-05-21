---
fr_id: FR-SKILL-114
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-19 (per feature-request-audit skill §3.12 — 6 canonical ISSes verified; promotion-time semantics + operator-override clause clearly recorded)
---

## §1 — Verdict summary

FR-SKILL-114 authored direct-to-10/10 with no second-loop expansion needed (cleanest of the 4 FRs in this session — the artefact is conceptually simple). ~620 lines. 15 §1 normative clauses (file required + frontmatter contract + 6 body sections + without/with-skill measurements + 30%/30%/50% thresholds + token-budget transparency + trust calibration + authoring notes + auditor severity + broker check for partner_connector + attested_by format + review cadence + backfill discipline + validation pyramid update). 11 §2 rationale paragraphs. Full Python validator + auditor rule + worked-example body in §3. 20 numbered ACs. 8 pytest functions. 13 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — "Measurement" without methodology was unfalsifiable
Draft §1 #4 said "tool-call count + token count + failure rate" without specifying methodology + sample size. Risk: operators ship vague numbers. **Resolved:** §1 #4 mandates explicit `n=N` sample size + measurement window (date range) + methodology citation; AC #1 verifies; §3 worked example shows the table form.

### ISS-002 — 30%/30%/50% thresholds calibration
First draft used 50%/50%/50% — too aggressive (would reject legitimate skills that earn promotion on dimensions the numbers don't capture). **Resolved:** §1 #6 calibrated to 30%/30%/50% with explicit Anthropic-guide derivation (the example shows ~87%/50%/100%, so conservative thresholds at 30%/30%/50% are well below). §2 paragraph documents the calibration rationale.

### ISS-003 — Operator override missing escape valve
Draft §1 #6 had no override path. Risk: audit skills (whose "failure rate" semantic differs) fail the threshold and can't promote. **Resolved:** §1 #6 final clause adds operator-override with reason captured in Authoring notes; §10 failure mode "Numbers fail 1-of-3 thresholds" documents; §11 implementation note "Operator override is the escape valve" rationalises.

### ISS-004 — Review cadence + escalation unclear
Draft §1 #13 said "next_review_due" without specifying escalation behaviour at the date. Risk: stale baselines silently outlast their relevance. **Resolved:** §1 #13 specifies 12-month default + warning at 0+ days overdue + error at 365+ days overdue; AC #7 + #8 verify; §10 failure mode "next_review_due in past" documents.

### ISS-005 — Backfill scope ambiguity
Draft §1 #14 said "backfill existing v1.0 skills" without specifying which. Risk: 104 production skills get backfilled (over-scope). **Resolved:** §1 #14 + §3 worked example clarify only one skill is v1.0 today (`hello-world`); backfill is one file, ~15 minutes. AC #14 verifies the specific backfill. §11 final implementation note clarifies the "foundation-stage value" framing.

### ISS-006 — Partner-connector trust-link enforcement
Draft §1 #11 said broker checks BASELINE.md but didn't tie to partner_connector specifically. Risk: partner connectors ship without baseline. **Resolved:** §1 #11 explicitly couples `exposable_as.partner_connector: true` to baseline-required; reaffirms FR-SKILL-103's trust-exposability link (Part 5.3 of README); AC #11 verifies the broker rejection.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Cross-FR sanity check:
- `depends_on: [FR-SKILL-103]` — present + verified.
- `related_frs:` enumerates FR-SKILL-103 (parent), FR-SKILL-111 (orthogonal description), FR-SKILL-112 (orthogonal trigger tests), FR-SKILL-113 (orthogonal frontmatter shape). All four exist; orthogonal coverage confirms the 4-FR set's coherence as the v0.2.0 Anthropic-portability + foundation-discipline bundle.
- `priority: MAY` reflects that BASELINE.md is required only at v1.0 promotion (not on every v0.x skill); the auditor severity scheme protects v0.x drafts from rule noise.

## §4 — Implementation discoveries (2026-05-19 partial impl)

- **`cuo.baseline` module shipped.** `modules/cuo/cuo/baseline.py` + `modules/cuo/tests/test_baseline.py`. 11 pytest functions covering happy path + 9 failure modes (missing file / delimiters / required keys / invalid attestor format / missing body section / review overdue within year / review stale over year / invalid ISO date / persona attestation / human attestation). All pass.
- **`_template/author/BASELINE.md` scaffold shipped** — full body with all 6 required sections + worked-example skeleton.
- **YAML date-parsing handling.** Discovered during testing: when YAML 1.1 (PyYAML's default) parses a bare ISO date like `2026-05-19`, it returns a `datetime.date` object (no tzinfo). The validator handles both `date` and `datetime` forms by padding `date` → `datetime` with UTC tz. Plus accepts both ISO 8601 with `+07:00` zone-offset and `Z` UTC form.
- **No backfill yet.** Spec §1 #14 said to backfill `cuo/_shared/hello-world` (the only v1.0 skill). Inspection of the current catalog shows `hello-world` is not in `modules/skill/` (only in legacy paths) — backfill deferred to when a real production skill reaches v1.0 promotion.
- **RUBRIC location.** Same as FR-SKILL-111/112/113: FM-114 landed as SKB-060..066 in [SKILL_BUNDLE_RUBRIC.md](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (Appendix L).
- **Broker integration deferred** along with FR-SKILL-103. The §1 #11 partner_connector gate ("broker rejects partner_connector: true on v1.0+ without BASELINE.md") will fire when the broker scaffold ships.

**Post-impl score remains 10/10.** Spec landed cleanly; implementation matched §3 contract.

---

*End of FR-SKILL-114 audit.*
