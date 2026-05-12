# `docs/memory/` — CyberOS BRAIN protocol + Reader's Guide

## What's in this folder

| File | Purpose | When to read |
| --- | --- | --- |
| [`README.md`](README.md) | **You are here.** On-ramp + 32-part operator manual + sister-folder index. | First read; recurring reference. |
| [`AGENTS.md`](AGENTS.md) | The protocol itself — single source of truth (114 KB, ~1,241 lines). | Authoritative reference when implementing a rule. |
| [`CHANGELOG.md`](CHANGELOG.md) | Daily landing log; every batch (1–27) recorded line-by-line. | Audit trail; "what changed today". |

## Sister folders under `docs/`

| Folder | What's there |
| --- | --- |
| [`../prd/`](../prd/) | Product Requirements Doc (`PRD.docx` + `CHANGELOG.md`). |
| [`../srs/`](../srs/) | System Requirements Spec (`SRS.docx` + `CHANGELOG.md`). |
| [`../skills/`](../skills/) | Skills-layer operator manual (single doc, Parts 1–30). |
| [`../contracts/`](../contracts/) | Versioned artefact schemas: `feature_request@1`, `task@1`, `project_brief@1`, `prd@1`, `srs@1`. |
| [`../tours/`](../tours/) | 10 guided walkthroughs (`.tour` files) for common workflows. |

## Symlink recipe (for new projects)

```bash
cd /path/to/your-project
ln -s /path/to/cyberos/docs/memory/AGENTS.md AGENTS.md
ln -s /path/to/cyberos/docs/memory/AGENTS.md CLAUDE.md
```

Both point at the SAME `AGENTS.md` — there's no compact variant since Batch 27 (single source of truth).

## Folder history

- **2026-05-12 (Batch 24)** — memory-protocol docs moved from `docs/CyberOS-*.md` into this folder.
- **2026-05-12 (Batch 25)** — sister folders `docs/prd/` and `docs/srs/` introduced.
- **2026-05-12 (Batch 27)** — `AGENTS-CORE.md` removed (single source of truth); `INDEX.md` merged into this README.

---

# Reader's Guide & Evolution Manual

The BRAIN (`.cyberos-memory/`) is your project's persistent memory — where agents go to recall facts, decisions, and context across sessions, machines, and people. This guide explains what the BRAIN is, how to use it day-to-day, and — critically — how to evolve the protocol that governs it without breaking memories already written.

This document is a **companion** to `CyberOS-AGENTS.md` (the protocol itself), not a replacement. The protocol is dense and rule-heavy because it must be unambiguous to two agents reading it from different machines and reach identical accept/reject decisions on every input. This README is the friendly on-ramp. Read this first; consult AGENTS.md for the exact wording when you're writing code that enforces a rule, designing a new feature, or proposing a protocol amendment.

> **Audience.** Humans onboarding to CyberSkill (current employees, future hires, future clients consuming the PORTAL slice) and agents on their first read of a project's BRAIN.

> **Scope.** Describes the BRAIN as it stands today (filesystem layer running on individual laptops) and as it evolves (multi-person sync via the BRAIN module at P1, multi-tenant SaaS at P3+). Where the answer is "today X, tomorrow Y," both are stated.

> **How to navigate this doc.** Parts 1–4 are the mental model. Part 5 is the personal/org boundary. Parts 6–7 are operational (distribution & self-audit). Part 8 is the safe-evolution playbook — read this before proposing any protocol change. Parts 9–12 are reference material you'll come back to.

---

## Part 1 — What is the BRAIN?

### Elevator pitch

A folder at `<project-root>/.cyberos-memory/` that holds every fact, decision, conflict, and context-clue an agent needs to behave like a competent collaborator on this project — written in plain text, audited cryptographically, portable as a zip. Two agents on two different days reach the same conclusions because they're reading the same store under the same rules.

It is the **single source of truth** for what's known about this project. Vector indexes, graph stores, and chat-context memory are derivable caches; never authoritative. If a fact isn't in the BRAIN, the agent doesn't know it.

### Why it exists

Three concrete problems:

**Agents forget.** A chat session ends; the next session starts cold. Without a persistent store, every conversation re-litigates settled decisions, re-asks resolved questions, and re-discovers known constraints. The BRAIN is what makes "Stephen told the agent on Monday" still true on Friday.

**Distributed teams diverge.** Ten employees on ten laptops with ten chat histories means ten different mental models of what's actually true. The BRAIN gives every employee's agent a path to the same ground truth.

**Memory needs to be **auditable**, not just persistent.** When an agent claims "DEC-094 was decided on 2026-06-01," there must be a way to verify *who* decided, *when*, *based on what source*, and *whether anything has since superseded it*. The audit ledger answers all four questions for every memory the BRAIN has ever held.

### What the BRAIN is NOT

- **Not a vector store.** Vector indexes (Layer 2 of the BRAIN architecture) sit *on top of* the filesystem layer; they're rebuilt from it, not authoritative against it.
- **Not chat history.** Conversations are ephemeral by design. The BRAIN is the residue of a conversation that the user explicitly chose to keep.
- **Not a documentation system.** Docs (Notion, Confluence, the `docs/` folder of a repo) are *sources* the BRAIN may ingest. The BRAIN is the agent's working notes about those sources, with provenance pointers back to them.
- **Not Git.** Git versions code; BRAIN versions facts and decisions. They're complementary — a BRAIN may live inside a Git repo, but the audit ledger is its own append-only chain independent of commits.
- **Not a search index.** The BRAIN's `index/` subdirectory is a regenerable cache; deleting it loses no truth, just speed.

### Where it lives

`<project-root>/.cyberos-memory/` on the **real local filesystem path** — the same folder you see in Finder. The protocol explicitly forbids operating against sandbox paths, virtualised mounts, or temporary directories (AGENTS.md §0.1). This is the single most common source of "lost memory" surprise: the agent appeared to write to the BRAIN, but it actually wrote to a sandbox copy that vanishes when the session ends.

If your agent says it wrote a memory but you can't find the file: check the path it claims to have written to. If it starts with `/sessions/`, `/.cyberos-memory/cache/folders/`, `.cyberos-memory/cache/`, or anything in §0.1's forbidden list, the write went to the void.

---

## Part 2 — The five pillars

The protocol's whole behaviour falls out of five non-negotiable principles. Internalise these and the rule-by-rule details of AGENTS.md become predictable.

### 1. Append-mostly

Every meaningful state change adds a row to `audit/<YYYY-MM>.jsonl`. The ledger is **append-only** — never edited in place, never reordered, never deleted. Mistakes are corrected by adding new rows that reference the bad ones (`op:"revert"`, `op:"corrects"`), not by erasing history. Two consequences: you can always reconstruct what the BRAIN believed at any past instant, and any tampering is detectable because the Merkle chain breaks.

### 2. Soft delete only

`delete` is a tombstone, not an erase. The file's body is preserved verbatim; its frontmatter flips `tombstoned: true` and gains `deleted_at`/`deleted_by`/`tombstone_reason`. Hard erasure exists only as a manual right-to-erasure flow (GDPR DSAR equivalent) initiated by a human. The reason: agents make mistakes; users change their minds; "I told the agent to forget" must always be reversible.

### 3. Conflicts preserved, never silently resolved

When two memories contradict each other, the BRAIN does not pick a winner unless it can do so with high confidence (same-class auto-resolution) or by source-tier comparison (§9.1 step 0). For anything touching `personnel` or `client` classification, conflicts always go to a human with four options: keep A, keep B, keep both as a disputed pair, or write a new memory replacing both. Disputed pairs are a valid permanent state — you don't have to resolve every contradiction.

### 4. Six file operations only

The agent can do exactly six things to memory files: `view`, `create`, `str_replace`, `insert`, `delete` (soft), `rename` (intra-scope only). Multi-region edits, overwrites, hard erasures, cross-scope renames, and direct mutation of `audit/` are all forbidden. Constraining the operation surface to six is what makes the protocol auditable: every change is one row, one diff, one before-hash, one after-hash.

### 5. Determinism

Two exports of the same state are byte-identical. Two agents on two laptops accept or reject the same input identically. This is achieved via canonical JSON for hashing (§7.2), NFC Unicode normalisation, deterministic zip generation (§11.2), and explicit validators with no implementation-defined behaviour. Determinism is what makes the BRAIN portable, replayable, and verifiable.

---

## Part 3 — Mental model: three layers

The full BRAIN architecture (per FACT-004 in your `.cyberos-memory/`) has three layers. The protocol you're reading governs Layer 1 only; Layers 2 and 3 are runtime services that ship with the CyberOS BRAIN module at P0+.

### Layer 1 — Filesystem `.cyberos-memory/` *(today)*

What you have right now. YAML frontmatter + Markdown body per file, one Merkle-chained audit row per mutation, six file ops. Lives on the user's real local filesystem. Portable as a deterministic zip. **This is what AGENTS.md describes.**

Designed for: durability, auditability, portability, single-actor sovereignty over personal memory.

### Layer 2 — Vector + graph fact memory *(BRAIN module, P0+)*

A retrieval substrate built on PostgreSQL extensions: `pgvector` (HNSW index) for vector search, Apache AGE for graph relationships, PGroonga for multilingual full-text. An LLM judge with a 0.85 confidence threshold runs four operations — ADD, UPDATE, DELETE, NOOP — against the fact graph derived from Layer 1. Hybrid retrieval pipeline: vector + lexical + graph → bge-reranker-v2-m3 → context expansion via Anthropic Contextual Retrieval. GraphRAG community summaries provide canonical-entity navigation.

Designed for: fast retrieval (p95 ≤ 250ms), multi-modal recall, agent-ergonomic Q&A.

### Layer 3 — Archival corpus *(BRAIN module, P0+)*

Cold-tier S3-compatible storage of all ingested content (chat threads, project artefacts, KB pages, email summaries, learning records). Used for replay, historical Q&A, and DSAR fulfilment. Retention rules per classification (§5.4).

Designed for: complete recall, compliance, evidentiary backstop.

### How they relate

Layer 1 is **authoritative**. Layer 2 is a derived **index**. Layer 3 is the **raw corpus** Layer 1 was distilled from. If Layer 2 disagrees with Layer 1, Layer 1 wins and Layer 2 gets rebuilt. If Layer 1 disagrees with Layer 3, that's a drift signal worth a human review.

A single tenant has one logical BRAIN spanning all three layers. CyberSkill (the company) is one tenant today. As CyberOS becomes multi-tenant SaaS, each customer organisation gets its own three-layer BRAIN, isolated by Postgres RLS plus per-tenant region pinning for residency.

---

## Part 4 — A day in the life

A concrete example to ground the mental model. Stephen records a pricing decision; the agent validates, audits, surfaces a conflict, the human resolves, and a week later a drift candidate flags that the source has moved on.

### Monday 09:14 ICT — Stephen makes a decision

Stephen is in a CyberOS sales conversation. He decides on three pricing tiers (Starter / Team / Enterprise). He asks the agent: *"Remember this pricing decision: Starter $19/seat/month, Team $49, Enterprise custom — effective from 2026-06-01."*

The agent classifies the request: `intent=create`, `scope=memories/decisions/`, `classification=operational`, `authority=human-edited` (it's coming from Stephen directly).

It runs the path-traversal guard (§4.1) on the proposed path `memories/decisions/DEC-094-pricing-tiers.md`, the content gate (§4.2) on the body, and the file-content hygiene gate (§4.3) on frontmatter + body. All three pass.

The two-phase atomic write (§4.4) kicks in: validate → append audit row carrying the after-hash → tmp+rename to disk → fsync. One audit row, one file on disk, atomic.

The §14 end-of-response block tells Stephen what landed:

```
- memories/decisions/DEC-094-pricing-tiers.md: created (mem_018f...; sha256:a3...)
- audit/2026-05.jsonl: 1 row appended; head=sha256:b7...
```

Total elapsed: under a second.

### Tuesday 14:22 ICT — Conflict surfaces

Miguel pulls down Stephen's `shared`-class memories from the org BRAIN (this flow ships with the BRAIN module at P1; today it's manual via export/import per §11.5). His agent ingests `DEC-094` and runs §9.1 conflict detection.

It finds `DEC-073-pricing-experiments.md` from three months ago, `classification=operational`, with content saying *"Experimenting with two-tier (Free/Pro) pricing."* Different facts, same scope.

§9.1 step 0: source-freshness tier check. DEC-073 is `tier:50` (chat-derived); DEC-094 is `tier:10` (founder-confirmed decision). Lower tier wins automatically. DEC-073 gets `superseded_by: mem_018f...` and `tombstoned: true`; DEC-094 stands.

But because both are `operational` (not `personnel` or `client`), this auto-resolution is allowed. If DEC-073 had been `client`-classified, the agent would have written a `conflicts/<…>.json` and waited for human review with the four options.

The §14 block on Miguel's side surfaces the auto-resolution; the audit row carries `provenance.source: "conflict_resolution"`.

### Wednesday 11:00 ICT — Consolidation runs

Miguel's session has accumulated 27 audit rows since last consolidation. §8 fires automatically. Phase 1 surfaces candidates; phase 2 detects conflicts (none new); phase 3 runs conservative merge; phase 4 reorganises any file >10KB; phase 5 updates manifest.

Phase 6 (the new self-audit per §8.7, once you adopt TIER 1) walks the whole store: schema validates, supersedes DAG is consistent, audit chain integrity confirmed, no orphan files. Health report appended at `meta/health/2026-05-13-sha256-c4....md`. Severity: all-INFO. The §14 block reports `health: 0 critical / 0 warn / 12 info`.

### Following Monday 09:00 ICT — Drift candidate

The Notion source page DEC-094 was derived from has been edited (Stephen tweaked the Enterprise tier description). The §8.6 source-coverage validator re-hashes the source, finds the SHA mismatch, writes `memories/drift/2026-05-19-pricing-source-update.md`, and surfaces a `WARN` to Stephen.

He now has three options: re-ingest (creates DEC-094-v2, supersedes original), accept the drift (DEC-094 stays as-is, drift record explains why source moved on), or update the Notion source to match DEC-094.

### What you just saw

Five protocol features in ten minutes of agent work: write validation, atomic write + audit chain, conflict detection, auto-resolution gating, source-coverage validation. All six file operations got exercised at least once. No memory was ever overwritten in place; no audit row was ever modified after writing.

This is what "auditable, append-only memory" looks like in practice.

---

## Part 5 — Personal vs org: the four sync classes

The biggest architectural question once a BRAIN goes multi-person: **which scopes are personal-only and which flow to a shared store?** The answer is the `sync_class` field on every memory, with sensible defaults per scope.

### The four classes

**`local-only`** — never leaves the machine. Operational machinery, personal-private memories, ephemeral indexes. Examples: `audit/`, `meta/`, `index/`, `exports/`, `conflicts/`, `.lock`, `member/<self>/private/`, `memories/drift/`.

**`publishable`** — local until the subject explicitly publishes; then mirrored into the org BRAIN. The subject (the person whose memory it is) controls publication. Examples: `member/<self>/` (non-private), `memories/preferences/`, `memories/refinements/`, `memories/decisions/` (until promoted), `memories/facts/` (personal facts).

**`shared`** — sourced from the org BRAIN; not authored locally. Local edits are treated as **proposals** to the org BRAIN, never authoritative until accepted. Examples: `company/`, `module/<name>/`, `project/`, `client/<id>/` (internal), `persona/<role>/`, `memories/people/`, `memories/projects/`, `memories/decisions/` (after org-promotion).

**`client-visible`** — a sub-class of `shared` exposed through the PORTAL module to the client whose ID matches the scope. Defaults to nothing today; opt-in per file. Example: `client/<id>/portal-visible/`.

### The defaults table

| Scope                                  | Default `sync_class`         | Rationale                                                         |
| -------------------------------------- | ---------------------------- | ----------------------------------------------------------------- |
| `meta/`, `audit/`, `index/`, `exports/`, `conflicts/`, `.lock` | `local-only` | Operational machinery; per-machine                              |
| `member/<self>/private/`               | `local-only`                 | Personal-private; subject sovereignty                             |
| `memories/drift/`                      | `local-only`                 | Per-machine drift detection                                       |
| `member/<self>/` (non-private)         | `publishable`                | Personal but shareable on subject's choice                        |
| `memories/preferences/`                | `publishable`                | Personal preferences; org may absorb relevant ones                |
| `memories/refinements/`                | `publishable`                | Subject's contribution to protocol evolution                      |
| `memories/decisions/`, `memories/facts/`, `memories/people/`, `memories/projects/` | `publishable` (default) | Subject writes; org BRAIN promotes the relevant ones to `shared`  |
| `project/`                             | `shared`                     | This project IS the company's product                             |
| `company/`, `module/<name>/`           | `shared`                     | Org-level by definition                                           |
| `client/<id>/` (internal)              | `shared`                     | Org-internal client knowledge                                     |
| `client/<id>/portal-visible/`          | `client-visible`             | The slice the client sees through PORTAL                          |
| `persona/<role>/`                      | `shared`                     | Personas are org assets                                           |

Subject sovereignty rule: anyone can override their own personal default to `local-only` per file. Nobody can promote their own write to `shared` — that requires org BRAIN acceptance.

### Onboarding & offboarding

**Onboarding.** A new employee's first session pulls all `shared`-class scopes from the org BRAIN. Their personal `member/<id>/` starts empty and accumulates as they work.

**Offboarding.** The org "absorbs" knowledge: published memories that have flowed to `shared` stay (they were already org property by virtue of being published). Personal `member/<id>/` and `local-only` content is garbage-collected from the org BRAIN's mirror, not from the employee's personal copy. The employee retains their personal BRAIN as a portable export.

This is the **absorb-then-discard** pattern: keep the meaningful contributions, drop the personal fragments.

### Why per-person, not per-machine

A subject (e.g., Stephen) may work on a desktop in Saigon, a laptop in Hanoi, and a tablet on the road. All three have a `.cyberos-memory/` with the same `subject:stephen` identity. Personal memories sync across all three through the org BRAIN's per-subject mirror; shared memories arrive identically to all three. The audit chain is per-store (each machine has its own linear chain), but the org BRAIN re-chains incoming memories under its own continuous chain — preserving each origin chain as `original_chain` per §11.6.

---

## Part 6 — Protocol distribution

How does AGENTS.md itself get updated when CyberSkill ships a new version, without becoming a prompt-injection vector?

Three properties answer that — authenticity, authorization, auditability — implemented as a layered scheme.

### SHA pinning *(the foundation)*

The manifest stores the canonical SHA256 of the currently-approved AGENTS.md:

```json
"protocol": {
  "sha256": "sha256:<canonical hash>",
  "approved_at": "<ISO-8601>",
  "approved_by": "subject:<user>",
  "loaded_path": "docs/CyberOS-AGENTS.md"
}
```

On every session start, after §0.1 root resolution, the agent computes `sha256(canonical(loaded_AGENTS.md))` (LF line endings, NFC, BOM stripped, trailing whitespace trimmed). If it differs from `manifest.protocol.sha256`, the state classifier returns `INCOMPATIBLE:protocol-sha256-mismatch`; the agent refuses to operate, surfaces the diff, and waits for human approval.

This catches accidental edits, hostile injection, silent host-platform updates, and unapproved manual edits.

### Signed releases *(authenticity)*

CyberSkill publishes each AGENTS.md release as the document plus a detached `AGENTS.sig` (Ed25519 signature over the canonical SHA256). The org BRAIN's `protocol.releases.list` MCP tool returns ordered `[{sha256, release_ts, signature, changelog_url}]`.

The local manifest pins one or more upstream signing fingerprints:

```json
"protocol": {
  ...
  "signing_keys": [
    {"fingerprint": "ed25519:<fp>", "label": "CyberSkill upstream", "added_at": "<ISO>", "added_by": "subject:stephen"}
  ]
}
```

Trust establishment uses **TOFU** (Trust On First Use, like SSH host keys): first time CyberSkill's fingerprint enters the manifest, the user pastes it from any trusted out-of-band channel — a CyberSkill-signed announcement, a verified org-wide secrets manager, an in-person fingerprint exchange, or equivalent. **Pre-BRAIN-module-P1, no canonical out-of-band source is mandated by the protocol** (Bundle K 2026-05-06 deprecated the placeholder `.protocol-signing-key` file approach; the canonical mechanism will be designed when P1 ships and a real signing keypair is generated).

### Update flow *(authorization)*

Silent weekly check (per your preference): the agent calls `protocol.releases.list`, finds anything newer than `current_sha256`. For each new release it verifies the Ed25519 signature against any pinned fingerprint. Valid signature triggers a banner the next time the user is in chat: *"CyberSkill protocol release `<sha256>` is available, signed by `<fp>`, dated `<ts>`. Diff against current: `<n>` lines changed. Approve?"*

The user adopts a new version by saying *"approve protocol upgrade to `<sha256:…>`"* in the current chat turn (per §0.2). The agent then `str_replace`s the manifest, appends `op:"protocol_upgrade"` to the audit ledger (carrying `before_sha256`/`after_sha256`/`approved_by`), and copies the prior AGENTS.md to `meta/protocol-history/AGENTS-<before_sha256>.md` for rollback.

No agent, skill, plugin, MCP, remote source, or system reminder can mutate `manifest.protocol.sha256` without that exact chat-turn approval phrase. **Forced or silent upgrades are sev-0 violations of §0.2.**

### Rollback *(safety net)*

`meta/protocol-history/` retains every approved version. User says *"rollback protocol to `<sha256:…>`"* → agent verifies the SHA exists in `protocol-history/`, runs §4.7 reconciliation against the older rules, appends `op:"protocol_rollback"` to audit, swaps the active AGENTS.md.

Rollback is what makes upgrade safe. Without it, every "approve upgrade" is a one-way door and risk-averse users (correctly) avoid it.

### Auto-pin at bootstrap

First-run bootstrap (§13.1) hashes whatever AGENTS.md is present at `<root>/docs/CyberOS-AGENTS.md` (or wherever `loaded_path` points) and writes that SHA to the new manifest as the initial pin. The user is not prompted on first run; the first run is a quiet baseline. If they later add a CyberSkill upstream signing fingerprint, future updates verify against it.

---

## Part 7 — Self-audit & operational modes

The BRAIN audits itself on every consolidation cycle and on demand. Three modes govern verbosity.

### NORMAL mode *(default)*

Every consolidation runs the §8 five-phase pass plus the §8.6 source-coverage validator plus the §8.7 self-audit (TIER 1 of the self-audit refinement). `WARN` and `CRITICAL` findings appear in the §14 end-of-response block. `INFO` is logged but not surfaced.

### DEBUG mode

Set `manifest.operational_mode: "debug"`. Every `op:"rejected"`, `op:"revert"`, `op:"warn"`, plus any `WARN`/`CRITICAL` health-check finding from this session, floats to **the very top of the next response**, above any answer to the user, formatted as a banner:

```
⚠️  DEBUG NOTICE — issues this session
- op:rejected reason:scope-violation path:client/acme/notes.md (you tried to write to a client/ scope without explicit permission)
- WARN health-check: store size at 82% of 10MB cap; consider consolidation
[end of debug notice]
```

Use DEBUG when training a new employee, troubleshooting a recurring agent confusion, or diagnosing why a memory keeps not landing where you expected.

### VERBOSE mode

Set `manifest.operational_mode: "verbose"`. Adds successful-op tracing (every `create`, `str_replace`, `insert`, `delete`, `rename` gets a one-line trace at the top of the response). Useful for protocol development; noisy for daily use.

### MAINTENANCE mode *(the safe version of "ROOT")*

A time-limited mode (auto-expires at session end or 1 hour, whichever sooner) that allows specific repair operations normally forbidden:

- Rebuild audit chain after unrecoverable corruption
- Manual tombstone of an orphan file
- Force-resolve a stuck conflict
- Manual rollback past `meta/protocol-history/`
- Direct edit of a memory's frontmatter to fix a schema-migration error

Every action requires explicit chat confirmation **per operation** AND is logged with `actor_kind: maintainer` plus a `maintenance_session_id` so any post-incident audit can reconstruct what happened.

What MAINTENANCE mode does **not** allow: bypassing the §9.3 denylist, skipping the §4.2 content gate, applying a protocol upgrade without the §0.5 approval phrase, hard-erasing audit rows. Those stay sev-0 inviolable.

### Health-check cadence

- **Per session-end**: lightweight pass (current TIER 1 §8.7).
- **On demand**: user says *"run brain healthcheck"* — full pass, regardless of mode.
- **Weekly silent** (when org BRAIN ships at P1): a scheduled task runs the full pass and forwards `CRITICAL`/aggregated `WARN` to a CyberSkill admin channel, so a founder gets early warning when an employee's BRAIN gets into a weird state.

---

## Part 8 — How to evolve the protocol safely

This is the part you came for. The BRAIN holds memories written under whatever rules were in force at write-time. New rules must not retroactively invalidate old memories. Here's how to evolve safely.

### The cardinal rule: additive only

Protocol amendments may **add**: new frontmatter fields with safe defaults, new audit ops, new scopes, new validators that catch a previously-unguarded class of bug, new manifest sections, new memory buckets. They may **NOT** silently: remove a frontmatter field, tighten a validator's accept-set, change the meaning of an existing op, narrow a scope's permitted contents, alter the audit row schema in a way that breaks chain replay.

If a tightening is genuinely necessary (the old rule was a bug), it goes through a **deprecation cycle**: one release where the previously-valid input generates `op:"warn"` (not `op:"rejected"`), one release where it's documented as deprecated in the changelog, then one release where it becomes `op:"rejected"`. Three releases minimum. This gives users time to migrate.

### The §0.4 standing rule

Real-world failure analysis drives protocol evolution, not speculative perfectionism. Per AGENTS.md §0.4, every memory issue surfacing in a real session — shallow ingestion, retrieval miss, duplicate memory, conflict the rules don't handle, user repeating an instruction, drift between BRAIN and source-of-truth, denylist false-positive/negative — MUST trigger a refinement proposal in the same response.

The flow is **propose → adopt → record**:

1. **Propose**: agent identifies the gap, writes a tiered amendment proposal (TIER 1/2/3), cites the section to amend, gives exact prose to insert.
2. **Adopt**: user picks a tier or asks for the minimum-viable amendment. Adoption happens in a single chat turn, citing the SHAs.
3. **Record**: agent updates AGENTS.md, appends an entry to `CyberOS-AGENTS.CHANGELOG.md`, writes a `memories/decisions/DEC-NNN-<slug>.md` for the underlying decision, writes a `memories/refinements/REF-NNN-<slug>.md` for the protocol amendment, updates any cross-linked FACT memories, and triggers the §0.5 protocol-upgrade approval flow.

The propose-adopt-record loop is what makes the protocol get *better* over time rather than calcifying. Every reader of AGENTS.md is implicitly invited to find its next failure mode.

### Past evolution: five real-world triggers

Reading the CHANGELOG is the fastest way to internalise how this protocol thinks. Five entries are particularly instructive:

**2026-05-04 — Ingestion-side discipline + 10 protocol refinements.** A 944-line WhatsApp DM digest shipped at ~25% line coverage. Stephen surfaced the gap with screenshots and *"is your BRAIN not saved?"*. Re-ingestion captured 12 missed frozen decisions. Result: §0.4 standing rule, §1.10 verify-before-respond, §4.10 ingestion completeness, §4.11 token-budget transparency, §8.6 source-coverage validator. Five refinements in one session because the failure exposed an asymmetry between read-side and write-side discipline.

