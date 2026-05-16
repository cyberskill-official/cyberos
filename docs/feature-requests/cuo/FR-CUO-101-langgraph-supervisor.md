---
id: FR-CUO-101
title: "CUO Phase 2 — LangGraph supervisor + LiteLLM cascade + confidence-band escalation + persona-aware routing + BRAIN audit per decision"
module: CUO
priority: MUST
status: draft
verify: T
phase: P0
milestone: P0 · exit
slice: 2
owner: Stephen Cheng (CPO)
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-AI-006, FR-AI-007, FR-AI-008, FR-AI-014, FR-AI-022, FR-AUTH-004, FR-AUTH-101, FR-BRAIN-101, FR-AI-003, FR-CUO-102, FR-CUO-103, FR-CUO-104, FR-CUO-105]
depends_on: [FR-AI-008]
blocks: [FR-CUO-102, FR-CUO-103, FR-CUO-104, FR-CRM-005, FR-CRM-006, FR-CRM-007, FR-DOC-009, FR-EMAIL-008, FR-INV-010, FR-OKR-006, FR-OKR-007, FR-PROJ-011, FR-PROJ-012, FR-RES-004]   # all 14 entries are placeholders — not yet specified (downstream consumers of CUO)

source_pages:
  - website/docs/modules/cuo.html#what
  - website/docs/modules/cuo.html#architecture
  - website/docs/modules/cuo.html#routing-flow
  - cuo/docs/AGENTS.md §3.2 (Phase 2 — LLM-driven router)
