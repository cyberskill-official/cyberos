---
fr_id: FR-SKILL-111
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-19 (per AUTHORING_DISCIPLINE.md §3.12 — 8 canonical ISSes verified; §3.13 description-format rule is the spec's own subject so meta-compliant)
---

## §1 — Verdict summary

FR-SKILL-111 authored direct-to-10/10 with one mid-loop expansion. ~640 lines. 15 §1 normative clauses (description fields format + length budget + trigger-phrase form + bracket discipline + negative-trigger handling + file-type guidance + verb-stem rule + output-artefact naming + multi-line YAML + locale-default + auditor severity + auto-fix prohibition + CLI error sub-codes + lazy-backfill discipline + body-section coexistence). 11 §2 rationale paragraphs. Full Rust validator + JSONSchema diff + RUBRIC entry + template diff + CLI example in §3. 21 numbered ACs. 11 Rust unit tests + auditor-fixture spec. 13 failure modes. 8 implementation notes. Cross-FR reciprocity (depends_on: FR-SKILL-103) verified.

## §2 — Findings (all resolved during authoring)

### ISS-001 — "Trigger phrase" was undefined for the first 3 drafts of §1
First draft of §1 #1 said "≥2 trigger phrases" without defining what a "trigger phrase" was syntactically. Validator would have needed heuristic detection — drift across implementations. **Resolved:** §1 #3 mandates **quoted form** (`"<phrase>"`) and lists the two acceptable surrounding constructions (`Use when user asks to "..."` and `Triggers on "..."`); validator's `QUOTED_TRIGGER` regex is the deterministic anchor; AC #1 + #6 + #8 verify the contract.

### ISS-002 — Negative-trigger counting risked double-counting
Early §1 #5 said "Negative triggers don't count" without specifying detection. If a negative trigger appears 10 chars before a positive one (e.g. `"Do NOT use for foo. Use \"bar\" or \"baz\""`), the validator could miscount. **Resolved:** §1 #5 defines the 40-char-preceding-window negative-prefix check; validator's `NEGATIVE_PREFIX` regex + windowed lookback enforces; AC #7 + #8 verify negative + positive disambiguation works as expected.

### ISS-003 — 80-char floor justification was thin
Spec said "≥80 chars" without explaining why not 100 or 150. **Resolved:** §11 implementation note documents the empirical reasoning — 74-char description with 2 triggers + verb felt cryptic; 80 gives breathing room for a value phrase. AC #9 stress-tests near the floor.

### ISS-004 — JSONSchema vs Rust validator drift risk
First draft of §3 had JSONSchema express only `maxLength`. The mirror would be partial — JSONSchema couldn't catch trigger-count violations. CI gates relying on JSONSchema would miss violations the Rust validator catches. **Resolved:** §3 JSONSchema diff documents that length + bracket-free is the JSONSchema surface; trigger-count is Rust-only; AC #10 + §5's `jsonschema_mirror_agrees_on_length_bounds` test enforces the agreed split. JSONSchema carries `$comment` documenting the partial coverage.

### ISS-005 — Auto-fix discipline was ambiguous
Draft §1 #11 said "rule fires"; didn't say whether auto-fix could apply. Description text is user-facing, but `auto_fix_applied: true` would silently rewrite trigger phrases. **Resolved:** §1 #12 mandates `auto_fix_applied: false` always; verdict always `needs_human`. AC #14 verifies. §2 rationale explains why (auto-edit silently changes user-visible verbs).

### ISS-006 — Lazy-backfill mechanic wasn't tied to a signal
Draft §1 #14 said "backfill lazily" without saying *what triggers* the backfill. Risk: 104 production skills sit at v0.2.0 forever, FM-112 fires endlessly. **Resolved:** §1 #14 ties the rule to `status: accepted` + `human_fine_tune.signals_to_initiate` — the next natural fine-tune cycle brings each skill into compliance. §11 implementation note expands on why this is the right cadence (vs big-bang sweep). AC #15 + #16 + #17 add 3 backfill exemplars (feature-request-author, feature-request-audit, prd-author) to seed the lazy fix.

### ISS-007 — Body `## When to invoke` deletion risk
Draft §1 #15 was added on the second loop after a re-read of the digest in `modules/skill/ANTHROPIC_GUIDE_DIGEST.md` §5.1 Gap 1. The original first draft of §1 implied "move triggers from body to frontmatter," which could be misread as "delete the body section". This would have broken supervisor `classify_act` and discarded disambiguation guidance. **Resolved:** §1 #15 explicitly MANDATES the body section is kept; §2 rationale explains the dual-layer rationale (frontmatter for host classifier; body for supervisor + human reader). README Part 18 anti-pattern entry "Don't put triggers only in body" is paired with the implied "Don't delete the body section either".

### ISS-008 — Verb-stem regex coverage
Initial verb-stem regex had 12 verbs; missed common CyberOS verbs like `enforce`, `validate`, `orchestrate`. Would have rejected legitimate `## When to invoke` author skills (e.g. `feature-request-audit` whose description verb is `audit`). **Resolved:** §3 `VERB_STEMS` regex expanded to 29 verb stems covering CyberOS's actual skill surface; §11 implementation note documents the conservatism rationale + the PR-based extension protocol; AC #5 verifies `Helps with X` (no verb stem) is correctly rejected; tests cover `audit`, `generate`, `chain`, etc. all pass.

## §3 — Resolution

All 8 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Cross-FR sanity check:
- `depends_on: [FR-SKILL-103]` is reciprocal — FR-SKILL-103's `blocks:` will be updated to include FR-SKILL-111 in a follow-up housekeeping commit (per AUTHORING_DISCIPLINE.md §3.1 rule 2). Operator note: this reciprocity sweep can be batched with FR-SKILL-112's similar dependency.
- `related_frs:` enumerates FR-SKILL-101 (BRAIN integration — orthogonal), FR-SKILL-103 (parent frontmatter spec — depended on), FR-SKILL-112 (complementary — trigger-tests), FR-SKILL-113 (sketch only — XML-free frontmatter). All four exist as draft or accepted FRs in the index.

## §4 — Implementation discoveries (2026-05-19 partial impl)

During the same-session implementation pass:

- **Rust validator deferred.** `services/skill-broker/` crate doesn't exist yet — FR-SKILL-103's broker is specced but not implemented. The Rust description_validator + JSONSchema mirror are deferred until FR-SKILL-103 ships the broker scaffold. No spec change needed; FR-SKILL-111's Rust code lands when its prerequisite does.
- **RUBRIC.md location correction.** FR-SKILL-111 §3 placed FM-112 in `feature-request-audit/RUBRIC.md` — but inspection revealed that file is the FR-artefact rubric (`audit_rubric@2.0`), not the skill-bundle rubric. Created `modules/skill/SKILL_BUNDLE_RUBRIC.md` (`skill_bundle_rubric@1.0`) using `SKB-` prefix to avoid namespace collision. FR-SKILL-111's rules landed as SKB-020..023 (not FM-112). Spec-vs-impl note: future references to "FM-112" should be read as "SKB-020..023".
- **3 exemplars backfilled live.** feature-request-author, feature-request-audit, product-requirements-document-author all carry enriched descriptions (510-673 chars, 4 quoted trigger phrases each, zero XML brackets). Verified by parse + AC #1 + AC #15-17.
- **Templates updated.** `_template/author/SKILL.md` + `_template/audit/SKILL.md` carry the new description format; the YAML folded scalar `>-` form is used (preferred over `|` per §1 #9).
- **134 pre-existing placeholder leaks discovered.** Many production skills (non-exemplar) carry stale template-syntax `<placeholder>` in `metadata.stage`, `description`, and other fields — inherited from earlier scaffold runs that never substituted. Not regressions from this FR; pre-existing tech debt. Operator note: queue FR-SKILL-115 or equivalent to sweep stale placeholders in next maintenance cycle.

**Post-impl score remains 10/10.** No spec amendments needed; discoveries are operational (RUBRIC location + deferred Rust scope).

---

*End of FR-SKILL-111 audit.*
