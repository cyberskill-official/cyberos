# `cyberos/runtime/` — runtime build plan + operational tooling

This folder has two roles:

1. **`runtime/tools/`** — eight Python CLIs you run today against any `.cyberos-memory/` (validator, doctor, search index, export, encryption, canonical SHA helper, AGENTS-CORE.md generator, benchmark). These are the operational layer of the local-optimization roadmap. See `runtime/tools/README.md` for the per-tool reference.

2. **The future BRAIN-service runtime** — engineering hand-off plan for when CyberOS's runtime modules ship (transpilers, host shims, LangGraph supervisor, NATS event bus, GraphQL Federation gateway, et al). The four sections below — Plan, Interfaces, Build Order, originally separate files — describe that future state. They were consolidated into this README on 2026-05-10 to reduce file sprawl.

When you're working on operational tooling today, the entry point is `runtime/tools/README.md`. When you're planning the future BRAIN service, the entry point is **Part 1** below.

For the broader CyberOS protocol context (what `.cyberos-memory/` is, how it's governed, what the cookbooks cover, the historical decision trail), see the master navigation hub at `docs/memory/README.md`.

---

## Part 1 — Plan (originally `runtime/PLAN.md`)

> **This folder is a build plan, not a built system.** It documents what the runtime MUST satisfy when implemented, in a form an engineering team can pick up and execute. The CONTRACT-level source of truth is the registry under `cyberos/docs/skills/` + `cyberos/docs/contracts/`. This folder translates that into a concrete delivery plan.

## Why this exists

By registry v0.2.6 (the moment this folder is authored), CyberOS has:

- 7 skills scaffolded across cpo + cto personas, all carrying full v0.2.0 frontmatter.
- 5 contracts (`feature-request@1`, `nats-subjects@1`, `project-brief@1`, `prd@1`, `srs@1`).
- A complete chain from "human idea + BRAIN" through to "engineering tech-specs", documented end-to-end at the contract level.
- Zero executable code.

Every skill carries `gated_until_phase: runtime_v0_3_0`. The supervisor MUST NOT route to any of them until the runtime ships. This folder is the bridge from documented intent to running system.

## Source of truth

Everything in this folder DERIVES from the registry. If this folder ever contradicts the registry, the registry wins. Specifically:

- Skills + invariants live under `cyberos/docs/skills/`.
- Contracts live under `cyberos/docs/contracts/`.
- Architecture decisions live in PRD §5.10/5.11 + SRS §6.1–§6.16 (the .docx files at `cyberos/docs/CyberOS-PRD.docx` + `cyberos/docs/CyberOS-SRS.docx`).
- Audit ledger schema lives in SRS §6.7 + AGENTS.md §7.

This folder's job is to sequence the build, not redesign it.

## Build phases (registry README Part 9 mapping)

| Phase | Name | Deliverable | Estimate | Status |
| --- | --- | --- | --- | --- |
| **A** | CCSM canonicalisation | SKILL.md is already the CCSM (Canonical CyberSkill Skill Manifest); no work. | 0 | ✅ done at registry v0.2.0 |
| **B** | Transpilers | `ccsm-to-anthropic-skill`, `ccsm-to-mcp-tool`, `ccsm-to-claude-plugin`, `ccsm-to-antigravity`, `ccsm-to-codex`, `ccsm-to-cursor`. Pure functions `CCSM → host-artefact-tree`. | 2-3 weeks | 🔵 planned |
| **C** | Host shim library | `cyberos-skill-runtime` (Python) + `@cyberos/skill-runtime` (Node). Provides uniform `runtime.brain` / `runtime.audit` / `runtime.invariants` / `runtime.envelope` / `runtime.untrusted` semantics regardless of host. | 1-2 weeks | 🔵 planned |
| **D** | Equivalence test matrix | Golden input/output runs across every transpilation target. CI gate. | 1 week | 🔵 planned |
| **E** | Partner connector pipeline | Per-skill DEC required for `partner_connector: true`; build pipeline that emits the partner-side artefact. | 2 weeks | 🔵 planned (gated on first DEC) |
| **F** | LangGraph supervisor | Topology per SRS §6.1.1. classify-act node + conditional edges + checkpointing. | 2 weeks | 🔵 planned |
| **G** | `genie.action_log` | Postgres table + tamper detector + hash-chain validator. Schema in SRS §6.7. | 1 week | 🔵 planned |
| **H** | NATS event bus | JetStream config matching `nats-subjects@1` contract. Subjects + QoS + durability. | 0.5 week | 🔵 planned |
| **I** | Auto-refinement engine | Reads `INVARIANTS.md`, runs checks at declared `self_audit.check_at` checkpoints, emits `refinement_proposal` envelopes, pauses pipeline. | 1 week | 🔵 planned |
| **J** | Acceptance-test harness | Per Recipe 8. Loads fixtures from each skill's `acceptance/` folder; runs against transpiled artefact tree; asserts equivalence. | 1 week | 🔵 planned |
| **K** | BRAIN MCP server | `brain.search` + `brain.write_memory` + scope-contract enforcement. Filesystem-local for self-hosted; Postgres-backed for cloud. | 1.5 weeks | 🔵 planned |
| **L** | KB MCP server | `kb.read` + `kb.search`. Pluggable backends (Notion, Confluence, Google Docs, etc.). | 1 week | 🔵 planned |
| **M** | PROJ MCP server | `proj.read` + `proj.create_issue`. Pluggable backends (Linear, Jira, GitHub, etc.). | 1 week | 🔵 planned |
| **N** | CHAT MCP server | `chat.notify` + `chat.review_request`. Slack / Teams / Discord adapters. | 0.5 week | 🔵 planned |
| **O** | EMAIL MCP server | `email.draft` (drafts only — never auto-sends). Gmail / Outlook adapters. | 0.5 week | 🔵 planned |

**Total estimate:** ~17 engineer-weeks for a single engineer; ~6-8 weeks with 2-3 engineers in parallel.

## Critical path

```
[A done] → [G action_log] ┐
                          ├→ [F supervisor] ┐
[H NATS] ─────────────────┘                 ├→ [I auto-refinement] ┐
[K BRAIN] ──────────────────────────────────┘                      ├→ [first chained run]
[C host shim] ┐                                                    │
[B transpilers] ┴→ [J acceptance-test harness] ────────────────────┘
[L/M/N/O peripheral MCPs]  (parallel; needed for end-to-end but not blocking the chain)
```

The blocking pieces are: **G (action_log) + H (NATS) + K (BRAIN) + F (supervisor) + I (auto-refinement)**. With those five, a chained skill run is observable end-to-end. Everything else (transpilers, peripheral MCPs, acceptance harness) can land in parallel.

## How to execute

1. **Read this folder + INTERFACES.md + BUILD_ORDER.md before writing any code.**
2. **Read PRD §5 + SRS §6** for architectural context the build plan summarises but doesn't replicate.
3. **Pick a phase, follow BUILD_ORDER.md.** Each phase has a "definition of done" inline.
4. **Write code under `cyberos/runtime/<component>/`** (Python under `cyberos/runtime/python/`, Node under `cyberos/runtime/node/`).
5. **Don't modify the registry while building.** The registry is the spec; the runtime IS the implementation. If the spec needs to change, the change goes through the registry CHANGELOG first, then implementation follows.
6. **Capture lessons learned in BRAIN under `memories/refinements/REF-NNN-*.md`** as you build. Future maintainers will thank you.

## Known unknowns (worth investigating first)

- **JetStream durability tuning** — actual retention budgets depend on production traffic; the contract sets minimums but ops will tune.
- **BRAIN scope-contract enforcement at the MCP layer** — needs careful design to enforce read_excluded patterns without leaking metadata.
- **Auto-refinement loop runaway** — what if INVARIANTS.md is wrong AND auto-refinement keeps proposing the same refinement? The escalation to manual fine-tune (signal: `self_audit_refinement_proposal_count_above`) is the answer; needs implementation.
- **Antigravity / Codex / Cursor adapter behaviours** — none of the three are documented in detail by their vendors; expect investigation cost in Phase B.

## When this folder retires

Once Phase J (acceptance-test harness) is green and at least one skill has run end-to-end through the chain in production, this folder becomes historical documentation. Move it to `cyberos/runtime/archive/` and replace with a `cyberos/runtime/README.md` pointing at the actual runtime code + its operations docs.

## Citations

- Registry README Part 9 — host-adapter strategy (Phases A-E).
- Registry README Part 12 — runtime architecture (LangGraph + action_log + NATS).
- Registry README Part 26 — honest inventory of what doesn't exist.
- SRS §6.1.1 — supervisor topology.
- SRS §6.7 — `genie.action_log` schema + tamper detector.
- SRS §6.13–§6.16 — runtime mechanisms (skills↔contracts split, dual-mode, self-audit, manual fine-tune, host adapter pipeline).
- AGENTS.md §7 — audit ledger semantics.
- DEC-090..093 — the four locked decisions the runtime implements.


---

## Part 2 — Interfaces (originally `runtime/INTERFACES.md`)

# CyberOS Runtime — Public Interfaces

> The surfaces every skill (Python, Node, or transpiled to a host-native format) sees regardless of host. Implemented by `cyberos-skill-runtime` (Python) and `@cyberos/skill-runtime` (Node). Source of truth: this file + the contracts under `cyberos/docs/contracts/`.

## `runtime.brain`

```python
runtime.brain.search(query: str, scope: str | list[str], limit: int = 10) -> list[Memory]
runtime.brain.write_memory(memory: Memory) -> WriteResult
runtime.brain.read(memory_id: str) -> Memory | None
```

**Scope enforcement:** `query.scope` MUST be a subset of the calling skill's `allowed_brain_scopes.read`. `write_memory` enforces `allowed_brain_scopes.write`. `read_excluded` patterns hide memories at query time (filter applied AFTER scope match, BEFORE return). All ops emit `op:view` (search/read) or `op:create`/`op:str_replace` (write_memory) audit rows per AGENTS.md §7.

**Failure modes:** Out-of-scope query → `ScopeViolationError`; never silent. Network unreachable → `BrainUnreachableError` (skills with INV-001-style "BRAIN must be reachable" invariant MUST refuse; others MAY degrade per their own policy).

## `runtime.audit`

```python
runtime.audit.append(row: AuditRow) -> AppendResult
runtime.audit.read(filter: AuditFilter, limit: int = 100) -> list[AuditRow]
runtime.audit.verify_chain(start: str | None = None, end: str | None = None) -> ChainStatus
```

**Hash-chain enforcement:** every `append` recomputes `chain = sha256(canonical_json(row_without_chain) + prev_chain)` per AGENTS.md §7.2. Mismatch on the prior row → `ChainBrokenError`; runtime surfaces to ops + freezes writes against that path.

**Read access:** filter by `trace_id`, `actor_id`, `op`, `scope`, `path`, `ts` range. `verify_chain` walks the JSONL end-to-end and returns `{valid: bool, last_valid_chain, first_invalid_audit_id?, error?}`.

**Tamper detection:** every read computes the chain's terminal hash and compares to `manifest.audit_chain_head`; mismatch → `ManifestDriftError`.

## `runtime.invariants`

```python
runtime.invariants.check(skill_id: str, checkpoint: str) -> list[Breach]
runtime.invariants.declare_breach(invariant_id: str, observation: dict) -> RefinementProposal
runtime.invariants.refinement_proposed(skill_id: str) -> list[RefinementProposal]  # for the supervisor
```

**Contract:** `check(skill_id, checkpoint)` reads the skill's `INVARIANTS.md`, runs every invariant whose `check_at` includes the current `checkpoint`, returns the breach set. `declare_breach` emits a `refinement_proposal` envelope to NATS (`cuo.refinement_proposed`) AND writes one `op:create` row to `genie.action_log` AND pauses the LangGraph pipeline (sets supervisor checkpoint).

**Anomaly signal accumulation:** the runtime tracks `self_audit.anomaly_signals` per skill_id over rolling windows. When a window threshold is crossed, `declare_breach` is called automatically. The supervisor's classify-act node consults this state when routing.

## `runtime.envelope`

```python
runtime.envelope.validate(envelope: dict, schema_ref: str) -> ValidationResult
runtime.envelope.synthesise_from_interview(skill_id: str, answers: dict) -> dict
runtime.envelope.coerce_to_chained(input_envelope: dict, upstream: SkillId) -> dict
```

**Schema validation:** uses `ajv` (Node) / `jsonschema` (Python). On failure, returns `{valid: false, errors: [...]}` with each error referencing a JSON Pointer into the envelope.

**Standalone-mode interview:** when a skill is invoked from chat without an envelope, the runtime loads the skill's `STANDALONE_INTERVIEW.md`, walks the user through it, and synthesises an envelope. `coerce_to_chained` upgrades a synthesised envelope with chain-mode metadata when an upstream skill triggered the run.

## `runtime.untrusted`

```python
runtime.untrusted.wrap(content: str) -> str
runtime.untrusted.scan_for_injection(content: str) -> list[InjectionMarker]
runtime.untrusted.unwrap_for_user_display(content: str) -> str
```

**Wrap policy:** every text from external systems (KB documents, email bodies, web content, BRAIN entries containing `provenance.source: imported` etc.) MUST be wrapped in `<untrusted_content>` tags before reaching skill bodies. The runtime enforces at the MCP boundary.

**Injection scan:** runs the marker set from AGENTS.md §4.2 + each skill's reference docs (UNTRUSTED_CONTENT.md). Returns marker locations + severity. Severity ≥ warning → `on_marker_hit: surface_to_human` policy fires.

## `runtime.nats`

```python
runtime.nats.publish(subject: str, payload: dict, qos: QoS) -> PubAck
runtime.nats.subscribe(subject: str, durable_name: str, ack_policy: AckPolicy) -> Subscription
```

**Subject validation:** every `publish` validates the subject against `nats-subjects@1` contract's inventory. Unrecognised subject → `UnknownSubjectError`. Payload validated against `schema.json#/payloads/<event_name>`.

**QoS levels:** `at_most_once` / `at_least_once` / `exactly_once` per `protocol.md`. Default for skill emissions: `at_least_once` with `Nats-Msg-Id` header for dedup.

## `runtime.kb / runtime.proj / runtime.chat / runtime.email`

Pluggable backends. Each exposes:

- `kb.read(uri: str) -> Document`
- `kb.search(query: str, source: str = "default", limit: int = 10) -> list[Document]`
- `proj.read(ticket_id: str) -> Ticket`
- `proj.create_issue(title: str, body: str, labels: list[str], assignee: str | None) -> Ticket`
- `chat.notify(channel: str, message: str) -> Message` (drafts only — `chat.send` requires explicit human approval per AGENTS.md prohibited-actions)
- `chat.review_request(reviewers: list[str], artefact: str) -> ReviewRequest`
- `email.draft(to: list[str], subject: str, body: str) -> DraftId` (drafts only; `email.send` is human-only)

Backends declared per-tenant in `manifest.json` under `mcp_backends:`. Auto-routing chooses a backend by matching the call's hostname + URI scheme.

## Envelope of envelopes — supervisor → skill invocation

```python
SkillInvocation = {
    "skill_id": str,                    # "cuo/cpo/fr-author"
    "skill_version": str,               # "0.2.2"
    "trace_id": str,                    # uuid
    "caller_persona": str,              # "cuo-cpo" or "cuo" if direct
    "input_envelope": dict,             # validated against skill's expects.schema_ref
    "checkpoint_state": dict | None,    # for resume-from-pause
    "upstream_context": dict | None,    # populated when chained
}
```

The supervisor builds this object, calls `runtime.invoke_skill(invocation)`, and the runtime:

1. Validates the input envelope against the skill's schema.
2. Loads the skill's body (SKILL.md frontmatter + body) into the model context.
3. Loads the skill's reference docs lazily (only when the skill body cites them).
4. Runs the skill (LLM inference + tool calls).
5. Validates the output envelope against the skill's `produces.schema_ref`.
6. Appends every audit row in order, with chain-hash continuity.
7. Publishes any NATS events the skill emitted.
8. Returns the output envelope to the supervisor for routing decisions.

## Citations

- SRS §6.13–§6.16 — runtime mechanisms.
- AGENTS.md §7 — audit ledger semantics.
- AGENTS.md §4 — file-operation contracts.
- `cyberos/docs/contracts/nats-subjects/CONTRACT.md` — wire-protocol surface.
- `cyberos/docs/skills/cuo/_shared/hello-world/SKILL.md` — first skill the runtime should run end-to-end as the smoke test.


---

## Part 3 — Build Order (originally `runtime/BUILD_ORDER.md`)

# CyberOS Runtime — Build Order

> Concrete sequence with definition-of-done per phase. Source of truth for "is this phase done?" is the `definition_of_done` block.

## Recommended sequence (single engineer)

If you have one engineer, work through phases in this order. Skip to the parallel section if you have multiple.

### Phase G — `genie.action_log`

**Why first:** every other component writes audit rows; without action_log nothing else can ship.

**Build:**
1. Postgres schema migration. Schema in SRS §6.7.
2. `runtime.audit.append` Python + Node implementations.
3. `runtime.audit.read` + `runtime.audit.verify_chain`.
4. Unit tests for chain integrity (the protocol that protects everything).

**Definition of done:**
- 1000 sequential appends produce a valid chain (verified by `verify_chain`).
- 1 tampered row (mid-chain) is detected by `verify_chain` with the correct first-invalid index.
- Cross-month rollover preserves the chain via `prev_chain` of first row in new month.
- Manifest's `audit_chain_head` always points at a real chain in the ledger.

**Estimate:** 1 week.

---

### Phase H — NATS event bus

**Why next:** the supervisor uses NATS to receive `cuo.refinement_proposed` and to chain between skills.

**Build:**
1. NATS cluster config (3-node, JetStream enabled). Per-tenant.
2. JetStream streams matching `nats-subjects@1` contract: subject names + QoS + durability tiers from CONTRACT.md inventory.
3. `runtime.nats.publish` + `runtime.nats.subscribe` Python + Node.
4. Subject validator (rejects publish to unregistered subjects).
5. Payload validator (per `schema.json#/payloads/<event_name>`).

**Definition of done:**
- `pub` to a contract-registered subject persists per its declared durability.
- `pub` to an unregistered subject → `UnknownSubjectError`.
- `sub` with durable name reconnects after restart and resumes from last-acked.
- `at-least-once` redelivery works; `Nats-Msg-Id` dedup works.

**Estimate:** 0.5 week.

---

### Phase K — BRAIN MCP server

**Why next:** every skill reads BRAIN; the supervisor classifies routes by reading persona memories.

**Build:**
1. Filesystem-local backend (`.cyberos-memory/` directly per AGENTS.md). Default for self-hosted.
2. Postgres-backed backend for cloud (mirror filesystem layout in DB tables; same schema).
3. `runtime.brain.search` + `runtime.brain.write_memory` + `runtime.brain.read`.
4. Scope-contract enforcement: callers' `allowed_brain_scopes.read` constrains the result set; `allowed_brain_scopes.write` constrains writes; `read_excluded` patterns filter results.
5. `op:view` audit rows for every search; `op:create`/`op:str_replace` for writes.

**Definition of done:**
- Search across `project:*` returns only project memories.
- Search with `read_excluded: member:*/private/` does NOT return private member memories.
- Write to `member:*` from a skill that lacks the scope → `ScopeViolationError`.
- All ops emit valid audit rows with chain continuity.

**Estimate:** 1.5 weeks.

---

### Phase F — LangGraph supervisor

**Why next:** with action_log + NATS + BRAIN, the supervisor can be built and traced end-to-end.

**Build:**
1. LangGraph state machine per SRS §6.1.1.
2. classify-act node: reads incoming user message, classifies into a skill_id, builds invocation envelope.
3. Conditional edges: per skill's `next_skill_recommendation` field, route to follow-up or terminate.
4. Checkpoint state at every node boundary (LangGraph built-in).
5. HITL pause + resume: when a skill's output sets `outcome: HALTED_HITL`, supervisor halts and surfaces; resume reads the answered HITL_BATCH_REQUEST and re-invokes the skill with `checkpoint_state`.
6. Crash recovery: per AGENTS.md §4.7, walk recent audit rows on startup; reconcile any orphan `session.start`.

**Definition of done:**
- A `cuo/_shared/hello-world` skill invoked via chat runs end-to-end + writes its audit row + emits its NATS event.
- A two-skill chain (fake skills A → B) routes correctly when A emits `next_skill_recommendation: B`.
- A HITL pause + resume round-trip preserves trace_id + checkpoint_state.
- Supervisor crash mid-chain is recovered cleanly on restart.

**Estimate:** 2 weeks.

---

### Phase I — Auto-refinement engine

**Why next:** with the supervisor running, the auto-refinement loop can fire on real invariant breaches.

**Build:**
1. Reads `INVARIANTS.md` for the running skill at every `self_audit.check_at` checkpoint.
2. Runs each invariant's check (deterministic predicate against state).
3. Tracks `self_audit.anomaly_signals` over rolling windows per skill_id.
4. On breach, calls `runtime.invariants.declare_breach` → emits `cuo.refinement_proposed` → writes audit row → pauses supervisor.
5. Supervisor's classify-act node has a "refinement_proposal pending" branch that surfaces the proposal as a Question primitive to the user.

**Definition of done:**
- Manually trigger an INV-001 breach in a fake skill; the runtime emits the proposal, pauses, the user sees the Question.
- The user's APPROVE/REVISE/REJECT response routes correctly.
- Anomaly-signal windows reset cleanly across pause/resume.

**Estimate:** 1 week.

---

### Phase B — Transpilers

**Why now:** the runtime works for Python skills; transpile to other host formats.

**Build (one transpiler per target):**
1. `ccsm-to-anthropic-skill` — emit a flat Anthropic SKILL.md (drop CyberOS-specific frontmatter; preserve body).
2. `ccsm-to-mcp-tool` — emit a `tool.json` from `expects:` + `produces:` schemas.
3. `ccsm-to-claude-plugin` — emit Claude Code plugin manifest.
4. `ccsm-to-antigravity` — investigate format; emit.
5. `ccsm-to-codex` — emit Codex agent format.
6. `ccsm-to-cursor` — emit `.cursorrules` snippet.

Each is a pure function `CCSM → host-artefact-tree`. CI runs `pytest` style equivalence tests.

**Definition of done:** every transpiler produces an artefact tree that passes a host-specific smoke test (e.g., the Anthropic SKILL.md loads without error in Anthropic's tooling).

**Estimate:** 2-3 weeks (1 transpiler per 3 days).

---

### Phase C — Host shim library

**Why now:** transpiled skills need a shim to provide uniform `runtime.*` semantics on hosts that don't have CyberOS MCP servers natively.

**Build:**
1. `cyberos-skill-runtime` Python package: implements `runtime.*` interfaces with filesystem-local fallbacks for BRAIN + audit when MCP servers are unreachable.
2. `@cyberos/skill-runtime` Node package: same.
3. Degraded-mode contract: when a host doesn't have the full runtime, the shim falls back to filesystem-local BRAIN + JSONL audit log. Skills still work; they just don't get cross-tenant routing.

**Definition of done:** a transpiled skill runs in Claude Code (Anthropic Skill format) using the shim, with full BRAIN scope enforcement + audit chain continuity, even though Claude Code has no native CyberOS server.

**Estimate:** 1-2 weeks.

---

### Phase J — Acceptance-test harness

**Why now:** with everything else running, run the per-skill `acceptance/` fixtures as CI gates.

**Build:**
1. Loads each skill's `acceptance/<NN>-<slug>/` directory.
2. Runs the input fixture through the runtime end-to-end.
3. Diffs the output against the expected output (per `acceptance/<NN>-<slug>/expected-output/`).
4. Reports per-fixture pass/fail.
5. Fails CI on any sev-0 fixture failure.

**Definition of done:** every priority sev-0 scenario in every skill's `acceptance/README.md` has a real fixture + passes.

**Estimate:** 1 week.

---

### Phases L/M/N/O — Peripheral MCP servers (in parallel)

KB, PROJ, CHAT, EMAIL. Each is a thin MCP wrapper around an external service's API.

**Definition of done:** each backend implements its respective `runtime.*` interface contract; smoke test invokes one operation against a sandbox account.

**Estimate:** 0.5-1 week each; can run in parallel with later phases.

---

### Phase E — Partner connector pipeline (gated)

**Trigger:** first per-skill DEC for `partner_connector: true`. Until then, skip.

**Build:** transpilation pipeline that emits a partner-side artefact (likely an MCP server image or REST API). Includes per-skill rate limit, per-tenant auth, billing hooks.

**Estimate:** 2 weeks (post-trigger).

## Recommended sequence (multiple engineers)

If you have 2-3 engineers, run these phase clusters in parallel:

- **Engineer 1 (foundation):** G → H → K → F → I (the critical path).
- **Engineer 2 (transpilation):** B → C → J.
- **Engineer 3 (peripheral MCP):** L + M + N + O in any order.

Total wall-clock with 3 engineers: ~6-8 weeks vs. ~17 weeks single-engineer.

## Citations

- SRS §6.1–§6.16 — runtime architecture details.
- Registry README Part 9 — host-adapter strategy phases.
- Registry README Part 26 — what doesn't exist yet.


---

*This consolidated README replaces 4 separate files (README.md, PLAN.md, INTERFACES.md, BUILD_ORDER.md) as of 2026-05-10. Originals deleted in the same change. Future-state runtime work resumes when CyberOS-the-product begins shipping the BRAIN service.*
