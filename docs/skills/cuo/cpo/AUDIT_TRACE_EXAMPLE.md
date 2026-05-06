# Audit-trace example — proving auditability + chainability

> Walks one Feature Request from PRD intake to PASS-audit, showing every `genie.action_log` row that fires. Demonstrates the three properties that drove the layout decision: **auditable** (every output has a row), **chainable** (`fr-author` and `fr-audit` compose via deterministic envelopes), **plug-in-able** (the same chain works whether installed at `~/.cyberos/skills/cuo/cpo/` or extracted to a different machine).

## The scenario

A founder hands the CUO supervisor a single PRD file and asks: "Generate the FR backlog and audit it." The supervisor classifies the intent (`cuo/cpo/fr-author`, then chained via the default `next_skill_recommendation` to `cuo/cpo/fr-audit`).

The classifier emits one trace_id (`a1b2c3d4-e5f6-7890-abcd-ef1234567890`) that flows through every row below.

## Step 0 — supervisor routes to fr-author

CUO supervisor's `classify_act` node (per SRS §6.1.1) returns `{persona_id: "cuo-cpo", skill_id: "cuo/cpo/fr-author", confidence: 0.92}`.

```sql
INSERT INTO genie.action_log (audit_id, ts, actor_kind, actor_id, persona,
  op, scope, path, memory_id, classification, authority,
  before_hash, after_hash, diff, reason, prev_chain, chain)
VALUES (
  'evt_01HMQ…',
  '2026-05-05T10:00:00.000+07:00',
  'agent', 'cuo', 'cuo',
  'persona_card_loaded', 'project:cyberos-skills', '<cuo/cpo/SKILL.md>',
  null, 'operational', 'human-confirmed',
  null, null, null,
  'classify_act routed to cuo-cpo (conf=0.92) for intent: generate-and-audit-FR-backlog',
  '<chain head from prior session>',
  'sha256:abc…'
);
```

## Step 1 — fr-author PLAN phase

`fr-author` reads the PRD inside an `<untrusted_content>` block, applies the EU AI Act decision tree, enumerates 3 candidate FRs, writes the manifest with `plan.status = AWAITING_APPROVAL`, and emits the plan-approval render.

```
PROPOSED FR BACKLOG  (from 1 requirements file, sha256:84897c69fb76...)
=============================================================================
Run scope:                    generate up to N=3 FRs this batch.
Total backlog:                3 FRs identified.
EU AI Act flags (escalate):   0.
EU AI Act flags (high):       0.
Open planning questions:      1 (listed at bottom).

FR-001  P1  brain-search-latency-budget
        Tighten the BRAIN search latency budget from 800ms p95 to 500ms p95.
        Depends on: —
        EU AI Act tentative: not_ai
        client_visible tentative: false   ai_authorship tentative: none
        Source: ./EXAMPLE-PRD.md, §5.10.1

FR-002  P1  brain-write-coverage-warning
        Add a warning row when consolidation detects shallow ingestion.
        Depends on: FR-001
        EU AI Act tentative: not_ai
        client_visible tentative: false   ai_authorship tentative: none
        Source: ./EXAMPLE-PRD.md, §8.6

FR-003  P2  cuo-defer-trigger-audit-row
        Surface every CUO defer-to-human event as an audit row in OBS.
        Depends on: —
        EU AI Act tentative: limited
        client_visible tentative: false   ai_authorship tentative: assisted
        Source: ./EXAMPLE-PRD.md, §6.4.1

Open questions identified during planning
-----------------------------------------
  Q1 (FR-001): Is 500ms p95 tied to a downstream SLO, or a soft target?

Approval options
----------------
  A) APPROVE          → generate FRs 1..3 this run
  B) REVISE           → reply with edits
  C) ABORT            → set plan.status = INVALIDATED and stop

Reply with one of:  APPROVE  |  REVISE: <your edits>  |  ABORT
```

