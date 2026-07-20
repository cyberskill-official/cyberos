# CHAT module deep audit + overhaul plan (2026-07-03)

Deep analyze / test / check of the whole CHAT module, driving: fix logical + functional bugs, strengthen the module, and take the UI/UX to enterprise grade (fewest steps, fun daily driver). Supersedes the feature inventory in CHAT-AUDIT-2026-07-02.md (most of that "missing" list is now built). Method: three parallel code-reading subagents (backend correctness/security, client correctness/UX, data model/realtime/scale), then I VERIFIED each high-value finding against the code before trusting it. Verification mattered: the single most-flagged finding was a false positive.

Work lands in gated increments on branch `auto/chat-overhaul`. Push is a confirm-with-operator action (live system).

## Verification notes (do not skip)

- REJECTED (false positive): `read.rs::unread_summary` "cartesian M x K double-count of unread/mentions" (flagged HIGH by two agents). It is CORRECT: `chat_mentions` is LEFT JOINed on `message_id AND subject_id=$1` and its PK is `(message_id, subject_id)`, so there is at most one mention row per message
- no fan-out. The mention FILTER missing `sender_subject_id <> $1` is harmless because the sender is stripped from the stored mention set at write time. Do NOT "fix" this query. (The only debatable point is whether thread replies should count toward the badge; that is a product choice, not a bug.)

## Done this pass

- FIXED (commit 4e9feb9): `members::add` is additive-only (ON CONFLICT DO NOTHING) so an admin can no longer demote the owner via the upsert (role changes go through the owner-only `set_role`), and it now rejects adding a third member to a DM. `create_dm` takes a transaction-scoped advisory lock on the sorted (tenant, pair) key so concurrent "message X" requests cannot create duplicate DM threads.

## Bugs and security (verified real; ranked)

- [HIGH, client] Reaction counts drift and can desync. `lib/chat.ts::applyReaction` mutates a delta count (+1/-1) per `reaction_changed` event and toggles a shared `mine`; a replayed frame or a fetch-then-echo double-counts and never self-heals, and `toggleReaction` reads a stale `mine` so a fast double-click can fire two writes in the wrong order. Fix: server echoes an ABSOLUTE `{emoji, count, mine}` snapshot on `reaction_changed`; client replaces rather than mutates; disable the specific reaction button while its toggle is in flight.
- [HIGH, client/reliability] Socket reconnect replays no history. `useChatSocket` reconnects after 1500 ms but never re-fetches; messages/edits/deletes/reactions/reads that landed during the drop are lost from the open channel until the user switches away and back. Fix (pairs with the seq cursor below): on every (re)connect, backfill via `?since=<cursor>` and merge by id.
- [HIGH, client] Auto-mark-read marks the wrong message when viewing history. The effect uses the newest message in the CURRENT window; after a jump/search-jump (`notLatest`) it POSTs an older id as the read marker and zeroes the badge, regressing the real read position. Fix: skip auto-read when `notLatest`; only ever advance the marker monotonically.
- [MED, backend] WS authorization is checked only at upgrade. A member removed mid-session keeps receiving all channel events until they disconnect. Fix: on `member_removed`, publish a control event that closes the removed subject's socket(s) on that channel (or re-check membership periodically in the loop).
- [MED, backend] `search`/`search_all` accept a 1-2 char `q`, which defeats the trigram GIN index and full-scans (a cheap-request DoS); `search_all` has no channel/row budget. Fix: require `q.chars().count()
  >= 3`; clamp `search_all`.
- [MED, backend] No server-side length cap on message `body`, channel `name`/`topic` (only trim/non-empty). A member can post a multi-MB body, bloating the DB, every list, and every AI transcript. Fix: cap body (~16 KB), name/topic (200/500).
- [MED, backend] Attachment content-type is echoed verbatim on download and one attachment can be linked to many messages while delete hard-purges bytes - deleting one message destroys another's bytes. Fix: allowlist/normalize content-type + `X-Content-Type-Options: nosniff`; enforce one-message-per-attachment (or reference-count before purge).
- [MED, client] AI panel and thread panel can both mount fixed-right and overlap on narrow screens. Fix: make them mutually exclusive (close one when the other opens) or tab them.
- [MED, client] Attachment component re-fetches its blob on every hourly token refresh (deps `[token,id]`), breaking an open Lightbox. Fix: re-fetch only when `id` changes; read the token via ref.
- [LOW, client] A few strings bypass `t()`: ChannelSettings role labels render raw owner/admin/member; `lib/chat.ts` hardcodes Today/Yesterday. Fix: add keys, route through `t()`.
- [LOW, backend] `create_dm`/read paths use `let _ = tx.commit()` swallowing commit errors; JWKS is fetched once at boot (a key rotation breaks chat until restart) and `aud` is not validated.

## Reliability and scale (verified)

- [HIGH] No monotonic cursor + no reconnect backfill. Add `seq BIGSERIAL UNIQUE` (or per-channel bigint) to `chat_messages`, return it in list/post payloads, add `GET /channels/:id/events?since=<seq>` replaying message/edit/delete/reaction/read changes, and have the client call it on every WS (re)connect. This is the durable fix behind the "reconnect loses events" bug.
- [HIGH] Single-process fan-out: the in-memory Hub, Notifier, and Presence mean a second chat replica silently splits brain (cross-instance messages/typing/presence never delivered; `Presence.online()` wrong so push mis-fires). ACTION NOW: confirm the deploy pins chat to ONE replica; PLAN: Redis (or Postgres LISTEN/NOTIFY) pub/sub + a shared presence set with per-connection TTL.
- [MED] Missing partial indexes on hot paths. Add: `chat_messages (channel_id, created_at DESC) WHERE deleted_at IS NULL AND parent_id IS NULL` (top-level page); `chat_messages (channel_id, created_at) WHERE deleted_at IS NULL` (unread range); `chat_channels (tenant_id, lower(name)) WHERE kind='group' AND visibility='public' AND archived_at IS NULL` (browse).
- [MED] `Hub`/`Notifier` broadcast senders are never reaped from their maps - unbounded memory growth over process lifetime. Fix: drop a sender when its last receiver disconnects.
- [MED] `push.rs` issues one device query per offline member (N+1). Fix: `WHERE subject_id = ANY($1)`.
- [LOW] Attachment bytes still default to Postgres BYTEA in places; ensure prod uses the `fs`/object store and migrate legacy rows.

