---
template: feature_request@1
id: FR-APP-005
title: "APP memory and audit-chain browser - search the knowledge layer and inspect the audit chain over the memory service"
author: "@stephen"
department: engineering
status: draft
priority: p3
created_at: "2026-06-29T11:10:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/src/api/memory.ts
  - apps/console/src/screens/memory_search.ts
  - apps/console/src/screens/memory_entities.ts
  - apps/console/src/screens/audit_chain.ts
  - apps/console/src/render/edge_list.ts
  - apps/console/src/render/chain_integrity.ts
  - apps/console/tests/memory_api_client.test.ts
  - apps/console/tests/memory_search_render.test.ts
  - apps/console/tests/chain_integrity_render.test.ts
depends_on: [FR-APP-001, FR-AUTH-004, FR-MEMORY-108, FR-MEMORY-124, FR-MEMORY-101]
---

# Feature Request

> Turn Your Will Into Real.

## Summary

The operator console needs a memory panel: one screen set in the existing console (the `app` module) for searching the knowledge layer and inspecting the audit chain that every CyberOS module writes. It is one more panel in the same static single-page app under `apps/console/`, extending the FR-APP-001 shell and auth gate; it does not define its own shell, auth, or design language. It searches the layer-2 knowledge (the `l2_memory` rows plus extracted entities) through the shipped memory search API (FR-MEMORY-108), browses entities and their relational edges (the `l2_edge` table), and gives the operator an audit-chain viewer over `l1_audit_log` - the hash-chained rows the memory service holds - with a chain-integrity indication from anchor verification. It is read-only and tenant-scoped through the session; the console never writes or edits memory. It reads only memory endpoints that already ship. Apache AGE has been removed from CyberOS, so the graph is the relational `l2_edge` table traversed by a recursive CTE on the service side; this browser reads relational data and does not assume a Cypher or AGE graph. It is operator-facing and CyberSkill-branded, the same Umber and Ochre CDS surface as the rest of the console and distinct from the tenant `portal`.

## Problem

CyberOS has a full memory service - a layer-2 knowledge store with vector, graph, and full-text search (FR-MEMORY-108), a relational edge table for entity relationships, and a hash-chained layer-1 audit log that every module appends to - and no operator screen over any of it. To search the knowledge layer, the operator hits `GET /v1/memory/search` by hand and reads JSON. To see how entities relate, there is no surface at all; the edges live in `l2_edge` and are only reachable through a service-side query. To inspect the audit chain or check that it has not been tampered with, the operator has no viewer; the chain-integrity signal (the `chain_anchor` recomputation that FR-MEMORY-108 already runs per result) is computed deep in the service and never shown to a person. The operator cannot answer "what does memory know about X", "how is X linked", or "is the chain intact" from one branded screen.

The APIs to back this panel already exist and ship. Memory search is live (FR-MEMORY-108) and returns ranked results with a snippet, a kind, a path, and a related-entity count, already RLS-scoped to the caller's tenant and already verifying each result's `chain_anchor` against layer 1. The audit ledger and its row kinds are defined (FR-MEMORY-124 enumerates `memory.awh_gate_result` alongside the existing aux kinds in the `l1_audit_log` AuditRecord schema), and the chain-anchor design comes from FR-MEMORY-101. What is missing is the presentation layer that ties search, entities and edges, and the audit chain into one auth-gated, CDS-branded panel in the console. The gap is purely the front-end. Building it as another panel in the existing static SPA adds no server code and reuses the console shell, the auth gate, and the Caddy front that FR-APP-001 already established. One correctness constraint shapes the work: because AGE is gone, the edge and graph data is relational (`l2_edge` traversed by recursive CTE), so the browser must render relational rows and must not assume a property-graph or Cypher result shape.

## Proposed Solution

A memory panel added to the console under `apps/console/`, built with the same CDS tokens and components and mounted inside the FR-APP-001 auth-gated shell. User-visible behaviour: the operator opens the memory panel from the console navigation, already signed in, and gets three screens. Knowledge search: a query box over `GET /v1/memory/search` (FR-MEMORY-108) that lists ranked results with kind, path, snippet, score, and related-entity count, scoped to the operator's tenant by the session. Entities and edges: select an entity from a result and see its relational edges from `l2_edge` - the linked entities and the relation on each edge - rendered as a plain list of relational rows, not a free-floating graph canvas. Audit chain: a viewer over `l1_audit_log` that lists recent hash-chained audit rows (the `memory.*` aux kinds plus the per-module rows) with a chain-integrity indication driven by the anchor verification the memory service already performs. Every panel is a read over an endpoint that already ships; the console renders, it does not compute and it does not write. The look is CDS, so the memory panel is recognisably the same CyberSkill operator surface as the rest of the console.

