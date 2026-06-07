---
id: FR-PROJ-014
title: "Kanban Board view — drag/drop status transition + keyboard-first navigation + 60fps virtualised list rendering"
module: PROJ
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-PROJ-002, FR-PROJ-003, FR-PROJ-004, FR-PROJ-017, FR-PROJ-018]
depends_on: [FR-PROJ-002]
blocks: [FR-PROJ-018]

source_pages:
  - website/docs/modules/proj.html#kanban
source_decisions:
  - DEC-350 (drag-to-column triggers FR-PROJ-004 transition; illegal transitions snap back)
  - DEC-351 (keyboard-first: every drag has equivalent J/K/arrow + Enter; WCAG AA)
  - DEC-352 (virtualised rendering: react-window for ≤ 60fps with 1000+ issues per column)

language: typescript 5.4 + react 18
service: cyberos/web/proj-client/
new_files:
  - web/proj-client/src/views/Kanban/Board.tsx
  - web/proj-client/src/views/Kanban/Column.tsx
  - web/proj-client/src/views/Kanban/IssueCard.tsx
  - web/proj-client/src/views/Kanban/DragLayer.tsx
  - web/proj-client/src/views/Kanban/KeyboardNav.tsx
  - web/proj-client/tests/kanban_test.tsx
  - web/proj-client/tests/kanban_a11y_test.tsx
modified_files:
  - web/proj-client/src/router.tsx                  # /proj/board/:cycle_id route
  - web/proj-client/package.json                    # react-window, @dnd-kit/core