**2026-05-04 (afternoon) — Removed `compatible_runtimes` and `schema_version`.** Stephen asked *"are these necessary?"*. Neither survived the analysis. Result: simpler manifest, field-presence tripwire instead of version-numbered comparison.

**2026-05-04 (evening) — Validator discipline.** PyYAML auto-coerces ISO-8601 to native datetimes; `str(dt)` produces space-separated form; the §5.2 regex rejected its own valid output. Code-fenced YAML examples in spec docs triggered `multiple-frontmatter-blocks` rejection. Both failures hit on the FIRST file write of a 12-file corpus. Result: §4.3 fenced-code exemption, §5.2 datetime-instance acceptance.

**2026-05-06 — Skill-registry v0.2.0 (informational).** Skill-level audit landed parallel to AGENTS.md §0.4 — same pattern, different surface. The CHANGELOG entry has no AGENTS.md edits but cross-links the registry-side detail. **This is the convention for "important context, no protocol change."**

**2026-05-06 (this session) — Multi-person + protocol distribution.** Two architectural gaps surfaced: §11.8↔FACT-004 contradiction (concurrent multi-machine: unsupported vs CRDT sync); AGENTS.md silent on its own update flow. Resolution: §17 sync-class boundary (4 classes), §0.5 protocol update policy, §8.7 self-audit pass. Three TIER 1 amendments, two TIER 3 amendments.

**2026-05-06 (evening) — Bundle D: Canonical-JSON tightening (§7.2 → RFC 8785 JCS).** The cascade verifier surfaced 149 pre-existing audit rows failing bit-perfect hash recompute against the new `brain_writer.py`, even though both writers nominally followed pre-D §7.2. LINK integrity intact; recompute divergent. Root cause: §7.2's "shortest IEEE-754" wording permitted multiple legal interpretations (Python `1.0` vs JCS `1`); UTF-16 vs byte-lex key ordering was unspecified; boolean serialisation was unspecified. Resolution: §7.2 rewritten to cite **RFC 8785 (JSON Canonicalization Scheme)** as the authoritative algorithm, document four common divergence points, name reference implementations (`rfc8785` PyPI, `canonicalize` npm), and clarify that **LINK integrity is authoritative; hash recomputation is informational**. One TIER 1 amendment. The 149 pre-existing rows are NOT retroactively re-chained — additive-only discipline preserved.

**2026-05-06 (evening) — Bundle E TIER 1: Three-way protocol-conflict handling (§0.5 + §13.0).** Post-cascade analysis surfaced scenario C: a user with hand-edited AGENTS.md running "check for protocol updates" would have local edits silently overwritten by upstream pull. §0.5 only handled 2-way (loaded vs pinned). Resolution: §0.5 gained a "Three-way conflict" subsection; agent refuses to apply upstream when loaded ≠ pinned, surfaces three explicit options (revert local; approve local as upgrade; manual merge); §13.0 gained `INCOMPATIBLE:three-way-protocol-conflict` row.