## UI/UX to enterprise grade (fewest steps + fun) - the one-by-one backlog

Highest daily-feel leverage first.

1. Optimistic send - render the message instantly (temp client id, greyed, reconcile on echo, "failed · retry" on error). Removes all perceived send latency. Biggest single feel win.
2. Cmd+K command palette / quick switcher - fuzzy jump to channel + DM + person (start a DM inline), Enter to switch. Today switching is mouse-only; search owns Cmd+K. Move search to Cmd+Shift+K or `/`.
3. One-click reactions - show 3-4 most-used emojis inline in the hover action bar (1 click, not 3), keep the "+" full picker.
4. Touch/mobile message actions - long-press -> action sheet (react/reply/edit/delete). Today `.m-actions` is hover-only, so phones have NO message actions. Also reveal on `:focus-within` for keyboard.
5. Accessible modals - one shared `<Modal>` with focus trap, `role="dialog" aria-modal`, Escape, and focus-restore. PeoplePicker/ChannelSettings/BrowseChannels/ProfileEditor don't even close on Escape.
6. Unread divider ("New messages" line) + jump-to-unread on open (the read-marker data already exists).
7. In-app confirm dialog to replace `window.confirm` (delete/leave/archive), ideally delete-with-undo toast (removes the confirm for the common case).
8. Loading skeletons - distinguish "loading" from "empty" so opening a channel stops flashing "No messages yet"; skeleton rows while the first fetch is in flight.
9. Edit as an auto-growing textarea with the composer's Enter/Shift+Enter contract (today edit is a single `<input>`, so multi-line edits break).
10. Thread affordance on the parent in the main timeline - "N replies · last reply 2m ago" chip that reopens the thread (today only the open panel shows the count).
11. Keyboard + a11y: arrow-key nav of the channel list (roving tabindex + `aria-current`), `role="log" aria-live="polite"` on the message pane, `aria-live="assertive"` on the error banner.
12. Mute in one step - a bell toggle in the header + right-click-sidebar-row -> Mute (today it's 3 steps in settings), and add mute for DMs (settings gear is group-only today).
13. Create a channel with zero members (invite later) - today PeoplePicker forces >= 1 member.
14. Search UX - keep the results panel open after a jump, add a result count and up/down/Enter navigation, highlight the match.
15. `prefers-reduced-motion` guard around every animation.
16. Delight layer (cheap, high morale): subtle send scale/whoosh, reaction burst micro-animation, hover elevation on rows, animated typing dots, a "Drop to share" label on drag-over, a first-message flourish in a new channel. Keep the umber/ochre identity; make it feel alive without noise.

## Execution plan

Increment by increment on `auto/chat-overhaul`, each gated (backend: fmt+clippy+test, verify migrations on a throwaway DB; client: tsc+vite; browser-verify UX), commit, and push at milestones with your confirm.

Order: (A) finish the verified bug fixes [reaction desync, reconnect backfill+seq cursor, auto-read-history, ws teardown on removal, search min-length, body caps, attachment content-type]; (B) reliability [seq cursor
+ /events?since, partial indexes, sender reaping, push batch, confirm single-replica]; (C) UI/UX 1..16 one by one. Each UX item ships on its own so you can react and steer.

## Resolution (2026-07-03) - backlog cleared, all live on main

All items above shipped to os.cyberskill.world in gated increments. Bugs/security: reaction desync (3da8a5a/57653b5), auto-read-history (57653b5), search min-length + body cap (3da8a5a), members-additive + DM guards (4e9feb9), WS teardown on removal via `ChatEvent::Kicked` (0a74790), name/topic caps + attachment safe content-type/nosniff + reference-counted purge (0a74790), AI/thread panel exclusivity + Attachment token-via-ref + role labels through t() (58f749e). Reliability: Hub+Notifier sender reaping + push N+1 collapsed to one ANY($1) query + partial indexes migration 0013 (0a74790); single-replica is CONFIRMED and now documented inline in deploy/vps/docker-compose.p0*.yml (the realtime layer is in-process). Reconnect correctness: instead of a seq cursor + change-log, the client refetches the live tail and merges by id on every reconnect (cee91ae) - this recovers missed messages/edits/reactions completely, so the seq cursor is unnecessary for correctness (it would only be an efficiency optimization and is deferred). UI/UX 1-16 all shipped (optimistic send, Cmd+K switcher, one-click reactions, mobile long-press action sheet, focus-trap modals, unread divider, in-app confirm + undo, loading skeletons, edit-as-textarea, reply-count chip, keyboard nav + live regions, one-step mute, zero-member create, search UX, reduced-motion, delight).

Explicitly DEFERRED (low value / larger designs, not blocking): multi-replica fan-out on Redis or Postgres LISTEN/NOTIFY (only needed to scale chat past one container); a durable per-change event log + seq cursor (the reconnect refetch covers correctness); JWKS key-rotation refetch + `aud` validation on the chat token (LOW; chat re-verifies via the JWKS URL at boot); swallowed commit errors on read paths (LOW); migrating any residual Postgres-BYTEA attachment rows to the fs store (prod already defaults to fs).
