---
task_id: TASK-MEMORY-122
audited: 2026-06-29
verdict: PASS
score: 10/10
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
eu_ai_act_review: limited — dedicated `## AI Risk Assessment` present (COND-003 closed); personal-data handling stated at the point of capture
---

## §1 — Verdict summary

TASK-MEMORY-122 makes day-1 wide capture real: a shared `cyberos-capture` crate (the build→`emit()` mechanism), per-module `capture.rs` emitters wiring AUTH (`auth.signed_in`, `auth.sign_in_failed`) and CHAT (`chat.message_created/_edited/_deleted`, `chat.channel_created/_joined/_left`, `chat.dm_opened`, `chat.presence_changed`), the flip that turns ON the chat→brain audit link (`CHAT_AUDIT_DATABASE_URL`, which P0 left OFF), and a bounded idempotent consent-gated backfill. Scope: 16 §1 normative clauses (shared emitter, emitter convention, AUTH wiring, CHAT wiring, link-ON, route-through-emit, best-effort, presence edge-dedup, backfill, deterministic backfill id+time, backfill consent, metrics, source_channel, trace_id, runbook, direct-write decoupling). 9 §2 rationale paragraphs. §3 carries the crate, the AUTH + CHAT emitter Rust, the `main.rs`/compose link flip, and the backfill function. 18 ACs; §5 has 3 AUTH test fns + a chat smoke (2 scenarios) + 2 backfill test fns; §10 lists 22 failure rows; §11 has 10 implementation notes plus a dedicated `## AI Risk Assessment`.

The decisions are encoded: DEC-2710 (wide, day-1, from login) → AUTH sign-in/presence + CHAT message/channel emitters; DEC-2711 (platform-only, content_ref not raw) → §1 #4 + the pointer-only message emitters; DEC-2712 (every emitter consent-gated) → §1 #6 routing through `emit()` + `disallowed_tools` forbidding hand-built rows; DEC-2713 (turn ON the chat→brain link) → §1 #5 + the `main.rs`/compose change, correctly grounded in the real P0 plumbing (chat already reads the var, holds `audit_pool`, writes via `cyberos-audit-chain`); DEC-2714 (define the emitter contract once) → §1 #1/#2 + the `CaptureEmitter` convention. Scope is correctly bounded to AUTH + CHAT + the contract; PROJ/EMAIL/APP/MCP emitters are deferred per-module against the same contract.

## §2 — Findings (all resolved)

### ISS-001 — Per-module capture drift
AUTH and CHAT could each wire capture differently, and later modules would re-litigate the shape. Resolved: §1 #1/#2 + the `cyberos-capture` crate (mechanism) + per-module `capture.rs` (thin translation) + DEC-2714; a new module adds a `capture.rs`, not a pattern. AC #1 asserts the shared path; §10 notes a module without `capture.rs` simply emits nothing (no half-wired state).

### ISS-002 — The chat→brain gap the plan names
P0 left `CHAT_AUDIT_DATABASE_URL` off, so live chat content never reaches the brain. Resolved: §1 #5 turns it on, grounded in the actual code (the `audit_pool` + the warn line in `main.rs`), flips warn→info, marks it required-in-prod, and sets it in deploy compose. AC #7 asserts rows appear with it set and none with it unset. This closes gap #1 of the brain plan.

### ISS-003 — An emitter forgetting the consent gate
If any emitter could write an audit row directly, it could capture an unacknowledged subject. Resolved: §1 #6 routes every emitter through TASK-MEMORY-121 `emit()` (gate + validation live there); `disallowed_tools` forbids hand-built `l1_audit_log` rows. AC #8 (live) and AC #13 (backfill) both assert the gate holds. The governance property is structural, not per-emitter discipline.

### ISS-004 — Raw message bodies leaking into the chain
The obvious chat emitter inlines the message text. Resolved: §1 #4 mandates `content_ref: pointer{chat_messages, id}` for create/edit and `none` for delete; bodies stay in chat's DB under chat's RLS. AC #4 asserts the row contains no text and the kind is `pointer`; AC #5 asserts delete is `none` (no dangling pointer).

### ISS-005 — Presence noise
A naive presence emit fires on every websocket, so multi-tab users spam the chain. Resolved: §1 #8 dedups to the 0↔1 connection-count edges, using the `Presence` map already in `realtime.rs`. AC #10 asserts two tabs → one `online`, last close → one `offline`, middle close → none.

### ISS-006 — History before the link / before acknowledgment
Live chat has pre-link history, and a late-acknowledging subject deserves their acknowledged-window activity. A naive backfill double-counts or captures non-consenting people. Resolved: §1 #9–#11 — bounded window, dry-run default, deterministic UUIDv5 `event_id` (idempotent), original `occurred_at_ns` (chronologically honest), and routing through `emit()` so backfill is consent-gated. AC #11/#12/#13 assert idempotency, original-time, and consent-skip.

### ISS-007 — Capture on the critical path
If capture could fail a sign-in or slow a message, it would degrade the product. Resolved: §1 #7/#16 — best-effort everywhere (log+swallow), direct-write to the shared audit DB (not via memory HTTP). AC #9 asserts the sign-in succeeds with the audit pool down; AC #18 asserts capture writes with the memory service stopped.

## §3 — Resolution

All seven concerns addressed. **Score = 10/10.** Depth matches the genuine surface — shared emitter contract × AUTH wiring × CHAT wiring × the link flip × consent-through-emit × presence edge-dedup × idempotent consent-gated backfill — not line targets. Every §1 clause is cited by ≥ 1 AC; every AC maps to a named test across the four test files (AUTH Rust, chat smoke, two backfill Rust); the protocol-/deploy-touching pieces (the link flip, the compose var, the runbook) each have a §11 anchor and a runbook deliverable. The `## AI Risk Assessment` closes COND-003 and, correctly, states the personal-data posture at the point where capture actually begins (platform-only, pointer-not-raw, consent-gated-and-enforced, per-tenant RLS, operator-transparent, no autonomous decision).

The FR correctly produces TASK-MEMORY-121 events rather than inventing a shape, and its `depends_on [TASK-MEMORY-121, TASK-AUTH-002, TASK-CHAT-101]` / `blocks [TASK-MEMORY-123]` matches the canonical BRAIN/EVAL map. Live-realism is strong: every wiring point (the `emit_token_issued` neighbour, the `Presence` count map, the `CHAT_AUDIT_DATABASE_URL`/`audit_pool` plumbing, the `chat_messages` pointer store, the deploy compose, the p0 runbook) is an existing, named artifact in the repo, so the spec is implementable against the code as it stands.

---

*End of TASK-MEMORY-122 audit.*
