# BRAIN Auto-Sync — Universal Personal + Lumi's BRAIN

> 📦 **ARCHIVED · 2026-05-18.** This is the original 2026-05-14 design lock, restored from git after the modules/ refactor accidentally deleted it. Implementation guidance has migrated:
> - **Layer-1 protocol spec** → [`../modules/memory/AGENTS.md`](../modules/memory/AGENTS.md) (the live normative spec)
> - **Layer-1 runtime + tests** → [`../modules/memory/README.md`](../modules/memory/README.md) (233/235 tests green)
> - **Layer-2 service** → [`../services/brain/README.md`](../services/brain/README.md) (Rust scaffold, Wave 1)
> - **Multi-device sync FR** → [`feature-requests/brain/FR-BRAIN-103-multi-device-sync.md`](feature-requests/brain/FR-BRAIN-103-multi-device-sync.md)
> - **Per-stage FRs** → [`feature-requests/brain/`](feature-requests/brain/) (FR-BRAIN-101…111)
>
> Treat the prose below as the original product vision. Where it contradicts a live FR or current README, trust the live doc.

**Status:** v1.0.0 — design lock, 2026-05-14
**Author:** Stephen Cheng (CEO) — vision; Cowork (Claude) — drafted from chat
**Companion files:** [`memory/docs/AGENTS.md §14`](../memory/docs/AGENTS.md) (interop) · [`memory/docs/PROPOSAL.md`](../memory/docs/PROPOSAL.md) (where P13 lands) · [`docs/FR_AUTHORING_WORKFLOW.md`](FR_AUTHORING_WORKFLOW.md) (FRs will be authored from this design)
**Use when:** building any component of the auto-sync system or extending BRAIN to a new capture surface.

This is the formal design for turning CyberOS BRAIN from a project-scoped manual audit ledger into **the universal personal-and-shared memory substrate for an entire human + their org**. The vision is much larger than the project that named it. CyberOS is the first consumer of the protocol; the protocol stands alone.

---

## §1 — Vision in one paragraph

Every person who triggers the protocol gets one **Personal BRAIN** — an append-only, audit-chained, offline-first long-term memory store that captures everything they do across every folder on every machine they own (files they edit, decisions they lock, discussions they have, notes they write). The store is **portable**: copy `~/.cyberos-memory/` between laptops, desktops, phones, and your memory comes with you. Personal BRAINs do **two-way sync** with **Lumi's BRAIN** (the cloud-hosted org-tenant BRAIN, also called CUO's BRAIN or CyberSkill's BRAIN) — shareable memories push up to Lumi; team-shared memories pull down to you. Over time, Lumi aggregates patterns across team members, deduplicates, and **auto-evolves** a wisdom layer that exceeds any one contributor. CyberSkill (and any tenant) gets a compounding moat: the longer the org uses the platform, the smarter Lumi becomes.

---

## §2 — Naming

| Name | Definition |
|---|---|
| **Personal BRAIN** | A user's individual `.cyberos-memory/` store. One per human per machine, but portable; copying the folder is equivalent to moving the BRAIN. The single source of truth for that user's local memory. |
| **Lumi** | The Genie persona that fronts CUO (Chief Universal Officer). One face, one voice, ten C-level skills hot-loaded on demand. Lumi is *the assistant*; Lumi's BRAIN is *the substrate Lumi reads/writes*. |
| **Lumi's BRAIN** | The cloud-hosted org-tenant BRAIN. Also called **CUO's BRAIN** or **CyberSkill's BRAIN** depending on conversational context. Same store, three names for three audiences. |
| **Shared memory** | A memory record with `sync_class: shareable` (or `team-public` / `org-only`) that flows from Personal BRAIN → Lumi's BRAIN and is visible to other team members. |
| **Private memory** | A memory record with `sync_class: private` that stays in the Personal BRAIN forever. Never reaches Lumi. |
| **Multi-brain power** | The capability that emerges once N personal BRAINs feed one Lumi's BRAIN — pattern recognition, dedup, synthesis, auto-evolved wisdom. |
| **The protocol** | The set of rules in [`AGENTS.md`](../memory/docs/AGENTS.md), [`memory.schema.json`](../memory/docs/memory.schema.json), and this document that together define how a BRAIN behaves. Open standard. |

Going forward in this document: "BRAIN" without qualifier means *the protocol*. "Personal BRAIN" and "Lumi's BRAIN" are the two concrete instances.

---