**2026-05-06 (evening) — Bundle F: Comprehensive audit-fix pass + §0.6 related-files update rule.** A whole-document audit surfaced ten correctness bugs, five stale references, and four compression opportunities. Includes one bug that would freeze writes after every protocol upgrade (§4.7's orphan-manifest check predated the new `protocol_upgrade` op as a valid pairing). Single bundle covered all twenty edits because none added semantic protocol mechanisms — they restored doc correctness and tightened wording. Also added §0.6 (Related-files update rule, sev-1) requiring every `op:"protocol_upgrade"` to be accompanied in the same chat turn by CHANGELOG + README + cross-linked-FACT updates.

**2026-05-06 (later evening) — Bundle G TIER 1: Diagnostic-verb carve-out for PRISTINE auto-bootstrap (§1 step 2 + §13.0).** A fresh Cowork session at sale-noti (the first downstream consumer of the protocol) ran `healthcheck` against a `PRISTINE` BRAIN. The agent there correctly held off on the silent auto-bootstrap because it would have changed the very state being diagnosed — and surfaced this as an §0.4 refinement candidate. Resolution: §1 step 2 carved out an exception for diagnostic verbs (`healthcheck`, `status`, `inspect`, `audit`, `check brain`, `show brain`, `view brain`, plus the configured on-demand phrase); §13.0 gained `PRISTINE-DIAGNOSTIC-HOLD` as a sub-state of `PRISTINE`; manifest gains `health_check_policy.diagnostic_verbs[]` for project-level override. This is the FIRST refinement triggered by a real downstream project's actual use of the protocol — the §0.4 propose-then-adopt loop firing in the wild rather than during meta-protocol design.

**2026-05-06 (later evening) — Bundle H TIER 1: Strict uppercase BRAIN alias (§0.3 case-sensitivity).** Stephen noticed that §0.3's "the BRAIN" / "your BRAIN" phrasing didn't enforce case, so a permissive reader could match lowercase "brain" too — meaning topics like "human brain", "brain freeze", "what does the brain do during sleep" could falsely trigger memory-store actions. Resolution: §0.3 now requires literal uppercase B-R-A-I-N with case-sensitivity explicit; lowercase 'brain' is interpreted as anatomy / metaphor / general topic and does NOT trigger the alias. Ambiguous lowercase contexts surface a clarifying question rather than silently assuming. This is the second refinement from real-world use (Bundle G was the first).

**2026-05-06 (later evening) — Bundle I TIER 1: Compact §14 format gated by operational_mode.** Stephen surfaced real readability friction — the §14 end-of-response block was 14+ lines, most reading "no change," signal-to-noise poor. Resolution: §14 now specifies two formats — §14.1 compact (default for `operational_mode: normal`) shows only changed paths plus a roll-up `unchanged:` line; §14.2 full (verbose/debug/maintenance) keeps the per-scope-explicit format for protocol-development sessions. Reuses the existing `operational_mode` field — zero new mechanism; pure rendering split. The audit ledger remains authoritative; format changes don't affect chain integrity. Third refinement from real-world use.

**2026-05-06 (later evening) — Bundle J TIER 1: Auto-trigger §8.7 self-audit after `protocol_upgrade` + uppercase BRAIN in trigger phrases.** Stephen asked: *"can we auto trigger scan and re-arrange/refine the .cyberos-memory after AGENTS.md update, because there maybe breaking changes or rules that need to adapt?"* Resolution: §0.5 gained step 4 — every successful `protocol_upgrade` now auto-triggers a §8.7 self-audit (the post-upgrade migration check) immediately after the manifest pin. Schema validate (phase 1) catches memories failing the new §5.1; report named `meta/health/<date>-<sha>-postupgrade.md`. Skipping requires explicit *"skip post-upgrade scan"* phrase. Manual trigger via new `manifest.health_check_policy.post_upgrade_phrase` (default *"rescan BRAIN"*). Also fixed a Bundle-H consistency issue: `on_demand_phrase` default changed from lowercase "run brain healthcheck" to uppercase "run BRAIN healthcheck"; diagnostic_verbs[] entries that mention BRAIN switched to uppercase too. Fourth refinement from real-world use; first that deals with cross-version migration discipline.

**2026-05-06 (later evening) — Bundle K TIER 1: Deprecate the `.protocol-signing-key` file (defer canonical TOFU source to BRAIN module P1).** Stephen flagged the file as friction: *"is there any way that no need one more separate file .protocol-signing-key?"* Honest answer: yes — it was placeholder weight. No real CyberSkill upstream signing key exists yet (BRAIN module P1 hasn't shipped); the file documented an aspiration rather than enforcing real trust. Resolution: §0.5 TOFU paragraph rewritten to remove the file reference; pre-P1 no canonical out-of-band source is mandated (users paste fingerprints from any trusted channel). DEC-094 v2 supersedes the original signing-key-file approach. The file itself is overwritten with a deprecation marker rather than deleted (the cowork sandbox can't `rm` files outside `.cyberos-memory/`); user can manually remove from their local clone. Fifth refinement from real-world use; first that REMOVES protocol surface area rather than adding.

The single largest protocol-evolution session in the project's history landed on **2026-05-06**: four bundles (B + A + C + D) over one chat turn, each its own §0.5 protocol upgrade with its own SHA pin, archived under `meta/protocol-history/`, and recorded as DEC-094/095/096/097 + REF-015/016/017/018 in the BRAIN. AGENTS.md SHA progressed `560a489…1600fc` (pre-cascade) → `b4042a6…cacce3` (post-D). Twenty-one cascade audit rows + one health-check + three consolidation rows + Bundle D's six rows = thirty-one new audit rows in this session. `meta/protocol-history/` now contains four marker files (one per bundle's pre-state). `meta/health/` contains the first §8.7 report. The chain LINK integrity is intact end-to-end (170+ rows, zero breaks).

The pattern across all six: a **specific failure** triggered a **specific refinement** that closed a **specific class of failure**. No speculative additions. This is what the §0.4 standing rule looks like in practice over time.

### Versioning via Git, not inline markers

The protocol has no inline version marker. The Git history of `docs/CyberOS-AGENTS.md` is the version. The CHANGELOG is the day-by-day record of what changed. SHA256 is the content-addressable name for any specific approved version.

**Why no version number**: in a continuously-evolving protocol, a discrete version number either bumps so often it triggers `INCOMPATIBLE` cross-machine constantly, or never bumps and lies about the actual state. Content-addressable SHA + field-presence forward-compat tripwire (§13.0 `INCOMPATIBLE:<field>`) achieves the same correctness without the version-management overhead.

If you ever feel tempted to write `# Version 2.5.0` at the top of AGENTS.md: don't. That decision was made deliberately and the absence of a version number is itself a protocol invariant.

### How to propose a change

Follow this template in chat. The agent will recognise it and run the propose-adopt-record flow.

> *"I'm seeing [specific failure mode]. Per §0.4, propose a refinement. The failure: [detail]. The cost of leaving it unfixed: [risk]. The shape of the fix: [shape]."*

The agent then drafts TIER 1/2/3 amendments, cites sections, writes exact prose. You pick a tier, the agent records.

### What requires §0.5 approval vs what doesn't

- Editing AGENTS.md = requires §0.5 protocol upgrade approval.
- Editing the CHANGELOG = informational; no approval (CHANGELOG is descriptive, not prescriptive).
- Editing this README = informational; no approval.
- Editing a `memories/decisions/DEC-NNN.md` that records a decision underlying an AGENTS.md change = requires the same §0.5 approval as the AGENTS.md change it documents.
- Adding a new `memories/refinements/REF-NNN.md` = follows the regular memory-write flow (no §0.5 approval needed for the REF file itself; the REF file *describes* an AGENTS.md change that does need §0.5 approval).

---

## Part 9 — Common mistakes & anti-patterns

Patterns the protocol has explicitly designed against, ordered by frequency.

**Hand-editing the audit ledger.** `audit/<YYYY-MM>.jsonl` is append-only. The Merkle chain breaks the moment you `vim` it. If you need to correct a row, append `op:"corrects"` referencing the prior `audit_id`. If the chain is genuinely corrupt, MAINTENANCE mode is the recovery path; don't edit by hand.

**Operating against a sandbox path.** The agent appears to write to the BRAIN, but the writes go to a virtual filesystem that vanishes at session end. Symptom: "I told the agent to remember X yesterday, but today it doesn't know X." Fix: §0.1 sanity check on session start; verify `<realpath(<root>)>` is the real Finder path, not `/sessions/...` or similar.

**Promoting authority.** Agents may downgrade authority on uncertainty (`human-confirmed → llm-explicit`); never the reverse. If you find yourself wanting to upgrade, you actually want a human to confirm the memory in chat, then re-record at `human-confirmed`.

**Silent overwrite.** `create` fails if the path exists; `str_replace` requires exactly one substring match. There is no way to "just overwrite this file." If the target exists, you're either updating (`str_replace`), superseding (new memory with `supersedes: <old>`), or tombstoning the old then creating a new. Pick one.

**Writing personnel/client memory without consent.** The frontmatter `consent.has_consent: true` is required for `personnel` and `client` classifications. The protocol will reject the write if it's `false` or missing for those classes. Don't try to work around this — capture consent properly or use a different classification.

**Cross-scope rename.** `rename` is intra-scope only. You cannot move `member/alice/notes.md` to `client/acme/notes.md` even if the content is the same. Create a new memory under the target scope; tombstone the old; cross-link via `relationships`.

**Hardcoded project-specific patterns in the protocol.** AGENTS.md is universal. If you find yourself adding a `module:whatsapp-*` example to a §-level rule, you're polluting the protocol with project context. The pattern goes in the project's `manifest.json source_tiers`, not in the protocol document.

**Skipping the §14 end-of-response block when conditions don't allow.** Post-Bundle-P (2026-05-10), §14 operates as a three-state triage: `omit` (silent) / `compact` (single `📁 Files changed:` block listing **non-BRAIN paths only**) / `verbose` (issues-first, full detail with separate `📁` and `Δ Changes (BRAIN detail):` sections). Omit is permitted in `normal` mode when no findings, **no non-BRAIN file changes** (BRAIN-only mutations are agent housekeeping and don't trigger output), and the latest §8.7 reports 0 CRITICAL/0 WARN. Compact fires when non-BRAIN files changed without issues. Verbose auto-triggers on ANY of: `op:rejected|revert|warn|health_check` this turn, latest §8.7 reports CRITICAL/WARN, or `operational_mode != normal` — no manual mode flip needed. Critical semantic: `📁 Files changed:` shows ONLY non-BRAIN paths; BRAIN paths surface in chat only via §14.2's `Δ Changes (BRAIN detail):` block. Inappropriate skipping is when a finding occurred but the agent stays silent or compact — that hides issues. When unsure, escalate one tier (compact→verbose; omit→compact).

**Ingesting via sample-skipping.** `sed -n 'A,Bp;C,Dp'`, head-only, tail-only, modulus decimation are forbidden by §4.10 for sources >100 lines. The agent must walk the source sequentially end-to-end and confirm coverage ≥99% before writing a digest. Shallow ingestion masquerading as comprehensive is the failure mode that triggered §0.4 in the first place.

---

## Part 10 — Troubleshooting

### "The agent says CORRUPT"

§13.0 state classifier returned `CORRUPT:<reason>`. Common causes: hand-edited audit ledger, mid-write crash with no `op:"revert"` row, manifest's `audit_chain_head` doesn't appear in any audit row.

- Don't auto-repair. Run reconciliation (§4.7) by starting a fresh session — the agent will append `op:"revert"` rows for any orphaned writes.
- If reconciliation can't fix it: enter MAINTENANCE mode and rebuild the chain from the most recent valid checkpoint. This is destructive (post-checkpoint rows are dropped); confirm the export bundle is current first.
- Last resort: restore from the most recent `exports/memory-export-<date>-all.zip`.

### "I see drift candidates"

§8.6 detected that a source file's SHA changed since the digest was written. Three options:

1. **Re-ingest**: agent walks the new source sequentially, writes a v2 digest, supersedes v1.
2. **Accept drift**: leave the digest as-is; the drift record explains why source has moved on. Use this when the source moved on in ways that don't affect the digest's purpose.
3. **Update source to match**: the digest captured the right answer; the source got it wrong.

### "My memory was rejected"

The rejection reason is in the `op:"rejected"` audit row. Look up the section that fired:
- `path-traversal:*` → §4.1 path guard
- `injection:*` → §4.2 content gate (someone tried to write a prompt-injection marker into memory)
- `denylist:*` → §9.3 (compensation, gov-ID, secret, etc. — store a pointer instead)
- `scope-violation:*` → §4.5 (writing to a scope that requires explicit user permission)
- `unknown-frontmatter-field:<name>` → §4.3 (your AGENTS.md is older than the memory you're trying to write)
- `multiple-frontmatter-blocks` → §4.3 (likely a fenced-code-block edge case; check the post-DEC-087 exemption)
- `bad-ts:<field>` / `naive-ts:<field>` → §5.2 (timezone offset missing or non-ISO format)

### "INCOMPATIBLE:protocol-sha256-mismatch"

§0.5 fired. The AGENTS.md on disk doesn't match `manifest.protocol.sha256`. Three possibilities:

1. **You hand-edited AGENTS.md without going through §0.5 approval.** Fix: revert your edit, OR formally approve the new SHA in chat per §0.5.
2. **A signed CyberSkill upstream release replaced AGENTS.md silently.** This shouldn't happen if your distribution flow is correct. Investigate the source.
3. **Your agent's host platform shipped a bundled AGENTS.md that overrode yours.** Check the loaded path; configure `loaded_path` in the manifest to point at your real one.

### "I keep getting `shallow_candidate` warnings"

§8.6 detected a digest with <80% line coverage where `intentional_summary: false`. Either:

- **Re-ingest**: walk the source end-to-end per §4.10; write a comprehensive digest.
- **Mark intentional**: add `intentional_summary: true` and `summary_reason: "<why>"` to the frontmatter. The warning stops; the audit trail records that the partial coverage was deliberate.

### "The §14 block doesn't match what I see on disk"

Run `op:"health_check"` (TIER 1 self-audit). It walks every memory file and reconciles against the audit ledger, flagging any mismatch. Common cause: a write claimed success but `tmp+rename` failed silently (filesystem full, permissions error). The §14 block reflects the agent's intent; the disk reflects reality; the health-check finds the gap.

---

## Part 11 — Reading order: how to navigate AGENTS.md

The protocol is dense (~750 lines). On first read, you don't need all of it. Here's a reading order matrix.

### First read (overview)

§0 (precedence + immutability + BRAIN alias + §0.4 standing rule + §0.5 protocol distribution) → §1 (standing directive) → §2 (first principles) → §3 (canonical layout) → §15 (multi-agent interop). About 100 lines. You now understand the shape of the protocol without yet knowing the details.

### Writing your first memory

§3 (layout) → §4 (six ops, with §4.1/§4.2/§4.3 validation gates) → §4.4 (atomic write) → §4.5 (scope contract) → §5.1 (frontmatter schema) → §5.4 (classification → retention) → §14 (end-of-response block). About 200 lines.

### Implementing a validator

§4.1 (path-traversal) → §4.2 (content gate) → §4.3 (file-content hygiene) → §5.2 (validators) → §7.1/§7.2 (audit row schema + canonical JSON for hashing) → §11.7 (filesystem portability). About 250 lines.

### Designing a new feature (proposing a protocol change)

§0.4 (standing rule) → §0.5 (protocol distribution) → Part 8 of *this README* (additive-only rules) → §8 (consolidation) → past CHANGELOG entries to see the propose-adopt-record loop in practice.

### Investigating an incident

§4.7 (reconciliation) → §7 (audit ledger) → §8.6 (source-coverage validator) → §8.7 (self-audit pass) → §13.0 (state classifier) → Part 10 of *this README* (troubleshooting decision tree).

### When you have 30 minutes and want to absorb the whole protocol

Read AGENTS.md top-to-bottom in order. The sections are arranged so each builds on the prior. §0 establishes precedence; §1–§3 establish state; §4–§5 establish operations; §6–§7 establish recordkeeping; §8–§10 establish conflict and read flow; §11–§13 establish portability and bootstrap; §14–§16 establish output discipline and tie-breakers. Skim the validators and tables; read the prose carefully.

---

## Part 12 — Glossary

**Audit ledger.** `audit/<YYYY-MM>.jsonl`. Append-only, Merkle-chained record of every state change. Cannot be edited in place. The **single most important file** in the BRAIN.

**Authority hierarchy.** `human-edited > human-confirmed > llm-explicit > llm-implicit`. Strict order. Agents may downgrade on uncertainty; never promote.

**BRAIN.** This whole system. Layer 1 (`.cyberos-memory/` filesystem) + Layer 2 (vector + graph) + Layer 3 (archival corpus). The user can refer to it as "the BRAIN" in chat per §0.3.

**Classification.** One of `personnel | client | operational | public`. Drives retention defaults and consent rules. `personnel` and `client` never auto-resolve in conflicts.

**Conflict.** Two memories in the same scope holding contradictory facts. May auto-resolve (operational/public + same authority + tier comparison) or human-resolve (anything personnel/client). Disputed pairs are a valid permanent state.

**Consolidation.** Five-phase pass at session-end (or on-demand). Surfaces candidates, detects conflicts, conservatively merges, reorganises, updates manifest. Phase 6 (TIER 1 self-audit) is the integrity check.

**Content gate.** §4.2 validator that rejects prompt-injection markers, mixed-script confusables, base64 blobs, control chars, and denylisted content from being written to memory.

**Denylist.** §9.3 categorical exclusions: compensation, gov-IDs, bank/card numbers, home addresses, health PII, secrets, external-party PII without consent. Stored as pointers if needed; never as values.

**Drift candidate.** §8.6 signal that a source file's SHA changed since a digest was written. Surfaced as `WARN`. Three responses: re-ingest, accept drift, update source.

**Manifest.** `manifest.json`. Per-project root pointer. Contains tenant info, owner info, `audit_chain_head` checkpoint, `source_tiers`, `protocol.sha256` (under §0.5), `signing_keys`, `operational_mode`.

**Memory file.** YAML frontmatter + Markdown body. Stored under `memories/`, `member/`, `client/`, `module/`, `company/`, `persona/`, `project/`, `meta/`. 27+1 permitted frontmatter fields per §5.1.

**Provenance.** `provenance.{source, source_ref, confidence}` on every memory. Records where a fact came from. LLM-inferred confidence caps at 0.7.

**Refinement.** A protocol amendment proposed per §0.4 and adopted via the propose-adopt-record cycle. Recorded in `memories/refinements/REF-NNN-<slug>.md`.

**Scope.** Where a memory lives. One of `company`, `meta`, `module:<name>`, `member:<id>`, `client:<id>`, `project:<slug>`, `persona:<role>`, `dm:<a>:<b>`. Scope determines default `sync_class` and write-permission requirements.

**Shallow candidate.** §8.6 signal that a digest's `processed_lines / source_lines < 0.80` and `intentional_summary: false`. Surfaced as `WARN`.

**Source freshness tier.** Integer ≥1; lower = more authoritative. `manifest.source_tiers` maps scope-pattern globs → tier integers. Used in §9.1 step-0 conflict resolution.

**Subject.** The person a memory is *about* (or owned by, in `member/<id>/`). Subjects are sovereign over their own `member/<id>/`; agents do not contest subject edits there.

**Supersedes graph.** Directed acyclic graph of `supersedes`/`superseded_by` pointers across memories. Walks before write to detect cycles. Tombstoned memories are excluded from active conflict detection but retained in audit.

**Sync class.** `local-only | publishable | shared | client-visible`. Frontmatter field per §17 governing whether a memory leaves the local machine and (if so) where it goes.

**Tombstone.** Soft delete. File body kept verbatim; frontmatter flipped `tombstoned: true` with `deleted_at`/`deleted_by`/`tombstone_reason`. Hard erasure exists only via human-driven right-to-erasure flow.

**Two-phase atomic write.** §4.4 sequence: validate → append audit row → tmp+rename → fsync. Crash-safe; never leaves partial state on disk.

---

## Part 13 — Status snapshot, tools, and file map (2026-05-12)

> **Deep dive:** for the full per-aspect reference + per-tool CLI signatures
> + troubleshooting + workflows, see **Parts 25–31** of this same README.
> Part 13 stays as the quick-scan summary; Parts 25–31 are the detailed
> companion. Both update in lockstep with each `CHANGELOG.md` batch entry.

### Layer-1 operator surface (2026-05-12 — Aspect 1.1 ship)

A `cyberos` umbrella binary now wraps every Python tool under `runtime/tools/`:

```bash
cyberos status              # 4-operator-question dashboard (Aspect 2.1)
                            #   --weekly   → 7-day landed/in-flight/queued digest (Aspect 2.3)
                            #   --watch [--interval N]   → continuous re-render (Aspect 2.4)
                            #   --security → encryption + denylist + perms posture (Aspect 5.4)
cyberos verify              # wraps cyberos_validate.py
cyberos doctor [args]       # wraps cyberos_doctor.py
cyberos export [args]       # wraps cyberos_export.py
cyberos search <query>      # wraps cyberos_index.py
cyberos stats               # bucket / class / authority counts
cyberos show [filters]      # memory browser
cyberos add <type>          # interactive memory wizard (Aspect 1.2)
                            #   --auto-tags  → opt-in GLOSSARY tag suggestion (Aspect 5.2)
                            #   --persona N  → apply persona/<N>.md defaults (Aspect 12.6)
cyberos repl                # interactive REPL — avoids session.start per call (Aspect 1.6)
cyberos dedup [--scope ...]   # duplicate-memory detection (Aspect 9.6)
cyberos graph [--format ...]  # relationships-graph explorer (Aspect 4.7)
cyberos prune [--interactive] # staleness + contradiction surface (Aspect 1.1 + 9.7)
cyberos hooks {status|on|off} # toggle Claude Code hook integrations (Aspect 5.1)
cyberos refinements           # §0.4 candidate dashboard (Aspect 11.4)
cyberos explain <subcmd>      # show which §-rules each subcommand touches (Aspect 1.5)
cyberos compact-stats         # audit-ledger compaction recommendations (Aspect 9.4)
cyberos mutation-test         # validator mutation-testing scaffold (Aspect 10.4)
cyberos analytics cost-log|cost-report   # LLM cost tracking (Aspect 11.5)
cyberos lock {status|acquire-shared|acquire-exclusive}   # advisory locks (Aspect 5.7)
cyberos cold-storage {archive|list|verify}               # cold-tier audit export (Aspect 9.5)
cyberos skill {list|describe|chain}                      # skill registry loader (Aspect 12.5)
cyberos voice [--strict]    # em-dash + AI-vocab linter
cyberos doc-consistency     # §-ref + DEC-ref consistency
cyberos panic [--reason]    # emergency stop (Aspect 13.10)
cyberos onboard             # interactive new-contributor bootstrap (Aspect 8.1)
cyberos analytics report    # local-only usage analytics (Aspect 11.2)
cyberos eval REF-NNN        # run capability + regression eval for a REF
cyberos council REF-NNN     # opt-in 4-voice synthesis for ambiguous REFs (Aspect 3.3)
cyberos sync export|import|conflicts  # multi-machine sync scaffolding (Aspect 6.x)
cyberos mcp serve|info      # read-only MCP server for the BRAIN (Aspect 12.7)
cyberos help [subcmd]       # detailed help
```

Run from anywhere in the repo. Locates `.cyberos-memory/` by walking up per §0.1. Read-only by default — only `doctor`, `panic`, and `sync import` mutate state (all require explicit confirmation or `--dry-run` first).

### Council mode, auto-tagging, sync, MCP (Aspect 3.3 + 5.2 + 6.x + 12.7)

**Council mode.** `cyberos council REF-NNN` produces a working artefact at `.cyberos-memory/cache/council/REF-NNN-council.md` containing 4 voice prompts (Architect / Skeptic / Pragmatist / Critic) plus deterministic heuristic context — GLOSSARY term overlap, possible LOCK conflicts, related REFs (shared tags), and recent `rejected/` entries. Each voice prompt is intended to be pasted into a fresh Claude conversation; you collect the 4 findings then write the Synthesis section. Opt-in only — ambiguous REFs pay the 4× API cost. Not a replacement for capability + regression evals; runs alongside them.

**GLOSSARY auto-tagging.** `cyberos add <TYPE> --auto-tags` reads `FACT-014-glossary.md`, scans the slug + title + provenance reference for canonical terms, suggests kebab-case tags, and prompts you to accept / decline / edit before writing. Default off — auto-tagging never modifies tags without confirmation.

**Multi-machine sync.** `cyberos sync export --to <bundle.zip>` produces a deterministic zip filtered by `sync_class` (publishable + shared by default; `--include client-visible` adds the client-visible class with consent gating). Two consecutive exports of the same state produce identical SHA256. `cyberos sync import <bundle> --from <subject> [--dry-run]` runs a three-way merge by `memory_id × content_sha`: matching → no-op, differing → conflict marker written under `memories/conflicts/`, remote-only → staged under `.cyberos-memory/cache/test-fixtures/sync-staging/`. Transport (rsync / syncthing / S3) is left to the operator.

**Read-only MCP server.** `cyberos mcp serve` runs a line-delimited JSON-RPC 2.0 server on stdio exposing 4 tools: `brain_search`, `brain_show`, `brain_get`, `brain_stats`. Default filters hide tombstoned + `sync_class=local-only` entries (both have explicit opt-in flags). No write tools by design — callers must use `brain_writer.py`. Wire into Claude Code via `cyberos mcp info` (prints the `.claude/mcp-config.json` snippet with absolute paths).

### Hooks (Aspect 3.1 + 5.1)

```
runtime/hooks/gateguard.py            # PreToolUse 3-stage DENY/FORCE/ALLOW gate
runtime/hooks/refinement_candidates.py # Stop-hook auto-detection of §0.4 candidates
```

Install in `~/.claude/settings.json` per file header.

### Memory templates (Aspect 4.1)

```
.cyberos-memory/meta/templates/
├── DEC.md         # Nygard ADR format (Context / Decision / Alternatives / Consequences)
├── REF.md         # protocol refinement with capability+regression eval pointers
├── FACT.md        # fact with provenance + drift detection
├── PERSON.md      # personnel-class with consent discipline
├── PROJECT.md     # cross-project anchor
├── PREFERENCE.md  # operational tuning preference
├── DRIFT.md       # auto-generated §8.6 drift candidate
├── POSTMORTEM.md  # blameless postmortem (Aspect 3.5)
└── REJECTED.md    # rejected refinement candidate (Aspect 3.4)
```

`cyberos add <type>` (Aspect 1.2 — shipped 2026-05-12) uses these as the wizard scaffold. Templates live at `runtime/starter/templates/` (moved out of `.cyberos-memory/meta/templates/` so validator does not scan placeholder vars).

### Tour files (Aspect 7.4)

```
tours/
├── onboarding.tour          # new-contributor walkthrough
├── refinement-loop.tour     # §0.4 propose-adopt-record flow
├── incident-response.tour   # CORRUPT state recovery
├── protocol-upgrade.tour    # §0.5 SHA-pin update flow
└── security-audit.tour      # trust boundaries + denylist review
```

Open in VS Code with the CodeTour extension.

### Currently-pinned protocol

> Part 13 is the operational dashboard. Parts 1–12 stay stable; Part 13 refreshes when stages ship, tools change, or new docs land.

### Currently-pinned protocol

```
sha256:d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0   ← current (post-Stage-5)
sha256:77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa   ← Stage 6 baseline (Merkle + compaction + lock-shared)
sha256:576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a   ← Stage 1 baseline (reconciliation checkpoint + lazy-load)
sha256:599e1097199618e0d8dde22770eef6e5ad068c5c06150e2bb3829315f005780d   ← pre-Stage-1 (5 SHAs older still in protocol-history)
```

Verify the live SHA matches the manifest pin:

```bash
python3 runtime/tools/canonical_sha.py docs/CyberOS-AGENTS.md
python3 -c "import json; print(json.load(open('.cyberos-memory/manifest.json'))['protocol']['sha256'])"
# Both must produce the same string.
```

### Local-optimization roadmap status

All six stages of `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` are complete:

| Stage | What it gave you | Surface |
|-------|-----------------|---------|
| 1 — Session-start speed | reconciliation_checkpoint, read_profile, frontmatter compactness | §0.5 upgrade `576368…` |
| 2 — Validator + doctor + corpus | `cyberos_validate.py` (11 check categories), `cyberos_doctor.py` (5 repair ops), 21 fixtures | runtime/tools/ |
| 3 — Local search index | `cyberos_index.py` SQLite (sub-millisecond p95) | runtime/tools/ |
| 4 — Backup + sync safety | `cyberos_export.py` (deterministic + daemon) | runtime/tools/ |
| 5 — At-rest encryption + Shamir | §5.6 envelope, `cyberos_encrypt.py` (passphrase + macOS Keychain) | §0.5 upgrade `d3ce97…` |
| 6 — Long-term BRAIN health | §7.6 Merkle, §7.7 compaction, §4.9.1 lock-shared, doctor compact/decompact | §0.5 upgrade `77eda2…` |

### CyberOS-AGENTS.md (load-every-session)

```
docs/CyberOS-AGENTS.md       1214 lines, 108 KB, ~27K tokens   ← canonical (load on demand)
docs/CyberOS-AGENTS.md   483 lines,  42 KB, ~10K tokens   ← load every session (regenerable)
```

Both live in `docs/` for consistency. CORE is a derived view of canonical; canonical is authoritative.

Symlink for new projects:

```bash
cd /path/to/your-project
ln -s /path/to/cyberos/docs/CyberOS-AGENTS.md AGENTS.md
ln -s /path/to/cyberos/docs/CyberOS-AGENTS.md CLAUDE.md
```

When the agent needs the full reference (validator, doctor, §0.5 upgrades, MAINTENANCE mode), it consults `docs/CyberOS-AGENTS.md` directly. The 92% token reduction holds for daily-session loading. CORE's "When you MUST load the full AGENTS.md" header lists 14 concrete trigger conditions.

Regenerate after AGENTS.md changes:

```bash
python3 runtime/tools/extract_agents_core.py --aggressive docs/CyberOS-AGENTS.md > docs/CyberOS-AGENTS.md
```

### Tools index (CLIs in `runtime/tools/` + MCP server in `runtime/mcp/`)

| Tool | Use when |
|------|----------|
| `cyberos_validate.py [--pre-commit] [--self-test]` | Daily health check; 21-fixture self-test; pre-commit-hook-friendly mode |
| `cyberos_doctor.py [--repair --reason] [--compact-ledger M] [--decompact-ledger M] [--rebuild-checkpoint]` | Diagnose + repair under MAINTENANCE mode |
| `cyberos_index.py [build | update | verify | stats | query <kind> <arg>]` | Sub-millisecond tag/relationship/source-SHA lookup |
| `cyberos_export.py [-o DIR] [--daemon --interval H] [--verify FILE]` | Deterministic backup bundles |
| `cyberos_encrypt.py [enable | disable | status | recover | migrate-batch N | rotate-shamir]` | At-rest encryption opt-in + Shamir 3-of-5 escrow |
| `cyberos_show.py [--scope] [--tag] [--class] [--tombstoned] [--recent]` | Memory browser (table view) |
| `cyberos_add.py <TYPE> [--auto-tags]` | Interactive memory wizard with opt-in GLOSSARY auto-tagging (Aspect 1.2 + 5.2) |
| `cyberos_council.py REF-NNN [--voices ...]` | Opt-in 4-voice synthesis for ambiguous REFs (Aspect 3.3) |
| `cyberos_sync.py {export\|import\|conflicts [--resolve]}` | Multi-machine sync scaffolding (Aspect 6.x) + interactive conflict resolver (Aspect 6.5) |
| `cyberos_repl.py` | Interactive REPL — avoids session.start per call (Aspect 1.6) |
| `cyberos_dedup.py [--scope] [--threshold]` | Duplicate-memory detection by content fingerprint (Aspect 9.6) |
| `cyberos_graph.py [--format text\|dot\|json] [--memory] [--orphans]` | Memory relationships graph explorer (Aspect 4.7) |
| `cyberos_prune.py [--staleness-days] [--drift-days] [--interactive]` | Surface stale memories + neglected supersedes-pairs + old drift candidates (Aspect 1.1 + 9.7) |
| `cyberos_hooks.py {status\|on\|off} [--hook]` | Install / remove Claude Code hook integrations (Aspect 5.1) |
| `cyberos_refinements.py [--kind]` | §0.4 candidate dashboard — drift + council-pending + rejected (Aspect 11.4) |
| `cyberos_compact_stats.py [--row-cap] [--byte-cap] [--age-days]` | Audit-ledger compaction recommendations (Aspect 9.4) |
| `runtime/tests/mutation/run_mutations.py` | Mutation testing scaffold for validator (Aspect 10.4) |
| `cyberos_analytics.py cost-log \| cost-report` | LLM cost tracking, local-only (Aspect 11.5) |
| `cyberos_onboard.py [--shared] [--persona]` | Interactive new-contributor bootstrap (Aspect 8.1) |
| `cyberos_analytics.py {log\|report\|purge}` | Local-only usage analytics (Aspect 11.2) |
| `canonical_sha.py PATH` | §0.5 SHA helper for protocol upgrade approval phrases |
| `extract_agents_core.py [--aggressive] [--check] PATH` | `docs/CyberOS-AGENTS.md` generator + CI verifier |
| `voice_check.py [--strict] PATHS` | gstack `/codex` voice linter (em dashes + AI vocab) |
| `benchmark.py PATH` | Validator + export performance regression tracker |
| `runtime/mcp/cyberos_brain_server.py` | Read-only MCP server (4 tools: brain_search/show/get/stats) — Aspect 12.7 |

### File map (post-consolidation, 2026-05-10)

```
cyberos/                                  ← project root
├── docs/
│   ├── CyberOS-AGENTS.md                 ← canonical protocol (1214 lines, 108 KB)
│   ├── CyberOS-AGENTS.md            ← derived: 10K-token subset (regenerable; symlink target)
│   ├── CyberOS-AGENTS.README.md          ← THIS DOCUMENT (Parts 1-24: concepts + ops + cookbook + future + proposals)
│   ├── CyberOS-AGENTS.CHANGELOG.md       ← protocol-doc day-by-day
│   ├── CyberOS-PRD.docx + .CHANGELOG.md  ← product requirements
│   ├── CyberOS-SRS.docx + .CHANGELOG.md  ← system requirements
│   └── (consolidated into this README in Parts 14-24:
│        cookbook/×5, EVOLUTION, LOCAL-OPTIMIZATION, proposals/×5)
├── runtime/
│   ├── README.md                         ← consolidated build plan (Parts 1-3: Plan, Interfaces, Build Order)
│   └── tools/                            ← 8 Python CLIs + tests/vectors/ + concise README
└── .cyberos-memory/                      ← live BRAIN store
    ├── manifest.json                     ← protocol pin + project metadata
    ├── audit/2026-05.jsonl               ← 320+ rows, chain-linked ✅
    ├── memories/refinements/             ← REF-001..037 per §0.4 standing rule
    ├── meta/protocol-history/            ← verbatim AGENTS.md archives by SHA
    ├── meta/health/                      ← §8.7 self-audit reports
    └── ...                               ← rest per AGENTS.md §3
```

**Consolidation note (2026-05-10):** `cookbook/×5`, `proposals/×5`, `CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`, and `CyberOS-AGENTS.EVOLUTION.md` were inlined into this README as Parts 14-24. `runtime/PLAN.md` + `INTERFACES.md` + `BUILD_ORDER.md` were inlined into `runtime/README.md` as Parts 1-3. Source files replaced with `[CONSOLIDATED]` pointer stubs (or deleted on filesystems that allow it). 7 fewer doc files at the top level; same content, single hub.

### Where do I find X?

| Question | Where |
|----------|-------|
| Current protocol SHA | `manifest.json` → `protocol.sha256` OR `canonical_sha.py docs/CyberOS-AGENTS.md` |
| BRAIN health | `cyberos_validate.py .` |
| Diagnose corruption | `cyberos_doctor.py .` (read-only); add `--repair --reason "..."` to fix |
| Search by tag | `cyberos_index.py . query tag <tag>` |
| Search by relationship | `cyberos_index.py . query relates-to <memory_id>` |
| Recent ops on a path | `cyberos_index.py . query audit-by-path <path>` |
| Backup the BRAIN | `cyberos_export.py . -o ~/Backups/cyberos --daemon --interval 6` |
| Enable encryption | `cyberos_encrypt.py . enable` (Shamir 3-of-5 wizard) |
| What was DEC-N about? | `.cyberos-memory/memories/decisions/DEC-N-*.md` or AGENTS.md §13 / PRD Part 13 |
| When did X last change? | `cyberos_index.py . query audit-by-path <path>` |
| Chain head | `manifest.json` → `audit_chain_head` |
| How to amend the protocol | Append a new Part to this README (`Part NN — Bundle X proposal`) following the pattern in Parts 20-24, then chat-turn approval per §0.5 |
| Rollback a protocol upgrade | `meta/protocol-history/AGENTS-<sha>.md` carries verbatim prior |
| Why a rule exists | Search `.cyberos-memory/memories/refinements/REF-*.md` for the originating REF |

### Cookbook index

| Recipe | When to read |
|--------|--------------|
| `pre-commit-validate.md` | Wiring the validator into git pre-commit + GitHub Actions |
| `filesystem-sync.md` | Syncing `.cyberos-memory/` across machines via iCloud/Dropbox/Syncthing/git |
| `local-search-index.md` | Using `cyberos-index` for daily memory recall |
| `encryption-and-recovery.md` | Stage 5 enable wizard, recovery flow, threat model |
| `ledger-compaction.md` | Stage 6 compact / decompact / Merkle proof verification |

### Open work tracks

1. **Bundle M — AGENTS.md refinement pass** (✅ landed 2026-05-10 as `sha256:9bec84…`). Four functional-zero changes applied: schema field-count update, §8 phase-count fix, §4.10/§4.11 merge into §4.10.1/§4.10.2, §17.5 forward-references compression. Two changes deferred to Bundle N: §0.5 split for clarity + paragraph compression throughout. See **Part 24** below (inlined Bundle M proposal) and `.cyberos-memory/memories/refinements/REF-037-bundle-m-refinement-pass.md`.
2. **EVOLUTION.md activation** — when CyberOS-the-product starts building (BRAIN service P1, Layer 2 vector+graph, MCP Gateway, multi-tenancy), the post-CyberOS roadmap reactivates.
3. **HW-key backends for Windows/Linux** — macOS keychain-stored variant ships in v1; Windows Hello + Linux TPM 2.0 / FIDO2 hmac-secret remain stubs.
4. **PRD/SRS .docx body integration** — DEC-106/107/108 + §5.x.x sub-sections currently live as appendix; full integration into §5/§6/§13 body is a docx-editing-session task.

---

*Part 13 refreshes after every meaningful change to `docs/`, `runtime/tools/`, or the protocol SHA. Last refresh: 2026-05-10 post-Stage-5 landing.*

---


---

## Part 14 — Cookbook: Pre-commit hook + CI integration

*Inlined cookbook recipe. Original at `docs/cookbook/pre-commit-validate.md` (deleted in consolidation).*

### Wire `cyberos-validate` into git pre-commit

Run `cyberos-validate` against your `.cyberos-memory/` before every commit so chain-corruption, schema-drift, supersedes-cycles, and cap-overruns surface immediately rather than during a future debugging session.

## Quick install

```bash
# From the project root:
pip3 install pyyaml --break-system-packages   # or via your venv
chmod +x runtime/tools/cyberos_validate.py
```

Verify it works against your store:

```bash
python3 runtime/tools/cyberos_validate.py .
```

You should see `✅ no findings; store appears healthy.` (or a list of findings to address).

## pre-commit hook (POSIX)

Drop this in `.git/hooks/pre-commit` and `chmod +x`:

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit
set -euo pipefail

# Only run if .cyberos-memory/ is staged or has uncommitted changes
if git diff --cached --name-only | grep -q "^.cyberos-memory/" \
   || git status --porcelain | grep -q "^.\? \.cyberos-memory/"; then
    echo "→ cyberos-validate"
    if ! python3 runtime/tools/cyberos_validate.py . --quiet; then
        echo "✘ BRAIN validation found CRITICAL issues — commit blocked."
        echo "  Run: python3 runtime/tools/cyberos_validate.py ."
        echo "  to see all findings."
        exit 1
    fi
fi
```

The `--quiet` flag suppresses INFO/WARN; commits only block on CRITICAL.

## pre-commit framework (recommended)

If you use [pre-commit](https://pre-commit.com), add this to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cyberos-validate
        name: cyberos-validate
        entry: python3 runtime/tools/cyberos_validate.py . --quiet
        language: system
        pass_filenames: false
        files: ^\.cyberos-memory/
        always_run: false
```

Then `pre-commit install` once and the hook runs on every relevant commit.

## CI integration (GitHub Actions example)

```yaml
# .github/workflows/cyberos-validate.yml
name: cyberos-validate
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install pyyaml
      - run: python3 runtime/tools/cyberos_validate.py . --format sarif > validate.sarif
      - uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: validate.sarif
```

The SARIF output integrates with GitHub's "Security" tab and PR review comments.

## What the validator catches today

- **Chain LINK invariant** (§7.2) — any row whose `prev_chain` ≠ previous row's `chain` fails CRITICAL.
- **Schema conformance** (§5.1) — required fields, valid memory_id (UUIDv7/ULID/legacy), valid timestamps (DEC-088), authority hierarchy (§5.3), classification set (§5.4), confidence in [0.0, 1.0].
- **Supersedes graph integrity** (§9.5) — cycles, dangling targets, dangling `superseded_by`, missing `tombstoned: true` on superseded predecessors.
- **Resource caps** (§5.5) — body >30KB hard cap, frontmatter >4KB, store >10MB, file count >10K.
- **Audit ledger health** (§7) — unparseable rows, oversized rows, malformed `audit_id`, manifest `audit_chain_head` reachability.
- **File hygiene** (§4.3) — UTF-8 BOM, bare CR.
- **Tombstone consistency** (§4.6) — `tombstoned: true` requires `deleted_at`/`deleted_by`/`tombstone_reason`.

## What the validator does NOT catch yet

These need future work; see `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`:

- RFC 8785 JCS chain-hash recomputation (Stage 2 follow-up — currently only the LINK invariant is checked)
- §4.2 content gate normalisation pipeline (homoglyph / ZWJ / confusable folding) — Stage 2 (denylist v2)
- Orphan-file detection beyond "path appears in audit" (Stage 6)
- Filesystem-sync collision detection (Stage 4)
- Encrypted-memory verification (Stage 5)

## Self-test

```bash
python3 runtime/tools/cyberos_validate.py --self-test
```

Expected:

```
✅ 01-clean-bootstrap
✅ 02-chain-break
✅ 03-supersedes-cycle
... (15 fixtures)
```

If any fixture fails, `cyberos-validate` itself has regressed — file an issue or roll back the change.

---

## Part 15 — Cookbook: Filesystem-sync compatibility

*Inlined cookbook recipe. Original at `docs/cookbook/filesystem-sync.md` (deleted in consolidation).*

### Filesystem-sync compatibility for `.cyberos-memory/`

The CyberOS BRAIN service (Yjs+Automerge over WebSocket subgraph, per PRD §5.3.3 and DEC-040) is the eventual canonical multi-machine sync mechanism. **Until that ships**, you may want to keep `.cyberos-memory/` synced across your laptop and desktop via off-the-shelf cloud sync tools. This is a recipe + caveats matrix.

## TL;DR

- **Safe with caveats**: Syncthing, git
- **Workable but watch for collisions**: iCloud Drive, Dropbox, OneDrive, Google Drive
- **Forbidden**: any sync tool that does block-level dedup on the audit ledger (none of the above do this by default)
- **Best practice regardless of tool**: run `cyberos-validate` at session start; treat collision-suspected findings as immediate `cyberos-doctor` work

## The hazards

The audit ledger (`audit/<YYYY-MM>.jsonl`) is **append-only** and **chain-linked**. Two failure modes you need to defeat:

1. **Mid-write sync delivery.** A cloud sync tool ingests `audit/2026-05.jsonl` while a local agent is mid-append. The synced copy contains a partial last line; the next session reads it and trips `audit-row-unparseable`.
2. **Concurrent-machine collision.** You start a session on your laptop, write 5 audit rows, the laptop syncs, you switch to the desktop without waiting, the desktop session also writes against an older view. Now the cloud has two divergent ledgers and most sync tools resolve via "last-write-wins on the file" — silent corruption.

The two-phase-write rule in AGENTS.md §4.4 prevents (1) on POSIX-correct filesystems, but cloud sync tools occasionally read mid-flight before the rename completes. Hazard (2) is the bigger killer.

## Sync tool matrix

### ✅ Syncthing (recommended)

- **Why**: peer-to-peer, no third-party storage, file-version conflict markers per file (`<file>.sync-conflict-<date>-<host>.<ext>`), respects symlinks, ignores `.lock` correctly via `.stignore`.
- **Setup**:
  ```bash
  cat > ~/.cyberos-memory/.stignore <<EOF
  .lock
  .tmp.*.part
  index/
  exports/
  EOF
  ```
  (Path syntax above is illustrative; place a `.stignore` in each device's `.cyberos-memory/` if you don't want index/exports replicating.)
- **Caveats**: still vulnerable to hazard (2) — Syncthing's conflict markers help you detect collisions but don't prevent them. Always run `cyberos-validate` at session start.
- **Pin**: enable "Watch for changes" + "Continuous Versioning" so you can roll back any specific file if a collision corrupts it.

### ✅ git (recommended for solo use)

The protocol's §13.1 step 11 (Bundle Q, 2026-05-11) handles `.gitignore` two ways: by default, adds a commented `# .cyberos-memory/` line so you can opt out later. If `.cyberos-memory/` is already UNCOMMENTED at bootstrap or any §4.7 reconciliation walk, the agent treats it as a deliberate opt-out, appends a one-time `op:"warn" reason:"brain-not-versioned"` audit row (deduplicated by `(reason, path)`), and adds a comment block above the line documenting the opt-out is intentional. To commit `.cyberos-memory/` and use `git push`/`git pull` as your sync vehicle, ensure the line is commented (or absent) — the next session start will detect the opt-in and treat the BRAIN as versioned.

- **Why**: explicit conflict resolution; no silent overwrites; full history; works offline.
- **Setup**: in your project's `.gitignore`, ensure `.cyberos-memory/` is NOT excluded; or use a separate inner repo per the future Stage 4 `cyberos init --git-backup` pattern.
- **Caveats**:
  - The audit ledger is append-only; merging two divergent ledgers requires manual conflict resolution. The Stage 4 cyberos-doctor will eventually handle this; for now, never edit `audit/<YYYY-MM>.jsonl` by hand.
  - Don't `git stash` the BRAIN — it can leave `.cyberos-memory/.lock` in an inconsistent state.

### ⚠️ iCloud Drive (workable)

- **Why caution**: macOS occasionally moves `.cyberos-memory/` content to "Optimize Mac Storage" (cloud-only) when disk pressure is high; first session-start after this is slow and triggers full-walk reconciliation.
- **Setup**:
  - In Finder, right-click `.cyberos-memory/` → **Keep Downloaded** (forces always-local).
  - System Settings → iCloud → iCloud Drive → **Optimize Mac Storage** OFF for the parent folder.
  - Apple's hidden-file ignore rule means `.lock` doesn't sync (good).
- **Caveats**:
  - iCloud occasionally reports stale mtime on synced files; `cyberos-validate` doesn't trust mtime, so this is fine, but tools that do (rsync, etc.) may report spurious changes.
  - Hazard (2) is real — iCloud's last-write-wins on the audit ledger silently corrupts on collision.
- **Mitigation**: always end a session before switching machines. Run `cyberos-validate` at session start.

### ⚠️ Dropbox (workable)

- **Why caution**: similar to iCloud; "smart sync" can offload files; partial-write windows are short but exist.
- **Setup**: right-click `.cyberos-memory/` → **Make Available Offline**. Add `.lock` and `index/` to Dropbox's selective-sync excludes.
- **Caveats**: same hazard (2) as iCloud. Dropbox's conflict files (`<file> (Stephen's MacBook conflicted copy).md`) are easier to spot but you have to know to look for them.

### ⚠️ OneDrive / Google Drive

Workable but more aggressive about cloud-only offloading. Same caveats as iCloud + Dropbox. Ensure offline pinning, ignore `.lock`.

### ❌ Block-level dedup tools

Don't use rsync with `--inplace` against the audit ledger. Don't use any sync tool that does block-level dedup on `*.jsonl` files. The append-only invariant assumes the file is rewritten atomically (via `tmp+rename`); block-dedup tools that patch in place can leave the file in an inconsistent state.

## Detection: how to tell if you've been bitten

Run `cyberos-validate` at the start of every session. Findings to watch:

- `chain-link-mismatch` — almost certainly a sync collision; file order in two divergent histories was preserved but the chains don't connect
- `audit-row-unparseable` — partial-write sync delivery; truncate to last good line via `cyberos-doctor` (Stage 2)
- `audit-chain-head-unreachable` — the manifest's pinned chain head doesn't appear in the (synced) ledger; usually means the manifest synced but the ledger didn't yet, or vice versa

Run after EVERY context switch between machines. The ~200ms cost is invisible; the cost of debugging a corrupted chain a week later is days.

## Recovery (Stage 2 cyberos-doctor — coming soon)

Until `cyberos-doctor` ships:

1. Stop using the BRAIN immediately.
2. Identify the most recent good state — usually the most recent `cyberos-export` bundle or git commit.
3. Restore that state (unzip bundle into a clean directory, or `git checkout`).
4. Re-apply any changes you made after the last good state by hand.
5. Treat the corrupted store as a forensic artefact; archive it under `~/cyberos-corrupted-<date>/` until you've verified the recovery.

## Long term

When CyberOS ships and the BRAIN service is live (PRD §5.3.3, DEC-040), Yjs+Automerge handles all of this with operational-transform CRDTs. Filesystem-sync becomes a fallback for offline work, not the primary mechanism. Until then: be paranoid, automate `cyberos-export`, run `cyberos-validate` constantly, and prefer Syncthing or git over the cloud-attached tools.

---

## Part 16 — Cookbook: Local search index

*Inlined cookbook recipe. Original at `docs/cookbook/local-search-index.md` (deleted in consolidation).*

### Local search index for `.cyberos-memory/`

Stage 3 ships `cyberos-index`, a SQLite-backed local index that converts grep-style memory walking into O(log N) lookup for the four high-traffic patterns:

- **Tag** lookup — "show me everything tagged `stage-1`"
- **Relationship** traversal — "what relates to DEC-094? what supersedes it?"
- **Source-SHA dedup** — "is this WhatsApp export already ingested?"
- **Audit-by-path** — "show me the last 5 ops against `manifest.json`"

Plus tombstone-set membership and supersedes-graph traversal.

## Quick start

```bash
# Full build (one time, or after major edits)
python3 runtime/tools/cyberos_index.py . build

# Incremental update (call as part of session-start)
python3 runtime/tools/cyberos_index.py . update

# Stats
python3 runtime/tools/cyberos_index.py . stats

# Verify index matches canonical store
python3 runtime/tools/cyberos_index.py . verify
```

## Where the index lives

**Default (post-2026-05-10):** routed automatically to a system cache directory outside any cloud-sync territory:

- macOS: `~/Library/Caches/cyberos/<store-fingerprint>/cyberos.db`
- Linux: `~/.cache/cyberos/<store-fingerprint>/cyberos.db`

This avoids the most common operational gotcha: iCloud Drive / Dropbox / OneDrive holding write locks on `.cyberos-memory/index/cyberos.db` when their daemon is reconciling, producing `sqlite3.OperationalError: database is locked`. `~/Library/Caches/` and `~/.cache/` are excluded from cloud sync by default.

The cache directory gets a per-store fingerprint subdirectory (16 hex chars of SHA-256 of the resolved store path) so multiple BRAINs don't collide.

**To opt back into the in-store location** (Stage 3 original behavior):

```bash
# Either pass an explicit path:
python3 runtime/tools/cyberos_index.py . --cache-dir .cyberos-memory build

# Or set the environment variable:
CYBEROS_INDEX_IN_STORE=1 python3 runtime/tools/cyberos_index.py . build
```

The in-store form is excluded from exports per AGENTS.md §11.1 in either case (the index is regenerable cache, never authoritative). Use this form when you specifically want the index portable inside the `.cyberos-memory/` zip bundle, OR when you're certain your filesystem doesn't have cloud-sync interference.

## Query examples

### Tag lookup

```bash
python3 runtime/tools/cyberos_index.py . query tag refinement
```

Returns memories tagged `refinement`, sorted by most-recently-updated, excluding tombstoned. Add `--include-tombstoned` to see all.

### Relationship traversal

```bash
python3 runtime/tools/cyberos_index.py . query relates-to mem_019e0dd6-600d-7803-8f35-32cf2c8bafc2
```

Returns inbound relationships, outbound relationships, supersedes, and superseded_by sets — all from one query.

### Source-SHA dedup check

```bash
python3 runtime/tools/cyberos_index.py . query source-sha sha256:abc123...
```

Used by the §8.6 source-coverage validator and the DEC-080 source-tier resolver to detect drift before re-ingesting. Returns the memory_id(s) already derived from that source SHA.

### Audit lookup by path

```bash
python3 runtime/tools/cyberos_index.py . query audit-by-path manifest.json --limit 10
```

Returns the last N ops against the given path. The path can be in either `.cyberos-memory/foo.md` or `foo.md` form — both match.

### Tombstone check

```bash
python3 runtime/tools/cyberos_index.py . query tombstoned mem_019e0dd6-600d-7803-8f35-32cf2c8bafc2
```

## Performance baseline (live store: 82 memories, 293 audit rows)

```
Full build:                ~120ms (one-time)
Incremental update:        ~35ms (no changes); ~50ms (1-2 changes)
Tag query (in-process):    p50 0.16ms  p95 0.18ms
Audit-by-path (in-process): p50 0.16ms  p95 0.17ms
Relates-to (in-process):   p50 0.12ms  p95 0.14ms
Tag query (CLI fork):      p50 25ms    p95 31ms (Python startup dominates)
DB size:                   377 KB
```

The in-process numbers are what matter when the index is consulted by skills running inside an existing Python session (the typical case). The CLI numbers include Python interpreter startup cost.

## Integration patterns

### Pre-session index update (recommended)

Add to your shell startup or before opening Claude Code / Cursor:

```bash
python3 runtime/tools/cyberos_index.py ~/Projects/CyberSkill/cyberos --cache-dir ~/.cache/cyberos update
```

The incremental update is fast enough to run on every session start.

### As a Python library

```python
import sys
sys.path.insert(0, "runtime/tools")
from cyberos_index import Indexer
from pathlib import Path

idx = Indexer(Path(".cyberos-memory"), cache_dir=Path("/tmp/cyberos-cache"))
results = idx.query_tag("refinement")
for r in results:
    print(r["memory_id"], r["file_path"])
```

### Consumed by future tools

When Stage 5 (encryption) ships, the index will check `tombstoned` flags before encrypting. When Stage 6 (Merkle checkpoints) ships, the indexer will read checkpoint metadata to bound the audit-row scan.

## Schema (SQLite)

Five tables (full schema in `cyberos_index.py`):

- `memories(memory_id, file_path, scope, classification, authority, version, created_at, last_updated_at, body_sha, tombstoned, source_sha)`
- `tags(memory_id, tag)` — many-to-many
- `relationships(from_id, to_id, kind)` — `kind ∈ {refines, contradicts, depends-on, derives-from, summarises, cites}`
- `supersedes(from_id, to_id)` — DAG edges per §9.5
- `audit_rows(audit_id, ts, op, path, memory_id, chain, prev_chain, actor_kind, actor_id, ledger)`
- `index_meta(key, value)` — schema_version, last_indexed_audit_id, last_built_at, etc.

## Caveats

- **Index is derived state.** Authoritative answers always come from `.cyberos-memory/` ground truth. The index is allowed to lag (use `update` to catch up) or to be deleted entirely (rebuild via `build`).
- **No chain LINK verification here.** That's `cyberos_validate.py`'s job, walking the JSONL in file order. The index orders by ts, which doesn't always match insertion order across mixed UUIDv7/ULID audit_ids.
- **No vector / semantic search.** Stage 3 deferred this — sentence-transformers + sqlite-vss would add ~80MB of model + dependency bloat for a stopgap that gets ripped out when Layer 2 ships with bge-m3 + reranker. For semantic recall today, let the consuming agent (Claude / Cursor / etc.) use its own context window over the tag/relationship indices above.
- **Schema version 1.** Future schema bumps will require a `build` (full rebuild). The schema_version key in `index_meta` is the gate.

## Troubleshooting

### `sqlite3.OperationalError: disk I/O error`

The filesystem under `.cyberos-memory/index/` doesn't tolerate SQLite. Use `--cache-dir`:

```bash
python3 runtime/tools/cyberos_index.py . --cache-dir ~/.cache/cyberos build
```

Common causes: cloud-FUSE drivers (some Dropbox configurations), sandbox mounts, network filesystems.

### `verify` reports `memory count mismatch`

Run a full rebuild:

```bash
python3 runtime/tools/cyberos_index.py . build
```

Then `verify` again. If still mismatched, the index has drifted from canonical — `cyberos_validate.py` against the canonical store will tell you where.

### Index size grows large

Run `build` (full rebuild) — drops accumulated tombstones and obsolete rows. The DB compacts via SQLite's native auto-vacuum if enabled; otherwise rebuild quarterly.

## What this enables next

- **Stage 5 (encryption)**: tombstone + classification queries to scope the encryption envelope
- **Stage 6 (Merkle)**: audit-row index becomes the input for incremental Merkle tree construction
- **Future BRAIN service** (PRD §5.4 Layer 2): the source-SHA index is the dedup boundary between Layer 1 ingest events and Layer 2 contextual-retrieval pipeline

---

## Part 17 — Cookbook: At-rest encryption + recovery

*Inlined cookbook recipe. Original at `docs/cookbook/encryption-and-recovery.md` (deleted in consolidation).*

### At-rest encryption and recovery

Stage 5 (`sha256:d3ce97…`) added the at-rest encryption envelope (AGENTS.md §5.6) and `cyberos_encrypt.py`. Encryption is **opt-in and OFF by default** — the protocol primitives are landed but no memory is encrypted until you run the enable wizard.

## When to enable

Turn encryption on when the answers to any of these become "yes":

- I might lend this Mac to a contractor / sell it / hand it to support
- I'll be travelling with the BRAIN on disk
- I'll be syncing `.cyberos-memory/` via iCloud/Dropbox/Syncthing across less-trusted machines
- I've started accumulating `personnel:` or `client:` memories that aren't pure ops notes

If none of those apply (solo workbench on FileVault-encrypted Mac), encryption is overhead without proportional benefit — leave it off.

## Pre-flight checklist

Before running the wizard:

- [ ] Pick **5 holders** for Shamir fragments. Suggested defensible pattern: yourself + spouse/co-founder + lawyer (sealed envelope) + family member (paper QR in a safe-deposit box) + geographically-distant trusted contact
- [ ] For each holder, decide the **delivery medium** — printed QR code on archival paper? base32 string in a password manager? read-aloud over a secure channel? Don't use anything backed up by an automated cloud service the holder doesn't control
- [ ] Run a fresh `cyberos-export` so you have a backup of the **plaintext** state in case anything goes wrong during the migration phase
- [ ] Have your passphrase ready — minimum 16 chars, zxcvbn score ≥3 (no dictionary words; consider a 5-word diceware phrase)

## Enable wizard

```bash
python3 runtime/tools/cyberos_encrypt.py . enable --passphrase
```

The wizard:

1. Prompts for passphrase + confirm; rejects below the strength bar
2. Derives master key via Argon2id (`t=3, m=64MiB, p=4` per RFC 9106) — takes ~2 seconds
3. Splits the master into 5 Shamir fragments (3-of-5 threshold)
4. Walks each fragment: prompts for holder label, prints the encoded fragment as `CYBOS-S5-<label>-<base32-with-dashes>`, asks "distributed? [y/N]"
5. Refuses to flip `encryption_policy.enabled = true` until all 5 are confirmed distributed
6. Pins the master-key fingerprint in `manifest.shamir_fragments.master_key_fingerprint`
7. Records each fragment's fingerprint + holder label + timestamps in the same manifest field

**The wizard never writes fragments to disk anywhere.** Only fingerprints land in the manifest. If you lose the printout/text and don't have ≥3 fragments out-of-band, you cannot recover.

## After enable: nothing happens automatically

This is intentional. After enable:

- `encryption_policy.enabled = true`
- 0 memories are encrypted (no automatic migration)
- New writes to in-scope memories will encrypt going forward
- Existing 80+ in-scope memories stay plaintext

To migrate existing memories at your own pace (Q5 = user-paced from the decision baseline):

```bash
python3 runtime/tools/cyberos_encrypt.py . migrate-batch 50
```

Each batch runs as one MAINTENANCE-mode envelope (§8.8) with one `op:"str_replace"` per memory. Watch the audit ledger; if anything looks weird, pause and run `cyberos-doctor`.

> ⚠️ `migrate-batch`, `disable`, and `rotate-shamir` are stubbed in the v0 of `cyberos_encrypt.py`. The enable wizard, status check, recovery flow, and Shamir crypto core are fully working in v0. Migration + rotation are scheduled for v1.

## Verifying encryption is working

```bash
# Check policy state
python3 runtime/tools/cyberos_encrypt.py . status
```

Returns JSON: `policy_enabled`, `shamir_master_key_fingerprint`, `memories_encrypted`, `memories_plaintext`, etc.

```bash
# Validate the BRAIN — picks up `encrypted: true` recognition + Shamir consistency check
python3 runtime/tools/cyberos_validate.py .
```

Should report 0 CRITICAL findings. New checks that come from Stage 5:

- `encryption-block-missing` — `encrypted: true` set but no `encryption:` frontmatter block
- `encryption-algo-unrecognised` — algorithm is not `xchacha20poly1305-ietf` or `xchacha20poly1305-ietf-v0`
- `encryption-nonce-length` — nonce is not 24 bytes
- `shamir-fingerprint-missing` — encryption enabled but no master_key_fingerprint pinned
- `shamir-incomplete-distribution` — fewer than `total` fragments confirmed distributed

## Recovery: when both passphrase AND HW key are lost

The whole point of the Shamir 3-of-5 design. Collect ≥3 fragments out-of-band from your holders, then:

```bash
python3 runtime/tools/cyberos_encrypt.py . recover
# (paste fragments one per line, empty line to end)
```

Wizard:

1. Decodes each `CYBOS-S5-…` fragment back to bytes
2. Picks the first 3 (threshold), runs Lagrange interpolation in GF(256) to reconstruct the master key
3. Hashes the reconstructed key, compares to `master_key_fingerprint` pinned in `manifest.shamir_fragments`
4. **MISMATCH → ABORT** (either you got the wrong fragments, or someone tampered with the BRAIN; investigate before doing anything)
5. **MATCH → reconstruct successful**; v0 prints "key NOT printed for security" and stops here. v1 will integrate the recovered key into the running session so you can re-derive your passphrase or set up new HW key.

## Hardware-key change procedure (planned, not yet implemented)

When you replace your Mac (HW key changes), use:

```bash
python3 runtime/tools/cyberos_doctor.py . --repair --reason "hardware-replacement"
# Will offer R6-rotate-master-key (planned for v1)
```

R6 will: derive master from new HW source; re-encrypt all in-scope memories under MAINTENANCE mode; audit each as `op:"str_replace"`; flip the manifest's `encryption_policy.key_derivation` field.

Until R6 ships, manual procedure: `disable` (decrypt all → plaintext), then re-`enable` on the new machine with fresh fragments. Costs the audit-row churn of an entire migration; document the reason in MAINTENANCE mode notes.

## Threat model — what the encryption protects against

| Adversary scenario | Protected by | Notes |
|---------------------|-------------|-------|
| Someone reads `.cyberos-memory/` from a backup, no key | XChaCha20-Poly1305 envelope on body | Frontmatter is plaintext — they see classification + scope + tags but not body content |
| Someone compromises your passphrase but not your machine | Hardware-key path (when implemented) | v0 ships passphrase-only, so this scenario is currently NOT mitigated |
| Someone tampers with the encrypted body to fool you | AEAD authentication tag | `decrypt_body` raises `InvalidTag` on any modification |
| Someone substitutes a different memory's body | AAD bound to memory_id + last_updated_at | Tag verification fails if the AAD doesn't match |
| Holder of 1 fragment becomes adversarial | Below threshold | Need 3 fragments to recover; 1 alone is useless |
| 3 holders collude | Cannot prevent | This is the explicit threshold tradeoff. Pick holders accordingly |
| You lose the passphrase + 2 fragments at once | Stop / call lawyer | The remaining 3 fragments still recover. If you lose 3+, the BRAIN is unrecoverable. Plan accordingly: keep your own fragment offline + protected; treat the remaining 4 as redundancy |

## What the encryption does NOT protect against

- The §9.3 denylist. Comp/ESOP/gov-IDs/secrets remain forbidden in any storage form. Encryption is a layer on top of an already-restricted surface, not a softener
- Live agent memory. While a session is active and the master key is derived, memory content is decrypted in process RAM. Don't run agents on machines you don't trust
- Side channels. AEAD doesn't hide ciphertext length; an attacker counting bytes can infer "this memory is bigger than that one"
- The export bundle. `cyberos-export` produces deterministic ZIP bundles that contain ciphertext when memories are encrypted. The bundle is as protected as the underlying memories, no more

## Operational reminder

Treat the master-key fingerprint pinned in `manifest.shamir_fragments.master_key_fingerprint` as a **public commitment**. Anyone with the BRAIN can see it; recovery only succeeds if reconstructed-key-fingerprint matches it. This protects against fragment substitution (an adversary who has 3 *wrong* fragments can't recover because the fingerprints won't match).

Don't change the fingerprint outside `cyberos_encrypt.py rotate-master-key` flow — direct manifest edits to that field break recovery semantics. The `op:"key_rotation"` audit row is the authoritative event marker.

---

## Part 18 — Cookbook: Audit ledger compaction

*Inlined cookbook recipe. Original at `docs/cookbook/ledger-compaction.md` (deleted in consolidation).*

### Audit ledger compaction (Stage 6)

Stage 6 (`sha256:77eda21…`) added Merkle checkpoints (§7.6) and audit ledger compaction (§7.7). After ~12 months of operation, the per-row JSONL ledger can be collapsed to per-memory final-state + Merkle proof, saving ~80% disk while preserving spot-verifiability via the cryptographic proof.

## Pre-conditions

Compaction refuses unless ALL of these hold (per AGENTS.md §8.9):

1. The cutoff month has at least one `op:"consolidation_run"` row carrying a `merkle_root` field — without a checkpoint there's nothing to anchor proofs against
2. The cutoff month is older than `manifest.compaction_policy.minimum_age_months` (default 12)
3. `cyberos_validate.py --self-test` passes; specifically no §8.7 phase 4 CRITICAL findings on the period being compacted

## Triggering compaction

Compaction requires the **explicit chat-turn phrase** per §0.5:

> *"compact ledger older than 2026-04-30"*

The agent then:

1. Acquires `.lock` (exclusive)
2. Verifies pre-conditions; aborts with `op:"rejected" reason:"compaction-precondition:<which>"` on failure
3. Walks rows in the cutoff period; builds a per-memory `final_state` map (memory_id → most recent op + chain)
4. Computes Merkle inclusion proofs for each `final_audit_id` against the period's checkpoint root
5. zstd-compresses the original JSONL into `archive/<YYYY-MM>.jsonl.zst`
6. Atomic-renames `audit/<YYYY-MM>.jsonl` → `audit/<YYYY-MM>.compacted.jsonl`
7. Appends `op:"ledger_compact"` to the live ledger
8. Releases `.lock`

The compaction implementation lands as a `cyberos_doctor.py compact-ledger` subcommand in v1 (currently the protocol is in place but the user-facing CLI to trigger it is part of Track A's follow-on work).

## Verifying a compacted ledger

`cyberos_validate.py` (post-Track-A) walks `audit/*.compacted.jsonl` files and verifies each row's `merkle_proof` against the period's checkpoint root. Mismatch → CRITICAL `merkle-proof-divergence`.

To spot-check a specific chain:

```bash
python3 runtime/tools/cyberos_index.py . --cache-dir /tmp/cyberos-cache query merkle-proof sha256:abc123...
```

Returns the inclusion path + the period's checkpoint root. Manually verify by:

```python
import hashlib
leaf = bytes.fromhex("abc123...")
current = leaf
for step in proof:
    sibling = bytes.fromhex(step["hash"].replace("sha256:", ""))
    if step["position"] == "left":
        current = hashlib.sha256(sibling + current).digest()
    else:
        current = hashlib.sha256(current + sibling).digest()
print("sha256:" + current.hex())  # must equal checkpoint_root
```

## Reversing compaction (decompact)

If you need the full per-row history back (audit, forensics, regulatory request), re-expand from archive:

```bash
python3 runtime/tools/cyberos_doctor.py . \
    --decompact-ledger 2026-04 \
    --reason "regulatory request: forensic audit of 2026-04 ops"
```

The doctor:

1. Acquires `.lock` under MAINTENANCE mode
2. zstd-decompresses `archive/2026-04.jsonl.zst`
3. Atomic-writes the decompressed bytes back to `audit/2026-04.jsonl`
4. Removes `audit/2026-04.compacted.jsonl`
5. Appends `op:"ledger_decompact"` to the live ledger
6. Releases lock

Decompaction is fully reversible — you can re-compact later with the same phrase.

## When NOT to compact

- If you might need quick access to per-row history (legal hold, ongoing investigation) — keep the JSONL form
- If your store is under the 1MB/10K-file size soft cap — compaction's disk savings are marginal
- If you haven't run a `consolidation_run` in the period being compacted — no Merkle root to anchor proofs against
- During an open `op:"maintenance.start"` envelope — wait for the maintenance session to close

## What compaction does NOT do

- It does NOT delete audit history. The original verbatim is preserved at `archive/<YYYY-MM>.jsonl.zst`. Re-expansion is one command away.
- It does NOT shrink memory files. Memories stay where they are; only the audit ledger is compacted.
- It does NOT bypass `op:"corrects"` or `op:"revert"` semantics. The compacted form preserves the *final state* per memory, but corrections-of-corrections are collapsed in the per-memory representation. If you need the full correction chain, decompact.

## Long-running schedule

A reasonable cadence: compact monthly, 12 months back. So in May 2027, compact 2026-05 (just turned 12 months). On the first of every month, the eligible period rolls forward by one. This keeps the live ledger sized to ~12 months of per-row data plus N years of compacted summaries.

The `manifest.compaction_policy.minimum_age_months` field is mutable only via §0.5 chat-turn approval — change it from 12 to 6 (more aggressive compaction) or 24 (more conservative) per your operational preference. The default of 12 reflects a balance between disk savings and "I might still need to look at this row" recency.

---

## Part 19 — Future-state outlook (post-CyberOS-product)

The local-optimization roadmap documented in Parts 13 (status hub) is **complete** as of 2026-05-10. Three protocol-level upgrades (Stages 1, 5, 6) + Bundle M refinement pass + 6 runtime tools + 21-fixture corpus + 5 inlined cookbooks above. The Layer-1 personal BRAIN protocol is production-ready.

**The next protocol-level horizon activates when CyberOS-the-product begins building.** That horizon is documented as eight stages mapped to CyberOS-PRD Part 5 (BRAIN architecture) + Part 9.1 (BRAIN module) + Part 11 (NFRs) + Part 12 (Compliance) + Part 14 (Phase plan); SRS §5.12 / §6.13–6.16. Currently dormant — none of these stages have been executed; they reactivate when BRAIN service P1 starts shipping.

| Stage | Focus | Status |
|-------|-------|--------|
| EV-1 | Performance + DX inside Layer 1 | mostly covered by local-optimization Stage 1+2+3 |
| EV-2 | Security hardening + Vietnamese-first compliance scaffolding | partial: at-rest encryption shipped (Stage 5); STRIDE doc + Decree 13 + EU AI Act mappings TODO |
| EV-3 | Layer-1↔Layer-2 indexing bridge + observability | needs Layer 2 (PostgreSQL + pgvector + AGE + bge-m3 + reranker) |
| EV-4 | Multi-machine sync via Yjs+Automerge BRAIN service | this IS BRAIN P1; critical path |
| EV-5 | Tenant isolation + cap evolution under DEC-070 supersede | needs multi-tenant infra |
| EV-6 | MCP Gateway integration + GraphQL contracts + skill-registry v0.2.0 wiring | needs CyberOS modules |
| EV-7 | AI-native: GraphRAG community summaries + multi-modal frames | needs Layer 2 |
| EV-8 | Compliance ring 2/3 + governance + post-quantum migration | enterprise-procurement readiness |

**Backward compatibility commitment:** every future stage preserves the §7.2 chain LINK invariant, the six-op surface (§4), and the closed-set §5.1 frontmatter. Schema additions land via §0.5 approved upgrades; pre-existing memories never need rewriting.

**Recalibration after local-optimization shipped (2026-05-10):** several gaps EV-1 / EV-2 / EV-3 / EV-5 listed are now partially or fully addressed at the local-Layer-1 level. When this roadmap is reactivated post-BRAIN-service, those are already-shipped primitives — they just need promotion into the multi-machine / multi-tenant context the BRAIN service introduces.

The detailed 8-stage breakdown lived at `docs/CyberOS-AGENTS.EVOLUTION.md` (deleted in consolidation; full content preserved in git history). When CyberOS-the-product starts building, regenerate from this Part 19 outline + the surrounding PRD/SRS context.

---

## Part 20 — Proposals: Stage 1 protocol upgrade (LANDED)

*Inlined historical proposal. Original at `docs/proposals/STAGE-1-PROTOCOL-UPGRADE.md` (deleted in consolidation). LANDED 2026-05-10 as `sha256:576368…`.*

### Stage 1 — Protocol Upgrade Proposal

**Status**: Draft, ready for §0.5 chat-turn approval
**Source plan**: `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` Stage 1
**Targets**: AGENTS.md §4.7, §5.5, §6, §8.7

This document is the **exact prose to insert/replace in AGENTS.md** when you approve Stage 1. After approval, the related-files chain (§0.6) requires updates to the CHANGELOG, PRD, SRS, and a `memories/refinements/REF-NNN-stage-1-session-speed.md` entry — all listed at the bottom.

To adopt: paste the approval phrase from §0.5 in chat, citing the SHA of the post-edit AGENTS.md. The agent computes the SHA after applying the edits below; you can also pre-compute it with `python3 runtime/tools/canonical_sha.py docs/CyberOS-AGENTS.md` (script ships with Stage 2).

---

## Change A — `manifest.reconciliation_checkpoint` field (§6 extension)

### Where

`docs/CyberOS-AGENTS.md` §6, inside the `manifest.json` schema block, alongside the existing `protocol`, `health_check_policy`, `source_tiers` blocks.

### Insert

```json
"reconciliation_checkpoint": {
  "audit_id": "<evt_…|null>",
  "chain": "<sha256:…|null>",
  "ts": "<ISO-8601|null>"
}
```

Add the following sentence after the existing manifest schema:

> **`reconciliation_checkpoint`** records the most recent successfully-completed `op:"session.end"` or `op:"consolidation_run"` row. §4.7 reconciliation walks only rows after this checkpoint when present; falls back to full walk on missing/stale (>30 days) checkpoints or any chain-mismatch. Updated atomically with `op:"session.end"` and `op:"consolidation_run"` writes; never edited independently.

### Why

§4.7 currently walks all audit rows newer than the last `consolidation_run` on every session start. On long-lived stores this is O(N) every session. The checkpoint pin makes it O(rows_since_last_session) for the common case while preserving full-walk safety for edge cases.

### Backward compat

Additive field. Older agents (pre-Stage-1) trip `INCOMPATIBLE:reconciliation_checkpoint` per §13.0's forward-compat tripwire — which is the correct behaviour. Run the latest AGENTS.md.

### Audit row impact

None — the checkpoint is a manifest field, written via the existing `op:"str_replace"` path that already exists for manifest mutations.

---

## Change B — §4.7 reconciliation update

### Where

`docs/CyberOS-AGENTS.md` §4.7 "Reconciliation (session start)".

### Replace

> Walk audit rows newer than the last `consolidation_run`. For each row with `op ∈ {create, str_replace, insert, rename}` that is the most-recent op against its `path` (not later reverted): …

### With

> Walk audit rows newer than `manifest.reconciliation_checkpoint.audit_id` if set; otherwise walk all rows newer than the last `consolidation_run`. If the checkpoint is older than 30 days OR `manifest.reconciliation_checkpoint.chain` does not match the corresponding row in the ledger, fall back to the full-walk path and emit `op:"warn" reason:"stale-checkpoint"`. For each row with `op ∈ {create, str_replace, insert, rename}` that is the most-recent op against its `path` (not later reverted): …

The remainder of §4.7 (existence verification, hash check, orphan detection) is unchanged.

### Why

Operationalises Change A. The 30-day stale-window prevents long-stored checkpoints from masking corruption that accumulated while the BRAIN was unused. The chain-mismatch fallback is the integrity guarantee — if anything has tampered with rows between the checkpoint and the present, full-walk catches it.

---

## Change C — `manifest.read_profile` field (§6 extension + §10 amendment)

### Where

`docs/CyberOS-AGENTS.md` §6 manifest schema, plus a small addition to §10 read protocol.

### Insert in §6

```json
"read_profile": {
  "eager_scopes": ["meta"],
  "lazy_scopes": ["company", "module", "member", "client", "project",
                  "persona", "memories"]
}
```

### Add to §10 (after step 1)

> **1a. Honour `manifest.read_profile`.** Eager scopes load on every session start. Lazy scopes load on first reference to a path within them per the request-implied logic in step 3. Default profile: `eager_scopes: ["meta"]`, all other scopes lazy. Projects may override.

### Why

The existing §10 specifies "load only what's needed" but leaves the eager/lazy boundary implicit. Making it explicit-and-configurable lets long-running sessions skip unrelated scope reads.

### Backward compat

Additive. Default profile is the existing implicit behaviour. Older agents that ignore the field continue working.

---

## Change D — Frontmatter compactness rule (§5.1 amendment)

### Where

`docs/CyberOS-AGENTS.md` §5.1, after the existing 28-field schema description.

### Insert

> **Frontmatter compactness (write-side).** When emitting frontmatter, omit any field whose value is `null` OR an empty array OR an empty object, EXCEPT for fields explicitly required by `classification` (consent block for `personnel`/`client`) or `tombstoned: true` (deleted_at/deleted_by/tombstone_reason). Read-side accepts both compact and verbose forms — omitted optional fields default to `null`/empty. The 28-field closed-set rule applies only to *recognised* fields; absence of optional fields is not a schema violation.

### Why

The 28-field schema currently encourages emitting every field — a chat memory often has `expires_at: null`, `embedding: {model: null, version: null, vector_id: null}`, `consent: {has_consent: null, ...}`, etc. These bloat frontmatter to 800+ bytes when 200 would suffice. Compactness drops typical frontmatter 30-40% and reduces the chance of hitting the 4 KB hard cap.

### Backward compat

Read-side change is purely permissive (already accepts missing optional fields). Write-side is opt-in via the rule; the canonical reference impl at `runtime/lib/brain_writer.py` (per AGENTS.md §0.6 line 175) updates on next §0.5 cycle.

---

## Change E — §8.7 self-audit phase 4 update

### Where

`docs/CyberOS-AGENTS.md` §8.7, in the "Six checks, in order:" list, replace check 4.

### Replace

> 4. **Audit chain integrity** — verify LINK integrity end-to-end (not just incremental like §4.7): for each row N, confirm `row[N].prev_chain == row[N-1].chain`. LINK integrity is the authoritative invariant per §7.2's cross-writer-version compatibility clause. Hash recomputation (`chain == sha256_hex(canonical_json(row_without_chain_or_prev_chain) || prev_chain)` per §7.2) MAY be performed and reported at INFO severity; recomputation differences across writer versions are NOT chain breaks. Confirm `manifest.audit_chain_head` is reachable in the ledger.

### With

> 4. **Audit chain integrity** — verify LINK integrity end-to-end (not just incremental like §4.7): for each row N, confirm `row[N].prev_chain == row[N-1].chain`. LINK integrity is the authoritative invariant per §7.2's cross-writer-version compatibility clause. Hash recomputation (`chain == sha256_hex(canonical_json(row_without_chain_or_prev_chain) || prev_chain)` per §7.2) MAY be performed and reported at INFO severity; recomputation differences across writer versions are NOT chain breaks. Confirm `manifest.audit_chain_head` is reachable in the ledger. **Additionally, if `manifest.reconciliation_checkpoint` is set, confirm `checkpoint.audit_id` resolves to a row in the ledger AND `checkpoint.chain` matches that row's `chain`. Mismatch → `CRITICAL stale-checkpoint`; freezes writes until reconciled per §4.7 fallback.**

### Why

Stage 1's checkpoint pin needs §8.7 to verify it as part of the routine self-audit. The check is cheap (one row lookup) but catches the case where a backup/restore drops the manifest in but ledger files out of sync.

---

## Order of operations to land Stage 1

Per AGENTS.md §0.5 + §0.6:

1. **Edit AGENTS.md** with Changes A–E above.
2. **Archive prior verbatim** to `meta/protocol-history/AGENTS-<before_sha256>.md` (per §0.5 step 1).
3. **`str_replace` on `manifest.json`** to update `manifest.protocol.sha256`, `approved_at`, `approved_by`.
4. **Append `op:"protocol_upgrade"`** to the audit ledger with `before_hash`/`after_hash` for the manifest, `reason: "<before_sha256> → <after_sha256> per §0.5 (Stage 1: session-start speed)"`.
5. **Auto-trigger §8.7 self-audit pass** per §0.5 step 4. Output → `meta/health/<YYYY-MM-DD>-<sha>-postupgrade.md`.
6. **Update CHANGELOGs** per §0.6:
   - `docs/CyberOS-AGENTS.CHANGELOG.md` — new dated section for this upgrade
   - `docs/CyberOS-PRD.CHANGELOG.md` — note Stage-1 absorption against PRD §5.3.2 (six file ops) and §5.3.5 (Auto Dream)
   - `docs/CyberOS-SRS.CHANGELOG.md` — note implementation specification for the new manifest fields + §4.7 amendment
7. **Write `memories/refinements/REF-NNN-stage-1-session-speed.md`** — refinement record per §0.4.
8. **Add new DEC entry** in PRD §5.9 / Part 13: `DEC-NNN — Reconciliation checkpoint + lazy-load profile + frontmatter compactness (Stage 1)`. Status: Locked. Cite Changes A–E as implementation refs.

## Approval phrase to land

In chat, you say:

> *"approve protocol upgrade to sha256:<computed-sha-after-applying-A-through-E>"*

The agent computes the canonical SHA of the post-edit AGENTS.md (NFC, LF, BOM strip, trim per line, single terminating LF — per §0.5 canonical form), confirms the SHA matches your phrase, then runs steps 1-8 above as a single atomic operation.

If you want a dry-run, say *"preview protocol upgrade for Stage 1"* — the agent walks you through the edits without applying them.

---

## After Stage 1 lands

The Stage 2 work (already shipped in `runtime/tools/cyberos_validate.py`) gains coverage of the new fields: it'll start verifying the checkpoint pin as part of the chain-integrity check. The validator's `--self-test` corpus gains a `16-stale-checkpoint/` fixture demonstrating the new failure mode.

The remaining Stage 2 work — `cyberos-doctor` recovery CLI — depends on Stage 1 being landed (it uses the checkpoint to scope diagnostic operations). Once Stage 1 lands, Stage 2 can complete.

Stages 3–6 follow.

---

## Part 21 — Proposals: Stage 5 open questions (decision baseline; LANDED)

*Inlined historical decision-rationale. Original at `docs/proposals/STAGE-5-OPEN-QUESTIONS.md` (deleted in consolidation). Decisions baseline for Stage 5 (LANDED 2026-05-10 as `sha256:d3ce97…`); 'go with your recs' approval.*

### Stage 5 — Open questions before encryption ships

**Status**: Decision-blocked. Stage 5 (at-rest encryption + Shamir 3-of-5 escrow) needs your input on five questions before I can write the proposal text + reference implementation.

This document lists the questions, the considered options, and my recommendation for each. Reply with your choices (or "go with your recs") and I'll draft the §0.5 upgrade proposal + ship `cyberos_encrypt.py`.

---

## Question 1 — Default scopes for encryption

When `manifest.encryption_policy.enabled = true`, which scopes get encrypted by default?

**Options:**
- **(a) Conservative**: only `member/<self>/private/` — narrowest possible scope, opt-in per-other-scope
- **(b) Sensitive-by-classification**: any memory with `classification: personnel` or `classification: client`
- **(c) Sensitive-plus-private**: combine (a) and (b) — covers private personal scope plus all personnel/client across scopes
- **(d) Everything except `public`**: encrypt all `personnel | client | operational` memories; only `public` stays plaintext

**Tradeoffs:**
- (a) is hardest to misuse but covers the least surface; an outsider opening `member/<self>/notes-on-employees.md` (not in `private/`) sees plaintext personnel notes
- (b) auto-protects sensitive content based on the existing classification system but doesn't catch private-but-operational memories (e.g., founder's own working notes)
- (c) is what most security frameworks would recommend
- (d) is broadest but slows down any tool that doesn't have the key (e.g., grep against `operational` notes won't work without decrypting)

**My recommendation:** **(c) Sensitive-plus-private**. The §9.3 denylist already structurally excludes the highest-stakes content (comp/ESOP/secrets) — encryption protects the second tier (perf reviews, client engagement context, founder's private notes). Allows fast grep against `operational` and `public` for daily search.

---

## Question 2 — Hardware-key fallback policy

When the hardware key is unavailable (Touch ID disabled, TPM not provisioned, FIDO2 token not plugged in), what happens?

**Options:**
- **(a) Refuse to operate** — if encryption is enabled and HW key is unavailable, the BRAIN is read-frozen until HW key returns
- **(b) Argon2id passphrase fallback** — prompt for a passphrase; derive master key via Argon2id (t=3, m=64MiB, p=4); cache in memory for the session
- **(c) Both — HW key OR passphrase** — accept either, even simultaneously enrolled; choice at runtime
- **(d) HW key only** — no passphrase fallback; lost HW = use Shamir 3-of-5 recovery

**Tradeoffs:**
- (a) is the safest but creates rage-quit moments when you're away from your usual machine
- (b) is the user-friendly default but a passphrase is the weakest link if it's not very strong
- (c) is most flexible but doubles the attack surface
- (d) is purest but means a dead Touch ID sensor = mandatory recovery flow (annoying for routine work)

**My recommendation:** **(c) Both — HW key OR passphrase**. Hardware key is the default daily flow; passphrase is the fallback when traveling, on borrowed equipment, or after a hardware failure. Shamir is for catastrophic loss only.

---

## Question 3 — Shamir 3-of-5 fragment holders

The Shamir Secret Sharing scheme splits the master key into 5 fragments; any 3 reconstruct it. Who holds the fragments?

**Default plan:** 5 holders, 3-of-5 threshold.

**Options for holder roles:**
- **Holder 1**: Stephen (primary; safe-deposit box or password manager)
- **Holder 2**: Co-founder / business partner (encrypted offline or hardware token)
- **Holder 3**: Family member (spouse / sibling) — physical paper QR in a sealed envelope
- **Holder 4**: Lawyer / notary — sealed and instructed to release only on specific conditions
- **Holder 5**: Geographically-distant trusted contact (e.g. a CyberSkill cofounder elsewhere)

**Questions for you:**
- (a) Are 3-of-5 thresholds right? Or do you want 2-of-3 (smaller circle, faster recovery) or 4-of-7 (larger redundancy, slower)?
- (b) Who specifically should hold each fragment? Or do you want me to leave holder identification as a guided wizard step at encryption-enable time?
- (c) Do you want a "deadman switch" pattern — fragments released after N months of inactivity to designated heirs? (Requires additional infrastructure; default OFF.)

**My recommendations:**
- (a) **3-of-5** is the canonical balance. Lose any 2 fragments and you're still fine; you need active cooperation of 3 to misuse.
- (b) **Wizard at enable time.** Don't bake holder identities into the proposal — ask at the wizard step. Holders can rotate later via `op:"shamir_rotation"`.
- (c) **Deadman switch DEFAULT OFF.** Adds infrastructure complexity (timer, notification system) for a feature most users won't use. Document as future opt-in.

---

## Question 4 — Encrypt frontmatter or body-only?

The XChaCha20-Poly1305 envelope can encrypt:
- **(a) Body-only** — frontmatter stays plaintext; indexes (Stage 3) work without the key
- **(b) Body + selective frontmatter fields** — encrypt fields like `tags`, `relationships`, `provenance.source_ref` if they leak intent
- **(c) Whole-file** — entire memory file encrypted; nothing readable without the key

**Tradeoffs:**
- (a) lets `cyberos_index.py` build tag/relationship/source-SHA indices over encrypted memories; lets `cyberos_validate.py` verify schema + chain integrity. Frontmatter leaks "this memory is `personnel` classification with `confidence: 0.85`" which is metadata-only.
- (b) protects intent fields but breaks tag/relationship indexing for those memories
- (c) maximises confidentiality but breaks all derived caches; key required for any operation including `view`

**My recommendation:** **(a) Body-only**. The frontmatter is metadata-only (no PII; the §5.1 closed-set forbids body content in frontmatter). Tools and indexes keep working. The PRD §5.8 denylist already structurally excludes the highest-stakes content from being stored at all.

---

## Question 5 — Migration strategy for existing memories

When you turn on encryption for an existing store with N memories already in plaintext, what happens?

**Options:**
- **(a) Lazy migration** — encrypt on first write of each memory; old plaintext stays until touched
- **(b) One-shot migration** — `cyberos-encrypt enable --migrate` walks all in-scope memories, re-writes each as encrypted, audits each as `op:"str_replace"`
- **(c) User-paced migration** — enable encryption for new writes; show a migration counter ("N memories still plaintext"); user runs `--migrate-batch 50` whenever they want
- **(d) Refuse to enable until empty** — only allow encryption on a fresh `.cyberos-memory/` (impractical)

**Tradeoffs:**
- (a) is simplest but leaves a long tail of plaintext-on-disk indefinitely
- (b) is safest but generates many audit rows + a long lock window during the rewrite
- (c) is the user-friendly compromise; user controls the audit-row generation rate
- (d) is purest but useless for the existing live store

**My recommendation:** **(c) User-paced migration** with `--migrate-batch <N>` (default 50). Each batch runs as one `op:"maintenance.start"`/`op:"maintenance.end"` envelope per §8.8 with one `op:"str_replace"` per memory. Lets you migrate incrementally, watch for issues, and pause if anything looks off.

---

## Summary — my recommendation rolled up

If you say "go with your recs":

- Default encrypted scopes: `member/<self>/private/` + all `personnel`/`client` classification memories
- Hardware key OR Argon2id passphrase, both accepted
- 3-of-5 Shamir with holders chosen via wizard at enable time; no deadman switch
- Body-only encryption (frontmatter stays indexable)
- User-paced migration via `--migrate-batch 50`

These five together give you a defensible defaults set that most security teams would sign off on.

## What I'll ship after you decide

Once you respond:

1. **`docs/proposals/STAGE-5-PROTOCOL-UPGRADE.md`** — the §0.5 upgrade text following the same pattern as Stage 1 and Stage 6 proposals. Adds `manifest.encryption_policy`, `manifest.shamir_fragments`, `op:"key_rotation"`, `op:"key_recovery_initiated"`, `op:"key_recovered"`, `op:"shamir_rotation"`, `op:"shamir_distribution_confirmed"`, `op:"encryption_policy_change"` as new audit op kinds. Adds the `encrypted: true` frontmatter flag.
2. **`runtime/tools/cyberos_encrypt.py`** — implementation. CLI subcommands:
   - `enable` — wizard flow: detect HW key, generate master, Shamir-split, walk holder distribution, confirm fragments, finally flip the flag
   - `disable` — decrypt all → re-encrypt with no policy → flip flag, audited
   - `migrate-batch <N>` — encrypt N more in-scope memories
   - `rotate-shamir` — generate fresh 5 fragments without changing master key
   - `recover` — accept ≥ threshold fragments, reconstruct master, re-key
   - `status` — show encryption coverage stats (encrypted vs plaintext per scope)
3. **`runtime/tools/cyberos_validate.py`** extension — verify encrypted memories' chain integrity (works without key; uses recorded `after_hash` over plaintext per Stage 5 design).
4. **`docs/cookbook/encryption-and-recovery.md`** — operational guide, including the recovery flow walkthrough.

Estimated work: 8–12 hours focused, depending on platform-specific HW-key wiring complexity.

---

## Prerequisites already satisfied

- ✅ Stage 1 landed (reconciliation_checkpoint provides ledger-state anchor for encrypted-memory writes)
- ✅ Stage 2 landed (cyberos_validate + cyberos_doctor — encrypted memories integrate with existing repair flows)
- ✅ Stage 3 landed (cyberos_index — frontmatter-based indexes work on encrypted memories per Q4 recommendation)
- ✅ Stage 4 landed (cyberos_export — bundles include encrypted memories naturally)

Stage 5 is unblocked from a tooling perspective. The decision is purely policy.

---

## Reply format

You can reply with any of:

- **"Go with your recs"** — I ship the defaults above.
- **"Q1=c Q2=a Q3=2-of-3 Q4=a Q5=b"** — terse override of any subset.
- **A free-form message** answering whichever questions you have opinions on; I'll ask follow-ups for the rest.

Or **"defer Stage 5"** — I close it as parked and the local-optimization roadmap is functionally complete with Stages 1, 2, 3, 4 shipped + Stage 6 proposal authored.

---

## Part 22 — Proposals: Stage 5 protocol upgrade (LANDED)

*Inlined historical proposal. Original at `docs/proposals/STAGE-5-PROTOCOL-UPGRADE.md` (deleted in consolidation). LANDED 2026-05-10 as `sha256:d3ce97…`.*

### Stage 5 — Protocol Upgrade Proposal

**Status**: Draft, ready for §0.5 chat-turn approval
**Source plan**: `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` Stage 5
**Decisions baseline**: `docs/proposals/STAGE-5-OPEN-QUESTIONS.md` — defaults selected via "go with your recs" (2026-05-10)
**Targets**: AGENTS.md §4.6, §5.1 (frontmatter extension), §5.6 (new), §6 (manifest extension), §7.1 (audit op enum extension), §9.3 (clarification), §17.6 (cross-link)

This document is the **exact prose to add/modify in AGENTS.md** when you approve Stage 5. Reflects your decisions:

- **Q1 = (c) Sensitive-plus-private** — encrypt `member/<self>/private/` + all `personnel | client` classification
- **Q2 = (c) Both — HW key OR Argon2id passphrase** — both accepted; passphrase enforced ≥16 chars + zxcvbn ≥3
- **Q3 = 3-of-5 Shamir, wizard-time holders, no deadman switch**
- **Q4 = (a) Body-only encryption** — frontmatter stays plaintext for index compatibility
- **Q5 = (c) User-paced migration via `--migrate-batch <N>`** (default 50)

To adopt: paste the §0.5 approval phrase in chat with the SHA after Changes A–F apply.

---

## Change A — §5.6 (new) at-rest encryption envelope

### Where

`docs/CyberOS-AGENTS.md` after §5.5 (Resource caps), before §6 (manifest schema). Insert new §5.6.

### Insert

```markdown
## 5.6 At-rest encryption (opt-in)

When `manifest.encryption_policy.enabled = true`, memories matching the policy's
scope filter are stored as XChaCha20-Poly1305 ciphertext in the body of the
memory file. Frontmatter stays plaintext (per §5.6.4 — preserves §5.1 schema
verifiability and Stage 3 indexing).

### 5.6.1 Encryption envelope (per-file)

Each encrypted memory file follows the §5.1 frontmatter shape with one new
required field set:

```yaml
encrypted: true
encryption:
  algorithm: xchacha20poly1305-ietf
  nonce: <base64 of 24 random bytes>
  aad: sha256(<memory_id> || <last_updated_at>)   # binds nonce to identity
```

Body is `base64(ciphertext || 16-byte tag)`. Plaintext recovered by:

```
plaintext = chacha20_decrypt(
    key      = master_key_derived_per_§5.6.2,
    nonce    = base64_decode(frontmatter.encryption.nonce),
    aad      = sha256_hex(memory_id || last_updated_at),
    body     = base64_decode(file.body),
)
```

Key reuse across files is permitted iff nonces are distinct (24-byte random
nonces collide with probability ~2⁻⁹⁶, far below any practical bound).

### 5.6.2 Key derivation

Master key derived via HKDF-SHA256 from one of two sources, both accepted when
configured:

- **Hardware-bound (preferred path):**
  - macOS: Apple Secure Enclave key (Touch ID prompt at first decrypt of session)
  - Windows: TPM 2.0 key (Windows Hello)
  - Linux: TPM 2.0 via `tpm2-tools` OR FIDO2 hmac-secret
- **Passphrase fallback (Argon2id):**
  - parameters: t=3, m=64MiB, p=4 (per RFC 9106 recommendation)
  - passphrase MUST satisfy: ≥16 chars AND zxcvbn score ≥3 at enable time
  - cached in memory for the session; never written to disk

Key cached in process memory only; never persisted in plaintext. Lost
key (both HW unavailable AND passphrase forgotten) → recover via §5.6.3.

### 5.6.3 Shamir 3-of-5 recovery escrow (mandatory)

Encryption-enable refuses to flip `enabled = true` until 5 fragments of the
master key have been generated via Shamir Secret Sharing (3-of-5 threshold)
AND the user has confirmed distribution to 5 holders.

Fragment fingerprints + holder labels + creation timestamps are recorded in
`meta/key-policy.md`. The fragments themselves NEVER enter `.cyberos-memory/`.

Recovery flow (under MAINTENANCE mode §8.8):
1. User collects ≥3 fragments out-of-band
2. `cyberos-encrypt recover` accepts fragments via stdin/QR/base32 paste
3. Master key reconstructed; verified against fingerprint pinned in
   `meta/key-policy.md`
4. `op:"key_recovery_initiated"` audit row appended at fragment intake
5. `op:"key_recovered"` audit row appended on successful reconstruction

Fragment rotation (refresh the 5 fragments without changing the master key):
- `op:"shamir_rotation"` audit row records the new fingerprint set
- Old fragments become useless once the new set is distributed

### 5.6.4 Indexability

Frontmatter remains plaintext so that:
- `cyberos_validate.py` verifies §5.1 schema + chain integrity without the key
- `cyberos_index.py` builds tag/relationship/source-SHA indices over encrypted
  memories
- `cyberos_doctor.py` repairs encrypted memories' chain consistency without
  decrypting bodies

The §9.3 denylist remains structural — encryption does NOT soften it. Comp,
ESOP, gov-IDs, raw secrets, special-category PII are still forbidden from
ANY storage form regardless of `encryption_policy`.

### 5.6.5 Audit-chain compatibility

Audit rows over encrypted memories store `after_hash` over the **plaintext**
body (computed at write time, before encryption). This preserves chain LINK
integrity when reading the BRAIN with the key. Without the key, chain
verification is degraded: LINK invariant remains verifiable, but plaintext
reconstruction for spot-verification requires the key.
```

### Backward compat

`encrypted: true` is a new optional frontmatter field; older agents trip
`INCOMPATIBLE` per §13.0 forward-compat tripwire (intended). Stores with
encryption disabled remain bit-identical to pre-Stage-5 stores.

---

## Change B — §6 manifest extensions

### Where

`docs/CyberOS-AGENTS.md` §6, inside the `manifest.json` schema block. Add new top-level fields after the existing `compaction_policy` block (Stage 6 already added it).

### Insert

```json
"encryption_policy": {
  "enabled": false,
  "scopes": ["member:<self>/private", "classification:personnel", "classification:client"],
  "algorithm": "xchacha20poly1305-ietf",
  "key_derivation": "hkdf-sha256-from-hardware-bound",
  "fallback_kdf": "argon2id-t3-m64-p4",
  "passphrase_strength_minimum": {"min_chars": 16, "zxcvbn_score": 3}
},
"shamir_fragments": {
  "threshold": 3,
  "total": 5,
  "master_key_fingerprint": null,
  "fragments": []
}
```

Each entry in `shamir_fragments.fragments` is `{label, fingerprint, created_at, distributed_at|null}`.

### Add to §6 explanation paragraphs

> **`encryption_policy`** — opt-in at-rest encryption per §5.6. Default
> `enabled: false`. Mutating any field requires the wizard flow at
> `runtime/tools/cyberos_encrypt.py enable` or chat-turn approval per §0.5.
> The `scopes` list uses the syntax `<scope-pattern>` for paths or
> `classification:<class>` for classification-keyed selection. Memories
> matching ANY entry are encrypted.
>
> **`shamir_fragments`** — recovery-escrow registry per §5.6.3. Default
> empty array. Fragments themselves are NEVER stored here — only their
> fingerprints. Threshold and total are pinned at enable time and rotated
> only via `op:"shamir_rotation"`.

---

## Change C — §7.1 audit op enum extension

### Where

`docs/CyberOS-AGENTS.md` §7.1, in the `op:` field's enum list.

### Replace

The existing op enum gains seven new values:

```
"op": "session.start|session.end|create|str_replace|insert|delete|rename|view|rejected|revert|corrects|consolidation_run|export|import|skipped-by-user|lock_recovered|protocol_upgrade|protocol_rollback|health_check|warn|drift_candidate|shallow_candidate|maintenance.start|maintenance.end|ledger_compact|ledger_decompact|encryption_policy_change|key_rotation|key_recovery_initiated|key_recovered|shamir_rotation|shamir_distribution_confirmed"
```

Notes:
- `encryption_policy_change` — emitted on enable/disable
- `key_rotation` — re-derive master key (e.g., HW key replaced); existing
  encrypted bodies re-encrypted with new key over multiple sessions via
  `--migrate-batch`
- `key_recovery_initiated` — fragment intake started
- `key_recovered` — master key reconstructed
- `shamir_rotation` — refresh fragment set without changing master key
- `shamir_distribution_confirmed` — emitted per fragment as user confirms
  distribution to its holder

### Backward compat

Older agents that don't recognise these op values will fail their op-enum
validation and emit `op:"rejected" reason:"unknown-op"`. This is the intended
forward-compat behaviour.

---

## Change D — §4.6 tombstone amendment for encrypted memories

### Where

`docs/CyberOS-AGENTS.md` §4.6, after the existing tombstone description.

### Insert

```markdown
**Encrypted memories.** `delete` on an encrypted memory tombstones the
frontmatter as usual; the encrypted body remains base64-ciphertext. Tombstoned
encrypted memories are decrypted ONLY during MAINTENANCE-mode hard-erase flows
(per §0.6 right-to-erasure documentation). Routine BRAIN reads SKIP tombstoned
encrypted bodies — no decryption attempt.
```

---

## Change E — §9.3 denylist clarification

### Where

`docs/CyberOS-AGENTS.md` §9.3, after the existing denylist enumeration, add a paragraph clarifying the encryption boundary.

### Insert

```markdown
**Encryption is NOT a denylist softener.** When `manifest.encryption_policy.enabled = true`
(§5.6), the encryption envelope protects classification-eligible content from
disk-level snooping. It does NOT change what content is allowed to be written.
The denylist categories above (compensation, ESOP, gov IDs, bank/card, home
addresses, health PII, secrets, external-party PII without consent) remain
forbidden from ANY storage form — encrypted or plaintext. The §4.2 content
gate runs BEFORE the encryption envelope; denylist hits are rejected before
any cryptographic operation.
```

---

## Change F — §17.6 cross-link

### Where

`docs/CyberOS-AGENTS.md` §17.6 (existing forward-reference about key management).

### Replace

The existing line:

> Cryptographic key rotation for `subject:<id>` Ed25519 keys (key-management
> policy belongs in `meta/key-policy.md`, not here).

### With

> Cryptographic key rotation for `subject:<id>` Ed25519 signing keys AND for
> `manifest.encryption_policy` master keys (per §5.6.2) belongs in
> `meta/key-policy.md`. Rotation events are audited via `op:"key_rotation"`
> + `op:"shamir_rotation"` per §7.1.

---

## Order of operations to land Stage 5

Per AGENTS.md §0.5 + §0.6:

1. **Edit AGENTS.md** with Changes A–F.
2. **Archive prior verbatim** to `meta/protocol-history/AGENTS-<before_sha256>.md`.
3. **`str_replace` on `manifest.json`** to add `encryption_policy` (default `enabled: false`) and `shamir_fragments` (default empty). Do NOT enable encryption yet — that's a separate `cyberos-encrypt enable` wizard step.
4. **Append `op:"protocol_upgrade"`** to the audit ledger.
5. **Auto-trigger §8.7 self-audit pass** per §0.5 step 4.
6. **Update CHANGELOGs** per §0.6 (AGENTS, PRD, SRS).
7. **Write `memories/refinements/REF-NNN-stage-5-encryption.md`** per §0.4.
8. **Add new DEC entry** in PRD §5.9 / Part 13: `DEC-NNN — Stage 5: At-rest encryption + Shamir 3-of-5 escrow`.

## Approval phrase to land

In chat:

> *"approve protocol upgrade to sha256:<computed-sha-after-applying-A-through-F>"*

For preview:

> *"preview protocol upgrade for Stage 5"*

---

## Implementation work that follows landing

After Stage 5 lands via §0.5, the following implementation work becomes possible (no further §0.5 approvals needed for any of these):

### `runtime/tools/cyberos_encrypt.py` (new)

Single-file Python tool, ~600 LOC. Subcommands:

- **`enable`** — wizard flow:
  1. Detect HW key (Secure Enclave / TPM / FIDO2) → pick available
  2. If no HW or user opts for passphrase: prompt + zxcvbn + Argon2id derive
  3. Generate 32-byte master key (CSPRNG)
  4. Shamir-split via `vsss-rs` (Rust) / `secretsharing` (Python) — 3-of-5
  5. Render 5 fragments as printable QR codes + base32 strings
  6. Walk holder distribution wizard (label each fragment, prompt for
     "press y when handed off")
  7. Append `op:"shamir_distribution_confirmed"` per fragment
  8. Verify master-key fingerprint pinned in `meta/key-policy.md`
  9. Atomic `str_replace` on manifest.json: flip `encryption_policy.enabled = true`
  10. Append `op:"encryption_policy_change"` audit row

- **`disable`** — decrypt all in-scope memories → re-write plaintext → flip
  `enabled = false` → audit `op:"encryption_policy_change"`

- **`migrate-batch <N>`** — encrypt N more in-scope plaintext memories under
  one MAINTENANCE-mode envelope; default N=50; surfaces "X of Y plaintext
  memories migrated" progress

- **`rotate-shamir`** — generate fresh 5 fragments without changing master key;
  audit `op:"shamir_rotation"` + 5× `op:"shamir_distribution_confirmed"`

- **`recover`** — accept ≥3 fragments via stdin/QR/base32 paste under
  MAINTENANCE mode; reconstruct + verify master key against pinned
  fingerprint; audit `op:"key_recovery_initiated"` then `op:"key_recovered"`

- **`status`** — show encryption coverage stats (encrypted vs plaintext per
  scope, key-derivation source, Shamir fragment registry)

### `runtime/tools/cyberos_validate.py` extension

- Recognise `encrypted: true` frontmatter
- Verify chain integrity using stored `after_hash` (over plaintext) without
  needing the key
- New finding: `encryption-aad-mismatch` if `aad` doesn't match
  `sha256(memory_id || last_updated_at)`
- New finding: `shamir-fingerprint-missing` if `enabled = true` but
  `shamir_fragments.master_key_fingerprint` is null

### `runtime/tools/cyberos_doctor.py` extension

- New repair op: `R6-rotate-master-key` — re-derive master from new HW key
  source after hardware change (e.g., new Mac); re-encrypts all in-scope
  memories under MAINTENANCE mode

### `docs/cookbook/encryption-and-recovery.md` (new)

Operational guide:
- Enable wizard walkthrough with screenshots
- Holder selection guidance (suggested patterns: founder + spouse + business
  partner + lawyer + geographically-distant trusted contact)
- Recovery flow walkthrough (what to do when HW key is lost AND passphrase
  is forgotten)
- Migration playbook (`migrate-batch 50` cadence; how to monitor progress)
- Hardware key change procedure (`rotate-master-key` after Mac replacement)
- "What can an attacker see with the encrypted store but no key?" threat model

---

## What stays unchanged

- The §9.3 denylist remains structural; encryption never softens it
- Audit chain LINK invariant (§7.2) preserved across encrypted memories
- §11 export determinism preserved — encrypted memories serialise byte-stably
- Stage 6 Merkle checkpoints + ledger compaction work unchanged on encrypted
  memories (the ledger is metadata-only; doesn't carry plaintext bodies)
- §13.0 state classifier rules unchanged

---

## Decision points already resolved

Per the "go with your recs" approval (2026-05-10), all five Q&A from
`STAGE-5-OPEN-QUESTIONS.md` are pinned to my recommendations. If you want to
revisit any before final §0.5 approval, edit Changes A–F above and re-issue
the preview.

---

## Part 23 — Proposals: Stage 6 protocol upgrade (LANDED)

*Inlined historical proposal. Original at `docs/proposals/STAGE-6-PROTOCOL-UPGRADE.md` (deleted in consolidation). LANDED 2026-05-10 as `sha256:77eda21…`.*

### Stage 6 — Protocol Upgrade Proposal

**Status**: Draft, ready for §0.5 chat-turn approval
**Source plan**: `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` Stage 6
**Targets**: AGENTS.md §4.9, §7 (new §7.6 + §7.7), §8 (new §8.9), §8.7

This document is the **exact prose to add/modify in AGENTS.md** when you approve Stage 6. Stage 6 is the most invasive protocol change in the local-optimization roadmap because it touches: the audit ledger format (Merkle checkpoints + compaction), the consolidation phase set (new phase 8.9), and the lock semantics (read-shared vs write-exclusive split).

To adopt: paste the §0.5 approval phrase in chat with the SHA after Changes A–D apply.

---

## Change A — Merkle checkpoints (§7.6 new)

### Where

`docs/CyberOS-AGENTS.md` after §7.5 (`op:"corrects"` semantics), insert new section §7.6.

### Insert

```markdown
### 7.6 Merkle checkpoints

Every successful `op:"consolidation_run"` writes an additional `merkle_root` field
into the audit row, recording the SHA-256 root of a Merkle tree built over the
prior N audit rows since the previous checkpoint (or genesis, on first run).

**Merkle tree construction (deterministic):**
- Leaves: each row's `chain` value (raw bytes, prefix `sha256:` stripped, hex-decoded
  to 32 bytes).
- Pairing: pad odd levels by duplicating the last leaf.
- Internal: `sha256(left || right)` (raw bytes).
- Root: prefix `sha256:` + hex.

**Field schema extension** (§7.1 row):
- `merkle_root: <sha256:…>` — set ONLY on `op:"consolidation_run"` rows;
  null/absent on all other ops.
- Validators that don't recognise the field treat it as an opaque extension
  per §13.0 forward-compat rules.

**Verification path:**
- Walk audit rows in file order.
- At each `op:"consolidation_run"` row, recompute the Merkle root over the rows
  since the previous checkpoint (or genesis). Verify equality with the stored
  `merkle_root`. Mismatch → CRITICAL `merkle-checkpoint-divergence`.
- Spot-verification of a prefix is O(log N): walk the row of interest's
  inclusion path against the next checkpoint's stored root.

**Why:** chain prefix verification becomes O(log N) instead of O(N) full-walk.
The linear `chain` LINK invariant remains canonical (the Merkle root is a
*derived* index, not a replacement). Stage 6 §8.9 ledger compaction depends on
this primitive.
```

### Backward compat

Additive field on consolidation_run rows. Older agents trip `INCOMPATIBLE` per
§13.0 (intended forward-compat tripwire). Existing chains continue to verify
via the LINK invariant; the Merkle root only adds an additional verification
path.

---

## Change B — Audit ledger compaction (§7.7 new)

### Where

`docs/CyberOS-AGENTS.md` after the new §7.6, insert §7.7.

### Insert

```markdown
### 7.7 Audit ledger compaction (sev-1)

Once a ledger month has been Merkle-checkpointed (§7.6) AND is older than the
retention horizon (default 12 months; configurable via
`manifest.compaction_policy.minimum_age_months`), the per-row JSONL MAY be
collapsed into a per-memory `final_state.jsonl` plus a Merkle proof — preserving
spot-verifiability without retaining every intermediate row.

**Compaction is opt-in.** Triggered ONLY by the explicit user phrase
*"compact ledger older than `<YYYY-MM-DD>`"* in the current chat turn. The
phrase MUST include the cutoff date so silent expansions are impossible (per
§0.5 chat-turn-approval-only mutation pattern).

**Compaction outputs:**
- `audit/<YYYY-MM>.compacted.jsonl` — one row per memory_id, carrying:
  - `memory_id`
  - `final_op` — `tombstoned | active`
  - `final_chain` — the chain of the last op against this memory_id in the
    compacted period
  - `final_audit_id`, `final_ts`
  - `merkle_proof` — inclusion path against the period's Merkle root
- `archive/<YYYY-MM>.jsonl.zst` — zstd-compressed verbatim copy of the
  original JSONL ledger. Source of truth for re-expansion.

**Compaction is reversible.** `cyberos-doctor decompact-ledger
--month <YYYY-MM>` re-expands the archive, atomically replacing the
`<YYYY-MM>.compacted.jsonl` with the original JSONL. Audited as
`op:"ledger_decompact"`.

**Compaction itself is audited.** On invocation:
- `op:"ledger_compact"` row appended at the live ledger tail with
  - `before_hash` over the original JSONL
  - `after_hash` over the compacted output
  - `reason` carrying the cutoff date and the user phrase verbatim

**Forbidden by §0.2.** Mutating `compaction_policy` outside the chat-turn
approval phrase is forbidden.

**Why:** typical disk savings ~80% on year-old ledgers. Spot-verification of
any row in the compacted period via Merkle proof + the period's checkpoint root.
```

### Backward compat

Compacted ledgers are recognised by the `.compacted.jsonl` filename suffix.
Validators MUST handle both forms; agents that don't recognise compaction
trip §13.0 INCOMPATIBLE on `audit/*.compacted.jsonl` files.

---

## Change C — Consolidation phase 8.9 (§8 extension)

### Where

`docs/CyberOS-AGENTS.md` after §8.8 (MAINTENANCE mode), insert §8.9.

### Insert

```markdown
### 8.9 Ledger compaction (opt-in, user-triggered)

Phase 8.9 is NOT part of the routine consolidation cycle (§8.1–§8.7). It runs
**only** on the explicit user phrase *"compact ledger older than `<YYYY-MM-DD>`"*
per §7.7.

**Pre-conditions** (refuse to compact if violated):
1. The cutoff month must have a `op:"consolidation_run"` row carrying a
   `merkle_root` per §7.6 (otherwise no checkpoint to anchor proofs against).
2. The cutoff month must be older than `manifest.compaction_policy.minimum_age_months`
   (default 12).
3. No CRITICAL findings from §8.7 phase 4 (audit chain integrity) for the
   period being compacted.

**Phase steps:**
1. Acquire `.lock` (exclusive).
2. Verify pre-conditions; abort with `op:"rejected" reason:"compaction-precondition:<which>"` on failure.
3. Build the per-memory `final_state.jsonl` from a single forward walk of the
   period's rows.
4. Compute Merkle inclusion proofs for each memory's `final_audit_id`.
5. zstd-compress the original JSONL into `archive/<YYYY-MM>.jsonl.zst`.
6. Atomic rename `audit/<YYYY-MM>.jsonl` → `audit/<YYYY-MM>.compacted.jsonl`
   (keeping the same path so older agents trip INCOMPATIBLE if they encounter
   a compacted form they don't recognise).
7. Append `op:"ledger_compact"` to the live ledger.
8. Release `.lock`.

**Re-expansion** (reverse of compaction) follows the inverse steps under
MAINTENANCE mode (§8.8); see §7.7.
```

### Backward compat

Phase 8.9 is opt-in and never auto-runs. Stores that never invoke compaction
remain bit-identical to pre-Stage-6 stores.

---

## Change D — `.lock.shared` (§4.9 amendment)

### Where

`docs/CyberOS-AGENTS.md` §4.9 — add a new sub-section for shared-read locking.

### Insert (after the existing "Held during" sentence)

```markdown
### 4.9.1 Shared-read lock (§4.9 extension)

Concurrent agents may safely run **read-only** operations against the same
store while one agent holds `.lock` for consolidation (§8.1–§8.4). The
shared-read mechanism uses a sibling file `.lock.shared`:

**Acquisition (read-only path):**
- POSIX: `flock(.lock.shared, LOCK_SH | LOCK_NB)`
- Windows: `LockFileEx(.lock.shared, 0)` (shared-mode LockFileEx without exclusive flag)

**Compatibility with `.lock`:**
- Read-only ops (`view` per §4) acquire `.lock.shared` only.
- Mutation ops (§4 except `view`) acquire `.lock` (exclusive) and additionally
  block until all `.lock.shared` holders release.
- Consolidation phases §8.1–§8.4 acquire `.lock.shared` (allowing other agents
  to `view` concurrently); upgrade to exclusive `.lock` for §8.5 (manifest
  update), §8.6 (source-coverage write), §8.7 (health checkpoint).

**Stale recovery** for `.lock.shared`: same semantics as `.lock` (§4.9 stale
block), 5-minute timeout for cross-host recovery.

**Backward compat:** older agents that don't recognise `.lock.shared` continue
to use exclusive `.lock` for everything — they get correct semantics, just
without the concurrency benefit.
```

### Backward compat

Older agents that don't honour `.lock.shared` simply ignore it; they continue
to acquire `.lock` exclusive, which is always safe. Stage 6's improvement is
opportunistic.

---

## Change E — §8.7 phase 4 Merkle verification

### Where

`docs/CyberOS-AGENTS.md` §8.7, in the "Six checks, in order:" list, extend check 4 (already amended in Stage 1).

### Replace

The current Stage-1 amended bullet 4. Replace with:

```markdown
4. **Audit chain integrity** — verify LINK integrity end-to-end (not just
   incremental like §4.7): for each row N, confirm
   `row[N].prev_chain == row[N-1].chain`. LINK integrity is the authoritative
   invariant per §7.2's cross-writer-version compatibility clause. Hash
   recomputation (`chain == sha256_hex(canonical_json(row_without_chain_or_prev_chain) || prev_chain)`
   per §7.2) MAY be performed and reported at INFO severity; recomputation
   differences across writer versions are NOT chain breaks. Confirm
   `manifest.audit_chain_head` is reachable in the ledger.
   **Additionally, if `manifest.reconciliation_checkpoint` is set, confirm
   `checkpoint.audit_id` resolves to a row in the ledger AND `checkpoint.chain`
   matches that row's `chain`. Mismatch → `CRITICAL stale-checkpoint`; freezes
   writes until reconciled per §4.7 fallback.**
   **Stage 6 extension:** for every `op:"consolidation_run"` row carrying a
   `merkle_root` field, recompute the Merkle root over the rows since the
   previous checkpoint and verify equality. Mismatch → `CRITICAL merkle-checkpoint-divergence`.
   For every compacted ledger (`audit/<YYYY-MM>.compacted.jsonl`), verify each
   row's `merkle_proof` against the period's checkpoint root. Mismatch →
   `CRITICAL merkle-proof-divergence`.
```

---

## Order of operations to land Stage 6

Per AGENTS.md §0.5 + §0.6:

1. **Edit AGENTS.md** with Changes A–E above.
2. **Archive prior verbatim** to `meta/protocol-history/AGENTS-<before_sha256>.md` per §0.5 step 1.
3. **`str_replace` on `manifest.json`** to update `manifest.protocol.sha256`, `approved_at`, `approved_by`. Also add (Stage 6) `manifest.compaction_policy = {minimum_age_months: 12}` if customisation desired.
4. **Append `op:"protocol_upgrade"`** with `before_hash`/`after_hash` for the manifest.
5. **Auto-trigger §8.7 self-audit pass** per §0.5 step 4. Output → `meta/health/<YYYY-MM-DD>-<sha>-postupgrade.md`.
6. **Update CHANGELOGs** per §0.6.
7. **Write `memories/refinements/REF-NNN-stage-6-long-term-health.md`** — refinement record per §0.4.
8. **Add new DEC entry** in PRD §5.9 / Part 13: `DEC-NNN — Stage 6: Merkle checkpoints + ledger compaction + lock-share split`.

## Approval phrase to land

In chat:

> *"approve protocol upgrade to sha256:<computed-sha-after-applying-A-through-E>"*

The computed SHA must be derived from a candidate file via `runtime/tools/canonical_sha.py`. To preview:

> *"preview protocol upgrade for Stage 6"*

---

## Implementation work that follows landing

After Stage 6 lands via §0.5, the following implementation work becomes possible:

**Validator extension** (cyberos_validate.py)
- Add `_check_merkle_checkpoints()` method walking consolidation_run rows + recomputing roots
- Add `_check_compacted_ledger()` for `audit/*.compacted.jsonl` files
- Test fixtures #17 (merkle-checkpoint-divergence) and #18 (compacted-ledger-good)

**Doctor extension** (cyberos_doctor.py)
- New repair op: R5-rebuild-merkle-checkpoint (re-derive merkle_root from period rows)
- New CLI: `cyberos-doctor decompact-ledger --month <YYYY-MM>` for §7.7 reverse path
- New CLI: `cyberos-doctor verify-merkle <chain>` for spot-verification of a specific row

**Indexer extension** (cyberos_index.py)
- New table: `merkle_checkpoints(audit_id, root, period_start_audit_id, period_end_audit_id)`
- New query: `cyberos-index query merkle-proof <chain>` returns the inclusion path

**Cookbook updates**
- `docs/cookbook/ledger-compaction.md` — when to compact, how to verify a compacted period

**Decision points for landing Stage 6 (none — all auto-applicable)**

Stage 6 changes are mechanically applicable from the proposal text. There are no decision points beyond "yes/no, approve via the §0.5 phrase." Compaction is opt-in, lock-shared is opportunistic, Merkle is additive.

This contrasts with Stage 5 (encryption + Shamir) which has multiple decision points; see `docs/proposals/STAGE-5-OPEN-QUESTIONS.md` once that document is authored.

---

## Part 24 — Proposals: Bundle M (LANDED) + Bundle N pending

*Inlined historical proposal. Original at `docs/proposals/STAGE-7-BUNDLE-M-PROPOSAL.md` (deleted in consolidation). Bundle M LANDED 2026-05-10 as `sha256:9bec84…` (Changes A-D applied). Changes E (§0.5 split) + F (paragraph compression) deferred to Bundle N.*

### Bundle M — AGENTS.md refinement pass (Stage 7 / post-local-optimization)

**Status**: Draft, ready for §0.5 chat-turn approval
**Source plan**: 2026-05-10 AGENTS.md scan (six refinement candidates surfaced during the all-tracks final pass)
**Targets**: AGENTS.md §0.5, §4.10/§4.11 (merge), §5.1, §8 heading, §17, paragraph-level compression throughout
**Functional impact**: ZERO — Bundle M is purely descriptive cleanup. No new ops, no schema changes, no validator changes. Result: ~950-line / ~85 KB AGENTS.md (down from 1210 / 108 KB) without functional drift.

To adopt: paste the §0.5 approval phrase in chat with the SHA after Changes A–F apply.

---

## Why this bundle is non-functional

Every change in Bundle M is one of:

1. **Header text correction** — e.g. "28 fields" → "30 fields" reflecting Stage 5's additions (`encrypted` + `encryption` block). Functionally the schema already accepts these; the header just lagged.
2. **Heading text correction** — e.g. §8 says "7 phases" but §8.9 exists post-Stage-6. Title clarification.
3. **Section consolidation** — §4.10 + §4.11 cover related concerns (sequential walk + announcement); merging into one §4.10 with sub-sections cleans cross-references.
4. **Forward-reference compression** — §17 has 50+ lines on multi-machine sync that's deferred to BRAIN module P1. Compress to 1 paragraph + cross-link to EVOLUTION.md Stage 4.
5. **Section split for clarity** — §0.5 mixes approval flow + signing-key TOFU + three-way conflict in 52 lines. Split into §0.5 (approval flow) + §0.5.1 (signing-key TOFU, deferred) + §0.5.2 (three-way conflict, deferred).
6. **Paragraph compression** — 55 paragraphs over 500 chars; many can be split into bullet structure or shorter prose without losing rules.

None of these change what is or isn't permitted. Two agents reading pre-Bundle-M AGENTS.md and post-Bundle-M AGENTS.md will reach identical accept/reject decisions on every input.

---

## Change A — §5.1 schema field-count header update

### Where

`docs/CyberOS-AGENTS.md` §5.1 — the heading text.

### Replace

```markdown
### 5.1 Frontmatter schema (only these 28 fields are permitted)
```

### With

```markdown
### 5.1 Frontmatter schema (closed set; 28 base fields + Stage 5 encryption block)

The schema below lists 28 base fields. Stage 5 (sha256:d3ce97…) added two
encryption-envelope fields (`encrypted: bool`, `encryption: {algorithm, nonce, aad}`)
that apply only when `manifest.encryption_policy.enabled = true` per §5.6.
The closed-set rule applies to the union of these; new fields beyond this
union require §0.5 protocol upgrade.
```

### Rationale (not landing in AGENTS.md; documentation only)

Pre-Bundle-M, the header text said "28 fields are permitted" but Stage 5 added `encrypted` + `encryption` block. The schema actually accepts these; the header just hadn't been updated. This change reconciles header to current reality without changing what's accepted.

---

## Change B — §8 heading clarification

### Where

`docs/CyberOS-AGENTS.md` §8 — the H2 heading.

### Replace

```markdown
## 8. Consolidation (7 phases — only on session-end, ≥25 rows since last, or user command)
```

### With

```markdown
## 8. Consolidation (7 routine phases + §8.9 user-triggered ledger compaction; only on session-end, ≥25 rows since last, or user command)
```

### Rationale

Stage 6 added §8.9 (ledger compaction) but the §8 heading still says "7 phases." Phase 8.9 is opt-in/phrase-triggered, not part of the routine cycle — the heading should reflect both.

---

## Change C — §4.10/§4.11 merge

### Where

`docs/CyberOS-AGENTS.md` §4.10 + §4.11.

### Replace

The current §4.10 (Ingestion completeness) and §4.11 (Token-budget transparency) — they cover sequential-walk and announcement of the same multi-message ingestion process.

### With

A single §4.10 with two sub-sections:

```markdown
### 4.10 Ingestion completeness (sev-1) — read-side counterpart to §4.4

#### 4.10.1 Sequential walk + coverage check

[existing §4.10 content unchanged]

#### 4.10.2 Token-budget transparency for >500-line sources

[existing §4.11 content unchanged]
```

### Rationale

Reduces section-count noise. The two rules belong together (read-side discipline for multi-message ingestion). Cross-references that pointed at §4.11 update to §4.10.2.

---

## Change D — §17 forward-references compression

### Where

`docs/CyberOS-AGENTS.md` §17 — multi-machine sync via BRAIN module P1.

### Replace

The current 50+ lines of §17.5 (publish flow forward reference) and §17.6 (what this protocol does NOT define).

### With

Compress to ~10 lines:

```markdown
### 17.5 Multi-machine sync (forward reference)

`sync_class` is metadata-only until the BRAIN service P1 ships. Until then,
`publishable` and `shared` memories stay local; the field is recorded for
future use. Multi-machine semantics, conflict resolution between subjects,
and the publish/pull wire protocol are deferred to the BRAIN module's
domain. Tracking: `docs/CyberOS-AGENTS.EVOLUTION.md` Stage 4.

### 17.6 What this protocol does NOT define

- BRAIN service wire protocol (Yjs+Automerge over WebSocket subgraph; deferred to P1)
- Per-tenant ACLs (PORTAL module)
- Concurrent-edit resolution between subjects (BRAIN module decision)
- Cryptographic key rotation policy (`meta/key-policy.md` per §17.5 amendment)
```

### Rationale

The current §17.5/§17.6 are aspirational forward-references that belong in EVOLUTION.md, not AGENTS.md. Compression makes the protocol focus on what it actually governs today.

---

## Change E — §0.5 section split

### Where

`docs/CyberOS-AGENTS.md` §0.5 — currently 52 lines mixing approval flow, signing-key TOFU, and three-way-conflict logic.

### Replace

Single §0.5 block with mixed concerns.

### With

Split into three sub-sections:

```markdown
### 0.5 Protocol update policy (sev-0)

[approval phrase + canonical SHA computation + chat-turn approval flow only]

### 0.5.1 Signing-key TOFU (deferred — pre-BRAIN-P1, no canonical out-of-band source mandated)

[the existing TOFU/sigstore/Rekor/DNS-TXT prose]

### 0.5.2 Three-way protocol conflict (deferred — applies only when upstream releases exist)

[the existing three-way conflict resolution prose]
```

### Rationale

The current §0.5 is dense because it tries to handle four scenarios (single-user approval, signing-key trust establishment, three-way upstream conflict, rollback) in one section. Splitting clarifies that everyday §0.5 is the chat-turn approval flow; the other concerns are deferred until they apply.

---

## Change F — Paragraph compression throughout

### Where

55 paragraphs over 500 chars across §0.2, §6 manifest description, §7.2 RFC 8785 prose, §8.7 self-audit, §13.0 state classifier, others.

### Replace

Long single-paragraph rule descriptions.

### With

Bulleted lists where the paragraph already has list-like structure, broken paragraphs where it doesn't. Lossless; pure formatting.

### Examples (illustrative, not exhaustive)

§7.2 RFC 8785 paragraph currently runs ~600 chars. Bullet form preserves all rules:

```markdown
**Canonicalisation algorithm: RFC 8785 JCS.** Implementations MUST conform.
Where this protocol's wording differs, RFC 8785 wins.

- **Object key ordering**: lexicographic on UTF-16 code units (RFC 8785 §3.2.3)
- **Whitespace**: none — no spaces between separators; no leading/trailing whitespace; no trailing newline
- **Separators**: `,` between items; `:` between key and value (single ASCII bytes; never with surrounding whitespace)
- **Strings**: UTF-8 encoded; non-ASCII preserved verbatim (NOT `\uXXXX`-escaped); ...
- **Numbers**: ECMAScript `Number.prototype.toString` per RFC 8785 §3.2.2.3 — shortest decimal that round-trips through IEEE-754 double
- **Booleans/null**: lowercase only
- **Arrays**: order-preserving; no canonicalisation of element order
- **No duplicate keys** in any object
```

### Rationale

Easier scanning, identical normative content. CI verification: regenerate AGENTS.md and confirm SHA — paragraph compression should not change which paragraphs the extractor keeps.

---

## Order of operations to land Bundle M

Per AGENTS.md §0.5 + §0.6:

1. **Edit AGENTS.md** with Changes A–F above. Manual editing because changes are textual + structural; not amenable to programmatic application like Stage 1/5/6 were.
2. **Archive prior verbatim** to `meta/protocol-history/AGENTS-<before_sha256>.md`.
3. **`str_replace` on `manifest.json`** to update `protocol.sha256`, `approved_at`, `approved_by`. No new manifest fields.
4. **Append `op:"protocol_upgrade"`** to the audit ledger.
5. **Auto-trigger §8.7 self-audit pass** per §0.5 step 4.
6. **Update CHANGELOGs** per §0.6: AGENTS, PRD, SRS.
7. **Write `memories/refinements/REF-NNN-bundle-m-refinement-pass.md`** per §0.4.
8. **No new DEC entry needed** — Bundle M is documentation cleanup, not a decision.
9. **Regenerate `AGENTS.md`** with `extract_agents_core.py --aggressive`. Should drop ~5KB / ~1500 tokens because compression in full = compression in core.
10. **Run validator self-test** (21 fixtures) — should still pass since no functional rules change.

## Approval phrase to land

In chat:

> *"approve protocol upgrade to sha256:<computed-sha-after-applying-A-through-F>"*

For preview:

> *"preview protocol upgrade for Bundle M"*

---

## Risk assessment

**Risk: A change accidentally drops a rule.** Mitigation: every change is documented with its before/after wording. The §8.7 self-audit pass after landing must report 0 CRITICAL findings on the live store (no schema invariant violations, no chain breaks).

**Risk: A change breaks a cross-reference.** Mitigation: §4.11 → §4.10.2 is the only renumbering; grep AGENTS.md for `§4.11` and verify all are updated.

**Risk: AGENTS.md regeneration produces drift.** Mitigation: regenerate after Bundle M; verify the resulting CORE still contains all required normative rules (manual diff review of the 17 KEEP_FULL sections).

**Risk: External documents reference §X.Y that moved.** Mitigation: 4 documents reference AGENTS.md sections (PRD, SRS, the cookbooks, REF-* memories). Bundle M renumbers only §4.11 → §4.10.2; grep all four sources, update cross-references in the same atomic upgrade.

## Why this is a single bundle

Each of A–F is independently small, but bundling them shares the §0.5 ceremony cost (one archive, one manifest update, one audit row, one §8.7 scan, one CHANGELOG entry). Six small upgrades = 6× ceremony; one Bundle M = 1× ceremony.

This pattern is the protocol's intended use: small refinements bundle, large amendments stand alone (Stage 1, 5, 6 each had their own §0.5 because each introduced new behaviour).

*This README is a living document. When AGENTS.md changes meaningfully (a new section, a new sync class, a new audit op), update the relevant Part above and append a one-line "README updated" entry to `CyberOS-AGENTS.CHANGELOG.md`. The README itself is informational and does not require §0.5 protocol approval; only AGENTS.md edits do.*

---

## Part 25 — Layer-1 architecture overview (operator reference)

> Parts 25–31 consolidate the per-aspect operator reference (formerly the
> standalone `CyberOS-LAYER-1-MANUAL.md`). Everything memory-related lives
> here now — protocol mental-model up top (Parts 1–24), operator surface
> here.

The Layer-1 surface is **three layers of code over one filesystem**:

```
┌──────────────────────────────────────────────────────────────┐
│  cyberos (umbrella CLI)         33 subcommands               │
│    runtime/tools/cyberos                                     │
└──────────────────────────────────────────────────────────────┘
              ↓ delegates via _delegate_to() helper
┌──────────────────────────────────────────────────────────────┐
│  Individual tool scripts                                     │
│    runtime/tools/cyberos_*.py        (16 standalone scripts) │
│    runtime/hooks/*.py                (gateguard, refinements)│
│    runtime/mcp/cyberos_brain_server.py  (read-only MCP)      │
│    runtime/tests/**                  (mutation, fuzz, etc.)  │
└──────────────────────────────────────────────────────────────┘
              ↓ read/write
┌──────────────────────────────────────────────────────────────┐
│  .cyberos-memory/   (the BRAIN)                              │
│    manifest.json + audit/ + memories/ + meta/ + persona/     │
│    company/ + (eventually) module/ + client/ + member/       │
└──────────────────────────────────────────────────────────────┘
```

**Three principles drive the design.**

1. **Real-filesystem-only (§0.1).** Every tool resolves `.cyberos-memory/`
   by walking up from CWD. Sandboxed agents reach the host filesystem
   via `$CYBEROS_HOST_MOUNT_PREFIX`.
2. **Append-only audit (§7).** Every state change appends a Merkle-chained
   row to `audit/<YYYY-MM>.jsonl`. The chain head lives in `manifest.json`.
3. **Validate before respond (§4.7 + §8.7).** Every session.start runs
   reconciliation + 6-phase self-audit before the agent answers anything.

> **Note on `§` references.** Where this manual writes "§N.N" it cites a
> section in `docs/CyberOS-AGENTS.md`. Where it writes "Aspect N.N" it cites
> the improvement catalog at `workbench/cyberos-layer1-deep-improvements.md`.
> Some `cyberos verify` findings emit a `[§12.1]`-style tag — this is a
> validator convention for "Aspect 12.1 pluggable-validator finding", not a
> literal AGENTS.md anchor. The two namespaces overlap by accident; tags in
> findings always trace back to the catalog Aspect.

**Authority hierarchy (§5.1).** `human-edited > human-confirmed >
llm-explicit > llm-implicit`. Strict; never promoted.

**Source-tier ordering (§9.6).** `company/locked-decisions.md` (tier 1)
beats everything else. Source tiers live in `manifest.source_tiers` and
are validated by `meta/validators/check-source-tiers.py`.

**Sync classes (§17).**
- `local-only` — never syncs.
- `publishable` — included in default sync bundles.
- `shared` — included; requires `consent.has_consent: true`.
- `client-visible` — included only with explicit `--include client-visible`.

---

## Part 26 — Per-aspect detail (Aspect 1.1 through 13.10)

### Aspect 1 — Operator CLI surface

#### 1.1 Single entry-point binary `cyberos`

- **Where:** `runtime/tools/cyberos` (the umbrella; calls 32 sub-tools via `_delegate_to()`)
- **Verify:** `cyberos --version` → `cyberos 0.1.0`
- **Pattern:** every subcommand either runs inline (`cmd_*` functions in
  the umbrella) or delegates to a sibling `cyberos_*.py` script.

#### 1.2 Interactive `add` wizard

- **Tool:** `runtime/tools/cyberos_add.py`
- **Templates read from:** `runtime/starter/templates/<TYPE>.md` (DEC, REF, FACT,
  PERSON, PROJECT, PREFERENCE, DRIFT)
- **Prompts:** slug, classification, authority, tags, sync_class, prov_source,
  prov_source_ref, freshness_tier
- **Auto-fills:** UUIDv7 memory_id, ts_now in ICT, subject id from env or
  git config, next NNN per bucket
- **Flags:** `--dry-run`, `--non-interactive`, `--auto-tags`, `--persona`
- **Atomic write:** stages to `.cyberos-memory/staging/`, then invokes
  `brain_writer.py write` with audit-row append + tmp+rename + fsync

#### 1.3 `--dry-run` on every mutating op

- **Audit (2026-05-12):**
  - `cyberos add --dry-run` — fully implemented; stages without writing
  - `cyberos sync import --dry-run` — fully implemented; reports without
    writing conflict markers
  - `cyberos doctor --repair --reason "<why>"` — required `--reason` is the
    de-facto sev-0 gate; no separate `--dry-run` needed because gate is
    already explicit
  - `cyberos panic --reason "<why>"` — same; reason-required gate
  - `cyberos encrypt enable/rotate-shamir` — already wizard-driven with
    confirmation prompts

#### 1.4 Shell tab completion

- **Files:** `runtime/completions/cyberos.{bash,zsh,fish}`
- **Install (bash):** `source /path/to/runtime/completions/cyberos.bash`
- **Install (zsh):** drop in `$fpath` as `_cyberos`
- **Install (fish):** `cp cyberos.fish ~/.config/fish/completions/`
- **Completes:** subcommands, enum values for `--classification`, REF-NNN
  for `council` and `eval`, `sync` + `mcp` subcommands

#### 1.5 `--explain` flag (`cyberos explain <subcmd>`)

- **Implemented as:** dedicated `cyberos explain` subcommand (clearer than
  cross-cutting flag)
- **Covers:** verify, add, sync, doctor, council, prune, export
- **Output:** `§-anchor | step name | brief description` for each step
- **Extension:** add a new subcommand by appending to the `EXPLAIN` dict
  in `runtime/tools/cyberos`

#### 1.6 `cyberos repl`

- **Tool:** `runtime/tools/cyberos_repl.py`
- **Forwards** each line to the umbrella binary as a subprocess (no
  session.start re-runs needed per command)
- **Meta-commands:** `.cd`, `.pwd`, `.last`, `.history`, `.save <path>`,
  `.env`, `.clear`, `.reload`, `.help`
- **Exit:** `exit`, `quit`, `q`, or Ctrl-D

### Aspect 2 — Operator dashboard & visibility

#### 2.1 4-operator-question dashboard

- **Inline in:** `runtime/tools/cyberos` → `_cmd_status_dashboard()`
- **Four questions:**
  - HEALTHY? — CRITICAL/WARN counts from validator
  - BOTTLENECK? — file-size cap headroom, drift candidates, oversize ledgers
  - CHANGED? — last 24h audit op counts
  - WHAT NOW? — 3 prioritised actions

#### 2.2 Trend lines on key metrics

- **Where:** `_compute_trends(brain)` inside `runtime/tools/cyberos`
- **Surfaced as:** `TRENDS  30-day rolling` section in `cyberos status`
- **Tracks:** memory net (creates − deletes), audit ops total + per-day,
  drift candidates surfaced in the last 30 days

#### 2.3 `cyberos status --weekly`

- **Function:** `_cmd_status_weekly()`
- **Framing per gstack `/landing-report`:**
  - LANDED — audit ops in last 7d
  - IN-FLIGHT — files staged at `.cyberos-memory/staging/`
  - QUEUED — drift candidates + pending council sessions

#### 2.4 `cyberos status --watch [--interval N]`

- **Function:** `_cmd_status_watch()`
- **Refresh:** every N seconds, default 30, minimum 5
- **Clear-screen via:** ANSI `\033[2J\033[H`
- **Exit:** Ctrl-C; loop catches `KeyboardInterrupt` cleanly

#### 2.5 Color-coded severity

- **Helpers:** `OK()`, `WARN()`, `CRIT()`, `DIM()`, `BOLD()` in cyberos
- **Color gate:** `_tty()` checks `sys.stdout.isatty()` + `$TERM` + `$NO_COLOR`

### Aspect 3 — Refinement loop (§0.4 propose-adopt-record)

#### 3.1 Stop-hook for refinement-candidate detection

- **Hook:** `runtime/hooks/refinement_candidates.py`
- **Scans:** audit ledger for patterns ≥ 3 occurrences in 30-day window
- **Patterns:** rejected, revert, drift_candidate, shallow_candidate,
  tag-duplication
- **Output:** writes `memories/drift/<date>-refinement-candidate-<slug>.md`
- **Install:** `cyberos hooks on --hook refinement_candidates`

#### 3.2 Eval harness for refinements

- **Layout:** `runtime/tests/refinements/REF-NNN/{capability.test.py,regression.test.py}`
- **Run via:** `cyberos eval REF-NNN`
- **Pass criteria:** both files must exit 0

#### 3.3 Council mode for ambiguous refinements

- **Tool:** `runtime/tools/cyberos_council.py` → `cyberos council REF-NNN`
- **4 voices:** Architect, Skeptic, Pragmatist, Critic
- **Output:** `.cyberos-memory/cache/council/REF-NNN-council.md` with prompt blocks + Synthesis template
- **Heuristic context:** GLOSSARY term overlap, LOCK conflicts, related
  REFs, recent rejected/ entries
- **Opt-in only:** the 4× API cost is worth it for ambiguous REFs; not
  every REF runs council

#### 3.4 Rejected refinement tracking

- **Path:** `.cyberos-memory/rejected/REJECTED-NNN-<slug>.md`
- **Template:** `runtime/starter/templates/REJECTED.md`
- **Surfaced by:** `cyberos refinements` (last 30d window)

#### 3.5 Postmortem template for missed refinements

- **Path:** `.cyberos-memory/memories/refinements/POSTMORTEM-NNN-<slug>.md`
  (or under `postmortems/`)
- **Template:** `runtime/starter/templates/POSTMORTEM.md`
- **Trigger:** a rejected refinement that later turned out to be real;
  document blamelessly

#### 3.6 Nygard ADR format for DECs

- **Template:** `runtime/starter/templates/DEC.md`
- **Sections:** Context / Decision / Status / Consequences / Alternatives
- **Used by:** every DEC the wizard scaffolds

#### 3.7 Bundle naming convention

- **Format:** `Bundle <letter> · <theme> · <SHA>`
- **Letters used so far:** A through P, plus Batches 4–10 of operator
  surface (not protocol amendments)

### Aspect 4 — Memory taxonomy & content discipline

#### 4.1 Memory templates per type

- **Path:** `runtime/starter/templates/{DEC,REF,FACT,PERSON,PROJECT,PREFERENCE,DRIFT,POSTMORTEM,REJECTED}.md`
- **Variables:** `${UUID7}`, `${TS_NOW}`, `${SUBJECT_ID}`, `${NEXT_NNN}`,
  `${SLUG_TITLE}`, `${CLASSIFICATION}`, `${AUTHORITY}`, `${TAGS}`,
  `${PROV_SOURCE}`, `${PROV_SOURCE_REF}`, `${SYNC_CLASS}`, `${FRESHNESS_TIER}`
- **Why outside `meta/templates/`:** validator scans `meta/` so
  templates with `${VAR}` placeholders would fail YAML validation.
  Living under `runtime/starter/templates/` keeps them out of validator scope.

#### 4.2 Memory-bucket usage dashboard

- **Implemented as:** `cyberos stats`
- **Lists:** memory by scope (meta, project, persona, company, …),
  memory by bucket (decisions, refinements, facts, …), total count

#### 4.3 First-class FACT discipline

- **Bucket:** `.cyberos-memory/memories/facts/FACT-NNN-<slug>.md`
- **Required fields:** `provenance.source_ref`, `provenance.confidence`,
  `source_freshness_tier` (lower = more authoritative)
- **Wizard support:** `cyberos add FACT`

#### 4.4 PEOPLE bucket

- **Bucket:** `memories/people/PERSON-NNN-<slug>.md`
- **Classification:** typically `personnel` (scope-rules plugin enforces)
- **Sync:** `local-only` or `shared` (scope-rules plugin denies `publishable`)

#### 4.5 PROJECT bucket

- **Bucket:** `memories/projects/PROJECT-NNN-<client>-<slug>.md`
- **Separate from:** `project/` (singular, per-project working memory)

#### 4.6 PREFERENCE bucket

- **Bucket:** `memories/preferences/PREF-NNN-<slug>.md`
- **Examples:** voice standard, compact §14 format, retention policy

#### 4.7 Memory relationships graph

- **Tool:** `runtime/tools/cyberos_graph.py` → `cyberos graph`
- **Edge kinds:** implements, supersedes, references, derives_from,
  contradicts, validates, satisfied_by
- **Formats:** text (default), dot (Graphviz), json
- **Ego graph:** `cyberos graph --memory <id> --hops N`
- **Dangling detection:** flags edges pointing at non-existent memory_ids

### Aspect 5 — Safety, enforcement, gates

#### 5.1 gateguard PreToolUse hook

- **Hook:** `runtime/hooks/gateguard.py`
- **Pattern:** 3-stage DENY → FORCE → ALLOW per gstack `gateguard`
- **State file:** `/tmp/cyberos-gateguard-state-${SESSION_ID}.json`
- **A/B-tested improvement:** +2.25 quality (per gstack benchmark)
- **Toggle via:** `cyberos hooks on --hook gateguard`

#### 5.2 Auto-tagging at write time

- **Function:** `load_glossary_terms()` + `suggest_tags()` in `cyberos_add.py`
- **Opt-in:** `cyberos add <TYPE> --auto-tags`
- **Source:** scans `FACT-014-glossary.md` for known terms appearing
  in slug + title + provenance ref
- **Caps:** `--auto-tags-max 5` by default
- **Confirmation:** interactive accept all / decline / edit list

#### 5.3 Per-write confirmation for sev-0 ops

- **Covered by:** `--reason` requirement on `cyberos doctor --repair`,
  `cyberos panic`, and gateguard PreToolUse on tool-use side
- **Pattern from:** `/careful` skill

#### 5.4 Encryption posture audit

- **Subcommand:** `cyberos status --security`
- **Surfaces:**
  - §5.6 encryption enabled/disabled + algorithm + KDF + Shamir threshold
  - §9.3 denylist test pass/fail (24/24 fixtures)
  - Filesystem perms on `manifest.json`, `audit/`, `.cyberos-memory/staging/`
  - §13.10 PANIC marker status (treats `(resolved)` titles as inactive)
  - §8.6 drift candidate count

#### 5.5 Denylist regression suite

- **Tests:** `runtime/tests/denylist/test_denylist.py`
- **Fixtures:** 24 patterns covering compensation, gov-IDs, bank, card
  numbers, secrets, health PII + evasion attempts
- **Run via:** `cyberos verify --denylist`

#### 5.6 §4.2 content-gate fuzz + body scan

- **Fuzz:** `runtime/tests/fuzz/test_content_gate_fuzz.py` (200 runs, 0 crashes)
- **Body scan:** `_check_content_gate_body()` in `cyberos_validate.py`
- **Markers:** `[INST]`, `<system>`, `<<SYS>>`, `<|im_start|>`,
  `<|system|>`, `<|assistant|>`, `###Instruction`, `###System:`,
  "ignore previous instructions", "ignore the above"
- **Whitelisted paths:** `tests/fuzz/`, `tests/mutation/`, REFs,
  `meta/validators/`, conflicts, postmortems (they document the markers
  legitimately)

#### 5.7 TOCTOU `.lock.shared` advisory locks

- **Module:** `runtime/tools/cyberos_lock.py`
- **Lock files:** `.cyberos-memory/.lock.shared`, `.cyberos-memory/.lock.exclusive`
- **Backend:** POSIX `fcntl.flock` with `LOCK_SH` for readers, `LOCK_EX` for writers
- **Context managers:** `shared_lock(brain_root)`, `exclusive_lock(brain_root)`
- **Degradation:** filesystems without `fcntl` (FUSE, some network FSes)
  fall back to no-op; never blocks

### Aspect 6 — Multi-machine personal sync

#### 6.1 `cyberos sync export`

- **Tool:** `runtime/tools/cyberos_sync.py`
- **Default classes:** `publishable` + `shared` (consent-gated)
- **Opt-in:** `--include client-visible` (consent required)
- **Determinism:** two consecutive exports produce identical SHA256

#### 6.2 Subject-identity stability

- **Mechanism:** `cyberos sync import --from subject:<id>` records the
  origin subject in `sync_manifest.json`
- **Resolution:** same subject id → merge as same identity; different →
  treat as different subject (writes to different `member/<id>/` scope)

#### 6.3 Per-export sanitization mode

- **Existing tool:** `runtime/tools/cyberos_export.py --sanitize-level <none|shareable|redacted>`

#### 6.4 Round-trip property verification

- **Manual:** `cyberos sync export → unzip → cyberos sync import --dry-run`
  on a fresh BRAIN must produce byte-identical manifest entries

#### 6.5 Conflict-resolution UX

- **Subcommand:** `cyberos sync conflicts --resolve`
- **Interactive picks:** `[l]ocal | [r]emote | [d]isputed | [o]pen | [s]kip | [q]uit`
- **Side effect:** appends a `## Resolution (<ts>)` block to the
  conflict marker so the §3 reconciliation row can reference it

### Aspect 7 — Documentation, voice, & protocol-doc discipline

#### 7.1 gstack /codex voice standard

- **Rules (verbatim):** no em dashes, no AI-vocab (leverage, robust,
  ensure, comprehensive, seamless, delve, navigate, tapestry), no
  marketing language, lead with the point
- **Memory:** `memories/preferences/PREF-001-voice-standard.md`

#### 7.2 `cyberos voice` linter

- **Tool:** `runtime/tools/voice_check.py`
- **Flags:** `--strict` (exit 1 on any finding), `--summary`
- **Usage:** `cyberos voice docs/`

#### 7.3 Cross-doc consistency

- **Subcommand:** `cyberos doc-consistency`
- **Checks:** every §X.Y reference in README exists as anchor in AGENTS.md,
  every DEC-NNN reference resolves to a real file

#### 7.4 Tour artifacts

- **Path:** `tours/*.tour` (10 tours)
- **Topics:** onboarding, refinement-loop, incident-response,
  protocol-upgrade, security-audit, repair-{audit-chain,tombstone-orphan,
  stuck-conflict,manual-rollback,fix-frontmatter}
- **Viewer:** VS Code CodeTour extension

#### 7.5 AGENTS.md compaction discipline

- **Soft cap:** 1500 lines, warnings at 1300/1400/1500
- **Mechanisms when approaching cap:** split a §-section into subdoc OR
  move to CORE.md OR `cyberos doctor --reorganise` on protocol doc

#### 7.6 Bundle thematic tagging

- **Convention:** CHANGELOG header lists `Bundle <letter> · <theme> · <SHA>`

#### 7.7 Glossary as first-class memory

- **Memory:** `memories/facts/FACT-014-glossary.md`
- **Format:** `## Term\ndefinition` per entry
- **Used by:** `cyberos_add --auto-tags`, `cyberos_council` heuristic context

### Aspect 8 — Onboarding & contributor experience

#### 8.1 `cyberos onboard`

- **Tool:** `runtime/tools/cyberos_onboard.py`
- **5-step wizard:** subject id → persona role → import shared zip (opt) →
  starter PREF → verify

#### 8.2 Starter-template repo

- **Path:** `runtime/starter/cyberos-starter/`
- **Contents:** README, pre-built `.cyberos-memory/manifest.json` with
  placeholders, `meta/retention-rules.md`, `meta/validators/README.md`,
  `docs/tours/onboarding.tour`
- **Use:** `cp -r runtime/starter/cyberos-starter ~/Projects/my-new-thing`

#### 8.3 Two-mode CLAUDE.md symlink

- **Recipe:**
  ```bash
  ln -s /path/to/cyberos/docs/CyberOS-AGENTS.md AGENTS.md
  ln -s /path/to/cyberos/docs/CyberOS-AGENTS.md CLAUDE.md
  ```

#### 8.4 Onboarding checklist memory

- **Generated by:** `cyberos onboard`
- **Path:** `memories/preferences/PREF-onboarding-checklist-<subject>.md`

#### 8.5 Persona-card auto-generation

- **Flag:** `cyberos onboard --persona <role>`
- **Output:** `.cyberos-memory/persona/<role>.md` with `persona_defaults`
  frontmatter block

### Aspect 9 — Performance & scaling

#### 9.1 Streaming session-start

- **Module:** `runtime/tools/cyberos_lazy.py`
- **Two phases:**
  - Phase A: manifest + checkpoint + legacy lists (~5 files, < 100 KB)
  - Phase B: lazy `stream_memories()` generator
- **Benchmark today:** 180.93 ms full eager vs 2.41 ms lazy first-5 → **74.9×**

#### 9.2 Incremental SQLite index updates

- **Hook:** `runtime/tools/cyberos_index_hook.py`
- **Modes:** `on-write` (called by brain_writer after each op), `stop-hook`
- **No-op when:** `index/cyberos.db` doesn't exist yet
- **Best-effort:** never blocks the underlying write

#### 9.3 `cyberos benchmark`

- **Tool:** `runtime/tools/benchmark.py`
- **Metrics:** p50/p95/p99 for `cyberos verify`, `cyberos export`,
  `cyberos search`

#### 9.4 Audit-ledger compaction tuning

- **Tool:** `runtime/tools/cyberos_compact_stats.py` → `cyberos compact-stats`
- **Thresholds:** `--row-cap 10000`, `--byte-cap 5000000`, `--age-days 90`
- **Recommends:** `cyberos doctor --compact-ledger MM` for each
  candidate ledger
- **Does NOT compact;** that's still doctor

#### 9.5 Cold-storage tier

- **Tool:** `runtime/tools/cyberos_cold_storage.py`
- **Subcommands:** `archive`, `list`, `verify`
- **Output:** deterministic `.cold.zip` per month with Merkle anchor
- **Operator uploads:** `aws s3 cp` / rclone / equivalent — tool never reaches a provider

#### 9.6 Memory deduplication

- **Tool:** `runtime/tools/cyberos_dedup.py` → `cyberos dedup`
- **Signals:**
  - Body 5-gram shingles, Jaccard ≥ 0.8
  - Slug-stem 3-gram similarity ≥ 0.85
- **Exclusions:** `meta/protocol-history/` (intentional snapshots),
  DEC↔REF implements-pair pattern (cross-bucket same-slug, low body sim)

#### 9.7 Memory-aging policy

- **Tool:** `runtime/tools/cyberos_prune.py` → `cyberos prune`
- **Three checks:**
  - Staleness via `last_updated_at` > `--staleness-days` (default 365)
    unless `retention.rule == "indefinite"`
  - Contradictions (`supersedes` edge to live memory; `contradicts` to live memory)
  - Unresolved drift > `--drift-days` (default 30)
- **NEVER deletes** — surface only; operator decides via `cyberos doctor`

### Aspect 10 — Testing, evals, fuzz, properties

#### 10.1 Test corpus growth

- **Target:** 16 → 200+
- **Current:** ~24 fixtures (8 mutation patterns × 3 fixtures + 24 denylist + 200 fuzz inputs)

#### 10.2 Property-based testing

- **File:** `runtime/tests/fuzz/test_content_gate_fuzz.py`
- **Generator:** biased random inputs toward injection markers, mixed-script
  confusables, BOM, surrogate pairs
- **Pass criteria:** 0 crashes

#### 10.3 Differential testing across implementations

- **Status:** blocked — only one impl exists (Python)
- **Unblocks when:** brain_writer is reimplemented in TypeScript / Go

#### 10.4 Mutation testing on validators

- **Tool:** `runtime/tests/mutation/run_mutations.py` → `cyberos mutation-test`
- **Mutations:** remove-memory-id, break-uuid-format, invalid-classification,
  inject-marker, invalid-authority, remove-provenance, negative-version,
  invalid-sync-class
- **Fixtures:** `fixture-valid-fact.md`, `fixture-valid-decision.md`,
  `fixture-valid-person.md`
- **Today:** 24 mutations × 0 SURVIVED

#### 10.5 Fuzz on content gate

- See 5.6 above (same file).

#### 10.6 Replay testing on audit ledger

- **File:** `runtime/tests/test_replay.py`
- **Asserts:** replaying every op in sequence yields same `audit_chain_head`

#### 10.7 Roundtrip property testing

- **File:** `runtime/tests/test_export_determinism.py`
- **Asserts:** export → unzip → re-import → re-export = byte-identical zip

#### 10.8 Eval-driven development

- See 3.2 above; every REF lands with capability + regression evals.

### Aspect 11 — Observability & telemetry

#### 11.1 Local-only analytics

- **File:** `~/.cyberos/analytics/skill-usage.jsonl`
- **Logged:** `ts`, `cmd`, `outcome`, `duration_ms`, `session`
- **Never sent anywhere** — local only by contract

#### 11.2 Usage report

- **Command:** `cyberos analytics report --period 7d --format table|json`

#### 11.3 Drift dashboard

- **Subcommand:** `cyberos drift`
- **Inline impl in:** `runtime/tools/cyberos` → `cmd_drift()`

#### 11.4 Refinement-candidate dashboard

- **Tool:** `runtime/tools/cyberos_refinements.py` → `cyberos refinements`
- **Buckets:**
  - drift candidates from §0.4 Stop-hook
  - council sessions awaiting synthesis (verdict-regex)
  - rejected entries in the last 30 days

#### 11.5 LLM cost analytics

- **File:** `~/.cyberos/analytics/llm-cost.jsonl`
- **Command:** `cyberos analytics cost-log` (record) + `cost-report` (totals)
- **Rates:** operator supplies `--input-per-mtok` + `--output-per-mtok`
  at call time; we never hardcode model pricing

### Aspect 12 — Architecture extension surfaces

#### 12.1 Pluggable validators

- **Loader:** `_run_pluggable_validators()` in `cyberos_validate.py`
- **Discovery:** `meta/validators/check-*.py` auto-imported
- **Signature:** `def check(memory: dict, manifest: dict) -> list[dict]`
- **Severity options:** `CRITICAL`, `WARN`, `INFO`
- **Exception isolation:** plugin errors surface as WARN, never crash validation
- **Shipped plugins:**
  - `check-tag-budget.py` — enforces ≤ 10 tags
  - `check-scope-rules.py` — reads `meta/scope-rules.md` (Aspect 12.2)
  - `check-source-tiers.py` — flags stale `source_tiers` patterns

#### 12.2 Custom scope rules

- **Config:** `meta/scope-rules.md`
- **Schema:** `## scope: <prefix>` + `- classification: allow/deny: [...]`,
  `- sync_class: allow/deny: [...]`, `- authority: minimum: <tier>`
- **Enforced by:** `meta/validators/check-scope-rules.py`

#### 12.3 Manifest-defined source tiers as data

- **Config:** `manifest.source_tiers[]` with `pattern` + `tier` + `rationale`
- **Validated by:** `meta/validators/check-source-tiers.py` (warns on stale patterns)

#### 12.4 Memory template variables

- See 4.1 above.

#### 12.5 Skill registry

- **Registry file:** `runtime/tools/skills/registry.json`
- **Loader:** `runtime/tools/cyberos_skill.py` → `cyberos skill {list|describe|chain}`
- **Each entry:** name, tool, umbrella_alias, verb, invocation_modes,
  depends_on, sections, mutates_brain, sev_0_ops (optional)
- **Chain safety:** `cyberos skill chain` warns when two mutating skills
  run without an intermediate `verify`

#### 12.6 Persona-defined defaults

- **Source:** `.cyberos-memory/persona/<role>.md` `persona_defaults` frontmatter
- **Consumed by:** `cyberos add --persona <role>`
- **Pre-fills:** `default_classification`, `default_authority`, `default_sync_class`

#### 12.7 Read-only MCP server

- **Server:** `runtime/mcp/cyberos_brain_server.py`
- **Wire:** `cyberos mcp info` prints the `.claude/mcp-config.json` snippet
- **Run:** `cyberos mcp serve` (line-delimited JSON-RPC 2.0 over stdio)
- **Tools:**
  - `brain_search` — keyword across slug/tags/scope/body
  - `brain_show` — list with optional filters
  - `brain_get` — fetch by memory_id or path
  - `brain_stats` — bucket + sync-class + tombstone counts
- **Default filters:** hide tombstoned + `sync_class=local-only` (both have opt-in flags)
- **No writes** — caller must use `brain_writer.py`

### Aspect 13 — Specific gaps observed

- **13.1** — covered in protocol-history scope coverage
- **13.2 — Empty `company/` directory** → fixed: `company/locked-decisions.md`
  now has 20 LOCK entries
- **13.3 — `cyberos doctor` repair op docs** → expanded via tour files
  (`tours/repair-*.tour`)
- **13.4 — Protocol-history INDEX** → `.cyberos-memory/meta/protocol-history/INDEX.md`
- **13.5 — `.cyberos-memory/cache/` transient artefacts** → `.gitignore` updated
- **13.6 — CONTRIBUTING.md** → at root level
- **13.7 — `.cyberos-memory/refinements/` workflow** → documented in this README
- **13.8 — Repo split** → architectural decision; not done yet
- **13.9 — `__pycache__/` exclusion** → in `.gitignore` + export skip list
- **13.10 — Emergency stop** → `cyberos panic` with `--reason` requirement

### Post-catalog Tier A — high-leverage Layer 1 amplifiers (Batch 11)

- **`.lock.shared` integration** in `cyberos_validate.py` — best-effort POSIX flock around the validate pass; `CYBEROS_NO_LOCK=1` to disable
- **Semantic search** — `runtime/tools/cyberos_semantic_search.py` + `cyberos semantic-search "<q>"`. TF-IDF default (zero-dep) + opt-in sbert
- **TUI dashboard** — `runtime/tools/cyberos_tui.py` + `cyberos tui --interval N`. Curses-based, single-screen live view
- **Diff + time-travel** — `runtime/tools/cyberos_history.py` + `cyberos history {diff|as-of}`. Reconstructs BRAIN state at any audit-chain point
- **Council `--run-now`** — `cyberos council REF-NNN --run-now` calls Claude per voice via anthropic SDK (degrades cleanly without it)

### Post-catalog Tier B — Batch 12

- **Branched BRAINs** — `runtime/tools/cyberos_branch.py` + `cyberos branch {list|create|switch|diff|merge|delete}`. Snapshots at `.cyberos-memory/.branches/<name>/`
- **LLM-assisted REF authoring** — `cyberos ref-from-drift <drift>.md [--with-llm]`. Reads a drift candidate, stages a structured REF draft
- **Auto-repair** — `cyberos autorepair [--apply] [--recipe X]`. 3 recipes (tag-budget, duplicate-tags, tombstone-missing-metadata); safety envelope prohibits touching authority/classification/memory_id
- **Web dashboard** — `cyberos serve --port N`. Stdlib `http.server`; routes: /, /memories, /memory/<id>, /audit, /stats.json
- **Auto-suggested supersedes** — `cyberos add` scans the same bucket for similar-stem files and hints

### Post-catalog Tier C — Batch 13

- **Replicated audit chain** — `cyberos replicate {status|push|verify}`. Best-effort sync to operator-supplied target dir
- **Multi-tenant scaffold** — `cyberos tenant {list|create|audit}` + `member/<slug>/` scope isolation
- **CRDT merge** — `cyberos crdt merge <conflict>`. Field-level merge: tags union, version max, sync_class tightens, classification refuses to auto-merge
- **Hypothesis property tests** — `runtime/tests/property/test_frontmatter_properties.py`. Round-trip yaml + uuid7 monotonicity

### Post-catalog Tier D — Batch 14

- **Ed25519 signed snapshots** — `cyberos sign {keygen|sign|verify|verify-all}`. Private key at `~/.cyberos/keys/`, public at `.cyberos-memory/meta/protocol-signing-pubkey.ed25519`
- **Parallel validator** — `cyberos parallel-validate --workers N`. Splits validator across N processes
- **Mobile static site** — `cyberos static --out <dir>`. Renders BRAIN as HTML with dark-mode CSS for phone reads

### Post-catalog Tier E — genuine Layer 1 wins (Batch 15)

- **Schema migration framework** — `cyberos migrate {list|plan|apply}`. Migrations under `runtime/migrations/<NNN>-<slug>.py`. State at `meta/migrations-applied.json`
- **Inline editor** — `cyberos edit <memory>`. $EDITOR + validate-on-save + brain_writer str-replace commit
- **Bulk edit** — `cyberos bulk-set <expr> --filter ...`. Refused-field guardrails for memory_id, authority, classification
- **Hybrid search (RRF)** — `cyberos hybrid-search "<q>"`. Reciprocal Rank Fusion over FTS + TF-IDF (+ optional sbert)
- **Audit streaming + alerts** — `cyberos audit-stream` + `cyberos alert {add|list|remove|run}`. Rules like `CRITICAL > 0` + slack-webhook actions
- **REPL history** — `~/.cyberos/repl-history` + readline integration + tab completion
- **Chaos tests** — `cyberos chaos-test`. 3 scenarios: tmp+rename atomicity, ENOSPC, concurrent writers — all 3 PASS
- **Per-memory ACLs** — `meta/validators/check-acl.py`. Personnel-class memories without acl block surface as WARN
- **Cleanup tool** — `cyberos cleanup --out-script <path>`. Emits host-side rm script for leftovers

---

## Part 27 — Per-tool CLI reference

### Status & diagnostics

```
cyberos status                          # 4-question dashboard
cyberos status --verbose                # show all findings
cyberos status --weekly                 # 7-day digest (Aspect 2.3)
cyberos status --watch [--interval N]   # continuous (Aspect 2.4)
cyberos status --security               # encryption posture (Aspect 5.4)
cyberos verify [--self-test] [--denylist]
cyberos drift                           # drift candidates only
cyberos refinements [--kind drift|council|rejected|all] [--json]
cyberos stats
cyberos compact-stats [--row-cap N] [--byte-cap N] [--age-days N] [--json]
cyberos mutation-test [--fixture name] [--json]
```

### Memory management

```
cyberos add <TYPE> [--slug ...] [--classification ...] [--authority ...]
                   [--tags ...] [--sync-class ...] [--prov-source ...]
                   [--prov-source-ref ...] [--freshness-tier N]
                   [--auto-tags] [--persona NAME]
                   [--non-interactive] [--dry-run]
cyberos show [--scope ...] [--tag ...] [--class ...] [--tombstoned]
             [--recent 7d]
cyberos search <query>
cyberos prune [--staleness-days N] [--drift-days N] [--interactive] [--json]
cyberos dedup [--scope ...] [--threshold 0.8] [--slug-threshold 0.85]
              [--since 30d] [--json]
cyberos graph [--format text|dot|json] [--scope ...] [--orphans]
              [--memory MEMORY_ID] [--hops N] [--verbose]
```

### Refinement loop

```
cyberos council REF-NNN [--voices ...] [--print]
cyberos eval REF-NNN
cyberos refinements [--kind ...] [--json]
```

### Sync + export

```
cyberos export [-o DIR] [--to PATH] [--sanitize-level ...]
               [--dry-run] [--daemon --interval H] [--verify FILE]
cyberos sync export --to PATH [--include client-visible]
cyberos sync import BUNDLE --from subject:NAME [--dry-run]
cyberos sync conflicts [--resolve]
cyberos cold-storage archive --age-months N --to PATH
cyberos cold-storage list DIR
cyberos cold-storage verify ARCHIVE
```

### Operational

```
cyberos doctor <op> [--repair --reason "..."] [--dry-run]
   ops: rebuild-chain, tombstone-orphan, resolve-conflict,
         manual-rollback, fix-frontmatter, compact-ledger, decompact-ledger
cyberos panic [--reason "..."] [--resolve "..."]
cyberos hooks {status|on|off} [--hook NAME]
cyberos lock {status|acquire-shared|acquire-exclusive} [--timeout N] [--hold N]
cyberos analytics {log|report|purge|cost-log|cost-report}
                  [--period 7d] [--format table|json]
```

### Onboarding + meta

```
cyberos onboard [--shared ZIP] [--persona ROLE] [--non-interactive]
cyberos voice [--strict] [--summary] [PATHS...]
cyberos doc-consistency [--strict]
cyberos explain [SUBCOMMAND]
cyberos skill {list|describe NAME|chain NAME1 NAME2 ...}
cyberos repl
cyberos mcp {serve|info}
cyberos help [SUBCOMMAND]
cyberos --version
```

### Post-catalog operator commands (Batches 11-15)

```
# Search + retrieval
cyberos semantic-search "<query>" [--scope ...] [--limit N] [--backend tfidf|sbert]
cyberos hybrid-search "<query>" [--scope ...] [--limit N]
cyberos history diff <memory-id-or-path> [--against HEAD~N]
cyberos history as-of <ISO-ts|HEAD~N>

# Operator UX
cyberos tui [--interval N]
cyberos serve [--port N] [--host H]
cyberos static [--out DIR]
cyberos repl                          # readline + tab completion

# State + branches
cyberos branch {list|create|switch|diff|merge|delete} NAME
cyberos lock {status|acquire-shared|acquire-exclusive}

# Authoring
cyberos add <type> [--auto-tags] [--persona NAME]
cyberos edit <memory>
cyberos bulk-set <expr> [--filter ...] [--apply] [--allow-protected]
cyberos autorepair [--apply] [--recipe X]
cyberos ref-from-drift <drift-path> [--tier N] [--with-llm]
cyberos migrate {list|plan|apply} NAME

# Sync + ops
cyberos sync conflicts [--resolve]
cyberos cold-storage {archive|list|verify} ...
cyberos replicate {status|push|verify}
cyberos tenant {list|create|audit}
cyberos crdt merge <conflict-marker>
cyberos sign {keygen|sign|verify|verify-all}

# Council + advanced
cyberos council REF-NNN [--run-now] [--voices ...]
cyberos advanced {fr-council|auto-decompose|client-chain|replan|marketplace}

# Audit ops + streaming
cyberos audit-stream [--from-start]
cyberos alert {add|list|remove|run}
cyberos analytics {log|report|purge|cost-log|cost-report}

# Quality + testing
cyberos chaos-test
cyberos parallel-validate [--workers N] [--format json]
cyberos mutation-test [--fixture name]
cyberos compact-stats [--row-cap N] [--byte-cap N] [--age-days N]
cyberos skill-quality run|calibration <skill-id>
cyberos cleanup [--apply] [--out-script <path>]
```

### Skills-layer operator commands (Batches 16-23)

```
# Chain orchestration
cyberos chain estimate --pitch "<text>" --profile solo|lean|standard|full
cyberos chain run --pitch "<text>" [--profile P] [--with-llm] [--model M]
                  [--max-iterations N] [--no-cache] [--max-tokens N] [--max-cost USD]
cyberos chain status [<output-dir>]
cyberos chain resume <output-dir> [--with-llm]
cyberos chain graph [<output-dir>]

# Feature-request browser
cyberos fr {list|show|graph|task-graph} <FR-id>

# Authoring helpers (shared library used by skill runners)
cyberos authoring {llm|voice|attribute|diff|interview} <args>

# Project-tracker sync
cyberos proj {backends|sync|pull} [FR-id] [--backend linear|jira|github]

# Skill testing + benchmarking
cyberos skill-test <skill-id> [--no-llm] [--max-iterations N]
cyberos skill-bench <skill-id> [--runs N] [--record] [--model M]
cyberos cross-skill <chain-output-dir>
```

---

## Part 28 — Common workflows

### Daily verify

```bash
cyberos status              # 5-second sanity check
cyberos verify              # full 11-category validator
```

If WARN > 5 or any CRITICAL, run `cyberos status --verbose` to see findings.

### Adding a new memory

```bash
cyberos add FACT --slug pricing-tier-3 --auto-tags --persona founder
# Wizard prompts for missing fields. Outputs a preview, then asks for
# confirmation before `brain_writer write` commits.
```

### Refinement cycle (§0.4)

```bash
# 1. Detect candidates auto-populated by the Stop-hook
cyberos refinements --kind drift

# 2. If ambiguous, run council
cyberos council REF-042
# (paste each voice prompt into a fresh Claude conversation; collect
#  the 4 findings; write Synthesis)

# 3. Run capability + regression evals
cyberos eval REF-042

# 4. If green, ship — and run the protocol-upgrade phrase if §0.4 needs it
```

### Sync to another machine

```bash
# Machine A
cyberos sync export --to ~/cyberos-bundle.zip
# scp / rsync / drop in shared drive

# Machine B
cyberos sync import ~/cyberos-bundle.zip --from subject:other-machine --dry-run
# Review the report at .cyberos-memory/cache/test-fixtures/sync/<run-id>.md
cyberos sync import ~/cyberos-bundle.zip --from subject:other-machine
# Resolve any conflicts:
cyberos sync conflicts --resolve
```

### Audit-ledger maintenance (~monthly)

```bash
cyberos compact-stats                   # see if any month should be compacted
cyberos doctor --compact-ledger 2026-04 # actually compact (sev-0)
cyberos cold-storage archive --age-months 12 --to ~/cold/  # bundle old ledgers
# upload to S3 / rclone / etc.
# After confirmed upload:
rm .cyberos-memory/audit/<archived>.jsonl
```

### Adding a project-specific validator

```bash
# 1. Drop the plugin
cat > .cyberos-memory/meta/validators/check-my-rule.py <<'PY'
def check(memory, manifest):
    if memory.get("classification") == "client" and not memory.get("tags"):
        return [{"severity": "WARN", "code": "client-needs-tags",
                 "message": "client memories must carry at least one tag"}]
    return []
PY

# 2. Verify auto-discovery
cyberos verify
# Findings now include [§12.1] client-needs-tags entries
```

### Read BRAIN from a different agent (Claude Code, Cursor)

```bash
cyberos mcp info
# Paste the printed snippet into .claude/mcp-config.json
# Restart the client; it now sees brain_search / brain_show / brain_get / brain_stats
```

---

## Part 29 — Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `no .cyberos-memory/ found` | CWD not inside a CyberOS project | `cd` into a project root or `export CYBEROS_HOST_MOUNT_PREFIX=<mount>` |
| `cyberos verify` shows `chain-link-mismatch` | audit chain corrupted | `cyberos doctor rebuild-chain --repair --reason "<why>"` (sev-0) |
| `provenance-missing` WARN | memory missing required block | Add `provenance: {source: ..., source_ref: ..., confidence: ...}` to frontmatter |
| `invalid-sync-class` WARN | typo in sync_class field | Pick one of `{local-only, publishable, shared, client-visible}` |
| `scope-sync-class-not-allowed` WARN | scope-rules.md violated | Check `meta/scope-rules.md` for the scope; pick an allowed sync_class |
| `tag-budget-exceeded` WARN | > 10 tags on a memory | Trim tag list to 10 or fewer (CyberSkill convention) |
| `cyberos panic` shows ACTIVE on a fresh session | stale marker from earlier self-test | Edit `meta/PANIC.md` title to include `(resolved)` |
| `cyberos sync conflicts` shows entries | another subject's bundle conflicted | `cyberos sync conflicts --resolve` and pick local / remote / disputed |
| `brain_writer` refuses to write in a sandbox | §0.1 real-filesystem enforcement | `export CYBEROS_HOST_MOUNT_PREFIX="/host-mount-path"` |
| Mutation test SURVIVED something | Validator gap | Either tighten the validator or remove the mutation if mutated input is genuinely valid |

---

## Part 30 — Deferred and blocked items

| Aspect | Status | Why |
| --- | --- | --- |
| 10.3 — Differential testing across implementations | **Blocked** | Only one impl (Python) exists. Unblocks when brain_writer is reimplemented in TS / Go. |
| 13.8 — Repo split (`cyberos-cli` separate from protocol) | **Deferred** | Architectural choice; ship when CLI iteration outpaces protocol. |

All other named aspects in `workbench/cyberos-layer1-deep-improvements.md`
are shipped. See `CyberOS-AGENTS.CHANGELOG.md` Batches 4–10 for landing
notes.

---

## Part 31 — File map

```
cyberos/
├── docs/
│   ├── CyberOS-AGENTS.md              ← protocol (authoritative)
│   ├── CyberOS-AGENTS.md         ← per-session load (regenerable)
│   ├── CyberOS-AGENTS.README.md       ← this file (on-ramp + operator manual)
│   └── CyberOS-AGENTS.CHANGELOG.md    ← daily log
├── runtime/
│   ├── tools/                         ← 19 Python scripts + 1 umbrella
│   │   ├── cyberos                    (umbrella; 33 subcommands)
│   │   ├── cyberos_validate.py
│   │   ├── cyberos_doctor.py
│   │   ├── cyberos_export.py
│   │   ├── cyberos_index.py
│   │   ├── cyberos_encrypt.py
│   │   ├── cyberos_add.py
│   │   ├── cyberos_show.py
│   │   ├── cyberos_search.py          (via cyberos_index.py)
│   │   ├── cyberos_council.py         ← Batch 4
│   │   ├── cyberos_sync.py            ← Batch 4 + 5
│   │   ├── cyberos_repl.py            ← Batch 5
│   │   ├── cyberos_dedup.py           ← Batch 5
│   │   ├── cyberos_graph.py           ← Batch 6
│   │   ├── cyberos_prune.py           ← Batch 7
│   │   ├── cyberos_hooks.py           ← Batch 7
│   │   ├── cyberos_refinements.py     ← Batch 8
│   │   ├── cyberos_compact_stats.py   ← Batch 8
│   │   ├── cyberos_lock.py            ← Batch 10
│   │   ├── cyberos_lazy.py            ← Batch 10
│   │   ├── cyberos_index_hook.py      ← Batch 10
│   │   ├── cyberos_cold_storage.py    ← Batch 10
│   │   ├── cyberos_skill.py           ← Batch 10
│   │   ├── voice_check.py
│   │   ├── canonical_sha.py
│   │   ├── extract_agents_core.py
│   │   ├── benchmark.py
│   │   ├── cyberos_onboard.py
│   │   ├── cyberos_analytics.py
│   │   ├── completions/cyberos.{bash,zsh,fish}
│   │   └── skills/registry.json       ← Batch 10
│   ├── hooks/
│   │   ├── gateguard.py
│   │   └── refinement_candidates.py
│   ├── mcp/
│   │   └── cyberos_brain_server.py    ← Batch 4
│   └── tests/
│       ├── denylist/test_denylist.py
│       ├── fuzz/test_content_gate_fuzz.py
│       ├── test_replay.py
│       ├── test_export_determinism.py
│       ├── refinements/<REF>/{capability,regression}.test.py
│       └── mutation/                  ← Batch 8
│           ├── run_mutations.py
│           └── fixtures/
│               ├── fixture-valid-fact.md
│               ├── fixture-valid-decision.md
│               └── fixture-valid-person.md
├── .cyberos-memory/cache/
│   ├── brain_writer.py                ← reference writer (mutates BRAIN)
│   ├── templates/                     ← memory templates (Aspect 4.1)
│   ├── staged-memories/               ← pre-commit staging
│   ├── council/                       ← council session artefacts
│   ├── sync/                          ← sync import reports
│   ├── sync-staging/                  ← imported memories awaiting review
│   ├── cold-test/                     ← test cold-storage archives
│   └── cyberos-starter/               ← Aspect 8.2 skeleton
└── tours/                             ← Aspect 7.4 CodeTour files
```

---

## Part 32 — Skills layer (CPO/CTO chain)

> The **skills layer** sits on top of Layer 1. Where Layer 1 is the BRAIN
> filesystem + operator tools, the skills layer is the **product-planning
> pipeline** — natural language → PRD → FRs → tasks → tickets — modelled
> as a chain of cyberos skills under `docs/skills/cuo/`.

### The 11 chain skills

```
docs/skills/cuo/
├── cpo/                                       # Chief Product Officer skills
│   ├── requirements-discovery/                # NL pitch → project_brief@1
│   ├── chain-selector/                        # Picks chain_profile (solo/lean/standard/full)
│   ├── prd-author/                            # project_brief → prd@1
│   ├── prd-audit/                             # Audits the PRD
│   ├── fr-author/                             # PRD → feature_request@1 list
│   ├── fr-with-tasks/                         # NEW: collapsed FR+spec for solo profile
│   └── fr-audit/                              # Audits FR list
└── cto/                                       # Chief Technical Officer skills
    ├── srs-author/                            # PRD → srs@1
    ├── srs-audit/                             # Audits the SRS
    ├── fr-to-tech-spec/                       # FR → tech_spec@1
    └── spec-to-impl-plan/                     # tech_spec → impl_plan@1 (tickets)
```

### Chain profiles

| Profile | Chain | When to use |
| --- | --- | --- |
| **`solo`** (default for internal work) | `[prd-author?]` → `fr-with-tasks` → `fr-audit` | 1-10 person team, internal product, client_visible: false |
| `lean` | `prd-author` → `fr-author` → `fr-audit` → `spec-to-impl-plan` | Small client work, eu_ai_act ≤ limited |
| `standard` | `prd-author` → `prd-audit` → `fr-author` → `fr-audit` → `fr-to-tech-spec` → `spec-to-impl-plan` | Standard client engagement, persona separation needed |
| `full` | Adds `srs-author` + `srs-audit` to standard | Regulated client (bank, government), eu_ai_act: high |

### task@1 — the assignable unit

Every FR carries an embedded `tasks:` list per the `task@1` contract at
`docs/contracts/task/CONTRACT.md`. Each task has:

- **`id`** matching `^FR-NNN-T-MM$` — addressable, greppable, PR-referenceable
- **`description`** — ≥200 chars, no upper cap; an engineer or AI agent can pick it up without re-reading the parent FR
- **`acceptance_test`** — concrete; exactly one of `shell` (command returning 0 on success) or `assertion` (structured assertion)
- **`assignable_to`** — `[human]`, `[ai-agent]`, or both; with `agent_profile` + `estimated_tokens` for AI tasks, `estimated_hours` for human tasks
- **`dependencies`** + **`parallelisable`** — explicit DAG; the chain enforces acyclicity

### Deterministic skill runners

Each skill has (or can have) a runner at `runtime/skill_runners/<name>.py`
subclassing `BaseSkillRunner`. The runner:

1. Conducts the interview (subclass overrides `interview(inputs)`)
2. Composes the LLM prompt (`build_prompt(inputs, prior_artefacts)`)
3. Calls Claude (the only non-deterministic step)
4. Runs INVARIANT checks (`validate_emit(body, inputs)`)
5. Iterates: if WARN, re-prompt with fix hints; if CRITICAL, HITL-pause; up to `--max-iterations`

This flips the ratio from "trust Claude to follow SKILL.md" (Batch 16) to
"runner enforces, Claude judges" (Batch 21). `runtime/skill_runners/fr_with_tasks.py`
is the reference implementation with 14 INVARIANT checks.

### Common skill workflows

**Drive a new project end-to-end:**

```bash
cyberos chain estimate --pitch "<paragraph>" --profile solo
# → tokens + cost estimate

cyberos chain run --pitch "<paragraph>" --profile solo --with-llm
# → writes feature_request@1 + tasks to planning/<date>-<slug>/

cyberos fr list                              # surveys all FRs
cyberos fr task-graph FR-001                 # mermaid DAG
cyberos skill-test fr-with-tasks --no-llm    # regression against test corpus
```

**Sync tasks to Linear / Jira / GitHub:**

```bash
cyberos proj sync FR-001 --backend github --dry-run
# Generates 6 envelopes; pipe to `gh issue create`
```

**Resume a paused chain:**

```bash
cyberos chain status                          # what's pending?
cyberos chain resume <output-dir> --with-llm  # picks up where it stopped
```

**Skill quality check:**

```bash
cyberos skill-quality run fr-with-tasks       # 5 checks per skill
cyberos skill-bench fr-with-tasks --runs 3    # token + cost regression vs baseline
cyberos cross-skill <output-dir>              # consistency across artefacts
```

### Contracts the skills depend on

```
docs/contracts/
├── feature-request/    # feature_request@1 — every FR conforms
├── task/               # NEW: task@1 — embedded in feature_request@1.tasks[]
├── chain-manifest/     # NEW: chain_manifest@1 — persistent state for cyberos chain run
├── prd/                # prd@1
├── srs/                # srs@1
├── impl-plan/          # impl_plan@1
├── project-brief/      # project_brief@1
└── nats-subjects/      # wire protocol for chain events
```

### Anti-fabrication discipline

All 11 chain skills declare `untrusted_content_wrapping: required` and ship
`references/ANTI_FABRICATION.md` with 7 rules covering source-grounded
claims, authority markers, HITL on ambiguity, untrusted_content wrapping,
no fabricated cross-references, no fabricated metrics, calibration tracking.

Each skill passes `cyberos skill-quality run <skill>` at 5/5 — antifab,
untrusted, grounding, calibration, deprecation checks.

### When NOT to use the skills layer

The skills layer is designed for **product-planning** workflows. It's
overkill for:

- Single-file edits (use `cyberos edit` directly)
- Drift-candidate triage (use `cyberos prune` + `cyberos refinements`)
- One-off facts (use `cyberos add FACT`)
- Memory queries (use `cyberos hybrid-search` or `cyberos fr show`)

Reach for `cyberos chain run` when you have a new project / feature
that needs to go from a sentence to assignable tasks.

---

## Cross-references

- **AGENTS.md sections cited above:** §0.1, §0.4, §0.5, §3, §4.2, §4.4, §4.6, §4.7, §5.1, §5.2, §5.6, §7, §7.6, §7.7, §8.6, §8.7, §9, §9.3, §9.6, §11.2, §11.3, §13, §13.10, §17.
- **Improvement catalog:** `workbench/cyberos-layer1-deep-improvements.md` — the source of every Aspect number for Batches 1–10.
- **Daily landing log:** `docs/CyberOS-AGENTS.CHANGELOG.md` — Batches 4–23 entries map landed code to Aspect numbers + post-catalog tiers + skills layer batches.
- **Contracts:** `docs/contracts/{feature-request,task,chain-manifest,prd,srs,impl-plan,project-brief,nats-subjects}/CONTRACT.md` — schemas every skill artefact conforms to.
- **Skills:** `docs/skills/cuo/{cpo,cto}/<skill>/SKILL.md` — 11 chain skills, all at 5/5 quality.
- **Skill runners:** `runtime/skill_runners/{base.py, fr_with_tasks.py, ...}` — deterministic runtime pattern.
- **Test corpus:** `runtime/tests/skills/<skill>/fixtures/*.yaml` — regression test fixtures per skill.

Parts 25–32 update in lockstep with each CHANGELOG batch entry. When new
aspects ship, add a subsection to Part 26 (Layer 1) or Part 32 (skills);
update the Part 27 CLI reference; bump the relevant CHANGELOG.