```sql
INSERT INTO genie.action_log VALUES (
  'evt_01HMR…',
  '2026-05-05T10:00:08.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'question', 'project:cyberos-skills', '<output_dir>/manifest.json',
  null, 'operational', 'llm-explicit',
  null, 'sha256:<approval_hash>', null,
  'plan-approval Question emitted; backlog=3 FRs; trace_id=a1b2c3d4',
  'sha256:abc…',
  'sha256:def…'
);
```

(The manifest write itself appends a separate row — `op: str_replace` on `manifest.json` — but is omitted here for brevity.)

## Step 2 — human approves; fr-author WORKER phase begins

Human replies `APPROVE`. The next invocation enters WORKER, claims FR-001:

- W1 CLAIM (`frs[FR-001].status = DRAFTING`; manifest write).
- W2 GENERATE — body adapted from `cyberos/docs/contracts/feature-request/template.md` (declared via `depends_on_contracts:` in fr-author v0.2.0+).
- W3 WRITE — `feature-requests/FR-001-brain-search-latency-budget.md`, hash `e7f1…`.
- W4 EMIT EVENT — NATS subject `cuo.fr_author.fr_written` published.
- W5 ROUTE — supervisor invokes `cuo/cpo/fr-audit` per the chain.

```sql
INSERT INTO genie.action_log VALUES (
  'evt_01HMS…',
  '2026-05-05T10:01:13.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'artefact_write', 'project:cyberos-skills',
  './feature-requests/FR-001-brain-search-latency-budget.md',
  null, 'operational', 'llm-explicit',
  null, 'sha256:e7f1…', null,
  'fr-author wrote FR-001 from ./EXAMPLE-PRD.md; trace_id=a1b2c3d4',
  'sha256:def…',
  'sha256:ghi…'
);
```

## Step 3 — supervisor chains to fr-audit (FR-001)

Supervisor passes `{fr_paths: [./feature-requests/FR-001-brain-search- latency-budget.md], upstream_context: {from_skill: cuo/cpo/fr-author, manifest_path: ./feature-requests/manifest.json}, trace_id: a1b2c3d4}` to `cuo/cpo/fr-audit`.

`fr-audit` runs the 8-step loop. Outcome: 2 errors (FM-110 missing `target_release`; QA-007 unsourced numeric target on the 500ms p95). QA-007 promotes to `needs_human` per the rubric. Audit report written with `overall_status: needs_human`.

```sql
INSERT INTO genie.action_log VALUES (
  'evt_01HMT…',
  '2026-05-05T10:01:38.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'artefact_write', 'project:cyberos-skills',
  './feature-requests/FR-001-brain-search-latency-budget.audit.md',
  null, 'operational', 'human-confirmed',
  null, 'sha256:b8d9…', null,
  'fr-audit wrote audit report; overall_status=needs_human; iter=1; trace_id=a1b2c3d4',
  'sha256:ghi…',
  'sha256:jkl…'
);
```

Then the HITL_BATCH_REQUEST emission:

```
HITL_BATCH_REQUEST
====================
audit_paths:                   [./feature-requests/FR-001-brain-search-latency-budget.audit.md]
total_paused_frs:              1
total_blocking_issues:         1
hitl_categories_present:       [success_metric_targets]

[FR-001-brain-search-latency-budget]  audit_path: ./…audit.md  iteration: 1/10

  ISS-001 (success_metric_targets) [rule_id: QA-007]
    Description (paraphrased):
      The target value 500ms p95 for BRAIN search latency is not derivable from
      the PRD. The PRD §5.10.1 mentions latency budgeting in general terms but
      cites no concrete target. The audit cannot fabricate this number.
    What was attempted:
      The audit searched the PRD for any mention of "500" or "p95"; both are
      absent. The audit also searched BRAIN for prior latency-budget decisions;
      no relevant memory found.
    Options:
      A) Provide the source — paste a citation or reply with the target you authorise.
      B) Mark the target TBD-HUMAN until a source is confirmed.
      C) Drop this metric from the FR (FR will continue auditing without it).

------------------------------------------------------------
How to answer
------------------------------------------------------------
Reply with ONE LINE per issue:

  FR-001/ISS-001: A; SLO doc target is 500ms p95, see brain-perf-budget memory mem_01HM…
  (or B / C)

When all 1 issues are answered, re-invoke fr-audit with the same fr_paths.
END_HITL_BATCH_REQUEST
```

