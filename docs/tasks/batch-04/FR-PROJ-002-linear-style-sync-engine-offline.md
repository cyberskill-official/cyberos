---
title: "PROJ — Linear-style sync engine: optimistic mutations, WebSocket fan-out, IndexedDB cache, offline replay"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the Linear-style sync engine for PROJ. **Optimistic local mutations** apply immediately to the in-browser cache; the change is sent to the server; on confirmation the server-canonical state replaces the local state; on conflict the server wins and the local state rebases. **WebSocket fan-out** broadcasts every change to other connected clients in the same project within 250 ms p95. **IndexedDB cache** holds a per-Member, per-project view-model that survives reload and reconnect. **Offline queue** retains mutations made while disconnected; replay on reconnect with deterministic ordering. **Vietnamese-mobile-network resilience** — intermittent connectivity is the default assumption; the UX never blocks on a network round-trip; every action confirms locally first. The same pattern is reused by CHAT (FR-CHAT-001 §"Sync engine") and EMAIL (FR-EMAIL-002 partial); PROJ is the most write-heavy consumer and the canonical implementation.

## Problem

Project management UX without optimistic mutation feels like loading a slow web page each time you change a field — and that is exactly what Asana / monday.com / ClickUp deliver on Vietnamese mobile networks where round-trip latency to a US-hosted backend exceeds 600 ms p95. The team's daily PM work today is degraded by exactly this: 5 seconds to drag a task to a column, 3 seconds to set an assignee, 2 seconds to add a comment.

The PRD §9.5.2 specifies the Linear sync-engine pattern as the floor: "optimistic local mutations, real-time WebSocket fan-out, server-canonical conflict resolution. On the client, all mutations are applied immediately to local state; the change is sent to the server; on confirmation, the server-canonical state replaces local; on conflict, the server wins and the local state rebases." Without this pattern, PROJ adoption fails — the team prefers the prior tracker that loaded faster on their phones.

The PRD's sync-engine commitment also names "the same pattern used in CHAT (§9.3.3)" — there is one architectural shape across modules; this FR is its canonical implementation.

## Proposed Solution

The shape of the answer is a `cyberos-proj-sync` server (Rust + Actix Web for low-overhead WebSocket fan-out), an IndexedDB-backed client SDK (`@cyberskill/proj-sync` published from the design-system monorepo), the optimistic-mutation contract on every PROJ GraphQL mutation, and the offline-queue replay protocol.

**Client-side state.**

```
IndexedDB (per Member per tenant per project view-model):
  ─ projects/<id>/issues               (full list; updated by deltas)
  ─ projects/<id>/cycles               (full list)
  ─ projects/<id>/snapshot_lsn         (last-applied delta sequence number)
  ─ pending_mutations/<txn-id>         (queue of unsynced local mutations)
  ─ tombstones/<entity-id>             (soft-delete markers awaiting server confirm)
```

The view-model is normalised by entity type; React + Zustand selectors read from IndexedDB through an in-memory cache invalidated on delta apply.

**Optimistic mutation contract.**

Every mutation in the GraphQL subgraph follows the same shape on the wire:

```graphql
input ProjMutationEnvelope {
  txnId: UUID!                # client-generated UUID; deduplicates retries
  baseSnapshotLsn: BigInt!    # client's latest LSN before applying locally
  mutationKind: String!       # "issue.update" | "issue.transition" | etc.
  payload: JSON!
}

type ProjMutationResult {
  txnId: UUID!
  status: ProjMutationStatus!  # "applied" | "rebased" | "conflict_dropped"
  resultLsn: BigInt
  rebasedFrom: JSON            # if status == "rebased", what changed
  conflictReason: String       # if status == "conflict_dropped"
  canonicalEntity: ProjEntity  # the post-mutation server-canonical entity
}
```

The client:

1. Generates `txnId` (UUID v4) before applying locally.
2. Updates IndexedDB optimistically; the UI reflects the change immediately.
3. Enqueues the mutation in `pending_mutations` with the current `snapshot_lsn`.
4. POSTs the mutation envelope to the GraphQL mutation endpoint.
5. On `applied` response: replaces the optimistic state with `canonicalEntity`; updates `snapshot_lsn`; removes from `pending_mutations`.
6. On `rebased`: applies the server-canonical state; surfaces a small toast if user-visible state changed materially ("your update was merged with a teammate's change").
7. On `conflict_dropped`: reverts the optimistic change; surfaces an error toast with retry option.
8. On network failure: keeps the mutation in `pending_mutations`; retries on reconnect.

**Server-side ordering.**

The server applies mutations in `txnId` arrival order *per tenant per project*; cross-project order is not guaranteed. A monotonic LSN (Log Sequence Number) is assigned per mutation per project; the LSN is what subscribers consume. Postgres-backed via a per-project sequence + a `proj.mutation_log` partitioned table:

```sql
CREATE TABLE proj.mutation_log (
  lsn BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL,
  txn_id UUID NOT NULL,
  actor_member_id UUID NOT NULL,
  mutation_kind TEXT NOT NULL,
  base_snapshot_lsn BIGINT NOT NULL,
  before_state JSONB,
  after_state JSONB NOT NULL,
  status TEXT NOT NULL,                       -- "applied" | "rebased" | "conflict_dropped"
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, project_id, txn_id)
) PARTITION BY RANGE (occurred_at);
```

Monthly partitions; 90-day hot retention; cold archive via the same path as `audit.entry` (FR-AUTH-002).

**Conflict resolution.**

The server is canonical. When a mutation arrives with a `baseSnapshotLsn` older than the entity's current LSN, the server attempts a three-way merge:

- **Field-level non-overlap.** A teammate updated `description_md`; the local update changes `assigneeMemberId`. Both apply; status: `applied`.
- **Field-level overlap, last-write-wins on the *server-arrival* time.** Both updated `state` to different values. The later arrival wins; the earlier mutation's status: `rebased` with `rebasedFrom` showing the original intent. The user sees the toast and can re-issue.
- **Hard conflict** (semantic — e.g. an issue was deleted while a Member edited it). Status: `conflict_dropped` with reason.

The merge logic is per-mutation-kind and lives in `cyberos-proj-sync`'s mutation handlers; complex mutations (cycle close + carryover) declare their own merge function.

**WebSocket fan-out.**

`cyberos-proj-sync` runs a WebSocket endpoint at `wss://api.cyberos.world/proj/sync/{tenant-slug}/{project-id}`; clients authenticate with the same OAuth bearer used for GraphQL. On connect:

1. Server reads the client's `last_seen_lsn` from the URL parameter.
2. Server streams every mutation in `proj.mutation_log` since `last_seen_lsn` for the project.
3. Server transitions to live mode: every applied mutation fan-outs to subscribed clients.

Per-tenant per-project subscription scope keeps connection load bounded; each WebSocket replica handles ≤ 5,000 concurrent connections (NFR-PERF-PROJ-SYNC-001). At P1 internal scale this is far below capacity.

NATS publishes the same event stream on `cyberos.{tenant}.proj.mutation.applied` for cross-module consumers (CUO observation, OBS dashboards, BRAIN ingestion of completed issues).

**Offline behaviour.**

When the client loses connectivity:

- Optimistic mutations continue to apply locally; they are written to `pending_mutations` with timestamps.
- The UI shows a small "offline" chip in the top bar.
- Search continues against IndexedDB.
- Subscriptions resume on reconnect; the client streams missed mutations from `last_seen_lsn` to current.
- The `pending_mutations` queue replays in order; rebases and conflicts are surfaced as a single batch toast ("3 changes synced; 1 rebased").

The replay is deterministic: each pending mutation includes its `txnId` so server-side dedup catches duplicates; replay order matches local creation order; the server applies in arrival order with rebasing.

**Vietnamese-mobile-network resilience.**

- Heartbeats: server pings every 30 seconds; client pongs; missed pong = reconnect.
- Backoff: reconnect uses exponential backoff (1s → 2s → 4s → 8s → 16s, cap 60s) with jitter.
- TLS resumption: session tickets enabled to skip full handshake on reconnect.
- Compression: per-message-deflate enabled; reduces bandwidth on the constrained networks.
- Bandwidth caps: per-client outbound ≤ 50 KB/s during sync to avoid saturating cellular links.