source_decisions:
  - DEC-160 (LangGraph supervisor — durable state machine, Postgres-checkpointed, EU AI Act Art. 12 compliant)
  - DEC-161 (LiteLLM as the LLM abstraction inside CUO; routes provider calls THROUGH AI Gateway FR-AI-008, never direct)
  - DEC-162 (Phase 2 LLM cascade triggers when confidence ∈ [0.10, 0.50] AND top candidate ≥ 0.10; otherwise rule-based decision stands)
  - DEC-163 (Phase 2 LLM output is structured pick `{skill_name, arguments, rationale, confidence}` validated via Pydantic; freeform text rejected)
  - DEC-164 (defer-to-human matrix is intrinsic to the persona — never overridable by env / config; matches EU AI Act Art. 26 human-oversight guarantee)
  - DEC-165 (every decision — rule path AND LLM path — emits a `cuo.routing_decision` BRAIN audit row; never silent)
  - DEC-166 (persona JWT carries `agent_persona: cuo-<persona-key>@<semver>` per FR-AI-014; the supervisor validates it at every state entry)
  - DEC-167 (LangGraph state schema is versioned via `cuo_state_v` integer; backward-compatible state replay tolerated for 2 versions)
  - DEC-168 (router state machine has exactly 5 nodes: `parse → rule_score → branch{auto|ask|cascade|defer} → invoke → record`; no other transitions)
  - DEC-169 (LLM cascade has a hard 3-second budget; timeout → fall through to "ask clarification" — never silently pick low-confidence skill)
  - DEC-170 (CUO never invokes a destructive skill on confidence alone; the capability broker FR-SKILL-104 enforces destructive-confirm regardless of supervisor decision)
  - EU AI Act Art. 12 (logging requirement — every AI decision retained 6 months minimum, replayable)
  - EU AI Act Art. 26 (human-oversight guarantee — defer-to-human is the operator's right, not a default)
  - EU AI Act Art. 13 (transparency — end-of-response surfaces chosen skill + confidence + alternatives)

language: python 3.12 + rust 1.81 (shared exit-code crate)
service: cyberos/cuo/
new_files:
  - cuo/cuo/supervisor/__init__.py                    # public API: Supervisor, run_supervisor, RoutingDecision
  - cuo/cuo/supervisor/graph.py                       # LangGraph StateGraph definition (5 nodes; closed transitions)
  - cuo/cuo/supervisor/state.py                       # CuoState TypedDict (versioned: cuo_state_v=1)
  - cuo/cuo/supervisor/nodes/parse.py                 # parse node — NFC + query envelope
  - cuo/cuo/supervisor/nodes/rule_score.py            # rule-based scorer — wraps existing Phase 1 router.py
  - cuo/cuo/supervisor/nodes/branch.py                # confidence-band branching (≥0.70 / 0.50-0.70 / 0.10-0.50 / <0.10)
  - cuo/cuo/supervisor/nodes/llm_cascade.py           # LLM cascade — LiteLLM call via AI Gateway FR-AI-008
  - cuo/cuo/supervisor/nodes/invoke.py                # invoke node — delegates to skill module CLI; capture stdout/stderr
  - cuo/cuo/supervisor/nodes/record.py                # record node — emit cuo.routing_decision BRAIN row + persona stamp
  - cuo/cuo/supervisor/litellm_proxy.py               # LiteLLM-shaped client that routes through FR-AI-008 (never direct provider call)
  - cuo/cuo/supervisor/persona.py                     # 11-persona catalogue (Genie + 10 C-level); defer-to-human matrix
  - cuo/cuo/supervisor/checkpointer.py                # Postgres checkpointer (in-memory at slice 2; Postgres ships in FR-CUO-102)
  - cuo/cuo/supervisor/transparency.py                # EU AI Act Art. 13 end-of-response disclosure builder
  - cuo/cuo/supervisor/audit.py                       # canonical cuo.routing_decision row builder (kind, payload, chain)
  - cuo/cuo/supervisor/errors.py                      # ExitCode re-export from cyberos-cli-exit + supervisor-specific codes
  - cuo/cuo/cli/supervisor.py                         # `cyberos-cuo supervisor route --query "..." [--invoke] [--record]`
  - cuo/tests/test_supervisor_graph.py                # graph topology — exactly 5 nodes, closed transitions
  - cuo/tests/test_supervisor_rule_path.py            # confidence ≥ 0.70 → auto-invoke, no LLM
  - cuo/tests/test_supervisor_ask_path.py             # confidence 0.50–0.70 → clarification with top 3
  - cuo/tests/test_supervisor_cascade_path.py         # confidence 0.10–0.50 → LLM cascade, structured pick validation
  - cuo/tests/test_supervisor_defer_path.py           # confidence < 0.10 → defer-to-human, no invocation
  - cuo/tests/test_supervisor_persona_defer_matrix.py # destructive op never auto-invoked regardless of confidence
  - cuo/tests/test_supervisor_audit_row.py            # every path emits exactly one cuo.routing_decision row
  - cuo/tests/test_supervisor_litellm_routes_via_gateway.py  # litellm_proxy MUST call AI Gateway, MUST NOT direct provider
  - cuo/tests/test_supervisor_llm_cascade_timeout.py  # 3s budget — fall through to clarification
  - cuo/tests/test_supervisor_freeform_rejection.py   # LLM response without structured shape → rejected, fall through
  - cuo/tests/test_supervisor_state_version.py        # cuo_state_v=1 in state; replay tolerates ±2 versions
  - cuo/tests/test_supervisor_persona_jwt.py          # rejects requests with mismatched agent_persona JWT
  - cuo/tests/test_supervisor_transparency.py         # end-of-response includes skill + confidence + alternatives
  - cuo/tests/test_supervisor_idempotency.py          # same query + same catalog snapshot → same decision (replay-equivalence)
  - cuo/tests/fixtures/golden_queries.jsonl          # 15 golden fixtures — extends Phase 1's set with 10 ambiguous-tail queries
modified_files:
  - cuo/cuo/core/router.py                            # expose score_one_off() for supervisor's rule_score node to reuse
  - cuo/cuo/core/__init__.py                          # re-export Supervisor for top-level import
  - cuo/cuo/cli/__init__.py                           # mount `cyberos-cuo supervisor` subcommand
  - cuo/pyproject.toml                                # +langgraph ==0.2.*, +litellm ==1.52.*, +pydantic >=2.7, +httpx, +psycopg[binary] (for checkpointer scaffolding)
  - cuo/docs/AGENTS.md                                # §3.2 Phase 2 normative — replace "(pending)" with this FR's contract
  - services/ai-gateway/src/handlers/chat.rs          # accept `X-CUO-Decision-Id` header so the supervisor's row chains to the AI Gateway row

allowed_tools:
  - file_read: cuo/**
  - file_read: services/ai-gateway/src/**
  - file_read: docs/feature-requests/ai/FR-AI-008-multi-provider-router.md
  - file_write: cuo/{cuo,tests}/**
  - bash: cd cuo && uv run pytest tests/test_supervisor_*
  - bash: cd cuo && uv run python -m cyberos.cuo.cli supervisor route --query "..."

disallowed_tools:
  - call any LLM provider (Bedrock, Anthropic, OpenAI) directly from CUO; ALL provider calls MUST go through AI Gateway FR-AI-008 (per DEC-161)
  - introduce a 6th supervisor node (per DEC-168 — graph topology is closed; new nodes require a new FR)
  - allow operator to disable the defer-to-human matrix via env var or config (per DEC-164 + EU AI Act Art. 26)
  - emit a routing decision without a `cuo.routing_decision` BRAIN audit row (per DEC-165 — never silent)
  - accept unstructured LLM responses (per DEC-163 — structured-pick only)
  - silently auto-invoke a destructive skill (per DEC-170 — capability broker FR-SKILL-104 enforces; supervisor refuses regardless)

effort_hours: 12
sub_tasks:
  - "0.5h: pyproject.toml dep additions (langgraph, litellm, pydantic, httpx, psycopg[binary])"
  - "1.0h: state.py — CuoState TypedDict; versioned cuo_state_v=1; per-node transitions documented"
  - "1.0h: graph.py — StateGraph(CuoState) with 5 nodes + conditional edges from `branch`"
  - "0.5h: nodes/parse.py — NFC normalisation, query envelope construction"
  - "0.5h: nodes/rule_score.py — wraps existing core/router.py; populates state['rule_scores']"
  - "0.5h: nodes/branch.py — confidence-band branching with hard thresholds 0.10 / 0.50 / 0.70"
  - "1.5h: nodes/llm_cascade.py — LiteLLM call via litellm_proxy + Pydantic-validated structured pick + 3s timeout"
  - "1.0h: nodes/invoke.py — delegate to skill CLI; capture stdout/stderr; map to InvocationResult"
  - "1.0h: nodes/record.py — build cuo.routing_decision row; chain via brain_writer FR-BRAIN-101 + FR-AI-003"
  - "0.5h: litellm_proxy.py — LiteLLM-shaped client that routes through AI Gateway FR-AI-008"
  - "0.5h: persona.py — 11-persona catalogue (Genie + 10 C-level) + defer-to-human matrix"
  - "0.5h: checkpointer.py — in-memory checkpointer scaffold (Postgres lands in FR-CUO-102)"
  - "0.5h: transparency.py — Art. 13 end-of-response disclosure"
  - "0.5h: audit.py — canonical row builder"
  - "0.5h: cli/supervisor.py — `cyberos-cuo supervisor route` subcommand"
  - "2.5h: Tests — 12 test files covering all 4 confidence paths + persona matrix + audit row + LiteLLM routing + timeout + freeform rejection + state versioning + persona JWT + transparency + idempotency"

risk_if_skipped: "CUO's Phase 1 rule router handles unambiguous queries fine but degrades to defer-to-human for the entire ambiguous tail. Every downstream FR that needs CUO routing (FR-PROJ-011 blocker notification, FR-PROJ-012 cycle review, FR-CRM-005 next-action skill, FR-EMAIL-008 Genie subject prefix, FR-INV-010 dunning draft, FR-OKR-006 Monday digest, FR-DOC-009 renewal proposal, FR-RES-004 hiring memo) assumes CUO can resolve ambiguous queries. Without the LangGraph supervisor: (1) ambiguous queries dead-end at 'defer-to-human' surfacing alternatives, requiring manual operator pick for every uncertain request; (2) no per-decision audit replay (EU AI Act Art. 12 compliance gap); (3) no structured persona routing — every persona uses the same fallback logic instead of its own keyword bank + defer matrix; (4) no transparent end-of-response disclosure (Art. 13 gap). The cost of skipping is a degraded CUO that handles only the easy half of the workload — and forces every downstream consumer to implement its own ambiguity-resolution logic, fragmenting the catalogue routing into per-module inventions that drift."
---

## §1 — Description (BCP-14 normative)

The CUO service **MUST** ship the LangGraph supervisor that adds the Phase 2 LLM cascade on top of the Phase 1 rule router, with persona-aware routing, BRAIN audit per decision, and EU AI Act Art. 12/13/26 compliance. Each requirement:

1. **MUST** implement the supervisor as a `langgraph.graph.StateGraph` over the `CuoState` TypedDict (defined in `state.py`) with exactly 5 nodes (per DEC-168): `parse`, `rule_score`, `branch`, `invoke`, `record`. The `branch` node is the conditional entry point that selects among four downstream paths: `auto`, `ask`, `cascade`, `defer`. The `cascade` path runs the LLM and re-enters `branch` with the LLM's confidence — no infinite loop because `cascade` is taken at most once per request (enforced by a `cascade_taken: bool` flag in state).

2. **MUST** route every NL request through the supervisor when invoked via `cyberos-cuo supervisor route --query "<text>"`. The CLI is the canonical entry point; library users invoke `from cyberos.cuo.supervisor import run_supervisor`.

3. **MUST** make the `parse` node the first step. It NFC-normalises the query (preserving diacritics for VN region scoring per existing Phase 1 behaviour), constructs the query envelope `{query, tenant_id, subject_id, persona_key, ts_ns, request_id}`, and validates `persona_key ∈ {genie, ceo, coo, cfo, cmo, cto, chro, cso, clo, cdo, cpo}`. Unknown persona → `BAD_REQUEST` (exit 65) and a `cuo.persona_unknown` BRAIN audit row.

4. **MUST** make the `rule_score` node run the existing Phase 1 rule scorer (`cyberos.cuo.core.router.score_one_off`) unchanged. The result populates `state['rule_scores']: list[Candidate]` sorted by descending confidence. This guarantees backward-compatibility with the Phase 1 deterministic golden-fixture suite (15 rule-path fixtures still pass).

5. **MUST** make the `branch` node select the next path using the **confidence-band table** (per DEC-162):
    - top candidate confidence ≥ **0.70** → `auto` path (invoke).
    - top candidate confidence ∈ [0.50, 0.70) → `ask` path (return top 3 alternatives; no invocation).
    - top candidate confidence ∈ [0.10, 0.50) **AND** `cascade_taken == false` → `cascade` path (LLM).
    - top candidate confidence < 0.10 → `defer` path (return decision `{routed: false, alternatives: []}`; no invocation).
    - `cascade_taken == true` AND LLM-output confidence < 0.70 → fall through to `ask`.

6. **MUST** make the `llm_cascade` node call the LLM through the `litellm_proxy` (which routes through AI Gateway FR-AI-008, never direct provider call — per DEC-161). The cascade:
   - Sends a structured prompt containing the query envelope + the top 5 rule-scored candidates + the persona-specific system prompt.
   - Expects a **structured response** validated by Pydantic (per DEC-163): `LlmRoutingPick {skill_name: str, arguments: dict, rationale: str, confidence: float}`. Schema-conformance failure → fall through to `ask` (never silently invoke).
   - Has a **hard 3-second budget** (per DEC-169). Timeout → fall through to `ask`; emit `cuo.llm_cascade_timeout` BRAIN audit row.
   - Sets `cascade_taken = true` AND `state['llm_pick'] = LlmRoutingPick(...)` AND re-enters `branch` with the LLM-derived confidence.

7. **MUST** make the `invoke` node delegate skill execution to the skill module's CLI per cuo/docs/AGENTS.md §0.5 (CUO does NOT implement skill execution). The node captures `stdout`, `stderr`, `exit_code`, `duration_ms` into `state['invocation_result']`. If the chosen skill is annotated `destructive: true` in the catalog, the capability broker (FR-SKILL-104) MUST gate via Elicitation flow; the supervisor refuses to bypass — even at confidence 1.0 (per DEC-170).

8. **MUST** make the `record` node emit exactly one `cuo.routing_decision` BRAIN audit row per request, in EVERY path (auto, ask, cascade, defer). The row carries:
    - `tenant_id`, `subject_id_hash16`, `persona_key`, `persona_version`, `agent_persona_jwt_iss` (from JWT).
    - `query` (NFC-normalised; PII-scrubbed via FR-BRAIN-111 before commit if persona requires).
    - `rule_scores` (top 3 with confidence values).
    - `path_taken` ∈ {`auto`, `ask`, `cascade`, `defer`, `cascade_then_ask`}.
    - `llm_pick` (present iff `path_taken` involved cascade).
    - `invocation_result` (present iff path = `auto` or `cascade_then_auto`).
    - `cuo_state_v` (per DEC-167, currently `1`).
    - `request_id`, `trace_id` (W3C-formatted, lower-hex 32-char per AUTHORING.md rule 24).
    - `ts_ns_start`, `ts_ns_end`.

9. **MUST** load the **11-persona catalogue** from `cuo/cuo/supervisor/persona.py`: Genie + 10 C-level (CEO, COO, CFO, CMO, CTO, CHRO, CSO, CLO, CDO, CPO). Each persona has: `key`, `display_name`, `keyword_bank` (list of trigger words), `system_prompt`, `defer_to_human_matrix` (list of operation types the persona MUST refuse to auto-invoke regardless of confidence). The matrix is **intrinsic** to the persona — not overridable by config (per DEC-164 + EU AI Act Art. 26).

10. **MUST** validate the caller's JWT carries `agent_persona: cuo-<persona-key>@<semver>` matching the requested persona. Missing or mismatched persona claim → `403 FORBIDDEN` (exit 77) with body `{"error":"persona_mismatch","claimed":"<x>","requested":"<y>"}`. Validation uses FR-AUTH-101's `RoleMatrix` for `agent-persona` role + FR-AI-014's persona-version stamping.

11. **MUST** route all LLM calls through the **`litellm_proxy`** module — a thin LiteLLM-shaped client that forwards to AI Gateway FR-AI-008 (POST `/v1/ai/chat`). The proxy MUST NOT include direct provider SDKs (no `import boto3`, no `import anthropic`, no `import openai`). The architectural test `test_supervisor_litellm_routes_via_gateway` asserts this by AST-walking the supervisor package and rejecting forbidden imports.

12. **MUST** use a **Postgres checkpointer** for LangGraph state persistence per EU AI Act Art. 12 (logging requirement). Slice 2 (this FR) ships an in-memory checkpointer scaffold — the production Postgres-backed checkpointer ships in FR-CUO-102. The state schema is versioned: `cuo_state_v = 1` (per DEC-167); replays from `cuo_state_v` within ±2 of current are tolerated, beyond rejected with `state_version_unsupported`.

13. **MUST** support **transparent end-of-response disclosure** (EU AI Act Art. 13): every supervisor return value includes `transparency: {skill_chosen, confidence, alternatives: list[{skill, confidence}], path_taken, llm_used: bool}` so the caller can render the disclosure to the user. CHAT/EMAIL/PROJ surfaces consume this for the "🤖 routed via Genie → cfo persona → vn-vat-invoice@1.2 · 0.84" footer.

14. **MUST** emit OTel span `cuo.supervisor.route` per request with attributes: `tenant_id`, `subject_id_hash16`, `persona_key`, `path_taken`, `confidence`, `llm_used`, `outcome` (success | routed_false | invoke_error | persona_mismatch | timeout | unknown_persona). Sampling: 1% steady-state; 100% on non-success outcomes.

15. **MUST** emit child spans for each node entry/exit: `cuo.supervisor.node.parse`, `.rule_score`, `.branch`, `.llm_cascade`, `.invoke`, `.record`. Spans carry the W3C `traceparent` propagated from caller; supervisor extends the trace through to the AI Gateway call so a single trace covers query → LLM → invoke → audit.

16. **MUST** emit OTel metrics:
    - `cuo_supervisor_route_total{outcome, persona, path_taken}` (counter).
    - `cuo_supervisor_route_latency_ms{path_taken}` (histogram; SLOs: auto p95 < 50ms, ask p95 < 50ms, cascade p95 < 3500ms, defer p95 < 30ms).
    - `cuo_supervisor_llm_cascade_total{outcome}` (counter; outcome ∈ {success, timeout, schema_violation, gateway_error}).
    - `cuo_supervisor_persona_defer_blocks_total{persona, operation}` (counter — defer-matrix refusals).
    - `cuo_supervisor_destructive_block_total{skill}` (counter — capability-broker refusals reflected back).

17. **MUST** guarantee **replay-equivalence** for the rule path: same `(query, persona_key, catalog_snapshot_hash)` → identical decision. The `test_supervisor_idempotency` test asserts this on the 15-fixture golden set; CI fails on any non-determinism.

18. **MUST NOT** invoke a skill on the **defer path**. The decision returned has `routed: false`, `alternatives: [<top 3>]`, and the caller (CHAT, EMAIL, etc.) renders the alternatives for the operator to pick — never auto-invoking. This is the EU AI Act Art. 26 hard guarantee.

19. **MUST** treat the **`defer_to_human_matrix`** as ROLE-INTRINSIC: even if the supervisor scores `vn-vat-invoice@1.2 + 0.95` for a CFO persona, if `cfo.defer_to_human_matrix` lists `invoice_emit`, the supervisor refuses to auto-invoke and returns `{routed: false, reason: "persona_defer_matrix", operation: "invoice_emit"}`. Emit `cuo.persona_defer_block` BRAIN audit row.

20. **MUST** support **two invocation modes** via CLI flags:
    - `--invoke` (default true): runs through to `invoke` node when path = `auto` or `cascade_then_auto`.
    - `--record` (default true): runs through to `record` node and emits the BRAIN row.
    Both flags MAY be `--no-invoke` / `--no-record` for dry-run analysis (e.g. "what would the supervisor do without actually invoking?"). Dry-run results are NOT recorded (no BRAIN row); they emit a `cuo.dry_run` OTel span with low sampling.

21. **MUST** complete the **rule path (auto/ask/defer)** in ≤ 50 ms p95 measured at supervisor entry → return. The LLM cascade path is budgeted at ≤ 3500 ms p95 (3000 ms LLM + 500 ms supervisor overhead). Performance test `test_supervisor_perf_rule_path` asserts the rule path; cascade-path perf is asserted in integration tests via a mocked AI Gateway response.

22. **MUST** ship the CLI subcommand `cyberos-cuo supervisor route --query "<text>" [--persona <key>] [--invoke|--no-invoke] [--record|--no-record] [--json]`. Exit codes (from `cyberos-cli-exit` shared crate per AUTHORING.md rule 9): 0 success-invoked, 1 success-but-ask, 2 success-but-defer, 64 invalid-argument, 65 invalid-data (unknown persona), 73 cant-create (audit failure), 75 temp-fail (timeout), 77 permission-denied (persona mismatch).

23. **MUST** validate the LLM cascade output against the `LlmRoutingPick` Pydantic schema. The schema requires `skill_name: str` (must match a known catalog skill), `arguments: dict` (must be JSON-serialisable), `rationale: str` (1–500 chars), `confidence: float` (0.0–1.0). Validation failure → re-emit prompt once with stricter instructions; second failure → fall through to `ask`. The retry counter is in state; max 1 retry.

24. **MUST** PII-scrub the `query` field of the BRAIN audit row using `cyberos-brain-pii` rules per AUTHORING.md rule 18, BEFORE chain commit. The original query is retained in the supervisor's OTel span (transient, < 30-day retention via FR-OBS-006 tail sampling); the BRAIN row holds the scrubbed form for long-term storage.

25. **MUST** support **multi-step chain entry stub**: the supervisor returns `{decision, next_step: null}` at slice 2. FR-CUO-104 (topological chain walk, slice 3+) consumes `next_step` to compose multi-skill flows. The slice-2 stub MUST set `next_step = null` unconditionally — never a stale value.

26. **MUST** emit `cuo.routing_decision` row carrying `next_step: null` at slice 2; the field is **PRESENT but null** (not absent). This contract guarantees forward-compatibility with FR-CUO-104's chain-walk consumer; absent-field would force FR-CUO-104 to handle both shapes.

---

## §2 — Why this design (rationale for humans)

**Why LangGraph and not a hand-rolled state machine (§1 #1, DEC-160)?** LangGraph gives us four things that a hand-rolled state machine would have to re-invent: (1) durable checkpointing — the state is persistable per node entry, satisfying EU AI Act Art. 12's "replay every decision" requirement; (2) conditional edges with a typed router — the confidence-band branch is one declaration, not 5 if/elif arms scattered through code; (3) a standard idiom — engineers familiar with LangGraph from agentic workflows can read the supervisor in a day; (4) instrumentation hooks — every node entry/exit is observable without wrapping each function manually. The cost is a Python dependency on `langgraph` (0.2.x); the benefit is that the supervisor's topology is the source of truth, not buried in code.

**Why LiteLLM as the LLM abstraction and not direct SDKs (§1 #11, DEC-161)?** LiteLLM provides a unified interface across providers (OpenAI, Anthropic, Bedrock) — but we don't want CUO making direct provider calls because (a) the AI Gateway FR-AI-008 already provides multi-provider routing, failover, and cost ledger; (b) direct CUO→provider calls bypass the cost ledger, breaking budget accounting; (c) two competing routing systems is a maintenance nightmare. The solution: the `litellm_proxy` module is shaped like the LiteLLM API but forwards every call to the AI Gateway. CUO benefits from LiteLLM's prompt/response normalisation; the AI Gateway remains the single egress point for provider calls. The architectural test (`test_supervisor_litellm_routes_via_gateway`) AST-walks the supervisor package and rejects any direct provider import — this is enforced at CI, not by reviewer vigilance.

**Why the confidence bands ≥0.70 / 0.50–0.70 / 0.10–0.50 / <0.10 (§1 #5, DEC-162)?** These come from the website docs §architecture and reflect a year of routing experiments. 0.70 is "confident enough to act without asking" — the rule scorer's top fixture matches all score above 0.70. The 0.50–0.70 band is "good guess but not certain" — surfacing alternatives is cheap and lets the operator confirm. The 0.10–0.50 band is the "ambiguous tail" where LLM cascade adds value — below 0.10, the candidates are noise and the LLM would hallucinate; the right action is defer-to-human. The numeric thresholds are hard-coded constants (not env-tuned) for replay determinism — Phase 2 ships them as the published contract, and tuning happens in subsequent FRs with explicit version bumps.

**Why a hard 3-second LLM cascade budget (§1 #6, DEC-169)?** User-facing latency budget for an ambiguous query is ~5 seconds total (perceived as "thinking, not broken"). 3 seconds for the LLM, 500 ms for supervisor overhead, 500 ms for skill invocation, 500 ms slack. Past 3 seconds the LLM is rarely going to produce a meaningfully better pick; the failure mode (timeout → fall through to `ask`) is benign — the user sees alternatives and picks. Better to fail fast and clearly than spin indefinitely and confuse the user.

**Why structured Pydantic output and never freeform text (§1 #6, DEC-163)?** Freeform LLM responses force regex parsing → high failure rate, security concerns (prompt injection in the rationale text could be acted upon). Structured output via Pydantic schemas: (a) the LLM is constrained at prompt time ("respond with JSON matching schema X"); (b) parsing failure is unambiguous (schema validation error); (c) the `rationale` field is bounded (1–500 chars), preventing prompt-injection-via-rationale. If the LLM produces non-conforming output, fall through to `ask` — never silently invoke based on unparseable text.

**Why the defer-to-human matrix is intrinsic to the persona (§1 #19, DEC-164)?** The EU AI Act Art. 26 human-oversight guarantee says "operators retain the right to refuse AI-initiated actions on high-risk operations." Encoding the matrix in `persona.py` as data (`cfo.defer_to_human_matrix = ["invoice_emit", "wire_transfer", ...]`) and never letting config override is the difference between "we are compliant" and "we claim to be compliant but config can disable it." The principle: matrix is code, not configuration. The cost is one Python file edit per matrix change (ADR-required); the benefit is that the gate cannot be turned off via env vars or runtime knobs.

**Why every decision emits a `cuo.routing_decision` BRAIN row (§1 #8, DEC-165)?** EU AI Act Art. 12 logging requires every AI decision retained 6 months minimum AND replayable. The BRAIN audit chain (Layer-1 memory per AGENTS.md §6) provides both: rows are append-only, chained via SHA-256 prev_chain, and PII-scrubbed before commit. Emitting on EVERY path (auto, ask, cascade, defer) — not just successful invocations — is the design assertion that "defer" is itself an AI decision worth auditing (the AI decided NOT to act). The row's `path_taken` enum lets analysts query "show me all decisions where cascade was triggered but fell through to ask."

**Why the 11-persona catalogue (§1 #9)?** Genie is the unspecialised entry point — the persona that handles general queries before a specialist takes over. The 10 C-level personas (CEO, COO, CFO, CMO, CTO, CHRO, CSO, CLO, CDO, CPO) match CyberSkill's organisational structure and the BACKLOG's role-RBAC catalogue. Each persona has its own keyword bank (CFO's bank includes "invoice", "VAT", "BHXH", "P1/P2/P3"; CTO's includes "deploy", "rollback", "incident") so the rule scorer can adjust weights per persona without bloating one giant keyword list. The catalogue is closed — adding an 11th specialist persona is an ADR, same discipline as RBAC.

**Why route LLM calls THROUGH the AI Gateway and not direct (DEC-161)?** Three reasons converge: (1) the AI Gateway is the cost-ledger authority — every LLM call is preflight-budgeted via FR-AI-001 and post-call reconciled via FR-AI-002; bypassing it breaks budget accounting. (2) the AI Gateway is the residency-pinning authority (FR-AI-016) — direct CUO→provider calls would not honour tenant residency policy. (3) the AI Gateway is the failover authority (FR-AI-008) — implementing failover in CUO too would duplicate logic and create drift. The `litellm_proxy` module solves the LiteLLM-style ergonomics on the CUO side while preserving the AI Gateway as single egress.

**Why slice-2 ships only the in-memory checkpointer (§1 #12)?** Splitting the Postgres-backed checkpointer to FR-CUO-102 keeps this FR focused on the supervisor topology + LLM cascade. The in-memory checkpointer is functionally complete for tests; production deployment of slice 2 runs without persistent checkpointing (state lost on supervisor restart — acceptable because each request is independent). FR-CUO-102 adds Postgres persistence for replay + EU AI Act Art. 12 full compliance.

**Why `cuo_state_v` field in state and BRAIN row (§1 #12, DEC-167)?** Slice 3+ will add fields to the state (multi-step `next_step`, chain context, etc.). Embedded version lets the supervisor reject state replays from incompatible versions cleanly — and lets the BRAIN row's analyst tooling filter by state-schema version. ±2 version tolerance (current is 1; tolerate 1–3) allows rolling upgrades without breaking replay during deployment windows.

**Why 5 nodes exactly and no more (§1 #1, DEC-168)?** Graph topology is a contract. New nodes (e.g. "post-process LLM response" or "rate-limit check") add hidden state transitions that consumers can't reason about. Five nodes is the minimum complete set: parse → score → decide → act → record. Anything else is a refactor — and a refactor of the supervisor is a new FR, not a code change.

**Why no fallback to direct provider on AI Gateway failure (§1 #11)?** AI Gateway failure should be a sev-1 alarm, not silently routed-around. If CUO falls back to direct provider on gateway failure, the cost ledger and residency pinning are silently bypassed — and operators never see the gateway outage because CUO masks it. The right behaviour: cascade fails → fall through to `ask` (no LLM input but user sees alternatives); the OTel `cuo_supervisor_llm_cascade_total{outcome=gateway_error}` counter triggers an alarm.

**Why `next_step: null` field explicitly present (§1 #25, §1 #26)?** FR-CUO-104 ships multi-step chain walks; the consumer reads `decision.next_step` to plan the next supervisor invocation. If slice 2 omits the field, FR-CUO-104 must handle both shapes (field-present + field-absent), which is two code paths to test forever. Setting it explicitly to `null` makes the contract one-shape — forward-compatible without ambiguity.

**Why persona JWT validation at every state entry (§1 #10, DEC-166)?** The persona claim is the "who is acting" identity. Validating it at supervisor entry prevents a `genie` JWT from triggering a `cfo`-persona routing path — which would bypass the CFO's defer-to-human matrix. The check is at the parse node; missing/mismatched persona → fail fast at entry, never reach the scorer.

**Why the LLM cascade emits a `cuo.llm_cascade_timeout` row on timeout (§1 #6)?** Timeouts are AI decisions ("AI failed to provide structured pick within budget"). The BRAIN row preserves the fact that an LLM was consulted and timed out; useful for SRE analysis ("which queries consistently timeout") and EU AI Act Art. 12 ("show me decisions where LLM was attempted but produced no usable output"). The cost of the row is ~700 bytes; the benefit is full audit reconstruction.

---

## §3 — API contract

### 3.1 — CuoState schema

```python
# cuo/cuo/supervisor/state.py
from typing import TypedDict, Literal, NotRequired
from pydantic import BaseModel

CUO_STATE_V = 1
CASCADE_THRESHOLD_LOW = 0.10
CASCADE_THRESHOLD_HIGH = 0.50
ASK_THRESHOLD = 0.70

PathTaken = Literal["auto", "ask", "cascade", "defer", "cascade_then_auto", "cascade_then_ask"]

class Candidate(BaseModel):
    skill_name: str
    confidence: float            # 0.0 – 1.0
    arguments: dict
    score_components: dict       # name_match, keyword_hits, region_bonus, etc.

class LlmRoutingPick(BaseModel):
    skill_name: str
    arguments: dict
    rationale: str               # 1–500 chars
    confidence: float

class InvocationResult(BaseModel):
    skill_name: str
    exit_code: int
    stdout: str
    stderr: str
    duration_ms: float

class TransparencyDisclosure(BaseModel):
    skill_chosen: str | None
    confidence: float
    alternatives: list[Candidate]
    path_taken: PathTaken
    llm_used: bool

class CuoState(TypedDict):
    # — request envelope —
    query: str
    tenant_id: str
    subject_id: str
    persona_key: str
    request_id: str
    ts_ns_start: int
    cuo_state_v: int             # always CUO_STATE_V; for replay-version-checks

    # — scoring + routing —
    rule_scores: list[Candidate]
    branch_decision: PathTaken
    cascade_taken: bool          # set true after first cascade entry; prevents re-entry

    # — LLM cascade —
    llm_pick: NotRequired[LlmRoutingPick]
    llm_attempts: int            # max 2 (first + 1 retry); see §1 #23
    llm_started_at: NotRequired[float]

    # — invocation —
    invocation_result: NotRequired[InvocationResult]

    # — output —
    transparency: NotRequired[TransparencyDisclosure]
    next_step: None              # slice 2 always null; FR-CUO-104 fills

    # — bookkeeping —
    ts_ns_end: NotRequired[int]
    audit_emitted: bool
```

### 3.2 — Graph construction

```python
# cuo/cuo/supervisor/graph.py
from langgraph.graph import StateGraph, END
from cyberos.cuo.supervisor.state import CuoState, PathTaken
from cyberos.cuo.supervisor.nodes import parse, rule_score, branch, llm_cascade, invoke, record

def build_supervisor_graph() -> StateGraph:
    g = StateGraph(CuoState)

    # Node registration — exactly 5 user-visible nodes (parse, rule_score, branch, invoke, record).
    # llm_cascade is reached via conditional edge from `branch`; it re-enters `branch`.
    g.add_node("parse", parse.parse_node)
    g.add_node("rule_score", rule_score.rule_score_node)
    g.add_node("branch", branch.branch_node)
    g.add_node("llm_cascade", llm_cascade.cascade_node)
    g.add_node("invoke", invoke.invoke_node)
    g.add_node("record", record.record_node)

    g.set_entry_point("parse")
    g.add_edge("parse", "rule_score")
    g.add_edge("rule_score", "branch")

    # Conditional edges from `branch` based on path_taken.
    def route_from_branch(state: CuoState) -> str:
        pt: PathTaken = state["branch_decision"]
        if pt == "auto" or pt == "cascade_then_auto":
            return "invoke"
        if pt == "cascade":
            return "llm_cascade"
        if pt in {"ask", "defer", "cascade_then_ask"}:
            return "record"
        raise ValueError(f"unknown path_taken: {pt}")

    g.add_conditional_edges("branch", route_from_branch, {
        "invoke": "invoke",
        "llm_cascade": "llm_cascade",
        "record": "record",
    })
    g.add_edge("llm_cascade", "branch")  # cascade re-enters branch with LLM confidence
    g.add_edge("invoke", "record")
    g.add_edge("record", END)

    return g.compile(checkpointer=InMemoryCheckpointer())  # Postgres in FR-CUO-102
```

### 3.3 — Branch node logic

```python
# cuo/cuo/supervisor/nodes/branch.py
from cyberos.cuo.supervisor.state import (
    CuoState, PathTaken, CASCADE_THRESHOLD_LOW, CASCADE_THRESHOLD_HIGH, ASK_THRESHOLD
)

def branch_node(state: CuoState) -> CuoState:
    top = state["rule_scores"][0] if state["rule_scores"] else None
    cascade_taken = state.get("cascade_taken", False)

    # If LLM cascade returned a pick, use its confidence for branching.
    if cascade_taken and state.get("llm_pick"):
        conf = state["llm_pick"].confidence
        if conf >= ASK_THRESHOLD:
            state["branch_decision"] = "cascade_then_auto"
        else:
            state["branch_decision"] = "cascade_then_ask"
        return state

    if top is None or top.confidence < CASCADE_THRESHOLD_LOW:
        state["branch_decision"] = "defer"
    elif top.confidence >= ASK_THRESHOLD:
        state["branch_decision"] = "auto"
    elif top.confidence >= CASCADE_THRESHOLD_HIGH:
        state["branch_decision"] = "ask"
    else:  # 0.10 ≤ conf < 0.50
        if cascade_taken:
            state["branch_decision"] = "ask"  # already cascaded once; never twice
        else:
            state["branch_decision"] = "cascade"
    return state
```

### 3.4 — LLM cascade node

```python
# cuo/cuo/supervisor/nodes/llm_cascade.py
import asyncio
import time
from pydantic import ValidationError
from cyberos.cuo.supervisor.state import CuoState, LlmRoutingPick
from cyberos.cuo.supervisor.litellm_proxy import litellm_call_via_gateway
from cyberos.cuo.supervisor.persona import get_persona
from cyberos.cuo.supervisor.audit import emit_cascade_timeout_row

LLM_BUDGET_SECONDS = 3.0
MAX_LLM_ATTEMPTS = 2

async def cascade_node(state: CuoState) -> CuoState:
    state["cascade_taken"] = True
    state["llm_started_at"] = time.monotonic()
    persona = get_persona(state["persona_key"])
    prompt = build_cascade_prompt(state, persona)

    for attempt in range(1, MAX_LLM_ATTEMPTS + 1):
        state["llm_attempts"] = attempt
        try:
            response_text = await asyncio.wait_for(
                litellm_call_via_gateway(prompt, tenant_id=state["tenant_id"]),
                timeout=LLM_BUDGET_SECONDS,
            )
        except asyncio.TimeoutError:
            emit_cascade_timeout_row(state)
            # Fall through to ask (do not set llm_pick).
            return state

        try:
            pick = LlmRoutingPick.model_validate_json(response_text)
        except ValidationError:
            if attempt < MAX_LLM_ATTEMPTS:
                prompt = strengthen_prompt(prompt)
                continue
            # Final retry failed — fall through to ask.
            return state

        if pick.skill_name not in state["rule_scores_known_names"]:
            # LLM hallucinated a skill not in catalog — reject.
            if attempt < MAX_LLM_ATTEMPTS:
                prompt = strengthen_prompt(prompt)
                continue
            return state

        state["llm_pick"] = pick
        return state
    return state
```

### 3.5 — LiteLLM proxy

```python
# cuo/cuo/supervisor/litellm_proxy.py
"""LiteLLM-shaped client that ALWAYS routes through AI Gateway FR-AI-008.

Direct provider SDKs (boto3, anthropic, openai) MUST NOT be imported in this package.
The architectural test `test_supervisor_litellm_routes_via_gateway` AST-walks the
package and rejects forbidden imports.
"""
import httpx
import os

AI_GATEWAY_URL = os.environ.get("CYBEROS_AI_GATEWAY_URL", "http://localhost:8080")

async def litellm_call_via_gateway(prompt: str, tenant_id: str) -> str:
    """Send a chat request to the AI Gateway and return the model's response text.

    Mirrors LiteLLM's `litellm.acompletion()` shape so prompts can be authored
    in the LiteLLM ecosystem and transparently routed through the gateway.
    """
    async with httpx.AsyncClient(timeout=30.0) as client:
        resp = await client.post(
            f"{AI_GATEWAY_URL}/v1/ai/chat",
            json={
                "tenant_id": tenant_id,
                "model": "haiku@cuo-supervisor",   # alias resolved by FR-AI-006
                "messages": [{"role": "user", "content": prompt}],
                "response_format": {"type": "json_object"},
                "temperature": 0.0,                  # deterministic for replay
                "max_tokens": 500,
            },
            headers={"X-Cuo-Decision-Id": _new_decision_id()},
        )
        resp.raise_for_status()
        return resp.json()["choices"][0]["message"]["content"]
```

### 3.6 — Persona catalogue

```python
# cuo/cuo/supervisor/persona.py
from pydantic import BaseModel

class Persona(BaseModel):
    key: str
    display_name: str
    keyword_bank: list[str]
    system_prompt: str
    defer_to_human_matrix: list[str]  # operation types this persona refuses to auto-invoke

PERSONA_CATALOGUE = {
    "genie": Persona(
        key="genie",
        display_name="Genie",
        keyword_bank=[],  # base; no specialised keywords
        system_prompt="You are Genie — the unspecialised CUO entry point. Route to a specialist persona when query intent is clear.",
        defer_to_human_matrix=["wire_transfer", "subject_role_grant_reserved"],
    ),
    "cfo": Persona(
        key="cfo",
        display_name="CFO",
        keyword_bank=["invoice", "vat", "hoa don", "bhxh", "p1", "p2", "p3", "payroll", "approve"],
        system_prompt="You are the CFO persona. Refuse to auto-execute disbursements; require human sign-off.",
        defer_to_human_matrix=["wire_transfer", "invoice_emit", "payroll_commit", "esop_grant_signoff", "po_cancellation"],
    ),
    "cto": Persona(
        key="cto",
        display_name="CTO",
        keyword_bank=["deploy", "rollback", "incident", "p0", "p1", "tech-debt", "security"],
        system_prompt="You are the CTO persona. Refuse to auto-execute production deploys or rollbacks.",
        defer_to_human_matrix=["prod_deploy", "prod_rollback", "key_rotation"],
    ),
    # ... 8 more personas (CEO, COO, CMO, CHRO, CSO, CLO, CDO, CPO) ...
}

def get_persona(key: str) -> Persona:
    if key not in PERSONA_CATALOGUE:
        raise ValueError(f"unknown_persona: {key}")
    return PERSONA_CATALOGUE[key]
```

### 3.7 — Audit row builder

```python
# cuo/cuo/supervisor/audit.py
import hashlib
from cyberos.cuo.supervisor.state import CuoState

def build_routing_decision_row(state: CuoState) -> dict:
    """Build the canonical `cuo.routing_decision` BRAIN audit row.

    Emitted by the `record` node on EVERY path (auto, ask, cascade, defer).
    """
    pii_scrubbed_query = apply_brain_pii_rules(state["query"])  # FR-BRAIN-111
    return {
        "kind": "cuo.routing_decision",
        "tenant_id": state["tenant_id"],
        "subject_id_hash16": hashlib.sha256(state["subject_id"].encode()).hexdigest()[:16],
        "persona_key": state["persona_key"],
        "persona_version": state.get("persona_version"),
        "query": pii_scrubbed_query,
        "rule_scores": [c.model_dump() for c in state["rule_scores"][:3]],
        "path_taken": state["branch_decision"],
        "llm_pick": state["llm_pick"].model_dump() if state.get("llm_pick") else None,
        "invocation_result": state["invocation_result"].model_dump() if state.get("invocation_result") else None,
        "cuo_state_v": state["cuo_state_v"],
        "next_step": None,    # slice 2 contract; FR-CUO-104 fills
        "request_id": state["request_id"],
        "trace_id": current_trace_id_hex(),
        "ts_ns_start": state["ts_ns_start"],
        "ts_ns_end": state.get("ts_ns_end"),
    }
```

### 3.8 — CLI entry point

```python
# cuo/cuo/cli/supervisor.py
import asyncio
import click
import json
from cyberos.cuo.supervisor import run_supervisor

@click.group()
def supervisor():
    """CUO Phase 2 supervisor commands."""

@supervisor.command()
@click.option("--query", required=True, help="Natural-language request")
@click.option("--persona", default="genie")
@click.option("--invoke/--no-invoke", default=True)
@click.option("--record/--no-record", default=True)
@click.option("--json/--no-json", "json_out", default=False)
def route(query: str, persona: str, invoke: bool, record: bool, json_out: bool):
    """Route a query through the supervisor."""
    result = asyncio.run(run_supervisor(query=query, persona_key=persona, do_invoke=invoke, do_record=record))
    if json_out:
        click.echo(json.dumps(result.model_dump(), ensure_ascii=False, indent=2))
    else:
        click.echo(result.transparency.summary_line())
    exit_code = {
        "auto": 0, "cascade_then_auto": 0,
        "ask": 1, "cascade_then_ask": 1,
        "defer": 2,
    }.get(result.path_taken, 65)
    raise SystemExit(exit_code)
```

---

## §4 — Acceptance criteria

1. **Graph topology fixed at 5 nodes** — `test_supervisor_graph::test_five_nodes_only` asserts `len(graph.nodes) == 5` with names `{parse, rule_score, branch, llm_cascade, invoke, record}` minus `llm_cascade` (auxiliary). User-visible decision points are 5.
2. **Rule path (≥ 0.70)** — query "validate MST 0301479073" with cfo persona → score 0.85 → auto-invoke → `path_taken = "auto"` → exit 0.
3. **Ask path (0.50–0.70)** — query "send invoice" → score 0.62 → return alternatives top 3 → `path_taken = "ask"` → exit 1.
4. **Cascade path (0.10–0.50)** — query "what should we do about the unpaid Q4 contract" → rule score 0.30 → LLM cascade → LLM returns `{skill_name: "inv-dunning-draft", confidence: 0.85}` → `path_taken = "cascade_then_auto"` → exit 0.
5. **Defer path (< 0.10)** — query "asdfghjkl" → rule score 0.03 → `path_taken = "defer"` → exit 2.
6. **Cascade fall-through to ask** — LLM returns confidence 0.40 → `path_taken = "cascade_then_ask"` → exit 1.
7. **No cascade re-entry** — query in cascade band; first cascade returns 0.20; `cascade_taken = true` blocks re-entry → falls through to `ask`.
8. **LLM timeout** — mocked AI Gateway delays 5s; supervisor falls through to `ask`; `cuo.llm_cascade_timeout` BRAIN row emitted.
9. **LLM freeform rejected** — mocked AI Gateway returns "I think you should..." → Pydantic validation fails twice → fall through to `ask`.
10. **LLM hallucinated skill rejected** — LLM returns `{skill_name: "make-up-skill"}` not in catalog → reject → retry → fall through to `ask`.
11. **Persona matrix blocks auto-invoke** — query routes to `vn-vat-invoice@1.2` with cfo persona at confidence 0.95; cfo.defer_to_human_matrix contains `invoice_emit` → refuse auto, return `{routed: false, reason: "persona_defer_matrix"}`; `cuo.persona_defer_block` BRAIN row.
12. **Destructive skill bypass blocked** — skill annotated `destructive: true` at confidence 1.0 → capability broker FR-SKILL-104 gates via Elicitation; supervisor's invoke node refuses to override.
13. **Persona JWT mismatch** — caller JWT has `agent_persona: cuo-cto@0.1.0` but requests `cfo` persona → 403 with `persona_mismatch`.
14. **Unknown persona** — `--persona foo` → exit 65, `cuo.persona_unknown` BRAIN row.
15. **Direct provider import forbidden** — `test_supervisor_litellm_routes_via_gateway` AST-walks `cuo/cuo/supervisor/` for `import boto3 | import anthropic | import openai | from openai`; finding any → test fails.
16. **Cuo state version present** — state contains `cuo_state_v == 1`; replay with `cuo_state_v == 4` rejected with `state_version_unsupported`.
17. **Audit row emitted every path** — assert exactly one `cuo.routing_decision` row after `route` for each of {auto, ask, cascade, defer}.
18. **Audit row carries `next_step: null` explicitly** — JSON serialised row has `"next_step": null` (key present), not absent.
19. **Audit row PII-scrubbed query** — query containing email/phone is scrubbed in the BRAIN row but preserved in OTel span (transient).
20. **Replay equivalence on rule path** — 15-golden-fixture suite: run twice on identical catalog → byte-identical decision JSONs.
21. **Rule path < 50ms p95** — `test_supervisor_perf_rule_path` 1000 iterations; p95 < 50 ms.
22. **Cascade path < 3500ms p95** — integration test with mocked AI Gateway returning at 2500 ms; p95 < 3500 ms.
23. **OTel span `cuo.supervisor.route` emitted with all required attributes** — `tenant_id`, `subject_id_hash16`, `persona_key`, `path_taken`, `confidence`, `llm_used`, `outcome`.
24. **OTel child spans per node** — assert spans `cuo.supervisor.node.{parse,rule_score,branch,llm_cascade,invoke,record}` present in trace tree.
25. **Counter `cuo_supervisor_route_total{outcome=success, persona=cfo, path_taken=auto}` increments** — every successful route bumps it.
26. **Counter `cuo_supervisor_persona_defer_blocks_total{persona=cfo, operation=invoice_emit}` increments** — every defer-matrix block bumps it.
27. **CLI exit codes correct** — auto→0, ask→1, defer→2, unknown_persona→65, persona_mismatch→77, timeout→75.
28. **Dry-run mode** — `--no-invoke --no-record` returns decision without invocation and without BRAIN row; OTel span `cuo.dry_run` emitted instead.
29. **Transparency disclosure present** — every response includes `transparency: {skill_chosen, confidence, alternatives, path_taken, llm_used}`.
30. **EU AI Act Art. 12 replay** — given a stored BRAIN row + catalog snapshot at the row's timestamp, running the supervisor produces an identical `path_taken` and `llm_pick`.
31. **Caller `traceparent` propagated to AI Gateway** — integration test: outbound request to gateway carries the same `traceparent` as the inbound caller request.

---

## §5 — Verification

```python
# cuo/tests/test_supervisor_graph.py
from cyberos.cuo.supervisor.graph import build_supervisor_graph

def test_five_nodes_only():
    g = build_supervisor_graph()
    node_names = set(g.nodes.keys())
    assert node_names == {"parse", "rule_score", "branch", "llm_cascade", "invoke", "record"}

def test_entry_point_is_parse():
    g = build_supervisor_graph()
    assert g.entry_point == "parse"
```

```python
# cuo/tests/test_supervisor_litellm_routes_via_gateway.py
import ast
from pathlib import Path

SUPERVISOR_DIR = Path("cuo/cuo/supervisor")
FORBIDDEN_IMPORTS = {"boto3", "anthropic", "openai", "google.cloud.aiplatform"}

def _collect_imports(tree: ast.AST) -> set[str]:
    out = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names: out.add(alias.name.split(".")[0])
        elif isinstance(node, ast.ImportFrom):
            if node.module: out.add(node.module.split(".")[0])
    return out

def test_no_direct_provider_imports_in_supervisor():
    found = set()
    for py in SUPERVISOR_DIR.rglob("*.py"):
        tree = ast.parse(py.read_text())
        found |= _collect_imports(tree)
    leakage = found & FORBIDDEN_IMPORTS
    assert not leakage, f"supervisor imports forbidden providers: {leakage}"
```

```python
# cuo/tests/test_supervisor_persona_defer_matrix.py
import pytest
from cyberos.cuo.supervisor import run_supervisor

@pytest.mark.asyncio
async def test_cfo_persona_blocks_invoice_emit_even_at_high_confidence(mock_catalog):
    mock_catalog.set_score("vn-vat-invoice@1.2", 0.95)  # high confidence
    result = await run_supervisor(
        query="emit hoa don for ACME 4M VND",
        persona_key="cfo",
        do_invoke=True, do_record=True,
    )
    assert result.routed is False
    assert result.reason == "persona_defer_matrix"
    assert result.operation == "invoice_emit"
    # BRAIN row was emitted
    rows = brain_audit_rows_for_request(result.request_id)
    assert any(r["kind"] == "cuo.persona_defer_block" for r in rows)
```

```python
# cuo/tests/test_supervisor_audit_row.py
import pytest
from cyberos.cuo.supervisor import run_supervisor

@pytest.mark.parametrize("query, expected_path", [
    ("validate MST 0301479073",            "auto"),
    ("send invoice",                       "ask"),
    ("what should we do about Q4 churn",   "cascade_then_auto"),
    ("asdfghjkl",                          "defer"),
])
@pytest.mark.asyncio
async def test_audit_row_emitted_on_every_path(query, expected_path, mock_catalog, brain_rows):
    pre = len(brain_rows.by_kind("cuo.routing_decision"))
    await run_supervisor(query=query, persona_key="genie")
    post = brain_rows.by_kind("cuo.routing_decision")
    assert len(post) == pre + 1
    assert post[-1]["path_taken"] == expected_path
    assert "next_step" in post[-1] and post[-1]["next_step"] is None   # AC #18
```

```python
# cuo/tests/test_supervisor_idempotency.py
import json
import pytest
from cyberos.cuo.supervisor import run_supervisor

@pytest.mark.asyncio
async def test_replay_equivalence_on_15_golden_fixtures():
    fixtures = load_golden_fixtures("cuo/tests/fixtures/golden_queries.jsonl")
    decisions = []
    for query in fixtures:
        r = await run_supervisor(query=query, persona_key="genie", do_invoke=False, do_record=False)
        decisions.append(r.model_dump_json(exclude={"request_id", "ts_ns_start", "ts_ns_end"}))
    # Replay
    decisions_replay = []
    for query in fixtures:
        r = await run_supervisor(query=query, persona_key="genie", do_invoke=False, do_record=False)
        decisions_replay.append(r.model_dump_json(exclude={"request_id", "ts_ns_start", "ts_ns_end"}))
    assert decisions == decisions_replay, "replay produced different decisions"
```

```python
# cuo/tests/test_supervisor_perf_rule_path.py
import asyncio
import time
import pytest
from cyberos.cuo.supervisor import run_supervisor

@pytest.mark.asyncio
async def test_rule_path_under_50ms_p95():
    latencies = []
    for _ in range(1000):
        t0 = time.monotonic()
        await run_supervisor(query="validate MST 0301479073", persona_key="cfo", do_invoke=False, do_record=False)
        latencies.append((time.monotonic() - t0) * 1000.0)
    latencies.sort()
    p95 = latencies[int(len(latencies) * 0.95)]
    assert p95 < 50.0, f"rule path p95 = {p95:.1f}ms > 50ms budget"
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton. The remaining 8 persona definitions in §3.6 — CEO/COO/CMO/CHRO/CSO/CLO/CDO/CPO — are filled in during implementation following the same shape as cfo/cto.)

---

## §7 — Dependencies

**Upstream (this FR depends on):**
- **FR-AI-008** — multi-provider router; `litellm_proxy` forwards every cascade LLM call to the AI Gateway's `/v1/ai/chat` endpoint.

**Downstream (this FR blocks — all 14 are placeholders / not yet specified):**
- **FR-CUO-102** — Postgres checkpointer; replaces this FR's in-memory checkpointer scaffold.
- **FR-CUO-103** — Phase 2 trace rows with prompt + model + temperature + seed for replay.
- **FR-CUO-104** — topological walk of `depends_on` chain; consumes this FR's `next_step` field.
- **FR-CRM-005, FR-CRM-006, FR-CRM-007** — CRM-side CUO skills (next-action, lead scoring, win/loss analysis).
- **FR-DOC-009** — renewal proposal CUO draft.
- **FR-EMAIL-008** — Genie subject-prefix routing.
- **FR-INV-010** — dunning draft CUO.
- **FR-OKR-006, FR-OKR-007** — Monday digest + quarterly retro CUO drafts.
- **FR-PROJ-011, FR-PROJ-012** — blocker notification + cycle-review draft (already specced; they consume CUO via webhook).
- **FR-RES-004** — hiring memo CUO draft.

**Cross-module (informational):**
- **FR-AI-014** — persona-version stamping; `cuo-<persona-key>@<semver>` is validated at parse node.
- **FR-AUTH-101** — RBAC catalogue; `agent-persona` role must be present in claims.
- **FR-AI-003** — BRAIN audit bridge; receives `cuo.routing_decision`, `cuo.llm_cascade_timeout`, `cuo.persona_defer_block`, `cuo.dry_run`, `cuo.persona_unknown`.
- **FR-SKILL-104** — capability broker; gates destructive-skill invocation regardless of supervisor decision.
- **FR-BRAIN-111** — PII detection; applied to `query` field before chain commit.
- **FR-AI-022** — OTel trace emission; supervisor spans correlate via `traceparent`.

---

## §8 — Example payloads

### 8.1 — CLI invocation (rule path)

```bash
$ cyberos-cuo supervisor route --query "validate MST 0301479073" --persona cfo --json
{
  "routed": true,
  "skill_chosen": "vn-mst-validate@1.0",
  "confidence": 0.92,
  "path_taken": "auto",
  "llm_used": false,
  "alternatives": [
    {"skill": "vn-bank-transfer@1.0", "confidence": 0.18},
    {"skill": "crm-account-create@1.0", "confidence": 0.12}
  ],
  "invocation_result": {
    "skill_name": "vn-mst-validate@1.0",
    "exit_code": 0,
    "stdout": "{\"mst\":\"0301479073\",\"valid\":true,\"company\":\"ACME JSC\"}",
    "stderr": "",
    "duration_ms": 14.2
  },
  "next_step": null,
  "transparency": {
    "skill_chosen": "vn-mst-validate@1.0",
    "confidence": 0.92,
    "path_taken": "auto",
    "llm_used": false
  }
}
```

### 8.2 — Cascade-then-auto decision

```bash
$ cyberos-cuo supervisor route --query "what should we do about the unpaid Q4 contract from ACME" --persona genie --json
{
  "routed": true,
  "skill_chosen": "inv-dunning-draft@1.2",
  "confidence": 0.83,
  "path_taken": "cascade_then_auto",
  "llm_used": true,
  "llm_pick": {
    "skill_name": "inv-dunning-draft@1.2",
    "rationale": "Query mentions an unpaid contract; dunning draft is the canonical response.",
    "confidence": 0.83
  },
  "alternatives": [
    {"skill": "crm-deal-update@1.0", "confidence": 0.31},
    {"skill": "email-genie-draft@1.0", "confidence": 0.22}
  ],
  "invocation_result": { /* ... */ },
  "next_step": null
}
```

### 8.3 — Defer decision (no LLM consulted)

```bash
$ cyberos-cuo supervisor route --query "asdfghjkl" --persona genie --json
{
  "routed": false,
  "path_taken": "defer",
  "llm_used": false,
  "alternatives": [],
  "next_step": null,
  "transparency": {
    "skill_chosen": null,
    "confidence": 0.03,
    "path_taken": "defer",
    "llm_used": false
  }
}
```

### 8.4 — `cuo.routing_decision` BRAIN audit row

```json
{
  "kind": "cuo.routing_decision",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "persona_key": "cfo",
  "persona_version": "cuo-cfo@0.4.1",
  "query": "validate MST [REDACTED-MST]",
  "rule_scores": [
    {"skill_name": "vn-mst-validate@1.0", "confidence": 0.92, "arguments": {"mst": "[REDACTED]"}, "score_components": {"name_match": 5.0, "keyword_hits": 3.0, "region_bonus": 2.0}}
  ],
  "path_taken": "auto",
  "llm_pick": null,
  "invocation_result": {"skill_name": "vn-mst-validate@1.0", "exit_code": 0, "duration_ms": 14.2},
  "cuo_state_v": 1,
  "next_step": null,
  "request_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "ts_ns_start": 1747920731000000000,
  "ts_ns_end": 1747920731017200000
}
```

### 8.5 — `cuo.persona_defer_block` row

```json
{
  "kind": "cuo.persona_defer_block",
  "tenant_id": "5e8f1d2a-...",
  "persona_key": "cfo",
  "operation": "invoice_emit",
  "rule_top_skill": "vn-vat-invoice@1.2",
  "rule_top_confidence": 0.95,
  "request_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "ts_ns": 1747920731000000000
}
```

### 8.6 — `cuo.llm_cascade_timeout` row

```json
{
  "kind": "cuo.llm_cascade_timeout",
  "tenant_id": "5e8f1d2a-...",
  "persona_key": "genie",
  "query": "the long ambiguous query that needed an LLM but timed out",
  "elapsed_ms": 3012,
  "attempts": 1,
  "request_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Persona-specific keyword bank tuning** — Phase 4 (slice 4+). Slice 2 ships seed banks; tuning via golden-fixture analysis lands later.
- **Per-tenant confidence threshold override** — slice 3+. Slice 2 hard-codes 0.10/0.50/0.70 for replay determinism.
- **Multi-step chain walk** — FR-CUO-104. Slice 2 ships `next_step: null` stub.
- **Postgres checkpointer** — FR-CUO-102. Slice 2 ships in-memory scaffold.
- **LLM cascade replay** — FR-CUO-103 captures full prompt + seed for Art. 12 replay; slice 2 stores only model + path.
- **Per-persona prompt iteration** — Phase 4 (slice 4+). Slice 2 ships baseline system prompts.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `langgraph` import fails (uninstalled) | Service startup | Service refuses to start; CI catches before deploy | `uv sync` reinstall |
| LangGraph version drift breaks graph compile | `test_supervisor_graph` | CI fails | Pin `langgraph==0.2.*` in pyproject.toml |
| Direct provider import added to supervisor | `test_supervisor_litellm_routes_via_gateway` AST walk | CI fails | Remove direct import; use `litellm_proxy` |
| AI Gateway unreachable | `httpx.ConnectError` in cascade | Fall through to `ask`; emit `cuo.llm_cascade_timeout` with `error=gateway_unreachable` | OBS sev-2; operator restores gateway |
| AI Gateway returns 5xx | `resp.raise_for_status()` | Fall through to `ask` | OBS sev-3; gateway-side investigation |
| LLM returns malformed JSON | Pydantic ValidationError | Retry once with strengthened prompt; if 2nd fails, fall through to `ask` | None — designed path |
| LLM hallucinates skill not in catalog | `skill_name not in known_names` check | Retry once; fall through to `ask` | None — designed path |
| LLM exceeds 3-second budget | `asyncio.wait_for` TimeoutError | Fall through to `ask`; emit timeout row | None — designed path |
| LLM cascade re-entry loop | `cascade_taken` flag check in branch | Forced to `ask` on second visit | None — invariant |
| Persona JWT missing | parse-node validation | 403 `persona_mismatch` | Caller re-auths with persona claim |
| Persona JWT version mismatch (`cuo-cfo@0.3.0` vs catalogue `cuo-cfo@0.4.1`) | parse-node check via FR-AI-014 | 403 with `persona_version_mismatch` | Refresh-token cycle |
| Unknown persona requested (`--persona foo`) | parse-node lookup | exit 65; `cuo.persona_unknown` row | Caller fixes persona key |
| `cuo_state_v` mismatch on replay | checkpointer load | `state_version_unsupported` error | Manually upgrade state via migration tool |
| Destructive skill picked at high confidence | `invoke` node checks `destructive: true` annotation | Capability broker FR-SKILL-104 gates via Elicitation | User confirms via Elicitation flow |
| Persona defer-matrix blocks high-confidence pick | `branch` node consults persona.defer_to_human_matrix | Return `routed: false, reason: persona_defer_matrix` | Operator handles manually |
| BRAIN audit row commit fails | brain_writer error | Supervisor returns 500 `audit_failed`; invocation rolled back if `auto` | OBS sev-1; brain_writer investigation |
| Audit row contains unscrubbed PII | `cyberos-brain-pii` scrubber called pre-commit; CI test asserts | Pre-commit failure | Investigate; add PII rule |
| Concurrent supervisors invoke same destructive skill | Capability broker idempotency check | Second invocation refused via Elicitation idempotency | None — designed |
| Caller's `traceparent` malformed | parse-node validation | Generate fresh trace at trust boundary per AUTHORING.md rule 22 | None — designed |
| Replay-equivalence broken on rule path | `test_supervisor_idempotency` | CI fails | Fix non-determinism (likely a dict iteration order or `time.now()` leak) |
| Rule path latency > 50ms p95 | `test_supervisor_perf_rule_path` | CI fails | Profile + optimise; common culprits: cold-import, JSON serialisation |
| Cascade path latency > 3500ms p95 | Integration test with mocked gateway | CI fails | Verify timeout budget; reduce prompt size |
| 11-persona catalogue divergence between docs + code | `test_persona_catalogue_matches_docs` (asserts 11 keys present) | CI fails | Sync persona.py with cuo.html §personas |
| `defer_to_human_matrix` empty for persona | Persona schema requires ≥ 1 entry for C-level personas | Schema validation fails at module import | Fix persona definition |
| `next_step` field absent (not null) in row | JSON-schema validation in brain_writer | Reject row | Fix audit.py builder |
| OTel span attributes missing | `test_supervisor_otel_attributes` | CI fails | Add missing attribute to span builder |
| Counter cardinality explosion (tenant_id label) | Cardinality budget alarm in OBS | sev-3 | Aggregate by tenant_id only at Grafana query layer |
| Dry-run mode accidentally emits BRAIN row | `test_supervisor_dry_run_no_row` | CI fails | Fix conditional in record node |
| FR-CUO-102 ships Postgres checkpointer but state schema unchanged | Forward-compat by `cuo_state_v` tolerance ±2 | Works without changes | None — designed |
| FR-CUO-104 ships chain walk; this FR's `next_step` always null | Forward-compat by always-null contract | FR-CUO-104 fills the field | None — designed |
| Tenant residency policy denies LLM call | FR-AI-016 returns 451 at gateway | Cascade fails → fall through to `ask`; emit timeout-shaped row with `error=residency_denied` | Operator reviews residency policy |
| Cost ledger insufficient budget | FR-AI-001 precheck rejects at gateway | Cascade fails → fall through to `ask`; emit row with `error=budget_exceeded` | Operator tops up budget or operator overrides |
| Catalog snapshot hash mismatch between request + record | Catch in record node | sev-3 alarm; record row anyway with snapshot mismatch flagged | Investigate catalog reload race |
| State serialisation fails (non-JSON value in arguments) | Pydantic dump | 500 `state_serialise_failed`; no BRAIN row | Fix arguments schema |
| `persona.py` missing a C-level persona | `test_persona_catalogue_complete` | CI fails | Add missing persona |
| Cascade prompt exceeds context window | Pre-prompt token estimator | Truncate from oldest rule_score; warn in span | None — designed |
| Replay across `cuo_state_v` ±3 attempted | `state_version_unsupported` raised | 500 error | Manual state migration tool |

---

## §11 — Implementation notes

- **LangGraph is the orchestration framework, not the LLM** — confusing the two trips up newcomers. LangGraph = state-machine framework with Postgres checkpointing; LiteLLM = LLM provider abstraction; AI Gateway = our actual LLM gateway. CUO is LangGraph orchestrating LiteLLM-shaped calls THROUGH the AI Gateway.
- **`asyncio.wait_for` is the right timeout primitive**, not `asyncio.timeout()` (3.11+ context manager). `wait_for` cancels the underlying task cleanly; `timeout()` requires careful exception handling that's easy to get wrong.
- **`InMemoryCheckpointer` is intentional for slice 2** — checkpointing is for state recovery on restart, but slice-2 requests are stateless per request (no multi-step chains yet). Adding the Postgres dependency now would buy us complexity without value; FR-CUO-102 ships when chains land.
- **`temperature: 0.0` in litellm_proxy** — replay determinism requires zero-temperature sampling. Cost: marginal quality loss on edge cases. Benefit: replay-equivalence claim is meaningful.
- **`response_format: {"type": "json_object"}`** — forces the LLM into structured-output mode at the provider; Pydantic validation is the second line of defence. Together they make the parse-success rate > 99%.
- **`max_tokens: 500`** — bounds the LLM response size; `rationale` field is capped at 500 chars; `skill_name + arguments` together fit in ~200 tokens; 500 is safe with slack.
- **`X-Cuo-Decision-Id` header on the AI Gateway call** — lets the gateway chain its own `ai.precheck/postcall_reconcile` rows under the same decision ID. Audit-trail follow-through across services.
- **`cascade_taken` flag is mutated, not removed** — keeping the flag in state through the second branch entry lets the audit row capture "cascade was attempted but didn't help" — useful for the analyst's "when does cascade rescue confidence" report.
- **`llm_attempts` counter at most 2** — first attempt + 1 retry with strengthened prompt. Tested in `test_supervisor_freeform_rejection`. More retries waste budget.
- **`PERSONA_CATALOGUE` is a module-level dict** — loaded at import time. Not in a database. Adding a persona is an ADR + code change, same discipline as adding an RBAC role.
- **`defer_to_human_matrix` lists operation TYPES, not skill names** — operations are abstract (e.g. `invoice_emit`); skills implement them. The skill catalogue carries `operation: invoice_emit` annotation; the supervisor consults the matrix by operation, not skill_name. Allows new skills implementing the same operation to inherit the matrix block.
- **The graph is recompiled per process** — not per request. `build_supervisor_graph()` is called once at module import; `run_supervisor` reuses the compiled graph. Performance: ~5 ms compile, amortised over thousands of requests.
- **`cuo_state_v = 1` is the published contract** — future state schema changes must consider backward compatibility for replay; ±2 version tolerance is the published guarantee.
- **W3C `traceparent` is generated at the trust boundary** when caller doesn't provide one (per AUTHORING.md rule 22). The supervisor entry is a trust boundary — chat/email/etc. may or may not propagate.
- **`OTel sampling 1%` for steady-state spans, 100% for non-success** — the high-volume happy path is sampled to keep storage costs sane; the rare error path is always captured for debugging. Matches FR-OBS-006 tail-sampling pattern.
- **`Counter cardinality`** — `persona` label has 11 values (closed catalogue); `path_taken` has 6 values; `outcome` has ~6 values. Combined cardinality: ~400. Adding `tenant_id` would explode this — explicitly NOT included in label set; aggregate at Grafana query time.
- **`request_id` is ULID** — sortable by time, 26-char, unique across services. Generated at supervisor entry; embedded in BRAIN row + every child span.
- **Persona JWT format `cuo-<persona-key>@<semver>`** — matches FR-AI-014; the version part lets us roll out persona prompt changes without breaking existing tokens (with grace window).
- **`test_supervisor_litellm_routes_via_gateway` is an AST walker, not a runtime check** — runtime would only catch the import if the path were exercised; AST walker catches all imports at CI before any code runs.
- **`apply_brain_pii_rules` is the FR-BRAIN-111 entry point** — same rules used by every BRAIN audit row builder; consistent PII scrubbing across modules.
- **`current_trace_id_hex` formats via `{}` not `{:?}`** — AUTHORING.md rule 24; Display, not Debug.
- **The CLI's exit codes follow `cyberos-cli-exit`** — shared crate per AUTHORING.md rule 9; tests assert numeric values to catch drift if the shared crate's mapping changes.
- **Per-persona defer matrix coverage** — `test_persona_catalogue_complete` asserts that every C-level persona's matrix has ≥ 1 entry (Genie's matrix is shorter — appropriate for the unspecialised entry point).
- **`max_tokens` budget vs `response_format: json_object`** — JSON mode forces structured output; max_tokens limits length. Edge case: LLM produces a valid JSON header `{"skill_name": "...` and gets cut off mid-rationale. Pydantic validation fails on truncated JSON → fall through to retry. Tested.

---

*End of FR-CUO-101.*
