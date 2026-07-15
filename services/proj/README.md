# CyberOS PROJ — Issue + Cycle + Engagement model

**Status:** TASK-PROJ-001..018 shipped as service slices — schema + types + FSM + audit row builders + handler orchestration + bidirectional symmetric link writer + memory decision anchoring + rate-card versioning + typed MEMORY_LINK helpers + CRDT/LWW collaboration + billing cascade/modes + mutation history events + citation drift + blocker detection + cycle review + estimate calibration + board/timeline/gantt/brief view models + design tokens/a11y gate.
**Depends on:** TASK-AUTH-001 (tenants) + TASK-AUTH-003 (RLS pattern) — both shipped.
**Blocks (downstream):** TASK-EMAIL-007 (convert-to-issue), TASK-RES-001, TASK-TIME-004/005, TASK-HR-008, TASK-LEARN-003.

---

## §1 — Model

- **Engagement** — the top-level scope for project work. Every issue belongs to one (DEC-213).
- **Cycle** — time-boxed window within an engagement (`starts_at < ends_at`).
- **Issue** — the unit of work. 5-status FSM + 4-priority enum. Optional cycle membership; assignee must be same-tenant.
- **Issue link** — `(issue_id, linked_to_id, link_type)` triple. Symmetric types auto-insert the inverse direction.
- **Decision anchor** — immutable `proj.decision` memory row payload per issue state change, with reason, prior-chain link, references, redaction, and retraction payload support.
- **Rate card** — append-only engagement pricing records with role, currency, hourly rate, billable default, and effective-date supersession.
- **MEMORY_LINK** — typed Issue ↔ memory relation (`cites | implements | supersedes | cites_with_quote`) with duplicate, scope, cross-tenant, and future-supersede guards.
- **Collaboration** — Yjs update journal contract for description/comment bodies, plus LWW scalar merge for metadata.
- **Billing** — member override → task class → role default → fallback cascade, with T&M/fixed-fee/retainer rollups.
- **Operations intelligence** — citation drift, blocker detection, cycle review drafts, and estimate calibration snapshots.
- **Views** — Kanban/timeline/gantt/brief modal view models and `tokens.proj.css` with an axe-style blocking-rule gate.

---

## §2 — FSM

```
   triage  ─▶  todo
              ├▶  doing  ─▶  review  ─▶  done   (terminal)
              ├▶  triage           (deferral)
              ╰▶  done
              doing  ─▶  todo      (pause)
              review ─▶  doing     (rejected back)
              review ─▶  todo      (significant rework)
```

`done` is terminal — reopening requires the explicit reopen API (not in this task). `deleted` is a reserved soft-delete state usable only by root-admin.

Illegal transitions return `400 BAD_REQUEST` with body:

```json
{
  "error": "illegal_status_transition",
  "from": "triage",
  "to": "done",
  "allowed": ["todo"]
}
```

---

## §3 — Link types

| Type | Inverse | Use |
|---|---|---|
| `blocks` | `blocked_by` (auto) | A must complete before B can progress |
| `duplicates` | `duplicated_by` (auto) | A is the same as B |
| `related` | — | Loose association |
| `derived_from_email_thread` | — | Issue was created from an EMAIL thread (TASK-EMAIL-007) |
| `derived_from_chat_thread` | — | Issue was created from a CHAT thread |
| `derived_from_meeting_decision` | — | Issue was created from a meeting decision |

Symmetric types (`blocks`/`blocked_by`, `duplicates`/`duplicated_by`) auto-insert the inverse row in the same transaction.

---

## §4 — memory audit row kinds

Canonical kinds emitted by the PROJ service slices:

