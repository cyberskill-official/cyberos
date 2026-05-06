# Standalone-mode entry interview (1-2 questions)

> When `prd-audit` is invoked from chat without a pre-built input envelope.

## Q1 — Which PRD(s)?

> "Which PRD(s) should I audit? Paste paths to the `prd@1` markdown files (e.g., `./prds/saved-filters.prd.md`)."

Validate: each path resolves; each parses as `prd@1` (frontmatter `template: prd@1`).

## Q2 — Rubric version (optional)

> "Which rubric version? Default: `prd_rubric@1.0`. Press Enter to accept the default or type an override."

Validate: matches `^prd_rubric@\d+\.\d+$`. Future rubric versions ship with skill MAJOR bumps.

## After the interview

Synthesise input envelope; emit CONTRACT_ECHO; proceed to PHASE_1_LOCATE.

## When the interview is skipped

Chained mode from `prd-author`: supervisor passes the input envelope; this skill validates + proceeds without re-prompting.

## Citations

- Pattern source — `cuo/cpo/fr-audit/STANDALONE_INTERVIEW.md`.
