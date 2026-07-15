# Changelog — INV

## 2026-05-15 — INV module page rewritten to Gold (billable rollup invoicing + hóa đơn emission + dunning automation)

Rewrote `website/docs/modules/inv.html` to Gold by encoding three strategic roles: (1) billable rollup → invoice line items (consumes TIME per-cycle rollup; rate-card snapshot preserved), (2) hóa đơn emission (Decree 123 + Circular 78 GDT XML via vietnam-vat-invoice skill; Mẫu 01/GTGT; MST validation gate), (3) revenue recognition + dunning (CUO drafts overdue chase; human sends; aging report; cash application via 4 rails).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + INV-in-orchestration-spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-INV-011..015): incomplete TIME rollup → missing hours · rate-card snapshot divergence · hóa đơn cancellation without dual approval (Critical) · dunning auto-send bug · Decree 123 amendment drift
- KPIs +5: TIME→INV bridge p95 · missing-Member draft rate · rate-card snapshot integrity (= 1.0) · dunning auto-send false-positive (= 0) · hóa đơn dual-approval rate (= 1.0)
- References expanded: §0 + 6 cross-module links + PROJ §2.6 billing modes + TIME §0 rollup contract + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + task-audit skill

