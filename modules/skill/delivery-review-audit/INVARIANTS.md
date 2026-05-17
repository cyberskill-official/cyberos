# `delivery-review-audit` — invariants

These invariants are checked at every node boundary, every 25 audit rows, and on completion. A breach emits a `refinement_proposal` and pauses the pipeline.

| id | invariant | rationale |
|---|---|---|
| INV-001 | Every audit report has `audit_template_version` matching this skill's declared `rubric_version`. | CONTRACT_DRIFT defence. |
| INV-002 | Every issue has a `rule_id` that exists in `RUBRIC.md`. | Anti-fabrication for audit rule citations. |
| INV-003 | No audit report is written outside the parent of any `artefact_path`. | Scope sandbox. |
| INV-004 | No HITL question is re-asked once its `resolution` is non-null. | User trust. |
| INV-005 | No two artefacts are audited concurrently. | Determinism. |
| INV-006 | Two runs against the same `audited_file_sha256 + rubric_version` produce byte-identical reports modulo timestamp fields enumerated in `REPORT_FORMAT.md`. | **deterministic_drift** invariant. |
| INV-007 | Every audit report's `iterations` count is ≤ `max_iterations`. | Termination guarantee. |
| INV-008 | Every `auto_fix_applied: true` issue has a non-empty `diff_hunk`. | Audit-trail completeness. |
| INV-009 | Every `needs_human` issue has a non-empty `category` from this skill's declared `hitl_categories`. | HITL routing correctness. |
| INV-010 | `confidence_band.default` is honoured — verdicts below `defer_below` trigger HITL escalation. | Trust calibration. |

## Anomaly signals (frontmatter `self_audit.anomaly_signals`)

| signal | trigger | meaning |
|---|---|---|
| `confidence_low_streak` | 3 verdicts below `defer_below` within a 10-verdict window | the rubric is uncertain in this domain; surface for fine-tune |
| `user_correction_streak` | 2 user corrections within 5 turns ("this rule is wrong") | rubric needs revision |
| `rule_reversal_streak` | 1 case where a previously PASSED artefact is later corrected to FAIL | rubric has a false-negative; investigate |
| `needs_human_rate_above` | >50% of artefacts pause for HITL within a 10-artefact window | rubric is asking too much of operators; tune thresholds |
| `deterministic_drift` | any case where same artefact + same rubric → different verdicts | **catastrophic — pause immediately** |

A breach of any signal emits a `refinement_proposal` and pauses the pipeline pending operator review.

The `deterministic_drift` signal is the most serious — it indicates the audit skill is non-deterministic, which violates a core protocol guarantee. The skill SHALL pause immediately on the first occurrence and demand operator intervention.
