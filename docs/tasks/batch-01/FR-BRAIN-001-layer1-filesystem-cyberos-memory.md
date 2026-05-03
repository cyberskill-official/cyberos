---
title: "BRAIN Layer 1 — filesystem `.cyberos-memory` with CRDT sync, six file ops, signed `.zip` export"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship Layer 1 of BRAIN: a Claude-style filesystem-synced memory folder named `.cyberos-memory` that lives at the root of every directory the user opts in to scan, contains plain-text Markdown files organised by category (`clients/`, `projects/`, `decisions/`, `people/`, `preferences/`, `glossary/`, `meetings/`, `risks/`), supports **six file operations** (read, write, edit, append, move, delete) all gated by a small anti-injection rule set, synchronises across the user's machines via CRDT (Yjs-based) so concurrent edits on a desktop and on the platform never lose data, and exports as a portable, signed `.zip` for migration or escrow. This is the human-readable, hand-editable, version-controllable foundation of the three-layer memory architecture (PRD §5.2). It ships in S0-3 alongside Layer 2 (FR-BRAIN-002).

## Problem

The PRD's central architectural bet is that memory becomes the platform's substrate. A purely vector-indexed memory is opaque, hard to audit, and difficult for the founder to inspect when the answer comes back wrong. A purely filesystem memory is easy to inspect but slow to query. The PRD's solution (PRD §5) is to layer them: Layer 1 is the human-readable Markdown filesystem; Layer 2 is the vector + graph index over Layer 1; Layer 3 is the cold archival corpus. Layer 1 must exist *first* so Layer 2 can index it.

Three product properties depend on Layer 1 being right:

- **Inspectability.** The founder must be able to open a Markdown file in any editor (VS Code, Obsidian, plain `cat`) and see exactly what the platform "remembers" about a topic. Without this, BRAIN is a black box.
- **Portability.** A tenant must be able to export their entire Layer 1 as a signed `.zip` and walk away. This is the basis of right-to-erasure compliance (PDPL Decree 13 plus GDPR Article 17 from P3+) and the basis of the "no lock-in" trust posture.
- **Cross-machine continuity.** The founder works from a laptop, a desktop, and an iPad-via-web-shell. A memory written on one device must be visible on the others within seconds — and concurrent edits from two devices must merge cleanly, not destructively.

S0-3 sprint risk-gate (PRD §17.3): "Citation correctness — the cited memory must be the actual source. A citation drift bug is sprint-blocking." Layer 1's deterministic file-as-source-of-truth pattern is what makes citation drift a tractable problem for Layer 2.

## Proposed Solution

The shape of the answer is a single `cyberos-brain-l1` Rust binary plus Postgres mirror table plus Genie panel UI. The binary runs on every machine the user authorises (`cyberos brain link <directory>`); it watches the filesystem, mirrors the canonical state to Postgres via the BRAIN subgraph, and applies remote changes received over the WebSocket sync channel.

**Directory layout.** A linked directory contains a top-level `.cyberos-memory/` folder with this layout:

```
.cyberos-memory/
├── INDEX.md                         # auto-generated; lists every memory file with one-line summary
├── clients/
│   ├── acme-corp.md
│   └── beta-llc.md
├── projects/
│   ├── project-alpha.md
│   └── project-beta.md
├── decisions/
│   └── 2026-04-22-adopt-cowork.md   # ISO date prefix; append-only by convention
├── people/
│   ├── jane-doe-acme.md
│   └── ravi-patel-acme.md
├── preferences/
│   └── stephen-cheng.md
├── glossary/
│   └── terms.md
├── meetings/
│   └── 2026-05-03-acme-sprint-14-review.md
├── risks/
│   └── 2026-04-30-mobile-release-freeze.md
├── .cyberos-meta/
│   ├── crdt.bin                     # Yjs document state vector; binary
│   ├── manifest.json                # files-and-hashes for the export pipeline
│   └── schema.json                  # the enforced frontmatter schema
└── README.md                        # human-friendly intro to the folder
```

Files are Markdown with YAML frontmatter:

```markdown
---
id: 01HRX9JK2N5PQT8WVZ7B9F3MGE
type: client
created_at: 2026-04-22T09:14:00+07:00
updated_at: 2026-05-03T11:40:00+07:00
authors: ["@stephen-cheng"]
tags: ["active", "long-term", "PST"]
links:
  - "../projects/project-alpha.md"
  - "../people/jane-doe-acme.md"
status: active
superseded_by: null
brain_layer1_version: 1
---

## Acme Corp

Type: long-term retainer (since 2024-09).
Primary contact: Jane Doe, VP Engineering, jane@acme.example.
…
```

