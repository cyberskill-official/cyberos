# `spec-to-impl-plan/acceptance/` — priority test scenarios (stub)

> Pending v0.3.0 harness.

## sev-0
1. INV-001 refuse non-pass tech-spec → `REFUSED_NON_PASS_INPUT`.
2. INV-001 refuse non-pass FR (lean profile) → same.
3. INV-002 never auto-create tickets → even with `create_tickets: true`, runtime halts at `HALT_BEFORE_CREATE_TICKETS` and waits for explicit human OK.
4. Happy path standard profile: tech-spec → impl-plan markdown with 3-7 tickets, sizing distribution sensible.
5. Happy path lean profile: audited FR → impl-plan markdown with `## Architecture Note` H2 filled in (since lean skips tech-spec).

## sev-1
6. INV-003 sizing-distribution warning fires when >50% XL.
7. INV-004 capacity-overage warning fires; user picks "extend release" → recorded in `## Risks`.
8. Tickets created (with explicit human approval) → `## Ticket Index` auto-populated; PROJ MCP returns ticket IDs + URLs.
9. Markdown-only mode: user picks (c); tickets NEVER created; impl-plan stays at `tickets_created: false` permanently.

## sev-2
10. Empty input (neither tech_spec_path nor fr_path) → schema validation fails → BOOT-003.
