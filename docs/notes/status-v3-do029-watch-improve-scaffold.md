# TASK-DOCS-029 — Post-rollout watch + `/cyberos:improve` scaffold

Date: 2026-07-27  
Status: **scaffolded only** — do not start the production watch loop until P7 fleet GO.  
Depends on: TASK-DOCS-027 (fleet rollout) after P6 tag.

## When to start

1. Operator cuts `v2.0.0` (P6).
2. Operator GO on fleet under `commit: all cleared`, `push: none`, skip `cyberos-12g-clone` (P7).
3. Pilots + remaining rollout report accepted.
4. Then run this watch for **one week**.

## Watch checklist (daily / as CI fires)

| Check | How | Pass look |
| --- | --- | --- |
| `suite-gate` on mothership `main` | GitHub Actions | green |
| `traceability-gate` on mothership `main` | GitHub Actions | green |
| `deploy.yml` docs job (if paths hit) | GitHub Actions | green; served page stays status-hub@3 |
| Served page freshness | `curl -fsS https://os.cyberskill.world/docs/reference/status.html \| grep status-hub@3` | template id present; VERSION matches latest release |
| New-commit traceability | status page Releases / Traceability band | new commits after cutoff show linked (trend → 100% for post-cutoff) |
| Hook cost | time a code-only commit with coverage-only regen | target **&lt;1s** page path; file follow-up if louder |

## Follow-up task filing rules

File new `TASK-IMP-*` / `TASK-DOCS-*` drafts when:

- Hook cost exceeds ~1s on code-only commits
- Any consumer reports install/regen breakage after 2.0.0
- Served page lags HEAD after deploy (truth-window bug)
- Traceability false positives/negatives on shorthand vs canonical

## `/cyberos:improve` pass

After the **first 10 gated tasks** complete under the new contribution contract:

1. Run `/cyberos:improve` (or `node .cyberos/docs-tools/workflow-improve.mjs` if installed).
2. Fold lessons into ship-tasks / commit-msg / status regen skills.
3. Record outcomes in BRAIN + a short note under `docs/notes/`.

## Explicit non-goals for this scaffold

- No fleet discovery, install, or commit-fleet runs from this note
- No Release-As / tag / channel publish
- No long-lived agent watch loop in this session
