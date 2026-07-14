---
task_id: TASK-EVAL-003
audited: 2026-06-29
verdict: PASS
score: 10/10
score_pre_revision: 7.5/10
score_post_revision: 10/10
issues_resolved: 12
template: engineering-spec@1
eu_ai_act_risk_class: high
authoring_md_compliance: 2026-06-29 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant; AI Risk Assessment present + central per high-risk class)
strict_redo_pass: 2026-06-29 (no-line-cap expansion per task-audit skill §0; ISS-007..012 added — fairness, appeal, rubric-pinning, revision-on-change, rationale-on-override, defense-in-depth citation)
---

## §1 — Verdict summary

TASK-EVAL-003 is the highest-stakes FR in the BRAIN/EVAL workstream — the engine that scores employees — and the audit holds it to the high-risk bar. Current scope: 20 §1 normative clauses (cadence+on-demand single path, consent gate before recall, TASK-MEMORY-123 recall-only evidence, evidence→rubric mapping, citation-coverage boundary invariant, draft-only output, mandatory HITL finalize gate, consequential-outcome hard gate, subject view+rebuttal, full audit-chain on every state change, ai-gateway with spend cap+residency+ZDR, no-score fallback, refuse-to-score prompt boundary, bias-blinding + disparity flag, appeal-to-second-reviewer, RLS visibility founder|manager|self, rubric-version pinning, append-only-on-finalize with revisions, mandatory reviewer rationale on override, metrics). 11 §2 rationale paragraphs tying each control to the governance plan's Phase 4 / "Doing the monitoring responsibly" and Stephen's decisions. §3 carries 4 migrations (assessment + assessment_score + assessment_state_event + assessment_rebuttal, with RLS + REVOKE + the citation CHECK + the human-finalize CHECK), the Rust `AssessmentState` machine with a `legal_transition` table that has no Genie→final edge, `FinalizeGuard`, the `validate_citation` boundary, and the Python GENIE orchestration that drafts only. 25 ACs. §10 lists 27 failure rows. §11 lists 12 implementation notes. The `## AI Risk Assessment` is present and central (Data Sources / Human Oversight / Failure Modes), as the high-risk class requires.

The structural claim the audit verified hardest: **there is no path — by config, flag, or code — by which a model output becomes a final assessment or a consequential outcome without a distinct human reviewer's explicit act.** This is enforced redundantly (state-machine edge table + DB CHECK + FinalizeGuard) and is the FR's reason to exist.

## §2 — Findings (all resolved)

### ISS-001 — Model could auto-decide a consequential outcome
The catastrophic failure for an employee-scoring system. Without a structural gate, a model number could flow into pay/progression. Resolved: §1 #7 (mandatory human finalize) + §1 #8 (consequential hard gate, inert-until-ack, no auto-apply) encoded as a state machine with **no Genie→final edge** and a DB `CHECK (new_state NOT IN ('approved','changed','rejected') OR actor_kind='human')`; AC #6 #9 #10; DEC-2511/2512.

### ISS-002 — Unsourced scores (unfalsifiable assertions)
A score a subject can't contest and a reviewer can't check is an accusation, not an evaluation. Resolved: §1 #5 citation-coverage boundary invariant (≥1 grounded evidence event + exactly 1 rubric clause per scored row), enforced in prompt + Rust `validate_citation` + DB CHECK; AC #3 #4 #5; DEC-2513.

### ISS-003 — Evaluating people who never consented
Drafting for a subject with no acknowledged TASK-EVAL-001 notice is a PDPD/Labor-Code breach. Resolved: §1 #2 gates *recall itself* on `has_acknowledged_notice` → `403 not_consented`, no recall, audit row; AC #1; DEC-2510.

### ISS-004 — Fabricated score on model failure
An offline/hallucinating model emitting a default number looks authoritative and punishes/launders silently. Resolved: §1 #12 #13 — the only legal fallback is `score=null, needs_human_review` + `eval.fallback_no_score`; `validate_citation` rejects any number on a non-scored clause; AC #16 #17; DEC-2514. Three places forbid a fabricated number; none default one.

