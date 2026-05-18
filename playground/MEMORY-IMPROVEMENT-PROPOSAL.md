# MEMORY Module — Improvement Proposal

> **Inputs analysed:**
> 1. `playground/Memory and dreaming for self-learning agents.mp4` (24:28, transcript at `extracts/memory-and-dreaming.transcript.txt`) — Maheesh / Anthropic platform team — launching the **Memory + Dreaming** primitive in the Cloud Managed Agents API
> 2. `playground/Agentic Memory - A Detailed Breakdown.mhtml` (Ramakrushna @techwith_ram, X article, transcript at `extracts/agentic-memory.article.txt`) — taxonomy of 4 memory types + reference implementation in ChromaDB
> 3. Academic grounding: MARS framework with Ebbinghaus decay [1]; Evo-Memory benchmark for self-evolving memory [2]; Zhang et al. survey of LLM-agent memory mechanisms [3]
> 4. Current code: `modules/memory/` — 255 tests green; protocol at AGENTS.md v2.0.0
>
> **Audience:** Stephen / CyberOS maintainers, treated as FR-style proposal candidates for `modules/memory/PROPOSAL.md` / `BACKLOG.md`.
> **Date:** 2026-05-19

---

## 1. Executive summary

CyberOS MEMORY is **best-in-class at the infrastructure layer** (audit chain, content-addressed files, deterministic export, encryption envelope, privacy classes, cross-BRAIN merge, semantic search, daily digest). It is built for a **single human + their agent on one project**.

Both source materials point to a different problem the field is now solving: **how a swarm of long-running agents builds, curates, and amortises a shared memory across hundreds of sessions and many days, with measurable behavioural learning (not just durable storage).**

The two sources converge on five capabilities CyberOS does not yet implement:

| Capability | Article (Ramakrushna) | Anthropic talk (Maheesh) | CyberOS today |
|---|---|---|---|
| **Episodic memory** (task / approach / outcome / quality_score, recall-similar-tasks) | §3 "Episodic memory" — explicit Episode dataclass + reflection loop | Implicit: agents catch their own mistakes from past transcripts | ❌ no `kind: episode`, no quality_score / outcome field |
| **Out-of-band reflection ("dreaming")** — async batch process that mines transcripts → produces memory diffs | §"Periodic consolidation" — nightly merge of near-duplicates | **Headline product**: scans sessions, dedupes, removes stale entries, adds verification notes, produces a reviewable diff | ❌ `cyberos consolidate` only compacts the ledger; never re-reads memory content for behavioural patterns |
| **Importance / decay scoring at write time** | §"Memory management" — Generative-Agents Park 2023 score = α·relevance + β·importance + γ·recency | Implicit in dedup + stale-removal | ⚠ partial — `meta.classification`/`acl` exist but no float importance, no recency decay in ranking |
| **Permission-scoped memory stores per agent** | Not mentioned | **Core property** — same agent has read-only access to org-wide knowledge, read-write access to working memory | ❌ one BRAIN per project, no per-store ACL within a BRAIN |
| **Multi-agent optimistic concurrency** | Not mentioned | **Core property** — content-hash preconditions; many agents on the same memory state without clobbering | ⚠ partial — `.lock` serialises one writer; no precondition-hash op for many-agent contention |

The rest of this document proposes **eight Feature Requests (FRs)** that close the gap, grouped into three waves so the highest-leverage items ship first.

---

## 2. Deep dive — what each source actually contributes

### 2.1 The X article (Ramakrushna, "Agentic Memory: A Detailed Breakdown")

A clean taxonomy and a reference implementation. Most useful contributions:

- **4-type model**: *in-context*, *external* (structured + vector), *episodic*, *parametric*. CyberOS today is strong on `external/structured` (markdown files + SQLite FTS) and `external/vector` (optional `--semantic`). **Episodic is entirely missing**.

- **The Episode dataclass** — task / approach / outcome (success|partial|failure) / duration_ms / token_cost / quality_score / notes / error. Cheap to add, very high signal. This is how the agent answers "have I tried this kind of task before, and did my approach work?".

- **The reflection loop** — store an Episode whenever a task completes, then on next similar task `recall_similar(task, k=3)` and inject the top-K past episodes into the system prompt.