```sql
INSERT INTO genie.action_log VALUES (
  'evt_01HMU…',
  '2026-05-05T10:01:39.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'question', 'project:cyberos-skills',
  './feature-requests/FR-001-brain-search-latency-budget.audit.md',
  null, 'operational', 'llm-explicit',
  null, 'sha256:<paused_issues_hash>', null,
  'fr-audit emitted HITL_BATCH_REQUEST; 1 paused FR; trace_id=a1b2c3d4',
  'sha256:jkl…',
  'sha256:mno…'
);
```

LangGraph pauses at the `interrupt()` node. State checkpoint flushed.

## Step 4 — human answers, audit resumes, FR-001 passes

Human replies `FR-001/ISS-001: A; SLO doc target is 500ms p95, see brain-perf-budget memory mem_01HM…`.

`fr-audit` re-invokes. Step 3 (load audit) succeeds; the existing report's `audited_file_sha256` matches the on-disk FR (FR was not edited externally, so the hash is unchanged). Apply the resolution: `audit.issues[ISS-001].resolution = "A; SLO doc target is 500ms p95, see brain-perf-budget memory mem_01HM…"`. The audit interprets option A as: cite the BRAIN memory in the FR's source_refs (auto-fixable addition; not a hallucination because the human supplied the citation).

Re-audit: QA-007 no longer fires (target now sourced). FM-110 still fires — the audit auto-fixes by inserting `target_release: "2026-Q3"` inferred from FR-002's earliest dependency cleared (Levenshtein-≤2 ambiguous-fix on a non-compliance-sensitive field). Re-re-audit: clean. Termination branch (a) PASS.

```sql
INSERT INTO genie.action_log VALUES (
  'evt_01HMV…',
  '2026-05-05T10:05:00.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'act', 'project:cyberos-skills',
  './feature-requests/FR-001-brain-search-latency-budget.audit.md',
  null, 'operational', 'human-confirmed',
  null, 'sha256:<applied_resolution_hash>', null,
  'fr-audit applied human resolution to ISS-001; trace_id=a1b2c3d4',
  'sha256:mno…',
  'sha256:pqr…'
);

INSERT INTO genie.action_log VALUES (
  'evt_01HMW…',
  '2026-05-05T10:05:04.000+07:00',
  'agent', 'cuo-cpo', 'cuo-cpo',
  'artefact_write', 'project:cyberos-skills',
  './feature-requests/FR-001-brain-search-latency-budget.audit.md',
  null, 'operational', 'human-confirmed',
  null, 'sha256:c4e0…', null,
  'fr-audit re-wrote audit report; overall_status=pass; iter=3; trace_id=a1b2c3d4',
  'sha256:pqr…',
  'sha256:stu…'
);
```

Audit envelope emitted:

```json
{
  "skill_id": "cuo/cpo/fr-audit",
  "skill_version": "0.1.0",
  "audit_rubric_version": "audit_rubric@2.0",
  "total_frs": 1,
  "overall_status_counts": {"pass": 1, "needs_human": 0, "fail": 0},
  "exit_code": 0,
  "per_fr": [
    {
      "fr_path": "./feature-requests/FR-001-brain-search-latency-budget.md",
      "audit_path": "./feature-requests/FR-001-brain-search-latency-budget.audit.md",
      "status": "pass",
      "iterations": 3,
      "audited_file_sha256": "e7f1…"
    }
  ],
  "hitl_required": false,
  "requires_regen": false,
  "next_skill_recommendation": ""
}
```

Supervisor reads `next_skill_recommendation: ""` → terminates the chain for FR-001. Returns control to `fr-author` to claim FR-002.

