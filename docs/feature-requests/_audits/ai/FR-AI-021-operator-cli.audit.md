---
fr_id: FR-AI-021
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (229 lines)
score_post_expansion: 9.0/10      # after expanding to FR-AI-014 / FR-AI-019 depth (~1000 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

FR-AI-021 was expanded from 229 lines to ~1000 lines matching FR-AI-014 / FR-AI-019 depth.

The expansion added 11 §1 normative clauses (#6 operator-token authentication with role-gating, #7 standardised exit codes, #8 versioned JSON schemas, #9 expanded subcommand catalogue with role + audit kind columns, #10 atomic multi-field policy set, #11 production-drill safety guard, #12 parseable diff output, #13 no-network policy validate, #14 secret redaction, #15 shell completions, #16 OTel CLI metrics), 8 substantive §2 rationale paragraphs (single-binary mental-model argument, JSON-versioning consumer-script stability, audit-with-command-SHA256 forensic-replay primitive, --confirm exit-4 distinct-from-user-error, operator-token blast-radius limitation, role-gating layered defence, production-drill ceremony rationale, secret-redaction operator-session-exposure boundary, policy-diff change-management workflow, MUST-priority-vs-COULD justification, memory-emit-dry-run validation precedence), full Rust type system in §3 (clap derives for every subcommand, ExitCode enum with explicit values, OperatorClaims + Role + require_role, JSON-schema validation helper, full canonical row builders for cli_policy_updated / cli_breaker_reset / cli_failover_drill / cli_expiry_repaired / cli_memory_emitted), expanded §4 from 10 to 19 acceptance criteria, full Rust test bodies in §5 (version + usage + policy-set-no-confirm-exit-4 + policy-set-with-confirm-emits-audit + missing-token-exit-2 + insufficient-role-exit-2 + production-drill-requires-extra-flag + breaker-reset-emits-audit + expiry-repair-dedupes + json-validates-schema + secret-redacted), expanded §6 with full main.rs entrypoint, expanded §7 with code/concept/operational dep split, 8 example payloads in §8 (usage human + JSON + policy-set audit + diff + breaker status + drill audit + expiry repair + production safety guard), 21 failure modes in §10 (vs. 5 in first pass), 9 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — No CLI authentication or role-gating; binary access = mutate access

- **severity:** error
- **rule_id:** security / least privilege
- **location:** §1 (no auth clause), §3 (no token verification)
- **status:** resolved

#### Description

The first-pass had no authentication. Any operator with the `cyberos-ai` binary could run `policy set --confirm`, `breaker reset --confirm`, `expiry repair --confirm` — including read-only operators on bastion hosts, automation scripts, or anyone who got temporary terminal access.

This is a least-privilege violation: read-only operators (e.g., support engineers viewing usage) should not be able to mutate production policy. A compromised laptop should not equal compromised production.

#### Suggested fix

1. Add §1 #6 normative requirement: `CYBEROS_AI_OPERATOR_TOKEN` env var with short-lived JWT (8-hour expiry); operator obtains via internal SSO.
2. Token carries `operator_id` (kebab-case email) AND `roles: Vec<Role>` (Read | Mutate | Admin).
3. `cli/auth.rs` module with `parse_token` + `require_role`.
4. Read commands accept any role; mutate commands require Mutate or Admin; `failover drill` and `expiry repair` require Admin.
5. Missing or invalid token → exit 2 (AuthFailed); insufficient role → exit 2 with "needed X; have Y" message.
6. Add ACs #12 + #13 + §5 tests for both auth failure modes.
7. `operator_id` is included in EVERY audit row (per §1 #4).

### ISS-002 — `failover drill` can take down production traffic with no safety guard

- **severity:** error
- **rule_id:** robustness / production safety
- **location:** §1 catalogue (drill mentioned), §10 (no guard row)
- **status:** resolved

#### Description

The first-pass `failover drill <provider>` command "forces a 5xx storm to test failover." Run in production, this deliberately fails real provider calls for the drill duration — affecting real tenants.

A production drill might be necessary occasionally (validate failover under real load), but it should be DELIBERATE, not a typo away. The first-pass had only `--confirm`, which is the same gate as a routine policy update. An operator habituated to typing `--confirm` could trip a production drill without conscious intent.

#### Suggested fix

1. Add §1 #11 normative requirement: in `production` env (`CYBEROS_DEPLOYMENT_TIER=production`), `failover drill` requires `--prod-confirmed-aware` flag AND interactive Y/N prompt.
2. Display affected-tenants and affected-aliases counts in the prompt before requesting confirmation.
3. Non-tty input in production exits 4 unconditionally (no piped scripts).
4. Add AC #14 + §5 test `failover_drill_in_production_requires_extra_flag`.
5. Add §10 row + §11 note explaining the ceremony principle.

### ISS-003 — Audit row builders not specified; mutating commands have no canonical audit shape

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §1 #4 + §1 #5 (mention audit rows), §3/§6 (no builders shown)
- **status:** resolved

#### Description

The first-pass §1 #5 said: *"MUST emit a memory audit row for any mutating operation (set policy, reset breaker, repair holds)."* But no builder shown. The subcommand catalogue listed audit-row kinds (`policy_updated`, `breaker_reset`, `failover_drill_started`, `expiry_repaired`) without specifying their payloads.

A code-gen agent has no template for what fields each row carries. Worse: each command's mutation context is different (policy set has `before`/`after` per field; breaker reset has `target`; failover drill has `duration_s` + `tier`; expiry repair has `deduped_count`). A single generic builder won't capture all the relevant context.

#### Suggested fix

Add five canonical builders in §3 + §6:
- `canonical::cli_policy_updated(operator_id, tenant, changes, command_sha256, request_id)`
- `canonical::cli_breaker_reset(operator_id, target, command_sha256, request_id)`
- `canonical::cli_failover_drill(operator_id, target, duration_s, tier, command_sha256, request_id)`
- `canonical::cli_expiry_repaired(operator_id, deduped_count, command_sha256, request_id)`
- `canonical::cli_memory_emitted(operator_id, emitted_kind, command_sha256, request_id)`

The `command_sha256` field is the audit-replay primitive (full command line hashed). Add §8 example payloads for each row variant. Add §5 tests asserting row emission per command.

### ISS-004 — `--json` output schema not specified; consumer scripts can't rely on shape stability

- **severity:** error
- **rule_id:** API stability
- **location:** §1 #3 (mentions `--json`), §3 (no schemas)
- **status:** resolved

#### Description

The first-pass said `--json flag produces machine-readable` but didn't specify the shape. Each command's output JSON could change between releases, breaking downstream automation.

This is the classic API-stability problem: the JSON output IS an API for consumer scripts. Adding a field is non-breaking; removing or renaming a field IS breaking. Without versioning, consumers parse a field one day and get errors the next.

#### Suggested fix

1. Add §1 #8 normative requirement: every JSON output starts with `"schema_version":"v1"`.
2. Schema files in `cli/json_schemas/<command>.v1.json` (JSON Schema draft-07).
3. Bumping a schema requires explicit FR amendment + retain prior version for one release cycle.
4. Add `cli/json_schemas.rs` with `validate_output` helper using the `jsonschema@0.18` crate.
5. Add AC #17 + §5 test `json_output_validates_against_usage_v1_schema`.
6. Add §10 row "JSON output schema drift → CI fails" + §11 note.

### ISS-005 — Exit codes not specified; scripts can't distinguish failure modes

- **severity:** warning
- **rule_id:** API stability / scriptability
- **location:** §1 (no exit-code clause), §3 (no enum)
- **status:** resolved

#### Description

The first-pass had implicit exit codes — likely `0` for success and `1` for "any error." CI scripts wrapping CLI calls need to distinguish failure modes (auth failed = refresh token; remote unreachable = retry; user error = fix args; destructive without confirm = re-run with --confirm).

Without standardised exit codes, scripts can either treat every non-zero as fatal (over-aggressive) or ignore exit codes entirely (under-aggressive). Neither is right.

#### Suggested fix

1. Add §1 #7 normative requirement: enumerated exit codes with stable numerical values.
2. `cli/exit_codes.rs` with `ExitCode` enum:
   - `0` Ok
   - `1` UserError
   - `2` AuthFailed
   - `3` RemoteUnreachable
   - `4` DestructiveWithoutConfirm
   - `5` SchemaViolation
   - `6` InternalError
3. Document exit codes in §1 #7 + §6 entrypoint maps errors to codes.
4. Add §5 tests asserting specific exit codes per failure mode.

### ISS-006 — `policy set --cap-usd` only updates one field; multi-field atomicity not addressed

- **severity:** warning
- **rule_id:** correctness / transaction discipline
- **location:** §1 #9 catalogue ("policy set <tenant> --cap-usd <N>"), §3 (no multi-field handling)
- **status:** resolved

#### Description

The first-pass catalogue showed `policy set <tenant> --cap-usd <N>` — a single-field mutation. But operators routinely need to update multiple fields in one operation (cap + ZDR requirement + residency together for a new tenant onboarding). With single-field-only design, operators run multiple `policy set` commands sequentially — risking partial-application failure (cap updates, then ZDR fails because the YAML has a typo, leaving the tenant in an inconsistent state).

#### Suggested fix

1. Update §1 #9 catalogue to show multi-field invocation: `policy set <tenant> --field=value [...] --confirm`.
2. Add §1 #10 normative requirement: multi-field set is atomic (single Postgres transaction); partial failure rolls back.
3. Show the clap structure in §3 with `--cap-usd: Option<f64>`, `--zdr-required: Option<bool>`, etc.
4. Add AC #6 asserting atomicity (both fields or neither).
5. Add §10 row "policy set partial failure (one field invalid) → atomic rollback; exit 5; no fields applied."
6. The audit row's `changes` field carries the full list of (before, after) per field — operators see the complete change in one row.

## §3 — Strengths preserved through expansion

- §3 introduces `Role::{Read, Mutate, Admin}` as the operator-permission primitive — the type system enforces "you can't call a mutating command with a Read-only token" without runtime reflection.
- §1 #8 versioned JSON schemas convert the CLI output from "best-effort string formatting" into a stable contract that future scripts can build on.
- §1 #11 production-drill safety guard makes accidental production impact structurally impossible (verbose flag name + interactive Y prompt + tty check).
- §1 #14 secret redaction is the operator-data-exposure boundary; tenant-supplied API keys never leak through CLI output even in shared sessions.
- §3 provides FIVE canonical row builders (one per mutating command), each carrying command-specific context PLUS the universal `command_sha256` for forensic replay.
- §10 inventory grew from 5 rows to 21 — including the expired-token path, the insufficient-role path, the partial-policy-failure-rollback path, the secret-leak detection path, and the audit-emit-fails-after-mutation sev-1 path. Each row has an unambiguous detection mechanism.
- §11 documents the operational philosophy ("CLI is the single ops surface; new capabilities ship as subcommands") so future scope-creep into web-UI etc. is consciously deferred not accidental.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: §1 #6 operator token + role-gating; `cli/auth.rs` with JWT parsing + `Role` enum + `require_role`; ACs #12 + #13 + §5 tests `missing_token_exits_2` and `insufficient_role_exits_2`; `operator_id` in every audit row payload.

- **ISS-002 RESOLVED**: §1 #11 production drill safety guard; `--prod-confirmed-aware` flag + interactive Y prompt + tty check; AC #14 + §5 test `failover_drill_in_production_requires_extra_flag`; §10 row + §8 example output showing affected-tenants count.

- **ISS-003 RESOLVED**: Five `canonical::cli_*` builders shown in §3 + §6 with full payload schemas; `command_sha256` audit-replay primitive in every row; §8 example payloads for `ai.cli_policy_updated`, `ai.cli_breaker_reset`, `ai.cli_failover_drill`, `ai.cli_expiry_repaired`; §5 tests assert row emission per mutating command. (Audit-row kinds use `ai.cli_*` namespace per FR-AI-003 closed-set discipline.)

- **ISS-004 RESOLVED**: §1 #8 normative; `cli/json_schemas/` directory with `<command>.v1.json` files; `validate_output` helper using `jsonschema@0.18`; every output starts with `"schema_version":"v1"`; AC #17 + §5 test `json_output_validates_against_usage_v1_schema`; §10 + §11 documented.

- **ISS-005 RESOLVED**: Shared `cyberos-cli-exit::ExitCode` re-export (numerical values 0–7 per cross-CLI contract: 0=Ok, 1=UserError, 2=AuthFailed, 3=RemoteUnreachable, 4=DestructiveWithoutConfirm, 5=AlreadyInitialised, 6=SchemaViolation, 7=InternalError); §1 #7 normative; §6 entrypoint maps errors to codes; §5 tests assert specific codes per failure mode (`code(2)` for auth, `code(4)` for missing-confirm, etc.).

- **ISS-006 RESOLVED**: §1 #9 catalogue updated to multi-field syntax; §1 #10 atomicity requirement (single Postgres transaction); §3 clap structure shows `Option<f64>` per field; AC #6 asserts atomicity; audit-row `changes` field carries full diff.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-021 audit (final). Status: PASS at 10/10.*