Frontmatter is mandatory (the file is invalid without it) and is validated against `.cyberos-meta/schema.json` on every write. The `id` is a ULID (sortable); the path is the human-friendly slug.

**Six file operations.** The Rust binary exposes exactly six operations to local processes (CLI, Genie panel, MCP) and to remote sync:

1. `read(path)` → returns the file's raw bytes plus the canonical hash.
2. `write(path, content, expected_prev_hash | none)` → atomic write-or-create. If `expected_prev_hash` is supplied and does not match, returns `Conflict` (the conflict-resolution flow in PRD §5.6 is then engaged).
3. `edit(path, edits[], expected_prev_hash)` → applies a list of structured edits (insert, replace, delete a Markdown block by section heading or line range) atomically.
4. `append(path, block)` → idempotent append to a file (used by `decisions/` files which are append-only by convention).
5. `move(src_path, dst_path)` → renames the file; updates inbound `links:` references in other files.
6. `delete(path)` → moves the file to `.cyberos-memory/.cyberos-trash/` (soft-delete, retained 30 days), updates Postgres mirror, broadcasts the deletion event. Hard-delete is a separate `purge` operation reachable only by the DPO with audit-log entry.

There is no seventh operation (no `chmod`, no `chown`, no symbolic-link). A reduced surface is a smaller attack surface.

**Anti-injection rule set.** Every write is filtered through these rules before persisting:

- **No tool-call markers.** A line beginning with `cyberos.` and containing `(...)` is rejected with `code: BRAIN_TOOL_INJECTION`. The user's intent for a tool call goes through the MCP gateway, not through a memory file.
- **No untrusted-content tags from external sources.** A write whose source is `external` (email body, web page paste, customer document) is wrapped in `<untrusted_content source="...">...</untrusted_content>` blocks; the BRAIN consumer (CUO retrieval, Layer 2 indexer) treats the wrapped content as data, not instructions. This is the same pattern used in the Feature Request template (`README.md` §13 validation contract).
- **No personally-identifying data in compensation/equity/health/government-ID forms.** A write that matches the BRAIN denylist regexes (PRD §5.8 "Never ingest"; FR-BRAIN-002 codifies the regex set) is rejected with `code: BRAIN_DENYLIST_VIOLATION`. The user is shown the masked match and asked to revise.
- **Maximum file size.** 256 KB per Markdown file. Larger content is split or moved to KB module.
- **Maximum directory depth.** 5 levels under `.cyberos-memory/`.

**CRDT sync.** Yjs is the CRDT primitive. Each Markdown file is modelled as a Yjs document; the document state vector is persisted at `.cyberos-meta/crdt.bin`. The Rust binary opens a WebSocket connection to `wss://brain.cyberos.world/sync` (canonical tenant) authenticated by the Member's OAuth token; concurrent edits from two devices merge with last-write-wins on individual character positions but with no data loss on insertions. Conflicts at the file level (the same heading rewritten with semantically conflicting content on two devices) surface in the conflict-resolution UI specified in PRD §5.6 and FR-BRAIN-CONFLICT-001 (batch-02).

**Postgres mirror.** Every file write is mirrored to `brain.layer1_file`:
```
id, tenant_id, path, content_hash, frontmatter (jsonb), body_text, body_md,
crdt_state (bytea), authors, tags, links_to (uuid[]), created_at, updated_at,
soft_deleted_at, frontmatter_id (uuid)  -- the ULID from the file
```
Layer 2 (FR-BRAIN-002) reads from this table when indexing. The file remains the source of truth; the table is a queryable mirror.

**Signed `.zip` export.** The `cyberos brain export` command produces:
```
cyberos-memory-{tenant-slug}-{iso-date}.zip
├── .cyberos-memory/                # all files
├── manifest.json                   # paths + sha256 + counts + tenant id + export time
└── manifest.signature              # Ed25519 signature over manifest.json by the tenant's export key
```
The signature is verifiable by anyone holding the tenant's export public key (published at `https://{tenant}.cyberos.world/.well-known/cyberos-export-pubkey`). Importing into a new tenant or another platform is supported by `cyberos brain import` which validates the signature, deduplicates by ULID, and re-mirrors to Postgres.

