---
id: TASK-PROJ-015
title: "Timeline view — cycle window × assignee swimlane with day-grid layout, drag-resize for date changes, and milestone markers"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-002, TASK-PROJ-003, TASK-PROJ-007, TASK-PROJ-014, TASK-PROJ-018]
depends_on: [TASK-PROJ-002]
blocks: []

source_pages:
  - website/docs/modules/proj.html#timeline
source_decisions:
  - DEC-360 (timeline = X-axis days × Y-axis assignees; one row per active member)
  - DEC-361 (resize-bar = LWW updates to issue.starts_at + issue.ends_at; immediate broadcast)
  - DEC-362 (Fixed-Fee milestones from TASK-PROJ-007 surface as gold-bordered markers)

language: typescript 5.4 + react 18
service: cyberos/web/proj-client/
new_files:
  - web/proj-client/src/views/Timeline/Timeline.tsx
  - web/proj-client/src/views/Timeline/Swimlane.tsx
  - web/proj-client/src/views/Timeline/DayGrid.tsx
  - web/proj-client/src/views/Timeline/IssueBar.tsx
  - web/proj-client/src/views/Timeline/MilestoneMarker.tsx
  - web/proj-client/tests/timeline_test.tsx
modified_files:
  # /proj/timeline/:cycle_id route
  - web/proj-client/src/router.tsx
