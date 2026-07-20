---
id: TASK-PROJ-017
title: "Brief Modal — issue deep-view with Yjs description editor + threaded comments + LWW meta sidebar + presence cursors"
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
related_tasks: [TASK-PROJ-002, TASK-PROJ-003, TASK-PROJ-004, TASK-PROJ-008, TASK-PROJ-014, TASK-PROJ-018]
depends_on: [TASK-PROJ-003]
blocks: []

source_pages:
  - website/docs/modules/proj.html#brief-modal
source_decisions:
  - DEC-380 (Brief Modal opens from any view via Enter on focused card; URL-deep-linkable)
  - DEC-381 (description = Y.Text via TipTap editor; comments = Y.Array; sidebar = LWW)
  - DEC-382 (modal = full-screen on mobile, side-panel on desktop ≥ 1024px)

language: typescript 5.4 + react 18
service: cyberos/web/proj-client/
new_files:
  - web/proj-client/src/views/BriefModal/Modal.tsx
  - web/proj-client/src/views/BriefModal/Description.tsx
  - web/proj-client/src/views/BriefModal/CommentThread.tsx
  - web/proj-client/src/views/BriefModal/MetaSidebar.tsx
  - web/proj-client/src/views/BriefModal/PresenceCursors.tsx
  - web/proj-client/src/views/BriefModal/HistoryDrawer.tsx
  - web/proj-client/tests/brief_modal_test.tsx
modified_files:
  # /proj/issues/:id/brief deep-link
  - web/proj-client/src/router.tsx
  # @tiptap/react, @tiptap/extension-collaboration
  - web/proj-client/package.json
