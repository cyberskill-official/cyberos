# HITL protocol — `HITL_BATCH_REQUEST` format

Version: 1.0.0  Status: Normative for every skill in the SKILL module.

This file is copied verbatim into every skill bundle. Customize `hitl_categories` only.

---

## §1  When to pause

The skill MUST pause and emit a `HITL_BATCH_REQUEST` when:

- The rubric assigns `→ needs_human` to any open issue.
- A field cannot be derived from sources alone (anti-fabrication discipline).
- A compliance boundary is reached (EU AI Act risk-class transition, GDPR / Vietnam Decree 13/2023 PDPD data class transition, OWASP A06 design-flaw inference).
- `confidence_band.default` drops below `defer_below`.

## §2  Pause categories (customize per skill)

| category | meaning |
|---|---|
| `customer_quotes` | Quote attribution unclear or unsourced. |
| `ai_act_risk_boundary` | EU AI Act risk class cannot be determined from inputs. |
| `success_metric_targets` | Numeric target without a citable source. |
| `cross_team_dependency` | Dependency on another team/module without ticket/owner/commitment. |
| `legal_compliance` | Article 5 / prohibited-practice trigger; legal review required. |
| `scope_decomposition` | Backlog item is too large to author as one artefact. |
| `stale_artefact_disposition` | Source hash drift — operator decides whether to revert or proceed. |

Each skill SHALL declare its supported categories in `SKILL.md` CONTRACT_ECHO.

## §3  Block format

```
HITL_BATCH_REQUEST
batch_run_id: <uuid>
skill_id:     rtm-author
total_paused: N

issue 1:
  artefact_id:   RTM-001
  category:      success_metric_targets
  rule_id:       QA-NUM-001
  question:      "Section §3.2 cites a 25% retention target. No source line provides this number. What is the source?"
  context:       <surrounding 2-3 lines from the artefact>
  required_form: free_text | choice[a,b,c] | numeric | datetime
  blocking:      true

issue 2:
  artefact_id:   RTM-002
  category:      ai_act_risk_boundary
  rule_id:       QA-001
  question:      "The artefact mentions 'biometric identification' but eu_ai_act_risk_class is set to minimal. Which class applies?"
  context:       <surrounding 2-3 lines>
  required_form: choice[minimal,limited,high]
  blocking:      true

issue 3:
  artefact_id:   RTM-003
  category:      cross_team_dependency
  rule_id:       QA-008
  question:      "Dependency on the data-pipeline team named without ticket or owner. Provide ticket ID + owner handle, or remove."
  context:       <surrounding 2-3 lines>
  required_form: free_text
  blocking:      false   # the artefact can ship without this resolved (warning-only), but operator review encouraged
```

## §4  Reply format

The user replies with one of:

- `RESOLVE issue <N>: <answer>` — provides the answer, marks the issue resolved.
- `REVISE issue <N>: <re-ask>` — asks the skill to reformulate the question.
- `DEFER issue <N>` — leaves the issue open; skill writes the artefact with TODO marker.
- `ABORT batch` — aborts the entire batch; manifest rolled back to pre-PLAN state.

The skill SHALL parse the reply, apply each resolution, and re-enter RESUME phase.

## §5  Re-ask prevention

The skill MUST NEVER re-ask a HITL question whose `resolution` is non-null. The reply parser sets `resolution` on each answered issue; subsequent runs skip those issues entirely.

## §6  Aggregation discipline

When multiple artefacts in a batch each have HITL issues, the skill aggregates ALL issues into ONE `HITL_BATCH_REQUEST` block at the LAST position in the response. Operators answer once per batch, not once per artefact. This satisfies the "halt batch on HITL" policy from the skill's CONTRACT_ECHO.

## §7  Cross-references

- AGENTS.md §11 (memory module) — untrusted-content discipline that informs the question wording.
- `references/ANTI_FABRICATION.md` (sibling file) — when to escalate vs. when to author with what you have.
- The matching audit skill's `RUBRIC.md` — every `→ needs_human` rule.