**Auto Dream nightly consolidation (scaffold).** A nightly job at 03:00 ICT walks Layer 1, regenerates `INDEX.md`, prunes the `.cyberos-trash/` of items older than 30 days, recomputes `links_to` for every file, and asks Layer 2 (when present) to recompute community summaries. P0 ships only the scaffold (job runs, regenerates `INDEX.md`); the full Layer 2 GraphRAG community-summary recomputation lands at P1 entry per PRD §14.1.2.

**Genie panel UI.** A small panel surface exposes the canonical CRUD: "show me what you remember about Acme", "remember that Jane is now CTO", "forget the note about the 2025 retreat budget". The panel translates natural language to Layer 1 operations via the natural-language CRUD specified in PRD §5.7 (the LLM authoring of operations is FR-BRAIN-NLCRUD-001, batch-02). For S0-3 only the manual surface ships: a tree view of `.cyberos-memory/`, a Markdown editor, and a confirm-on-delete dialog.

**MCP tool surface.**
- `cyberos.brain.read_memory(path)` — read a Layer 1 file.
- `cyberos.brain.list_memories(prefix?, type?, tag?)` — list with filters.
- `cyberos.brain.write_memory(path, content, expected_prev_hash?)` — `destructive: true; requires_confirmation: true` if `expected_prev_hash` is missing AND the file already exists.
- `cyberos.brain.append_memory(path, block)` — `destructive: false` (idempotent append).
- `cyberos.brain.delete_memory(path)` — `destructive: true; requires_confirmation: true`.
- `cyberos.brain.export_memories(format: "zip")` — read-only.

The CUO persona scope contract for the CEO/COO/CTO skills includes `cyberos.brain.*` (read + write) per PRD §6.4; non-scoped personas (or anonymous read-only auditors) have read-only access.

## Alternatives Considered

- **Database-only memory (no filesystem).** Rejected: kills inspectability and forces every audit to ride through the application layer. The founder cannot `git diff` their own memory.
- **Filesystem only (no Postgres mirror).** Rejected: Layer 2's hybrid retrieval needs a queryable substrate; walking the filesystem on every retrieval is too slow and too IO-heavy.
- **JSON files instead of Markdown.** Rejected: Markdown is human-editable in any tool; JSON is not. We pay for this with frontmatter validation, which is cheap.
- **Per-tenant Git repository as the storage backend.** Rejected: appealing for the "git diff" property but introduces the operational cost of a Git server, the merge-conflict semantics of Git (which differ from Yjs CRDT), and the security risks of arbitrary `.gitconfig` injection. Tenants who want Git can run `git init` inside their `.cyberos-memory/` and we will not interfere — the directory is theirs.
- **iCloud / Dropbox / OneDrive as the sync backend.** Rejected: cross-tenant boundaries are not enforceable through consumer cloud sync; the residency story breaks; the binary signature on `.cyberos-meta/crdt.bin` would be mangled by these services.
- **Mem0-style vector-only memory at this layer.** Rejected: that is exactly Layer 2; this layer's purpose is the human-readable surface.

## Success Metrics

- **Primary metric.** S0-3 demo passes: (1) the founder writes "remember that Acme Corp is on a 90-day payment cycle" via the Genie panel, (2) the file `clients/acme-corp.md` appears in `.cyberos-memory/` within 1 second on both the desktop and the laptop, (3) a manual edit on the laptop and a parallel edit on the desktop both land in the file with no data loss after a 30-second offline window, (4) `cyberos brain export` produces a signed zip; importing it into a fresh synthetic tenant restores the same files with byte-identical hashes.
- **Guardrail metric.** Citation drift = 0 over the lifetime of P0. A "drift" is defined as: Layer 2 cites a Layer 1 file as the source of an answer, but the cited file does not contain the substring or paraphrase that supports the answer. This is sprint-blocking per PRD §17.3.
- **Performance NFR.** Layer 1 write-then-mirror p95 ≤ 500 ms; Layer 1 read p95 ≤ 50 ms (NFR-PERF-BRAIN-L1-001).

## Scope

**In-scope (S0-3).**
- `cyberos-brain-l1` Rust binary; macOS, Linux, Windows builds.
- The directory layout, six file operations, frontmatter schema, anti-injection rule set.
- Yjs-based CRDT sync over WebSocket against `wss://brain.cyberos.world/sync`.
- Postgres mirror table `brain.layer1_file` with full content, frontmatter, links.
- `cyberos brain link`, `cyberos brain unlink`, `cyberos brain status`, `cyberos brain export`, `cyberos brain import` CLI commands.
- Genie panel tree-view + Markdown editor + delete-confirmation surface.
- Auto Dream nightly job scaffold (regenerates `INDEX.md` only at P0; full consolidation at P1 entry).
- MCP tools: `read_memory`, `list_memories`, `write_memory`, `append_memory`, `delete_memory`, `export_memories`.
- Audit integration: every write/edit/move/delete records an audit row in scope `brain.l1.{tenant}`.