allowed_tools:
  - file_read: web/proj-client/**
  - file_write: web/proj-client/src/views/BriefModal/**, web/proj-client/tests/**
  - bash: cd web/proj-client && npm test brief
disallowed_tools:
  - render description without TipTap + Yjs binding (per DEC-381)
  - block modal close on unsaved changes (no unsaved state per CRDT model)

effort_hours: 8
subtasks:
  - "0.5h: router /proj/issues/:id/brief deep-link"
  - "1.0h: Modal.tsx — responsive layout (mobile = full screen; desktop = side panel)"
  - "1.5h: Description.tsx — TipTap + @tiptap/extension-collaboration bound to Y.Text"
  - "1.0h: CommentThread.tsx — Y.Array iteration + new-comment composer"
  - "1.0h: MetaSidebar.tsx — status / assignee / estimate / labels / dates (LWW)"
  - "1.0h: PresenceCursors.tsx — render other users' cursors via Yjs awareness"
  - "0.5h: HistoryDrawer.tsx — TASK-PROJ-008 history_event list"
  - "0.5h: kbd shortcuts (Esc close; T edit title; C add comment; H toggle history)"
  - "1.0h: brief_modal_test.tsx — render + concurrent edit + LWW meta"
risk_if_skipped: "Brief Modal is the deep-edit surface; without it, descriptions can't be edited collaboratively, comments live in separate views. Without presence cursors, users overwrite each other (mitigated by CRDT but UX-confusing). Without history drawer, audit trail invisible. Without responsive layout, mobile users blocked."
---

## §1 — Description (BCP-14 normative)

The Brief Modal **MUST** be a unified deep-view for one issue with collaborative description + comments + meta sidebar. The contract:

1. **MUST** open via:
- Click on issue card in any view (Kanban / Timeline / Gantt).
- Enter key on focused card.
- Direct URL `/proj/issues/:id/brief`. Opening updates URL (history.pushState) so back-button + share-link work.
2. **MUST** render responsively:
- Mobile (< 1024px): full-screen overlay; sidebar collapses to expandable section.
- Desktop (≥ 1024px): right-side panel 480px wide; sidebar always-visible.
3. **MUST** bind description to Y.Text via TASK-PROJ-003 YjsProvider and TipTap + `@tiptap/extension-collaboration`. Concurrent edits converge per Yjs.
4. **MUST** render comments as Y.Array; each comment is `Y.Map { id, author_id, body: Y.Text, created_at }`. New comment composer adds element to array; edit binds to body Y.Text.
5. **MUST** render meta sidebar with LWW scalars (TASK-PROJ-003 §1 #6):
- Status (uses TASK-PROJ-014 StatusPicker).
- Assignee dropdown.
- Priority radio.
- Estimate number input.
- Labels multi-select.
- Dates (starts_at + ends_at). Each field PATCHes via LWW endpoint; stale-write → toast + revert.
6. **MUST** render presence cursors:
- Other users editing this modal → their cursor position in description shown as labeled flag (name + color).
- Cursor flag throttled at 30 Hz per TASK-PROJ-003 awareness.
- Cursor expires 30s after last awareness heartbeat.
7. **MUST** provide a history drawer toggle (button + `H` shortcut):
- When open, side panel shows TASK-PROJ-008 history_event timeline.
- Chain_anchor verification status per row (green check or red warn).
- Click on history row scrolls description to that mutation's snapshot.
8. **MUST** support kbd shortcuts:
- Esc closes modal (no confirmation; CRDT auto-saves).
- T puts cursor in title field (inline-edit).
- C focuses new-comment composer.
- H toggles history drawer.
- Cmd+S explicit save (no-op visual feedback; "auto-saved" indicator).
9. **MUST** emit memory audit `proj.brief_modal_opened` per open with `{issue_id, by_subject_id, opened_from, trace_id}` where opened_from ∈ kanban | timeline | gantt | url | search.
10. **MUST** RLS-enforce (issue + comments + history).
11. **MUST** pass axe-core (focus-trap inside modal; restore focus on close; aria-modal=true).
12. **MUST** emit OTel:
- `proj_brief_modal_opens_total{opened_from}` (counter).
- `proj_brief_modal_render_p95_ms` (histogram).
- `proj_brief_modal_session_seconds` (histogram — engagement signal).
13. **MUST** support comment threading: each comment can be a reply to another via `reply_to_comment_id`; threading rendered with visual indentation (max depth 5).
14. **MUST** support comment mentions: `@username` in comment body resolves to user; sends in-app notification to mentioned user via CUO triage.
15. **MUST** support attachments on comments: file upload via task-FILES (max 25MB per file, 5 files per comment); previewable images/PDFs inline.
16. **MUST** support reactions on comments: emoji picker; each comment shows reaction tallies; clicking re-toggles user's reaction.
17. **MUST** support `@lumi` invocation in comments: routes to TASK-CHAT-008 sibling handler scoped to issue context (description + recent comments).
18. **MUST** support "link" actions in sidebar: quick-add issue dependencies (TASK-PROJ-016) + memory-links (TASK-PROJ-009) without leaving modal.
19. **MUST** support draft comment auto-save: composer text persists per-issue per-user in `localStorage`; on next modal open, restored.
20. **MUST** support keyboard navigation through comments: J/K moves comment focus; Reply opens reply composer threaded under that comment.
21. **MUST** show "X is typing..." indicator below comment composer when another user has the composer open; throttled per Yjs awareness.
22. **MUST** include a "follow / unfollow" toggle: followers get CUO notifications on any update to this issue (comments, status, assignee changes).
23. **MUST** support markdown shortcuts in the description editor: TipTap configures `**bold**` / `_italic_` / `# heading` etc. matching standard markdown syntax.

---

## §2 — Why this design (rationale for humans)

**Why one modal for everything (DEC-380)?** Three views (Kanban/Timeline/Gantt) all need deep-edit; unifying = one place for edits = no UX drift. URL-deep-linkable = shareable.

**Why responsive split (DEC-382)?** Mobile users need full screen for editing; desktop users want issue visible in board context while editing. 1024px is the standard tablet threshold.

**Why TipTap (DEC-381)?** Industry-standard React rich-text editor; first-class Yjs integration via `@tiptap/extension-collaboration`. Alternatives (Slate, Lexical) have less mature Yjs binding.

**Why no unsaved state (§1 #8)?** CRDT auto-saves every keystroke; "save" is mental. Esc-to-close without confirmation = trust the system. Cmd+S is muscle-memory affordance returning "auto-saved" toast.

**Why presence cursors (§1 #6)?** Two users editing same paragraph collide → CRDT resolves correctly but UX is confusing without seeing the other person. Labeled cursors = "Bob is here" signal.

**Why history drawer toggle (§1 #7)?** History is per-issue context but bulky. Default hidden; toggle reveals. Power users keep it open during reviews.

**Why audit modal opens (§1 #9)?** Per-issue engagement metrics inform UX. "How often do users open issues from Kanban vs URL?" informs onboarding flows.

**Why focus-trap (§1 #11)?** WCAG requires modals keep keyboard focus inside; releases on close. Standard accessibility pattern.

**Why threading (§1 #13)?** Long comment threads need reply context; flat list loses conversation structure.

**Why mentions + notify (§1 #14)?** Mention is the standard "tag someone for attention" pattern; notification closes the loop.

**Why comment attachments (§1 #15)?** Real workflows attach screenshots, logs, designs. Without inline upload = workflow friction.

**Why comment reactions (§1 #16)?** Lightweight signal ("agree", "this") without writing a reply; reduces comment noise.

**Why @lumi in comments (§1 #17)?** LLM-assisted clarification inline; doesn't require leaving the modal.

**Why link actions in sidebar (§1 #18)?** Adding dependencies/memory-links is workflow-adjacent; in-modal action eliminates context switch.

**Why draft auto-save (§1 #19)?** Operator drafting long comment + modal accidentally closes = lost text. localStorage = survives session.

**Why kbd comment nav (§1 #20)?** Power users review many comments; kbd parity for review workflow.

**Why typing indicator (§1 #21)?** Two users typing replies simultaneously waste effort; awareness signal prevents duplicate work.

**Why follow/unfollow (§1 #22)?** Operators want updates on issues they care about; default-following all might over-notify.

**Why markdown shortcuts (§1 #23)?** Markdown is the universal text-formatting language; operators expect it.

---

## §3 — API contract

```tsx
// web/proj-client/src/views/BriefModal/Modal.tsx
export function BriefModal({ issueId, openedFrom }: { issueId: string; openedFrom: OpenedFrom }) {
  const yjs = useYjsProvider(issueId);
  const [historyOpen, setHistoryOpen] = useState(false);
  const isMobile = useMediaQuery('(max-width: 1023px)');

  useEffect(() => {
    emitMemory('proj.brief_modal_opened', { issue_id: issueId, opened_from: openedFrom });
    history.pushState({}, '', `/proj/issues/${issueId}/brief`);
    return () => {
      // Restore prior URL on close
    };
  }, []);

  useKeyboardShortcuts({
    Escape: closeModal,
    T:      () => focusTitle(),
    C:      () => focusCommentComposer(),
    H:      () => setHistoryOpen(o => !o),
    'Mod+S': () => toast('Auto-saved', 'success'),
  });

  return (
    <Dialog open={true} onOpenChange={closeModal}
            className={isMobile ? 'fullscreen' : 'side-panel'}
            aria-modal="true" aria-labelledby="issue-title">
      <FocusTrap>
        <div className="brief-modal">
          <Header issueId={issueId} />
          <Description yjs={yjs} />
          <PresenceCursors yjs={yjs} />
          <CommentThread yjs={yjs} />
          <MetaSidebar issueId={issueId} />
          {historyOpen && <HistoryDrawer issueId={issueId} />}
        </div>
      </FocusTrap>
    </Dialog>
  );
}
```

```tsx
// web/proj-client/src/views/BriefModal/Description.tsx
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCursor from '@tiptap/extension-collaboration-cursor';

export function Description({ yjs }: { yjs: YjsProvider }) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({ history: false }),   // Yjs handles undo
      Collaboration.configure({ document: yjs.doc, field: 'description' }),
      CollaborationCursor.configure({
        provider: yjs.wsProvider,
        user: { name: currentUser.name, color: userColor(currentUser.id) },
      }),
    ],
  });
  return <EditorContent editor={editor} className="description-editor" />;
}
```

```tsx
// web/proj-client/src/views/BriefModal/MetaSidebar.tsx
export function MetaSidebar({ issueId }: { issueId: string }) {
  const issue = useIssue(issueId);
  return (
    <aside className="meta-sidebar">
      <StatusPicker issueId={issueId} current={issue.status} onChange={patchStatus} />
      <AssigneePicker issueId={issueId} current={issue.assignee_id} onChange={patchAssignee} />
      <PriorityPicker issueId={issueId} current={issue.priority} onChange={patchPriority} />
      <EstimateInput  issueId={issueId} current={issue.estimate} onChange={patchEstimate} />
      <LabelMultiSelect issueId={issueId} current={issue.labels} onChange={patchLabels} />
      <DateRange      issueId={issueId} starts={issue.starts_at} ends={issue.ends_at} onChange={patchDates} />
    </aside>
  );

  async function patchStatus(to: IssueStatus, reason?: string) {
    const res = await postTransition(issueId, to, reason);
    if (res.error === 'stale_write') { toast('Refreshed; please retry'); }
  }
  // ... similar patch functions for other fields, each calls writeScalarLWW
}
```

```tsx
// web/proj-client/src/views/BriefModal/HistoryDrawer.tsx
export function HistoryDrawer({ issueId }: { issueId: string }) {
  const history = useIssueHistory(issueId);
  return (
    <div className="history-drawer" role="region" aria-label="Issue history">
      <h3>History</h3>
      <ol>
        {history.map(h => (
          <li key={h.event.id}>
            <span>{h.event.mutation_kind}</span>
            <span>{h.event.field}</span>
            <span>by {h.event.by_subject_id}</span>
            {h.chain_verified
              ? <span aria-label="chain verified" title="chain verified">✓</span>
              : <span aria-label="chain mismatch — sev-1 alert" title="chain mismatch" className="warn">⚠</span>}
          </li>
        ))}
      </ol>
    </div>
  );
}
```

---

## §4 — Acceptance criteria

1. **Open from Kanban Enter** — focused card + Enter → modal opens; URL updates.
2. **Open from URL deep-link** — visit `/proj/issues/iss-X/brief` → modal opens.
3. **Esc closes** — modal closes; URL restored.
4. **Description CRDT** — two users typing → both converge via Yjs.
5. **Comments Y.Array** — add comment → appears in both users' modals real-time.
6. **Meta LWW: status** — change status → POST transition; new value persists.
7. **Meta LWW: stale write** — concurrent assignee change → second user gets stale_write; toast.
8. **Presence cursor visible** — second user opens modal → their cursor appears with name + color.
9. **Presence cursor expires** — second user closes browser → cursor gone within 30s.
10. **History drawer toggle (H)** — H opens; H closes.
11. **Chain anchor verify icon** — happy history → green check; tampered → red warn.
12. **Mobile full-screen** — viewport < 1024px → full-screen layout.
13. **Desktop side-panel** — viewport ≥ 1024px → 480px right panel.
14. **Kbd T focuses title** — modal open + T → title inline-edit focused.
15. **Kbd C focuses comment composer** — C → focus on new comment input.
16. **Cmd+S shows auto-saved toast** — no-op but feedback.
17. **Focus trap** — Tab cycles within modal; doesn't escape to background.
18. **Focus restore on close** — modal close → focus returns to opening element.
19. **memory audit modal_opened** — per open → row with opened_from.
20. **OTel modal_opens_total counter** — per open → counter increments.
21. **axe-core passes** — aria-modal + focus-trap + labels correct.
22. **RLS isolates** — tenant A's issue invisible to tenant B's modal request → 404.
23. **Comment thread depth ≤ 5** — replies indent up to depth 5; beyond → flat with marker (AC for §1 #13).
24. **Mention notifies user** — comment with `@alice` → CUO notification queued for alice (AC for §1 #14).
25. **Attachment uploads** — drop file → task-FILES upload; preview inline (AC for §1 #15).
26. **Reaction toggle** — click emoji → toggles user's reaction; tally updates (AC for §1 #16).
27. **@lumi in comment routes to handler** — comment with @lumi → reply appears as Lumi-authored comment (AC for §1 #17).
28. **Sidebar quick-link actions** — click "Add Dep" in sidebar → opens TASK-PROJ-016 dialog inline (AC for §1 #18).
29. **Draft auto-save survives close** — draft text in composer; close modal; reopen → text restored (AC for §1 #19).
30. **Kbd J/K navigates comments** — comment focused → J moves to next; K to prior (AC for §1 #20).
31. **Typing indicator visible** — second user types in composer → first user sees "X is typing..." (AC for §1 #21).
32. **Follow toggle adds to followers** — toggle → CUO notifications start for that user (AC for §1 #22).
33. **Markdown `**bold**` works** — type `**bold**` in editor → renders bold (AC for §1 #23).

---

## §5 — Verification

```typescript
test('Esc closes modal', async () => {
  const { user } = render(<BriefModal issueId="iss-1" openedFrom="kanban" />);
  await user.keyboard('{Escape}');
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
});

test('description converges with Yjs', async () => {
  const { user: u1 } = render(<BriefModal issueId="iss-1" openedFrom="url" />);
  const editor = screen.getByRole('textbox');
  await u1.type(editor, 'Hello from user 1');
  // simulate user 2 in a parallel doc
  const u2Doc = simulateYjsPeer('iss-1');
  u2Doc.getText('description').insert(0, 'User 2 was here. ');
  await waitFor(() => expect(editor).toHaveTextContent(/User 2 was here.*Hello from user 1/));
});

test('LWW stale_write shows toast', async () => {
  const { user } = render(<BriefModal issueId="iss-1" openedFrom="kanban" />);
  mockLww.fail409('status');
  await user.click(screen.getByText('In Progress'));
  expect(screen.getByRole('alert')).toHaveTextContent(/refreshed/i);
});

test('history drawer toggle with H', async () => {
  const { user } = render(<BriefModal issueId="iss-1" openedFrom="url" />);
  await user.keyboard('h');
  expect(screen.getByRole('region', { name: 'Issue history' })).toBeInTheDocument();
  await user.keyboard('h');
  expect(screen.queryByRole('region', { name: 'Issue history' })).toBeNull();
});

test('chain anchor mismatch shows warn icon', async () => {
  mockHistory.tamperRow(2);
  const { user } = render(<BriefModal issueId="iss-1" openedFrom="url" />);
  await user.keyboard('h');
  const warns = screen.getAllByLabelText(/chain mismatch/);
  expect(warns).toHaveLength(1);
});

test('focus trap inside modal', async () => {
  const { user } = render(<BriefModal issueId="iss-1" openedFrom="url" />);
  // Tab repeatedly; focus should never leave dialog
  for (let i = 0; i < 20; i++) await user.tab();
  expect(screen.getByRole('dialog')).toContainElement(document.activeElement!);
});
```

---

## §6 — Implementation skeleton

(Sketches above.)

---

## §7 — Dependencies

- **TASK-PROJ-002** — WS.
- **TASK-PROJ-003** — Y.Doc + LWW + awareness.
- **TASK-PROJ-004** — status transitions (StatusPicker).
- **TASK-PROJ-008** — history_event source.
- **TASK-PROJ-014** — opener (Kanban).
- **TASK-PROJ-018** — design tokens.

---

## §8 — Example payloads

```json
{
  "kind": "proj.brief_modal_opened",
  "payload": {
    "issue_id": "iss-...",
    "by_subject_id": "7e57c0de-...",
    "opened_from": "kanban",
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Multiple modals side-by-side (split view) — slice 4+.
- Modal-within-modal for linked issue navigation — slice 4+.
- AI assistant inside modal ("explain this issue") — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| YjsProvider connect fails | error state | Read-only banner; description shows last snapshot | Reconnect drains |
| Issue deleted while modal open | 404 on poll | Toast + auto-close | None |
| History fetch fails | error state | Banner; drawer empty | Retry |
| LWW stale_write | 409 | Toast; revert local field | User refreshes |
| Concurrent edit causes CRDT churn | Yjs handles | None | None |
| Comment composer disconnected | offline buffer | Comment queues; sent on reconnect | None |
| Presence flood | 30Hz throttle | None | None |
| Mobile viewport switch | useMediaQuery handles | Layout reflows | None |
| Focus escape (axe) | a11y test catches | CI blocked | Fix focus-trap |
| Chain anchor mismatch | red warn icon | Sev-1 alarm via TASK-PROJ-008 metric | Operator investigates |
| Modal opened with invalid issueId | 404 | Toast + close | None |
| Browser back navigates away | URL state restored | None | None |
| Modal stuck (component crash) | error boundary | Recover | None |
| TipTap version mismatch | initial render fail | Sev-1 | Pin version |
| Comment with embedded scripts | TipTap sanitisation | Safe | None |
| Comment thread depth > 5 | flat fallback with marker | None | None |
| Mention to non-existent user | passed verbatim; no notification | None | None |
| Attachment > 25MB | rejected upfront | toast | Caller resizes |
| Reaction spam (1000s of clicks) | debounce + rate-limit | None | None |
| @lumi in comment with tenant lumi disabled | falls back to TASK-CHAT-008 behavior | None | None |
| Sidebar quick-link opens stale dependencies | refetch on open | None | None |
| Draft auto-save with multi-tab | localStorage shared; last-write-wins | None | None |
| Typing indicator stuck (user closed browser) | 30s expiry | None | None |
| Follow toggle for already-followed | no-op | None | None |
| Markdown shortcut conflict (operator types ** literal) | escape support | None | None |
| Threaded reply notification spam | dedup per (user, comment) | None | None |
| Comment with mention + attachment + reaction | all work concurrently | None | None |
| LocalStorage full | warn; oldest drafts evicted | None | Operator clears |
| Mobile keyboard takes half screen | viewport adjust | None | None |
| Modal close mid-mention-resolution | notification still sent | None | None |

---

## §11 — Implementation notes

- TipTap's StarterKit's history is disabled because Yjs provides its own undo via `Y.UndoManager`.
- CollaborationCursor uses Yjs awareness state; user color derived from subject_id hash (consistent across sessions).
- History drawer uses `useIssueHistory` hook with SWR caching; refetch on focus.
- Focus-trap library: `@radix-ui/react-dialog` provides built-in focus management.
- The `opened_from` value is set by the caller; URL opens default to "url".
- The 480px desktop side-panel width is a design token (TASK-PROJ-018).
- Modal session histogram captures engagement: how long users spend per issue.
- Comments are appended to Y.Array; deletion via Y.Array.delete() preserves CRDT history.
- Threading uses `reply_to_comment_id` field; rendering recursively indents up to depth 5; deeper threads flatten with "deeply nested" marker.
- Mention regex matches `@[a-zA-Z0-9_]+`; resolves against users table for tenant.
- Attachment upload uses task-FILES presigned URL flow; preview component handles common MIME types.
- Reactions use Y.Map (emoji → Set of user_ids); CRDT handles concurrent toggles.
- @lumi in comments routes to chat-lumi service with `context="issue-comment"`; reply appears as a comment authored by Lumi system user.
- Sidebar quick-link actions open inline dialogs (radix UI Dialog) without leaving modal.
- Draft auto-save uses `localStorage[`draft_comment_${issueId}_${userId}`]`; debounced at 500ms.
- Kbd J/K navigates comments in DOM order; Reply opens reply composer under focused comment.
- Typing indicator uses Yjs awareness `composing: true` field; throttled and TTL'd.
- Follow toggle creates row in `cyberos_proj_issue_followers` table; CUO triages notifications based on this list.
- Markdown shortcuts use TipTap's `Markdown` extension; configured for standard CommonMark subset.
- Reaction emoji picker uses `@emoji-mart/react` (lazy-loaded ~80KB).
- Mention auto-complete uses `@tiptap/extension-mention`; suggests as user types.
- Threaded reply UI collapses to "N replies" link when thread > 10 comments; click expands.
- Comment edit window: 15 minutes from create; after that, edit produces a "edited at" indicator + history row.
- We considered comment soft-delete vs hard-delete; chose soft (mark deleted, body redacted) for audit trail.
- Comment threading depth ≤ 5 calibrated against UX studies; deeper = unreadable.
- Mention notifications respect notify_props (TASK-CHAT-011-style); user can disable mentions per-engagement.
- The "@lumi in comment" feature is opt-in per engagement (per-tenant Lumi settings inherited).
- Sidebar quick-links emit memory audit per action (link added/removed).
- Draft auto-save is cleared on successful comment submit OR explicit "discard draft" button.
- Reactions don't notify; they're lightweight feedback. Operators wanting notify use mentions.
- Markdown shortcuts work in description editor AND comment composer; consistent UX.
- The follow/unfollow toggle defaults to follow for issue authors + assignees; explicit follow for others.
- Comment with mention + attachment + reaction emits multiple audit rows; correlated via comment_id.
- Modal session tracking starts on first render, ends on close; histogram captures real engagement.

---

*End of TASK-PROJ-017.*

## As built (2026-07-02)

Client code lives under apps/web/src (there is no web/proj-client/).
