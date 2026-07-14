---
task_id: TASK-PROJ-015
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 15
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..015 added)
---

## §1 — Verdict summary

TASK-PROJ-015 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 23 §1 clauses (X-axis days, Y-axis swimlanes, bar positioning, drag-move, edge-resize, LWW PATCH, milestones, today indicator, kbd parity, lazy render, audit, OTel, RLS, a11y, overlap stacking, URL window override, snap-to-week/month, per-assignee filter, basic dependency arrows, show-non-active toggle, weekend highlight, kbd swimlane reorder, cycle goal banner). 16 §2 rationale paragraphs. §3 contains: Timeline.tsx + IssueBar.tsx with resize handles + LWW PATCH integration. 27 ACs. §10 lists 28 failure rows. §11 lists 26 implementation notes covering DAY_PX zoom constants, IntersectionObserver pattern, absolute positioning, today indicator pulse, overlap stacking, URL window precedence, snap-to-week math, dependency arrow SVG, weekend bg tokens, swimlane reorder persistence, cycle banner sticky-top, prefers-reduced-motion handling.

## §2 — Findings (all resolved)

### ISS-001 — Drag UX (move vs resize)
Without edge-vs-middle distinction, drags ambiguous. Resolved: §1 #4 #5 + ResizeHandle component; AC #4 #5.

### ISS-002 — 1-day minimum
Sub-day bars are invalid; without snap, drags create zero-width bars. Resolved: §1 #5 + AC #6.

### ISS-003 — Performance at scale
50+ swimlanes janks. Resolved: §1 #10 IntersectionObserver lazy render; AC #12.

### ISS-004 — Milestone visibility
Fixed-Fee milestones invisible at planning time. Resolved: §1 #7 + DEC-362 gold markers; AC #7.

### ISS-005 — LWW reject UX
Stale resize silently lost = user confusion. Resolved: §1 #6 + AC #13 rollback + toast.

### ISS-006 — Kbd parity for resize
Per TASK-PROJ-014 precedent, kbd must match. Resolved: §1 #9 Shift+Arrow + Cmd+Shift; AC #10 #11.

### ISS-007 — Concurrent issue overlap invisible (strict-redo pass)
3 issues at same time = stacked on top each other invisibly. Resolved: §1 #15 + vertical stack + badge + AC #19.

### ISS-008 — Cycle-bound window inflexible (strict-redo pass)
Operators want to look beyond cycle boundary. Resolved: §1 #16 + URL window override + AC #20.

### ISS-009 — Drag precision at coarse zoom (strict-redo pass)
Week zoom + day-precision drag = imprecise. Resolved: §1 #17 + snap-to-week + modal for precision + AC #21.

### ISS-010 — 1:1 review view missing (strict-redo pass)
Operator wants single-assignee full-width view. Resolved: §1 #18 + per-assignee filter + AC #22.

### ISS-011 — Dependencies invisible in timeline (strict-redo pass)
Issue dependencies were known but not visualised. Resolved: §1 #19 + basic arrow rendering + AC #23.

### ISS-012 — Capacity planning blocked (strict-redo pass)
Empty swimlanes hidden by default; can't plan against unused capacity. Resolved: §1 #20 + show-non-active toggle + AC #24.

### ISS-013 — Weekend planning unsignaled (strict-redo pass)
Bars spanning weekends look the same as workdays. Resolved: §1 #21 + faded weekend bg + AC #25.

### ISS-014 — Swimlane order fixed (strict-redo pass)
Operators want to reorder lanes (team lead first). Resolved: §1 #22 + kbd reorder + persistence + AC #26.

### ISS-015 — No cycle context (strict-redo pass)
Operators looking at timeline don't see cycle goal / countdown. Resolved: §1 #23 + sticky banner + AC #27.

## §3 — Resolution

All 15 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (day-grid × swimlanes × drag/resize × milestones × kbd × overlap × URL window × snap × dependency arrows × capacity × weekend × reorder × banner), not by line targets.

---

*End of TASK-PROJ-015 audit.*
