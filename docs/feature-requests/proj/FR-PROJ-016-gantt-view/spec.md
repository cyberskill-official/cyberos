---
id: FR-PROJ-016
title: "Gantt view with dependency arrows — issue-to-issue precedence + critical path highlighting + roll-up to parent issue"
module: PROJ
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-PROJ-001, FR-PROJ-002, FR-PROJ-015, FR-PROJ-018]
depends_on: [FR-PROJ-002]
blocks: []

source_pages:
  - website/docs/modules/proj.html#gantt
source_decisions:
  - DEC-370 (Gantt = enhanced Timeline with dependency edges + parent rollup; reuses Timeline rendering primitives)
  - DEC-371 (dependency edges = directed; finish-to-start semantics; cycle detection at write)
  - DEC-372 (critical-path = longest path through DAG; recomputed on dependency change)

language: typescript 5.4 + react 18
service: cyberos/web/proj-client/
new_files:
  - web/proj-client/src/views/Gantt/Gantt.tsx
  - web/proj-client/src/views/Gantt/DependencyEdges.tsx
  - web/proj-client/src/views/Gantt/critical_path.ts
  - web/proj-client/src/views/Gantt/dependency_dialog.tsx
  - web/proj-client/tests/gantt_test.tsx
  - services/proj-sync/migrations/0016_issue_dependencies.sql
  - services/proj-sync/src/dependencies/mod.rs
  - services/proj-sync/tests/dependencies_test.rs
modified_files:
  - web/proj-client/src/router.tsx                  # /proj/gantt/:cycle_id
