# Status v3 integration note

The approved `docs/status-v3-preview/` canvas was promoted to the platform standard in
Phase 2 of `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md`
(TASK-DOCS-013..016). Phase 8 docs: TASK-DOCS-028.

- Generator: `tools/docs-site/render-status-hub.mjs` emits status-hub@3 (tabless canvas)
  as `reference/status.html`, plus `status-legacy.html` (v2 escape hatch) for one minor cycle.
- Data: `status-feed@1` — contract at [`docs/reference/status-feed.md`](../reference/status-feed.md);
  embedded as `#sv3-data` and written to `data/status-feed.json`.
- Traceability: [`docs/runbooks/traceability.md`](../runbooks/traceability.md).
- Rollback: `CYBEROS_STATUS_LEGACY=1` makes the v2 page the primary emission.
- Live page: `docs/status/index.html` (regenerated via `.cyberos/lib/status-page.sh`).

The preview folder was deleted in the same change that landed the swap.