**Cross-module reuse.**

The `@cyberskill/proj-sync` library is renamed `@cyberskill/sync` after the first ship and reused by CHAT (already in FR-CHAT-001 with a Mattermost-specific WebSocket layer; the next CHAT iteration converges) and EMAIL (FR-EMAIL-002 typing-indicator + thread-state already use a similar pattern). PROJ is the canonical implementation; the others adopt the shared library as a P2 chore.

**Audit integration.** Every applied mutation writes an audit row (the canonical PROJ audit log); rebases and conflict-drops write info-level audit rows in the same scope so a forensic query can reconstruct what the user *intended* even when the server overwrote.

**MCP tool surface.** No new MCP tools in this FR; sync is a transport, not an agent surface. FR-PROJ-008 ships the mutation tools that ride on top.

## Alternatives Considered

- **Synchronous mutations with no optimistic UI.** Rejected: round-trip latency on Vietnamese mobile makes the UX intolerable.
- **Yjs CRDT for issues** (the same library used for BRAIN Layer 1 in FR-BRAIN-001). Rejected for issues: CRDT semantics are right for free-form text but wrong for structured fields with intent (`state`, `assignee`, `priority`); last-write-wins per field is the correct semantic. Yjs is reused for issue *description* + *comment body* edits as an enhancement at P2 (FR-PROJ-RICHTEXT-001 in batch-08).
- **GraphQL subscriptions only, no WebSocket-direct sync server.** Rejected: GraphQL subscriptions are sufficient for read-only fan-out but the round-trip semantics of optimistic mutation + rebase + conflict are awkward to express in pure GraphQL; a dedicated sync endpoint with the envelope contract is cleaner.
- **Server-Sent Events instead of WebSocket.** Rejected: bidirectional flow (client mutations + server fan-out on the same connection) is what we need; SSE is one-way.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) two browsers open the same project; one Member drags an issue between cycles, the other browser sees the change in ≤ 250 ms p95; (2) a Member is taken offline mid-edit, makes 5 mutations, comes back online, and the mutations replay with the server-canonical result in ≤ 5 s p95; (3) two Members edit the same issue's `state` simultaneously; the server arbitrates and one Member sees the rebase toast.
- **Latency NFR.** Optimistic-mutation local apply ≤ 16 ms; server-confirm round-trip p95 ≤ 200 ms on local network; ≤ 600 ms on Vietnamese 4G.
- **Reliability metric.** Zero lost mutations across the 14-day P1-exit observation window. (A "lost" mutation is one in `pending_mutations` that never reaches `applied` or `conflict_dropped`.)

## Scope

**In-scope.**
- `cyberos-proj-sync` server with the envelope contract.
- IndexedDB client SDK + Zustand integration.
- `proj.mutation_log` partitioned table + monotonic LSN.
- Three-way merge per mutation kind.
- WebSocket fan-out + heartbeat + backoff.
- Offline queue + deterministic replay.
- NATS publish for cross-module consumers.
- Audit integration.

**Out-of-scope (deferred).**
- Yjs CRDT for description + comment rich-text (P2 FR-PROJ-RICHTEXT-001).
- Cross-tenant subscription federation (forbidden by design).
- Mobile native WebSocket client (P3).
- Bandwidth-throttling adjustments per Member (P2 if needed).

## Dependencies

- FR-PROJ-001 (schema).
- FR-INFRA-001 (Postgres + NATS + Cloudflare WS routing).
- FR-AUTH-001 / FR-AUTH-002.
- FR-MCP-001 (the eventual mutation MCP tools).
- Compliance: PDPL Decree 13 (mutation log contains personal data; same retention floors as the canonical audit log apply).
- Locked decisions referenced: DEC-102 (Linear sync-engine pattern), DEC-103 (server-canonical conflict resolution; client rebases), DEC-104 (per-project subscription scope; no cross-project broadcast).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The sync engine is deterministic; AI-derived behaviour layers on top in FR-PROJ-006.
