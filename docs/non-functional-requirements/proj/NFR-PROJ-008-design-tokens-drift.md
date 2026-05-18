---
id: NFR-PROJ-008
title: "PROJ design-tokens drift — UI MUST consume tokens; raw hex/px values are CI-banned"
module: PROJ
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "0 raw hex or raw px values in PROJ component CSS/JSX; tokens consumed exclusively"
owner: CTO
created: 2026-05-18
related_frs: [FR-PROJ-018]
---

## §1 — Statement (BCP-14 normative)

1. PROJ UI component source (`apps/proj/src/`) **MUST NOT** contain raw hex color values (`#...`), raw RGB/HSL strings, or raw pixel values for spacing/typography.
2. All styling **MUST** consume design tokens from the design-system package via CSS variables or imported constants.
3. The CI lint **MUST** scan source files and reject violations; merge is blocked on any violation.
4. Allowed exceptions: token-definition files themselves; explicitly marked `/* a11y-allow-raw */` lines (rare, justified, reviewed).
5. New tokens added to the design system **MUST** be reviewed by the design-system owner before component code starts consuming them.

## §2 — Why this constraint

Design tokens are the platform's design-consistency mechanism. Raw values bypass the token discipline, creating drift between components and breaking theming. The strict CI gate is the only practical enforcement — code review will silently miss occasional hex strings. The exception path exists because there are legitimate raw-value cases (e.g., a specific brand SVG), but they require explicit justification.

## §3 — Measurement

- CI metric `proj_raw_color_value_count` — must be 0.
- CI metric `proj_raw_px_value_count` — must be 0.
- Counter `proj_a11y_allow_raw_marker_count` — surfaces exception use.

## §4 — Verification

- CI lint (T) — regex-scan; assert 0 violations.
- Snapshot test (T) — fixture component rendered with each theme; assert color drift across themes is zero (consequence of token consumption).
- Quarterly design-system audit.

## §5 — Failure handling

- Violation → CI block.
- Exception marker overused (> 10 per file) → sev-3; design review.
- Drift between deployed components and current token set → sev-3 design audit.

---

*End of NFR-PROJ-008.*