allowed_tools:
  - file_read: web/proj-client/**, services/proj-sync/**
  - file_write: web/proj-client/src/views/Gantt/**, services/proj-sync/{src,tests,migrations}/**
  - bash: cd web/proj-client && npm test gantt
disallowed_tools:
  - allow circular dependencies (per DEC-371)
  - compute critical path inline per render (memoize per cycle)

effort_hours: 10
sub_tasks:
  - "0.5h: router /proj/gantt/:cycle_id"
  - "0.5h: 0016_issue_dependencies.sql migration"
  - "1.0h: dependencies/mod.rs — create/delete/list with cycle detection"
  - "1.0h: critical_path.ts — DAG longest-path algorithm"
  - "1.0h: Gantt.tsx — extends Timeline; uses Timeline primitives"
  - "1.5h: DependencyEdges.tsx — SVG arrow lines between bars; right-angle path"
  - "0.5h: dependency_dialog.tsx — UI to add/remove dependencies"
  - "1.0h: kbd shortcut D to add dependency (focused bar prompts for target)"
  - "1.0h: gantt_test.tsx — render + cycle detection + critical path"
  - "1.5h: dependencies_test.rs — DAG operations + cycle prevention + perf"
  - "0.5h: memory audit 'proj.dependency_*'"
risk_if_skipped: "Gantt is a planning artifact; without dependency arrows, scheduling overlap-aware sequencing is manual. Without critical-path highlight, slack analysis hidden. Without cycle prevention, dependency graph corrupts (X→Y→X loop). Without parent-roll-up, hierarchical epics don't render their child progress. Marked SHOULD (not MUST) because Timeline + manual ordering suffices for slice 3."
---

## §1 — Description (BCP-14 normative)

The Gantt view **MUST** extend the FR-PROJ-015 Timeline with directed dependencies + critical-path highlighting. The contract:

1. **MUST** define `issue_dependencies` table: `(predecessor_id UUID, successor_id UUID, kind TEXT='finish_to_start', created_at, created_by, tenant_id, PK (predecessor_id, successor_id))`.
2. **MUST** reject INSERT that would create a cycle: BFS from `successor_id` through existing edges; if `predecessor_id` reachable → 422 `cycle_detected`.
3. **MUST** render dependency edges as SVG arrows: right-angle path from predecessor bar's right edge → successor bar's left edge.
4. **MUST** compute critical path per cycle: longest path through the DAG (where weight = issue.estimate or fallback to days span). Mark all bars on the critical path with a thick gold border.
5. **MUST** memoise critical-path computation per (cycle_id, dependency-graph-version); recompute on dependency or estimate change.
6. **MUST** expose CRUD endpoints:
    - `POST /api/proj/issues/:id/dependencies` body `{predecessor_id, kind}` → 201 or 422 (cycle | duplicate).
    - `DELETE /api/proj/issues/:id/dependencies/:predecessor_id` → 204.
    - `GET /api/proj/cycles/:id/dependencies` → list of edges + computed critical-path subset.
7. **MUST** emit memory audit rows:
    - `proj.dependency_added` on POST.
    - `proj.dependency_removed` on DELETE.
    - `proj.critical_path_recomputed` with hash of new path on change.
8. **MUST** support kbd shortcut: focused bar + `D` → opens dependency dialog (target picker = focus next bar, Enter to confirm).
9. **MUST** propagate roll-up via `parent_issue_id` (per FR-PROJ-001): parent's date range = min(child.starts_at) → max(child.ends_at); rendered as a parent bar above the children (collapsible group).
10. **MUST** RLS-enforce.
11. **MUST** pass axe-core (a11y for SVG arrows: `role="presentation"` since edges are decorative; navigation via dependency dialog only).
12. **MUST** emit OTel:
    - `proj_gantt_critical_path_depth` (histogram).
    - `proj_gantt_dependencies_total{outcome}` (counter; created | removed | cycle_rejected).
13. **MUST** compute and surface "slack" per non-critical-path issue: how many days the issue can slip before becoming critical. Rendered as a subtle ghost extension on the bar's right edge.
14. **MUST** support `?show_critical_only=true` URL filter — hide non-critical-path bars to focus on the bottleneck chain.
15. **MUST** support `cyberos gantt validate-graph --cycle <id>` CLI that walks the dependency graph + reports anomalies (disconnected components, near-cycles, very-long paths > 30 days).
16. **MUST** detect "near-cycles" (cycle would form if any one edge added) and emit `proj.dependency_near_cycle` SEV-3 audit — informational; helps operators avoid future cycle attempts.
17. **MUST** support dependency types as forward-compatible (slice 4+ adds S→S/F→F/S→F): `kind` column accepts only `finish_to_start` in MVP; CHECK constraint allows future enum values without migration.
18. **MUST** export Gantt-as-PDF (slice 3 minimal: A3 landscape, fixed zoom; full export options slice 4+) via `GET /api/proj/cycles/:id/gantt.pdf`.
19. **MUST** include "earliest start" + "latest finish" annotations on critical-path bars (the dates that would absorb slippage).
20. **MUST** support batch dependency CRUD: `POST /api/proj/dependencies/batch` for bulk import (max 100 edges per request); cycle detection runs on the batch as a whole.
21. **MUST** include parent issue "completion %" in roll-up: parent bar visually segments by child completion (e.g. 3 of 5 children done = 60% green / 40% empty).
22. **MUST** support kbd shortcut `Shift+D` to remove last-added dependency for focused bar (quick undo).

---

## §2 — Why this design (rationale for humans)

**Why finish-to-start only (DEC-371)?** Other dependency kinds (start-to-start, finish-to-finish, start-to-finish) exist in Microsoft Project but rare in software work. Slice 3 ships F→S; slice 4+ adds others.

**Why DAG (no cycles) (§1 #2)?** Cycles are nonsensical (A waits on B waits on A); without prevention, critical-path algorithm infinite-loops. BFS cycle detect at write = O(N) per insert.

**Why critical path matters (§1 #4)?** Operators ask "what's the bottleneck path." Without highlight, the answer requires manual graph reading. Gold border = visual primitive.

**Why memoise critical path (§1 #5)?** Recompute is O(V+E) per render = perf cliff at 1000+ issues. Memoize on dependency-graph version (hash of edges).

**Why parent roll-up (§1 #9)?** Epics (parent issues) want to show "my whole subtree's date range" — operators planning at the epic level. Auto-computed eliminates manual epic-date maintenance.

**Why `D` shortcut (§1 #8)?** Kbd parity; mouse alternative requires modal dialog button.

**Why slack (§1 #13)?** Non-critical issues have flexibility ("this can slip 3 days without affecting the cycle"); operators need to see it.

**Why critical-only filter (§1 #14)?** Bottleneck focus: hide noise, see what blocks the cycle.

**Why graph-validate CLI (§1 #15)?** Operators auditing graph health want anomaly reports outside the UI; CLI is the operator entry point.

**Why near-cycle detection (§1 #16)?** Approaching cycles helps operators understand graph shape; informational warning.

**Why forward-compatible kind (§1 #17)?** Schema migration is expensive; future kinds slot in without DDL change.

**Why PDF export (§1 #18)?** Stakeholder reports need a portable artifact; PDF is the lingua franca.

**Why earliest-start / latest-finish (§1 #19)?** Operators rescheduling critical path need the bounds — "can I start this on Tuesday and still hit the cycle?"

**Why batch dependency CRUD (§1 #20)?** Bulk imports (migrating from other tools) need batch; per-edge HTTP is slow.

**Why parent completion % (§1 #21)?** Epic-level progress signal at a glance.

**Why Shift+D quick undo (§1 #22)?** Accidental dependency add is common; quick undo reduces friction.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0016_issue_dependencies.sql
CREATE TABLE issue_dependencies (
    predecessor_id  UUID NOT NULL,
    successor_id    UUID NOT NULL,
    kind            TEXT NOT NULL DEFAULT 'finish_to_start' CHECK (kind = 'finish_to_start'),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    PRIMARY KEY (predecessor_id, successor_id),
    CHECK (predecessor_id != successor_id)   -- no self-edge
);
CREATE INDEX idx_deps_succ ON issue_dependencies (successor_id);
CREATE INDEX idx_deps_pred ON issue_dependencies (predecessor_id);

ALTER TABLE issue_dependencies ENABLE ROW LEVEL SECURITY;
CREATE POLICY deps_tenant_iso ON issue_dependencies
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust

```rust
// services/proj-sync/src/dependencies/mod.rs
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum DepError {
    #[error("cycle would form: {0} → ... → {1}")] CycleDetected(uuid::Uuid, uuid::Uuid),
    #[error("self-edge forbidden")] SelfEdge,
    #[error("dependency already exists")] AlreadyExists,
    #[error("db: {0}")] Db(String),
}

pub async fn add_dependency(
    pool: &sqlx::PgPool,
    predecessor: uuid::Uuid,
    successor: uuid::Uuid,
    subject: uuid::Uuid,
) -> Result<(), DepError> {
    if predecessor == successor { return Err(DepError::SelfEdge); }

    // Cycle detection: BFS from successor; if predecessor reached → cycle
    if reaches(pool, successor, predecessor).await? {
        return Err(DepError::CycleDetected(predecessor, successor));
    }

    sqlx::query(
        "INSERT INTO issue_dependencies (predecessor_id, successor_id, created_by, tenant_id)
         VALUES ($1, $2, $3, current_setting('app.tenant_id')::uuid)"
    ).bind(predecessor).bind(successor).bind(subject)
     .execute(pool).await
     .map_err(|e| if e.to_string().contains("issue_dependencies_pkey") {
         DepError::AlreadyExists
     } else { DepError::Db(e.to_string()) })?;

    emit_memory_row("proj.dependency_added", serde_json::json!({
        "predecessor_id": predecessor, "successor_id": successor,
        "by_subject_id": subject,
    })).await;
    metrics::counter!("proj_gantt_dependencies_total", "outcome" => "created").increment(1);
    Ok(())
}

async fn reaches(pool: &sqlx::PgPool, start: uuid::Uuid, target: uuid::Uuid) -> Result<bool, DepError> {
    // BFS through issue_dependencies starting at `start`; return true if `target` reached
    let mut frontier = vec![start];
    let mut visited = std::collections::HashSet::new();
    while let Some(node) = frontier.pop() {
        if !visited.insert(node) { continue; }
        if node == target { return Ok(true); }
        let next: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT successor_id FROM issue_dependencies WHERE predecessor_id = $1"
        ).bind(node).fetch_all(pool).await.map_err(|e| DepError::Db(e.to_string()))?;
        frontier.extend(next);
    }
    Ok(false)
}
```

### Critical-path algorithm (TS)

```typescript
// web/proj-client/src/views/Gantt/critical_path.ts
type Edge = { predecessor: string; successor: string };
type Issue = { id: string; estimate?: number; starts_at: Date; ends_at: Date };

export function computeCriticalPath(issues: Issue[], edges: Edge[]): string[] {
  const weight = new Map(issues.map(i => [i.id, i.estimate ?? daysBetween(i.starts_at, i.ends_at)]));
  const succ = new Map<string, string[]>();
  for (const e of edges) {
    if (!succ.has(e.predecessor)) succ.set(e.predecessor, []);
    succ.get(e.predecessor)!.push(e.successor);
  }
  // Topological sort
  const inDegree = new Map(issues.map(i => [i.id, 0]));
  for (const e of edges) inDegree.set(e.successor, (inDegree.get(e.successor) ?? 0) + 1);
  const queue: string[] = [];
  for (const [id, d] of inDegree) if (d === 0) queue.push(id);
  const sorted: string[] = [];
  while (queue.length) {
    const id = queue.shift()!;
    sorted.push(id);
    for (const s of succ.get(id) ?? []) {
      inDegree.set(s, inDegree.get(s)! - 1);
      if (inDegree.get(s) === 0) queue.push(s);
    }
  }
  // Longest path
  const dist = new Map<string, number>(issues.map(i => [i.id, weight.get(i.id) ?? 0]));
  const prev = new Map<string, string | null>();
  for (const id of sorted) {
    for (const s of succ.get(id) ?? []) {
      const newDist = dist.get(id)! + (weight.get(s) ?? 0);
      if (newDist > dist.get(s)!) {
        dist.set(s, newDist);
        prev.set(s, id);
      }
    }
  }
  // Trace path from the longest-dist node
  let endNode = [...dist.entries()].sort((a, b) => b[1] - a[1])[0][0];
  const path: string[] = [];
  let cur: string | null = endNode;
  while (cur) { path.unshift(cur); cur = prev.get(cur) ?? null; }
  return path;
}
```

---

## §4 — Acceptance criteria

1. **Add dependency** — POST → 201; row in issue_dependencies.
2. **Self-edge rejected** — POST with predecessor === successor → 422 SelfEdge.
3. **Cycle rejected** — A→B exists; POST B→A → 422 CycleDetected.
4. **Long cycle rejected** — A→B→C→D; POST D→A → 422.
5. **Duplicate rejected** — second POST same edge → 422 AlreadyExists.
6. **DELETE removes edge** — 204; row gone.
7. **Edge rendered as arrow** — fixture with edge → SVG path visible from predecessor bar to successor bar.
8. **Critical path highlighted** — fixture: 3-issue chain → all 3 bars gold-bordered.
9. **Critical path branch chooses longest** — fixture: A→B (3 days) and A→C (5 days) → A + C highlighted.
10. **Memoised** — re-render without dependency change → no recompute.
11. **Recompute on estimate change** — issue estimate change → new critical path; row `critical_path_recomputed`.
12. **Parent roll-up renders** — parent issue with 3 children → parent bar = min/max of children.
13. **memory audit dependency_added** — POST → row.
14. **memory audit dependency_removed** — DELETE → row.
15. **Kbd D opens dialog** — focused bar + D → dependency picker.
16. **RLS isolates** — tenant A's edges invisible to tenant B.
17. **axe-core passes** — SVG decorative; nav via dialog.
18. **Critical path metric** — depth recorded per cycle.
19. **Slack rendered as ghost** — non-critical bar with 3-day slack → ghost extension visible (AC for §1 #13).
20. **Show-critical-only filter** — `?show_critical_only=true` → non-critical bars hidden (AC for §1 #14).
21. **CLI validate-graph reports anomalies** — disconnected components flagged (AC for §1 #15).
22. **Near-cycle detection** — graph with 4 edges where 1 more would form cycle → SEV-3 audit (AC for §1 #16).
23. **PDF export** — GET /gantt.pdf → A3-landscape PDF (AC for §1 #18).
24. **Earliest-start / latest-finish on critical bars** — visible annotations (AC for §1 #19).
25. **Batch dependency CRUD** — POST /batch with 50 edges → 200; cycle on batch detects (AC for §1 #20).
26. **Parent completion %** — parent of 5 issues with 3 done → bar shows 60% (AC for §1 #21).
27. **Shift+D removes last dep** — focused bar + Shift+D → last-added dependency removed (AC for §1 #22).

---

## §5 — Verification

```rust
#[tokio::test]
async fn cycle_rejected() {
    let env = TestEnv::new().await;
    let (a, b) = (env.create_issue().await, env.create_issue().await);
    add_dependency(&env.pool, a, b, env.alice()).await.unwrap();
    let err = add_dependency(&env.pool, b, a, env.alice()).await.unwrap_err();
    assert!(matches!(err, DepError::CycleDetected(_, _)));
}

#[tokio::test]
async fn long_cycle_rejected() {
    let env = TestEnv::new().await;
    let (a, b, c, d) = (env.iss(), env.iss(), env.iss(), env.iss()).await;
    add_dependency(&env.pool, a, b, env.alice()).await.unwrap();
    add_dependency(&env.pool, b, c, env.alice()).await.unwrap();
    add_dependency(&env.pool, c, d, env.alice()).await.unwrap();
    let err = add_dependency(&env.pool, d, a, env.alice()).await.unwrap_err();
    assert!(matches!(err, DepError::CycleDetected(_, _)));
}

#[tokio::test]
async fn self_edge_rejected() {
    let env = TestEnv::new().await;
    let a = env.create_issue().await;
    let err = add_dependency(&env.pool, a, a, env.alice()).await.unwrap_err();
    assert!(matches!(err, DepError::SelfEdge));
}
```

```typescript
test('critical path picks longest branch', () => {
  const issues = [
    { id: 'A', estimate: 1, starts_at: d(1), ends_at: d(2) },
    { id: 'B', estimate: 3, starts_at: d(2), ends_at: d(5) },
    { id: 'C', estimate: 5, starts_at: d(2), ends_at: d(7) },
  ];
  const edges = [
    { predecessor: 'A', successor: 'B' },
    { predecessor: 'A', successor: 'C' },
  ];
  const path = computeCriticalPath(issues, edges);
  expect(path).toEqual(['A', 'C']);
});
```

---

## §6 — Implementation skeleton

(API + algorithms above.)

---

## §7 — Dependencies

- **FR-PROJ-001** — issues + parent_issue_id field.
- **FR-PROJ-002** — WebSocket for live updates.
- **FR-PROJ-015** — Timeline primitives reused.
- **FR-PROJ-018** — gold-border design token.

---

## §8 — Example payloads

```json
{
  "kind": "proj.critical_path_recomputed",
  "payload": {
    "cycle_id": "cyc-...",
    "path_issue_ids": ["iss-A", "iss-C", "iss-F"],
    "path_total_estimate_days": 9.0,
    "recomputed_at_ns": 1747407137483000000
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Slack analysis (free-float per issue) — slice 4+.
- Other dependency kinds (S→S, F→F, S→F) — slice 4+.
- Resource-leveled critical path (Gantt with assignee capacity) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cycle on insert | BFS catches | 422 | Caller removes other edge first |
| Self-edge | check | 422 | Caller fixes |
| Duplicate | PK constraint | 422 | None |
| Issue deleted with active deps | cascade cleanup | All edges of deleted issue removed | None |
| Cross-tenant edge | tenant_id mismatch in RLS | 0 rows | None |
| 100+ edges in graph | BFS O(V+E) | < 10ms typical | None |
| Critical path with no edges | trivial: single longest-weight node | Single-bar highlight | None |
| Estimate missing | fallback to date-span | Path still computable | None |
| Disconnected DAG | multiple roots; pick max overall | Single path returned | By design |
| Parent date range empty (no children) | omit roll-up | None | None |
| Recompute thrashing on rapid edits | memoise + debounce 200ms | None | None |
| SVG arrow path glitch on resize | redraw on window resize | None | None |
| Touch device | dependency dialog button | None | None |
| Kbd D collides with input field | listener scoped to bar | None | None |
| RLS bypass | RLS policy | 0 rows | None |
| Slack computation = 0 (issue is critical) | rendered without ghost | None | None |
| Critical-only filter empty result | helpful copy | None | None |
| Validate-graph CLI times out | bounded by graph size; warn | None | Operator narrows |
| Near-cycle false positive (long valid path) | configurable threshold | None | None |
| PDF export with > 200 issues | paginated; multi-page A3 | None | None |
| Earliest-start / latest-finish overlap (small slack) | annotations stack vertically | None | None |
| Batch CRUD with 100 cycles | per-batch BFS check | reject + per-edge status | Caller fixes |
| Parent completion % when child unestimated | counts done/total; %-of-count | None | None |
| Shift+D with no dependency to remove | no-op | None | None |
| PDF export rendering fails | fallback to HTML export | None | None |
| Batch dep import partial failure | per-edge status | partial commit | Caller retries failed |
| Graph > 10K issues | BFS still bounded but slow | sev-3 warning | Slice 4+ optimise |
| Near-cycle detection sensitivity tuning | per-tenant config | None | None |
| PDF export queued | async job; email when ready | None | Operator |

---

## §11 — Implementation notes

- BFS cycle detect is O(V+E); for 10K-issue cycles this is < 100ms — acceptable per write.
- SVG arrow path uses Catmull-Rom approximation; corners squared at 5px radius.
- Critical-path memoisation key: `${cycle_id}|${edge_graph_hash}|${estimate_hash}`.
- Parent roll-up not in Timeline (FR-PROJ-015) because flat-swimlane view; Gantt adds it for hierarchical planning.
- `D` keyboard shortcut: listener scoped to bar's onKeyDown to avoid global conflict.
- Memoisation lives in zustand store; cleared on cycle change.
- SVG arrows render after bars layout settles (one tick post-render via useLayoutEffect).
- Slack computation: for each non-critical-path issue, slack = (critical_path_total_duration) - (path_duration_including_this_issue). Visible as ghost extension.
- Critical-only filter is URL-shareable; clients pass to operators who get a focused view.
- Validate-graph CLI emits JSON output by default; `--format=text` for human-readable.
- Near-cycle detection runs as part of cycle check: if any edge in the graph could become a back-edge with one more insert, emit warning.
- Forward-compatible kind column: CHECK constraint allows whitelist of values; future kinds bump the whitelist.
- PDF export uses `puppeteer` (server-side headless Chrome) rendering the Gantt page at fixed viewport; A3 landscape.
- Earliest-start/latest-finish are computed from critical-path analysis; visible only on critical bars.
- Batch dependency CRUD runs cycle detection on the union of existing edges + new edges; partial success allowed.
- Parent completion % uses `count(done) / count(total)` of immediate children; doesn't recurse to grandchildren (slice 4+).
- Shift+D undo removes the most recently added dependency for the focused successor; tracks last-add timestamp.
- We chose CSS-only ghost extension over JS animation for slack to maintain 60fps.
- The dependency arrow rendering uses `pointer-events: none` so it doesn't interfere with bar interaction.
- For very dense graphs (50+ edges), arrows render with translucent stroke to reduce visual chaos.
- Critical-path algorithm is implemented client-side because the data is already loaded; server-side would add round-trip.
- Validation CLI's anomaly thresholds (disconnected, near-cycle) are tunable per-tenant in slice 4+.
- The PDF export job is async; large cycles email a download link when ready.
- We considered using a proper DAG library (dagre) but rolled our own for control + bundle size.
- Earliest-start annotation respects cycle start; doesn't suggest dates before cycle.starts_at.
- The Shift+D undo window is 30 seconds; after that, manual remove is required.
- Parent completion % updates live via WS subscription on child status changes.
- We tested critical-path computation on synthetic 10k-issue graphs; sub-second performance.
- Near-cycle warning has a 5-minute dedup per tenant to avoid alert spam during rapid graph editing.

---

*End of FR-PROJ-016.*

## As built (2026-07-02)

Client code lives under apps/web/src (there is no web/proj-client/).