### Section 1 - normative requirements (BCP-14)

1. The memory panel MUST be one panel in the same static single-page app as FR-APP-001, under `apps/console/`, mounted inside the FR-APP-001 shell and behind its auth gate. It MUST NOT define its own shell, navigation chrome, sign-in flow, or design language; it reuses what FR-APP-001 established.

2. The panel MUST consume only memory service APIs that already ship: the memory search endpoint (FR-MEMORY-108), the entity-and-edge read it exposes over `l2_edge`, and the audit-chain read over `l1_audit_log` (the ledger whose row kinds FR-MEMORY-124 enumerates and whose anchor design is FR-MEMORY-101). It MUST NOT introduce a new backend endpoint or any server-side component; a screen that appears to need a new endpoint is a signal to extend the memory service's FR, not to add a backend in `app`.

3. The panel MUST be read-only. It MUST NOT write, edit, delete, or otherwise mutate any memory: no layer-2 row, no entity, no edge, and no audit row. It issues read requests only; memory mutation is out of scope for this panel and stays with the memory service.

4. The panel MUST treat the graph as relational. The entity-and-edge screen reads `l2_edge` rows (linked entity plus relation) that the memory service traverses by recursive CTE, and the client MUST render those relational rows. It MUST NOT assume, request, or parse a Cypher or Apache AGE property-graph result shape; AGE has been removed from CyberOS and no graph query language is in play.

5. The panel MUST be tenant-scoped through the session. Every memory read MUST carry the FR-AUTH-004 session token the FR-APP-001 shell holds, and the results the operator sees MUST be the tenant scope that the memory service's RLS applies to that token (FR-MEMORY-108 already scopes search by tenant). The console MUST NOT attempt to widen scope or pass a tenant other than the session's.

6. The audit-chain screen MUST surface a chain-integrity indication derived from the memory service's anchor verification (the `chain_anchor` recompute of FR-MEMORY-108 / FR-MEMORY-101), not from a check the console invents. When the service reports a verified chain, the screen shows intact; when the service reports an anchor mismatch, the screen shows the chain as suspect for the affected rows and MUST NOT present it as intact.

7. The panel MUST use CDS design tokens and components for layout, colour, type, and controls, the same Umber and Ochre palette and component set as the rest of the console. It MUST NOT introduce ad-hoc styling or a second design language.

8. On an expired or rejected session (a 401 from any memory read) the panel MUST defer to the FR-APP-001 auth gate and return the operator to sign-in, rather than rendering a partial or stale screen against a dead token.

9. The panel MUST fail visibly when a memory read is unreachable or returns a non-2xx: the affected screen shows a clear error state, not an empty result that reads as "memory knows nothing" or "the chain is empty". It MUST NOT fabricate search results, entities, edges, or audit rows for a call that did not succeed, and an empty-but-successful response (no matches, HTTP 200) MUST be shown as "no matches", distinct from an error.

10. The memory API-client and the view-render functions (search results, the `l2_edge` row list, and the chain-integrity indication) MUST be pure where they can be and unit-tested without a live backend, against fixture responses for the search, edge, and audit-chain shapes. A live render against a running memory service is an owner-run check, not a unit-test dependency.

## Alternatives Considered

Add a separate memory-admin web app instead of a panel in the console. Rejected for the same reason FR-APP-001 is one console rather than one app per engine: a second app would duplicate the shell, the auth gate, the CDS wiring, and the Caddy front, and split the operator across two surfaces that look alike but are maintained twice. The founder's decision is one unified console with one panel per module; the memory browser is that panel for memory, not a new front-end.

Render the entity relationships as an interactive graph canvas built from a property-graph query. Rejected on two grounds. First, AGE has been removed from CyberOS, so there is no Cypher or property-graph result to drive such a canvas; the relationships live in the relational `l2_edge` table and are traversed by recursive CTE. Second, the first release is a read-and-inspect operator surface, and a relational edge list (linked entity plus relation per row) answers "how is this linked" directly without a graph-drawing layer the console would have to own and the service does not return. A visual graph view, if ever wanted, is a later additive screen, not the first release.

