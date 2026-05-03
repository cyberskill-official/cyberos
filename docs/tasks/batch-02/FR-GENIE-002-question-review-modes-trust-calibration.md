---
title: "GENIE / CUO — activate Question and Review modes, trust calibration, acceptance-rate auto-pause"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Flip on the **Question** and **Review** modes that FR-GENIE-001 wired but feature-flagged off, ship the **trust-calibration** machinery (per-persona confidence threshold, per-Member acceptance-rate tracking, 7-day rolling acceptance metric, **auto-pause** when the rolling rate falls below threshold for 7 consecutive days), and ship the **founder kill-switch** that pauses CUO globally in 30 seconds. Together with FR-GENIE-001 (Notify mode + base personas), this completes the three-mode interaction model from PRD §6.5 and the trust-calibration controls from PRD §6.4 — the pre-conditions for the P1 → P2 exit gate criterion (PRD §14.2.3): "Genie acceptance rate (across Notify/Question/Review) is ≥ 40% on a 7-day rolling window for at least 14 consecutive days."

## Problem

A persona that never asks and never drafts is a low-friction Notify-only assistant — useful but capped. A persona that auto-acts on Question/Review without trust calibration is dangerous (PRD §6.5 cites Microsoft Recall as the cautionary tale). The platform's commercial bet (CUO as the brand) requires *both* Question and Review at adequate quality, gated by an acceptance-rate floor that triggers self-pausing rather than waiting for a human to notice the regression. Without auto-pause, a model degradation in production silently erodes trust before anyone files a bug.

The PRD §14.2.3 P1 exit gate is the proximate driver: "Genie acceptance rate ≥ 40% on a 7-day rolling window for at least 14 consecutive days. If < 40% for 7 days, the auto-pause behaviour from Part 6.6.1 has been observed and either resolved or quarantined." The auto-pause must therefore exist by P1 entry — i.e. ship in P0 stabilisation.

## Proposed Solution

The shape of the answer is three modules of behaviour layered on the FR-GENIE-001 substrate: Question-mode authoring + lifecycle, Review-mode authoring + lifecycle, and the trust-calibration loop with auto-pause + kill-switch.

**Question mode lifecycle.** A Question is a CUO-authored ask requiring a human response before downstream action. PRD §6.5 example: "Which contact is the right one to email at Acme — Jane (now CTO) or Ravi (new VP Eng)?".

Lifecycle:

1. **Authoring.** A skill (CEO/COO/CTO) calls `cyberos.genie.ask_question(member_id, prompt, options?, context_ref?, expires_at?)`. The persona-version is recorded; the question lands in the recipient Member's panel "Today" tab as a card with the prompt, the option chips (if any), and a free-text reply box.
2. **Display.** Cards are sorted by created_at; expiring cards (default 7 days) show a countdown chip.
3. **Response.** The Member clicks an option or types a reply. The response is sent to the asking skill via `cyberos.genie.answer_question(question_id, response_text, chosen_option?)`. The asking skill re-runs its decision loop with the response in context and may produce a follow-up Notify, Question, or Review.
4. **Dismissal.** The Member can dismiss without answering; the dismissal counts toward the acceptance-rate denominator. Dismissed-without-answer Questions do not block downstream skill behaviour but are visible in the persona's quality dashboard.
5. **Expiry.** At `expires_at`, an unhandled Question is marked `status: expired` and the asking skill is notified; default skill behaviour is to retry once after a 24-hour silence then drop.

Storage: `genie.question` table with `id, tenant_id, member_id, persona_version, asking_skill, prompt, options, context_ref, status, response_text, chosen_option, created_at, responded_at, expires_at`.

**Review mode lifecycle.** A Review is a CUO-authored long-form draft surfaced for human approval before any side-effect. PRD §6.5 example: a draft cycle-review or a draft client status update.

Lifecycle:

1. **Authoring.** A skill calls `cyberos.genie.draft_review(member_id, title, draft_body, attached_actions?, context_ref?)`. The draft is rendered in the panel "Today" tab as an expandable card; clicking opens the full draft in a side-by-side view (draft on left, citation panel on right showing every BRAIN fact + Layer 3 source the draft cites).
2. **Inline edit.** The user can edit the draft inline. Edits are tracked; the diff is preserved in the audit log.
3. **Approve & execute.** The user clicks "Approve & send" / "Approve & post" / "Approve & save"; the `attached_actions` (e.g. send a CHAT message, post to a project task as an update, save to a KB page) are executed as separate destructive-confirmation tool calls (the actions are *not* implicit in the approval — each action is a standalone confirmable step).
4. **Approve only.** The draft is saved as a finalised Review with `status: approved` but no side-effect taken.
5. **Discard.** Discards count in the acceptance-rate denominator; the audit row records the discard reason if the user provides one.

Storage: `genie.review` table with the same shape as `genie.question` plus `draft_body, edited_body, attached_actions, approved_at, executed_at, discard_reason`. Citation mappings are stored in `genie.review_citation` for forensic reproducibility.

**Trust calibration — the loop.** A small consumer service `cyberos-genie-calibrator` subscribes to `cyberos.{tenant}.genie.{notify|question|review}.{posted|accepted|dismissed|expired|discarded}` events. For every (persona_version, mode, member, day) tuple, it computes the acceptance rate:

- Notify: `accepted / (accepted + dismissed + expired)`
- Question: `(answered + chosen) / (answered + chosen + dismissed + expired)`
- Review: `(approved + edited_then_approved) / (approved + edited_then_approved + discarded)`

The 7-day rolling per-persona-version acceptance rate is published as a Prometheus metric `cyberos_genie_acceptance_rate{persona_version, mode}` and surfaced in the OBS persona-quality dashboard.

**Auto-pause.** A persona-version whose 7-day rolling acceptance rate stays below 40% for **seven consecutive days** is automatically paused:

1. The calibrator emits `cyberos.{tenant}.genie.persona.auto_paused{persona_version}`.
2. The AI Gateway's persona-loader is updated: requests for the paused persona_version are rejected with `code: "PERSONA_AUTO_PAUSED"` and a fallback to the previous signed persona version is attempted (if one exists and is not also paused).
3. The Genie panel shows a banner "Genie is paused: acceptance rate below threshold. Founder review required."
4. A high-priority Notify card is sent to the founder + Engineering Lead with the metrics and a deep link to the calibration dashboard.
5. The audit log captures the pause event in scope `genie.calibration.{tenant}`.

A paused persona is reactivated only by the founder + Engineering Lead dual-sign: rerun of the eval suite + explicit unpause command. Reactivating a paused persona without resolving the root cause is itself audit-logged with an explanatory note required.

**Confidence threshold per persona.** Each persona's `SKILL.md` declares its retrieval-confidence threshold (FR-GENIE-001 default: CEO 0.65, COO 0.55, CTO 0.6). Below threshold the persona must default to Question mode rather than Notify. This forces low-confidence answers into a clarifying loop rather than a low-quality Notify.

**Per-Member preferences.** A Member can adjust their notify-density: "review-everything" (force Review for any non-trivial CUO write), "ask-more" (force Question for low-confidence Notify), "default" (the persona-defined confidence floors apply), or "minimum" (suppress Notify cards below high-confidence). The preference is stored in `genie.member_preference` and respected by the calibrator + the panel renderer.

**Founder kill-switch.** A founder-only command `cyberos genie pause-all <reason>` (or panel button "Pause Genie globally") sets `cyberos_meta.persona_version_active.global_paused: true` for the tenant. The AI Gateway respects this within 30 seconds across all replicas (the value is broadcast on NATS `cyberos.{tenant}.genie.global_pause` and watched by every gateway replica). The kill-switch is itself audit-logged in scope `genie.kill_switch.{tenant}`. Resuming is symmetric (`cyberos genie resume <reason>`) and also audit-logged. The kill-switch is the architectural escape hatch when a regression is observed and the on-call cannot wait for the auto-pause threshold.

