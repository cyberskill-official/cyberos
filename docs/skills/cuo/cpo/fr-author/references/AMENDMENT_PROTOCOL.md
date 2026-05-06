# Amendment protocol (PLAN_AMENDMENT_REQUEST + batch aggregation)

> Sourced from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §10.6, §10.7, §6.7.

Amendments are mid-flight changes to the approved backlog (add / split / merge / reorder / reclassify FRs). Unlike HITL pauses, amendments do NOT halt the batch — they accumulate and emit one consolidated request at batch end.

## Record schema (PLAN_AMENDMENT_REQUEST — never emitted standalone post-v2.0.0)

```
PLAN_AMENDMENT_REQUEST
amendment_id:         AMD-NNN
trigger_fr:           FR-NNN
proposed_change:      ADD | SPLIT | MERGE | REORDER | RECLASSIFY
risk_class:           low | medium | high
proposed_backlog_diff:
  + FR-NNN+1  <new title>  <one-liner>
reason:               <2-4 sentences citing source_refs>
options:
  A) APPROVE_AMENDMENT  — apply (inline if low; halt-and-replan if medium/high)
  B) REJECT_AMENDMENT
  C) PAUSE_FOR_REVIEW
END_PLAN_AMENDMENT_REQUEST
```

## risk_class table

| `proposed_change` | `risk_class` | Rationale |
| --- | --- | --- |
| RECLASSIFY (one FR's tentative fields only) | low | Editorial; no graph topology change. |
| REORDER (within a single priority band) | low | Topological order preserved. |
| REORDER (across priority bands) | medium | Affects shipping order. |
| ADD (no current-batch dep) | medium | Scope grew. |
| ADD (with depends_on on UNCLAIMED FR in this batch) | **high** | Triggers `AMENDMENT_DEP_BREAKS_BATCH` exception in WORKER. |
| SPLIT | medium | Topology change. |
| MERGE | medium | Topology change. |

## PLAN_AMENDMENT_BATCH_REQUEST

Aggregates all `amendments_pending[]` entries with `resolution = null`. Maps to the CyberOS Review primitive (SRS §6.6.3) — the human reviews multiple proposed changes and approves/edits/rejects each.

```
PLAN_AMENDMENT_BATCH_REQUEST
====================
manifest_path:                 <output_dir>/manifest.json
plan_approval_hash:            <hex>
total_pending_amendments:      <int>
risk_class_distribution:       {low: <int>, medium: <int>, high: <int>}
amendment_stats_summary:       <ratio> (threshold <ratio_threshold>)

[AMD-NNN]  trigger_fr: FR-NNN  proposed_change: <type>  risk_class: <class>
  reason: <2-4 sentences>
  proposed_backlog_diff:
    + FR-NNN+1  <new title>
  options:
    A) APPROVE_AMENDMENT
    B) REJECT_AMENDMENT
    C) PAUSE_FOR_REVIEW

(repeat per AMD)

------------------------------------------------------------
How to answer
------------------------------------------------------------
Reply with ONE LINE per amendment, using the exact format:

  AMD-NNN: <letter>

When all <int> amendments are answered, re-invoke fr-author.
END_PLAN_AMENDMENT_BATCH_REQUEST
```

Emitted as the LAST thing in the response when amendments are pending. If both HITL pauses AND amendments are pending, emit HITL_BATCH_REQUEST first, then PLAN_AMENDMENT_BATCH_REQUEST.

Each emission appends one `genie.action_log` row with `row_kind: review` (per SRS §6.6.3 + §6.7).

## Inline-amendment apply (low-risk only — §6.7)

When the human's reply contains `AMD-NNN: A` (APPROVE_AMENDMENT):

- **If `risk_class: low`** (RECLASSIFY only, OR REORDER inside the SAME priority band) → apply the diff inline to `manifest.plan.backlog`, recompute `manifest.plan.approval_hash`, set `amendments_pending[i].resolution = "applied:<timestamp>"`, log to `BATCH_RUN_LOG.md` as `AMENDMENT_APPLIED_INLINE`, and continue WORKER without a fresh PLAN render.
- **If `risk_class: medium` or `high`** → set `manifest.plan.status = AMENDED_AWAITING_APPROVAL`, write the manifest, HALT. Next invocation re-renders the plan-approval block with the amended backlog and requires fresh `APPROVE`.

For `AMD-NNN: B` (REJECT): set `resolution = "rejected:<timestamp>:<reason>"`. Backlog unchanged.

For `AMD-NNN: C` (PAUSE_FOR_REVIEW): set `resolution = null` and `manifest.plan.status = AMENDED_AWAITING_APPROVAL`. HALT.

Inline apply MUST NOT touch any FR whose `status = PASS` unless that FR is the explicit target of a RECLASSIFY (changing its `tentative_*` fields). RECLASSIFY of a PASS FR additionally marks it `status = STALE` and surfaces a `stale_fr_disposition` HITL issue per `MANIFEST_SCHEMA.md` §3.2 step 2.