## §3 — The three layers

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│            LAYER 3 · Lumi's BRAIN (Cloud, multi-tenant)                  │
│            ┌─────────────────────────────────────────────┐               │
│            │  org_tenant/                                │               │
│            │    ├── shareable memories from N people    │               │
│            │    ├── synthesised patterns                 │               │
│            │    ├── dedup-ed decisions                   │               │
│            │    └── auto-evolved wisdom                  │               │
│            └─────────────────────────────────────────────┘               │
│                            ▲             ▲                               │
│                            │ push        │ pull                          │
│              shareable     │             │   shared                      │
│              memories      │             │   memories                    │
│                            │             │                               │
│   LAYER 2 · Sync orchestrator  (per Personal BRAIN, runs in background)  │
│            ┌─────────────────────────────────────────────┐               │
│            │  state machine: local-write → pending-push  │               │
│            │                  → pushed → confirmed       │               │
│            │  conflict resolution: chain-position wins   │               │
│            │  privacy filter: sync_class enforcement     │               │
│            │  interval: 5 min default, configurable      │               │
│            └─────────────────────────────────────────────┘               │
│                            ▲                                             │
│                            │ tail audit chain                            │
│                            │                                             │
│   LAYER 1 · Personal BRAIN  (one per human, portable, offline-first)     │
│            ┌─────────────────────────────────────────────┐               │
│            │  ~/.cyberos-memory/                         │               │
│            │    ├── manifest.json                        │               │
│            │    ├── HEAD                                 │               │
│            │    ├── audit/  (append-only, MMR-chained)   │               │
│            │    └── memories/<kind>/<hex>/<file>.md      │               │
│            └─────────────────────────────────────────────┘               │
│                            ▲                                             │
│                            │ canonical Writer (put / move / delete)      │
│                            │                                             │
│   LAYER 0 · Capture surfaces  (any folder, any conversation, any agent)  │
│            ┌─────────────────────────────────────────────┐               │
│            │  watched folders ──┐                        │               │
│            │  cowork sessions ──┼─► Capture daemon       │               │
│            │  claude-code ──────┤   (auto-emits put)     │               │
│            │  slack / zalo ─────┤                        │               │
│            │  granola / meet ───┤                        │               │
│            │  notes / obsidian ─┘                        │               │
│            └─────────────────────────────────────────────┘               │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

Read top-to-bottom for the architecture; bottom-to-top for the data flow.

---

## §4 — Personal BRAIN (universal — any folder, any human)

### §4.1 Core invariants

- **One per human.** Not one per project. The Personal BRAIN is the user's *singular* long-term memory across everything they do on every machine they own.
- **Lives at a known XDG-style location.** Default: `$HOME/.cyberos-memory/` on macOS / Linux / WSL; `%APPDATA%\cyberos-memory\` on Windows. Override via `$CYBEROS_BRAIN_HOME`.
- **Offline-first.** Every read and write is local-disk-only. Network is optional and asynchronous (Layer 2 sync orchestrator).
- **Portable.** `cp -r ~/.cyberos-memory/ /Volumes/USB/` is a complete backup. Move the folder between machines; the BRAIN moves with it. No DB-rebuild step.
- **Append-only.** Per AGENTS.md §3: all mutations are `put`/`move`/`delete`. No in-place edits, no migrations that mutate prior frames.
- **Audit-chained.** MMR + Ed25519 STH. Every memory is provably from the user's BRAIN, in order.
- **Universal protocol — works on any folder.** You activate the BRAIN once; it captures activity across all *watched folders*. Watched folders are a config list maintained inside `manifest.json`.

### §4.2 Activation

The Personal BRAIN is activated once per human-machine:

```bash
# First time, on any new machine:
cyberos brain init                    # creates ~/.cyberos-memory/
cyberos brain watch ~/Projects        # add Projects/ as a watched folder
cyberos brain watch ~/Documents       # add Documents/
cyberos brain watch ~/Desktop         # add Desktop/

