# MCP gate-flip checklist (prep 2026-06-28)

What to run after the MCP PR (auto/mcp-finish-008-006-007) is merged, and exactly which FR statuses to
flip. Prepared so the flips are honest: a green gate certifies the code that exists, not the FR clauses
we deliberately deferred to the database slice. Do not flip a slice-FR to done just because the gate is
green.

## DB slice (branch auto/mcp-db-slice) - built + green locally 2026-06-28

A separate branch from auto/mcp-finish-008-006-007, commit-per-FR, all compile + clippy clean locally (143
tests). The DB read/write paths themselves still need Postgres integration tests (run after applying the
migrations). This slice landed much of what the per-FR lists below marked "deferred to the DB slice"; treat
this section as the current truth where it disagrees with the older section.

- FR-MCP-003 slice 3: CI grep gate (scripts/check_sep986_naming.sh + .github/workflows/mcp-sep986-check.yml)
  and the 4 naming audit kinds. The slice-3 remainder is built.
- KMS: src/kms.rs - XChaCha20-Poly1305 envelope seal under MCP_KMS_KEY, swappable for a managed KMS.
- FR-MCP-008 + FR-MCP-006: migration 0016 (mcp_elicitations) + src/elicitation_pg.rs store-of-record. The
  destructive-confirm gate and the elicitation REST handlers persist, are caller-scoped, seal payloads, and
  resume after restart. 006's "persistence of the confirmation" is done.
- FR-MCP-007: migration 0017 (mcp_tasks) + src/tasks_pg.rs store-of-record (caller-scoped, sealed).

Extra steps before the flip, in addition to the cargo/caf/awh steps below:
- Apply 0016 + 0017 to Postgres (add mcp-gateway to the migrate loop after auth) and run the DB-path
  integration tests.
- Put a real MCP_KMS_KEY (base64 of 32 bytes) in the gitignored .env; with MCP_REQUIRE_AUTH=1 plus a
  database but no key, the gateway warns at boot and cannot seal elicitation/task payloads.

Narrowed deferrals after this slice (still enough to keep these at implementing):
- FR-MCP-006: only the audit-sampling story remains.
- FR-MCP-007: worker pool, NATS progress, checkpoints + resume, the long_running annotation + tools/call
  async routing (no request-path task creator yet), idempotency wiring, rate limit, TTL sweeper, prune.
- FR-MCP-008: NATS push, LISTEN/NOTIFY, S3 file_upload, rate limit, timeout sweeper, prune.
- FR-MCP-005: unchanged (no meaningful DB increment) - drift table, rate limit, OTel p95, NATS, residency.

## Gate coverage (verified, no change needed)

- awh goldenset (modules/mcp/.awh/goldenset.yaml): the full-suite task runs
  `cd services && cargo test -p cyberos-mcp-gateway`, which compiles and runs every new FR-004..008 test
  automatically. Plus the held-out error-code acceptance task. No per-FR task to add.
- caf audit-profile (modules/mcp/audit-profile.yaml): RUN_COMMANDS now runs the crate test suite AND
  `cargo clippy -p cyberos-mcp-gateway --all-targets -- -D warnings` (added this prep; the crate is
  `#![warn(missing_docs)]` + `#![deny(missing_debug_implementations)]`, so clippy -D is part of the floor).
- scripts/awh_gate_coverage.py: KNOWN_RED is empty, so mcp is a covered, non-excluded module.

## Run on your Mac (from repo root unless noted)

1. `cd services && cargo test -p cyberos-mcp-gateway && cargo clippy -p cyberos-mcp-gateway --all-targets -- -D warnings`
2. `bash scripts/caf_gate.sh mcp`   (target health + audit conformance; needs the Rust toolchain)
3. Re-seal the awh baseline (the suite grew with 004..008; more green is not a regression, but re-seal to
   keep it current):
   `awh eval modules/mcp/.awh/goldenset.yaml --base-dir . --seeds 1 --out modules/mcp/.awh/eval-baseline.json`
4. Then `awh eval modules/mcp/.awh/goldenset.yaml --base-dir . --seeds 1 --baseline modules/mcp/.awh/eval-baseline.json --max-regression 0.0`

## Status flips (the honest mapping)

Already correct, no change:
- FR-MCP-001 spec compliance: done.
- FR-MCP-002 heartbeat lifecycle: done.

Flip on green:
- FR-MCP-004 OAuth 2.1 + PKCE: implementing -> done. The full flow is built and gate-green (authorize,
  token, refresh, revoke, introspect, DCR incl. confidential + Argon2, the tools/call audience gate, 8
  audit kinds). One minor clause deferred and default-open: the per-tenant redirect-host allowlist
  (clause #11). Flip to done only if you accept that as a follow-up hardening, not a blocker; otherwise
  leave at implementing.

Already flipped to implementing in this prep (were draft; substantial code now exists and is green, but
core FR clauses remain deferred to the DB slice, so NOT done yet):
- FR-MCP-005 protected resource metadata: deferred = drift table + detector, rate limit, OTel p95, NATS
  cache invalidation, tail-sampled prm_served, 4-issuer residency list, EdDSA.
- FR-MCP-006 tool-annotation gating: deferred = persistence of the confirmation + the audit-sampling
  story; the synchronous destructive-confirm gate itself is built and green.
- FR-MCP-007 tasks primitive: deferred = mcp_tasks table + RLS, worker pool, NATS progress, checkpoints
  and resume, the long_running annotation + tools/call async routing, idempotency, rate limit, TTL sweep.
- FR-MCP-008 elicitation: deferred = mcp_elicitations table + RLS, KMS, NATS, LISTEN/NOTIFY, S3
  file_upload, rate limit, sweeper, prune.

Unchanged (separate remaining slice):
- FR-MCP-003 sep986 naming: stays implementing. Slice 3 left: the CI grep gate (scripts +
  .github workflow, DEC-2362) and the four naming audit kinds (DEC-2364).

## Bottom line

At this gate only FR-MCP-004 is a candidate for done. 003 and 005..008 stay implementing until the DB
slice lands their deferred clauses. The deferred items per FR are recorded in the space memory
(cyberos-mcp-build-state) and in each FR body, so the eventual done-flip is unambiguous.

## 2026-07-02 - FR-MCP-004 flipped to done (operator decision)

Per the module review (docs/reviews/MODULE-REVIEW-2026-07-02.md), the operator accepted the ledgered
deferral of the per-tenant redirect-host allowlist (clause #11) as a follow-up, so FR-MCP-004 flips to
done on its green gate. The allowlist remains a named follow-up here and in the FR body; 003 and 005..008
stay implementing until the DB slice lands, exactly as decided at the 2026-06-28 gate.
