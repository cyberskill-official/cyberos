---
id: TASK-CUO-106
title: "CUO supervisor Phase 4 — 5 special-case workflow handlers: time-critical SLA bypass, per-instance iteration, multi-output fan-out, sequential-approval gating, persona-pair partnership"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-18T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: CUO
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CDO)
created: 2026-05-18
shipped: 2026-05-18
memory_chain_hash: null
related_tasks: [TASK-CUO-101, TASK-CUO-104, TASK-CUO-105, TASK-SKILL-001, TASK-MEMORY-111]
depends_on: [TASK-CUO-104, TASK-CUO-105]
blocks: []

source_pages:
  - website/docs/modules/cuo.html#phase-4-handlers
  - modules/cuo/README.md#roadmap

source_decisions:
  - DEC-2380 2026-05-18 — During Sessions D–N (catalog completion), five workflow patterns surfaced that the default left-to-right chain walker cannot handle. Phase 4 ships one Handler subclass per pattern, dispatched by workflow frontmatter `pattern:` field
  - DEC-2381 2026-05-18 — Closed enum `workflow_pattern` = {linear (default), time_critical, per_instance, multi_output, sequential_approval, persona_pair}; cardinality 6
  - DEC-2382 2026-05-18 — Time-critical handler bypasses scheduler queueing; logs SLA-breach event if duration > workflow.sla_minutes. Workflows declaring this: chief-privacy-officer/breach-response-cycle, chief-communications-officer/per-crisis-response, chief-trust-officer/per-trust-incident-update
  - DEC-2383 2026-05-18 — Per-instance handler iterates the chain once per element of workflow.instance_descriptor[]; fan-in summary row aggregates per-instance ChainResults. Workflow declaring this: chief-sales-officer/quarterly-account-plan (10–20 instances/quarter)
  - DEC-2384 2026-05-18 — Multi-output handler runs chain once but fans out final step's output per workflow.output_recipients[]; emits one memory row per recipient. Workflow declaring this: chief-legal-officer/quarterly-regulatory-cycle
  - DEC-2385 2026-05-18 — Sequential-approval handler chains workflow A → halt for approval → chain B; approval is a HITL pause requiring explicit operator action. Workflow declaring this: chief-ethics-officer/per-model-card-ethics-sign-off gates chief-ai-officer/per-model-card-release
  - DEC-2386 2026-05-18 — Persona-pair handler runs two persona chains in interleaved fashion with shared artefact ownership; 4 patterns: churn-collaboration (cro-revenue ↔ cco-customer), content-vs-distribution (cmo ↔ cco-communications), risk-lens-vs-engineering (cro-risk ↔ cto), CX-vs-CDO (cco-customer ↔ cdo-data)
  - DEC-2387 2026-05-18 — Handler dispatch happens BEFORE execute_chain(); the supervisor reads workflow.pattern frontmatter and picks the Handler subclass. Default pattern (linear) uses the existing execute_chain() unchanged
  - DEC-2388 2026-05-18 — memory audit kinds: cuo.handler_dispatched, cuo.time_critical_sla_breach, cuo.per_instance_iteration, cuo.per_instance_summary, cuo.multi_output_fanout, cuo.sequential_approval_halted, cuo.sequential_approval_resumed, cuo.persona_pair_handoff

