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
| 6 | Traceability cutoff | After merge: follow-up on `main` sets cutoff = merge SHA (not before merge) |
| 7 | Ship path | One PR for `feat/status-v3-platform` as-is |

## Post-merge operator checklist

1. Merge the Status v3 PR (do not tag v2.0.0 yet — P6 still blocked).
2. On `main`, set branch protection required checks: `traceability-gate`, `suite-gate`.
3. Follow-up commit: set mothership `traceability.cutoff` (and script default if any) to the **merge SHA**.
4. P2 visual review of the emitted page remains welcome; P6/P7 stay blocked.
