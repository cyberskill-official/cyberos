# Status v3 integration note

The approved `docs/status-v3-preview/` canvas was promoted to the platform standard in
Phase 2 of `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md`
(TASK-DOCS-013..016). Phase 8 docs: TASK-DOCS-028. P0 decisions:
[`status-v3-p0-decisions.md`](status-v3-p0-decisions.md).
P2 review package (HITL open): [`status-v3-p2-review/README.md`](status-v3-p2-review/README.md).

- Generator: `tools/docs-site/render-status-hub.mjs` emits status-hub@3 (tabless canvas)
  into a temp `reference/status.html`, then migrate publishes `docs/status/index.html`,
  plus `status-legacy.html` (v2 escape hatch) for one minor cycle.
- Data: `status-feed@1` — contract at [`docs/reference/status-feed.md`](../reference/status-feed.md);
  embedded as `#sv3-data` and written to `data/status-feed.json`.
- Traceability: [`docs/runbooks/traceability.md`](../runbooks/traceability.md).
- Rollback: `CYBEROS_STATUS_LEGACY=1` makes the v2 page the primary emission.
- Local page: `docs/status/index.html` (regenerated via `tools/install/lib/status-page.sh`
  or `.cyberos/lib/status-page.sh` after install).
- Served docs site: `https://os.cyberskill.world/docs/reference/status.html` (built by
  `tools/docs-site/build.sh` on deploy — not the same path as the local `docs/status/` tree).

The preview folder was deleted in the same change that landed the swap.