allowed_tools:
  - file_read: web/proj-client/**
  - file_write: web/proj-client/src/views/Timeline/**, web/proj-client/tests/**
  - bash: cd web/proj-client && npm test timeline
disallowed_tools:
  - render > 100 swimlanes (large teams paginate)
  - skip kbd parity for date-resize (per TASK-PROJ-014 precedent)

effort_hours: 8
subtasks:
  - "0.5h: router /proj/timeline/:cycle_id"
  - "1.0h: Timeline.tsx — orchestrator + zoom controls (day/week/month)"
  - "1.0h: DayGrid.tsx — column-per-day with date headers; today indicator"
  - "1.0h: Swimlane.tsx — one row per assignee; lazy-render off-screen rows"
  - "1.5h: IssueBar.tsx — positioned by starts_at/ends_at; drag to move, edge-drag to resize"
  - "0.5h: MilestoneMarker.tsx — gold vertical bars at TASK-PROJ-007 milestones"
  - "1.0h: kbd parity (J/K swimlanes; H/L days; Shift+Arrow resize ±1day)"
  - "1.0h: timeline_test.tsx — drag-resize + milestone render + kbd"
  - "0.5h: PATCH endpoint integration (LWW per TASK-PROJ-003)"
risk_if_skipped: "Timeline is the canonical 'when' view; Kanban shows 'what' but not 'how long'. Without milestone markers, Fixed-Fee progress is invisible. Without drag-resize, every date change is a form submit. Without lazy swimlane render, 50-person teams jank. Without kbd parity, motor-impaired ops can't use."
---

## §1 — Description (BCP-14 normative)

The Timeline view **MUST** render issues as horizontal bars on a day-grid × assignee swimlane. The contract:

1. **MUST** present X-axis = days within cycle window (defaults: cycle.starts_at → cycle.ends_at; configurable zoom day/week/month).
2. **MUST** present Y-axis = swimlanes, one per assignee active in cycle (i.e. assigned to ≥1 issue in the cycle); ordered alphabetically; "Unassigned" lane at top.
3. **MUST** render each issue as a horizontal bar positioned by `starts_at..ends_at`. Bar height proportional to swimlane row height; bar colour from status enum (TASK-PROJ-018 design tokens).
4. **MUST** support drag-to-move: dragging mid-bar changes both starts_at and ends_at by Δdays.
5. **MUST** support edge-drag-to-resize: left edge changes starts_at; right edge changes ends_at. Minimum bar = 1 day; resize past min snaps.
6. **MUST** PATCH date changes via TASK-PROJ-003 LWW scalar handlers (issue.starts_at + issue.ends_at are LWW fields).
7. **MUST** render milestone markers from TASK-PROJ-007 Fixed-Fee config: vertical gold-bordered line at `target_date`; hover tooltip shows milestone name + amount.
8. **MUST** show today-indicator: vertical thin line marking current date; subtle pulse animation.
9. **MUST** support keyboard navigation parity:
    - Tab cycles swimlanes + bars.
    - J/K move swimlanes; H/L move within swimlane.
    - Shift+→/← extend/shrink bar by 1 day.
    - Shift+Cmd+→/← move bar by 1 day.
    - Enter opens Brief Modal.
10. **MUST** lazy-render swimlanes off-screen via IntersectionObserver; ≥ 60fps with 50+ swimlanes.
11. **MUST** emit memory audit `proj.timeline_bar_moved` per resize/move with `{issue_id, field, before, after, was_keyboard, trace_id}`.
12. **MUST** emit OTel client metrics:
    - `proj_timeline_render_p95_ms`.
    - `proj_timeline_resize_latency_ms`.
13. **MUST** RLS-enforce (issues + assignees only for tenant).
14. **MUST** pass axe-core (per TASK-PROJ-018 a11y CI).
15. **MUST** support workload overlap visualisation: when multiple bars in same swimlane overlap (one assignee has 3 concurrent issues), stack them vertically OR show a "3" badge on the densest period.
16. **MUST** support `?from=&to=` URL params overriding cycle window (operator scrolls beyond cycle); preserve in URL for shareability.
17. **MUST** support snap-to-week + snap-to-month at week/month zoom: drag end snaps to nearest week boundary; precise date entry via Brief Modal.
18. **MUST** support per-assignee filter: `?assignee=<uuid>` shows only that swimlane (full-width). Useful for 1:1 reviews.
19. **MUST** include dependency arrows between issues (basic, slice-3 minimal version): when issue A `depends_on` issue B, render thin arrow from B's right edge to A's left edge. Full Gantt is TASK-PROJ-016.
20. **MUST** support "show non-active members" toggle: by default, only members with ≥1 issue in cycle; toggle shows all team members (even with no issues) to plan capacity.
21. **MUST** highlight bars that span weekends/holidays differently (faded weekend background) so operators see when work is planned across non-business days.
22. **MUST** support keyboard reordering of swimlanes: focused swimlane + Shift+Up/Down moves it; persists in `cyberos_proj_timeline_swimlane_order` per user.
23. **MUST** include cycle goal/theme banner above the timeline: shows cycle name + goal text + days-remaining count.

---

## §2 — Why this design (rationale for humans)

**Why day-grid × assignee (DEC-360)?** Two questions ops ask: "what is everyone doing this week" + "when does X land." Swimlane answers both — vertical scan shows team coverage; horizontal scan shows duration.

**Why LWW resize (DEC-361)?** Date changes are simple scalars; TASK-PROJ-003 already provides LWW infrastructure. Two users dragging same bar → last-writer-wins with subject_id tie-break.

**Why milestone markers (DEC-362)?** Fixed-Fee engagements bill at milestones; visible markers show "are we tracking to next milestone." Without them, milestones are invisible until billing closes.

**Why zoom (§1 #1)?** A 6-month engagement at day-resolution = 180 columns; too wide. Week-zoom = 26 columns. Month-zoom = 6. Operator chooses based on planning horizon.

**Why lazy swimlane render (§1 #10)?** 50-person teams produce 50 swimlanes × 60 days × 5 bars/lane = thousands of DOM nodes. Off-screen lanes don't render until scrolled to.

**Why 1-day minimum (§1 #5)?** Sub-day estimates aren't tracked (time-entries are; but issue planning works in days). Snap-to-minimum prevents zero-width bars.

**Why overlap visualisation (§1 #15)?** Operator scanning workload should see "Alice has 3 concurrent issues" — overlap signals over-commitment.

**Why URL window override (§1 #16)?** Operators want to scroll past the cycle boundary (look at next cycle's work-in-flight); URL preserves the view for sharing.

**Why snap-to-week/month (§1 #17)?** At week zoom, single-day precision in drag is hard; snap-to-week matches the visual grid. Precise dates available in modal.

**Why per-assignee filter (§1 #18)?** 1:1 reviews focus on one person's work; full-width view = readable.

**Why minimal dependency arrows here (§1 #19)?** Operators in slice-3 want basic "X depends on Y" visualisation; full Gantt (critical path, parallel chains) is TASK-PROJ-016.

**Why show non-active members (§1 #20)?** Capacity planning: "who's free to take this on?" requires seeing empty swimlanes.

**Why weekend highlight (§1 #21)?** Work-planning-on-weekends often unintentional; visual cue surfaces.

**Why kbd swimlane reorder (§1 #22)?** Operators prioritise lane order (team-lead first, etc.); keyboard parity for ordering.

**Why cycle goal banner (§1 #23)?** Context-setting: what is this cycle for? Days remaining = urgency signal.

---

## §3 — API contract

```tsx
// web/proj-client/src/views/Timeline/Timeline.tsx
type Zoom = 'day' | 'week' | 'month';

export function Timeline({ cycleId }: { cycleId: string }) {
  const [zoom, setZoom] = useState<Zoom>('day');
  const cycle = useCycle(cycleId);
  const issues = useIssuesForCycle(cycleId);
  const milestones = useMilestonesForCycle(cycleId);
  const swimlanes = useMemo(() => groupBySwimlane(issues), [issues]);

  return (
    <div className="timeline" role="application" aria-label="Timeline view">
      <ZoomControls value={zoom} onChange={setZoom} />
      <DayGrid start={cycle.starts_at} end={cycle.ends_at} zoom={zoom}>
        {milestones.map(m => <MilestoneMarker key={m.id} date={m.target_date} milestone={m} />)}
      </DayGrid>
      <TodayIndicator />
      {swimlanes.map(lane => (
        <Swimlane key={lane.assigneeId ?? 'unassigned'} lane={lane}>
          {lane.issues.map(issue => <IssueBar key={issue.id} issue={issue} zoom={zoom} />)}
        </Swimlane>
      ))}
    </div>
  );
}
```

```tsx
// web/proj-client/src/views/Timeline/IssueBar.tsx
export function IssueBar({ issue, zoom }: { issue: Issue; zoom: Zoom }) {
  const [draftDates, setDraftDates] = useState<{starts_at: Date; ends_at: Date}>({
    starts_at: issue.starts_at, ends_at: issue.ends_at,
  });

  const left  = computeOffset(draftDates.starts_at, zoom);
  const width = computeWidth(draftDates.starts_at, draftDates.ends_at, zoom);

  return (
    <div className="issue-bar"
         style={{ left, width, backgroundColor: tokens.status[issue.status] }}
         role="button" tabIndex={0}
         aria-label={`${issue.title} from ${fmt(issue.starts_at)} to ${fmt(issue.ends_at)}`}
         onKeyDown={handleKbd}>
      <ResizeHandle edge="left"  onDrag={Δ => setStartsAt(addDays(issue.starts_at, Δ))} />
      <span className="title">{issue.title}</span>
      <ResizeHandle edge="right" onDrag={Δ => setEndsAt(addDays(issue.ends_at, Δ))} />
    </div>
  );

  async function setStartsAt(newDate: Date) {
    setDraftDates(d => ({ ...d, starts_at: newDate }));
    const res = await writeScalarLWW(issue.id, 'starts_at', newDate.toISOString(), jwt);
    if (!res.accepted) { rollbackDates(); toast('Stale write; refreshed'); }
    emitMemory('proj.timeline_bar_moved', { issue_id: issue.id, field: 'starts_at',
                                            before: issue.starts_at, after: newDate,
                                            was_keyboard: false });
  }

  function handleKbd(e: React.KeyboardEvent) {
    if (e.shiftKey && e.key === 'ArrowRight') {
      setEndsAt(addDays(issue.ends_at, 1));
    } else if (e.shiftKey && e.key === 'ArrowLeft') {
      setEndsAt(addDays(issue.ends_at, -1));
    }
    // ... Cmd+Shift+Arrow for move, etc.
  }
}
```

---

## §4 — Acceptance criteria

1. **Day-grid renders cycle window** — cycle Jan 1 → Mar 31 → 90 day columns at day-zoom.
2. **Swimlanes for active assignees** — fixture: 3 members assigned → 3 lanes (+ Unassigned if applicable).
3. **Issue bar positioned correctly** — starts_at Mar 5, ends_at Mar 10 → bar at days 64..68 from cycle start.
4. **Drag move shifts both dates** — drag bar 3 days right → starts_at + 3, ends_at + 3; PATCH fires.
5. **Edge-resize updates one date** — drag right edge → ends_at only updates.
6. **1-day minimum enforced** — drag right edge left past starts_at → snaps to starts_at + 1.
7. **Milestone markers render** — Fixed-Fee milestones → gold lines at target_date.
8. **Today indicator pulses** — current date → animated vertical line.
9. **Zoom day→week** — bar width recomputes; columns coalesce.
10. **Kbd: Shift+→ extends end by 1 day** — focused bar + shortcut → ends_at +1.
11. **Kbd: Cmd+Shift+→ moves bar 1 day forward** — both dates +1.
12. **Lazy swimlane render** — 50 swimlanes; ~10 visible → only those rendered.
13. **LWW reject on stale** — concurrent edit detected → rollback + toast.
14. **memory audit per move** — `proj.timeline_bar_moved` row.
15. **OTel resize latency metric** — drag completion → histogram populated.
16. **axe-core passes** — no critical/serious violations.
17. **Hover milestone tooltip** — hover gold line → tooltip with name + amount_minor formatted.
18. **RLS isolates** — tenant A's cycle invisible to tenant B.
19. **Overlap stacking** — 3 concurrent bars in one swimlane → stacked vertically; badge if > 5 (AC for §1 #15).
20. **URL window override** — `?from=2026-01-01&to=2026-06-30` → renders extended window; URL preserved (AC for §1 #16).
21. **Snap-to-week at week zoom** — drag to mid-week at week zoom → snaps to nearest week boundary (AC for §1 #17).
22. **Per-assignee filter** — `?assignee=<uuid>` → single swimlane full-width (AC for §1 #18).
23. **Dependency arrow rendered** — A depends_on B → arrow from B's right to A's left (AC for §1 #19).
24. **Show non-active members toggle** — toggle on → empty swimlanes appear (AC for §1 #20).
25. **Weekend background faded** — Sat/Sun column has faded bg color (AC for §1 #21).
26. **Kbd swimlane reorder** — focused swimlane + Shift+Up → moves up; persists (AC for §1 #22).
27. **Cycle banner shows goal + days remaining** — banner above timeline (AC for §1 #23).

---

## §5 — Verification

```typescript
test('issue bar positions by starts_at/ends_at', () => {
  const cycle = mkCycle({ starts_at: '2026-01-01', ends_at: '2026-03-31' });
  const issue = mkIssue({ starts_at: '2026-01-05', ends_at: '2026-01-10' });
  render(<Timeline cycleId={cycle.id} />);
  const bar = screen.getByLabelText(/from 2026-01-05/);
  expect(parseInt(bar.style.left)).toBeCloseTo(4 * DAY_PX, 0);
  expect(parseInt(bar.style.width)).toBeCloseTo(5 * DAY_PX, 0);
});

test('edge-resize updates ends_at only', async () => {
  const { user } = render(<Timeline cycleId={cycle.id} />);
  const bar = screen.getByTestId('issue-bar-iss-1');
  const rightHandle = within(bar).getByTestId('resize-right');
  await user.drag(rightHandle, { delta: { x: 2 * DAY_PX, y: 0 } });
  expect(mockPatch).toHaveBeenCalledWith('iss-1', 'ends_at', expect.any(String));
  expect(mockPatch).not.toHaveBeenCalledWith('iss-1', 'starts_at', expect.anything());
});

test('1-day minimum snap', async () => {
  const issue = mkIssue({ starts_at: '2026-01-05', ends_at: '2026-01-10' });
  const { user } = render(<Timeline cycleId={cycle.id} />);
  const rightHandle = screen.getByTestId('resize-right');
  await user.drag(rightHandle, { delta: { x: -100 * DAY_PX, y: 0 } });
  // ends_at can't go past starts_at + 1
  expect(mockPatch).toHaveBeenLastCalledWith('iss-1', 'ends_at', '2026-01-06T...');
});

test('milestone markers from fixed-fee', () => {
  const eng = mkEngagement({ billing_mode: FixedFee, milestones: [
    { id: 'm1', target_date: '2026-02-15', amount_minor: 50_000_000 }
  ]});
  render(<Timeline cycleId={cycle.id} />);
  expect(screen.getByTestId('milestone-marker-m1')).toBeInTheDocument();
});

test('kbd Shift+Right extends', async () => {
  const { user } = render(<Timeline cycleId={cycle.id} />);
  const bar = screen.getByTestId('issue-bar-iss-1');
  bar.focus();
  await user.keyboard('{Shift>}{ArrowRight}{/Shift}');
  expect(mockPatch).toHaveBeenCalledWith('iss-1', 'ends_at', expect.stringMatching(/2026-01-11/));
});
```

---

## §6 — Implementation skeleton

(Sketches above.)

---

## §7 — Dependencies

- **TASK-PROJ-002** — WebSocket sync.
- **TASK-PROJ-003** — LWW scalar (starts_at, ends_at).
- **TASK-PROJ-007** — milestone source.
- **TASK-PROJ-014** — kbd-nav patterns reused.
- **TASK-PROJ-018** — design tokens.

---

## §8 — Example payloads

```json
{
  "kind": "proj.timeline_bar_moved",
  "payload": {
    "issue_id": "iss-...",
    "field": "ends_at",
    "before": "2026-03-10",
    "after": "2026-03-12",
    "was_keyboard": true,
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Dependency arrows between issues (Gantt-style) — TASK-PROJ-016.
- Multi-cycle view (combine adjacent cycles) — slice 4+.
- Workload heatmap (overlapping bars per assignee) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Drag past cycle boundary | clamp at boundary | Visual snap | None |
| Concurrent resize | LWW reject | Rollback + toast | User refreshes |
| Issue without dates | omit from timeline | Visible in Unassigned lane | Operator sets dates |
| 100+ swimlanes | lazy + scroll | Smooth | None |
| Milestone after cycle.ends_at | render at right edge | Visible | None |
| Locale RTL | flex reverses; bars maintain order | Usable | None |
| Touch device | TouchSensor | Drag works | None |
| Browser zoom > 200% | reflow | Usable | None |
| Date in past clamp at cycle.starts_at | render | Bar starts at edge | None |
| WS disconnect | offline banner | Drags queued | Reconnect drains |
| Mid-drag tab close | server state untouched | Local UI snaps back on remount | None |
| Milestone target_date null | skip render | None | None |
| Cycle duration > 1 year | day-zoom unusable | UI defaults to week-zoom | None |
| Issue ends_at < starts_at (data bug) | clamp + log warning | Single-day bar | Operator fixes data |
| Overlap > 10 concurrent bars | badge shows count; "10+" | Operator clicks to expand | None |
| URL window > 2 years | warn at load | renders but slow | Operator narrows |
| Snap-to-week with operator wanting day precision | modal entry available | None | Operator uses modal |
| Per-assignee filter with no issues | empty timeline + helpful copy | None | None |
| Dependency arrow with cycle (A→B→A) | arrow rendered with dashed style | visible cycle warning | Operator fixes |
| Dependency arrow crossing many swimlanes | rendered with curve | None | None |
| Show non-active toggle with 100+ members | virtualised | None | None |
| Weekend bg color clashes with bar color | accessibility check | None | Operator changes theme |
| Swimlane reorder race with WS update | local order preserved | None | None |
| Cycle goal text > 500 chars | truncated with tooltip | None | None |
| Days remaining negative (cycle in past) | banner shows "ended N days ago" | None | None |
| RTL locale | timeline reverses; bars maintain order | None | None |
| Dependency arrow with non-rendered issue (collapsed lane) | arrow goes to lane edge | partial visualisation | None |

---

## §11 — Implementation notes

- Day-pixel constant `DAY_PX` configured per zoom: day=40px, week=80px (per 7 days), month=200px.
- `IntersectionObserver` watches swimlane wrappers; off-screen → suspend rendering inner bars.
- IssueBar uses `position: absolute` within Swimlane (`position: relative`); left/width derived from dates.
- Today indicator is a thin vertical div with `animation: pulse 2s infinite`.
- LWW PATCH endpoint reused from TASK-PROJ-003 §3 (same as Kanban scalar updates).
- Milestone markers render absolutely positioned by date; hover via :hover tooltip.
- Awareness presence (other users editing same issue dates) shown via colored outline on bar — slice 4+; deferred from MVP.
- Overlap stacking uses sub-row layout within the swimlane; max 3 rows visible, then "+N" badge.
- URL window override (`?from=&to=`) takes precedence over cycle bounds; cycle bounds are the default.
- Snap-to-week/month uses Math.round on day delta; precision dates entered via Brief Modal (TASK-PROJ-017).
- Per-assignee filter renders single full-width swimlane; bar bg color changes to indicate filtered view.
- Dependency arrow rendering uses SVG `<line>` elements; computed positions from bar bounding rects.
- Show non-active toggle persists in tenant settings; default off (less clutter).
- Weekend background uses CSS `:nth-child` of day-grid; static color from TASK-PROJ-018 tokens.
- Swimlane reorder uses drag-handle + Shift+Up/Down kbd; persisted per-user in `cyberos_proj_timeline_swimlane_order`.
- Cycle banner is sticky-top during scroll; days-remaining computed live; pulses red when ≤ 2 days.
- Operators dragging bars near cycle edges trigger automatic horizontal scroll; bounded at cycle window unless URL override.
- We considered animating bar transitions on WS update (smooth shift) but kept instant for clarity; flash animation slice 4+.
- Dependency arrows have a max length; cross-swimlane arrows render with arched paths to avoid overlap chaos.
- Per-assignee filter shows the assignee's avatar prominently + their cumulative hours from time entries (TASK-TIME-005).
- The 1-day minimum bar applies even during keyboard resize (Shift+Left past 1 day → no-op + flash).
- Cycle goal banner text is editable inline by tenant admin; emits memory `proj.cycle_goal_updated` audit.
- We don't show milestones in the swimlane area (only on date axis); they're date-based, not assignee-based.
- The today-indicator pulse animation is reduced to subtle if `prefers-reduced-motion` is set.
- Day-zoom > 90 days OR week-zoom > 2 years triggers a UX warning suggesting different zoom.
- IssueBar tooltip on hover shows full issue details (assignee, estimate, blocker count, dependencies).
- Overlap badge color reflects severity (3 overlaps = warning yellow; 5+ = red).
- Drag preview ghost (showing target dates) appears during drag; replaces actual bar on release.
- Cycle banner shows decision_session count + completed-issues count as live stats.

---

*End of TASK-PROJ-015.*

## As built (2026-07-02)

Client code lives under apps/web/src (there is no web/proj-client/).
