# Architecture Decision Records

Index of irreversible / high-cost decisions for CyberOS. Newer ADRs use the
`architecture-decision-record@1` frontmatter shape. Cross-links to DEC-* rows in
task specs are noted inline where a ledger entry already exists; a standalone
company DEC ledger file is not required for these backfills (TASK-IMP-058).

| ADR | Title | Status |
|---|---|---|
| [ADR-001](ADR-001-own-chat-from-scratch.md) | Own chat from scratch (not Mattermost-forever) | accepted |
| [ADR-002](ADR-002-age-removal-relational-graph.md) | AGE removal — relational `l2_edge` graph | accepted |
| [ADR-003](ADR-003-payload-vs-platform-scope.md) | Payload vs platform scope for CyberOS 1.x | accepted |
| [ADR-004](ADR-004-rules-sha-content-cone.md) | `rules_sha` content cone | accepted |
| [ADR-005](ADR-005-hitl-acceptance-gates.md) | HITL acceptance gates are non-bypassable | accepted |
| [ADR-IMP-068-001](ADR-IMP-068-001-ci-proof-over-committed-dist.md) | CI proof over committed dist/ | accepted |
| [ADR-OBS-003-001](ADR-OBS-003-001-red-via-middleware.md) | RED via middleware | accepted |

## When to add an ADR

Add one when a decision is expensive to reverse (data model, public surface,
install contract, security boundary) and future agents would otherwise re-litigate
it from chat history. Prefer a short ADR over a long strategy essay.
