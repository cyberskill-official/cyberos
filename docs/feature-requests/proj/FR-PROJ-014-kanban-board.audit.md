---
fr_id: FR-PROJ-014
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..016 added)
---

## §1 — Verdict summary

FR-PROJ-014 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (6 columns, WS subscribe, drag/drop + snapback, kbd parity, virtualisation, IssueCard, awareness, audit, OTel metrics, a11y, offline, URL filter, soft WIP limits, compact cards at low zoom, bulk select, quick-add `c`, swimlanes by assignee, scroll preservation, 5s undo, drag-target preview, keyboard reorder, WIP overflow indicator). 18 §2 rationale paragraphs. §3 contains: Board.tsx + Column.tsx + KeyboardNav.tsx component sketches with virtualisation + dnd-kit + filter. 30 ACs. §10 lists 30 failure rows. §11 lists 28 implementation notes covering WIP soft-block rationale, compact-card thresholds, bulk select state, quick-add familiarity, swimlane virtualisation, scroll preservation method, undo window calibration, drag preview GPU acceleration, keyboard reorder persistence, WIP aria-live, touch device limitations, RTL handling.

## §2 — Findings (all resolved)

### ISS-001 — Mouse-only drag
WCAG fail without kbd parity. Resolved: §1 #4 + DEC-351 J/K/H/L + Cmd+Shift arrows; AC #6 #7 #8.

### ISS-002 — Latency feedback gap
Naive impl waits for server. Resolved: §1 #3 optimistic UI; AC #4.

### ISS-003 — Illegal-transition invisible rejection
Without animation, user assumes succeeded. Resolved: §1 #3 snapback + toast; AC #3.

### ISS-004 — Render perf at scale
1000+ items janks. Resolved: §1 #5 + DEC-352 react-window; AC #11 #12.

### ISS-005 — Awareness on shared view
Without indicators, no presence signal. Resolved: §1 #7 + YjsProvider awareness.

### ISS-006 — URL state for sharing
Filter state lost on reload. Resolved: §1 #12 URL filter sync; AC #15.

### ISS-007 — No WIP limits (strict-redo pass)
Lean/Kanban workflows need WIP discipline. Resolved: §1 #13 + soft block + override audit + AC #21.

### ISS-008 — Dense boards visually overload (strict-redo pass)
50+ cards visible = cognitive overload. Resolved: §1 #14 + compact mode at low zoom + AC #22.

### ISS-009 — No bulk operations (strict-redo pass)
Sprint planning moves dozens of cards; per-card friction. Resolved: §1 #15 + multi-select + bulk drag + AC #23.

### ISS-010 — Card creation requires nav out (strict-redo pass)
Keyboard-driven users want in-place create. Resolved: §1 #16 + `c` shortcut + inline creator + AC #24.

### ISS-011 — Standup view limited (strict-redo pass)
Daily standup pivots by assignee; columns-only view doesn't help. Resolved: §1 #17 + swimlanes + AC #25.

### ISS-012 — WS updates reset scroll (strict-redo pass)
Browsing experience broken by live updates. Resolved: §1 #18 + scroll preservation + AC #26.

### ISS-013 — Accidental drag unrecoverable (strict-redo pass)
Misdrag = stuck with wrong move. Resolved: §1 #19 + 5s undo + audit + AC #27.

### ISS-014 — Drag insert position unclear (strict-redo pass)
Order within column matters; drop landed somewhere. Resolved: §1 #20 + drag-target preview + AC #28.

### ISS-015 — No keyboard reorder (strict-redo pass)
Mouse-required for in-column ordering. Resolved: §1 #21 + Shift+J/K + persisted order + AC #29.

### ISS-016 — WIP overflow invisible (strict-redo pass)
Operators glancing at board don't see overflow. Resolved: §1 #22 + visual indicator + AC #30.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (drag/drop × kbd parity × virtualisation × WIP × compact × bulk × quick-add × swimlanes × scroll preservation × undo × drag preview × reorder), not by line targets.

---

*End of FR-PROJ-014 audit.*