### ISS-005 — Evidence could leak to an unbounded provider
Sensitive employment evidence to a retaining/cross-region model is a data-governance failure. Resolved: §1 #11 — all calls via the ai-gateway `eval.score` route (spend cap + residency + ZDR on); no provider keys in the eval service; AC #14 #15.

### ISS-006 — No audit trail of who decided what on what evidence
A grievance/wrongful-termination review needs a defensible record. Resolved: §1 #10 — every state change emits an append-only `assessment_state_event` chained to `l1_audit_log` (TASK-PROJ-008 chain_anchor pattern) carrying actor, evidence ids, rubric version, change summary; AC #13; DEC-2515.

### ISS-007 — No fairness/bias guardrail (strict-redo pass)
Scoring on identity rather than behavior is the classic high-risk-AI harm. Resolved: §1 #14 — protected-attribute blinding before the model sees text + post-hoc group-disparity check that **flags for a human, never auto-adjusts** (an auto-correcting pass is its own bias); AC #18 #19.

### ISS-008 — No escalation beyond the in-cycle rebuttal (strict-redo pass)
A finalized-but-disputed assessment needs a real appeal. Resolved: §1 #15 — appeal routes to a *second* human reviewer (≠ original), recorded as `eval.assessment_appealed`, within the appeal window; AC #20.

### ISS-009 — Past assessments silently rewritten by a new rubric (strict-redo pass)
If the rubric isn't pinned, every historical review changes meaning on a rubric bump. Resolved: §1 #17 — `rubric_version` pinned per assessment; AC #22.

### ISS-010 — In-place edits would destroy the GENIE-vs-human diff (strict-redo pass)
The draft-vs-decision diff is the most valuable fairness/rubric-learning artifact. Resolved: §1 #18 — scores append-only-on-finalize; a `change` writes a revision (`revision_of`) and preserves `genie_score`; AC #23.

### ISS-011 — Silent overrides hide whether rubric/model/reviewer is at fault (strict-redo pass)
A high override rate without reasons is undiagnosable. Resolved: §1 #19 — non-empty `reviewer_rationale` required on `change`/`reject` → `422 rationale_required`; the override rate becomes a first-class signal (§1 #20 metric); AC #24.

### ISS-012 — Rubber-stamping around a subject's rebuttal (strict-redo pass)
A written disagreement that can be approved-around is theater. Resolved: §1 #9 — a pending rebuttal hard-blocks `approved` until `rebuttal_considered=true`; AC #11 #12.

## §3 — Resolution

All 12 concerns resolved. The depth is bounded by the genuine safety surface of an employee-evaluation engine (mandatory-human gate × citation invariant × consent gate × no-score fallback × ZDR routing × audit chain × fairness blinding × appeal × rubric pinning × revision history × override rationale), not by a line target, per task-audit skill §0 master rule.

The `## AI Risk Assessment` section is present and load-bearing (not perfunctory): Data Sources names the captured platform interactions + the three signed documents and excludes private life/keystroke/screen; Human Oversight names the mandatory reviewer (Article 14), the subject rebuttal, the second-reviewer appeal, and the audit chain; Failure Modes maps hallucination/offline/bias each to the human-review fallback with "no change/no score" as the safe state. Cross-references to TASK-EVAL-001 (consent/access), TASK-EVAL-002 (rubric), TASK-MEMORY-123 (evidence), TASK-CUO-204/ai-gateway (GENIE), TASK-AUTH-003 (RLS), TASK-PROJ-008 (chain pattern), and TASK-EVAL-004 (consumer) are present and directionally correct (depends_on [TASK-MEMORY-123, TASK-EVAL-002]; blocks [TASK-EVAL-004]).

**Score = 10/10.** Eligible to move `draft → ready_to_implement` on Stephen's high-risk sign-off (the FR's own §11 + AI Authorship note require it before leaving draft).

---

*End of TASK-EVAL-003 audit.*
