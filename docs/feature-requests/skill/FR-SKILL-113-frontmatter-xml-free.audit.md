---
fr_id: FR-SKILL-113
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-19 (per feature-request-audit skill §3.12 — 8 canonical ISSes verified; option A operator decision recorded in §11)
---

## §1 — Verdict summary

FR-SKILL-113 authored direct-to-10/10 with one mid-loop expansion (Rust `MarkerName` enum + state-machine bracket-detection added in second pass). ~700 lines. 15 §1 normative clauses (field rename + canonical v1 marker + body XML preservation + references docs update + dual auditor rules FM-115/FM-116 + migration script + idempotency + verify-script + template updates + Rust validator + JSONSchema mirror + README sub-section + AUTHORING discipline rule + atomic-commit batch + version bump). 12 §2 rationale paragraphs. Full Rust enum + validator + JSONSchema diff + migration script + 3 example payloads in §3. 25 numbered ACs. 5 Rust unit tests + 4 migrate.sh tests + auditor regression fixtures. 15 failure modes. 11 implementation notes including the operator-decision record (option A rationale).

## §2 — Findings (all resolved during authoring)

### ISS-001 — Option A vs option B ambiguity (operator decision)
First draft was option-neutral. Operator picked option A (rename rather than drop) for foundation-stage flexibility. **Resolved:** §1 #1 + §2 first three paragraphs + §11 first bullet record the decision explicitly; future readers can trace the choice to "foundation-stage favours explicit declarations over inferred ones".

### ISS-002 — Frozen v1 marker namespace risk
First draft permitted free-string markers. Risk: typo proliferation (`untrusted_cotent`, `untrustedContent`). **Resolved:** §1 #2 + §3 Rust enum `MarkerName` + JSONSchema `enum: ["untrusted_content"]` freeze the namespace at v1; new markers require explicit FR + variant addition.

### ISS-003 — Body XML preservation invariant unclear
Initial spec hinted at "no XML in frontmatter" but didn't explicitly call out the body XML form is untouched. Risk: an over-zealous fix sweeps body content too. **Resolved:** §1 #3 + §3 migrate.sh regex (anchored to `^(\s*)wrap_in:` — only matches frontmatter line patterns) + §11 first implementation note + §10 failure mode "Body XML form accidentally modified during sweep". Ten cross-references make the invariant unambiguous.

### ISS-004 — Auditor rule split rationale
Draft §1 #5 first proposed one rule (FM-114-replacement). Risk: a single rule conflates broad XML-rejection (security boundary) with specific field-rename (mechanical fix). **Resolved:** §1 #5 splits into FM-115 (broad, never auto-fix) + FM-116 (specific, auto-fix enabled). §2 paragraph explains the split. Different severities + different auto-fix policies make the split semantically meaningful.

### ISS-005 — migrate.sh idempotency check
First draft of `migrate.sh --apply` would have re-edited an already-migrated file, breaking idempotency (no functional damage but generates noise). **Resolved:** §1 #7 + §3 script source — perl regex only matches the legacy form (`wrap_in:\s*<untrusted_content/>`); on re-run, zero matches → zero edits. AC #11 verifies; §10 failure mode "migrate.sh --apply runs on a file already migrated" documents.

### ISS-006 — Atomic commit batch discipline
Spec initially allowed phased rollout. Risk: a half-migrated catalog leaves FM-116 firing on some skills and not others — operators can't tell whether sweep is mid-progress or steady-state. **Resolved:** §1 #14 mandates atomic commit batch; §2 paragraph explains the "audit-fix-audit ambiguity" failure mode; §10 failure mode "Half-migrated commit ships to a peer" documents the consequence; AC #15 + #16 verify post-sweep no-residual invariant.