| Kind | When | Payload |
|---|---|---|
| `proj.issue_created` | POST /v1/proj/issues | engagement_id, priority, by_subject_id, initial status, optional assignee |
| `proj.issue_status_changed` | PATCH that mutates status | from_status, to_status, by_subject_id |
| `proj.issue_assigned` | PATCH that changes assignee | from_subject_id, to_subject_id, by_subject_id |
| `proj.issue_linked` | POST /v1/proj/issues/{id}/links | linked_to_id, link_type, by_subject_id |
| `proj.decision_recorded` | Issue state-change decision is anchored | issue_id, from_status, to_status, reason, prior_chain |
| `proj.decision_retracted` | Decision anchor is retracted with replacement context | decision_id, retraction_reason, replacement_decision_id |
| `proj.rate_card_created` | Engagement rate-card row is created | engagement_id, role, currency, hourly_rate, effective_from |
| `proj.rate_card_superseded` | Rate-card row is superseded | old_rate_card_id, new_rate_card_id |
| `proj.memory_link_created` | Issue cites/implements/supersedes a memory | issue_id, memory_id, relation |
| `proj.memory_link_removed` | MEMORY_LINK is soft-removed | link_id, reason |

The memory write transport is wired in the binary (TASK-PROJ-008 expands handler-level mutation coverage and carries the chain-anchor responsibility).

---

## §5 — Build + test

```bash
cd services
cargo build -p cyberos-proj
cargo test  -p cyberos-proj --lib                          # inline tests
cargo test  -p cyberos-proj --test status_fsm_test
cargo test  -p cyberos-proj --test audit_row_test
cargo test  -p cyberos-proj --test link_types_test
cargo test  -p cyberos-proj --test error_mapping_test
cargo test  -p cyberos-proj --test productivity_slice_test
```

---

## §6 — Layout

```
services/proj/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── migrations/
│   ├── 0001_engagements.sql      engagements + RLS + tenant_scoped policy
│   ├── 0002_cycles.sql           cycles + RLS + ends_at > starts_at CHECK
│   ├── 0003_issues.sql           issues + RLS + updated_at trigger
│   ├── 0004_issue_links.sql      issue_links + RLS via issues join + self-ref CHECK
│   ├── 0005_proj_decisions.sql   decision anchors + retraction rows + RLS
│   ├── 0006_rate_cards.sql       append-only rate cards + active uniqueness + RLS
│   └── 0007_memory_links.sql     typed Issue-memory links + soft remove + RLS
├── src/
│   ├── lib.rs
│   ├── types.rs                  IssueStatus, IssuePriority, LinkType, Issue, Engagement, Cycle, Actor, requests
│   ├── errors.rs                 IssueError + .code() + .http_status()
│   ├── status_fsm.rs             allowed_transitions + validate (with same-status no-op)
│   ├── audit.rs                  4 memory row builders + ProjAuditRow struct
│   ├── decisions.rs              TASK-PROJ-002 memory decision anchoring + DB writer
│   ├── crdt.rs                   TASK-PROJ-003 collaboration update journal
│   ├── rate_card.rs              TASK-PROJ-005 append-only rate-card DB writer
│   ├── billing.rs                TASK-PROJ-006/007 cascade + rollups
│   ├── history.rs                TASK-PROJ-008 history_event hash helpers
│   ├── memory_link.rs            TASK-PROJ-009 typed MEMORY_LINK DB writer
│   ├── drift.rs                  TASK-PROJ-010 citation drift detector
│   ├── blockers.rs               TASK-PROJ-011 blocker parser/dwell monitor
│   ├── cycle_review.rs           TASK-PROJ-012 deterministic review draft inputs
│   ├── estimate.rs               TASK-PROJ-013 calibration snapshots
│   ├── views.rs                  TASK-PROJ-014..018 view models + a11y gate
│   ├── links.rs                  bidirectional symmetric link writer + self-link guard
│   ├── repo.rs                   sqlx CRUD layer (engagement, cycle, issue) + RLS GUC setter + validators
│   └── handlers.rs               handler-layer orchestration + audit-row construction
└── tests/
    ├── status_fsm_test.rs        TASK-PROJ-001 §4 #3 + §4 #4 — FSM coverage
    ├── audit_row_test.rs         §4 #1 + §4 #5 — memory row builders
    ├── link_types_test.rs        §4 #10 + §4 #11 — link inverses + cross-module
    ├── error_mapping_test.rs     §4 #3 + §4 #6 + §4 #7 + §4 #14 — error → HTTP status
    └── productivity_slice_test.rs TASK-PROJ-003..018 coverage
```
