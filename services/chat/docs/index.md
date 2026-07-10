---
title: CHAT — P0 dogfood gate · Mattermost fork · @lumi memory capture · CyberOS
source: website/docs/modules/chat/index.html
migrated: FR-DOCS-002
---

CHAT is the **internal team-communication module** , layered on a Mattermost fork that provides battle-tested messaging primitives (channels, threads, attachments, reactions, presence, mobile push). On top of that fork, CyberOS adds: a native auth bridge to the AUTH service (no separate Mattermost user database), a Tauri-based desktop + mobile shell that matches the rest of CyberOS, PGroonga as the full-text engine (for Vietnamese tokenisation — measured recall ≥ 80% on a public VN test set), a Slack/Zalo import path, AI-native features driven by the AI Gateway (`@genie` inline mentions, `/summarise` thread compactions, smart-reply, daily digest), an offline draft sync engine using Yjs CRDT, and a memory bridge that streams every message into Layer 3 of memory within p95 ≤ 5 s. The compliance export reuses Mattermost's format plus the CyberOS audit chain anchor — so regulators see one bundle. 

Status

Planned · slice-1 design-locked

P0 · P0 · slice 3 → P0 · exit · gates P0 exit

Decom gate

Slack + Zalo killed by P0 · exit

the dogfood signal · P0 exit criterion

Strategy

Fork Mattermost

open-core · MIT/Apache · pinned commit

Search engine

PGroonga + TinySegmenter

VN tokeniser · recall ≥ 80%

Send p95

≤ 200 ms

primary perf target

Availability

≥ 99.9%

required to displace Slack

memory ingest p95

≤ 5 s

every @lumi msg → memory row

E2EE decision

Per-tenant Postgres encryption

NOT end-to-end · see §2.7

Desktop / mobile

Tauri + Yjs CRDT

offline-first drafts

Voice ASR

Whisper-large-v3

self-hosted · VN + EN

0

## The bigger picture — three strategic roles

CHAT is the most strategically loaded P0 module because it carries three obligations that no other module shares simultaneously. Reading it under any single lens misses two-thirds of the design. The three roles below MUST all be true at P0 exit (P0 · exit); otherwise the platform fails its dogfooding bet. 

Role 1 · Dogfood gate

⛔

Kill Slack + Zalo by P0 · exit

Per audit-and-plan §3, P0 exit = "Slack + Zalo decommissioned." The 10-Member CyberSkill team must voluntarily abandon Slack + Zalo before P0 · exit. If they don't, the platform's central thesis (internal-first compounds into external) collapses — the dogfooding signal that gates P1+ never materialises.

Concrete KPI: `messages_in_chat / total_team_messages` ≥ 95% by P0 · exit · §kpi-decom

Role 2 · memory capture surface

🧠

@lumi → memory row

Per MEMORY_AUTOSYNC_DESIGN.md §5, CHAT is one of three first-class capture surfaces (Cowork, Claude Code, CHAT). Every `@lumi`-tagged message emits a memory row to the user's Personal memory. Untagged messages are NOT auto-captured (privacy floor). Sample-based capture optional via per-channel config for high-signal channels (#decisions, #rfcs).

memory ingest p95 ≤ 5 s · sync_class default `shareable` in shared channels, `personal` in DMs

Role 3 · Vietnamese-first chat

🇻🇳

PGroonga search · > 80% VN recall

No global vendor (Slack, Discord, Teams) handles Vietnamese full-text well — they rely on naive whitespace tokenisation that misses 40%+ of recall for diacritic-laden compound terms. PGroonga + TinySegmenter + VietBERT-derived rules give CHAT a defensible local edge. Be Vietnam Pro font throughout.

Recall measured against public VN test set · NFR-CHAT-001

### P0-exit gate dependency map