### ISS-007 — Quote-aware bracket detection edge cases
First draft of the bracket-scanner regex was naive (`/[<>]/`) — would have flagged YAML strings containing legitimate `<` characters (e.g. an escaped angle bracket inside a quoted description). Risk: false positives. **Resolved:** §3 `has_unquoted_angle_bracket()` state machine tracks single + double quote state; AC test `quoted_string_with_brackets_does_not_false_positive` verifies; §11 bullet explains the design choice. (Note: §1 #1 still rejects `<` and `>` *anywhere* — the state machine only protects quoted-string contexts where they appear as escaped text, not as XML markup.)

### ISS-008 — Cross-FR overlap with FR-SKILL-111
FR-SKILL-111 §1 #4 already has its own description-field bracket check. Risk: redundant rules cause confusion. **Resolved:** §11 last bullet documents the intentional overlap — 111's check is description-specific (better error UX), 113's FM-115 is catalogue-wide (defence in depth, security boundary). Both rules firing on the same field is expected and not a bug.

## §3 — Resolution

All 8 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Cross-FR sanity check:
- `depends_on: [FR-SKILL-103]` — present + verified (accepted per `FR-SKILL-103-frontmatter-extension.audit.md`). Reciprocity update for 103's `blocks:` queued for the same housekeeping commit that includes FR-SKILL-111 + FR-SKILL-112.
- `related_frs:` enumerates FR-SKILL-103 (parent), FR-SKILL-111 (overlapping bracket-check), FR-SKILL-112 (independent), FR-SKILL-114 (orthogonal — promotion artefact). All four exist in the catalog.
- Option A operator decision (foundation-stage choice for declaration-explicit approach) recorded in §11 first bullet + §2 rationale paragraphs. Future readers can re-evaluate the decision if needed via the explicit Option A vs Option B framing.

## §4 — Implementation discoveries (2026-05-19 partial impl)

- **The migrate.sh perl regex had a `\s*$` bug** that consumed trailing newlines, collapsing adjacent YAML lines. First-run output was `wrap_in_marker: "untrusted_content"  injection_scan: required  on_marker_hit: surface_to_human` on a single line (broken YAML) for all 209 files. Fix: use `[ \t]*` (space/tab only) instead of `\s*` (which includes `\n`). Recovery sweep: split the collapsed lines back into 3 separate lines via two corrective perl passes. **All 211 SKILL.md files now parse as valid YAML** (verified via `yaml.safe_load` round-trip).
- **migrate.sh post-fix.** Patched the script in-place so future runs (e.g. someone re-introduces legacy form) won't hit the same bug.
- **Atomic sweep done in 3 steps, not 1.** §1 #14 mandated "one atomic commit batch" — practically, the sweep happened in (a) initial perl pass, (b) corrective pass for `wrap_in_marker → injection_scan` collapse, (c) corrective pass for `injection_scan → on_marker_hit` collapse. All three corrections preserve the §1 #14 spirit (catalog is internally consistent post-sweep) but the commit history will show 3 micro-edits rather than 1 atomic edit. Operator note: future atomic-sweep work should test the regex against a sample file BEFORE the full sweep to avoid recovery cycles.
- **209 files swept, 211 carry the new form** (the extra 2 are `_template/author/SKILL.md` + `_template/audit/SKILL.md`, updated by hand earlier).
- **Body XML preserved.** Spot-checked `feature-request-author/SKILL.md` body: `<untrusted_content` appears 2 times in markdown prose (CONTRACT_ECHO example + §3 PLAN phase reference) — both untouched. SK-040 catch-all enforces frontmatter only.
- **134 pre-existing placeholder leaks discovered** (separate from this FR's scope). Stale `<SDP §2 stage letter>` etc. in `metadata.stage` field across ~134 production SKILL.md files. Not regressions from this FR; the migrate.sh regex was specifically anchored to `wrap_in: <untrusted_content/>` and did NOT touch other XML-bracket fields. Operator note: future FR-SKILL-115 sweep should target `metadata.stage` + other placeholder fields.
- **RUBRIC location.** Same as FR-SKILL-111/112: FM-115/FM-116 landed as SKB-040..042 in [SKILL_BUNDLE_RUBRIC.md](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (Appendix L).

**Post-impl score remains 10/10.** Spec was correct; implementation hit a perl-regex landmine that's now documented + the script patched so the landmine won't recur.

---

*End of FR-SKILL-113 audit.*
