# Pipeline — `fr-create` → `fr-audit` (and beyond)

> The split of v2.0.0's monolithic FR_CREATE_AND_AUDIT prompt into two CyberOS skills exists precisely so they can be chained, run independently, or composed with other skills. This file shows three realistic chains.

## Chain 1 — Create then audit (the most common case)

This is the original v2.0.0 `create_and_audit` mode, expressed as a two-skill chain.

```
                    ┌─────────────────┐                ┌─────────────────┐
   PRD/spec docs ──▶│  cuo/cpo/       │  FR markdowns  │  cuo/cpo/       │
                    │  fr-create      │ ──────────────▶│  fr-audit       │
                    │  v0.1.0         │  (NATS event:  │  v0.1.0         │
                    └─────────────────┘  cuo.fr_create.└─────────────────┘
                            │             fr_written)         │
                            ▼                                 ▼
                    fr-manifest@2                     <fr-path>.audit.md
                    + FR-NNN-*.md files               + AUDIT_BATCH_SUMMARY
                    + genie.action_log                + genie.action_log
```

**Wiring at the LangGraph level (SRS §6.1.1):**

```python
# CUO supervisor graph (illustrative — the real graph lives in genie.persona_config)
graph = StateGraph(CuoState)
graph.add_node("fr-create", fr_create_skill)
graph.add_node("fr-audit",  fr_audit_skill)

# Conditional edge: chain only when fr-create's output envelope says so
graph.add_conditional_edges(
    "fr-create",
    lambda state: "fr-audit" if state.last_output.next_skill_recommendation == "cuo/cpo/fr-audit" else END,
)

# fr-audit edges to either END or back to fr-create on a STALE-001 escalation
graph.add_conditional_edges(
    "fr-audit",
    lambda state: "fr-create" if state.last_output.requires_regen else END,
)
```

**State persistence:** every node transition is checkpointed to `genie.graph_checkpoint` (per SRS §6.1.1). A worker crash mid-audit means the next invocation resumes the audit cleanly without re-running `fr-create`.

## Chain 2 — Audit-only (existing FRs from any source)

```
                    ┌─────────────────┐
   FR markdown    ──▶│  cuo/cpo/       │ ──▶  <fr-path>.audit.md
   (any source)      │  fr-audit       │      AUDIT_BATCH_SUMMARY
                    │  v0.1.0         │      genie.action_log
                    └─────────────────┘
```

`fr-audit` is invoked standalone with `fr_paths: [...]`. The FRs may have been authored by `fr-create`, by a human, by an external tool, or reconstructed from BRAIN — the audit doesn't care. This is the original v2.0.0 `audit_only` mode, now its own atomic skill.

## Chain 3 — Create then audit then technical-spec generation (future)

Once `cuo/cto/fr-to-tech-spec` lands, the chain extends naturally:

```
   PRD ──▶ fr-create ──▶ fr-audit ──▶ cuo/cto/fr-to-tech-spec ──▶ tech-spec.md
                                              │
                                              └─ reads cyberos/docs/contracts/feature-request/v1/template.md
                                                 (declared via depends_on_contracts:) to know what an
                                                 FR looks like before deriving spec
```

This is exactly the "persona-grouped, multi-skill, chainable" property the CyberOS skill registry is designed for. `cuo/cpo/fr-create` produces FRs that any downstream skill (own-persona or cross-persona) can consume, because the output envelope schema is documented and stable.

## Failure handling across the chain

If `fr-audit` returns `overall_status: needs_human` for FR-NNN, the supervisor:

1. Writes back to `fr-create`'s manifest: `frs[FR].status = HITL_PAUSE`, `frs[FR].audit_hash = <new hash from fr-audit>`, `frs[FR].blocking_issues += <issues from audit>`.
2. Aggregates the HITL_BATCH_REQUEST per `references/HITL_PROTOCOL.md`.
3. Halts the entire CUO graph at the question node (LangGraph's `interrupt()` primitive).

When the human answers, the graph resumes:

1. Apply each ISS-NNN resolution per `references/HITL_PROTOCOL.md` §6.3.
2. Re-invoke `fr-audit` with the answer payload appended.
3. If audit now returns `pass`, write `frs[FR].status = PASS` and continue to the next FR in `fr-create`'s WORKER loop.

## Audit-row continuity

Both skills append rows to `genie.action_log`. To trace a single FR end to end, query:

```sql
SELECT *
FROM genie.action_log
WHERE trace_id = '<from CONTRACT_ECHO>'
  AND payload_data ->> 'fr_id' = 'FR-007'
ORDER BY created_at;
```

Expected rows for one FR's full lifecycle (create → audit → human → audit-pass):

1. `cuo/cpo/fr-create` · `row_kind: question` · plan-approval ask
2. `cuo/cpo/fr-create` · `row_kind: artefact_write` · FR-007.md written
3. `cuo/cpo/fr-audit`  · `row_kind: artefact_write` · FR-007.audit.md written, `overall_status: needs_human`
4. `cuo/cpo/fr-audit`  · `row_kind: question` · HITL_BATCH_REQUEST emitted
5. (human answers — separate row from CHAT module)
6. `cuo/cpo/fr-audit`  · `row_kind: artefact_write` · FR-007.audit.md re-written, `overall_status: pass`

The hash chain across these six rows lets the auditor reconstruct exactly what the skills did. A missing row (or a mismatched `before_hash`/ `after_hash`) is detected by CP's tamper detector (SRS §10.4.6).
