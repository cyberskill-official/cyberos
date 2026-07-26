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

1. DONE — Merge the Status v3 PR (#171). Do not tag v2.0.0 yet — P6 still blocked.
2. DONE — On `main`, set branch protection required checks: `traceability-gate`, `suite-gate`.
3. DONE — Mothership cutoff = merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (`scripts/check_task_link.sh` `_DEFAULT_CUTOFF`; PR #172).
4. OPEN — **P2 visual HITL** — review package at [`status-v3-p2-review/README.md`](status-v3-p2-review/README.md). Machine DOM suite green; human eyes still required.

## Phase execution snapshot (2026-07-27)

| Phase | Gate | State |
| --- | --- | --- |
| 0 | P0 | **Done** — PR #171 + protection + cutoff #172 |
| 1 | P1 | **Done** — status-feed@1 on main |
| 2 | P2 | **Code shipped; waiting on visual HITL** |
| 3 | P3 | **Largely done** — wide regen + coverage-only + deploy trigger work landed |
| 4 | P4 | **Mostly done** — installer/cutoff/preflight warn landed; 148-row triage ledger recorded (accept-as-history); no mass ledger edit |
| 5 | P5 | **Mostly done** — matrix + offline cert green; full `fleet-install-test.sh` deferred (mutates consumers; re-run at P7) |
| 6 | P6 | **Blocked** — needs Stephen GO (changelog, Release-As, tag, flip release-range blocking) |
| 7 | P7 | **Blocked** — needs Stephen GO (discovery/pilots/rollout; confirm `push: none`) |
| 8 | P8 | **Docs mostly landed** — feed spec + runbook + AGENT-ENTRY text; watch/`improve` scaffolded only |

## Still needs Stephen

1. P2 visual accept / change list  
2. P6 v2.0.0 GO  
3. P7 fleet GO (+ confirm push:none)  
4. Any destructive choice (delete 12g-clone, push fleet, further branch-protection changes)
