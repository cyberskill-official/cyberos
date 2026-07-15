# PROJ module — agent instructions

Supplementary to the root `AGENTS.md` (CyberOS Layer-1 Memory Protocol).
This file is NOT a §0.1-precedence override.

---

## §1 — Hard rules

- **No orphan issues.** Per DEC-213, every issue belongs to an engagement.
  The `engagement_id NOT NULL REFERENCES engagements(id)` FK enforces; any
  PR that softens this is a spec violation.

- **No custom statuses at slice 1.** Per DEC-210 the 5-state FSM is closed.
  Adding new statuses without amending TASK-PROJ-001 §1 #2 is forbidden.

- **Status transitions go through the FSM.** Any code path that writes
  `issues.status` MUST first call `status_fsm::validate`. The illegal-
  transition error is the contract.

- **`done` is terminal.** Reopening goes through a separate API path
  (slice 2). Patching `done → *` MUST return `illegal_status_transition`.

- **Bidirectional links auto-insert the inverse.** `LinkType::Blocks` →
  also writes the `BlockedBy` row. `Duplicates` → `DuplicatedBy`. The
  asymmetric types (`Related`, `DerivedFromEmailThread`, etc.) do NOT
  auto-insert.

- **Cross-tenant assignee is fail-closed.** `repo::validate_assignee_in_tenant`
  runs with the actor's RLS GUC set; a cross-tenant subject returns 0 rows
  and surfaces as `AssigneeCrossTenant`. The HTTP layer never sees a
  cross-tenant value land on the issue row.

- **Audit row is constructed in the handler, NOT the repo.** The repo
  layer is RLS-aware sqlx; the handler decides which audit rows to emit.
  Keeps the audit emission policy auditable.

---

## §2 — Where to find things

| Concern | Location |
|---|---|
| SQL schema | `migrations/0001..0004_*.sql` |
| Domain types | `src/types.rs` |
| Error type | `src/errors.rs` |
| Status FSM | `src/status_fsm.rs` |
| memory audit row builders | `src/audit.rs` |
| Repo layer | `src/repo.rs` |
| Handler orchestration | `src/handlers.rs` |
| Link writer | `src/links.rs` |

---

## §3 — Spec divergences (see audit dossier §10.6)

- **`estimate_hours`:** spec §3 declares `NUMERIC(6,2)`. Implementation uses
  `DOUBLE PRECISION` with a CHECK constraint capping at 9999.99 — avoids
  adding the `sqlx/bigdecimal` workspace feature for the slice-1 surface.
- **RLS GUC name:** spec §3 uses `app.tenant_id`. Implementation aligns
  with TASK-AUTH-003 §10.6 amendment using `app.current_tenant_id`.