**Out-of-scope (deferred).**
- Conflict-resolution UI for semantic conflicts between two devices' edits (FR-BRAIN-CONFLICT-001 in batch-02).
- Natural-language CRUD ("forget that") authored by the LLM (FR-BRAIN-NLCRUD-001 in batch-02).
- Layer 2 GraphRAG community-summary recomputation (FR-BRAIN-002).
- iOS / Android sync clients (P3 mobile).
- End-to-end encryption of `.cyberos-memory/` content at rest on user devices (P2 — relies on a per-tenant key escrow path that we do not yet have).
- Per-folder-scope authorisation (Member-X-can-read-the-`clients/`-folder-but-not-the-`people/`-folder); P3.

## Dependencies

- FR-INFRA-001 (Postgres + NATS scaffold).
- FR-AUTH-001 (Member identity for the WebSocket sync).
- FR-AUTH-002 (audit log).
- FR-AI-001 (the natural-language CRUD path will route through here in batch-02; not a dependency of S0-3).
- The Yjs library (`y-crdt` Rust port) — vendored and pinned at S0-3.
- Compliance: PDPL Decree 13 (every write is personal-data-eligible; the denylist plus the per-folder DPIA template covers it). GDPR Article 17 (P3+; the `.zip` export plus `purge` operation provide the "right to be forgotten" path).
- Locked decisions referenced: DEC-032 (BRAIN three-layer architecture), DEC-033 (Layer 1 filesystem with Markdown + frontmatter), DEC-034 (Yjs CRDT sync), DEC-035 (six file operations only), DEC-036 (BRAIN ingestion-side denylist), DEC-037 (signed .zip export).

## AI Risk Assessment

Layer 1 itself is filesystem storage; the AI relevance is that Layer 2 indexes it and CUO retrieves from it. EU AI Act risk class: `limited` because Layer 1 is the substrate that an AI-generated answer cites. The three required subsections follow.

### Data Sources

Layer 1 stores per-tenant data, mostly written by the tenant's own Members. External content (emails, web pages, customer documents) enters Layer 1 only after passing through the CaMeL dual-LLM (FR-EMAIL-001 / FR-CHAT-001 in subsequent batches) which extracts sanitised facts and wraps any quoted text in `<untrusted_content>` blocks. No third-party training data is ingested into Layer 1.

### Human Oversight

Layer 1 is human-readable and hand-editable; the human is *the* oversight. Every edit by the LLM (when natural-language CRUD ships in batch-02) is presented as a diff in the Genie panel for the human to accept, edit, or reject before persisting. Deletion is soft (30-day trash) with a separate human-only purge path. The CUO never silently rewrites Layer 1; every write carries the persona-version stamp and a human acceptance step.

### Failure Modes

- **CRDT divergence between two devices.** Yjs's last-write-wins at the character level is the floor; semantic conflicts at the file level go to the conflict-resolution UI (batch-02). For S0-3 a divergence flag is raised on the file's frontmatter (`crdt_divergence_pending: true`) and the file is read-only until resolved.
- **Anti-injection bypass.** A user pastes a malicious tool-call marker into a memory file. The rule set rejects the write; an audit row records the attempt. Repeated attempts trigger a manual review.
- **Denylist false positive.** A legitimate note about a Member's role gets rejected because of a name pattern. Mitigation: the rejection message shows the masked match and the Member can override with a per-write "I confirm this is not personal data" attestation, recorded in the audit row.
- **Postgres mirror diverges from filesystem.** The reconciler reads the filesystem, computes content hashes, and updates the mirror to match; runs hourly. If the filesystem is the divergent side, the audit log records the reconciliation.
- **Sync server outage.** Local writes continue; the binary queues outbound CRDT updates; on reconnect, queued updates are replayed in order.
- **Lost laptop.** The platform's copy is the authority; `cyberos brain link` on a new device pulls down the canonical state. End-to-end encryption at rest on the device (P2) is the deferred mitigation for the "stolen laptop reads my memory" attack class.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted directory layout, six-file-ops contract, anti-injection rule set, MCP tool surface, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; CRDT semantics to be re-verified by the Engineering Lead at PR-review time.
