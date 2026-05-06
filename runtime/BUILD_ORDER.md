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
