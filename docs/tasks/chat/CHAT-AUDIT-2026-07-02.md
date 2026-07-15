# CyberOS chat module audit (2026-07-02)

Read-only audit of the CyberOS-native chat module: the Rust service at `services/chat` and the React client at `apps/web`. Every claim below is grounded in a file path. The goal is to drive two follow-on plans: make chat full-featured, then overhaul the UI/UX (the owner's words: "it looks very outdated").

The module is a real, first-party team chat. The data model, realtime hub, per-tenant RLS, and audit-chain wiring are solid and honest about their limits (comments in the code repeatedly say where a slice stops). The gaps are not bugs; they are unbuilt features and an unfinished visual language. What exists works; what is missing is either standard team-chat surface area (mentions, notifications, formatting, richer channels) or infrastructure that was deliberately deferred (object storage, TURN, real push).

## Executive summary

- The backend covers the messaging core well: channels and DMs (`channels.rs`), messages with threads, edit, and soft-delete (`messages.rs`), reactions (`reactions.rs`), attachments in Postgres (`attachments.rs`), read receipts and unread counts (`read.rs`), presence and typing over one websocket per channel (`realtime.rs`), accent-insensitive Vietnamese search (`messages.rs::search`), and inline translation via the ai-gateway (`translate.rs`). Everything is tenant-scoped through RLS (`db.rs`, migrations `0001`/`0003`/`0004`/`0008`).
- The client is a single 1240-line `Chat.tsx` plus seven components. It renders all of the above and adds 1:1 WebRTC calling (`lib/call.ts`, `components/CallOverlay.tsx`), a people picker for DMs/groups/calls (`components/PeoplePicker.tsx`), and a profile editor (`components/ProfileEditor.tsx`).
- The biggest missing features, benchmarked against Slack/Discord/Mattermost, are: no @-mentions and therefore no real notifications, no per-user notification socket (presence and ringing only work while a channel is open), no message formatting or markdown or code blocks, a fixed six-emoji reaction set, attachments capped at 5 MB in the database with no object storage and one file at a time, thin channel management (no topic/description, no archive, no public/private, no channel browser, no group-DM), and no admin or moderation surface.
- AI is a live opening, not a built feature. `translate.rs` already calls the ai-gateway's `/v1/chat` completion route server-side; the same path could power summarize, smart replies, and action items with no new external dependency. Today only translation exists and it degrades to a clean 502 when the gateway is unset.
- The UI is a warm umber/ochre dark theme in a single hand-written `styles.css` (326 lines, no framework). It is coherent but under-developed: near-monotone browns, low text contrast on the muted tiers, tiny 28px controls, sparse vertical rhythm, and thin empty states. It reads like an internal-tool scaffold. It does not need a framework or a rewrite; it needs a tightened token set (contrast, spacing, radii), denser and more legible message rows, a real composer, and stronger empty/hover states.
- There is no i18n. `apps/web/package.json` pulls only `react` and `react-dom`; every string in `Chat.tsx` and the components is hardcoded English, on a team the memory notes describe as bilingual (Vietnamese/English).

---

## HALF A - Feature inventory (present vs missing)

Benchmark: Slack / Discord / Mattermost as the reference for a modern team chat. For each area: what exists (with the file, endpoint, or ws event), then what is missing or thin.

### 1. Messaging core

Present:
- Send, list, edit, soft-delete. `POST/GET /v1/chat/channels/:id/messages`, `PATCH/DELETE /v1/chat/channels/:id/messages/:msg` (`services/chat/src/lib.rs:59-65`, `services/chat/src/messages.rs`). Edit is sender-only (`messages.rs:319-323`); delete is sender-or-manager (`messages.rs:412-417`); delete is a soft-delete that also blanks the body (`messages.rs:418`).
- Threads. `parent_id` on a message (`migrations/0002_chat_slice2.sql`), posting a reply validates the parent is live in the channel (`messages.rs:138-153`), and list takes `?parent_id=` to fetch a thread (`messages.rs:240-252`). Client thread panel in `apps/web/src/components/ThreadPanel.tsx`, opened from `Chat.tsx:672-682`.
- Live fan-out. Every post/edit/delete publishes a `ChatEvent` on the channel hub (`messages.rs:179-182`, `messages.rs:337-344`, `messages.rs:425-428`; enum in `realtime.rs:24-57`), and the client folds them in (`Chat.tsx:330-359`).
- Optimistic-ish send with dedupe by id (`Chat.tsx:477`, `Chat.tsx:690`).
- Day separators and 5-minute sender grouping (`Chat.tsx:712-726`, `GROUP_WINDOW_MS` at `Chat.tsx:31`).

Missing or thin:
- No message formatting at all. The body is stored and rendered as plain text: `.m-body { white-space: pre-wrap }` (`styles.css:178`) and `{m.body}` rendered raw (`Chat.tsx:1032`). No markdown, no bold/italic, no code blocks, no inline code, no blockquotes, no lists. There is no markdown or sanitizer library (`package.json` has only react). This is the most visible gap against Slack/Discord.
- No link handling. URLs are not linkified and not made clickable; a pasted link is inert text. No link unfurl/preview.
- No drafts. The composer draft lives in component state (`Chat.tsx:89`) and is dropped on channel switch; nothing is persisted per channel.
- No message pinning, no bookmarks/saved items, no "copy link to message," no forward/quote.
- No pagination in the UI. The client fetches the latest page (`Chat.tsx:282`) and never requests older messages; the backend supports `?before=` and `?limit=` (`messages.rs:78-82`, `messages.rs:233-234`) but there is no infinite-scroll-up wired in the client.
- Edit and delete are blocked on messages that carry an attachment: the client hides the edit button when `m.attachment_id` is set (`Chat.tsx:1095`), and a delete only removes the message row, never the attachment bytes (`messages.rs:418` blanks the body but the `chat_attachments` row and its bytes remain).

### 2. Reactions

Present:
- Add/remove a reaction, member-only, idempotent. `POST /v1/chat/channels/:id/messages/:msg/reactions` and `DELETE .../reactions/:emoji` (`lib.rs:76-82`, `reactions.rs`). Unique per `(message, subject, emoji)` (`migrations/0008_chat_reactions.sql`).
- Reactions are folded into the message list in one extra query (`messages.rs:270-286`, `reactions.rs::summarize`), so a row renders an emoji-and-count strip without a second round-trip.
- Live `ReactionChanged` event patches every client, including the originator, so the client does not double-count (`reactions.rs:100-108`, `Chat.tsx:351-359`, `lib/chat.ts::applyReaction`).

Thin:
- Fixed six-emoji set, client-side only: `REACTION_EMOJIS = [thumbsup, heart, joy, tada, check, eyes]` (`apps/web/src/lib/chat.ts:44`). The server accepts any short string up to 32 bytes (`reactions.rs:30`, `reactions.rs:37`), so the limit is purely the client picker. There is no full emoji picker, no search, no skin tones, no custom/uploaded emoji, and no per-emoji hover list of who reacted (the `mine` flag exists but the reactor names are not surfaced).

### 3. Attachments and media

Present:
- Upload (base64 JSON, member-only, 5 MB cap) and download (member-only, streamed), plus a metadata endpoint so a linked attachment renders without downloading bytes. `POST /v1/chat/channels/:id/attachments`, `GET /v1/chat/attachments/:att`, `GET /v1/chat/attachments/:att/meta` (`lib.rs:84-89`, `attachments.rs`).
- A message links to one attachment via `attachment_id` (`migrations/0005_chat_message_attachment.sql`, `messages.rs:123-137`).
- Client staging with image preview, drag-and-drop, and paste-to-attach (`Chat.tsx:522-536`, `Chat.tsx:941-963`), inline image render or a download chip (`apps/web/src/components/Attachment.tsx`).

Missing or thin:
- Bytes live in Postgres (`chat_attachments.data BYTEA`, `migrations/0003_chat_slice3.sql`), capped at 5 MB (`attachments.rs:16`). The code comment says object storage is "a later slice" (`attachments.rs:2`). No S3/GCS/R2, so large files, video, and audio are impractical, and the DB carries blob weight.
- One file per message. `attachment_id` is a single nullable column; there is no multi-file message and no album/grid.
- No previews beyond raw images. No thumbnail generation, no PDF preview, no video/audio player, no file-type icons beyond the generic paperclip chip.
- No image lightbox/gallery; clicking an image just `window.open`s the object URL (`Attachment.tsx:56`).
- No virus/type scanning, no EXIF stripping.

### 4. Mentions and notifications

This is the weakest area against the benchmark.

Present:
- Unread counts per channel. `GET /v1/chat/channels/:id/unread` (`read.rs::unread`), polled every 15 s by the client and shown as a sidebar badge (`Chat.tsx:231-236`, `Chat.tsx:790`).
- Push intent scaffolding. On a new message, members who are offline and have a registered device are computed as push targets and logged, but not delivered: "the actual APNS/FCM delivery is a deploy-time integration" (`push.rs:1-4`, `push.rs:37-46`). Device registration exists (`POST /v1/chat/devices`, `devices.rs`).

Missing:
- No @-mentions. There is no mention parsing, no mention storage, no autocomplete, no "mentions me" flag, and no per-message highlight. Nothing in `messages.rs` or `Chat.tsx` handles `@`.
- No mention badges or a "@ mentions" inbox. The sidebar badge is a raw unread count only (`Chat.tsx:790`); there is no distinction between "unread" and "mentioned."
- No per-user notification socket. Every websocket is scoped to one channel (`realtime.rs::ws_handler` requires a `channel` query param and channel membership, `realtime.rs:124-158`). The client opens exactly one socket, for the active channel (`Chat.tsx:316-321`). Consequences stated in the code itself:
  - Presence for a DM only updates while that DM is open (`Chat.tsx:773-776`).
  - Call ringing only reaches a callee who has that conversation open (`lib/call.ts:6-7`).
  - There is no way to receive a message event, a mention, or a ring for a channel you are not currently viewing.
- No desktop notifications. There is no `Notification` API use anywhere in `apps/web/src`.
- No real push. As above, `push.rs` logs intent; there is no APNS/FCM client, no VAPID web-push, and the task for mobile push is separate (`docs/tasks/chat/TASK-CHAT-011-mobile-push/spec.md`).
- No notification preferences (per-channel mute, all/mentions/nothing, keywords, schedules).

### 5. Channels and org

Present:
- Create a named group channel; creator becomes owner. `POST /v1/chat/channels` (`channels.rs::create`).
- Find-or-create a two-person DM, idempotent from either side. `POST /v1/chat/dms` (`channels.rs::create_dm`).
- List the caller's channels, with the DM partner's id resolved for labeling. `GET /v1/chat/channels` (`channels.rs::list`).
- Membership: add (owner/admin), list (any member), remove (owner-only). `POST/GET /v1/chat/channels/:id/members`, `DELETE .../members/:subject` (`members.rs`). Roles are `owner > admin > member` (`db.rs::is_manager`, `members.rs:55`).
- Client: create-channel and add-people flows through the picker (`Chat.tsx:633-659`), DM list sorted by recent activity (`Chat.tsx:754-758`).

Missing or thin:
- No channel topic, description, or purpose. `chat_channels` has only `name`, `created_by`, `kind` (`migrations/0001`, `migrations/0006`). The header shows only the name and a coarse subtitle like "3 online" (`Chat.tsx:795-803`).
- No public vs private channels and no join/browse. A channel is only reachable if you are already a member (RLS + `role_in_channel` gate every read, e.g. `messages.rs:85-97`). There is no channel directory, no "browse channels," no self-join, no invite link.
- No archive or delete of a channel, no rename (the create is the only write to channel metadata).
- Membership management UI is minimal. There is an "add people" picker but no member roster view in the client, no role editing UI, no remove-from-channel UI, no leave-channel action (the server has `DELETE members/:subject` but the client never calls it; `Chat.tsx` has no remove path).
- No group DM (a multi-person DM without a channel name). `create_dm` is strictly two-person (`channels.rs:146-151`), and a group is a named channel. Slack/Discord both have unnamed multi-person DMs.
- No channel sections/folders, no favorites/starred channels, no drag-reorder. DMs sort by activity but groups keep server order (`Chat.tsx:751`).

### 6. Presence, typing, read receipts

Present and reasonably complete for the single-channel model:
- Presence. Tracked from live connections as a per-channel open-connection count (`realtime.rs::Presence`), broadcast on the 0<->1 edge (`realtime.rs:169-186`, `realtime.rs:221-237`), listed via `GET /v1/chat/channels/:id/presence` (`realtime.rs::presence_list`). Client renders online dots on avatars (`components/Avatar.tsx:29`, `Chat.tsx:777-783`).
- Typing. Client sends `{type:"typing"}` throttled to 1.5 s (`Chat.tsx:458-470`), server rebroadcasts as `ChatEvent::Typing` (`realtime.rs:255`), client shows "X is typing..." with a 2.5 s clear (`Chat.tsx:368-371`, `Chat.tsx:1126-1128`).
- Read receipts. Per-(channel, subject) last-read marker via `POST /v1/chat/channels/:id/read`, read back with `GET .../receipts` (`read.rs::mark`, `read.rs::receipts`, `migrations/0004`). Broadcast as `ChatEvent::Read` (`read.rs:77-84`), rendered as a "Seen by" line under the caller's last message (`Chat.tsx:738-749`, `Chat.tsx:1115-1120`). Auto-mark-read is debounced 500 ms (`Chat.tsx:448-456`).

Thin (all downstream of the per-channel socket, item 4):
- Presence is not global; it is per-open-channel (`Chat.tsx:773-776`). A teammate shows offline in the DM list until you open that DM.
- Typing has no aggregation ("Several people are typing") and only tracks the most recent typer (`typingSubject` is a single string, `Chat.tsx:97`).
- No away/idle/do-not-disturb states; presence is a binary online/offline derived from socket count.

### 7. Search

Present:
- In-channel, accent- and case-insensitive substring search, backed by a GIN trigram index and a `chat_norm(body)` immutable function (`migrations/0003_chat_slice3.sql`, `messages.rs::search`). `GET /v1/chat/channels/:id/search?q=`. Client search bar with inline results (`Chat.tsx:908-937`).

Missing or thin:
- Channel-scoped only. There is no global/cross-channel search; the endpoint takes a channel id in the path (`lib.rs:74`) and the client only searches the active channel (`Chat.tsx:703`).
- No filters (from:person, in:channel, has:link/file, before/after date), no ranking (it is a `LIKE` substring match ordered by recency, `messages.rs:486-491`), no snippet highlighting (the result shows the raw body, `Chat.tsx:931`).
- No jump-to-message. Clicking a search result does nothing; it cannot scroll the timeline to that message. Search does not cover thread replies distinctly and does not cover DMs across the org.

### 8. Calls

Present:
- 1:1 WebRTC audio/video over the chat signaling relay. The client drives `getUserMedia` + `RTCPeerConnection` and exchanges offer/answer/ICE inside `{type:"signal", to, data}` frames (`apps/web/src/lib/call.ts`), which the server forwards privately to the addressed subject (`realtime.rs::handle_inbound` "signal" case, `realtime.rs:256-266`; delivery gated to `to == me`, `realtime.rs:195`). Call UI, incoming ring, mute/cam toggles, PiP in `components/CallOverlay.tsx`.

Limits (stated in the code):
- No TURN, STUN only (two Google STUN servers, `lib/call.ts:11-16`). The comment notes TURN "is a later addition" (`lib/call.ts:6`); calls will fail on restrictive/symmetric-NAT networks.
- 1:1 only. No group calls, no screen share, no call from a group channel to multiple peers (the group path just opens the people picker to choose one peer, `Chat.tsx:661-670`).
- Ring only when the channel is open. Because signaling rides the active channel socket, the callee must have that conversation open to get the ring (`lib/call.ts:6-7`). This is the per-user-socket gap again.
- No call history, no missed-call record, no voicemail, no in-call chat, no recording.

### 9. Translation and AI

Present:
- Inline translate-to-English of a message. `POST /v1/chat/translate {text, target_lang}` calls the ai-gateway server-side with the `chat.fast` alias and a translation system prompt (`translate.rs`, prompt in `translate.rs::build_messages`). The message text only leaves chat to the one configured gateway, never to the browser (`translate.rs:2-6`). Client shows an inline translation block, toggle-off on a second click, and a graceful "unavailable" note (`Chat.tsx:586-621`, `Chat.tsx:1051-1060`).
- Gateway-dependent and safe: when `AI_GATEWAY_URL` is unset or the call fails, it returns a clean 502 and never blocks chat (`translate.rs:99-107`, `translate.rs::call_gateway`).

AI-native openings (present capability, not yet built into chat):
- The ai-gateway already exposes `/v1/chat` (completion) and `/v1/embeddings` (`services/ai-gateway/src/server/mod.rs:359-360`). Chat only uses `/v1/chat` for translation today (`translate.rs:109`). The same server-side path, with the same tenant header and audit pattern, could power: thread/channel summarize, smart replies, "catch me up," action-item extraction, and semantic search over `/v1/embeddings`. None of these exist yet.
- Capture already emits `chat.message_created` etc. as interaction-events into the BRAIN audit chain when `CAPTURE_ENABLED` is on (`capture.rs`, wired in `messages.rs:201-216`), which is the substrate an "assistant that reads the channel" would build on. Message bodies are never inlined (pointer refs only, `capture.rs:44-63`), so a summarizer would need to read `chat_messages` directly.
- The memory notes reference a "Lumi/GENIE" assistant and `TASK-CHAT-008-lumi-mention/spec.md`; there is no `@lumi` or bot participant in the current code.

### 10. i18n and Vietnamese UI

Missing entirely:
- No i18n framework or locale files. `apps/web/package.json` depends only on `react` and `react-dom`; there is no `react-intl`, `i18next`, or message catalog.
- Every user-facing string is hardcoded English: "Welcome to CyberOS Chat" (`Chat.tsx:861`), "No messages yet. Say hello." (`Chat.tsx:967`), "Direct message" / "Active now" / "Channel" (`Chat.tsx:795-803`), "Reconnecting..." (`Chat.tsx:851`), and every title/placeholder in the components. `lib/chat.ts::formatDay` returns English "Today"/"Yesterday" (`lib/chat.ts:114-116`).
- The one nod to Vietnamese is functional, not UI: accent-insensitive search (`chat_norm`, item 7) and a translate-to-English action. There is no VN interface, no VN date/number formatting, and no language switch, on a team the memory describes as bilingual.

### 11. Admin and moderation

Almost absent:
- The only moderation primitive is channel-role delete: a manager can delete another member's message (`messages.rs:412-417`), and an owner can remove a member (`members.rs::remove`, though the client never calls it).
- Every action writes an audit-chain row (`audit.rs::emit`, called across the handlers), and the console has a read-only audit browser (`auditlog.rs`, `GET /v1/chat/audit`). That is observability, not moderation.
- Missing: workspace/tenant admin console for chat, retention/deletion policies, message export/DSAR (there is a task `TASK-CHAT-012-dsar-export/spec.md` but no code), banned words/content filters, report-a-message, user suspend/deactivate from within chat, channel-level permissions beyond the three roles, guest accounts, legal hold.

### 12. Mobile, responsive, accessibility

Thin:
- Layout is fixed-width desktop. Sidebar is a hard `270px` (`styles.css:121`), thread panel `360px` (`styles.css:250`), and `styles.css` has no `@media` query at all, so there is no responsive collapse; on a phone the three columns will not fit.
- There is a PWA/installable effort noted in memory and a Tauri desktop wrapper, but the chat layout itself is not adaptive.
- Accessibility is partial. Icon buttons mostly have `title`/`aria-label` (e.g. `CallOverlay.tsx:51`, `Chat.tsx:887`), SVG icons are `aria-hidden` (`icons.tsx:135`). But: `window.confirm` for delete (`Chat.tsx:556`) and `window.prompt`-style flows are not accessible dialogs; there is no focus management/trap in the modals (`PeoplePicker`, `ProfileEditor` use a plain `.picker-bg`); no visible focus ring beyond the default; no keyboard nav for the message list or channel list; no live-region announcements for incoming messages; contrast on muted text is questionable (see Half B).

---

### Ranked gap table (impact vs effort)

Impact = how much it closes the distance to a modern team chat / how much the team feels it. Effort: S (days), M (1-2 weeks), L (multi-week). "Where": B backend, C client, X external dependency/infra.

| # | Gap | Impact | Effort | Where | Notes / dependency |
|---|-----|--------|--------|-------|--------------------|
| 1 | Per-user notification socket (presence, ring, messages, mentions while not in-channel) | Very high | M | B + C | Unblocks 4/6/8 gaps; the single most leveraged backend change. New ws that fans out the user's events. |
| 2 | @-mentions (parse, store, autocomplete, "mentions me", highlight) | Very high | M | B + C | Needs mention storage + the directory (already loaded, `Chat.tsx:195`). Pairs with #1 for notifications. |
| 3 | Message formatting: markdown, code blocks, inline code, links (linkify + sanitize) | Very high | M | C (+ small B if server-side render) | Add a markdown renderer + sanitizer; today body is raw text (`Chat.tsx:1032`). No lib present. |
| 4 | Desktop / web-push notifications | High | M | C + B + X | Browser `Notification` + VAPID web-push; APNS/FCM for mobile is separate (`push.rs`, TASK-CHAT-011). Depends on #1. |
| 5 | Object storage for attachments (S3/GCS/R2) + larger cap + multi-file | High | M | B + X | Replaces BYTEA (`migrations/0003`, `attachments.rs:16`); enables video/audio, albums. |
| 6 | Global / cross-channel search + jump-to-message + filters | High | M | B + C | Endpoint is channel-scoped (`lib.rs:74`); add a tenant-wide search + result navigation. |
| 7 | Channel management: topic/description, public/private, browse+join, archive, member roster + role UI, leave | High | L | B + C | `chat_channels` needs columns (`migrations/0001`); client needs a roster + browser. |
| 8 | Full emoji picker (search, skin tones, custom emoji) + "who reacted" | Medium | S-M | C (+ B for custom emoji) | Server already accepts any emoji (`reactions.rs:37`); today the client caps it to six (`lib/chat.ts:44`). |
| 9 | i18n + Vietnamese UI (framework, catalog, locale dates, language switch) | High (for this team) | M | C | No i18n today (`package.json`); all strings hardcoded English. |
| 10 | AI-native features: summarize, smart replies, action items, semantic search | High | M | B (+ C) | Reuses the ai-gateway `/v1/chat` + `/v1/embeddings` already wired for translate (`translate.rs`, `ai-gateway .../mod.rs:359-360`). No new external dep. |
| 11 | Message pagination / load-older in the client | Medium | S | C | Backend supports `?before=`/`?limit=` (`messages.rs:78-82`); client never scrolls up (`Chat.tsx:282`). |
| 12 | Drafts (per-channel), pin, save/bookmark, copy-link, forward/quote | Medium | M | B + C | None exist today. Drafts can be client-only first. |
| 13 | Group DM (unnamed multi-person) | Medium | M | B + C | `create_dm` is strictly two-person (`channels.rs:146-151`). |
| 14 | TURN server + group calls + screen share | Medium | L | X + B + C | STUN-only today (`lib/call.ts:11-16`); depends on #1 for reliable ringing. |
| 15 | Attachment previews (thumbnails, PDF/video/audio players), image lightbox/gallery | Medium | M | C (+ B for thumbnails) | Today: inline `<img>` or a generic chip (`Attachment.tsx`). |
| 16 | Admin / moderation console + retention + DSAR export + report/suspend | Medium | L | B + C | Only role-delete + audit read exist today (`messages.rs`, `auditlog.rs`); tasks exist (TASK-CHAT-012). |
| 17 | Responsive / mobile layout | Medium | M | C | `styles.css` has no media queries; fixed 270/360px columns. |
| 18 | Accessibility: focus trap in modals, keyboard nav, accessible delete dialog, live regions | Medium | M | C | `window.confirm` delete (`Chat.tsx:556`); no focus management in pickers. |
| 19 | Notification preferences (mute, all/mentions/none, keywords) | Medium | M | B + C | Depends on #2/#4. |
| 20 | Message editing/deleting for attachment messages + delete the bytes | Low-medium | S | B + C | Edit hidden when attachment present (`Chat.tsx:1095`); delete leaves bytes (`messages.rs:418`). |

Suggested first wave (highest leverage, contained effort): #1 per-user socket, then #2 mentions and #3 formatting on top of it, then #9 i18n and #10 AI-native. Those five change the daily feel the most and #1 unblocks the rest.

---

## HALF B - UI/UX assessment

Source of truth: `apps/web/src/styles.css` (326 lines, one hand-written stylesheet, no framework) and the JSX in `Chat.tsx` + `components/*`. The live impression (warm dark-brown, thin sidebar, plain rows, sparse dark expanse, tiny controls, low contrast) is explained by the specifics below.

### 13. What is wrong, concretely

Color system and contrast:
- The palette is near-monotone brown. Backgrounds step through `--bg #1e0f08`, `--panel #2a1509`, `--panel-2 #39200f`, `--line #3c2414` (`styles.css:2-6`) - four very close dark browns with low separation, so panels barely read as distinct surfaces. The only accent is `--ochre #f4ba17` (`styles.css:10`).
- Text tiers are `--ink #f6ecdd`, `--muted #c2ab8e`, `--faint #917a62` (`styles.css:7-9`). `--faint` on `--panel`/`--bg` is low contrast and is used for a lot of real content: timestamps, section labels, the connection status, DM subtitles, day separators (`styles.css:128`, `:131`, `:155`, `:162`, `:170`, `:176`). Several of these are likely below WCAG AA for small text.
- Ochre is the single accent and it carries everything: active channel tint (`--ochre-soft`, `styles.css:11`, `:137`), primary buttons (`styles.css:53`), focus rings (`styles.css:82`), links, sender-name-when-mine (`styles.css:175`), reaction "mine," translate rail. There is no secondary accent and no semantic color use beyond `--ok`/`--bad` (`styles.css:12-13`), so hierarchy is flat: everything important is the same yellow.

Typography:
- One system font stack (`styles.css:15`), no display/mono pairing (a chat with code blocks will want a mono face). Sizes are ad hoc and clustered: body 14px (`styles.css:178`), names 14px (`styles.css:174`), timestamps 10-11px (`styles.css:170`, `:176`), labels 11px (`styles.css:131`). There is no defined type scale, so headings, names, and metadata are only 1-3px apart and the hierarchy is weak.
- Line-height on the message body is 1.5 (`styles.css:178`), which is fine, but rows are packed tightly (see spacing).

Spacing, density, rhythm:
- Message rows are `padding: 4px 14px`, and grouped rows drop to `1px` top/bottom (`styles.css:166-168`). Combined with the 36px gutter (`styles.css:169`), the timeline is dense and a little cramped, yet the surrounding page feels empty because there is no max content width and no card/inset around the timeline - text runs full-bleed across a wide dark pane.
- There is no consistent spacing scale (values are 4/6/8/10/11/13/14/16/18/20/22px scattered through the file), so vertical rhythm is irregular between the header, day separators (`margin: 18px 6px 10px`, `styles.css:162`), rows, composer, and typing line.
- The empty channel state is a single centered line, "No messages yet. Say hello." (`Chat.tsx:965-968`, `.empty-sub` only), with no illustration, no channel context, no first-action prompt. The no-channel state is slightly richer (mark + title + sub, `Chat.tsx:856-863`) but still minimal.

Layout:
- Fixed three-column desktop shell: sidebar 270px (`styles.css:121`), main flex, thread 360px (`styles.css:250`). No responsive behavior anywhere (`styles.css` has zero `@media`). On smaller widths the columns will overflow or crush.
- The channel header is a thin 10px-padded bar with the name, a coarse subtitle, and four 36px icon buttons (`Chat.tsx:866-906`, `styles.css:150`). It does not show member avatars, topic, or a member count as a control - just "3 online" text (`Chat.tsx:800-802`).
- The sidebar is a single scroll of Channels then Direct messages (`Chat.tsx:827-848`); there is no search/filter box, no unread filter, no sections/favorites, no threads/mentions/activity entries that a modern client puts above channels.

Message row design and grouping:
- Grouping works (5-min window, `Chat.tsx:717-721`) but the grouped row shows only a hover timestamp in the gutter (`styles.css:170`), and the avatar+name only appear on the first message of a group (`Chat.tsx:993`). This is the Slack model and is fine; the issue is the row has no hover affordance beyond a nearly invisible `rgba(255,255,255,0.02)` background (`styles.css:167`), so the interactive area is unclear.
- Reactions strip is reasonable (`styles.css:190-195`) but the pills are small (12px, `styles.css:191`).

Empty states, hover, the action bar:
- The hover action bar floats top-right, 28px buttons, `opacity 0 -> 1` on row hover (`styles.css:179-182`, `Chat.tsx:1062-1113`). At 28px with `--muted` icons on a dark panel these are small and low-contrast; the bar also overlaps the row above (`top: -13px`, `styles.css:179`), which can feel jumpy. On touch there is no hover, so the actions are unreachable (no long-press or always-visible affordance).
- The reaction picker is a fixed six-emoji popover (`styles.css:197-199`, `Chat.tsx:1072-1085`) - fine as a quick bar, but there is no "more" affordance into a full picker.

Composer:
- Auto-growing textarea + paperclip + send, capped at 140px (`styles.css:211-221`, `Chat.tsx:1157-1190`). It works but is bare: no formatting toolbar, no emoji button in the composer, no @-mention trigger, no slash-commands, no "shift-enter for newline" hint, no send-vs-newline affordance. The send button is ochre and always the same weight whether or not there is content (only `disabled` opacity changes, `styles.css:221`).

Dark/light and brand:
- Dark only. There is no light theme and no theme toggle; all colors are literal hex in `:root` with no light override.
- Brand expression is present but faint: the topbar wordmark "CyberOS" with an ochre "OS" and an italic slogan (`styles.css:40-43`), and ochre accents. It does not yet feel like a designed product - the umber/ochre is applied as flat fills rather than as a system with depth, and "Turn Your Will Into Real" appears only as small italic slogan text.

### 14. Redesign direction (not a rewrite)

Target look and feel: keep the warm CyberSkill identity (umber ground, ochre accent, "Turn Your Will Into Real") but make it a designed dark product - more surface separation, stronger text contrast, a real type and spacing scale, and denser-but-legible message rows. Think "Slack/Linear-grade polish in an umber/ochre skin," not a new visual language. Keep it a single hand-written `styles.css` + React with no heavy UI framework; the surface area does not justify Chakra/MUI, and a token-driven CSS refactor gets 90% of the benefit. A small headless primitive for accessible modals/menus (e.g. Radix primitives, unstyled) is the only dependency worth arguing for, and only to fix focus-trap/keyboard gaps (item 18) - style it with the same tokens.

Tightened design tokens (roles, not just hues):
- Surfaces: define `--surface-0` (app bg), `--surface-1` (sidebar/panels), `--surface-2` (raised: header, composer, cards), `--surface-3` (popovers/modals), each a clearer step apart than today's four browns; add one true elevation shadow token. Widen the lightness gap between `--panel` and `--panel-2` so panels separate.
- Text: keep `--ink` for primary, lighten `--muted` and especially `--faint` until small text clears WCAG AA on `--surface-1` (raise the faint tier or stop using it for load-bearing content like timestamps and status). Add `--text-on-ochre` (the existing `--umber-900`) as an explicit role.
- Accent: keep `--ochre` as the single brand accent but reserve it for primary action + active + brand, not for every metadata state. Add a neutral "interactive" hover token (a warm gray) so hover/selection is not always yellow. Keep `--ok`/`--bad` and add `--warn`/`--info` for message states (sending, failed, mention).
- Type scale: define a real scale, e.g. 11 / 12 / 13 / 15 / 18 / 22 with named roles (meta, body, name, title, section-header). Pull body to 15px for readability; keep meta at 11-12 but on a higher-contrast tier. Add a `--font-mono` for code blocks.
- Spacing scale: adopt a 4px base with a fixed ramp (4/8/12/16/24/32) and replace the scattered ad-hoc values so header/row/day-sep/composer share one rhythm.
- Radii: you already lean on `border-radius: 30%` for avatars (a squircle look, `styles.css:64`) and 8-16px on cards; formalize `--r-sm 8 / --r-md 12 / --r-lg 16 / --r-avatar 30%` and apply consistently.

Component-level fixes:
- Sidebar: add a top search/filter, add a "Threads" and "Mentions" entry above Channels (pairs with Half A #1/#2), section headers with clearer contrast, unread rows with a left accent bar rather than only bolding, and a proper account/status control at the bottom (presence + set-status), replacing the plain "Connected" text (`Chat.tsx:849-852`).
- Channel header: show member avatars (stacked) + count as a clickable control that opens the roster, show the topic inline, and group the call/video/search/members icons with slightly larger (32-36px) targets and clearer hover.
- Message rows + grouping: keep the group model but give rows a clear hover surface (raise the `0.02` background), align the gutter and content on the spacing scale, render the sender name at the title tier and the timestamp at a higher-contrast meta tier, and add a subtle inset/max-width to the timeline so it does not run full-bleed on wide screens.
- Composer: make it a raised card with a thin top border, add a left affordance row (attach, emoji, @-mention, and later formatting), a clear send button that changes weight when there is content, and a small "Shift+Enter to add a line" hint. This is where formatting (Half A #3) and mentions (#2) surface.
- Empty states: give the empty channel a real state (channel name + purpose + first-action buttons: "Send the first message," "Add people," "Set a topic") and an illustration mark reusing `.empty-mark` (`styles.css:239`). Same for search-no-results and directory-unavailable.
- Action bar: make it slightly larger (32px), higher contrast, and anchored so it does not overlap the row above; add a "more actions" overflow into a proper menu; provide an always-visible or long-press path for touch.
- Reactions strip + picker: keep the inline pills but size them up a touch, add a "+" that opens a full emoji picker (Half A #8), and a hover tooltip listing who reacted.
- Threads panel: it is already the right shape (`components/ThreadPanel.tsx`); align it to the new tokens, add the same composer treatment, and make it slide over rather than push on narrow widths.
- Modals (picker, profile, ring): move to a headless primitive for focus trap + Escape + return-focus, keep the current `.picker` styling, and fix the accessibility gaps (item 18) at the same time.

Structural refactor to do first:
- `Chat.tsx` is 1240 lines and owns state, websocket, all handlers, and the entire render tree. Before a visual overhaul, split it (there is already an open task, "Split Chat.tsx into components"). Suggested seams: `useChatSocket` (the effect at `Chat.tsx:262-397`), `useChannels`/`useUnread`, `<Sidebar>`, `<ChannelHeader>`, `<MessageList>` + `<MessageRow>` (the map at `Chat.tsx:970-1123`), `<Composer>`, and keep the modals as they are. This makes the token/redesign work land in small components instead of one giant file, and it isolates the message-row redesign (the most-touched surface).

Brand while modernizing: lead with umber as the ground and ochre as a disciplined single accent, add depth through surface steps and one shadow rather than more colors, use the squircle avatar/radius language consistently as a brand tell, and give the slogan a real home (e.g. the login and the empty-app state) rather than only tiny italic text in the topbar. The result stays unmistakably CyberSkill but reads as a finished product.

---

## Notes and honest limits of this audit

- This is a static read of the code as of 2026-07-02. It did not run the service, the client, or any test, and did not run git.
- Contrast claims (Half B) are eyeballed from the hex values in `styles.css`; the exact WCAG pass/fail per pairing should be measured, but the near-monotone palette and heavy use of `--faint` for load-bearing text make several failures likely.
- "Missing" means not found in the read files (`services/chat/src/*`, `services/chat/migrations/*`, `apps/web/src/*`). A few features exist as authored tasks under `docs/tasks/chat/` (mobile push, DSAR export, Lumi mention, imports) but have no code in the module today; those are flagged inline.