- **Time-decay scoring** — `score = 0.4·relevance + 0.3·importance + 0.3·(decay^hours_old)`. Park et al. (Generative Agents, 2023). Stops old memories from drowning new ones at recall time.

- **Importance scoring at write time** — let a small/cheap LLM rate the candidate memory 0.0-1.0 before storing. Filters noise at the source rather than retrieval.

- **Periodic consolidation by semantic dedup** — nightly: group memories with cosine-sim ≥ 0.92, summarise each group into a single canonical record. Crucial to keep the recall surface clean over months/years.

### 2.2 The Anthropic talk (Maheesh, "Memory and dreaming for self-learning agents")

The frontier vendor's bet on what a production multi-agent memory system needs. Concrete properties they ship:

- **File-system-as-memory model** (Claude Opus 4.7 onward) — instead of a tool with rigid schema, Claude treats memory as a hierarchy of files it can `bash`/`grep`/`cat`/`sed`. **This is exactly what CyberOS's BRAIN already is** (markdown under `<memory-root>/`). ✅ Strong alignment.

- **Permission-scoped memory stores** — the live SRE demo had three stores attached to a single agent:
  - `org-wide-knowledge` (read-only, runbooks, SLO guidelines — slow to update, broadcast to all agents)
  - `sre` (read-write, working memory the agent updates as it learns)
  - `code-base` (read-write per-area)

  Agents pick which stores to attach. **CyberOS today: one store per project.**

- **Optimistic concurrency with content-hash preconditions** — `put(path, body, precondition_hash)`. If two agents both think they're updating the same file, the second one's hash mismatches → it re-reads + retries. This is exactly the seqlock pattern but **made explicit at the file level so 1000 agents can hammer one store**.

- **Version history + attribution metadata** — every memory update records *which agent*, *what session*, *what time*, with the ability for the agent itself to walk the audit log. CyberOS has this in the audit chain but does not surface it ergonomically: there is no `cyberos history <memory-path>` that shows per-file version diffs to the agent at recall time.

- **Standalone portable API** — customers run their own pipelines (PII scanning, cloning to external systems, manual review). CyberOS's `cyberos serve` (REST) already covers this. ✅ Strong alignment.

- **Dreaming — the headline new thing** — async batch process, runs out-of-band from sessions, scans recent transcripts across many agents, finds:
  - Common mistakes across multiple agents (e.g. "5 different sessions all hit the same 60-second retry pattern")
  - Stale entries that no longer match observed reality
  - Duplicate facts in the store
  - Opportunities to **verify** existing memories (annotating "as of session X this fact still held")
  Produces a **diff** the operator can review, then apply.

  Reported impact: Harvey → 6× legal-benchmark completion rate; Rakerton → 90% drop in first-pass agent mistakes; consistent token-efficiency and latency wins.

  Design framing the talk emphasises: *"separate the memory-quality objective from the task-completion objective."* Today CyberOS implicitly tries to do both during a single session. Dreaming separates them.

