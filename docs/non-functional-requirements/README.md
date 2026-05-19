# Non-Functional Requirements (NFRs) — CyberOS

This directory holds the per-module NFR specs that back the rendered catalog at
`website/docs/reference/nfr-catalog.html`. Each NFR is a single Markdown file
with a normative §1 statement (BCP-14), an SLO, a measurement spec, a
verification pointer, and a failure-handling stanza.

## Layout

```
docs/non-functional-requirements/
├── README.md                                    # this file
├── ai/                                          # AI Gateway NFRs
│   ├── NFR-AI-001-provider-failover-budget.md
│   └── …
├── auth/                                        # AUTH NFRs
├── memory/                                       # memory (memory) NFRs
├── chat/                                        # CHAT NFRs
├── obs/                                         # Observability NFRs
└── …                                            # one folder per module
```

Folder naming mirrors `docs/feature-requests/` exactly — same `<module>/`
slugs, same `<ID>-<slug>.md` file shape. The renderer enumerates this tree
and projects each frontmatter row into the catalog's `NFR_DATA` array.

## NFR vs FR

- **FR** specifies a *feature surface* the system MUST expose.
- **NFR** asserts a *cross-cutting quality property* the system MUST hold
  (performance, security, reliability, scalability, observability, etc.).

NFRs are tighter than FRs by design: one constraint, one SLO, one
measurement, one verification path. Target body size is 80-150 lines.

## Template

See `modules/skill/contracts/non-functional-requirement/template.md` (not yet
shipped — for now, copy any existing NFR sibling as the canonical shape).

The required frontmatter keys are:

```yaml
id: NFR-<MODULE>-<NUMBER>
title: "<one-line, ≤80 chars>"
module: <AI | AUTH | memory | CHAT | OBS | …>
category: <performance | reliability | security | privacy | scalability | observability | maintainability | usability | compliance>
priority: <MUST | SHOULD | COULD>
verification: <T | I | A | D>
phase: P0..P4
slo: "<measurable target>"
owner: <CTO | CSO | CFO | …>
created: 2026-05-18
related_frs: [FR-…, FR-…]
```

## Authoring discipline

- One claim per NFR. If you find yourself writing "and also" — split.
- Always cite the shipped code path (e.g. `services/auth/src/rbac/refresher.rs`
  for refresher cadence claims). Don't invent SLOs; mirror what the code does.
- Verification pointer is mandatory — name the benchmark, test, or audit that
  proves the NFR holds.
- Failure handling §5 must say: detection, alert, on-call action, escalation.

## Batch authoring history

- **Batch 1 (2026-05-18)** — 42 NFRs across AI/OBS/AUTH/memory/CHAT.
  Resolves ~50 of the `(FR pending)` placeholders in the rendered catalog.
- Subsequent batches will cover SKILL, CUO, MCP, GraphQL, REW, CP, TEN, KMS,
  EMAIL, KB, NATS, and the remaining cross-cutting modules.
