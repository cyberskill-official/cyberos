# cyberos-ai-gateway

**P0 module · the cost-of-everything gate.**
Implements [`docs/feature-requests/ai/FR-AI-001..022`](../../docs/feature-requests/ai/) — pre-call cost-ledger, multi-provider router, PII redaction, persona injection, residency pinning, audit-row emission. Every LLM call across the platform routes through this service.

## Status (2026-05-19 wave)

| FR | Title | Status |
|---|---|---|
| **FR-AI-003** | memory audit-row bridge (canonical Writer subprocess) | **shipped** (core path + path-validation + chain-verify + 5s timeout-with-SIGTERM; typed builders for the slice-1 closed set: precheck · invocation · invocation_failed · hold_expired · persona_loaded) |
| **FR-AI-005** | Tenant-policy YAML loader | **shipped** (10/10 ACs covered by unit + integration tests; `ArcSwap` lock-free cache; `notify` file-watcher; `cyberos-ai policy validate` + `policy list` CLI) |
| FR-AI-001 | Cost-ledger pre-call check | pending (next session — depends on FR-AI-003 + FR-AI-005, both now ✅) |
| FR-AI-002 | Cost-ledger post-call reconcile | pending |
| FR-AI-004 | Cost-hold expiry cleanup job | pending |
| FR-AI-006..022 | Router · PII · residency · cache · operator CLI · OTel | pending (slices 2–5) |

## Layout

```
src/
├── lib.rs                       # public re-exports
├── policy.rs                    # FR-AI-005 module root
├── policy/
│   ├── schema.rs                # TenantPolicy + AiPolicy + Provider + Residency + EmergencyOverride
│   ├── cache.rs                 # ArcSwap-backed lock-free cache
│   └── loader.rs                # init_loader / load_for_tenant / shutdown_loader / validate_yaml + file-watcher
├── memory_writer.rs              # FR-AI-003 module root
├── memory_writer/
│   └── canonical.rs             # AGENTS.md §6.2 canonical-JSON serialiser
└── bin/
    ├── cyberos_ai.rs            # `cyberos-ai` operator CLI (slice-1 subcommands)
    └── gen_schema.rs            # JSONSchema emission for CI drift detection
config/tenants/
├── EXAMPLE.tenant.yaml          # documented schema (NOT loaded — `EXAMPLE.` prefix)
└── <tenant-id>.yaml             # per-tenant policy (added by ops)
tests/
├── policy_loader_test.rs        # FR-AI-005 §5 integration tests (AC #1..#10)
└── fixtures/policy/             # valid · invalid-schema · missing-required
```

## Local development

```bash
# Build everything in the workspace (workspace root is `services/`):
cargo build -p cyberos-ai-gateway

# Run the policy tests:
cargo test -p cyberos-ai-gateway --lib policy
cargo test -p cyberos-ai-gateway --test policy_loader_test ac1_valid_yaml_loads_and_matches
cargo test -p cyberos-ai-gateway --test policy_loader_test ac3_invalid_schema_rejected_on_init
cargo test -p cyberos-ai-gateway --test policy_loader_test ac4_out_of_range_values_rejected

# Run the OnceCell-shared singleton tests one at a time (AC#2/AC#5/AC#6/AC#7/AC#9):
cargo test -p cyberos-ai-gateway --test policy_loader_test -- --ignored --test-threads=1

# Validate a YAML without loading it:
cargo run -p cyberos-ai-gateway --bin cyberos-ai -- policy validate \
    services/ai-gateway/config/tenants/EXAMPLE.tenant.yaml

# Emit the JSONSchema mirror (CI gate — `git diff --exit-code` on the file):
cargo run -p cyberos-ai-gateway --bin gen-schema -- \
    --out services/ai-gateway/config/tenants/SCHEMA.json
```

## FR-AI-003 dependency note

`emit()` invokes `python3 -m cyberos.writer put` as a subprocess. The Writer ships from `modules/memory/runtime/` and MUST be on the PATH at runtime. The startup health check (`memory_writer::check_writer_available`) is the contract gate per FR-AI-003 §1 #10.

Integration-tested behaviour requires a live `<memory-root>/` and the Python Writer module installed. The slice-1 ship lands the Rust-side bridge fully; the cross-language smoke tests sit behind a feature flag (`integration-writer`) until the memory runtime install path is documented for CI.

## §14 protocol emission

This module participates in the `AGENTS.md §14.1` protocol — every commit that touches `services/ai-gateway/**` emits a §14.1 block alongside the diff. The block is committed to `cyberos.invariants.yaml` and surfaced in `docs/feature-requests/BACKLOG.md §0.5` production-status table.

## Next-session todo

1. Wire FR-AI-001 (cost-ledger pre-call) on top of `policy::load_for_tenant` + `memory_writer::emit`. Migration `services/ai-gateway/migrations/0001_cost_ledger.sql` per FR-AI-001 §3.
2. FR-AI-002 reconcile path.
3. FR-AI-004 expiry-cleanup pg-scheduler job.
4. FR-AI-006/008 router → wire residency + provider selection.
5. FR-AI-022 OTel traces — required by FR-OBS-004 (cross-pillar correlation).