# Subsequent times — auto-discovers via manifest.json
cyberos brain status                  # see watched folders + last sync
```

After activation, the **capture daemon** (Layer 0) sits in the user's session and observes every watched folder for events worth remembering. Activity in unwatched folders is ignored entirely (privacy floor).

### §4.3 What gets captured automatically

A memory record is emitted for every activity that crosses any of these thresholds:

| Surface | Trigger | Memory kind | sync_class default |
|---|---|---|---|
| Watched folder · file create | new file > 1 KB | `facts` (a new file is a fact) | inherits folder default |
| Watched folder · file modify | content hash changes | `facts` (with version pointer to prior) | inherits folder default |
| Watched folder · git commit | `git commit` lands | `decisions` (with commit hash + message + diff stat) | inherits folder default |
| Cowork session | end-of-turn that "locks" a decision | `decisions` | `shareable` |
| Cowork session | discussion that resolves an open question | `decisions` | `shareable` |
| Claude Code session | tool calls that mutate the working folder | `facts` | folder default |
| Slack / Zalo (via MCP) | messages tagged `@brain` or `@lumi` | `discussions` | `shareable` (default) |
| Slack / Zalo (via MCP) | every Nth message in a thread (sampling for context) | `discussions` | `private` (default) |
| Granola / Meet (via MCP) | meeting end → ingest transcript | `discussions` | `shareable` |
| Notes app (Apple Notes / Obsidian / Notion) | note saved | `facts` | inherits folder default |
| Browser (via Claude in Chrome) | bookmark + selected text + read time | `references` | `private` |
| Email / Calendar (via MCP) | sent/received with `@brain` tag | `discussions` | folder default |

The capture daemon is **conservative by default** — when in doubt, *don't* capture. Stephen's principle "remember everything, even discussion" is honored via the higher-frequency surfaces (cowork, claude-code, granola, slack-when-tagged), not via blanket filesystem scraping. Anything written by the user with intent gets captured; tool noise does not.

### §4.4 Privacy classes (sync_class)

Inheriting and extending the existing AGENTS.md §15 + memory.schema.json `sync_class` enum:

| Class | Visibility | Default for |
|---|---|---|
| `private` | Personal BRAIN only — never pushed | Random browser scraps, half-finished notes, tool-output snippets |
| `personal` | Personal BRAIN — syncs across user's own machines (laptop ↔ desktop ↔ phone) but NOT to Lumi | Personal notes, drafts, exploratory work |
| `shareable` | Eligible for push to Lumi's BRAIN; subject to ACL filter at push time | Decisions, completed deliverables, meeting outcomes |
| `team-public` | Pushed to Lumi's BRAIN AND visible to all org members via shared scope | Locked policy decisions, RFC approvals, OKR commits |
| `org-only` | Pushed to Lumi's BRAIN but restricted by RBAC role | Compensation decisions, hiring decisions, financial state |

Per AGENTS.md §15, `private` is the default. Per-folder defaults can override (e.g. `~/Projects/cyberos/` defaults to `shareable`; `~/Documents/Personal/` stays `private`).

### §4.5 Portability — moving a Personal BRAIN

The "copy the folder" guarantee. Three concrete paths:

| Method | Frequency | Latency |
|---|---|---|
| Cold copy via USB / `rsync` | manual, one-off | hours |
| iCloud Drive / Dropbox / OneDrive folder sync | continuous | seconds (with conflict-sibling discipline per AGENTS.md §4 — the protocol detects sync-FS conflict siblings) |
| Lumi's BRAIN 2-way sync (Layer 2) | continuous, online | minutes |

For users who don't yet have Lumi's BRAIN deployed (org of one, offline-first, no cloud subscription), iCloud Drive sync is the recommended cross-device path. The protocol's sync-conflict detection (`layout-no-sync-conflict-siblings` invariant) catches iCloud's "filename (Mac)" / "filename (iPhone)" sibling creation and surfaces it to the user.

---

## §5 — Capture daemon (Layer 0)

The capture daemon is a long-running per-user process that observes the capture surfaces and emits Writer ops. It runs at user-session level (not root); it uses inotify / fseventsd / ReadDirectoryChangesW for the filesystem watches.

### §5.1 Subsystems

| Subsystem | Implementation | Lifecycle |
|---|---|---|
| **Filesystem watcher** | Rust binary using `notify` crate; watches all paths in `manifest.json.watched_folders` | Started on login via launchd / systemd-user / Task Scheduler |
| **Cowork session hook** | Cowork plugin that hooks every Cowork turn; emits a `put` for the turn's decision (if any) | Lives inside the Cowork session |
| **Claude Code hook** | `~/.claude-code/hooks/on-edit.sh` — fires after every file mutation | Lives inside the CC session |
| **MCP ingestion service** | Long-running service that pulls from Slack/Zalo/Granola/Meet/Apple-Notes MCP feeds | Started on login |
| **Browser companion** | Claude-in-Chrome integration emits bookmark + read-time memories | Lives inside Chrome extension |
| **CLI flush** | `cyberos brain capture <path-or-text>` for one-off manual capture | On-demand |

### §5.2 Capture envelope

Every captured event becomes a memory file with this frontmatter:

```yaml
---
id: <generated>
kind: facts | decisions | discussions | references | preferences
title: <auto-extracted>
source:
  surface: filesystem | cowork | claude-code | slack | zalo | granola | meet | notes | browser | cli
  path_or_url: <where it came from>
  captured_at: 2026-05-14T18:42:31.512Z
  capture_daemon_version: 1.0.0
sync_class: private | personal | shareable | team-public | org-only
classification: public | internal | confidential | restricted
acl:
  - subject: user:stephen
    rights: [read, write]
  # — for shareable+
  - subject: org:cyberskill
    rights: [read]