build_envelope:
  language: python 3.10+
  service: modules/cuo/cuo/core/
  new_files:
    - modules/cuo/cuo/core/handlers/__init__.py
    - modules/cuo/cuo/core/handlers/base.py
    - modules/cuo/cuo/core/handlers/time_critical.py
    - modules/cuo/cuo/core/handlers/per_instance.py
    - modules/cuo/cuo/core/handlers/multi_output.py
    - modules/cuo/cuo/core/handlers/sequential_approval.py
    - modules/cuo/cuo/core/handlers/persona_pair.py
    - modules/cuo/cuo/core/handlers/dispatch.py
    - modules/cuo/tests/test_applier_paths.py
    - modules/cuo/tests/test_proposal_applier.py
    - modules/cuo/tests/test_per_instance_handler.py
    - modules/cuo/tests/test_multi_output_handler.py
    - modules/cuo/tests/test_proposal_applier.py
    - modules/cuo/tests/test_proposal_applier.py

  modified_files:
    - modules/cuo/cuo/core/supervisor.py
    - modules/cuo/cuo/core/memory_bridge.py
    - modules/cuo/cuo/cli.py
    - modules/cuo/cuo/__init__.py
    - modules/cuo/pyproject.toml
    - website docs (CUO appendices)
    - "3 time-critical workflow YAML frontmatter (pattern: time_critical, sla_minutes: <N>)"
    - chief-sales-officer/quarterly-account-plan.md (pattern: per_instance, instance_descriptor field)
    - chief-legal-officer/quarterly-regulatory-cycle.md (pattern: multi_output, output_recipients field)
    - "chief-ethics-officer/per-model-card-ethics-sign-off.md + chief-ai-officer/per-model-card-release.md (pattern: sequential_approval, gates: field)"
    - 4 persona-pair workflow pairs (pattern: persona_pair, peer_persona + shared_artefact fields)

  allowed_tools:
    - file_read: modules/cuo/**, modules/skill/**, modules/memory/**
    - file_write: modules/cuo/cuo/core/handlers/**, modules/cuo/tests/**, modules/cuo/cuo/{supervisor.py,memory_bridge.py,cli.py,__init__.py}
    - bash: cd modules/cuo && pytest tests/test_*handler* -v

  disallowed_tools:
    - bypass HITL halt in sequential_approval handler (DEC-2385)
    - skip SLA-breach logging when time_critical exceeds limit (DEC-2382)
    - mutate workflow.skill_chain at runtime (forbidden by AGENTS.md §12)

effort_hours: 30
subtasks:
  - "1.0h: handlers/__init__.py + base.py (Handler ABC)"
  - "3.0h: dispatch.py (read workflow.pattern, return Handler subclass)"
  - "3.5h: time_critical.py (bypass scheduler, log SLA breach)"
  - "5.0h: per_instance.py (iterate chain ×N, fan-in summary)"
  - "3.5h: multi_output.py (fan-out final step per recipient)"
  - "4.5h: sequential_approval.py (chain A → HITL halt → chain B)"
  - "5.5h: persona_pair.py (interleaved chains with shared artefact)"
  - "4.0h: tests — 6 test files (one per handler + dispatch)"

risk_if_skipped: "Without Phase 4 handlers, 9 workflows in the production catalog (3 time-critical + 1 per-instance + 1 multi-output + 1 sequential-approval pair + 4 persona-pair pairs = 9 affected workflows out of 194) silently fall through to the default linear walker and produce WRONG output. Time-critical workflows lose SLA tracking; per-instance workflows produce 1 output instead of N; multi-output workflows route to 1 recipient instead of N; sequential-approval workflows skip the approval gate; persona-pair workflows lose the peer-handoff entirely. This is a correctness regression, not a feature gap."
---

## §1 — Description (BCP-14 normative)

The CUO supervisor **MUST** ship 5 workflow Handler subclasses at `modules/cuo/cuo/core/handlers/` dispatched from workflow `pattern:` frontmatter, with 8 memory audit kinds, and updated workflow YAML in the 9 affected catalog workflows.

1. **MUST** validate `workflow_pattern` against closed enum per DEC-2381 (cardinality 6, default `linear`).

2. **MUST** dispatch in `dispatch.py::pick_handler(workflow)` per DEC-2387:
   - Read `workflow.frontmatter.pattern` (default `linear`)
   - Return matching Handler subclass instance
   - Linear pattern → existing `execute_chain()` (unchanged path)
   - All others → new Handler subclass

3. **MUST** implement `TimeCriticalHandler` per DEC-2382:
   - Bypass any queueing/batching/work-stealing
   - Read `workflow.frontmatter.sla_minutes`
   - Start timer, walk chain
   - If `total_duration_ms > sla_minutes * 60 * 1000`: emit `cuo.time_critical_sla_breach` memory row with `extra.breach_severity = (actual - sla) / sla`
   - Affected workflows: `chief-privacy-officer/breach-response-cycle` (sla_minutes: 240 = 4h), `chief-communications-officer/per-crisis-response` (sla_minutes: 120 = 2h), `chief-trust-officer/per-trust-incident-update` (sla_minutes: 240)

4. **MUST** implement `PerInstanceHandler` per DEC-2383:
   - Read `workflow.frontmatter.instance_descriptor` (list of dicts)
   - For each instance: invoke `execute_chain()` with `inputs.merged(instance)`
   - Collect ChainResults in list; build fan-in summary `ChainResult` with `outcome="COMPLETED_BATCH"`, `per_instance: list[ChainResult]`
   - Emit one `cuo.per_instance_iteration` row per instance + one `cuo.per_instance_summary` row for the batch
   - Affected workflow: `chief-sales-officer/quarterly-account-plan` (10–20 top-tier accounts per quarter)

5. **MUST** implement `MultiOutputHandler` per DEC-2384:
   - Run chain end-to-end ONCE
   - Read `workflow.frontmatter.output_recipients` (list of `{recipient_id, format, delivery_method}`)
   - For each recipient: render final step's output through `recipient.format`, deliver via `recipient.delivery_method`
   - Emit one `cuo.multi_output_fanout` memory row per recipient
   - Affected workflow: `chief-legal-officer/quarterly-regulatory-cycle` (1 source artefact → N regulator filings)

6. **MUST** implement `SequentialApprovalHandler` per DEC-2385:
   - Read `workflow.frontmatter.gates` (list of `{approver_persona, approver_workflow}`)
   - Execute the gating workflow first (the approver's chain)
   - If approver chain `outcome != COMPLETED`: halt parent chain with `outcome=BLOCKED`, emit `cuo.sequential_approval_halted`
   - If approver chain emits explicit approval audit kind: proceed with gated chain, emit `cuo.sequential_approval_resumed`
   - Affected pair: `chief-ethics-officer/per-model-card-ethics-sign-off` gates `chief-ai-officer/per-model-card-release`

7. **MUST** implement `PersonaPairHandler` per DEC-2386:
   - Read `workflow.frontmatter.peer_persona` + `workflow.frontmatter.shared_artefact`
   - Run primary chain up to declared handoff step
   - Pause, dispatch to peer persona's matching workflow (looked up by shared_artefact content_hash)
   - Receive peer's contribution; resume primary chain
   - Emit one `cuo.persona_pair_handoff` row per peer-direction transition
   - Affected pairs:
     - `chief-revenue-officer/churn-collaboration` ↔ `chief-customer-officer/churn-collaboration` (shared: churn cohort analysis)
     - `chief-marketing-officer/content-strategy` ↔ `chief-communications-officer/distribution-strategy` (shared: campaign plan)
     - `chief-risk-officer/postmortem-risk-lens` ↔ `chief-technology-officer/postmortem-engineering-lens` (shared: incident report)
     - `chief-customer-officer/customer-360-cx-lens` ↔ `chief-data-officer/customer-360-data-lens` (shared: customer profile)

8. **MUST** preserve audit-chain integrity: all 8 new memory audit kinds (DEC-2388) routed through `cyberos.core.writer.Writer` (no direct file writes).

9. **MUST** wire handler dispatch into `cli.py execute` subcommand: when workflow has `pattern != linear`, log `# dispatched to <HandlerClass>` before invoking.

10. **MUST** version-bump `modules/cuo/pyproject.toml` from `3.0.0a3` to `3.0.0a4`.

11. **MUST NOT** mutate `skill_chain[]` at runtime (forbidden by CUO AGENTS.md §A.12).

12. **MUST NOT** bypass HITL halts in sequential_approval (the approval gate IS a HITL pause).

13. **MUST NOT** drop the `cuo.handler_dispatched` memory row for any non-linear pattern execution.

---

## §2 — Why this design

**Why dispatch by frontmatter `pattern:` field (DEC-2387)?** Workflow author declares the pattern in YAML; the supervisor reads it and picks the Handler. Keeps the linear/default path unchanged (zero performance regression for 185/194 workflows) and makes the special cases self-documenting.

**Why one Handler subclass per pattern (DEC-2381)?** Each pattern has distinct invariants — time-critical wants SLA tracking, per-instance wants fan-in summary, multi-output wants fan-out delivery, sequential-approval wants HITL gates, persona-pair wants peer handoff. Lumping them into a generic handler with a giant switch statement loses these invariants in code review.

**Why peer lookup by `shared_artefact.content_hash` (DEC-2386)?** Persona-pair handoffs are about shared artefact ownership, not about routing strings. Looking up by content hash ensures both personas see the same artefact even if their workflows name it differently.

**Why version `3.0.0a4` not `3.1.0`?** Phase 4 is alpha-grade like Phase 1–3 — handler implementations are basic; production hardening (retries, timeouts, observability) comes in `3.2.0`.

---

## §3 — API contract

Workflow frontmatter additions (per affected workflow):

```yaml
# Time-critical workflow
pattern: time_critical
sla_minutes: 240
```

```yaml
# Per-instance workflow
pattern: per_instance
instance_descriptor:
  source: workflow.inputs.account_list
  fields: [account_id, account_name, account_tier]
```

```yaml
# Multi-output workflow
pattern: multi_output
output_recipients:
  - { recipient_id: "vn-mst", format: "filing-xml-mst", delivery_method: "email" }
  - { recipient_id: "vn-mof", format: "filing-pdf-mof", delivery_method: "portal" }
```

```yaml
# Sequential-approval workflow (the gated one)
pattern: sequential_approval
gates:
  - { approver_persona: "chief-ethics-officer", approver_workflow: "per-model-card-ethics-sign-off" }
```

```yaml
# Persona-pair workflow
pattern: persona_pair
peer_persona: "cco-customer"
peer_workflow: "churn-collaboration"
shared_artefact: "churn-cohort-analysis"
handoff_step: 4
```

CLI surface:

```text
cyberos-cuo execute <persona>/<workflow>  # auto-detects pattern from frontmatter
  → dispatched to TimeCriticalHandler   (when pattern: time_critical)
  → dispatched to PerInstanceHandler    (when pattern: per_instance)
  → dispatched to MultiOutputHandler    (when pattern: multi_output)
  → dispatched to SequentialApprovalHandler  (when pattern: sequential_approval)
  → dispatched to PersonaPairHandler    (when pattern: persona_pair)
```

---

## §4 — Acceptance criteria

1. **workflow_pattern enum cardinality 6**.
2. **Default pattern (linear) routes through existing execute_chain() unchanged** — no perf regression for 185 affected workflows.
3. **TimeCriticalHandler emits sla_breach when duration > limit**.
4. **TimeCriticalHandler bypasses any scheduling layer** — invokes synchronously.
5. **PerInstanceHandler iterates exactly len(instance_descriptor) times**.
6. **PerInstanceHandler fan-in summary `outcome=COMPLETED_BATCH` when all succeed; `outcome=PARTIAL` when any fail**.
7. **MultiOutputHandler renders final-step output once per recipient**.
8. **MultiOutputHandler emits 1 memory row per recipient**.
9. **SequentialApprovalHandler halts on approver failure**.
10. **SequentialApprovalHandler resumes on approver success**.
11. **PersonaPairHandler routes to peer at declared handoff_step**.
12. **PersonaPairHandler shared_artefact content_hash matches across peer chains**.
13. **All 8 new memory audit kinds emit through cyberos.core.writer.Writer**.
14. **CLI `execute` prints `# dispatched to <HandlerClass>` for non-linear patterns**.
15. **9 affected workflows updated with correct `pattern:` frontmatter**.
16. **Existing 21/22 tests still pass post-change**.
17. **6 new test files green** (one per handler + dispatch).
18. **pyproject.toml version bumped to 3.0.0a4**.
19. **CUO docs site §12 Roadmap updated** — Phase 4 marked shipped.
20. **No workflow's skill_chain[] mutated at runtime**.

---

## §5 — Verification

```python
# modules/cuo/tests/test_applier_paths.py
def test_dispatch_default_is_linear():
    """Workflows without a pattern: field route to LinearHandler (= existing execute_chain)."""
    from cuo.core.handlers.dispatch import pick_handler
    workflow_dict = {"frontmatter": {}, "body": "..."}
    handler = pick_handler(workflow_dict)
    assert handler.__class__.__name__ == "LinearHandler"

def test_dispatch_reads_pattern_frontmatter():
    """Workflows with pattern: time_critical route to TimeCriticalHandler."""
    from cuo.core.handlers.dispatch import pick_handler
    workflow_dict = {"frontmatter": {"pattern": "time_critical", "sla_minutes": 240}}
    handler = pick_handler(workflow_dict)
    assert handler.__class__.__name__ == "TimeCriticalHandler"
    assert handler.sla_minutes == 240


# modules/cuo/tests/test_proposal_applier.py
def test_time_critical_emits_sla_breach_when_slow(tmp_memory):
    """If actual_duration > sla, memory gets a cuo.time_critical_sla_breach row."""
    from cuo.core.handlers.time_critical import TimeCriticalHandler
    handler = TimeCriticalHandler(sla_minutes=1)  # 1 minute SLA
    # Mock chain that takes 90 seconds
    result = handler.execute(slow_chain_fixture, memory_root=tmp_memory)
    breach_rows = [r for r in tmp_memory.audit_rows() if r.extra.get("kind") == "cuo.time_critical_sla_breach"]
    assert len(breach_rows) == 1
    assert breach_rows[0].extra["breach_severity"] > 0.5


# tests/test_per_instance_handler.py
def test_per_instance_iterates_once_per_account():
    """instance_descriptor with 5 accounts → 5 chain invocations + 1 summary."""
    from cuo.core.handlers.per_instance import PerInstanceHandler
    instances = [{"account_id": f"acct-{i}"} for i in range(5)]
    handler = PerInstanceHandler(instance_descriptor=instances)
    result = handler.execute(cso_sales_workflow_fixture)
    assert result.outcome == "COMPLETED_BATCH"
    assert len(result.per_instance) == 5


# tests/test_multi_output_handler.py
def test_multi_output_fanout_to_recipients():
    """3 recipients → final step output rendered 3 times + 3 memory rows."""
    from cuo.core.handlers.multi_output import MultiOutputHandler
    recipients = [
        {"recipient_id": "vn-mst", "format": "xml", "delivery_method": "email"},
        {"recipient_id": "vn-mof", "format": "pdf", "delivery_method": "portal"},
        {"recipient_id": "vn-sbv", "format": "json", "delivery_method": "api"},
    ]
    handler = MultiOutputHandler(output_recipients=recipients)
    result = handler.execute(clo_legal_workflow_fixture, memory_root=tmp_memory)
    fanout_rows = [r for r in tmp_memory.audit_rows() if r.extra.get("kind") == "cuo.multi_output_fanout"]
    assert len(fanout_rows) == 3


# modules/cuo/tests/test_proposal_applier.py
def test_sequential_approval_halts_on_ethics_reject():
    """If ethics-sign-off chain fails, model-card-release does NOT execute."""
    from cuo.core.handlers.sequential_approval import SequentialApprovalHandler
    handler = SequentialApprovalHandler(gates=[{
        "approver_persona": "chief-ethics-officer",
        "approver_workflow": "per-model-card-ethics-sign-off"
    }])
    # Mock approver chain that fails
    result = handler.execute(caio_per_model_card_release_fixture, approver_outcome="FAILED")
    assert result.outcome == "BLOCKED"
    halt_rows = [r for r in tmp_memory.audit_rows() if r.extra.get("kind") == "cuo.sequential_approval_halted"]
    assert len(halt_rows) == 1


# modules/cuo/tests/test_proposal_applier.py
def test_persona_pair_handoff_at_declared_step():
    """At handoff_step, primary pauses + peer invoked + result threaded back."""
    from cuo.core.handlers.persona_pair import PersonaPairHandler
    handler = PersonaPairHandler(
        peer_persona="chief-customer-officer",
        peer_workflow="churn-collaboration",
        shared_artefact="churn-cohort-analysis",
        handoff_step=4,
    )
    result = handler.execute(cro_revenue_churn_fixture)
    handoff_rows = [r for r in tmp_memory.audit_rows() if r.extra.get("kind") == "cuo.persona_pair_handoff"]
    assert len(handoff_rows) >= 1
    # Verify shared artefact content hash matches across both legs
    primary_hash = result.shared_artefact_hash
    peer_hash = result.peer_artefact_hash
    assert primary_hash == peer_hash
```

---

## §7 — Dependencies

**Upstream:** TASK-CUO-104 (topological chain walk), TASK-CUO-105 (per-step rollback — sequential_approval halt may trigger rollback of completed steps).

**Cross-module:** TASK-SKILL-001 (skill registry for peer-workflow lookup), TASK-MEMORY-111 (PII scrubbing for SLA-breach reason field).

**Downstream:** None — Phase 4 closes the supervisor design. Future work (TASK-CUO-107+) shifts to production hardening (retries, observability, multi-tenant).

---

## §10 — Failure modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown `pattern:` value in workflow frontmatter | dispatch.py rejects | refuse to execute; emit `cuo.handler_dispatch_failed` | author fixes frontmatter |
| time_critical chain hangs past SLA | timer in TimeCriticalHandler | sla_breach row emitted; chain continues (don't kill, deliver late + audit) | operator reviews breach in memory |
| per_instance empty descriptor | empty list check | refuse to execute; outcome=BLOCKED | author populates descriptor |
| multi_output zero recipients | empty list check | refuse to execute; outcome=BLOCKED | author adds recipients |
| sequential_approval approver chain has no halting step | approver chain returns COMPLETED without explicit approval audit | treat as auto-approved + log warning | operator decides if approval is implicit-OK |
| persona_pair peer workflow not found | catalog lookup miss | outcome=FAILED; emit `cuo.persona_pair_peer_not_found` | author fixes peer_persona/peer_workflow |
| persona_pair shared_artefact hash mismatch | content_hash comparison | outcome=FAILED; emit `cuo.persona_pair_artefact_drift` | author reconciles peer workflows |
| Handler raises uncaught exception | supervisor try/except | outcome=FAILED with stack trace in `notes` | bug report + fix |
| Concurrent execution of same persona_pair from both sides | content_hash dedup | second invocation joins first's result | inherent |