allowed_tools:
  - file_read: web/proj-client/**
  - file_write: web/proj-client/src/views/Kanban/**, web/proj-client/tests/**
  - bash: cd web/proj-client && npm test kanban
disallowed_tools:
  - mouse-only interactions (per DEC-351 — keyboard parity mandatory)
  - non-virtualised list rendering for > 200 items (per DEC-352)

effort_hours: 10
sub_tasks:
  - "0.5h: router /proj/board/:cycle_id"
  - "1.0h: Board.tsx — column layout + WebSocket subscribe via YjsProvider"
  - "1.0h: Column.tsx — virtualised list (react-window FixedSizeList)"
  - "1.0h: IssueCard.tsx — title, assignee, estimate, badges"
  - "1.5h: DragLayer.tsx — @dnd-kit/core integration; on-drop calls transition endpoint"
  - "1.0h: KeyboardNav.tsx — focus indicator, J/K/arrows for navigation, Enter to open modal"
  - "0.5h: snap-back animation on illegal-transition rejection"
  - "1.5h: kanban_test.tsx — drag happy + illegal snapback + virtualised render"
  - "1.0h: kanban_a11y_test.tsx — axe-core; tab order; ARIA roles"
  - "1.0h: optimistic UI update + reconcile on FR-PROJ-002 confirmation"
risk_if_skipped: "Kanban is the bread-and-butter view; without drag/drop, every status change is a click-menu-click flow. Without keyboard nav, blind/motor-impaired operators can't use the board (WCAG fail). Without virtualisation, cycles with 1000+ issues lag the browser. Without optimistic UI, every drag has 200ms latency feedback gap."
---

## §1 — Description (BCP-14 normative)

The Kanban Board **MUST** present issues grouped by status with drag/drop transitions + keyboard parity. The contract:

1. **MUST** render 6 columns matching FR-PROJ-004 status enum: Backlog, Todo, InProgress, InReview, Done, Cancelled.
2. **MUST** subscribe to FR-PROJ-002 WebSocket for live updates; YjsProvider for description/comments.
3. **MUST** support drag-and-drop between columns via @dnd-kit/core:
    - Drop on column → POST `/api/proj/issues/:id/transition` with `to: <new_status>`.
    - Illegal transition (422 from FR-PROJ-004) → snap card back to original column with 250ms spring animation.
    - Successful transition → emit toast + audit-trail link.
    - Optimistic UI: card visually moves immediately; reconciled on server confirmation.
4. **MUST** support keyboard navigation parity:
    - Tab/Shift-Tab cycles focus through columns + cards.
    - J / K / Down / Up move focus within column.
    - H / L / Left / Right move focus between columns.
    - Enter opens FR-PROJ-017 Brief Modal for focused card.
    - Cmd/Ctrl + Shift + → moves focused card to next-rightward column (if legal); ← to next-leftward.
    - Esc dismisses any open transient UI.
5. **MUST** virtualise long columns via `react-window FixedSizeList` when > 200 items; 60fps maintained.
6. **MUST** render IssueCard with: title (≤ 2 lines truncated), assignee avatar, estimate badge, priority indicator, blocker count badge (FR-PROJ-011), labels.
7. **MUST** show real-time presence indicators (FR-PROJ-003 awareness): cursor + selection of other users editing the same issue (when in Brief Modal).
8. **MUST** emit memory audit row `proj.kanban_card_moved` per drag-induced transition; payload `{issue_id, from_status, to_status, by_subject_id, was_keyboard: bool, trace_id}`.
9. **MUST** emit OTel client-side metrics via `web-vitals`:
    - `proj_kanban_render_p95_ms` (LCP for board page).
    - `proj_kanban_drag_latency_ms` (drag start → drop reflected).
    - `proj_kanban_transitions_total{outcome}` (counter; outcome ∈ accepted | rejected | optimistic_rollback).
10. **MUST** pass axe-core a11y audit: no critical/serious violations; ARIA roles for `application`/`group`/`listitem` correct; keyboard-only test passes.
11. **MUST** handle WebSocket disconnect gracefully: banner "offline; changes will sync when connected"; drags queued in offline buffer (FR-PROJ-003 §1 #8).
12. **MUST** support `?member=<uuid>` and `?label=<id>` URL query filters; updates URL on filter change for shareability.
13. **MUST** support WIP (Work-In-Progress) limits per column when configured: `cyberos_proj_engagement_settings.wip_limits = {in_progress: 5, in_review: 3}`. Drag into a column at-or-above limit → warning banner + soft block (user can confirm to proceed; emit `proj.wip_limit_overridden` audit).
14. **MUST** support card minification: at zoom level < 75% OR per-engagement preference, render compact cards (title only, no badges). Reduces visual noise on dense boards.
15. **MUST** support bulk operations via multi-select: Shift-click selects range; Cmd/Ctrl-click toggles; bulk drag moves all selected. Bulk transitions emit one memory row per issue (per FR-PROJ-002 §1 #16).
16. **MUST** support quick-add via keyboard: pressing `c` from column-focused state opens an inline issue creator at the top of that column with status=column's status. ESC cancels; Enter creates.
17. **MUST** support swimlanes by assignee: optional view mode `?swimlanes=assignee` renders rows per assignee with status columns. Helps team standups.
18. **MUST** maintain scroll position across re-renders (e.g. WebSocket update doesn't reset scroll). Test: scroll column to 50%, receive update, scroll position preserved.
19. **MUST** support undo for the last drag operation via Cmd/Ctrl+Z within 5 seconds; emits `proj.kanban_card_move_undone` audit; reverses the transition.
20. **MUST** show drag-target preview: ghost card appears in target column at expected insert position during drag; users see exactly where it'll land.
21. **MUST** support keyboard-driven card reorder within column: J/K with `Shift` held reorders the card (vs. moving focus); persists card order in `cyberos_proj_kanban_order` table.
22. **MUST** include real-time issue count + WIP overflow indicators in column headers: "InProgress 7/5 ⚠" when over WIP limit.

---

## §2 — Why this design (rationale for humans)

**Why @dnd-kit (DEC-351 enabling)?** Provides keyboard parity out of the box (focus-management hooks); react-dnd is mouse-first. WCAG AA mandates parity; @dnd-kit accelerates compliance.

**Why optimistic UI (§1 #3)?** 200ms reconciliation gap feels janky; optimistic move + rollback-on-error matches expected drag latency.

**Why snap-back animation (§1 #3)?** Without animation, rejection is invisible; user assumes drag succeeded. Spring animation signals "rejected; try again."

**Why react-window FixedSizeList (DEC-352)?** Variable size = layout recalculation thrash; fixed size = O(1) per scroll frame. Issue cards are visually uniform anyway.

**Why keyboard shortcut for cross-column move (§1 #4)?** Power users move many issues quickly; mouse drag for 50 cards is friction. Cmd+Shift+→ is the discoverable shortcut.

**Why URL filter sync (§1 #12)?** Operators share "look at Alice's bug backlog" via URL — board state must be URL-encodable.

**Why audit was_keyboard flag (§1 #8)?** Distinguishes user behaviour patterns; informs UX research ("kb users move 3× faster but make 30% more illegal-attempts").

**Why soft WIP limit (§1 #13)?** Hard block frustrates operators; soft warn-and-confirm respects autonomy while flagging overflow.

**Why card minification (§1 #14)?** Dense boards (50+ cards visible) overload visual processing; compact mode trades detail for density.

**Why bulk operations (§1 #15)?** Bulk status moves at sprint planning are common; single-card drag for 30 cards is friction.

**Why quick-add `c` (§1 #16)?** Keyboard-driven workflows need card creation without context switching. `c` is the standard issue-create keystroke in Linear / GitHub Issues.

**Why swimlanes (§1 #17)?** Standup view: "what is each person doing" maps naturally to rows-per-person.

**Why scroll preservation (§1 #18)?** WebSocket updates re-render; scrolling-to-the-top is jarring during browsing.

**Why undo (§1 #19)?** Accidental drag is the most common kanban error; undo within 5s = forgiveness without overhead.

**Why drag-target preview (§1 #20)?** Card order matters within column; preview removes guesswork about insert position.

**Why keyboard reorder (§1 #21)?** Operators reordering for prioritisation need keyboard parity; Shift+J/K is the discoverable pattern.

**Why WIP overflow indicator (§1 #22)?** Operators glancing at the board see overflow immediately; "5/3 ⚠" is visual quick-glance signal.

---

## §3 — API contract (component sketches)

```tsx
// web/proj-client/src/views/Kanban/Board.tsx
export function Board({ cycleId }: { cycleId: string }) {
  const issues = useIssuesForCycle(cycleId);
  const [filter, setFilter] = useUrlFilter();
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: kbdCoordinateGetter }),
  );
  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
      <div role="application" aria-label="Kanban board">
        {COLUMNS.map(status => (
          <Column key={status} status={status}
                  items={issues.filter(i => i.status === status && matchesFilter(i, filter))} />
        ))}
        <DragLayer />
      </div>
    </DndContext>
  );

  async function handleDragEnd(ev: DragEndEvent) {
    const issueId = ev.active.id as string;
    const newStatus = ev.over?.id as IssueStatus | undefined;
    if (!newStatus) return;
    const issue = issues.find(i => i.id === issueId)!;
    if (!isLegal(issue.status, newStatus)) {
      toast('Cannot move there', 'error');
      return;   // dnd-kit snaps back automatically
    }
    optimisticMove(issueId, newStatus);
    try {
      const reason = requiresReason(issue.status, newStatus) ? await promptReason() : undefined;
      await postTransition(issueId, newStatus, reason);
      emitMemory('proj.kanban_card_moved', { issueId, from: issue.status, to: newStatus,
                                              wasKeyboard: ev.activatorEvent instanceof KeyboardEvent });
    } catch (e) {
      rollbackMove(issueId);
      toast('Transition rejected', 'error');
    }
  }
}
```

```tsx
// web/proj-client/src/views/Kanban/Column.tsx
import { FixedSizeList as List } from 'react-window';

export function Column({ status, items }: { status: IssueStatus; items: Issue[] }) {
  const VIRTUALISE_THRESHOLD = 200;
  return (
    <div className="kanban-column" role="group" aria-label={status}>
      <header><h2>{status}</h2><span>{items.length}</span></header>
      {items.length > VIRTUALISE_THRESHOLD ? (
        <List height={600} itemCount={items.length} itemSize={92} width="100%">
          {({ index, style }) => (
            <div style={style}><IssueCard issue={items[index]} /></div>
          )}
        </List>
      ) : (
        items.map(i => <IssueCard key={i.id} issue={i} />)
      )}
    </div>
  );
}
```

```tsx
// web/proj-client/src/views/Kanban/KeyboardNav.tsx
function kbdCoordinateGetter(event: KeyboardEvent, args: any) {
  // J/K = down/up, H/L = left/right
  // Cmd+Shift+→ = move card to next-rightward legal column
  // Implementation per @dnd-kit/sortable docs
}
```

---

## §4 — Acceptance criteria

1. **6 columns render** — board mounts with all 6 status columns.
2. **Drag valid → server accepts** — drag Todo→InProgress → POST returns 200; card stays in InProgress.
3. **Drag illegal → snap back** — drag Backlog→InReview → 422; card snaps back with animation; toast shown.
4. **Optimistic UI** — drag → card visually moves before server response (verified via mock latency).
5. **Reason prompt on re-open** — drag Done→InProgress → reason modal appears; submit moves card.
6. **Keyboard nav: Tab moves between columns** — Tab cycles through column headers + cards.
7. **Keyboard nav: J/K within column** — focus moves down/up.
8. **Keyboard nav: Cmd+Shift+→ moves to next legal column** — works only when target is legal.
9. **Enter opens Brief Modal** — focused card + Enter → FR-PROJ-017 modal opens.
10. **Esc dismisses overlays** — Esc closes modal / toast / reason prompt.
11. **Virtualised at > 200 items** — fixture with 500 items → only ~30 rendered DOM nodes.
12. **60fps maintained on scroll** — Chrome devtools performance trace shows no frame > 16ms during scroll.
13. **Awareness indicator visible** — second user opens same Brief Modal → cursor indicator shows on first user's view.
14. **Offline banner appears** — disconnect WebSocket → banner; drags queued.
15. **URL filter sync** — filter by member → URL updates; reload preserves filter.
16. **axe-core passes** — no critical/serious violations on board page.
17. **memory audit row per move** — every drag-induced transition → row.
18. **was_keyboard flag** — keyboard-triggered move → flag true; mouse → false.
19. **Optimistic rollback metric** — server reject → counter `optimistic_rollback` increments.
20. **LCP p95 < 2.5s** — empirical on 1000-issue board.
21. **WIP soft block** — drag into column at WIP limit → confirm dialog; confirm proceeds + audit (AC for §1 #13).
22. **Compact card at low zoom** — set zoom 50% → cards render compact (no badges) (AC for §1 #14).
23. **Bulk-select via Shift-click** — Shift-click two cards → range selected; bulk drag moves both (AC for §1 #15).
24. **Quick-add `c` opens creator** — Tab to column header + `c` → inline creator (AC for §1 #16).
25. **Swimlanes by assignee** — `?swimlanes=assignee` → rows per assignee (AC for §1 #17).
26. **Scroll preserved across update** — scroll to 50%, receive WS update → scroll preserved (AC for §1 #18).
27. **Undo within 5s** — drag card; Cmd+Z within 5s → card reverts; `proj.kanban_card_move_undone` row (AC for §1 #19).
28. **Drag-target preview shows position** — ghost card appears in target at insert position (AC for §1 #20).
29. **Shift+J/K reorders** — focused card + Shift+J → moves down within column; persists to DB (AC for §1 #21).
30. **WIP overflow indicator in header** — column with 7 cards + WIP=5 → header shows "7/5 ⚠" (AC for §1 #22).

---

## §5 — Verification

```typescript
// kanban_test.tsx
test('drag valid status accepted', async () => {
  const { user } = render(<Board cycleId={cycle.id} />);
  const card = screen.getByText(/My Issue/);
  await user.drag(card, screen.getByRole('group', { name: 'in_progress' }));
  await waitFor(() => expect(card).toHaveAttribute('data-status', 'in_progress'));
});

test('drag illegal status snaps back', async () => {
  const { user } = render(<Board cycleId={cycle.id} />);
  mockTransition.fail422();
  const card = screen.getByText(/Backlog Issue/);
  await user.drag(card, screen.getByRole('group', { name: 'in_review' }));
  await waitFor(() => expect(card).toHaveAttribute('data-status', 'backlog'));
  expect(screen.getByRole('alert')).toHaveTextContent(/cannot move/i);
});

test('keyboard nav: J moves focus down', async () => {
  const { user } = render(<Board cycleId={cycle.id} />);
  const first = screen.getAllByTestId('issue-card')[0];
  first.focus();
  await user.keyboard('j');
  expect(screen.getAllByTestId('issue-card')[1]).toHaveFocus();
});

test('virtualised at 500 items', async () => {
  setupCycle({ issueCount: 500 });
  render(<Board cycleId={cycle.id} />);
  const cards = screen.queryAllByTestId('issue-card');
  expect(cards.length).toBeLessThan(50);
});
```

```typescript
// kanban_a11y_test.tsx
test('axe-core passes', async () => {
  const { container } = render(<Board cycleId={cycle.id} />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});

test('keyboard-only workflow', async () => {
  const { user } = render(<Board cycleId={cycle.id} />);
  await user.tab();   // first column
  await user.tab();   // first card
  await user.keyboard('{Enter}');   // Brief Modal opens
  expect(screen.getByRole('dialog')).toBeVisible();
});
```

---

## §6 — Implementation skeleton

(Component sketches above.)

---

## §7 — Dependencies

- **FR-PROJ-002** — WebSocket sync.
- **FR-PROJ-003** — Y.Doc + LWW + awareness.
- **FR-PROJ-004** — status FSM + transition endpoint.
- **FR-PROJ-017 (sibling)** — Brief Modal opened on Enter.
- **FR-PROJ-018 (downstream)** — design tokens + a11y CI.

---

## §8 — Example payloads

```json
{
  "kind": "proj.kanban_card_moved",
  "payload": {
    "issue_id":    "iss-...",
    "from_status": "todo",
    "to_status":   "in_progress",
    "by_subject_id": "7e57c0de-...",
    "was_keyboard": false,
    "trace_id":   "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- WIP limits per column — slice 4+; ops-configurable.
- Multi-cycle board (cross-cycle view) — slice 4+.
- Swimlanes by assignee — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Server rejects transition | 422 | Snap back + toast | User retries |
| WS disconnect mid-drag | useWebSocketStatus | Queue in offline buffer | Reconnect drains |
| 1000+ items in one column | virtualisation | 60fps; no jank | None |
| Keyboard nav loses focus | focus-trap fallback | Re-focus first column | None |
| Drag from filtered view | filter still applies; underlying issue moves | Card disappears from view post-drag | By design |
| Awareness flood (10 users) | YjsProvider throttle | 30Hz cap | None |
| LCP regression | web-vitals reporting | sev-2 alarm | Investigate via Lighthouse |
| Browser without IntersectionObserver | polyfill via npm | None | None |
| Touch device | @dnd-kit TouchSensor | Drag works on iPad | None |
| Browser zoom > 200% | reflow gracefully; no overlap | Usable | None |
| Cross-tab sync via BroadcastChannel | OPTIONAL slice 4+ | None | None |
| RTL locale | flex direction reversed | Columns flow right-to-left | None |
| Issue deleted during drag | server returns 404; toast | Card removed | None |
| Optimistic move + rollback ordering | UI state machine | Last-write-wins on local state | None |
| WIP soft block but operator overrides | audit emitted | None | None |
| WIP limits not configured | feature absent | works without WIP | None |
| Compact card cuts off important info | hover shows tooltip | None | Operator can re-enlarge |
| Bulk drag with > 50 cards | bounded; warn at 30 | bulk transition slow but completes | None |
| Quick-add with invalid title | inline validation | error shown; not created | None |
| Swimlanes with many assignees (50+) | virtualised; collapsible | None | None |
| Scroll preservation breaks on layout change | property test | None | Author fixes |
| Undo after 5s | rejected | None | None |
| Drag preview lag at 60fps boundary | RAF-throttled | smooth | None |
| Shift+J reorder race with WS update | conflict resolution via LWW | None | None |
| WIP overflow indicator wraps | CSS layout | None | None |
| Quick-add `c` triggers in input field | event filter (no in inputs) | None | None |
| Touch device drag-and-drop with bulk select | partial support; warn | bulk single-card only on touch | None |
| Locale RTL with swimlanes | reflow correctly | None | None |
| WS connection flapping mid-undo | preserves intent; reapplies on reconnect | None | None |

---

## §11 — Implementation notes

- @dnd-kit/core provides `useSensor(KeyboardSensor)`; we override `coordinateGetter` to map J/K/H/L to coordinate changes.
- `react-window` requires fixed `itemSize`; we measure once at mount and assume cards are uniform height.
- Optimistic state uses zustand store with rollback function returned at action time.
- Awareness rendering uses YjsProvider's `awarenessChange` event; throttled by 30Hz cap.
- The `was_keyboard` flag detects via `event instanceof KeyboardEvent` on dnd-kit's activator event.
- Color tokens for column headers come from FR-PROJ-018.
- Brief Modal is lazy-loaded via React.lazy to keep Board bundle ≤ 200 KB gzip.
- Filter URL serialization: `?member=<uuid>&label=<id>` — simple flat schema.
- WIP limits enforce soft-block via dialog rather than hard reject because: enforcing too strictly demotivates flow; soft signal preserves autonomy.
- Compact card threshold is browser zoom % AND per-engagement preference; either triggers.
- Bulk operations use a temporary `selected_ids: Set<string>` in zustand; cleared on action complete.
- Quick-add `c` is the canonical issue-create shortcut across modern tools (Linear, GitHub); user familiarity drives adoption.
- Swimlanes by assignee are an additional axis (rows) layered onto columns; virtualised per-row for performance.
- Scroll position preservation uses `ScrollRestoration` from react-router or manual `scrollTop` cache; tested across WS updates.
- Undo within 5s uses a debounced timer; after 5s, undo is unavailable. Why 5s: balances forgiveness against operator clarity ("the move stuck after a moment").
- Drag-target preview is provided by @dnd-kit's `useDroppable` + custom ghost overlay; performance is GPU-accelerated transform.
- Keyboard reorder (Shift+J/K) updates the `kanban_order` column; persisted to enable cross-device order consistency.
- WIP overflow indicator uses `aria-live="polite"` so screen readers announce the overflow.
- Quick-add `c` event listener filters out typing in input/textarea elements; only triggers when focus is on column / card.
- We considered animated transitions for swimlane rearrange but kept simple to maintain 60fps on low-end devices.
- Touch device support for bulk select is limited (no Shift key); users tap individual cards then drag. Future slice 4+ adds long-press multi-select.
- RTL locale handling: column flex-direction reverses; J/K maps to logical direction (J=down regardless of LTR/RTL).
- WebSocket reconnect after undo: the undo operation is the persisted state; reconnect resyncs.
- Drag preview rendering uses `pointer-events: none` ghost element to avoid interfering with hit-testing.
- The 5-second undo window emits a small toast "Press Cmd+Z to undo" so operators discover the feature.
- WIP limits per column are stored in JSONB; operators set via engagement settings UI.
- Compact card mode preserves keyboard navigation; only the visual presentation changes.
- Multi-select keyboard support (slice 4+): Shift+Arrow extends selection.

---

*End of FR-PROJ-014.*