graph LR memory["🧠 memory  
shipped"] AI["⚡ AI Gateway  
@ P0 · slice 1"] AUTH_STUB["🔐 AUTH stub  
@ P0 · slice 2"] MCP["🔌 MCP Gateway  
@ P0 · slice 3"] CHAT["💬 CHAT  
@ P0 · exit"] SLACK_OFF["✂️ Slack decommissioned"] ZALO_OFF["✂️ Zalo decommissioned"] P0_EXIT["✅ P0 exit · P0 · exit"] memory --> CHAT AI --> CHAT AUTH_STUB --> CHAT MCP --> CHAT CHAT --> SLACK_OFF CHAT --> ZALO_OFF SLACK_OFF --> P0_EXIT ZALO_OFF --> P0_EXIT classDef shipped fill:#f5ede6,stroke:#45210e,stroke-width:2px classDef gating fill:#fde7b3,stroke:#9c750a,stroke-width:2px classDef self fill:#f9c64f,stroke:#45210e,stroke-width:2.5px classDef gate fill:#cba88a,stroke:#2a1208,stroke-width:2px class memory shipped class AI,AUTH_STUB,MCP gating class CHAT self class SLACK_OFF,ZALO_OFF,P0_EXIT gate 

**What this means in practice:** CHAT is the LAST P0 module to ship (P0 · exit), but it is the GATE for P0 exit. If AI Gateway slips, CHAT slips, Slack survives, dogfooding fails. The 28-day window between AUTH stub (P0 · slice 2) and CHAT (P0 · exit) is the single most-watched milestone in the platform. 

1

## Why CHAT exists

Building a chat server from scratch is months of work whose differentiator (CyberOS auth, AI features, Vietnamese search, memory bridge) is at the edges, not the middle. Mattermost has solved channels, threads, attachments, presence, mobile push, federation, and admin tooling at production quality. Forking it lets the CyberOS team focus on the differentiators while keeping a vendor-independent path (no Mattermost SaaS dependency, no per-seat fee). 

🏗

Don't rebuild the wheel

Mattermost has years of production hardening on the messaging core. Forking + integrating lets us ship in months not years.

🇻🇳

Vietnamese-first

PGroonga (with TinySegmenter and VietBERT-derived rules) gives the Vietnamese tokenisation the upstream lacks. Search recall ≥ 80% on the public VN test set.

🧠

AI is a feature, not a bolt-on

`@genie` inline mentions, `/summarise` threads, smart-reply, daily digest — all driven by the AI Gateway, persona-stamped, audit-chained.

The bet: take the part Mattermost did well (the messaging substrate), wrap it in the parts CyberOS does well (auth, audit, AI, Vietnamese), and ship a chat product that feels like Slack-quality and is end-to-end ours. 

2

## What it does — 5W1H2C5M

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is CHAT?| A team-chat module. Mattermost-derived core + CyberOS auth/UI/search/AI plug-ins. Channels (public + private + DM), threads, file attachments, reactions, mentions, voice messages, smart replies, daily digest, Slack/Zalo import, compliance export.  
**5W · Who**|  Who uses it?| **Members** of a tenant (team chat). **Genie** (`@genie` mention → AI answer with citations). **CUO digest bot** (morning brief). **Alert bots** (#cyberos-alerts from OBS). **Owner:** CPO seat.  
**5W · When**|  When does it run?| 24/7. Hot push for online members; FCM/APNs for mobile when offline. memory ingest within 5 s of message create.  
**5W · Where**|  Where does it run?| Fargate task (Go core from Mattermost) + Postgres + Redis + MinIO/S3 for attachments. Per-region; SG-1 at P0.  
**5W · Why**|  Why a separate module?| Because chat is the highest-bandwidth surface between humans + AI; co-locating it with the platform unlocks audit, search, and AI features the SaaS world can't match.  
**1H · How**|  How does it work?| Browser/desktop/mobile clients hit the CyberOS API gateway. CHAT service serves messages via WebSocket + HTTP. Postgres stores messages, channels, users (synced from AUTH). PGroonga indexes content with VN tokenisation. AI features call AI Gateway. memory bridge tails the Postgres logical replication slot and streams messages into memory's Layer 3 corpus.  
**2C · Cost**|  Cost?| P0: ~$45/month (Fargate + RDS small). 50-tenant: ~$280/month (Fargate scale-out + Redis cluster + MinIO).  
**2C · Constraints**|  Constraints?| (a) PGroonga tokenisation must hit ≥ 80% recall on VN test set. (b) Send p95 ≤ 200 ms. (c) Per-channel ACL enforced; DMs namespace-isolated in memory. (d) PDPL Art. 14 DSAR — full message export per subject.  
**5M · Materials**|  Stack?| Go (Mattermost-derived) · PostgreSQL 16 + PGroonga · Redis 7 (presence + push fanout) · MinIO (attachments, S3-compatible) · Tauri (desktop + mobile shell) · Yjs CRDT (offline drafts) · Whisper-large-v3 (ASR) · AI Gateway (summarisation, smart-reply).  
**5M · Methods**|  Method choices?| WebSocket for hot push; long-poll fallback. Logical replication slot to memory. PGroonga full-text index per channel. CRDT for offline draft sync. ASR runs locally on a shared GPU pod.  
**5M · Machines**|  Deployment?| Fargate (Go service · 2 CPU · 4 GB). RDS Postgres 16. Redis 7. MinIO on EBS-backed volumes.  
**5M · Manpower**|  Who maintains?| 0.3 FTE CPO + 0.5 FTE CTO at P0. P1+: 1 FTE PM + 1 FTE FE eng + ongoing.  
**5M · Measurement**|  How measured?| N(FR pending) (send p95 ≤ 200 ms), N(FR pending) (availability ≥ 99.9%), (FR pending) (VN search recall ≥ 80%), (FR pending) (memory ingest p95 ≤ 5 s).  
  
2.5

## Real-time stack — why Mattermost fork (decision-locked)

Four real-time architectures were evaluated for CHAT. The decision (locked 2026-05-14) is **fork Mattermost** \+ layer CyberOS auth/UI/search/AI on top. The decision-record table below is the formal rationale. 

Option| Cost to ship by P0 · exit| Production hardening| Vendor risk| Vietnamese-tokenisation fit| Verdict  
---|---|---|---|---|---  
**Mattermost fork** (chosen)| Low — ~3 person-weeks for auth/UI/search wrap| High — years of production scaling at > 1M MAU deployments| Medium — Mattermost has had license churn (MIT → MIT+SSPL +MIT once); we pin to a known-MIT/Apache commit| Medium — replace stock tokeniser with PGroonga| ✅ Ship-by-P0 · exit wins  
Matrix (Synapse / Conduit / Dendrite)| High — federation complexity is gratuitous for a single-tenant chat; Dendrite is still pre-1.0| Medium — Element's deployments hit scale but at sustained ops cost| Low — Matrix.org Foundation governance is robust| Low — would re-tokenise on top of an opaque event-graph DB| ❌ Federation overkill for internal chat  
Phoenix Channels (Elixir / OTP)| Very high — build everything (channels, threads, attachments, mobile push) ground-up in Elixir| Low — would be greenfield code; no operational base| Low — pure-OSS Erlang/Elixir| High — Elixir's `:utf8` tokeniser is great; could use Vntk natively| ❌ Build cost >> P0 · exit window  
Build from scratch (Rust + WebSocket)| Astronomical — 6-P0 → P3 horizon minimum| None| Lowest possible| Highest possible (custom)| ❌ The "rebuild Slack" trap  
  
### What we own vs what Mattermost owns

Concern| Owned by Mattermost fork| Owned by CyberOS  
---|---|---  
Channel + thread + message + attachment + reaction primitives| ✅ unchanged| —  
WebSocket transport + Redis presence fanout| ✅ unchanged| —  
Mobile push (FCM / APNs)| ✅ unchanged| —  
User identity database| ❌ removed (we trust AUTH JWTs)| ✅ AUTH module's tenant + subject tables  
Search engine| ❌ swapped out| ✅ PGroonga + TinySegmenter  
Desktop / mobile UI shell| ❌ replaced| ✅ Tauri + CyberOS design system (Liquid Glass, Be Vietnam Pro, Umber/Ochre)  
AI features (@lumi, /summarise, smart-reply, daily digest)| —| ✅ via AI Gateway + CUO router  
memory capture pipeline| —| ✅ Postgres logical replication slot → memory Writer  
Voice ASR| —| ✅ Whisper-large-v3 self-hosted  
Slack / Zalo import| Slack import exists; ❌ rewired to AUTH-managed subjects| ✅ Zalo import is custom (no SDK)  
Admin REST surface| ❌ replaced| ✅ unified CyberOS admin REST via AUTH-issued admin JWT  
Compliance export| Partial — Mattermost format| ✅ extended with CyberOS audit-chain anchor  
  
**Fork governance:** Mattermost upstream tracking via quarterly rebase + security-patch cherry-pick (handled by CSEC). Fork pinned at `cyberos/chat-server` with explicit upstream commit hash in `VERSION` file. Per R-CHAT-013, license drift is monitored quarterly; if upstream re-licenses (e.g. to SSPL or AGPL), we have a documented downstream-only path. 

2.6

## @lumi → memory capture — chat as a memory surface

Per [MEMORY_AUTOSYNC_DESIGN.md §5](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>), CHAT is one of three first-class capture surfaces (Cowork · Claude Code · CHAT). The capture rule is conservative-by-default: **only`@lumi`-tagged messages auto-capture**. Passive messages are NOT captured (privacy floor) unless the channel has explicit opt-in sampling for high-signal flows. 

### Capture rules

Trigger| Captures?| memory target| sync_class default| Owner consent  
---|---|---|---|---  
Message contains `@lumi` (or `@genie`)| ✅ yes — primary capture trigger| Author's Personal memory| Channel default (shared chan: `shareable`; DM: `personal`)| Per-tenant onboarding consent at signup  
Message in `#decisions` / `#rfcs` / similar opted-in channels| ✅ yes — sampling at every Nth msg (default N=1, every msg)| Author's Personal memory| `shareable`| Tenant-admin opts the channel in  
Message in any other channel (no `@lumi`)| ❌ no — privacy floor| —| —| —  
Direct message (DM)| ❌ no unless `@lumi` mentioned| If `@lumi` → Author's Personal memory| `personal`| Both parties' consent for capture  
Voice message ASR transcript| Inherits trigger from message text| Same as text trigger| Same as text trigger| Same  
File attachment| ❌ no auto-capture — files are referenced, not embodied; the message itself may capture per above rules| —| —| —  
  
### @lumi handling flow (message → CUO route → memory memory row)

sequenceDiagram autonumber participant U as User (CHAT client) participant CHAT as CHAT service  
(Mattermost fork) participant LP as @lumi parser  
(CHAT plugin) participant CUO as CUO router participant AI as AI Gateway participant BR as memory Writer  
(Personal) participant L as Lumi's memory  
(cloud · pull queue) U->>CHAT: "Hey @lumi, summarise this thread for me" CHAT->>CHAT: persist message (WebSocket fanout) CHAT->>LP: hook(message_create) · text contains @lumi LP->>BR: put(memories/discussions/chat-<chan>-<msg>.md, sync_class=channel_default) BR-->>LP: chain_hash · audit row appended LP->>CUO: route(message_body, channel_context, user_id) CUO->>CUO: rule-based scoring (Phase 1) · pick skill alt high-confidence rule match CUO->>AI: invoke skill via AI Gateway with persona stamp AI-->>CUO: response · persona-version stamped else low confidence CUO->>AI: LLM cascade (Phase 2 — gated by AI Gateway budget) AI-->>CUO: response end CUO->>BR: put(memories/decisions/cuo-response-<id>.md) · the response is itself memory BR-->>CUO: chain_hash CUO->>CHAT: post_reply(channel, thread_id, response, persona_version) CHAT->>U: real-time response via WebSocket BR-->>L: sync orchestrator next window: push shareable rows to Lumi's memory L-->>BR: confirmed · lumi_chain_hash recorded 

**What the audit row captures:** the original message body, the user_id, the channel id (subject to ACL), the thread id, the timestamp, the `@lumi` trigger flag, the resolved CUO skill, the persona-version stamp, the AI Gateway latency + token cost, the memory chain hash, and the channel's sync_class default. Every `@lumi` response is fully replayable from memory — "what did the CFO skill tell us about runway last Tuesday?" → grep audit chain. 

2.7

## E2EE decision — server-visible by design (locked)

CHAT does NOT use end-to-end encryption. Messages are server-visible to the tenant's CHAT service. Encryption-at-rest is provided at the Postgres column level (KMS-wrapped keys per tenant). This is a deliberate trade — not an accident — and documenting it here heads off the recurring debate. 

### The decision-record

Capability requirement| E2EE (Signal-style)| Per-tenant Postgres encryption-at-rest (chosen)  
---|---|---  
@lumi can read messages| ❌ no — would defeat the purpose| ✅ yes — AI features work  
PGroonga full-text search| ❌ no — server can't index ciphertext| ✅ yes  
memory capture from @lumi messages| ❌ no — server can't read| ✅ yes  
Compliance export (PDPL Art. 14 DSAR)| Possible but complex — keys held by user; server can't decrypt for regulator without user signoff| ✅ straightforward — tenant admin exports  
Key escrow on employee offboarding| Hard — Signal-style keys are tied to devices| ✅ trivial — KMS key escrow  
Threat model: rogue server admin| ✅ mitigated| ❌ NOT mitigated — server admin can decrypt  
Threat model: external attacker with DB dump only| ✅ mitigated| ✅ mitigated — column ciphertext, KMS-wrapped keys  
Threat model: physical disk loss| ✅ mitigated| ✅ mitigated  
Threat model: backup compromise| ✅ mitigated| ✅ mitigated — backups use separate KMS key  
Onboarding UX (new device key exchange)| Hard — Signal protocol or WebCrypto + Web Push| ✅ trivial — login + JWT  
  
### Why per-tenant Postgres encryption wins

  1. The platform's value proposition _requires_ server-side message access — @lumi, search, memory capture, daily digest. E2EE would break all four.
  2. The rogue-server-admin threat is real but manageable: SOC 2 access controls + audit-chain on every admin operation + four-eyes principle for production database access. Risk is procedural, not cryptographic.
  3. The other four common threat models (external attacker with DB dump, physical disk loss, backup compromise, snapshot theft) ARE mitigated by per-tenant Postgres encryption-at-rest — same security posture as E2EE for these.
  4. Slack, Notion, Linear, Carta, every B2B platform with AI features makes the same call. The market expectation is "encrypted-at-rest, not E2EE."
  5. Re-evaluating this decision is a P3+ exercise — only if a specific high-security tenant requests E2EE as a contract requirement. At which point a separate "E2EE-only channel" mode could be offered with explicit @lumi-disabled limitation.



### What encryption-at-rest looks like concretely

Per-tenant KMS-wrapped column encryption on `messages.body`, `messages.attachments_meta`, `direct_messages.body`. Each tenant has a dedicated AWS KMS Customer Managed Key. CHAT service holds the data key in memory only (re-fetched on rotation). DB dump alone yields ciphertext. Backups use a separate KMS key with separate rotation cadence. Per-tenant KMS revocation is supported (within 15 minutes a tenant's data becomes unreadable system-wide). 

2.8

## Slack / Zalo migration — the import tooling

Slack and Zalo migration is non-negotiable for the P0 · exit dogfood gate. If the team can't bring their 3-year Slack history with them, they won't move. The tooling spec below targets a one-day cutover for the 10-Member CyberSkill team and a one-week cutover for an external 100-Member tenant. 

### Slack import tool — `cyberos-chat import slack`

Step| What it does| Tooling| Time (10-Member team)  
---|---|---|---  
1\. Export| Slack admin exports workspace data (Standard Export or Corporate Export depending on plan)| Slack admin web UI · ~30 min wait for Slack to bundle| 30 min  
2\. Validate| Run `cyberos-chat import slack --validate ./slack-export.zip` — checks user-mapping completeness, channel structure, attachment integrity| CHAT CLI| 2 min  
3\. User map| Map Slack user IDs to AUTH subjects. Auto-match on email; manual review for ambiguities| Interactive CLI| 5 min  
4\. Dry-run| Generate the would-be import plan: channels created, messages mapped, attachments uploaded| CHAT CLI · prints summary| 2 min  
5\. Import| Execute. Channels created. Messages inserted with original timestamps + original-Slack-msg-ID preserved in `messages.metadata.slack_origin_id`. Attachments uploaded to MinIO/S3. Reactions, threads, pinned messages all preserved.| CHAT CLI · parallelised| ~15 min for 3-year history  
6\. PGroonga index| Trigger PGroonga reindex on imported content (Vietnamese-tokenised)| auto-triggered| ~5 min for 100k messages  
7\. memory sample-capture| For channels marked high-signal (decisions, rfcs), retroactively emit memory rows. **Default OFF** — opt-in only.| CHAT CLI `--retro-capture` flag| variable  
8\. Switch| Tenant admin posts in #general: "Slack is read-only from today; new messages in CyberOS CHAT only." Cutover.| Manual| 0 min  
  
### Zalo import — the harder problem

Zalo has NO official enterprise export API. Two pragmatic paths: 

  1. **Manual per-conversation export** via Zalo's "Save chat" feature. Each user runs the export against their conversations and uploads to a CHAT staging bucket. CHAT then imports via `cyberos-chat import zalo --personal ./exports/`. Awkward but works. ~1 hr per user.
  2. **Read-from-Zalo desktop client** via OS-level integration (macOS Accessibility API) — captures the on-screen chat list in batches. Higher infrastructure cost; lower per-user time. Requires `cyberos-zalo-bridge` companion (~Q3 2026 if priority).



The 10-Member CyberSkill team can do Zalo migration manually in one afternoon. For external tenants at 50+ members, path 2 is required — adds 4 weeks to TEN module's onboarding spec. 

2.9

## Decommission KPI — the only metric that matters at P0 exit

P0 exit (P0 · exit) is gated on Slack + Zalo decommissioned. That is not a vibes assertion; it is a measurable KPI. The definition below is what gets evaluated at the P0 exit gate review. 

### Definition
    
    
    decommission_signal := (
      messages_per_day_in_cyberos_chat
      /
      (messages_per_day_in_cyberos_chat + messages_per_day_in_slack + messages_per_day_in_zalo)
    ) ≥ 0.95
    
    over a rolling 14-day window, sampled daily.
    
    Plus:
    - Slack workspace set to read-only (admin signoff)
    - Zalo company group: announced sunset (manual confirmation)
    - No team member has logged into Slack in the prior 7 days
    

### Why 95%, not 100%

100% is unrealistic — the team will receive Slack DMs from external partners using old habits for months. The decommission gate is about _where new conversation lands by default_ , not about achieving zero Slack presence ever again. The "no team member has logged into Slack in the prior 7 days" qualifier handles the case where the messages-per-day ratio passes but a few stragglers are still using Slack as their primary surface. 

### Tracking instrumentation

Source| What we instrument| How  
---|---|---  
CHAT (CyberOS)| messages_per_day count, by-user breakdown| OBS metric · `cyberos_chat_message_count`  
Slack| messages_per_day, login_per_user_per_day| Slack analytics export · weekly cron pull  
Zalo| self-reported per-team-member weekly| 1-question Cowork survey at end of every Friday standup  
  
### What happens if P0 · exit misses the gate

Per `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`), if the P0 · exit dogfood gate misses, the P1 · start P1-scope-collapse fail-mode immediately activates: HR-full split + LEARN deferred. The platform doesn't fail; the timeline slips by 1 quarter while CHAT's missing features get prioritised. Concrete remediation triggers: if `decommission_signal < 0.50` at P0 · exit → 90-day CHAT-only sprint, no P1 modules start. If `0.50 ≤ signal < 0.95` at P0 · exit → 30-day catch-up, P1 modules start at P1 · start with reduced scope. 

3

## Architecture

Three layers stacked. **Substrate:** Mattermost-derived Go service (channels, threads, attachments, presence, push). **Native CyberOS layer:** auth bridge, UI shell (Tauri), PGroonga search adapter, memory bridge, voice ASR. **AI layer:** Genie agent, summarisation, smart-reply, daily digest — all calling AI Gateway. 

graph TB subgraph CLIENTS ["Clients (Tauri shell)"] DT["Desktop (Tauri)"] MOB["Mobile (Tauri / WebView)"] WEB["Browser SPA"] end subgraph CHAT ["CHAT service (Go · Mattermost fork)"] WS["websocket.go  
hot push"] API["api.go  
HTTP + GraphQL"] CHAN["channels.go  
public · private · DM"] THR["threads.go  
reply chains"] MSG["messages.go  
create · update · delete"] PRES["presence.go  
online/away/dnd"] NOTIF["notifications.go  
FCM + APNs fanout"] end subgraph CYBER ["CyberOS native plug-ins"] AUTHB["auth_bridge.go  
JWT verify + RBAC.Check"] SEARCH["search.go  
PGroonga adapter · VN tokenise"] MEMORYB["memory_bridge.go  
logical replication tail"] IMP["importers/  
slack · zalo"] ASR["asr_client.go  
Whisper gRPC"] YJS["yjs_sync.go  
CRDT draft sync"] end subgraph AI ["AI features"] GENIE["genie.go  
@genie mention handler"] SUMM["summarise.go  
/summarise thread"] SREPLY["smart_reply.go  
3 suggestions / mention"] DIG["digest.go  
daily channel digest"] end subgraph STORES PG[("PostgreSQL 16  
\+ PGroonga  
messages · channels  
RLS by tenant_id")] REDIS[("Redis 7  
presence + push fanout")] MINIO[("MinIO (S3-compatible)  
attachments")] end subgraph EXT ["External CyberOS services"] AUTH["🔐 AUTH"] AIGW["🧠 AI Gateway"] memory["🧠 memory"] OBS["👁 OBS"] GPU["Whisper GPU pod"] end DT --> WS DT --> API MOB --> WS MOB --> API WEB --> WS WEB --> API API --> AUTHB WS --> AUTHB AUTHB --> AUTH API --> CHAN API --> THR API --> MSG MSG --> PG CHAN --> PG THR --> PG WS --> PRES PRES --> REDIS MSG --> NOTIF NOTIF --> REDIS API --> SEARCH SEARCH --> PG MSG --> MEMORYB MEMORYB --> memory MSG --> GENIE THR --> SUMM MSG --> SREPLY GENIE --> AIGW SUMM --> AIGW SREPLY --> AIGW DIG --> AIGW API --> IMP WS --> YJS YJS --> PG MSG --> ASR ASR --> GPU CHAT --> OBS classDef planned fill:#ecfdf5,stroke:#45210e classDef store fill:#f5f3ff,stroke:#7c3aed classDef ext fill:#fef6e0,stroke:#9c750a class WS,API,CHAN,THR,MSG,PRES,NOTIF,AUTHB,SEARCH,MEMORYB,IMP,ASR,YJS,GENIE,SUMM,SREPLY,DIG,DT,MOB,WEB planned class PG,REDIS,MINIO store class AUTH,AIGW,memory,OBS,GPU ext 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`websocket.go`| services/chat/server/websocket.go| Persistent WS connection per client. Routes incoming events (typing, presence) and pushes outgoing (new message, mention, reaction).  
`api.go`| services/chat/server/api.go| HTTP REST + GraphQL endpoints. OpenAPI documented; deprecation policy quarterly.  
`channels.go`| services/chat/server/channels.go| CRUD on channels. Per-channel ACL: public, private (member-of), DM (1:1), group-DM (≤ 8).  
`threads.go`| services/chat/server/threads.go| Reply chains. Notifies thread followers; surfaces in unread tray.  
`messages.go`| services/chat/server/messages.go| Send · edit · delete · pin · react. Mention parsing (@user, #channel, @genie).  
`presence.go`| services/chat/server/presence.go| Online · away · DND. Redis-backed; TTL-refresh from WS heartbeats.  
`notifications.go`| services/chat/server/notifications.go| Fanout: in-app, FCM (Android), APNs (iOS), email digest.  
`auth_bridge.go`| services/chat/native/auth_bridge.go| Replaces Mattermost native auth. Verifies CyberOS JWT; resolves subject; calls AUTH RBAC for channel ACLs.  
`search.go`| services/chat/native/search.go| PGroonga adapter. VN-tokeniser config; recall measured against test set.  
`memory_bridge.go`| services/chat/native/memory_bridge.go| Tails Postgres logical-replication slot. Streams new messages into memory Layer 3 corpus within p95 ≤ 5 s.  
`importers/slack.go`| services/chat/native/importers/slack.go| Slack Workspace export → CyberOS channels + messages. Preserves threads, attachments, reactions.  
`importers/zalo.go`| services/chat/native/importers/zalo.go| Zalo OA API import (where API allows). Mapping limited by Zalo API surface.  
`asr_client.go`| services/chat/native/asr_client.go| Voice-message ASR via Whisper-large-v3 (self-hosted L4 GPU pod). VN + EN.  
`yjs_sync.go`| services/chat/native/yjs_sync.go| Yjs CRDT-based offline draft sync. Draft conflicts resolved per CRDT rules.  
`genie.go`| services/chat/ai/genie.go| @genie mention handler. Builds context from thread + channel; calls AI Gateway; posts answer with citations.  
`summarise.go`| services/chat/ai/summarise.go| `/summarise` slash command. Thread → summary with TL;DR and action items.  
`smart_reply.go`| services/chat/ai/smart_reply.go| On every @-mention to a user, generate 3 reply suggestions (latency p95 ≤ 1.4 s, (FR pending)).  
`digest.go`| services/chat/ai/digest.go| Daily channel digest ((FR pending)) for members in "Notify" mode.  
`tauri_shell/`| services/chat/tauri/| Tauri Rust shell. Desktop (macOS · Windows · Linux) + mobile (iOS · Android via Tauri 2.0 mobile).  
  
4

## Data model

Postgres-backed. Schema follows the Mattermost shape with CyberOS-specific extensions for audit linkage and memory ingest watermarks. All tables RLS-scoped by tenant_id. 

erDiagram TENANT ||--o{ CHANNEL: "owns" TENANT ||--o{ USER_REF: "synced from AUTH" USER_REF ||--o{ MEMBERSHIP: "joins" CHANNEL ||--o{ MEMBERSHIP: "has" CHANNEL ||--o{ MESSAGE: "contains" MESSAGE ||--o{ REACTION: "receives" MESSAGE ||--o{ MENTION: "carries" MESSAGE ||--o{ ATTACHMENT: "has" MESSAGE ||--o| AUDIT_LINK: "links to memory row" CHANNEL ||--o{ ACL_RULE: "scoped by" USER_REF ||--o{ PRESENCE: "current" IMPORTED_BUNDLE ||--o{ MESSAGE: "imported from" CHANNEL { uuid id PK uuid tenant_id FK string slug string kind "public or private or direct or group_direct" string display_name string purpose timestamp created_at string memory_corpus_slot "memories-chat-channel path" } USER_REF { uuid id PK uuid auth_subject_id FK uuid tenant_id FK string display_name string locale "vi-VN or en-US or other" } MEMBERSHIP { uuid channel_id FK uuid user_id FK string role "owner or admin or member or guest" timestamp joined_at timestamp last_read_at string notify_mode "all or mention or quiet or digest" } MESSAGE { uuid id PK uuid channel_id FK uuid parent_id "FK to MESSAGE thread root - self-reference" uuid author_id FK string body obj blocks "rich content" timestamp created_at timestamp edited_at timestamp deleted_at bigint memory_ingest_seq string memory_chain string source "native or slack_import or zalo_import" } REACTION { uuid id PK uuid message_id FK uuid user_id FK string emoji timestamp ts } MENTION { uuid id PK uuid message_id FK string target "user-uuid or channel-uuid or genie" } ATTACHMENT { uuid id PK uuid message_id FK string filename string content_type bigint size_bytes string s3_key string transcript "voice note ASR" } ACL_RULE { uuid id PK uuid channel_id FK string scope "role or user" string code "founder or member or other" string action "read or write or invite or archive" } PRESENCE { uuid user_id PK string status "online or away or dnd" timestamp last_seen_at } AUDIT_LINK { uuid message_id FK bigint memory_seq string memory_chain } IMPORTED_BUNDLE { uuid id PK string source "slack or zalo" timestamp imported_at int message_count obj manifest } 

### Channel kinds + ACL rules

Kind| Visibility| Default ACL| memory slot  
---|---|---|---  
public| All members of tenant| everyone read · channel members write| company:  
private| Member-of only| members read+write; admin manages| module:chat:private:  
direct (DM)| 1:1| both parties read+write| member::dm:  
group_direct| ≤ 8 members| all members read+write| module:chat:group:  
  
5

## API surface

### GraphQL subgraph (federated)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type Channel @key(fields: "id") {
     id: ID!
     slug: String!
     kind: ChannelKind!
     displayName: String!
     purpose: String
     members: [Membership!]! @requiresScopes(scopes: [["chat.read"]])
     recentMessages(limit: Int = 50): [Message!]! @requiresScopes(scopes: [["chat.read"]])
    }
    
    type Message @key(fields: "id") {
     id: ID!
     channelId: ID!
     authorId: ID!
     body: String!
     blocks: JSON
     parentId: ID
     createdAt: DateTime!
     editedAt: DateTime
     reactions: [Reaction!]!
     attachments: [Attachment!]!
     memoryChain: String # links to memory audit row
    }
    
    type Reaction { emoji: String! userId: ID! ts: DateTime! }
    
    type Attachment {
     id: ID!
     filename: String!
     contentType: String!
     sizeBytes: Int!
     url: String!
     transcript: String # voice notes only
    }
    
    type Membership {
     user: User!
     role: MembershipRole!
     notifyMode: NotifyMode!
     lastReadAt: DateTime
    }
    
    enum ChannelKind { PUBLIC PRIVATE DIRECT GROUP_DIRECT }
    enum MembershipRole { OWNER ADMIN MEMBER GUEST }
    enum NotifyMode { ALL MENTION QUIET DIGEST }
    
    type Query {
     channel(id: ID!): Channel
     channels(kind: ChannelKind): [Channel!]!
     searchMessages(q: String!, channelId: ID, limit: Int = 50): [Message!]!
    }
    
    type Mutation {
     sendMessage(channelId: ID!, body: String!, blocks: JSON, parentId: ID): Message!
     editMessage(id: ID!, body: String!): Message!
     deleteMessage(id: ID!): Boolean!
     addReaction(messageId: ID!, emoji: String!): Reaction!
     createChannel(slug: String!, kind: ChannelKind!, displayName: String!): Channel!
     @requiresScopes(scopes: [["chat.channel_create"]])
     inviteToChannel(channelId: ID!, userIds: [ID!]!): Boolean!
     @requiresScopes(scopes: [["chat.channel_invite"]])
     summariseThread(threadId: ID!): String! # calls AI Gateway
    }
    
    type Subscription {
     channelMessages(channelId: ID!): Message!
     presenceChanges(userIds: [ID!]!): Presence!
    }

### WebSocket events

Event| Direction| Payload  
---|---|---  
`message.created`| server → client| full Message  
`message.edited`| server → client| Message diff  
`message.deleted`| server → client| {id, channel_id}  
`reaction.added`| server → client| {message_id, emoji, user_id}  
`typing.started`| bidirectional| {channel_id, user_id}  
`presence.changed`| server → client| {user_id, status}  
`thread.updated`| server → client| {parent_id, reply_count}  
`smart_reply.suggested`| server → client| {message_id, suggestions:[…]}  
  
### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.chat.send_message`| channel_id, body, blocks?| {message_id, ts}| destructive=false · scope=chat.write  
`cyberos.chat.search_messages`| q, channel_id?, limit?| Message| readOnly=true · scope=chat.read  
`cyberos.chat.summarise_thread`| thread_id| {summary, action_items}| readOnly=true · scope=chat.read  
`cyberos.chat.list_channels`| —| Channel| readOnly=true · scope=chat.read  
`cyberos.chat.create_channel`| slug, kind, name| Channel| destructive=false · scope=chat.channel_create  
`cyberos.chat.invite`| channel_id, user_ids| {ok}| destructive=false · scope=chat.channel_invite  
`cyberos.chat.export_dsar`| subject_id, since?| {zip_url}| destructive=false · scope=chat.dsar_export  
  
6

## Key flows

### Flow 1 — Send message (mention fanout + memory ingest)

sequenceDiagram autonumber participant U as User (desktop) participant WS as CHAT websocket participant API as CHAT api participant AUTH as 🔐 AUTH RBAC participant PG as Postgres + PGroonga participant NOT as notifications participant BB as memory_bridge participant BR as 🧠 memory participant FCM as FCM/APNs U->>WS: send message {channel, body, mentions:[@bao, @genie]} WS->>AUTH: RBAC.Check(chat.write, channel) AUTH-->>WS: allow WS->>API: persist API->>PG: INSERT message, mentions, etc. PG-->>API: row id API->>NOT: fanout {mentioned:[bao], followers:[…]} NOT->>WS: push to online users (channel.members) NOT->>FCM: push to offline users par memory ingest (async, p95 ≤ 5s) PG-->>BB: logical replication event BB->>BR: put memories/chat/<channel>/<msg>.md BR-->>BB: {seq, chain} BB->>PG: UPDATE memory_chain on message and Genie answer (if mention) NOT->>WS: trigger @genie handler end 

(FR pending): memory ingest p95 ≤ 5 s. Logical replication is the load-bearing piece — Postgres ships the WAL, the bridge transforms each row into a memory put.

### Flow 2 — Thread reply

sequenceDiagram autonumber participant U as User participant WS as websocket participant API as api participant PG as Postgres U->>WS: send {parent_id:<root>, body} WS->>API: persist as reply API->>PG: INSERT message WHERE parent_id=<root> PG-->>API: row + thread.reply_count++ API->>WS: thread.updated event to followers WS-->>U: ack with thread_id Note over WS: followers root + repliers get desktop badge plus unread tray entry 

### Flow 3 — Vietnamese search (PGroonga)

sequenceDiagram autonumber participant U as User participant API as api participant S as search.go (PGroonga adapter) participant PG as Postgres + PGroonga participant AUTH as RBAC U->>API: GET /api/v1/search?q="hợp đồng singapore" API->>AUTH: RBAC.Check(chat.read, *) AUTH-->>API: allow API->>S: search(q, user_scope) S->>S: tokenise VN ("hợp đồng singapore" → [hợp_đồng, singapore]) S->>PG: SELECT … WHERE body @@ tokens AND channel_id IN (membership) PG-->>S: matching rows (with snippet highlights) S-->>API: 23 hits API-->>U: results 

(FR pending): PGroonga VN tokeniser; recall ≥ 80% on the public VN test set. Membership filter applied first to enforce ACL.

### Flow 4 — Thread summarisation (`/summarise`)

sequenceDiagram autonumber participant U as User participant API as api participant SUM as summarise.go participant AI as 🧠 AI Gateway participant PG as Postgres U->>API: slash command "/summarise" API->>SUM: thread_id SUM->>PG: SELECT messages WHERE parent_id=<thread> ORDER BY created_at PG-->>SUM: 47 messages SUM->>SUM: build context, redact PII names SUM->>AI: ChatComplete(persona="genie", model=auto, messages=…) AI-->>SUM: summary + action items SUM->>API: render as inline reply API-->>U: "TL;DR: … Action items: …" 

(FR pending): thread summarisation p95 ≤ 3 s. Most cost in AI Gateway call; CHAT adds ~80 ms overhead.

### Flow 5 — Slack import

sequenceDiagram autonumber participant ADMIN as Tenant admin participant CLI as cyberos-chat import participant IMP as importers/slack.go participant PG as Postgres participant BB as memory_bridge participant BR as 🧠 memory ADMIN->>CLI: cyberos-chat import slack --bundle slack-export.zip CLI->>IMP: parse bundle IMP->>IMP: enumerate channels, users, messages Note over IMP: 142 channels · 8,420 users · 1.4M messages loop per channel IMP->>PG: INSERT channel IMP->>PG: INSERT messages (batched 500) PG-->>IMP: rows IMP->>BB: trigger backfill ingest BB->>BR: put memories/chat/<channel>/... end IMP-->>CLI: done CLI-->>ADMIN: report (channels, messages, attachments transferred) 

(FR pending): Slack import path; Zalo where API allows. Backfill memory ingest is throttled to avoid swamping the chain (max 1k ops/sec).

7

## Message lifecycle

A single message traverses up to seven states from draft to memory ingest. Edits + deletes are soft; the memory audit row remains. 

stateDiagram-v2 [*] --> Drafting: user typing (Yjs CRDT if offline) Drafting --> Sending: user hits enter / send button Sending --> Persisted: Postgres INSERT Persisted --> Fanned: WebSocket + FCM/APNs delivery Fanned --> Acknowledged: clients render Persisted --> MEMORYIngested: logical replication → memory put Acknowledged --> Edited: user edits Edited --> Persisted: UPDATE on message Acknowledged --> Reacted: emoji added Acknowledged --> Threaded: reply created (separate message) Acknowledged --> Deleted: soft-delete (deleted_at set) Deleted --> Purged: DSAR purge or retention expiry MEMORYIngested --> [*] Purged --> [*] 

### Notify-mode behaviour

Mode| When notified| Channel  
---|---|---  
`all`| every new message| desktop + mobile push  
`mention`| @self or @channel or @here| desktop + mobile push  
`quiet`| only @self| desktop only  
`digest`| daily digest summary| email + in-app inbox  
  
8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| CHAT message-deliver p95| ≤ 200 ms| WebSocket round-trip · k6  
`N(FR pending)`| CHAT availability (28-day)| ≥ 99.9%| SLO  
`N(FR pending)`| Thread summarisation p95| ≤ 3 s| (FR pending) test  
`N(FR pending)`| Smart-reply suggestion p95| ≤ 1.4 s| (FR pending)  
`N(FR pending)`| VN search query p95| ≤ 250 ms over 1M messages| k6 search test  
`N(FR pending)`| memory ingest end-to-end p95| ≤ 5 s| (FR pending)  
`N(FR pending)`| VN search recall on test set| ≥ 80%| (FR pending)  
`N(FR pending)`| VN search precision on test set| ≥ 85%| (FR pending) test  
`N(FR pending)`| Concurrent WS connections (P0)| ≥ 5,000| load test  
`N(FR pending)`| Messages / second (P0)| ≥ 500| k6 sustained  
`N(FR pending)`| DSAR export completeness| 100% messages + reactions + attachments| fixture test  
`N(FR pending)`| Message → memory audit link rate| 100% (no dropped messages)| continuous reconciliation  
`N(FR pending)`| P0 infra cost| ≤ $45/month| OBS cost dashboard  
  
10

## Dependencies

graph LR subgraph upstream ["CHAT depends on"] AUTH["🔐 AUTH  
JWT + RBAC  
\+ user sync"] memory["🧠 memory  
message corpus  
(Layer 3)"] AIGW["🧠 AI Gateway  
genie · summarise · smart-reply"] OBS["👁 OBS  
traces + metrics"] PG["🗄 Postgres  
\+ PGroonga"] REDIS["⚡ Redis  
presence + push"] S3["☁️ MinIO/S3  
attachments"] GPU["Whisper GPU  
voice ASR"] end CHAT["💬 CHAT"] subgraph consumers ["CHAT used by"] CUO["🎯 CUO digest"] SK["🛠 Skill (bot mentions)"] OBS2["👁 OBS alert bot"] USERS["End users"] end AUTH --> CHAT memory --> CHAT AIGW --> CHAT OBS --> CHAT PG --> CHAT REDIS --> CHAT S3 --> CHAT GPU --> CHAT CHAT --> CUO CHAT --> SK CHAT --> OBS2 CHAT --> USERS classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#ecfdf5,stroke:#45210e class memory,SK shipped class CHAT,AUTH,AIGW,OBS,CUO,OBS2,USERS planned class PG,REDIS,S3,GPU planned 

11

## Compliance scope

Regulation / standard| Article / clause| CHAT feature  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| Per-subject message export via `cyberos-chat export-dsar`.  
Vietnam PDPL| Art. 16 — Erasure| Soft-delete + memory audit-row purge; chain row preserved.  
Vietnam Decree 13/2023| Art. 17 — Processing log| Every message → memory row; chain provides processing log.  
Vietnam Decree 53/2022| Art. 26 — Data residency| Per-tenant residency tag honoured by RDS region selection.  
GDPR| Art. 17 — Right to erasure| DSAR purge; memory audit fact remains.  
GDPR| Art. 32 — Security of processing| TLS in transit, AES-256-GCM at rest, per-channel ACL.  
EU AI Act| Art. 12 — Logging| Genie answers + summaries land in memory with persona-version.  
EU AI Act| Art. 13 — Transparency| @genie answers carry "AI-generated" label + citations.  
ISO/IEC 27001:2022| A.5.30 — ICT readiness| memory bridge provides off-cluster message replica.  
ISO/IEC 27001:2022| A.8.5 — Secure authentication| AUTH service handles all logins; no Mattermost native auth.  
SOC 2 Type II| CC6.1 — Logical access| Per-channel ACL via AUTH RBAC.  
SOC 2 Type II| CC7.2 — Monitoring| OBS SLO dashboards for CHAT availability + latency.  
  
12

## Risk entries

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-CHAT-001`| Mattermost upstream license change breaks fork| Low| High| CPO| Fork at known-MIT/Apache version; back-port security patches only; ongoing legal review.  
`R-CHAT-002`| memory ingest backlog → messages "missing" from search| Medium| Medium| CTO| SLO: ingest p95 ≤ 5 s. Alerting on backlog > 60 s. Logical replication slot monitoring.  
`R-CHAT-003`| VN tokeniser regression below 80% recall| Medium| Medium| CPO| CI test on public VN test set; PGroonga config under version control; quarterly review.  
`R-CHAT-004`| Cross-channel message leak via crafted GraphQL query| Low| High| CSO| RBAC enforced at API; cross-channel property-based test in CI.  
`R-CHAT-005`| @genie injection — user hides instructions in message body| Medium| Medium| CSO| System prompt at AI Gateway level (not in CHAT); content-safety filter; CaMeL enforcement.  
`R-CHAT-006`| Voice-message ASR transcript inaccurate → wrong context for @genie| Medium| Low| CPO| Whisper-large-v3 self-hosted; user can edit transcript pre-send.  
`R-CHAT-007`| Slack import drops threads / reactions| Medium| Low| CPO| Importer test fixtures cover thread + reaction + attachment paths; manifest validates before write.  
`R-CHAT-008`| WebSocket connection limit hit at scale| Medium| Medium| CTO| P0 budget: 5k concurrent. Horizontal scale-out via Redis pub-sub for fanout at P1+.  
`R-CHAT-009`| Mobile draft sync conflict produces double-post| Low| Medium| CPO| Yjs CRDT semantics + send-once token on submit.  
`R-CHAT-010`| Compliance export omits CyberOS audit anchor| Low| Medium| CLO| Export test verifies memory chain hashes for every message in bundle.  
`R-CHAT-011`| **Dogfooding never actually happens** — team keeps falling back to Slack/Zalo past P0 · exit because CHAT is "almost there"| High| Critical| CEO| Hard `decommission_signal ≥ 0.95` gate at P0 · exit; miss-the-gate = 2-week sprint freeze on net-new modules, focus only on CHAT polish + migration tooling (§2.9 remediation tier 1). If still < 0.85 by P1 · start, escalate to platform-thesis review.  
`R-CHAT-012`| Enterprise tenant demands E2EE → forces design pivot mid-build| Medium| High| CSO| §2.7 decision document — present per-tenant Postgres encryption-at-rest + signed audit chain as compliance equivalent for SOC 2 / ISO 27001 / PDPL. If specific tenant requires E2EE, offer optional per-channel client-side encryption plugin at P3 (search disabled on those channels).  
`R-CHAT-013`| Voice ASR leaks PII into memory before redaction policy applied| Medium| High| DPO| ASR pipeline runs through AI Gateway PII scrubber before memory ingest; tested with VN-specific PII set (CCCD, phone, NĐD). Voice OFF by default at P1; tenant opt-in via Lumi setting.  
`R-CHAT-014`| Mattermost upstream license drift escalation (MIT/Apache → BSL or AGPL)| Low| High| CLO| Fork at pinned LTS commit (v9.x last MIT/Apache); legal monitor on Mattermost GitHub LICENSE file via watcher. If drift detected, freeze upstream merges and back-port security only. Long-term fallback: pivot to Synapse (Matrix) bridge at P4.  
`R-CHAT-015`| @lumi mention rate-limit abuse → AI Gateway cost overrun| Medium| Medium| CTO| Per-user @lumi rate limit (≤ 30/hour default) at AI Gateway. Per-tenant monthly @lumi budget surfaced in CUO dashboard. Burst @lumi → fast-fail with friendly "Lumi đang nghỉ ngơi, thử lại sau 1 phút".  
`R-CHAT-016`| Cross-tenant message leak via PGroonga search index corruption| Low| Critical| CSO| Tenant_id is mandatory shard key on all PGroonga indexes; cross-tenant property-based test in CI; quarterly red-team query test against staging multi-tenant fixture. RLS (row-level security) enforced at Postgres role layer.  
`R-CHAT-017`| Slack import partial failure leaves CHAT inconsistent (some threads imported, some not)| Medium| Medium| CPO| Importer is idempotent + checkpointed (resume from last `channel_id × msg_id`). Pre-import dry-run report. On failure, partial state is tagged `state=importing` and hidden from search until reconciled.  
`R-CHAT-018`| Retro-capture (@lumi remember the last 10 messages) breaks privacy floor when applied to non-@lumi history| Medium| High| DPO| Retro-capture is per-message opt-in (UI dialog), never automatic. Audit row records the explicit user action + scope (e.g. `retro_capture_count=10`). Cannot retro-capture other users' messages without their consent (cross-author consent prompt).  
`R-CHAT-019`| Mobile push notification carries message body → leaks via locked-screen preview| Medium| Medium| CSO| Push payload = title + sender only ("Stephen sent you a message"); body fetched after device unlock. Configurable per-user (default = privacy-on).  
`R-CHAT-020`| VN tokeniser cannot disambiguate code-switched VN/EN messages → search misses| Medium| Low| CDO| Dual-pass indexing: TinySegmenter for VN tokens + ICU for EN tokens, both into the same PGroonga index. CI test includes 200 code-switched fixtures from real Slack export.  
  
13

## KPIs

KPI| Formula| Source| Target  
---|---|---|---  
**Send p95 latency**|  histogram| OBS| ≤ 200 ms  
**Availability (28d)**|  1 − error_minutes / total| OBS SLO| ≥ 99.9%  
**memory ingest p95**|  histogram| memory_bridge metrics| ≤ 5 s  
**VN search recall**|  TP / (TP + FN)| CI test set| ≥ 80%  
**VN search query p95**|  histogram| OBS| ≤ 250 ms  
**@genie usage (DAU)**|  distinct subjects / day| chat events| tracked; ≥ 60% of users  
**Concurrent WS connections (peak)**|  gauge| OBS| ≤ N(FR pending) (5k P0)  
**Message volume / day (tenant)**|  count| OBS| tracked for capacity planning  
**DSAR fulfilment time**|  request → export delivered| request tracker| ≤ 24 h  
**decommission_signal (P0-exit gate)**|  msgs_in_chat / (msgs_in_chat + msgs_in_slack + msgs_in_zalo), 14-day rolling| OBS + Slack export ETL + Zalo manual log| **≥ 0.95 by P0 · exit** ; miss → §2.9 tier-1 remediation  
**@lumi capture-rate**|  @lumi-mentions stored as memory rows / total @lumi-mentions| chat events + memory audit| ≥ 0.999 (loss budget: 1 in 1000)  
**@lumi response p95**|  histogram, @lumi mention → first response token| OBS span tags| ≤ 4 s (CUO-mediated answer)  
**VN tokeniser recall (continuous)**|  TP / (TP + FN) on production-grade 1,000-fixture set, weekly run| CI nightly + weekly| ≥ 0.80 (alert < 0.78)  
**memory-ingest backlog (max)**|  max(replication_slot_lag_seconds) over 24h| memory_bridge metrics| ≤ 60 s; page on > 300 s  
**Retro-capture opt-in rate**|  retro_capture dialogs accepted / shown| chat events| tracked; expect 30–60%  
**Mobile push delivery rate**|  delivered notifications / sent (excluding user-disabled)| APNs + FCM webhooks| ≥ 0.98 on a 7-day rolling window  
**Cross-tenant query reject rate**|  RLS rejections / total queries| Postgres audit log| tracked; spike = security alert  
**Dogfooding intensity (internal-only KPI, P0 · start..P0 · exit)**|  distinct internal users posting ≥ 5 msgs/day in CHAT| chat events filtered to `tenant_id=org:cyberskill`| P0-gate: 100% of full-time team by P0 · slice 2  
  
14

## RACI matrix

Activity| CEO| CPO| CTO| CDO| CSO| DPO  
---|---|---|---|---|---|---  
Product spec + UX| A| R| C| I| I| I  
Implementation (Go fork + plugins)| I| A| R| I| I| I  
VN search tokeniser tuning| I| C| C| A/R| I| I  
memory bridge integration| I| C| R| A| I| I  
AI feature integration (genie, summarise, smart-reply)| A| R| C| C| I| I  
Slack/Zalo importers| I| A/R| C| I| I| I  
Compliance export (DSAR)| I| C| C| R| C| A  
Mobile app (Tauri)| I| A/R| C| I| I| I  
  
15

## Planned CLI surface

Operator CLI `cyberos-chat` for admin tasks. Members interact via the GUI shell.

### 1\. List channels
    
    
    $ cyberos-chat channels list --tenant acme
    
    SLUG KIND MEMBERS MESSAGES LAST_ACTIVITY
    general public 42 18,420 2026-05-14T07:21Z
    engineering public 18 9,420 2026-05-14T07:19Z
    hanoi-office public 12 3,210 2026-05-14T06:45Z
    ceo-stephen-vy direct 2 412 2026-05-14T05:33Z
    project-alpha private 8 2,847 2026-05-14T07:02Z

### 2\. Send a message via CLI (for automation)
    
    
    $ cyberos-chat send --channel engineering --body "Deploy v0.9.2 starting at 14:00 VN. Watch #cyberos-alerts."
    
    [sent] message_id=msg_01HZJ…XK · channel=engineering · ts=2026-05-14T07:22:08Z
    [memory] ingest queued · expected p95 ≤ 5 s

### 3\. Search Vietnamese content
    
    
    $ cyberos-chat search "hợp đồng Singapore HoldCo"
    
    [search] channel=engineering 3 hits recall_est=87%
    msg_01HZG… 2026-05-12 stephen@: "đang draft hợp đồng Singapore HoldCo, deadline T5"
    msg_01HZH… 2026-05-13 bao@: "đã review hợp đồng, comment đã gửi"
    msg_01HZI… 2026-05-14 stephen@: "Singapore HoldCo flip nếu ARR ≥ $1.5M"

### 4\. Import Slack workspace
    
    
    $ cyberos-chat import slack --bundle slack-export.zip --tenant acme
    
    [parse] 142 channels · 8,420 users · 1.4M messages · 32k attachments
    [map] user mapping: 8,420 → 8,420 (zero unmapped)
    [backfill] 142 channels created
    [ingest] 142,000 messages / hour (throttled to memory ingest budget)
    [done] elapsed 9h 47m · all messages backfilled to memory
    [verify] random sample: 100/100 round-trip ok

### 5\. DSAR export
    
    
    $ cyberos-chat export-dsar --subject acme-contact@acme.com --output dsar-chat.zip
    
    [export] subject: acme-contact@acme.com
    [export] channels: 14 (12 public, 2 DM)
    [export] messages: 4,217
    [export] attachments: 142 (1.4 GB total)
    [export] reactions: 1,032
    [export] memory_anchor: included (audit chain hashes per message)
    [written] dsar-chat.zip · 1.5 GB

### 6\. Summarise a thread (operator-side)
    
    
    $ cyberos-chat summarise-thread --id thr_01HZJ…XK
    
    TL;DR: The team is debating whether to ship feature-X with the experimental
     summariser or hold for v2 of the prompt. Stephen leans ship-it; Bao
     wants one more A/B test.
    Action items:
     • Bao: prepare A/B comparison by Wed 2026-05-15
     • Stephen: review prompt v2 draft
     • Both: align on go/no-go at the Friday standup

### 7\. Health + SLO
    
    
    $ cyberos-chat health
    
    availability_28d: 99.96% ✓ (target ≥ 99.9%)
    send_p95_ms: 142 ✓ (target ≤ 200)
    memory_ingest_p95_s: 3.2 ✓ (target ≤ 5)
    vn_search_recall: 0.87 ✓ (target ≥ 0.80)
    ws_connections_now: 1,124
    messages_today: 18,420

16

## Phase status & estimates

Status

Planned

P0 · design phase · P0 · slice 2

Est. LoC (CyberOS plugins)

~12,000

Go + Rust shell + Tauri

Mattermost fork base

v9.x LTS

MIT/Apache core

Planned tests

200+

incl. VN search recall fixtures

P0 monthly cost

~$45

Fargate + RDS small + Redis + MinIO

CLI commands

~20 planned

`cyberos-chat`

Capability| Status  
---|---  
Mattermost fork + auth_bridge to AUTH| planned · P0  
Channels (public, private, DM, group-DM)| planned · P0  
Threads, reactions, attachments| planned · P0  
PGroonga VN search| planned · P0  
memory bridge (logical replication)| planned · P0  
@genie inline mention| planned · P0  
/summarise thread command| planned · P0  
Slack import| planned · P0  
Tauri desktop shell (mac · win · linux)| planned · P0  
DSAR export| planned · P0  
Smart-reply suggestions| planned · P1  
Daily digest (notify=digest mode)| planned · P1  
Voice messages + Whisper ASR| planned · P1  
Zalo import (where API permits)| planned · P1  
Mobile app (Tauri 2.0 mobile)| planned · P3  
Yjs CRDT offline draft sync| planned · P3  
Multi-region active-active| planned · P3+  
  
17

## References

  * **CHAT module specification** — (FR pending) through (FR pending).
  * **NFR — latency** — N(FR pending) (CHAT message-deliver p95 ≤ 200 ms).
  * **NFR — availability** — N(FR pending) (CHAT availability ≥ 99.9%).
  * **Formal FR catalogue** — (FR pending) catalogue with verification methods.
  * **Mattermost upstream** — open-core MIT/Apache messaging substrate.
  * **PGroonga** — Postgres full-text extension with VN tokenisation support.
  * **Yjs** — CRDT library for offline draft sync.
  * **Tauri 2.0** — Rust-based desktop + mobile shell.
  * **Whisper-large-v3** — self-hosted ASR for VN + EN voice messages.
  * **Vietnam PDPL (Law 91/2025)** — Art. 14 DSAR.
  * **EU AI Act** — Art. 12 logging, Art. 13 transparency (AI label on @genie answers).
  * **Architecture context:** [infrastructure.html#chat](<../../architecture/infrastructure.html#chat>).
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5 (capture surfaces)](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — CHAT is one of four canonical capture surfaces (alongside FS watcher, Cowork, Claude Code hook).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>) — CHAT FRs re-authored one-by-one via the `feature-request-author` Agent Skill; placeholder "(FR pending)" markers in this page are intentional.
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — CHAT placed at P0 · slice 2 in the reordered P0 build sequence (AI Gateway → AUTH stub → MCP → CHAT → Slack/Zalo decom).
  * **Research review:** `archive/2026-05-14/RESEARCH_REVIEW.md` (archived; see `cyberos/CHANGELOG.md`) — external reviewer rated CHAT module as "Solid" (8/10) with caveat that decommission gate is the biggest single risk in P0.
  * **Mattermost upstream governance:** [docs.mattermost.com/about/licensing](<https://docs.mattermost.com/about/licensing.html>) — fork pinned at last MIT/Apache LTS commit; license-drift watcher in legal-monitor pipeline.
  * **PGroonga + TinySegmenter:** [pgroonga.github.io](<https://pgroonga.github.io/>) \+ [TinySegmenter](<https://github.com/cmkang/tinysegmenter4j>) — VN full-text recall baseline.
  * **Vietnam PDPL (Law 91/2025):** Art. 7 (data-subject rights), Art. 14 (DSAR), Art. 20 (security obligations), Art. 38 (cross-border transfer).
  * **EU AI Act:** Art. 12 (logging — memory audit chain satisfies), Art. 13 (transparency — @lumi/@genie carry "AI-generated" label), Art. 50 (provider transparency obligation).



[← Previous: OBS](<../obs/index.html>) [All modules →](<../index.html#catalog>)

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
