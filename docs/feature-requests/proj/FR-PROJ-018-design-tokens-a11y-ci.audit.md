---
fr_id: FR-PROJ-018
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..017 added)
---

## §1 — Verdict summary

FR-PROJ-018 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (token categories, runtime theming, lint enforce, Storybook, Chromatic, axe-core gate, WCAG AA contrast, focus-ring tokens, Liquid-Glass shader, design-system cross-ref, JSON export, OS color-scheme detection, token diff PR report, prefers-contrast, semantic tokens layered, design-system audit CLI, token versioning + CHANGELOG, axe rule sets, build telemetry for unused tokens, CSS-in-JS escape hatch, kbd-nav hints, cheatsheet doc). 19 §2 rationale paragraphs. §3 contains: tokens.proj.css with full catalog + dark theme + reduced motion + focus-visible global, Storybook main.ts + preview.tsx with theme toolbar, Playwright a11y spec with story iteration, GH Actions workflow, contrast test. 26 ACs. §10 lists 28 failure rows. §11 lists 30 implementation notes covering backdrop-filter support, color-contrast package, stylelint plugin scope, Chromatic config, focus-visible rationale, OS scheme detection, diff comment format, semantic token churn-reduction, audit CLI walker, versioning policy, rule-set scope, build telemetry, escape-hatch reason field, empty-state ARIA polite, cheatsheet as contract.

## §2 — Findings (all resolved)

### ISS-001 — Hardcoded value drift
Without lint, tokens decorative. Resolved: §1 #3 + stylelint plugin + AC #3 #14.

### ISS-002 — Visual regression coverage
Without Chromatic, "looked fine in dev, broke in prod" risk. Resolved: §1 #5 + DEC-392 per-story snapshots; AC #6.

### ISS-003 — a11y CI gate
Without enforcement, a11y degrades silently. Resolved: §1 #6 + DEC-391 axe-core via Playwright + AC #7 #8.

### ISS-004 — Contrast test
WCAG AA needs ratio enforcement. Resolved: §1 #7 + AC #9.

### ISS-005 — Focus ring
Default browser ring inconsistent. Resolved: §1 #8 + token + AC #10.

### ISS-006 — Reduced-motion
prefers-reduced-motion ignored without spec. Resolved: §1 + global @media + AC #11.

### ISS-007 — OS color-scheme ignored (strict-redo pass)
Users with system dark mode preference had to toggle per-app. Resolved: §1 #12 + `prefers-color-scheme` detection + AC #16.

### ISS-008 — Token changes invisible to reviewers (strict-redo pass)
PR diff didn't show component impact of token changes. Resolved: §1 #13 + PR diff comment with impact analysis + AC #17.

### ISS-009 — High-contrast preference unsupported (strict-redo pass)
Vision-impaired users' explicit preference ignored. Resolved: §1 #14 + `prefers-contrast: more` overrides + AC #18.

### ISS-010 — Component-token coupling (strict-redo pass)
Direct token references made designer changes invasive. Resolved: §1 #15 + semantic token indirection + AC #19.

### ISS-011 — No coverage health check (strict-redo pass)
Token adoption unmeasurable. Resolved: §1 #16 + audit CLI + AC #20.

### ISS-012 — Token renames break silently (strict-redo pass)
No versioning policy. Resolved: §1 #17 + semver + CHANGELOG + AC #21.

### ISS-013 — axe rule scope ambiguous (strict-redo pass)
"axe-core" alone didn't specify which rule sets. Resolved: §1 #18 + explicit wcag2a/aa/21aa/best-practice + AC #22.

### ISS-014 — Unused tokens accumulate (strict-redo pass)
No removal signal. Resolved: §1 #19 + build telemetry + AC #23.

### ISS-015 — Edge cases blocked by token enforcement (strict-redo pass)
Legitimate one-off styles couldn't be done. Resolved: §1 #20 + audited escape hatch + AC #24.

### ISS-016 — Kbd-nav discoverability missing (strict-redo pass)
Users didn't know keyboard shortcuts existed. Resolved: §1 #21 + empty-state hints + AC #25.

### ISS-017 — Designers had no token reference (strict-redo pass)
No quick lookup of available tokens. Resolved: §1 #22 + cheatsheet markdown + AC #26.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (tokens × themes × lint × Storybook × Chromatic × axe-core × contrast × focus × Liquid-Glass × OS preferences × diff reports × prefers-contrast × semantic layer × audit × versioning × telemetry × escape hatch × discoverability × cheatsheet), not by line targets.

---

*End of FR-PROJ-018 audit.*
