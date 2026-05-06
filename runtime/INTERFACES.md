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