## Step 5 — repeat for FR-002 and FR-003

Same pattern; assume both pass on first audit (no needs_human). Each generates one `artefact_write` row from `fr-author`, one `artefact_write` row from `fr-audit`. Then `fr-author` emits BATCH_COMPLETE.

## Audit reconstruction query

To prove auditability — the founder can reconstruct exactly what happened:

```sql
SELECT audit_id, ts, persona, op, path, reason
FROM genie.action_log
WHERE reason LIKE '%trace_id=a1b2c3d4%'
ORDER BY ts;
```

Result (truncated):

```
evt_01HMQ… 10:00:00  cuo      persona_card_loaded  cuo/cpo/SKILL.md         classify_act routed to cuo-cpo …
evt_01HMR… 10:00:08  cuo-cpo  question             …/manifest.json          plan-approval Question emitted …
…           …        …        str_replace          …/manifest.json          plan.status APPROVED
evt_01HMS… 10:01:13  cuo-cpo  artefact_write       …/FR-001-….md             fr-author wrote FR-001 …
evt_01HMT… 10:01:38  cuo-cpo  artefact_write       …/FR-001-….audit.md       fr-audit wrote audit report; overall_status=needs_human …
evt_01HMU… 10:01:39  cuo-cpo  question             …/FR-001-….audit.md       fr-audit emitted HITL_BATCH_REQUEST …
evt_01HMV… 10:05:00  cuo-cpo  act                  …/FR-001-….audit.md       fr-audit applied human resolution …
evt_01HMW… 10:05:04  cuo-cpo  artefact_write       …/FR-001-….audit.md       fr-audit re-wrote audit report; overall_status=pass …
…           …        …        artefact_write       …/FR-002-….md             fr-author wrote FR-002 …
…           …        …        artefact_write       …/FR-002-….audit.md       fr-audit pass …
…           …        …        artefact_write       …/FR-003-….md             fr-author wrote FR-003 …
…           …        …        artefact_write       …/FR-003-….audit.md       fr-audit pass …
…           …        …        notify               …/manifest.json          BATCH_COMPLETE outcome=BATCH_COMPLETE
```

Hash chain integrity check (per AGENTS.md §11.4): every row's `prev_chain` matches the previous row's `chain`. Tampering with any row breaks the chain — detected by CP's tamper detector (SRS §10.4.6).

## What this proves

1. **Auditability** — every concrete output (artefact write, question, review action) becomes one row. The trace is reconstructible from `genie.action_log` alone, with no other state required.
2. **Chainability** — `fr-author.produces.next_skill_recommendation` drives the supervisor's conditional edge into `fr-audit`. The two skills compose without shared state besides the input/output envelope.
3. **Plug-in-ability** — replace any path above with a different absolute path (e.g., a teammate's machine) and the trace is identical modulo file paths. `cp -r cuo/cpo/` to another machine reproduces the chain exactly.
4. **Determinism (audit-side)** — re-running `fr-audit` against the same `audited_file_sha256` produces a byte-identical report (modulo `last_audit_at`). The chain hash for evt_01HMW is the same on every re-run.
5. **Human-in-the-loop fidelity** — the never-re-ask invariant (HITL_PROTOCOL.md §6.6) holds across the chain. Once `audit.issues[ISS-001].resolution` is non-null, no future invocation re-asks.

## Failure-mode trace (orthogonal example)

Same scenario but `fr-author`'s W3 WRITE fails (disk full). The skill:

- Does NOT write a partial FR (per `SKILL.md` MUST NOT).
- Does NOT advance `manifest.frs[FR-001].status` from DRAFTING.
- Emits `BOOTSTRAP_FAILURE code: BOOT-005` with remediation text.
- Appends one `notify` row to `genie.action_log` with the failure reason.

The supervisor sees a `notify` instead of an `artefact_write`, classifies the chain as ERRORED, and surfaces a Notify to the user. No subsequent `fr-audit` invocation occurs because the chained skill has nothing to audit.