brain_chain_hash: <set by Writer>
brain_chain_prev: <set by Writer>
related: []
auto_capture: true
---
```

The `auto_capture: true` flag distinguishes daemon-emitted memories from human-authored ones. Useful for filtering during sync (e.g. "don't push auto-captured discussions to Lumi unless the user explicitly tagged them").

### §5.3 Rate limiting + dedup

The daemon implements three safety valves:

1. **Per-surface rate limit** — max 60 captures per minute per surface; excess goes into a queue with content-hash dedup.
2. **Content dedup** — content-addressed via SHA-256; an identical-content `put` is idempotent per AGENTS.md §3.4 but the daemon also pre-filters to avoid wasted work.
3. **Boring-edit suppression** — small whitespace-only or auto-format edits below a configurable threshold are coalesced into the next meaningful change.

---

## §6 — Lumi's BRAIN (Layer 3 — Cloud, shared)

### §6.1 What it is

A cloud-hosted multi-tenant CyberOS deployment whose primary product is *being the sync target for its tenant's team members' Personal BRAINs*. The same BRAIN protocol applies — same `Writer`, same `audit/binlog`, same MMR — but the store is shared and deployment-managed.

### §6.2 Per-tenant isolation

- One Lumi's BRAIN per tenant (org).
- Backed by RLS-isolated Postgres + S3 / R2 / MinIO at the storage layer.
- Tenant subject = `org:<slug>` (e.g. `org:cyberskill`).
- Per-user identity = `user:<email>` issued by AUTH module.
- Read scoping: every cross-module call passes a `tenant_id` claim; Lumi's BRAIN reads/writes are scoped to that tenant.

### §6.3 What Lumi sees

Lumi (the Genie / CUO) has *read access* to Lumi's BRAIN as part of its `allowed_brain_scopes` (per AGENTS.md §3.6 / SKILL.md frontmatter):

```yaml
allowed_brain_scopes:
  read:
    - org:<tenant>:shareable
    - org:<tenant>:team-public
    - org:<tenant>:org-only (subject to RBAC role)
    - user:<self>:*           # the asking user's own personal memories
  write:
    - org:<tenant>:lumi-decisions    # Lumi's own routing decisions
    - user:<self>:lumi-responses     # Lumi's responses to that user
```

Lumi cannot read another user's `private` or `personal` memories. The protocol enforces this; the user's privacy is non-negotiable.

### §6.4 Multi-tenancy + the existing TEN module

Lumi's BRAIN is *the killer feature* of the TEN module. The reviewer flagged: "ship a thin TEN-billing slice at P2 instead of P4" — Lumi's BRAIN is the *product* that thin slice sells.

Existing TEN spec at `website/docs/modules/ten.html` covers per-tenant subdomain, Stripe/VietQR billing, branded shell. Add: Lumi's BRAIN per tenant, provisioned at tenant create-time, scaled at tenant add-seat-time.

---

## §7 — Sync orchestrator (Layer 2)

### §7.1 State machine per memory record

```
              ┌─────────────────────────────────────────────────────────┐
              │                                                         │
              ▼                                                         │
   ┌──────────────────┐                                                 │
   │ local-write      │  user / daemon emits a `put`                    │
   └──────────────────┘                                                 │
              │                                                         │
              │ sync_class != private and != personal                   │
              ▼                                                         │
   ┌──────────────────┐                                                 │
   │ pending-push     │  queued for next sync window                    │
   └──────────────────┘                                                 │
              │                                                         │
              │ sync window opens (default every 5 min, configurable)  │
              ▼                                                         │
   ┌──────────────────┐                                                 │
   │ pushed           │  POSTed to Lumi's BRAIN; received OK            │
   └──────────────────┘                                                 │
              │                                                         │
              │ Lumi's BRAIN replays + chains; returns confirm + hash  │
              ▼                                                         │
   ┌──────────────────┐                                                 │
   │ confirmed        │  record's frontmatter updated: lumi_chain_hash  │
   └──────────────────┘                                                 │

   For pulls (Lumi → Personal):

   ┌──────────────────┐
   │ remote-write     │  another user's BRAIN pushed a memory to Lumi   │
   └──────────────────┘
              │
              │ sync window; this user's ACL grants read
              ▼
   ┌──────────────────┐
   │ pending-pull     │  queued for local apply                         │
   └──────────────────┘
              │
              │ apply: fresh `put` on local chain per AGENTS.md §14.2  │
              ▼
   ┌──────────────────┐
   │ imported         │  local record created; `extra.imported_from`    │
   │                  │  + `extra.foreign_chain` set per §14.2          │
   └──────────────────┘
