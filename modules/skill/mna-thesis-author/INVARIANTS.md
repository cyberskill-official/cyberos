# `mna-thesis-author` — invariants

These invariants are checked at every node boundary, every 25 audit rows, and on completion. A breach emits a `refinement_proposal` and pauses the pipeline.

| id | invariant | rationale |
|---|---|---|
| INV-001 | Every artefact file written has a corresponding manifest entry with matching `artefact_hash`. | Re-entrancy guarantee. |
| INV-002 | Every `artefacts[X].status` transition (DRAFTING → PASS / HITL_PAUSE / EXHAUSTED) is reflected in exactly one `genie.action_log` row. | Audit-trail completeness. |
| INV-003 | No artefact is written outside `output_dir`. | Scope sandbox. |
| INV-004 | No HITL question is re-asked once its `resolution` is non-null. | User trust. |
| INV-005 | No two artefacts are generated concurrently. | Determinism + audit clarity. |
| INV-006 | Every claim in an artefact body has an `authority` marker. | AGENTS.md §5.1 compliance. |
| INV-007 | Every source file is read inside an `<untrusted_content>` block before any reasoning. | Prompt-injection defence. |
| INV-008 | `confidence_band.default` is honoured — claims below `defer_below` trigger HITL escalation. | Trust calibration. |
| INV-009 | Manifest is written after every state transition (not batched). | Crash recovery. |
| INV-010 | `source_hash` is recomputed on every invocation and compared against the manifest's last-known value. Drift surfaces as `INPUTS_CHANGED`. | Source-tracking. |

## Anomaly signals (frontmatter `self_audit.anomaly_signals`)

| signal | trigger | meaning |
|---|---|---|
| `confidence_low_streak` | 3 claims below `defer_below` within a 10-claim window | model is uncertain in this domain; surface for fine-tune |
| `user_correction_streak` | 2 user corrections within 5 turns | user disagrees with model output; recalibrate |
| `denylist_near_miss_streak` | 2 near-misses of the content denylist within 20 turns | content gate is too loose or too tight |
| `scope_rejection_streak` | 1 BRAIN scope rejection | the skill is requesting BRAIN data it shouldn't |
| `citation_missing_streak` | 2 claims without `source_ref` within 10 claims | anti-fabrication discipline slipping |

A breach of any signal emits a `refinement_proposal` and pauses the pipeline pending operator review.
