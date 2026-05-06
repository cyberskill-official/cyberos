# CyberOS BRAIN — Reader's Guide & Evolution Manual

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

If your agent says it wrote a memory but you can't find the file: check the path it claims to have written to. If it starts with `/sessions/`, `/var/folders/`, `outputs/`, or anything in §0.1's forbidden list, the write went to the void.

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

**Skipping the §14 end-of-response block.** "I didn't write anything this turn" is still a valid §14 block — every line says `no change` plus a one-sentence justification. Skipping the block entirely makes it impossible to audit what the agent thinks it did vs what it actually did.

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

*This README is a living document. When AGENTS.md changes meaningfully (a new section, a new sync class, a new audit op), update the relevant Part above and append a one-line "README updated" entry to `CyberOS-AGENTS.CHANGELOG.md`. The README itself is informational and does not require §0.5 protocol approval; only AGENTS.md edits do.*
