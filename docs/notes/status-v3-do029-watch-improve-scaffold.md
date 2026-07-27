# TASK-DOCS-029 — Post-rollout watch + `/cyberos:improve` scaffold

Date: 2026-07-27  
Status: **scaffold complete; production watch optional / deferred** (operator skip of one-week loop).  
Depends on: TASK-DOCS-027 (fleet rollout) after P6 tag — **done** at **1.12.0** (not 2.0.0).

## Prerequisites (met)

1. Operator cut **`v1.12.0`** (P6; override — not `v2.0.0`).
2. Operator GO on fleet under commit-all + push allowlist wave (#176); skip `cyberos-12g-clone` (P7).
3. Pilots + remaining rollout report accepted (`docs/reviews/fleet-status-v3-migration-2026-07-27.md`).
4. One-week production watch is **optional** — do not start unless operator asks.

## Watch checklist (if/when started)

| Check | How | Pass look |
| --- | --- | --- |
| `suite-gate` on mothership `main` | GitHub Actions | green |
| `traceability-gate` on mothership `main` | GitHub Actions | green |
| `deploy.yml` docs job (if paths hit) | GitHub Actions | green; served page stays status-hub@3 |
| Served page freshness | `curl -fsS https://os.cyberskill.world/docs/reference/status.html \| grep status-hub@3` | template id present; VERSION matches latest release |
| New-commit traceability | status page Releases / Traceability band | new commits after cutoff show linked (trend → 100% for post-cutoff) |
| Hook cost | time a code-only commit with coverage-only regen | target **&lt;1s** page path; file follow-up if louder |
| `audit-fleet.sh` bands | v3 pages via `assets/status.js` band ids | no false `band:pulse` etc. (fixed in 1.13.0 closeout) |

## Follow-up task filing rules

File new `TASK-IMP-*` / `TASK-DOCS-*` drafts when:

- Hook cost exceeds ~1s on code-only commits
- Any consumer reports install/regen breakage after 1.12.0 / 1.13.0
- Served page lags HEAD after deploy (truth-window bug)
- Traceability false positives/negatives on shorthand vs canonical

## `/cyberos:improve` pass

After the **first 10 gated tasks** complete under the new contribution contract:

1. Run `/cyberos:improve` (or `node .cyberos/docs-tools/workflow-improve.mjs` if installed).
2. Fold lessons into ship-tasks / commit-msg / status regen skills.
3. Record outcomes in BRAIN + a short note under `docs/notes/`.

## Explicit non-goals for this scaffold

- No fleet discovery, install, or commit-fleet runs from this note
- No Release-As / tag / channel publish from the watch scaffold alone
- No long-lived agent watch loop unless operator starts one
