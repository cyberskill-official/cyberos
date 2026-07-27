# Status v3 — P0 decisions (locked)

Date: 2026-07-27  
Operator: Stephen Cheng  
Plan: `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md` §9

## Original P0 lock (merge path)

| # | Decision | Choice |
| --- | --- | --- |
| 1 | `cyberos-12g-clone` | Skip forever — exclude from fleet discovery/migration |
| 2 | Fleet commit/push | `commit: all cleared`, `push: none` until a second explicit allowlist |
| 3 | `traceability.scaffold_ci` | Keep `false` (opt-in); do not flip |
| 4 | Legacy page | Keep `status-legacy.html` through one minor cycle (see override below) |
| 5 | Branch protection | After merge: require `traceability-gate` + `suite-gate` on `main` |
| 6 | Traceability cutoff | Set to merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (PR #171) |
| 7 | Ship path | One PR for `feat/status-v3-platform` as-is |

## Operator override lock — 2026-07-27 (execute non-stop)

| # | Decision | Choice |
| --- | --- | --- |
| P2 | Status v3 visual HITL | **APPROVE as-is** — package at `status-v3-p2-review/`; SHA at approval recorded there |
| P6 | Release version | **1.12.0 minor** — **NOT** 2.0.0. Do not create a `v2.0.0` tag. Status v3 landing is a minor platform release |
| P6b | Legacy window reinterpret | Keep `status-legacy.html` through **1.12.x**; remove at next minor **1.13.0** (was “through 2.0.x / remove at 2.1.0”) |
| P6c | Release-range preflight | Flip warn → **blocking** at the 1.12.0 release (plan said 2.0.0; operator spirit = enforcement with the Status v3 platform release) |
| P7 | Fleet posture | Confirm **commit-all** (cleared repos), then push allowlist wave (#176); skip `cyberos-12g-clone` |
| DOCS-021 | 148 unlinked current-epoch | **Accept all 148 as pre-cutoff history** — no mass recovery via `commit-links.yaml` |

## Post-merge operator checklist

1. DONE — Merge the Status v3 PR (#171).
2. DONE — On `main`, set branch protection required checks: `traceability-gate`, `suite-gate`.
3. DONE — Mothership cutoff = merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (`scripts/check_task_link.sh` `_DEFAULT_CUTOFF`; PR #172).
4. DONE — **P2 visual HITL** — operator approved as-is (2026-07-27). See [`status-v3-p2-review/README.md`](status-v3-p2-review/README.md).
5. DONE — **P6** — VERSION **1.12.0**; tag `v1.12.0` @ `4a4c2cb9`; release-range preflight **blocking**; no `v2.0.0` tag.
6. DONE — **P7** — rollout + push wave (#176); report `docs/reviews/fleet-status-v3-migration-2026-07-27.md`.
7. DONE — **DOCS-021** — accept-all 148 pre-cutoff (triage note + operator lock).
8. DONE — **P6b / 1.13.0** — remove `status-legacy.html` + `CYBEROS_STATUS_LEGACY` dual emission; previews confirmed gone.

## Phase execution snapshot (2026-07-27 closeout)

| Phase | Gate | State |
| --- | --- | --- |
| 0 | P0 | **Complete** |
| 1 | P1 | **Complete** |
| 2 | P2 | **Complete** (visual HITL accept as-is) |
| 3 | P3 | **Complete** |
| 4 | P4 | **Complete** |
| 5 | P5 | **Complete** (offline cert + matrix; fleet-install-test covered by P7 rollout) |
| 6 | P6 | **Complete** — tagged `v1.12.0`; legacy removed at **1.13.0** |
| 7 | P7 | **Complete** — rollout + push wave (#176) |
| 8 | P8 | **Scaffold only** — watch/`improve` deferred (operator skip); not a production watch loop |

## Still open (non-blocking leftovers)

1. `cyber-click` / `styx`: no remotes — local upgrade only until an origin is configured
2. No-git trees: leave local-only
3. `Personal/my-cv` hook debt: `pnpm exec cyberskill lint-staged` (CLI not a dep) — best-effort documented; optional follow-up
4. Destructive deletes (`cyberos-12g-clone`) — never; skip forever