- **The three-layer mental model** — *storage* / *structure & content* / *process*. CyberOS has shipped storage and structure; **the process layer is mostly empty** (only `cyberos consolidate` exists, and that's ledger maintenance, not memory curation).

### 2.3 Academic grounding

- **MARS** [1] adds Ebbinghaus-curve forgetting to a 3-agent (User / Assistant / Checker) reflective loop. Confirms the article's α/β/γ scoring is the right direction; Ebbinghaus is a specific decay function worth borrowing.
- **Evo-Memory** [2] benchmarks 10 memory modules on streaming task sequences and shows **continual experience reuse via action-think-memory updates** is what separates good agents from forgetful ones. Their `ReMem` pipeline is essentially the article's reflection loop, evaluated.
- **Zhang et al. survey** [3] — the foundational mapping of "what kinds of memory matter for agents". Identifies three roles (continuity / context / learning) that the article echoes verbatim — likely because the article is summarising this survey.

---

## 3. Gap analysis

### 3.1 What CyberOS already gets right

| Concern | Current state | Verdict |
|---|---|---|
| Durability + chain integrity | Binary framed binlog, crc32c, SHA-256 chain, MMR + STH opt-in | ✅ Stronger than article or talk |
| Determinism / portability | `cyberos export` byte-identical zip | ✅ Stronger than vendor offerings |
| Privacy + encryption | `meta.classification`, `meta.cipher`, encryption envelope | ✅ Stronger than article |
| File-system-as-memory model | Markdown under `<memory-root>/`, Claude uses Read/Edit | ✅ Exactly matches Anthropic's bet |
| Cross-BRAIN merge | `cyberos import` with provenance tracking | ✅ Beyond what either source covers |
| Semantic search | Optional `--semantic` with int8 MiniLM | ✅ Solid baseline |
| Daily digest | `cyberos digest --since 24h` | ⚠ Exists, but doesn't *act* on findings |
| HTTP API | `cyberos serve` | ✅ Matches the standalone-API requirement |

### 3.2 What's missing

| Gap | Severity | Source |
|---|---|---|
| **No episodic memory** — no `kind: episode`, no outcome/quality_score | High | Article §3, Evo-Memory [2] |
| **No reflection loop** — agent can't ask "have I tried this task before?" | High | Article §"Reflection loop", Anthropic talk implicit |
| **No write-time importance scoring** — every memory has equal weight at recall | Medium | Article §"Memory management" |
| **No recency decay in recall ranking** — old facts crowd out new at top-K | Medium | Article §"Time-based decay", MARS [1] |
| **No semantic-dedup consolidation** — `cyberos consolidate` only touches the ledger | High | Article §"Periodic consolidation" |
| **No dreaming / out-of-band batch reflection** | **Critical** | **Anthropic talk** is built around this |
| **No per-memory-store permission scopes within a BRAIN** | High | Anthropic talk live demo |
| **No optimistic-concurrency `put` with precondition-hash** | Medium | Anthropic talk concurrency property |
| **No `cyberos history <path>` that surfaces per-file version + attribution at recall** | Medium | Anthropic talk version-history demo |
| **No memory-quality objective separated from task objective** | Medium (design) | Anthropic talk design rationale |
| **No multi-agent shared memory at scale** (lock is single-writer; no contention model) | Medium | Anthropic talk: 100s-1000s of agents share state |

---

## 4. Proposed Feature Requests

Numbered FR-MEM-12x to fit the existing `modules/memory/` numbering (current proposals run to P18; switching to FR style for the BACKLOG.md convention).

### Wave 1 — High-leverage, low-protocol-impact (ship first)

#### FR-MEM-120 — Episodic memory kind + recall-similar-tasks API

**Why:** the single highest-leverage gap. Once agents log episodes, every other improvement in this proposal compounds.

**Spec sketch:**
- Add `episode` to the `kind` enum in `memory.schema.json` (sibling of `decisions | facts | people | ...`).
- Frontmatter additions for `kind: episode`:
  ```yaml
  task: "Audit MEMORY module for self-learning gaps"
  approach: "MHTML + ffmpeg/whisper extraction; cross-ref vs AGENTS.md"
  outcome: "success"          # enum: success | partial | failure
  duration_ms: 1847000
  token_cost: 145000
  quality_score: 0.92         # 0.0–1.0, optional
  error: null                 # optional, free-text on failure
  ```
- New CLI: `cyberos recall-similar <task-string> [--k 3] [--min-relevance 0.65]` — semantic-search filtered to `kind: episode`, ranks by `relevance · 0.4 + quality_score · 0.3 + recency_factor · 0.3`.
- New writer convenience: `cyberos episode log --task ... --approach ... --outcome ... [--quality-score ...]`.

**Effort:** ~2 days. Schema change is additive (non-breaking). No protocol amendment needed (§5.2 only requires `kind` be in the schema's closed enum — we extend the enum).

**Acceptance:** new test `test_episode_recall_similar.py` shows that after 5 dummy episodes are logged, `recall_similar("foo")` returns them ranked by combined score.

---

#### FR-MEM-121 — Recency-decay + importance in recall ranking

**Why:** prevents old memories from dominating top-K at retrieval time.

**Spec sketch:**
- Extend `cyberos.core.semantic.recall()` with new parameters: `recency_weight: float = 0.3`, `decay_factor: float = 0.995` (per-hour half-life ≈ 4 days; configurable via `manifest.json`).
- Combined score: `score = relevance · w_r + importance · w_i + recency · w_t`. Default weights from Park et al. (Generative Agents): `w_r=0.4, w_i=0.3, w_t=0.3`.
- Add optional `meta.importance: float (0.0–1.0)` to frontmatter. When absent, default to 0.5.

**Effort:** ~1 day. Ranking change only; no protocol amendment.

**Acceptance:** existing `test_semantic.py` cases pass; new case asserts that two memories with identical relevance but different `importance` rank in the expected order.

---

#### FR-MEM-122 — Write-time importance scoring via tiny model

**Why:** filters noise at write time so the store stays clean.

**Spec sketch:**
- New module: `cyberos.core.importance.score(content: str) -> float`. Uses `claude-haiku-4-5` (or any configurable cheap LLM). Prompt: "Rate the importance of saving this for future interactions. 0.0 = trivial. 1.0 = critical. Reply with only the number."
- Triggered opt-in: `cyberos put --score-importance ...`. Drops to default `0.5` when offline / no API key.
- Caching: SHA-256(content) → score, stored alongside the FTS5 index. Re-scoring is no-op for unchanged content.

**Effort:** ~1 day. Already follows the same pattern as `cyberos.core.semantic` (soft-dependency probe, falls back gracefully).

**Acceptance:** trivial content ("hello") scores < 0.3; decisions / preferences score > 0.6.

---

### Wave 2 — The headline feature: Dreaming

#### FR-MEM-123 — `cyberos dream` — out-of-band batch reflection

**Why:** the most-cited new capability in the Anthropic talk; Harvey reports 6× completion-rate gain, Rakerton 90% mistake-drop.

**Spec sketch (4-phase pipeline, mirrors the talk's design):**

1. **Input** — `cyberos dream --since 24h [--sessions <id>...] [--scope memories/sre]`
   - Default input: BRAIN audit rows in `[--since]` window plus a separately-emitted **session transcript ledger** (see FR-MEM-127 below)
   - For first ship, input can be just the audit chain + memory diffs

2. **Pattern detection** — spin a Claude session per dream job (delegates to `services/skill-broker` once available; otherwise direct `anthropic.Anthropic()` call). Tasks the dream agent with:
   - Find memories whose content overlaps with `cosine_sim >= 0.92` → propose merge
   - Find memories whose claims contradict the more recent audit rows → propose stale-mark
   - Find recurring patterns across multiple sessions (the talk's "5 agents hit the same 60-second retry pattern" example) → propose new memory under `memories/refinements/`
   - Find memories whose facts the latest sessions actually used and verified → annotate with `meta.last_verified_at`

3. **Output a diff** — written to `dreams/<utc-timestamp>/diff.json`:
   ```json
   {
     "dream_id": "drm_2026-05-19T08:00:00Z",
     "scope": "memories/sre/",
     "input_sessions": ["sess_a", "sess_b", "sess_c"],
     "proposals": [
       {"op": "merge",  "paths": ["a.md", "b.md"], "into": "merged.md", "rationale": "..."},
       {"op": "stale",  "path": "old.md",          "rationale": "contradicted by sess_c at row 4291"},
       {"op": "new",    "path": "memories/refinements/dispatch-retry-pattern.md", "content_preview": "..."},
       {"op": "verify", "path": "facts/x.md",      "verified_against": "sess_b row 4198"}
     ],
     "metrics": {"input_token_cost": 142000, "duration_ms": 84000, "rows_scanned": 2310}
   }
   ```

4. **Apply** — `cyberos dream apply <dream_id> [--proposal-ids ...] [--interactive]`. Each applied proposal becomes ordinary `put`/`move`/`delete(tombstone)` rows on the BRAIN chain with `extra.dream_id`, `extra.proposal_id` for provenance.

**Why this design:**
- Out-of-band → no latency hit on the agent's hot path (talk §"design perspective")
- Diff-then-apply → operator review gate (talk §"manual review")
- Provenance via `extra.dream_id` → audit transparency: every dream-applied row traces back to its job
- Re-runnable: dreams are idempotent given identical inputs

**Effort:** ~5–7 days. Needs:
- `cyberos.core.dream` module
- `dreams/<ts>/diff.json` storage convention (no schema change to existing layout)
- LLM-driven proposal generation (mock-LLM for tests, real Anthropic for production — same pattern as `modules/cuo/cuo/invokers.py` LLMInvoker that already exists)
- New CLI subcommand pair (`dream` + `dream apply`)
- Audit kind addition: `dream.start`, `dream.complete`, `dream.proposal_applied`

**Acceptance:**
- Fixture BRAIN with 3 known duplicates → `cyberos dream` returns 3 merge proposals.
- Fixture BRAIN with one contradiction (memory says "Linear project INGEST"; later audit row says "moved to Jira project PIPE") → `cyberos dream` returns one stale proposal.
- `cyberos dream apply` advances HEAD seq by exactly N rows where N = applied proposal count.

**Protocol amendment required:** one new section in AGENTS.md — `§7.7 Dreaming` — specifying that dream-applied rows MUST carry `extra.dream_id` provenance. Use the `propose-now` grammar: `APPROVE protocol change P19 §7.7`.

---

#### FR-MEM-124 — Semantic-dedup pass inside `cyberos consolidate`

**Why:** lighter-weight cousin of dreaming. The article's nightly `consolidate_memories()` — group by cosine ≥ 0.92, summarise, replace.

**Spec sketch:**
- Add a new phase to the consolidation pipeline: **Walk → Compact → Sign → Publish → SemanticDedup**.
- The SemanticDedup phase produces the same `dreams/<ts>/diff.json` artefact but with `proposals[].op == "merge"` only. Apply is gated (default `--dry-run`).
- Optional flag `cyberos consolidate --semantic-dedup` (off by default — operator must opt in until the dream pipeline shipping confidence is high).

**Effort:** ~2 days, shares code with FR-MEM-123.

---

### Wave 3 — Multi-agent + multi-store scaling

#### FR-MEM-125 — Per-store ACL within a BRAIN (memory-store as a permission scope)

**Why:** the SRE demo in the Anthropic talk attached **three stores** to one agent (one read-only, two read-write). CyberOS currently has one store per project.

**Spec sketch:**
- Logical "store" = a subtree of `<memory-root>/` rooted at one of the existing top-level dirs (`memories/`, `meta/`, `company/`, `client/`, `project/`, etc.).
- Each subtree gets a `STORE.yaml` at its root:
  ```yaml
  store_id: org-wide-knowledge
  acl:
    - {actor: "*",            mode: "read"}
    - {actor: "stephen@*",    mode: "read-write"}
    - {actor: "dream-agent",  mode: "read-write"}  # only the dream pipeline can write here
  default_mode: read
  ```
- `cyberos.core.writer.Writer` enforces ACLs at every `put`/`move`/`delete`. Reads remain unrestricted to all local processes (still subject to OS file permissions).
- `INTEROP.md` gains one line: "Consumers MUST honour `STORE.yaml` `acl` for writes; reads MAY ignore."

**Effort:** ~3 days. Touches writer + walker + invariants.

**Protocol amendment:** new section `§14.4 Store-level ACL`. Use propose-now grammar.

---

#### FR-MEM-126 — `put_if` — optimistic concurrency primitive with precondition-hash

**Why:** the talk's headline concurrency property. The current `.lock`-based serialisation does not scale to 100s of concurrent agents.

**Spec sketch:**
- New op: `put_if(path, body, meta, precondition_body_hash: Optional[str])`.
- If `precondition_body_hash is None` → behaves like `put` (creates if absent).
- If `precondition_body_hash != None` → server reads current `body_hash`; if it matches, commits; otherwise returns `op:"rejected" reason:"precondition_failed"`.
- The lock-based `.lock` remains for ledger-tail consistency; `put_if` only adds an in-application optimistic check, so 1000 agents reading the same memory and then writing back diverged versions converge to one winner per round.
- Agents observe the rejection and either re-read → re-compute → retry, or escalate.

**Effort:** ~1 day. Pure addition; existing `put` keeps its semantics.

**Acceptance:**
- Two simultaneous `put_if` against the same path with the same precondition: one commits, the other returns `precondition_failed`.
- A `put_if` with `precondition_body_hash=None` on a fresh path always succeeds.

**Protocol amendment:** add `put_if` to the canonical-op list in §3.1. Use propose-now grammar.

---

#### FR-MEM-127 — Session-transcript ledger (input source for dreaming)

**Why:** dreaming needs raw conversation context; agents would otherwise rediscover the same patterns from scratch every dream run. The talk explicitly references "input sessions" the dream agent looks through.

**Spec sketch:**
- Optional opt-in: `cyberos session start | append | end`.
- New audit kinds: `session.start | session.turn | session.end`.
- Stored under `sessions/<utc-date>/<session-id>.binlog.zst` (separately compressed; one segment per session).
- Privacy: declared in `manifest.json` — `manifest.sessions.classification: confidential` (default) — encryption envelope MUST apply to body.
- Retention: configurable `manifest.sessions.retention_days: 30` (default). After expiry, only the audit-row hashes remain on chain; body is purged with the same provenance pattern as `delete(purge)`.

**Effort:** ~3 days.

**Acceptance:** `cyberos session start --id s1 ... cyberos session append --id s1 --role user --content ...` → recoverable via `cyberos session read s1`.

---

#### FR-MEM-128 — `cyberos history <path>` ergonomic version-walking

**Why:** Anthropic's demo emphasised that agents can walk per-file history at recall time. CyberOS has the data in the audit chain but no UX surface.

**Spec sketch:**
- `cyberos history memories/decisions/x.md` returns chronological list of all audit rows touching that path, with diffs (body), actor, session_id, ts.
- `cyberos history --reverse --limit 5 <path>` — most recent first.
- Add `/api/v2/memories/<path>/history` to `cyberos serve`.

**Effort:** ~1 day. Pure projection over existing audit rows.

---

### 4.9 Summary table

| FR | Capability | Effort | Wave | Protocol amendment |
|---|---|---|---|---|
| FR-MEM-120 | Episodic memory + recall-similar | 2d | 1 | No (additive enum) |
| FR-MEM-121 | Recency-decay recall ranking | 1d | 1 | No |
| FR-MEM-122 | Write-time importance scoring | 1d | 1 | No |
| FR-MEM-123 | `cyberos dream` (out-of-band reflection) | 5-7d | 2 | Yes — §7.7 |
| FR-MEM-124 | Semantic-dedup in consolidate | 2d | 2 | No (reuses §7) |
| FR-MEM-125 | Per-store ACL via `STORE.yaml` | 3d | 3 | Yes — §14.4 |
| FR-MEM-126 | `put_if` precondition-hash op | 1d | 3 | Yes — §3.1 |
| FR-MEM-127 | Session-transcript ledger | 3d | 3 | Yes — new §18 |
| FR-MEM-128 | `cyberos history <path>` | 1d | 3 | No |

**Wave 1 total: ~4d.** Wave 2 total: ~9d. Wave 3 total: ~8d. End-to-end including FR authoring (10/10 audit per FR per the project convention) realistically ~6–8 weeks of solo work.

---

## 5. Suggested ship sequence

| Week | Drop |
|---|---|
| 1 | FR-MEM-120 + FR-MEM-121 + FR-MEM-122 authored at 10/10 with `.audit.md` siblings |
| 2 | FR-MEM-120, FR-MEM-121, FR-MEM-122 implemented + tested (target 280+ tests green) |
| 3 | FR-MEM-123 authored at 10/10; protocol-amendment §7.7 prepared for chat-turn approval |
| 4-5 | FR-MEM-123 implemented; dogfooded against the CyberOS BRAIN itself (it's now > 0.5 M rows in the audit chain — perfect dataset) |
| 6 | FR-MEM-124 + FR-MEM-128 (quick wins on top of dream infra) |
| 7-8 | FR-MEM-125, FR-MEM-126, FR-MEM-127 (multi-agent scale) |

**Validation milestone after Wave 1:** retain a fixed eval set of "starts a new project, gets 10 similar tasks over a week" and measure first-pass success rate before vs after episodic memory + recall-similar. The article's reflection-loop expectation is a meaningful, measurable lift (Rakerton's 90% mistake-drop is the upper bound to aim at).

**Validation milestone after Wave 2:** dogfood `cyberos dream` against the cyberos repo's own BRAIN (which is now massive: 245 FRs + 221 workflows + 99 skills, each with audit rows). If it finds even a handful of legitimate dedup / stale / new-pattern proposals on the first run, the design is validated.

---

## 6. What NOT to do (anti-patterns surfaced by the comparison)

1. **Don't bolt a vector DB onto every install.** The article uses ChromaDB by default; CyberOS already does the right thing — semantic is an *optional* soft dependency. Keep it that way.

2. **Don't move memory off the filesystem.** Both the article and the Anthropic talk converge on file-system-as-memory. CyberOS already has this. Resist the temptation to replace markdown files with a "memory database" — the file system is the feature.

3. **Don't merge the dream pipeline into the session hot path.** The talk explicitly separates them. Dreaming runs out-of-band on a cron / post-task trigger; sessions stay fast. CyberOS's existing `cyberos consolidate --background` is the right precedent.

4. **Don't have a single global importance threshold.** Importance is contextual. Make `min_importance_to_store` a per-store setting in `STORE.yaml` (FR-MEM-125).

5. **Don't conflate "audit chain integrity" with "memory quality."** The chain proves what was written; it does not prove the content is still true. The dream pipeline is the answer for memory-quality, and it must produce its own first-class audit rows.

---

## 7. Open questions for Stephen

Before authoring the FRs at 10/10, please confirm:

1. **Approval for protocol amendments** — FR-MEM-123 (§7.7 Dreaming), FR-MEM-125 (§14.4 Store ACL), FR-MEM-126 (§3.1 put_if), FR-MEM-127 (new §18 Sessions) each need an explicit `APPROVE protocol change P<n> §<section>` chat-turn per AGENTS.md §0.2 + §16.2. Bundle them, or one-at-a-time per FR?

2. **Default LLM for importance scoring and dreaming** — local-only (Ollama-style + small model) or assume Anthropic API key is available? The current cyberos/cuo Phase 3 already supports `mock-llm` + `anthropic`; reuse that pattern?

3. **Session transcript privacy default** — FR-MEM-127 stores raw conversation. Should the default `classification` be `confidential` (encryption-recommended) or `restricted` (encryption-required)? Restricted is safer but slower.

4. **Store-as-subtree mapping** — for FR-MEM-125, do you want one `STORE.yaml` per existing top-level dir (`memories/`, `meta/`, `company/`, …) auto-generated on migration, or only the dirs you explicitly mark? Auto would land 8-10 stores on every existing BRAIN.

5. **Where to deliver this proposal** — keep this file in `playground/MEMORY-IMPROVEMENT-PROPOSAL.md` (scratchpad), or promote to `modules/memory/docs/PROPOSALS/MEMORY-IMPROVEMENT-WAVE-2026Q3.md`?

---

## References

[1] [MARS: Memory-Enhanced Agents with Reflective Self-improvement](https://consensus.app/papers/details/33146ddb26f25e60877ca1b2c76602aa/?utm_source=claude_code) (Xuechen Liang et al., 2025, ArXiv) — 3-agent framework (User / Assistant / Checker) using iterative feedback + Ebbinghaus-curve forgetting; supports the recency-decay direction taken in FR-MEM-121.

[2] [Evo-Memory: Benchmarking LLM Agent Test-time Learning with Self-Evolving Memory](https://consensus.app/papers/details/82360096847158c2890886056a3d5675/?utm_source=claude_code) (Tianxin Wei et al., 2025, ArXiv) — streaming-task benchmark covering 10 memory modules; their `ReMem` action-think-memory pipeline is essentially the article's reflection loop, formalised. Useful as the eval framework after Wave 1 ships.

[3] [A Survey on the Memory Mechanism of Large Language Model-based Agents](https://consensus.app/papers/details/49b68544092450f9a9c74be746426a4f/?utm_source=claude_code) (Zeyu Zhang et al., 2024, ACM TOIS, 315 citations) — the foundational survey. The three roles (continuity / context / learning) Ramakrushna uses are from here; the article is largely a popularisation of this survey + a code reference.

---

## Sources for the source analysis

- [Playground extract: agentic-memory.article.txt](extracts/agentic-memory.article.txt)
- [Playground extract: memory-and-dreaming.transcript.txt](extracts/memory-and-dreaming.transcript.txt)
- [Original article snapshot (X)](https://x.com/techwith_ram/article/2037499938574110770)

Create or connect a free Consensus account to return more than 3 results per search in Claude Code.: https://consensus.app/sign-up/?utm_source=claude_code&auth=claude_code