Have the console recompute the chain integrity itself by pulling raw `l1_audit_log` rows and re-hashing them client-side. Rejected because it would duplicate the anchor verification that the memory service already performs (FR-MEMORY-108 verifies each result's `chain_anchor` against layer 1, per the FR-MEMORY-101 design) and would move a security-load-bearing check into the browser, where it is weaker and easy to get subtly wrong. The console surfaces the service's verdict; it does not become a second, divergent verifier of the chain.

## Success Metrics

Primary metric - operator memory and audit checks done through the console.
- Definition: fraction of the operator's memory-search and audit-chain inspections in a week that are done through the console memory panel rather than by hitting the raw `GET /v1/memory/search` endpoint or querying the audit ledger by hand.
- Baseline: 0%. No memory panel exists; today these checks go to raw JSON endpoints or direct queries.
- Target: at least 50% of those checks done through the console within six weeks of the panel's first release.
- Measurement method: the memory request log carries a client-id tag; count requests tagged with the console over total operator requests to the search and audit-chain reads.
- Source: the memory search request log (FR-MEMORY-108) and the audit-ledger read path over `l1_audit_log` (FR-MEMORY-124).

Guardrail metric - new backend endpoints introduced by the memory panel.
- Definition: number of new server-side endpoints or backend components the memory panel requires in order to function.
- Baseline: 0. The panel is specified as a pure front-end over shipped memory reads.
- Target: zero. Any screen that appears to need a new endpoint is a signal to extend the memory service's FR (FR-MEMORY-108 for search and entities/edges, the ledger FRs for the audit chain), not to add a backend inside `app`.
- Measurement method: review of the panel's `apps/console/src/api/memory.ts` client against the existing memory route list and OpenAPI; any call to a route that does not already exist fails the check.
- Source: the existing memory service route definitions plus code review of the panel's API layer.

## Scope

In scope: the three memory screens (knowledge search over FR-MEMORY-108, entities and their `l2_edge` relations as a relational row list, and the audit-chain viewer over `l1_audit_log` with a chain-integrity indication), the memory API client, the render functions for search results / edge rows / chain integrity, and unit tests for the client and the render functions against fixtures. All of it mounts inside the FR-APP-001 shell and auth gate and uses CDS.

### Out of scope

- Any new backend API or server-side component. The panel is a front-end only; new data needs go to the memory service's FR. The recursive-CTE traversal of `l2_edge` and the anchor verification are the service's job, not the console's.
- Any memory mutation: writing, editing, deleting, or re-ingesting layer-2 rows, entities, edges, or audit rows. The first release is read-only; operator mutation screens for memory, if ever wanted, are a separate later FR.
- A Cypher or Apache AGE property-graph view, or any interactive graph-drawing canvas. AGE is removed; the relationships are relational `l2_edge` rows rendered as a list. A visual graph view is a later additive screen.
- Client-side recomputation or re-hashing of the audit chain. The console shows the memory service's anchor-verification verdict; it does not become a second verifier.
- The shell, the auth gate, the sign-in flow, and the CDS token wiring. Those are FR-APP-001; this FR reuses them and does not redefine them.

## Dependencies

- FR-APP-001 APP CDS web console - the shell, the auth gate, the CDS tokens and components, and the Caddy static-serving path this panel mounts into; this FR adds a panel, not a new app.
- FR-AUTH-004 JWT and JWKS - the session token the shell holds and the panel carries on every memory read; the token's claims are what the memory service scopes results by.
- FR-MEMORY-108 memory search - the shipped `GET /v1/memory/search` the knowledge-search screen reads, already RLS-scoped per tenant and already verifying each result's `chain_anchor`; it also backs the entity and `l2_edge` relation reads the entities screen renders.
- FR-MEMORY-124 awh_gate_result audit row - enumerates the audit row kinds in the `l1_audit_log` AuditRecord schema that the audit-chain viewer lists; the ledger this FR reads is the one that FR defines rows for. (Renumbered from FR-MEMORY-121; 121 now carries the BRAIN interaction-event schema.)
- FR-MEMORY-101 layer-2 ingest pipeline - the source of the `chain_anchor` design and the layer-1-versus-layer-2 trust rule the chain-integrity indication reflects; the console surfaces this verification rather than reimplementing it.
- Cross-cutting: the existing Caddy front that FR-APP-001 deploys behind; the memory panel ships as part of the same static bundle and adds no new ingress. AGE removal is a standing constraint: the graph data is relational `l2_edge`, not a property graph.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from the founder's unified-admin-console decision and the existing module FRs (FR-APP-001 for the console, FR-MEMORY-108 / FR-MEMORY-124 / FR-MEMORY-101 for the memory reads).
- Scope: full draft of this specification, including the normative clauses, the alternatives, the metrics, and the scope boundaries. No console code is written by this FR; the memory panel is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The "panel in the same SPA, no new backend, read-only, relational not AGE" boundaries are operator-mandated, and the paired audit (FR-APP-005.audit.md) validates the format before merge.
