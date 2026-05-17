---
fr_id: FR-MCP-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands the MCP Elicitation primitive per MCP 2025-11-25 spec; closes FR-MCP-006's elicit-mode placeholder; exposes server-initiated structured prompts via tool-side `TaskCtx::elicit()` API. Final form: 1,090 lines, 25 §1 normative clauses (1 migration, 5 elicitation types with fixed per-type JSON Schemas, request/response/cancel endpoints, JSON Schema validation, NATS push + polling transports, file_upload via presigned S3, FR-MCP-006 confirmation integration, 5 BRAIN audit kinds + 1 cross-caller security audit kind, sync-tool ban), 20 acceptance criteria, 10 verification tests, 22 failure-mode rows, 20 implementation notes.

6 issues caught by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — Worker holds task slot during elicitation wait (long-running)

§11.18 acknowledges worker holds slot while awaiting human response. For 30-min max timeout this means 30 min of worker semaphore consumption. Worker pool size = 4 (default) means 4 concurrent elicits saturate the pool. Resolved: §11.18 marks this as ACCEPTED tradeoff at slice 3 — task-bounded concurrency means slot reservation is correct (the work IS in progress, blocked on user input). Slice 4 may add "suspend slot on elicit, resume on response" pattern. AC for worker-pool isolation (FR-MCP-007 AC #13) still holds because elicit-blocking tasks belong to one module's pool.

### ISS-002 — Sync-tool elicit ban enforcement timing

§1 #21 + DEC-1147 ban sync-tool elicit. The check is at runtime (`task_id.is_none()`). But sync tool implementers might not test this path. Resolved: §11.13 documents that registration-time linting (slice 4) would catch this at PR; slice 3 relies on runtime error.

### ISS-003 — Cross-caller security audit kind outside 5-core list

§17 emits 5 core kinds. The cross-caller security event (`mcp.cross_caller_access_denied`) is outside but mentioned in §11.5 + §10 row + §6.2 code. AUTHORING.md rule 8 closed-set extension requires explicit listing. Resolved: §11.16 explicitly adds `mcp.cross_caller_access_denied` as a 6th security-signal kind (outside the 5-core because emitted by the security boundary check, not the lifecycle path); FR-AI-003 closed-set extension covers 6 total.

### ISS-004 — Idempotency uniqueness on response_payload_sha256

§14 + DEC-1156 say idempotent on `(elicitation_id, response_payload_sha256)`. But the table doesn't have a UNIQUE on this combination. Re-submission relies on application-level check at handler. Resolved: §11.20 clarifies — application-level idempotency via UPDATE WHERE status='pending' returns 0 rows on second attempt; no need for additional DB constraint. The `(elicitation_id, response_payload_sha256)` semantic is enforced at handler-level, not schema.

### ISS-005 — Presigned S3 URL ttl vs elicitation timeout mismatch risk

§10 row "Presigned S3 URL expired before upload" notes ttl matches elicitation timeout per DEC-1153. But what if S3 clock skew vs server clock = expiry mismatch? Resolved: §11.4 documents S3 + server using same NTP source typically; if mismatch occurs, the presigned URL ttl provides the binding (S3 enforces); elicitation timeout is a soft upper-bound. Worst case: 10s skew = elicitation looks open but S3 PUT fails; caller retries; eventually elicitation expires.

### ISS-006 — Tool API generic R: DeserializeOwned without schema-type binding

§3.4 + §6.1 tool API `elicit<R: DeserializeOwned>()` returns parsed `R` but the response_schema isn't statically bound to R's shape. Tool could pass schema that doesn't match R, leading to runtime parse errors after schema validation passes. Resolved: §11.17 documents this is tool-implementer responsibility; framework can't statically verify schema-vs-type alignment in Rust generics. Test pattern is for tool implementers to validate their schemas with sample responses in unit tests.

## §3 — Resolution

All 6 mechanical concerns addressed. Worker-slot semantics documented as accepted tradeoff; sync-tool ban scope clarified; 6th audit kind (cross-caller security) explicit; idempotency mechanism documented; presigned URL clock-skew handling noted; tool API generic-vs-schema limitation acknowledged as implementer responsibility.

The 1,090-line length is justified by 5 elicitation types × per-type fixed schemas + dual transport (polling + NATS push) + file upload via S3 + FR-MCP-006 integration + JSON Schema validator + 22 failure modes. Density comparable to peer MCP FRs.

**Score = 10/10.**

---

*End of FR-MCP-008 audit.*