**Persona regression eval suite.** Every persona-version PR runs an eval suite of ≥ 60 curated cases per skill (FR-GENIE-001 §"Persona regression test suite"). The eval cases include:

- Citation-correctness cases (the persona must cite exactly the right BRAIN fact).
- Scope-adherence cases (the persona must refuse out-of-scope tool calls with a structured response).
- Voice-adherence cases (the persona must answer in the declared voice register).
- Safety cases (synthetic prompt-injections testing whether the persona escapes its scope).
- Vietnamese-locale cases (the persona must answer in vi-VN when the prompt is Vietnamese).

A regression on any case category blocks the PR. Pass-rate target: ≥ 95% on every category for every persona-version that goes to production.

**Mascot animation states.** Per PRD §13.2.2, the panel header mascot (the genie/lamp from CyberSkill's logo) cycles through animation states: idle, thinking, answering, error, deferring, paused (new in this FR — visually distinct so the user sees the global pause). Lottie files are added to the design-tokens package (FR-DESIGN-001 in this batch); they are static placeholders in P0 and full animations in P2 per the PRD's animation roadmap.

**MCP tool surface (added on top of FR-GENIE-001).**

- `cyberos.genie.ask_question(member_id, prompt, options?, context_ref?, expires_at?)` — read.
- `cyberos.genie.answer_question(question_id, response_text, chosen_option?)` — read.
- `cyberos.genie.draft_review(member_id, title, draft_body, attached_actions?, context_ref?)` — read.
- `cyberos.genie.approve_review(review_id, edited_body?)` — `destructive: true; requires_confirmation: true`.
- `cyberos.genie.discard_review(review_id, reason?)` — `destructive: false`.
- `cyberos.genie.persona_pause(persona_version, reason)` — founder-only; `destructive: true; requires_confirmation: true`.
- `cyberos.genie.persona_resume(persona_version, reason)` — founder + engineering-lead; dual-sign required.
- `cyberos.genie.global_pause(reason)` / `cyberos.genie.global_resume(reason)` — founder-only kill-switch.
- `cyberos.genie.acceptance_rate(persona_version?, member_id?, mode?, since?, until?)` — read; for dashboards.

The CUO personas can call `ask_question` and `draft_review` (drafting is fine); they cannot call `approve_review` (that requires the human) or any pause/resume tool.

## Alternatives Considered

- **No auto-pause; alert only.** Rejected: a regression that stays below the threshold for a week is *already* a customer-experience problem; the pause is a feature for the user, not the operator.
- **Auto-pause on a single-day acceptance dip.** Rejected: too sensitive; one busy day where the user dismisses everything triggers a pause and the next day proves it was noise.
- **Per-Member personas.** Rejected for P0: persona personalisation is a P2 deliverable; per-Member preference (the notify-density slider) is the P0 floor.
- **A separate "Audit" mode in addition to Notify/Question/Review.** Rejected: Audit is not an interaction mode; it's a read surface served by OBS + the persona-quality dashboard.

## Success Metrics

- **Primary metric.** S0-5 → S0-6 demo passes: (1) the COO skill produces a Question card on a CHAT scheduling-discussion pattern; the founder responds; the COO follow-up Notify includes the response; (2) the CEO skill drafts a weekly Review summary; the founder edits and approves; the citation panel shows correct provenance to BRAIN; (3) a synthetic acceptance-rate drop to 30% over 7 days triggers auto-pause; (4) the founder uses the kill-switch and Genie pauses within 30 seconds across all gateway replicas.
- **Acceptance metric.** Combined acceptance rate ≥ 40% on a 7-day rolling window across the 10 employees by P1 → P2 (PRD §14.2.3); P0 measures and surfaces the metric without enforcing the gate.
- **Kill-switch latency.** ≤ 30 seconds from command to active enforcement at every gateway replica (NFR-PERF-GENIE-002).
- **Eval-suite floor.** ≥ 95% pass on every category on every persona-version that ships to production.

## Scope

**In-scope (S0-5 + S0-6).**
- Question mode end-to-end with the `genie.question` table and lifecycle.
- Review mode end-to-end with the `genie.review` table, inline editor, and citation panel.
- `cyberos-genie-calibrator` consumer publishing acceptance metrics + auto-pause + persona-quality dashboard.
- Founder kill-switch with NATS-broadcast 30-second propagation.
- Per-Member preference UI in `/auth/account` settings.
- Persona regression eval suite extended to 60 cases per skill across the five categories.
- Mascot animation states with the new "paused" state.
- The MCP tools listed above with persona-scope contract enforcement.
- Audit integration in scopes `genie.question.{tenant}`, `genie.review.{tenant}`, `genie.calibration.{tenant}`, `genie.kill_switch.{tenant}`.

**Out-of-scope (deferred).**
- The remaining seven C-level skills (CFO/CMO/CHRO/CSO/CLO/CDO/CPO) — P1 and P2.
- Per-Member persona personalisation beyond the density slider (P2).
- Voice mode for Question / Review (P3 mobile).
- Cross-Member Review workflows (one Member drafts, another approves) — P2.

## Dependencies

- FR-INFRA-001 (host shell + NATS).
- FR-AUTH-001 / FR-AUTH-002.
- FR-AI-001 (persona-version stamping + AI Gateway pause behaviour).
- FR-MCP-001 (destructive-confirmation; persona-scope contract for the new tools).
- FR-BRAIN-001 / FR-BRAIN-002 (citation provenance for Reviews).
- FR-BRAIN-NLCRUD-001 (Reviews can attach NLCRUD-derived memory writes as `attached_actions`).
- FR-GENIE-001 (substrate this FR builds on).
- FR-OBS-001 / FR-OBS-002 (persona-quality dashboard).
- Compliance: EU AI Act Article 14 (human oversight: Question and Review *are* the oversight controls), Article 50 (transparency: every Question and Review carries persona-version + ai_disclosure_id).
- Locked decisions referenced: DEC-045 (three-mode interaction model), DEC-059 (auto-pause at 40% / 7 consecutive days), DEC-060 (founder kill-switch with 30-second NATS-broadcast propagation).

## AI Risk Assessment

This FR is the activation of the highest-risk CUO surfaces (drafts that affect downstream actions). EU AI Act risk class: `limited`.

### Data Sources

The Question and Review drafting prompts run through the AI Gateway; per-tenant residency; persona-version stamping; no third-party training data. Drafts cite BRAIN facts and Layer 3 sources from the same tenant.

### Human Oversight

- Question requires a human response before downstream skill action.
- Review requires explicit approve-or-discard; `attached_actions` are never implicit in the approval — each is a separate destructive confirmation.
- Auto-pause triggers when acceptance falls; the persona stops shipping low-quality drafts before the user has to file a bug.
- Founder kill-switch is the 30-second escape hatch.
- Persona regression eval suite blocks bad releases.

### Failure Modes

- **Sycophantic acceptance.** Members click "approve" without reading drafts; acceptance rate looks healthy but quality is declining. Mitigation: a sampled human-review program (founder + DPO) inspects 5% of approved Reviews weekly; quality drift surfaces as a separate metric.
- **Auto-pause oscillation.** A persona crosses below 40% one day, recovers, drops again. Mitigation: the seven-consecutive-days requirement smooths this; below-threshold-but-not-paused state is itself an alert.
- **Kill-switch propagation gap.** A gateway replica misses the NATS event during a network partition. Mitigation: the gateway also polls the canonical `cyberos_meta.persona_version_active` table every 10 seconds as a backstop.
- **Eval-suite regression.** A new persona-version regresses on a category. Mitigation: PR-blocking; the persona-version is rejected and the prior version remains active.
- **Member preference abuse.** A Member sets "minimum" to suppress Notify but later complains the platform missed a critical alert. Mitigation: critical alerts (sev-0/sev-1 from OBS) bypass the preference; the preference governs persona-driven Notifies only.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted Question + Review lifecycles, calibrator semantics, auto-pause flow, kill-switch propagation, eval-suite categories, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; persona-version pause-resume audit semantics to be re-verified by the Engineering Lead.
