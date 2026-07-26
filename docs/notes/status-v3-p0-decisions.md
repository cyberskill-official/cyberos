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
| P7 | Fleet posture | Confirm **commit-all** (cleared repos), **push: none** (no second allowlist; do not push fleet remotes) |
| DOCS-021 | 148 unlinked current-epoch | **Accept all 148 as pre-cutoff history** — no mass recovery via `commit-links.yaml` |

## Post-merge operator checklist

1. DONE — Merge the Status v3 PR (#171).
2. DONE — On `main`, set branch protection required checks: `traceability-gate`, `suite-gate`.
3. DONE — Mothership cutoff = merge SHA `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (`scripts/check_task_link.sh` `_DEFAULT_CUTOFF`; PR #172).
4. DONE — **P2 visual HITL** — operator approved as-is (2026-07-27). See [`status-v3-p2-review/README.md`](status-v3-p2-review/README.md).
5. DONE — **P6** — VERSION **1.12.0**; tag `v1.12.0` @ `4a4c2cb9`; release-range preflight **blocking**; no `v2.0.0` tag.
6. DONE (pending push allowlist) — **P7** — rollout clean; 21 local upgrade commits; push: none; report `docs/reviews/fleet-status-v3-migration-2026-07-27.md`.
7. DONE — **DOCS-021** — accept-all 148 pre-cutoff (triage note + operator lock).

## Phase execution snapshot (2026-07-27)

| Phase | Gate | State |
| --- | --- | --- |
| 0 | P0 | **Done** — PR #171 + protection + cutoff #172 |
| 1 | P1 | **Done** — status-feed@1 on main |
| 2 | P2 | **Approved** — visual HITL accept as-is |
| 3 | P3 | **Largely done** — wide regen + coverage-only + deploy trigger work landed |
| 4 | P4 | **Done for release** — installer/cutoff/preflight; DOCS-021 accept-all locked; no mass ledger edit |
| 5 | P5 | **Mostly done** — matrix + offline cert green; full `fleet-install-test.sh` at P7 |
| 6 | P6 | **Done** — tagged `v1.12.0` (not 2.0.0); preflight blocking |
| 7 | P7 | **Rollout done** — commits local; push deferred; see migration report |
| 8 | P8 | **Docs mostly landed** — feed spec + runbook + AGENT-ENTRY text; watch/`improve` scaffolded only |

## Still needs Stephen (true blockers only)

1. Second explicit allowlist if any fleet remote must be **pushed**
2. Any destructive choice (delete 12g-clone, further branch-protection changes)
3. P7 report review / exceptions after migration report lands
