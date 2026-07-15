---
task_id: TASK-CHAT-268
audited: 2026-07-11
verdict: PASS (after revision)
score_pre_revision: 5/10
score_post_expansion: 8/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

## §1 - Verdict summary

TASK-CHAT-268 specifies user blocking: 15 normative §1 clauses, one migration (`chat_blocks`, directional,
RLS-scoped), three endpoints, four named server-side enforcement points, 16 acceptance criteria, 8 Rust
integration tests, and a 14-row failure-mode inventory. Eight findings were raised across three rounds; all
eight are resolved. Two of them - ISS-001 (notification fan-out) and ISS-002 (403 to the blocked sender) -
were not cosmetic: the first would have shipped a "block" that still lit up the blocker's lock screen with
the blocked person's name, and the second would have shipped a block that actively endangers the person it
is meant to protect.

## §2 - Findings (all resolved)

### ISS-001 - The block would have been defeated by the push notification
The first draft enforced blocking in the message-list query and nowhere else, which reads as complete and is
not. `notify.rs::fan_out` selects channel members and pushes to their registered devices; it has no notion of
blocks. So the blocked person posts, the message never appears in the blocker's list - and a push notification
carrying the blocked person's display name and the first line of their message lands on the blocker's lock
screen anyway. The most visible surface in the product would have been the one that leaked. Resolved: §1 #4
enumerates **four** enforcement points as normative (list, realtime, notification/push, DM list), §3 shows
`blockers_of` applied inside `notify.rs` before recipient retention, and AC #9 asserts zero notifications and
zero pushes *including for an `@mention`*. §10 row 2.

### ISS-002 - Returning 403 to the blocked sender endangers the blocker
The draft refused the blocked person's message with `403 Forbidden`. Honest, simple, and the wrong call: it
tells a harasser they have been blocked, which is a documented escalation trigger - the person who was being
ignored becomes a person who knows they were rejected, and moves to another channel. Resolved: §1 #7 and #8
mandate the silent-drop model (the message is persisted, the sender sees it in their own client, it is simply
never delivered), and §1 #8 extends the prohibition to *every* observable: status code, error string, read
receipt, delivery indicator. §2 explains why at length. AC #7 asserts identical status and body shape before
and after the block, and asserts nothing in B's entire visible world names the block. §10 rows 3 and 4.

### ISS-003 - A timing side-channel would have reinstated the disclosure
Having removed the 403, an obvious implementation applies the block at *write* time - look up the recipient's
blocks, and skip the write. That is measurably faster than the unblocked path, so the block becomes inferable
from latency, which re-opens ISS-002 through the back door. Resolved: §1 #7 requires the message to be
persisted normally, and the block is applied at *read* fan-out. §10 row 4 names the failure and the fix.

### ISS-004 - Deleting a blocked person's messages would have corrupted the conversation
The draft removed blocked senders' messages from every channel. In a group channel that silently rewrites
history for one participant: replies to a now-absent message become nonsense, thread counts stop matching, and
the blocker ends up more confused than protected. Resolved: §1 #5 collapses rather than removes in group
channels - the row keeps its id and position, the content is withheld, and a `blocked_sender` flag drives a
client placeholder with an explicit "show anyway". §1 #6 keeps *removal* for DMs, where there is no
surrounding conversation to contextualise and a column of placeholders is just a drip-feed of the harassment.
§2 argues both halves. AC #4 and AC #5.

### ISS-005 - An already-open WebSocket would have kept delivering after the block landed
Each socket caches its owner's blocked-set at connect. Nothing in the draft invalidated it, so a block placed
in one tab left every other open tab receiving the blocked person's frames until it happened to reconnect -
a live, observable failure of §1 #4 that no unit test on the HTTP layer would catch. Resolved: §6 publishes
`invalidate_blocks(blocker)` to the *blocker's own control topic* on every mutation; §11 explains why the
blocker's topic and not the channel's (a block is nobody else's business). AC #8 opens the socket *before*
the block to force the path. §10 row 5.

### ISS-006 - Reaction counts would have leaked the blocked person's presence
Blocking the sender of a message says nothing about a blocked person *reacting* to someone else's message. The
draft filtered by message sender only, so a blocker would see "2 reactions" on a message where one of the two
was from the person they blocked - which both leaks their activity and is quietly maddening. Resolved: §1 #9
suppresses reactions and mentions authored by a blocked person; §3's message-list enforcement retains the
folded reaction set by *reactor* id, not sender id. AC #10.

### ISS-007 - The moderation queue would have been blinded by the reviewer's own block
The likeliest person to report someone is the same person who blocked them, and in a small workspace that is
often the administrator. If `blocked_by` were applied uniformly, the admin would open TASK-CHAT-269's queue and
find the reported content redacted - by their own block. Resolved: §1 #12 carves the moderation queue out
explicitly, §7 makes it a normative constraint *on TASK-CHAT-269* rather than a note, and AC #12 asserts the
queue renders the full snapshot for an admin who has blocked the reported person. §10 row 10.

### ISS-008 - N+1 on the hottest endpoint in the service
The first cut queried `chat_blocks` per message while mapping the list. The message list is the single
most-called endpoint in chat, and this would have added one round trip per row. Resolved: §3 reads the
blocked-set **once** per request into a `HashSet<Uuid>` and threads it through all four enforcement points;
`blockers_of` takes the candidate recipient list so the notification fan-out asks one indexed question rather
than scanning. §11 documents both. §10 row 6.

## §3 - Resolution

Eight findings, all resolved. The spec's central insight - that a block is not one filter but four, and that
the blocked person must observe *nothing* - is now pinned by acceptance criteria rather than by prose: AC #9
fails if a future contributor adds a notification path that forgets blocks, AC #7 fails if anyone
"helpfully" restores a 403, and AC #8 fails if socket invalidation is dropped.

**Score = 10/10.**

---

*End of TASK-CHAT-268 audit.*