```

### §7.2 Conflict resolution

Per AGENTS.md §14.2, every imported memory becomes a fresh `put` on the local chain. Two key consequences:

1. **No "merge" at the file level** — the local BRAIN never overwrites its own audit chain. A foreign record is a new local row that *references* the foreign one.
2. **Chain-position wins** — if two users both author memory ids that collide (e.g. both decided to call something `decisions/2026-05-14-foo.md`), the later-pushed wins by chain position on Lumi's BRAIN. The losing record is preserved as a `superseded_by:` reference.

Conflict frequency expected: low. Most memories are content-different even when topic-similar; SHA-256 content addressing dedups exact duplicates.

### §7.3 Sync transport

- **Protocol:** HTTP/2 + JWT (issued by AUTH).
- **Wire format:** Same canonical-JSON envelope as the binlog frame (msgspec sorted-keys). Each push is a batch of audit records + their body content. Each pull is a stream of records the user's ACL grants.
- **Authentication:** Per-user JWT (RS256, 15-min access + 30-d refresh per AUTH RFC).
- **Idempotency:** Content-addressed; the server safely deduplicates by record hash.
- **Resumability:** Each push records `last_pushed_seq` in `manifest.json`. Each pull records `last_pulled_seq_per_peer` per user-pair.
- **Offline tolerance:** Sync windows are best-effort. Personal BRAIN works fully offline; the orchestrator queues until network returns.

### §7.4 Sync interval — configurable

| Mode | Default interval | Use case |
|---|---|---|
| `realtime` | 10 s | Online collaboration during active meetings |
| `frequent` | 1 min | Active workday |
| `normal` (default) | 5 min | Default for desktop use |
| `infrequent` | 1 hr | Background; preserves battery on laptops |
| `manual` | `cyberos brain sync` | User-controlled |

Switch via `cyberos brain sync-mode normal`.

---

## §8 — Multi-brain power + auto-evolve (§3 Layer 3 enrichment)

### §8.1 The pattern

Once N personal BRAINs feed one Lumi's BRAIN, three new affordances become possible:

1. **Cross-person dedup.** Two users both authored a "we decided to switch from Postgres to CockroachDB" memory. Lumi recognises the semantic overlap and synthesises one canonical decision row that references both originals.
2. **Pattern recognition.** When 5 people across 3 months each author memories tagged with `compensation-question`, Lumi notices the pattern and creates a meta-memory: "compensation is a recurring discussion topic — does the team need a comp policy?" surfaces at the next CUO-CHRO weekly digest.
3. **Wisdom synthesis.** Periodically (weekly cadence), Lumi runs a synthesis pass over the previous interval's memories — clustering by topic, ranking by recurrence and depth, emitting `synthesis@1` artefacts that capture *what the org learned this week*.

### §8.2 Implementation sketch

The wisdom layer is a CUO sub-skill, not a separate module:

```
cuo/personas/synthesis-author/SKILL.md
```

Runs nightly. Walks the Lumi's BRAIN ledger for the prior 24 hours. Produces:

- `memories/synthesis/daily/2026-05-14.md` — what happened today
- `memories/synthesis/weekly/2026-W20.md` — what we learned this week
- `memories/synthesis/decisions-pending/2026-05-14-untriaged.md` — open questions surfaced from multi-person discussion threads

The synthesis output is itself a memory and follows the protocol. It's transparent — every claim Lumi's synthesis makes is hash-anchored to the underlying personal-BRAIN-originated rows. No black-box AI summarisation.

### §8.3 Compounding moat

This is the strategic payoff of the entire design:

- A new user joining a Lumi's BRAIN inherits *the org's accumulated wisdom* from day one.
- Lumi's quality of responses to any user improves as the org's collective memory grows.
- Switching cost climbs over time — your org's BRAIN is the most valuable asset on the cloud bill.

The reviewer's GTM concern was that the "ecosystem-as-a-service" thesis becomes a moat. **This is the moat.** Not the marketplace; the memory.

---

## §9 — Dependency map (what blocks what)

```
                  ┌────────────────────────────────────────┐
                  │ Multi-brain auto-evolve (Lumi wisdom)  │  ← Stage 5
                  └─────────────────────┬──────────────────┘
                                        │
                  ┌─────────────────────┴──────────────────┐
                  │ 2-way sync (Personal ↔ Lumi's BRAIN)   │  ← Stage 4
                  └─────────────────────┬──────────────────┘
                                        │
                  ┌─────────────────────┴──────────────────┐
                  │ Lumi's BRAIN deployment                │  ← Stage 3
                  │ (gated on TEN + AUTH + AI Gateway)     │
                  └─────────────────────┬──────────────────┘
                                        │
                  ┌─────────────────────┴──────────────────┐
                  │ Discussion + activity capture daemon   │  ← Stage 2
                  │ (Cowork hook, MCP feeds, etc.)         │
                  └─────────────────────┬──────────────────┘
                                        │
                  ┌─────────────────────┴──────────────────┐
                  │ Personal BRAIN — universal protocol    │  ← Stage 1
                  │ (any folder, portable, offline-first)  │
                  └────────────────────────────────────────┘
                                        │
                                        ▼
                              memory module (shipped — Stage 0)
```

### §9.1 Stage gating

| Stage | What | Gating dependency | Buildable when |
|---|---|---|---|
| **Stage 0** | Memory module shipped | — | ✅ Done (255 tests, 13/13 doctor invariants) |
| **Stage 1** | Personal BRAIN — universal | Extend memory module: `brain init`, `brain watch`, `brain status`, multi-folder watch in manifest.json | **Now** — no external dep |
| **Stage 2** | Capture daemon + Cowork hook + initial MCP feeds | Stage 1 + capture-daemon design + agreed event-to-memory mappings | **Within 2-4 weeks of Stage 1** |
| **Stage 3** | Lumi's BRAIN deployment | TEN module + AUTH module + AI Gateway (per reviewer's reorder) | **P2 (M+9)** — gated on those three modules shipping |
| **Stage 4** | 2-way sync orchestrator | Stage 3 + sync state machine + JWT-based wire protocol | **P2 (M+9) — runs in parallel to Stage 3** |
| **Stage 5** | Multi-brain auto-evolve | Stage 4 + synthesis sub-skill | **P3 (M+12)** |

Stage 1 + 2 are **buildable today** and don't require any of the unbuilt P0+ modules to ship first. Stages 3–5 ride the P0+P2 critical path.

---

## §10 — Privacy + governance

### §10.1 Default-deny

Every capture is `private` unless the user explicitly elevates it or the folder default elevates it. A user who never configures anything sees zero memories pushed to Lumi.

### §10.2 Explicit opt-in by folder

```bash
cyberos brain watch ~/Projects/cyberos --default-sync-class shareable
cyberos brain watch ~/Documents/Personal --default-sync-class private
```

### §10.3 Per-memory opt-down / opt-up

```bash
# Mark a memory as fully private (won't sync), even if folder defaults shareable:
cyberos brain reclass <memory-id> private

# Mark a private memory as shareable (will sync on next window):
cyberos brain reclass <memory-id> shareable
```

### §10.4 Right to forget

Per AGENTS.md §3.6 + §17.1, GDPR Art. 17 erasure is supported via `delete(path, "purge")`. For Lumi's BRAIN, this propagates: a purge on Personal BRAIN emits a `purge_propagate` request that, after AUTH-verifying the requester, redacts the corresponding record on Lumi's BRAIN. The *fact* of erasure remains audit-chained on both BRAINs (per §3.6 — purge of purge is forbidden).

### §10.5 PII detection at capture time

The capture daemon runs Presidio (or equivalent) at write time. Detected PII is flagged in the memory's frontmatter:

```yaml
pii_flags:
  - kind: email
    redacted_in: body
  - kind: phone
    redacted_in: body
  - kind: cccd  # Vietnamese citizen ID
    redacted_in: body
```

Memories with `pii_flags` and `sync_class != private` are *held back from sync* and surface as a Question in the next Cowork session: "this memory contains PII — confirm sync intent?".

---

## §11 — AGENTS.md additions (Proposal P13)

This design requires three protocol-level additions to AGENTS.md. They land as **Proposal P13** in `memory/docs/PROPOSAL.md`. Stages:

### §11.1 P13 Stage 1 — universal personal BRAIN

- **§4.6 (new)** — `BRAIN_HOME` resolution. Default `$HOME/.cyberos-memory/`. Override via `$CYBEROS_BRAIN_HOME`. Resolved once at process start; cached for the session.
- **manifest.json schema extension** — add `watched_folders: [{path, default_sync_class, since}]` array.
- **CLI additions** — `brain init`, `brain watch <path>`, `brain unwatch <path>`, `brain status`, `brain capture <path-or-text>`.
- **Doctor invariant additions** — `layout-watched-folders-exist` (warn if a watched folder no longer exists), `layout-watched-folders-permissions` (warn if a watched folder is unreadable).

### §11.2 P13 Stage 2 — capture daemon

- **§5 (new)** — capture surfaces, rate limiting, dedup, auto_capture flag.
- **memory.schema.json `Frontmatter` extension** — `source` block, `auto_capture` boolean, `pii_flags` array.
- **invariants.md** — `capture-daemon-not-stuck` (warn if daemon hasn't emitted in > 24 h on a watched-folder), `capture-daemon-version-current` (warn if daemon version is > 1 minor behind protocol).

### §11.3 P13 Stage 3+4 — sync

- **§14.3 (new)** — bi-directional sync protocol. Builds on §14.1 (interop) and §14.2 (cross-BRAIN import). Adds: sync state machine, JWT wire format, `manifest.imports.lumi.last_pulled_seq` / `manifest.pushes.lumi.last_pushed_seq`.
- **sync_class enum extension** — add `personal`, `team-public`, `org-only` (current spec has `private` + `shareable`).
- **Doctor invariant additions** — `sync-orchestrator-running`, `sync-last-success-recent`.

### §11.4 P13 Stage 5 — synthesis

- **§7.6 (consolidation extension)** — synthesis as a fourth phase: Walk → Compact → Sign → **Synthesise** → Publish.
- **memory.schema.json `MemoryKind` extension** — add `synthesis` to the closed enum.
- **CUO sub-skill registration** — `cuo/personas/synthesis-author` becomes a first-class skill bundle.

---

## §12 — CyberOS-specific implications

This vision changes the strategic narrative.

### §12.1 The product CyberSkill sells

Before this design: "an internal-ops platform with a marketplace."

After this design: **"the personal-and-team memory layer for every knowledge worker, with internal-ops modules built on top."**

Personal BRAIN ships first (Stage 1 + 2) — useful to *anyone* with a laptop, including non-CyberOS users. Lumi's BRAIN ships at P2 — useful to *any org*, with CyberOS's 22 modules as the value-add.

This sharpens the GTM concern the reviewer flagged. The natural buyer is no longer just *other Vietnamese software agencies* (~200–500 firms, $27M ceiling) — it's *any team that wants a private long-term memory layer*. That's a much larger TAM. The wedge into Vietnamese SMEs becomes one of several wedges.

### §12.2 Open-source distribution

Personal BRAIN as Stage 1 should be **OSS day-1**. Free CLI. Apache 2.0. No login required. This is the OSS distribution mechanism the strategy doc already calls out (Level 1 — Internal → Level 2 — OSS distribution).

Cloud BRAIN (Stage 3+) is the commercial product — sold by tenant, by user-count, by storage-tier. Lumi's brain capacity scales the price.

### §12.3 Compounding moat

This is *the* answer to the reviewer's GTM #5 score. The marketplace is premature; the *memory* is the moat. A tenant six months into using Lumi's BRAIN has accumulated context that no competitor can replicate. Switching cost = the value of the org's BRAIN.

### §12.4 Scope risk

Honest call-out: this design adds meaningful scope to the platform vision. Stage 1+2 are small extensions of the existing memory module (~2–3 sprints). Stage 3+ rides the P0+P2 critical path and is *additive* to TEN, not separate.

Recommendation: **adopt Personal BRAIN (Stage 1+2) as a P0 extension of the memory module** rather than as a new module. It doesn't add to the 22-module count; it deepens module #1.

---

## §13 — Naming + branding decisions to lock

These are open today; lock with Stephen before Stage 1 codes:

| Decision | Recommendation |
|---|---|
| Genie name | **Lumi** (per Stephen). Short, gender-neutral, "light" connotation, Vietnamese-friendly pronunciation. |
| Shared-cloud BRAIN name in user-facing copy | **Lumi's BRAIN** (warm, persona-attached) for end-user surfaces; **CUO's BRAIN** for technical/architectural docs; **CyberSkill's BRAIN** as the deployment-specific instance. |
| CLI brand | `cyberos brain ...` — keep the protocol generic; do not brand the CLI as "lumi" (the protocol stands alone from any one assistant). |
| Manifest header naming | `manifest.json` key `brain_owner: { name, email, lumi_tenant }` — explicit ownership for portability + sync auth. |
| Marketing name for the protocol | **The BRAIN Protocol** (capitalised). Document as "BRAIN" only when context is clear; "the BRAIN Protocol" in marketing + spec contexts. |

---

## §14 — What ships in the next 4 weeks

Concrete sprint plan for Stage 1+2:

| Week | Slice | Output |
|---|---|---|
| 1 | Personal-BRAIN protocol extension | `cyberos brain init/watch/unwatch/status` CLI. Manifest schema + invariants. Tests. |
| 2 | Filesystem-watcher daemon | Rust binary, `notify`-based, launchd/systemd-user unit files. Emits `facts` for file events; rate-limited. |
| 3 | Cowork session hook | Cowork plugin that emits a `decisions` memory at end of every session where a `lock` was detected. |
| 4 | MCP capture for Slack + Granola | Long-running service tailing Slack channels tagged `@lumi` and Granola transcripts. Emits `discussions` memories. |

After week 4, every CyberSkill team member has automatic, offline-first, portable, audit-chained capture across their laptop and team conversations. Lumi's BRAIN (Stages 3-5) lights up at P2.

---

## §15 — Locked decisions (2026-05-14)

The five design questions raised in this section's prior draft were answered by Stephen on 2026-05-14. The answers are normative.

### §15.1 CLI brand → `cyberos brain ...`

Single binary. All BRAIN-protocol commands ship under `cyberos brain *`. No standalone `brain` binary. Reasons:
- Consistency with existing 30 `cyberos` subcommands.
- One PATH entry, one update story, one auth context.
- The protocol stands beyond CyberOS conceptually, but in practice every consumer this year is running on a machine that also has the `cyberos` CLI.

If/when the protocol gains non-CyberOS consumers (hypothetical: another vendor adopting AGENTS.md), they can ship their own binary; the protocol is open.

### §15.2 Watched-folder default → explicit opt-in for every folder

`cyberos brain init` creates the store but watches nothing. The user must run `cyberos brain watch <path>` to opt each folder in. Rationale: privacy floor is maximally conservative. The "remember everything" guarantee only applies to folders the user explicitly opted in. Footgun risk = zero on first install.

Default sync_class for a watched folder is `private` unless overridden at watch-time via `--default-sync-class <class>`.

### §15.3 MCP capture surfaces → Claude-only, current scope

Stephen's current toolchain is Anthropic Claude only (Cowork + Claude Code). The capture daemon's initial implementation prioritises these two surfaces. Other MCP feeds (Slack, Zalo, Granola, Apple Notes, Notion, etc.) are deferred until the team adopts them or the use case demands them.

Stage-2 sprint scope simplifies to:

| Sprint week | Surface | Output |
|---|---|---|
| Week 1 | Personal-BRAIN protocol extension (`brain init/watch/unwatch/status/capture`) | CLI + invariants + tests |
| Week 2 | Filesystem watcher daemon | Rust binary; launchd/systemd-user; `facts` for file events |
| Week 3 | **Cowork session hook** | Cowork plugin emits `decisions` memory at session-end-with-lock |
| Week 4 | **Claude Code hook** | `~/.claude-code/hooks/on-tool-use.sh` emits `facts` for working-folder mutations |

Other MCP capture surfaces are documented in §5.1 of this doc as the protocol's *capability* but are not part of Stage 2's *delivery*. They join the roadmap when explicitly requested.

### §15.4 Per-folder sync_class ergonomics → flag at watch-time

`cyberos brain watch <path> --default-sync-class <private|personal|shareable|team-public|org-only>`. No interactive prompt. If `--default-sync-class` is omitted, the value is `private`. Per-memory override remains available via `cyberos brain reclass <memory-id> <class>`.

### §15.5 User-prompt UX → inline in Cowork + desktop-notif fallback

Two-channel surfacing:

| Context | Surface |
|---|---|
| Active Cowork session present | The prompt renders inline in the next Cowork turn. Lowest context-switch cost; matches existing interaction model. |
| No active Cowork session | Desktop notification (macOS UNUserNotificationCenter / Linux libnotify / Windows ToastNotification). Click-through opens the relevant `cyberos brain pending` view. |
| Non-urgent items | Accumulate in `cyberos brain pending`; user reviews on own cadence (e.g. end-of-day). |

The "urgency" classification is:
- **Urgent** (PII confirmation, sync-class elevation request, irreversible delete): desktop notif if no Cowork; Cowork inline if present.
- **Non-urgent** (new watched folder reaching sync window for first time, summary digest): always `cyberos brain pending`; no notification.

---

## §15-bis — Carried-forward open questions (lower priority)

These are not blocking Stage 1 but should be answered before Stage 2 ships:

1. **Watcher granularity** — `inotify` / `fseventsd` watches every file event; do we want a debounce window (e.g. batch 10 events within 200ms into one capture)?
2. **Encryption-at-rest for Personal BRAIN** — per AGENTS.md §5.4 the `Envelope` schema supports `cipher`. Should `cyberos brain init` default to encrypted-store? (Default-no for ergonomics; opt-in via `--encrypted`.)
3. **`brain status` UX** — what does the user see? Watched-folder list, last-sync time, pending-PII count, audit-row count, doctor invariants — full digest in one screen?
4. **iCloud / Dropbox folder support** — when `~/.cyberos-memory/` is itself synced by a cloud-FS provider, the protocol's `layout-no-sync-conflict-siblings` invariant detects + surfaces conflicts but doesn't resolve them. Add a resolver UX?

---

## §16 — Where to read next

- [`memory/docs/AGENTS.md`](../memory/docs/AGENTS.md) — current BRAIN protocol (the substrate this design extends)
- [`memory/docs/PROPOSAL.md`](../memory/docs/PROPOSAL.md) — where Proposal P13 (stages 1-5) lands as formal protocol extension
- [`memory/docs/memory.schema.json`](../memory/docs/memory.schema.json) — schema additions for this design
- [`docs/FR_AUTHORING_WORKFLOW.md`](FR_AUTHORING_WORKFLOW.md) — every FR in this design is authored via `fr-author`
- [`docs/AUDIT_AND_PLAN_2026_05_14.md`](AUDIT_AND_PLAN_2026_05_14.md) — where Stage 3+ slots into the build sequence
- [`docs/RESEARCH_REVIEW_2026_05_14.md`](RESEARCH_REVIEW_2026_05_14.md) §2 + §7 — confirms the strategic case (this is the moat the reviewer was looking for)
- [`strategy/CYBEROS_STRATEGY.md`](../strategy/CYBEROS_STRATEGY.md) §4 — to be updated: Personal BRAIN as a Level-2 OSS distribution surface; Lumi's BRAIN as a Level-3 SaaS product

---

*End of design.* This is the lock for the BRAIN auto-sync vision. Stages 1+2 are buildable now; Stages 3-5 ride the P0+P2 critical path.
