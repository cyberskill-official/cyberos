---
task_id: TASK-PROJ-017
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..017 added)
---

## §1 — Verdict summary

TASK-PROJ-017 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 23 §1 clauses (open paths, responsive, TipTap+Yjs, Y.Array comments, LWW sidebar, presence cursors, history drawer, kbd, audit, RLS, a11y, OTel, comment threading, mentions+notify, attachments, reactions, @lumi in comments, sidebar quick-links, draft auto-save, kbd comment nav, typing indicator, follow/unfollow, markdown shortcuts). 18 §2 rationale paragraphs. §3 contains Modal + Description + MetaSidebar + HistoryDrawer + presence + responsive layout. 33 ACs. §10 lists 31 failure rows. §11 lists 30 implementation notes covering TipTap config, awareness color derivation, focus-trap library, draft localStorage scope, threading depth UX, mention regex, attachment upload flow, @lumi opt-in, follow-default heuristics.

## §2 — Findings (all resolved)

### ISS-001 — Editor + CRDT binding
Without TipTap+Collab extension, custom Yjs binding ships drift. Resolved: §1 #3 + DEC-381 standard extension.

### ISS-002 — Responsive layout
Without spec, breakpoint left to whim. Resolved: §1 #2 + DEC-382 1024px threshold.

### ISS-003 — Save semantics with CRDT
Cmd+S in CRDT world is no-op; but users expect feedback. Resolved: §1 #8 + AC #16 toast feedback.

### ISS-004 — Presence visibility
Without cursor flags, two users overwrite invisibly. Resolved: §1 #6 + CollaborationCursor; AC #8.

### ISS-005 — Focus trap
WCAG modal requirement. Resolved: §1 #11 + AC #17.

### ISS-006 — Deep-link URL
Without URL state, modal not shareable. Resolved: §1 #1 + AC #2.

### ISS-007 — Comment thread loses structure (strict-redo pass)
Flat list lost conversation context. Resolved: §1 #13 + threading + depth-5 cap + AC #23.

### ISS-008 — Mentions absent (strict-redo pass)
Standard @user tagging missing. Resolved: §1 #14 + auto-complete + notify + AC #24.

### ISS-009 — Attachments missing (strict-redo pass)
Comments couldn't carry files (screenshots, logs). Resolved: §1 #15 + task-FILES upload + AC #25.

### ISS-010 — Reactions missing (strict-redo pass)
Lightweight feedback signal absent. Resolved: §1 #16 + reaction tally + AC #26.

### ISS-011 — No in-context LLM (strict-redo pass)
@lumi worked in chat but not in issue comments. Resolved: §1 #17 + comment routing + AC #27.

### ISS-012 — Sidebar quick-links missing (strict-redo pass)
Adding deps/links required leaving modal. Resolved: §1 #18 + inline dialogs + AC #28.

### ISS-013 — Lost draft text on close (strict-redo pass)
Accidental modal close lost composer text. Resolved: §1 #19 + localStorage auto-save + AC #29.

### ISS-014 — No kbd comment navigation (strict-redo pass)
Mouse-required for comment review. Resolved: §1 #20 + J/K nav + AC #30.

### ISS-015 — Duplicate concurrent typing (strict-redo pass)
Two users typing simultaneously waste effort. Resolved: §1 #21 + typing indicator + AC #31.

### ISS-016 — No follow/unfollow control (strict-redo pass)
Operators couldn't opt in/out of issue updates. Resolved: §1 #22 + follow toggle + AC #32.

### ISS-017 — Markdown shortcuts absent (strict-redo pass)
Operators expected standard markdown. Resolved: §1 #23 + TipTap markdown ext + AC #33.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (TipTap × Yjs × CRDT × LWW × presence × history × responsive × kbd × threading × mentions × attachments × reactions × lumi × quick-links × draft × follow × markdown), not by line targets.

---

*End of TASK-PROJ-017 audit.*
