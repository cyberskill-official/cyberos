---
title: "OKR — quarterly cycle workflow, CUO/CEO + CSO-skill review, alignment heatmap, cycle-close drafts"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q2"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the quarterly OKR cycle workflow (PRD §9.19, §14.3.1): **cycle planning** (founder kickoff → company-level objectives → cascade); **mid-cycle check-ins** (weekly Member + monthly team-level + monthly company-level); **CUO/CEO + CSO-skill review** — at quarter-mid + quarter-close, a CUO-drafted strategic-alignment review surfaces "what's on track / what's at risk / what's misaligned"; the **alignment heatmap** visualisation showing per-team coverage of company objectives + per-Member coverage of their team's objectives; **cycle-close drafts** (CUO drafts the cycle-review narrative for the founder's edit-and-publish, including notable wins, missed KRs with rationale, learnings, recommended next-cycle adjustments). All AI surfaces operate read-only on OKR + cross-module data; the persona-scope contract excludes any tool that mutates OKR commitments. PRD §14.3.2 P2 → P3 exit gate: "OKR cycle close has been completed entirely inside OKR module for at least 1 quarter."

## Problem

OKR adoption fails when the cycle becomes ceremonial — KRs set + not checked in + reviewed only at year-end. Three failure modes:

- **Setup decay.** Without a structured kickoff + cascade flow, the founder spends 3 days each quarter writing OKRs by hand; team leads scramble to align by guessing.
- **Mid-cycle invisibility.** Without weekly + monthly check-ins, "are we on track" is unanswerable until close.
- **Cycle-close opacity.** Without a CUO-drafted review, the founder hand-writes the retrospective; learnings are lost across quarters.

## Proposed Solution

The shape of the answer is the cycle workflow + the CUO/CEO + CSO scopes + the heatmap + the close-draft pipeline.

**Cycle workflow phases.**

1. **Pre-cycle kickoff** (last 2 weeks of prior quarter).
   - Founder authors the kickoff narrative + 3-5 company objectives with 3-5 KRs each.
   - CUO/CEO drafts a kickoff narrative based on the prior cycle's review + recent BRAIN-derived signals (recent strategic conversations from FR-CHAT-001 + EMAIL); the founder edits before publishing.
   - Founder signs off the company OKRs.
2. **Cascade phase** (first 2 weeks of new quarter).
   - Team leads cascade team-level objectives aligning to company.
   - Members cascade individual-level objectives aligning to team.
   - Each owner signs off their own; team leads sign off their teams; founder signs off all team-level.
   - CUO/CSO surfaces alignment-coverage warnings: "Team Engineering has 4 objectives but only 1 cascades from a company OKR — review for completeness."
3. **Active phase** (rest of quarter).
   - Weekly individual check-ins (the Member updates `current_value` + confidence + commentary).
   - Monthly team check-ins (the team lead aggregates + comments at team level).
   - Monthly company-level CUO/CEO + CSO review (next section).
4. **Mid-cycle review.**
   - At week 6 (mid-quarter), CUO/CEO + CSO produces a structured mid-cycle review surfaced to the founder + team leads.
5. **Cycle-close.**
   - Final-week check-ins.
   - CUO/CEO + CSO drafts the cycle-close review.
   - Founder edits + publishes; the cycle is closed.

**CUO/CEO + CSO-skill review.**

A new persona surface (alongside the existing CEO + COO + CTO + CFO + CHRO + CRO skills) — the **Chief Strategy Officer** skill (CSO). Combined with the existing CEO skill, the OKR review path consults both.

The mid-cycle + cycle-close review:

- **Inputs.** All cycle's objectives + KRs + check-ins; cross-module signals (PROJ velocity vs. KR-linked-issue count; CRM deal flow vs. KR-linked-deal-amount; OBS metrics vs. KR-linked-metric); company-level financial signals (FR-INV in P2 batch-08 if shipped, else stub); recent decisions from the locked-decisions ledger (FR-CP-001).
- **Output structure.**
  - **Highlights.** What's on track + working well (cite specific KRs).
  - **At-risk.** KRs with `at_risk` or `off_track` status; the *why* (citing check-in commentary + linked-artefact signals); suggested adjustments.
  - **Misalignments.** Patterns where multiple teams have similar KRs that overlap; or where team objectives don't trace cleanly to company; suggestions to consolidate.
  - **Learnings.** Patterns from prior cycles' close-reviews that are relevant.
  - **Recommended adjustments.** For mid-cycle: "consider lowering target on KR-X" or "consider re-cascading team objective Y." Recommendations are advisory; the founder accepts/rejects.
- **Latency.** Pre-computed at the mid-week 6 + final-week of the quarter; on-demand recompute ≤ 10 s p95.

The CSO skill is configured per FR-GENIE-001's persona Skills format (`~/.cyberos/skills/cuo/cso/SKILL.md`); dual-signed at the founder + Engineering Lead.

**Alignment heatmap.**

A `okrAlignmentReport(cycleId)` GraphQL field returning structured data the heatmap renders:

- Rows: company objectives.
- Columns: teams.
- Cell: count of team objectives + their owners + average confidence at last check-in. Cells colour-coded:
  - Green: team has explicit cascading objectives + recent check-ins on-track.
  - Yellow: team has objectives but at-risk on confidence; or has cascading-but-stale check-ins.
  - Red: team has no objective cascading from this company OKR; or all check-ins off-track.
- A second view: per Member's coverage of their team's objectives (similar matrix).

The heatmap is the founder's quarterly artefact; FR-OKR-003 renders it.

**Cycle-close draft pipeline.**

The `okrCloseCycle(id, reviewMd)` mutation requires a `reviewMd`. Before the founder writes one, CUO/CEO + CSO drafts a cycle-review (similar shape to FR-PROJ-006's cycle-review draft):

```
Cycle 2026-Q3 Summary

Highlights:
- Company OKR-1 ("Ship the Acme onboarding flow") fully achieved (4 of 4 KRs).
- Engineering team's velocity KR exceeded target (9.2 cycle-completion vs. 8.0 target).
...

At-risk → Missed:
- Company OKR-3's KR-2 (NPS target ≥ 50) ended at 47. Citing check-ins from
  team Account Management: the customer signal from Acme's quarterly review
  (CRM-Account: acme-corp; Activity: Q3 review meeting) suggested 3 specific
  feature gaps that drove the score lower than projected.
...

Misalignments:
- The "developer experience" theme appeared in 3 team-level objectives without
  a parent company OKR. Recommend either elevating to a company OKR for Q4 or
  consolidating into the existing platform reliability OKR.

Learnings (carried forward):
- Cycle 2026-Q2 surfaced the same "ambitious-then-overshoot" pattern on KR-A;
  Q3 we deliberately set lower targets and exceeded them. Continue this calibration.

Recommended next-cycle adjustments:
- Promote "developer experience" to a company OKR for Q4.
- Re-base NPS target to 45 with a plan to close the 3 specific gaps.
- ...
```

The draft cites: per-KR check-ins, PROJ + CRM + OBS signals, prior cycle reviews. The founder edits + signs + publishes; the cycle is closed.

**Persona scope contract.**

CUO/CEO + CUO/CSO declare for the OKR-review path:
- `tools_allowed`: `cyberos.okr.*` (read), `cyberos.proj.*` (read), `cyberos.crm.*` (read), `cyberos.obs.list_metrics` (read), `cyberos.brain.*` (read), `cyberos.genie.draft_review`, `cyberos.genie.notify`.
- `tools_forbidden_explicit`: any OKR mutation tool (`cyberos.okr.create_*`, `cyberos.okr.update_*`, `cyberos.okr.close_cycle`, etc.); any cross-module mutation; compensation reads; equity reads.

**MCP tool surface (read-only; AI tools).**

- `cyberos.okr.draft_kickoff(cycle_id)` — read; pre-cycle CUO/CEO kickoff draft.
- `cyberos.okr.alignment_warnings(cycle_id)` — read; cascade-coverage warnings.
- `cyberos.okr.draft_mid_cycle_review(cycle_id)` — read; mid-cycle CUO/CEO + CSO review.
- `cyberos.okr.draft_close_review(cycle_id)` — read; cycle-close CUO/CEO + CSO draft.
- `cyberos.okr.alignment_heatmap(cycle_id)` — read; the structured heatmap data.
- `cyberos.okr.list_at_risk_krs(cycle_id, owner_id?)` — read.

CUO uses these internally; the human commits any state change.

**Latency budgets.**

- Kickoff draft: pre-computed; ≤ 10 s p95 on demand.
- Alignment warnings: ≤ 4 s p95.
- Mid-cycle review: pre-computed; ≤ 12 s p95 on demand (longer; complex retrieval).
- Close review: pre-computed at quarter-end; ≤ 15 s p95.
- Heatmap: ≤ 600 ms p95 (deterministic; cache-friendly).

**Notify integrations.**

- Founder: kickoff-draft-ready; mid-cycle review ready; close-draft ready.
- Team leads: cascade phase reminder; weekly check-in reminder; monthly team-review-due reminder.
- Members: weekly individual check-in reminder.
- Cycle-close: alignment heatmap + close review delivered to founder + team leads.

## Alternatives Considered

- **Skip the AI review; founder hand-writes.** Rejected: PRD §9.19 + §14.3.1 explicitly name "CUO/CEO + CSO-skill review."
- **Use Lattice OKRs / Mooncamp.** Rejected: residency + cross-module linkage + CUO persona stamping; same as FR-OKR-001.
- **AI auto-publishes the cycle review.** Rejected: founder authorship is the floor.
- **Single review at cycle-close only.** Rejected: mid-cycle review is the leverage point for adjustments.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate (PRD §14.3.2): "OKR cycle close has been completed entirely inside OKR module for at least 1 quarter."
- **Quality metric.** Cycle-close draft acceptance: founder edits + publishes ≥ 80% of CUO-drafted reviews with ≤ 30 minutes of editing time.
- **Adoption metric.** 100% of active Members complete weekly check-ins on their KRs for ≥ 75% of weeks in the cycle.
- **Latency NFR.** Per the budgets above.

## Scope

**In-scope.**
- 5-phase cycle workflow.
- CUO/CEO + CSO-skill personas (CSO is new in this FR).
- Alignment-coverage warnings.
- Mid-cycle + cycle-close review draft pipeline.
- Alignment heatmap data primitive.
- Notify cadence per role.
- Persona scope contract enforcement.
- The 6 MCP tools.
- Audit integration in scope `okr.review.{tenant}`.

**Out-of-scope (deferred).**
- Frontend rendering of heatmap + reviews (FR-OKR-003).
- Multi-cycle trend visualisation (P3).
- Auto-published cycle review (deliberately not — founder authorship is the floor).
- CSO emergent-role expansion to broader strategy work beyond OKR (P3 — covers founder strategic-thinking time more broadly).

## Dependencies

- FR-OKR-001.
- FR-PROJ-001 / FR-PROJ-007 (linked-artefact signals).
- FR-CRM-001 (crm_deal_amount KR sources).
- FR-OBS-001 / FR-OBS-002 (obs_metric KR sources).
- FR-BRAIN-001 / FR-BRAIN-002 (cycle-history retrieval; learnings from prior reviews).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify/Question/Review modes; CUO/CEO + CSO personas).
- The signed Total Rewards Appendix (the founder kickoff cites strategic context).
- Compliance: EU AI Act Article 50 (review drafts render disclosure chip); Article 14 (human-in-the-loop on cycle close).
- Locked decisions referenced: DEC-221 (CSO is a new emergent C-skill in P2), DEC-222 (CUO drafts cycle reviews; founder authors the canonical), DEC-223 (mid-cycle review at week 6).

## AI Risk Assessment

The cycle reviews + alignment warnings + kickoff drafts are AI surfaces visible to natural persons making strategic decisions. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: OKR data + PROJ + CRM + OBS + BRAIN. CSO + CEO personas run through the AI Gateway with persona-stamping. No third-party data; no cross-tenant.

### Human Oversight

- Founder authors the canonical cycle review.
- CUO drafts; the human commits.
- Alignment warnings are advisory; the team lead decides.
- Notify cadence is informational.
- The kill-switch from FR-GENIE-002 silences the OKR AI surfaces.

### Failure Modes

- **Review hallucinates a KR signal.** Caught by citation-correctness regression suite; the cited KR + check-in must support the claim.
- **Alignment warning false-positive.** The team lead can dismiss with documented justification; pattern feeds persona tuning.
- **Notify cadence overwhelms.** Members can adjust per-event preferences (FR-PROJ-010 pattern reused).

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted cycle workflow, persona scope, review draft architecture, heatmap data primitive, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the CSO persona's first published version + the first cycle review will be hand-validated by the founder.
