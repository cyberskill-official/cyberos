# cyberos-ai operator CLI

`cyberos-ai` is the AI Gateway operator binary for read-only inspection and audited mutations.

## Global flags

- `--json` emits versioned machine-readable output.
- `--confirm` is required for every mutating command.

All commands except `completions` require `CYBEROS_AI_OPERATOR_TOKEN`. Mutating commands require `mutate` or `admin`; `failover drill`, `expiry repair`, and non-dry-run `memory emit` require `admin`.

## Commands

| Command | Role | Mutation | Audit row |
|---|---:|---:|---|
| `usage [--tenant <id>] [--month YYYY-MM]` | read | no | - |
| `models list` | read | no | - |
| `models pricing` | read | no | - |
| `policy set <tenant> [--cap-usd N] [--zdr-required bool] [--residency sg-1\|eu-1\|us-1\|vn-1] [--allowed-personas ...] --confirm` | mutate | yes | `ai.cli_policy_updated` |
| `policy validate <yaml-file>` | read | no | - |
| `policy diff <tenant> --vs <yaml-file>` | read | no | - |
| `failover drill <provider:model> [--duration s] --confirm [--prod-confirmed-aware]` | admin | yes | `ai.cli_failover_drill` |
| `invoice export <tenant> --period YYYY-MM [--format json\|csv\|pdf]` | read | no | `ai.cli_invoice_exported` |
| `breaker status` | read | no | - |
| `breaker reset <provider:model> --confirm` | mutate | yes | `ai.cli_breaker_reset` |
| `expiry status` | read | no | - |
| `expiry repair --confirm` | admin | yes | `ai.cli_expiry_repaired` |
| `memory emit <yaml-file> --dry-run` | read | no | - |
| `memory emit <yaml-file> --confirm` | admin | yes | `ai.cli_memory_emitted` |
| `memory audit-trail <tenant> --since <iso8601>` | read | no | - |
| `completions bash\|zsh\|fish` | none | no | - |

## Exit codes

| Code | Meaning |
|---:|---|
| 0 | success |
| 1 | user error |
| 2 | authentication or authorization failure |
| 3 | remote dependency unreachable |
| 4 | destructive operation missing required confirmation |
| 5 | already initialized, reserved |
| 6 | schema violation |
| 7 | internal error |

## JSON schemas

JSON output starts with `"schema_version": "v1"`. Schemas live under `src/cli/json_schemas/` and are treated as compatibility contracts for automation.

## Production failover drill

When `CYBEROS_DEPLOYMENT_TIER=production`, `failover drill` requires `--confirm`, `--prod-confirmed-aware`, and an interactive `Y` prompt on a terminal before the audit row is emitted.
