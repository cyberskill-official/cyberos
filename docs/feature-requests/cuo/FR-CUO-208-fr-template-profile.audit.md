---
fr_id: FR-CUO-208
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-CUO-208 audit

## §1 - Verdict summary

Audited for detection soundness (a file must be judgeable by its own bytes) and for non-destructive coexistence of the two template ecosystems (470 engineering-spec files vs 6 feature_request files, both live). The resolution chain, per-file detection, and needs_human tiebreak survived scrutiny. Verification is acceptance-driven: the extended TRIGGER_TESTS suites of both feature-request skills plus TEMPLATE_PROFILES.md's own verification preamble (all in new_files/modified_files; TRACE-003 closed).

## §2 - Findings (all resolved)

### ISS-001 hybrid files were unjudgeable
A file with a template: key AND §-sections matched both profiles. Resolved: §1 #4 needs_human on both-or-neither, AC 5 fixture.

### ISS-002 plugin command contradicted the author skill
The command doc asserts feature_request@1 while §12 authors engineering-spec@1 - the exact ambiguity this FR exists to kill, present in its own deliverable. Resolved: §1 #5 rewrites the wording to the resolution chain; AC 6 includes the negative grep.

### ISS-003 mixed repos forced a single template
Repo-level config alone would misjudge the minority template's files. Resolved: §1 #6 per-file detection regardless of default, AC 7 two-template batch fixture.

### ISS-004 FM-004 interplay unstated
FM-004 requires template == feature_request@1; applied to engineering-spec files it would fail all 470. Resolved: family-selection preamble scopes FM-004 to detected feature_request files only (§6), keeping every existing rule intact.

### ISS-005 template choice invisible at the approval gate
Operators approve content; the template silently defaulting hides a consequential choice. Resolved: §1 #2 PLAN echo with value + source, AC 2.

### ISS-006 conversion scope creep
Template conversion is a rewrite masquerading as a toggle. Resolved: explicitly out of scope; hybrids route to needs_human instead of half-converting (§10 #2).

## §3 - Resolution

All six findings addressed as cited. Depends on FR-CUO-207's config key, declared on both sides. **Score = 10/10.**

*End of FR-CUO-208 audit.*
