# `spec-to-impl-plan` self-audit invariants (scaffold)

## Invariants

### INV-001 — refuse non-pass upstream artefact

**Statement.** Refuse if the upstream artefact (tech-spec for standard/full; audited FR for lean) is in non-pass state.

**Severity.** `error` (sev-0).

### INV-002 — never auto-create tickets without explicit human approval

**Statement.** Even when `create_tickets: true` is in the input envelope, the runtime MUST emit a final `HALT_BEFORE_CREATE` HITL prompt + wait for explicit human "yes, create the tickets" before any `proj.create_issue` call. The envelope's flag is "permission to ASK", not "permission to CREATE."

**Check.** Audit row sequence: any `op:proj.create_issue` row MUST be preceded by an `op:hitl_approval` row in the same trace_id with category `ticket_creation_approval`.

**Severity.** `error` (sev-0). Ticket creation is reversible (delete) but creates user-facing artefacts in external systems.

### INV-003 — sizing distribution warning

**Statement.** If >50% of tickets are XL-sized OR >30% are L+XL combined, surface a warning to the user. Highly XL-skewed plans are a signal that the breakdown is too coarse and tickets won't fit one sprint.

**Severity.** `warning`.

### INV-004 — capacity overage check

**Statement.** Sum of `total_estimated_engineer_days` across the impl-plan MUST NOT exceed the team's available capacity for `target_release` window (per `member:*` BRAIN reads). If it does, surface the overage with options: (a) reduce scope, (b) extend release window, (c) hire/contract, (d) proceed with documented overage (rare).

**Severity.** `warning`.
