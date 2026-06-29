---
fr_id: FR-EVAL-004
audited: 2026-06-29
verdict: PASS
score: 10/10
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-06-29 (no-line-cap expansion per feature-request-audit skill §0; ISS-001..013 resolved)
---

## §1 — Verdict summary

FR-EVAL-004 is the surfacing layer (brain-evaluation plan Phase 5): an access-restricted console panel over the eval service that shows FR-EVAL-003 assessments to a strictly-bounded audience, always marks the auto-score as a DRAFT a human must approve, gives the employee the right to see and respond to their own record, and audits every cross-person read.

Scope: 16 §1 clauses (panel-in-the-FR-APP-001-shell with CDS; the three-relationship deny-by-default access — founder / manager-of-report / self; access as FR-EVAL-001-grant + AUTH manager-of, not an invented rule; the manager view with rubric mapping + evidence + the draft score + the approve/override control; the auto-score always shown as a clearly-marked DRAFT that can never read as final; the mandatory human approve/override gate with a required reviewer and override reason; the employee self-view; the employee response right + "not closed until they can respond"; every cross-person AND self read audited; approve/override/response audit rows; tenant-scoping via the session; no backend beyond the eval service; visible+safe error handling distinguishing denied/empty/error; the browser never computing score/state/access; pure unit-tested render; OTel read-by-relationship metrics). 9 §2 rationale paragraphs. §3 carries the deny-by-default `authorize_read`, the read path that audits the cross-person read, the human review (approve/override) + employee response + close state machine, and the pure `renderAssessmentCard` whose DRAFT badge is bound to the service `state`. 23 ACs (incl. a TypeScript render block). §10 lists 22 failure rows. §11 lists 11 implementation notes. The `## AI Risk Assessment` section is present (high-risk) with risk-classification, data-sources, human-oversight, traceability, and failure-modes subsections, naming EU AI Act Article 14 as the core of the FR; an `## AI Authorship Disclosure` section is also present.

Frontmatter conforms to engineering-spec@1 (the FR-PROJ-008 shape): id FR-EVAL-004; module EVAL; priority MUST; status draft (per STATUS-REFERENCE.md); verify T; phase P5; milestone/slice present; depends_on [FR-EVAL-003, FR-APP-001]; blocks []; eu_ai_act_risk_class high; language rust 1.81 + static JS (apps/console); service cyberos/services/eval/ + apps/console; new_files/modified_files span both the eval service and the console; allowed_tools/disallowed_tools encode the five DEC guardrails (access-restricted, draft-not-final, response-before-close, read-audited, no-new-backend/no-invented-access); source_decisions DEC-2620..2624 capture Stephen's 2026-06-29 directions; risk_if_skipped present and names PDPD + the model-decides risk.

## §2 — Findings (all resolved)

### ISS-001 — Access could be advisory/client-side
An evaluation is the most sensitive record about a person; a browser-side access rule is bypassable. Resolved: §1 #2 server-side `authorize_read`, deny-by-default, exactly three relationships, 403 with no content/existence leak; §3 single-gate code; AC #2 #3 #4; §11 single-gate note.

### ISS-002 — A second, drifting access rule
Inventing a panel-specific access rule would create two sources of truth. Resolved: §1 #3 composes the FR-EVAL-001 view grant with AUTH's manager-of; AC #5; §11 compose-not-invent note; DEC-2624.

### ISS-003 — Auto-score reading as a decision
A bare model score reads as a verdict; a verdict affecting pay must be human. Resolved: §1 #5 persistent DRAFT badge bound to service `state`, draft visually/textually distinct, no "final" affordance on a draft; AC #6 #7 #21; §3 `renderAssessmentCard`; EU AI Act Article 14 cited.

### ISS-004 — Nothing forces a human to finalise
Showing a draft is not enough; the human's act must be the thing that finalises. Resolved: §1 #6 approve/override records a human `reviewer_subject_id`, rejects a service account (403 review_requires_human), override demands a reason; AC #8 #9 #10 #11.

### ISS-005 — Employee could be unable to see/contest
An evaluation the subject cannot see or answer is surveillance. Resolved: §1 #7 self-view, §1 #8 response capture + "not closed until opportunity to respond" (409 response_opportunity_pending); AC #12 #13 #14; §11 enforced-gate note; DEC-2622.

### ISS-006 — Reads could be silent
Who reads whose evaluation is itself sensitive. Resolved: §1 #9 every cross-person read AND self-view emits a hash-chained `eval.assessment_read` naming reader/subject/relationship; AC #15 #16; §3 read path emits before returning; DEC-2623.

### ISS-007 — Sensitive actions unaudited
Approve/override/response must be reconstructable. Resolved: §1 #10 `eval.assessment_{approved,overridden,response_recorded}` chained into l1_audit_log (FR-PROJ-008 / FR-MEMORY-123 pattern); AC #17; §8 example payloads.

### ISS-008 — Hidden backend creep
A surfacing panel could quietly grow server logic in the wrong place. Resolved: §1 #1 #12 panel in the FR-APP-001 console, no backend beyond the eval service, access/audit/state-machine live server-side (the FR-APP-005 discipline); AC #19; §11 no-backend note; DEC-2624.

### ISS-009 — Browser as a second authority on score/state/access
Client-computed score/state/allow would be wrong or forgeable. Resolved: §1 #14 the panel renders the service verdict verbatim, never recomputes, never up-grades a draft, never makes its own allow/deny; AC #21; §11 structural-enforcement note.

### ISS-010 — 403 vs empty vs error conflated
Conflating denied with "no assessment" either leaks or misleads. Resolved: §1 #13 three distinct states (access-denied / no-assessment-yet / error), none fabricated; AC #20; §3 render distinguishes; §11 distinct-states note.

### ISS-011 — Self could approve own / manager could approve a non-report
The reviewer must be the right human. Resolved: §1 #6 + §3 `review_assessment` rejects the self relationship and re-checks manager-of; AC #11; §10 self-approve row.

### ISS-012 — Stale manager-of mid-session
A manager who loses a report should lose access. Resolved: §11 + §10 — the relationship is re-checked on every read, so access drops on the next call; AC implied by per-call `authorize_read`.

### ISS-013 — Override outcome outside the rubric scale
A reviewer could record an outcome the rubric does not define. Resolved: §1 #11 / §11 override outcome validated against the FR-EVAL-002 rubric version's scale (422 otherwise); §10 out-of-range row.

## §3 — Resolution

All thirteen concerns resolved. The spec's depth is bounded by the genuine surface — deny-by-default three-relationship access × the DRAFT-vs-approved structural distinction × the mandatory human approve/override gate × the employee response-before-close gate × the audit-every-read requirement × the no-backend/no-second-authority discipline — not by a line target. The five Stephen-2026-06-29 decisions (access-restricted + contract-disclosed; both auto-score AND HITL with the draft clearly marked; employee right to respond; every cross-person read audited; panel in the console with no new backend) are each encoded in a normative clause, a decision record, an acceptance criterion, and a disallowed-tools guardrail. Human oversight (EU AI Act Article 14) is the core of the FR and is operationalised in the approve/override state machine, not bolted on. **Score = 10/10.**

---

*End of FR-EVAL-004 audit.*
