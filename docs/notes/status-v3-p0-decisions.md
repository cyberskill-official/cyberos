# Status v3 — P0 decisions (locked)

Date: 2026-07-27  
Operator: Stephen (“go as your judgment”)  
Plan: `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md` §9

| # | Decision | Choice |
| --- | --- | --- |
| 1 | `cyberos-12g-clone` | Skip forever — exclude from fleet discovery/migration |
| 2 | Fleet commit/push | `commit: all cleared`, `push: none` until a second explicit allowlist |
| 3 | `traceability.scaffold_ci` | Keep `false` (opt-in); do not flip |
| 4 | Legacy page | Keep `status-legacy.html` through 2.0.x; remove at 2.1.0 |
| 5 | Branch protection | After merge: require `traceability-gate` + `suite-gate` on `main` |
| 6 | Traceability cutoff | Set to merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (PR #171) |
| 7 | Ship path | One PR for `feat/status-v3-platform` as-is |

## Post-merge operator checklist

1. DONE — Merge the Status v3 PR (do not tag v2.0.0 yet — P6 still blocked).
2. DONE — On `main`, set branch protection required checks: `traceability-gate`, `suite-gate`.
3. DONE — Mothership cutoff = merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (`scripts/check_task_link.sh` `_DEFAULT_CUTOFF` + local `.cyberos/config.yaml`).
4. P2 visual review of the emitted page remains welcome; P6/P7 stay blocked.
