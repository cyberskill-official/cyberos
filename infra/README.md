# infra/

Infrastructure as Code, load tests, anti-regression harness.

## Layout

```
infra/
├── terraform/
│   ├── shards/
│   │   ├── vn/        # AWS ap-southeast-1 (Singapore) for VN tenants
│   │   ├── sg/        # AWS ap-southeast-1 (Singapore) for SG / ASEAN tenants
│   │   ├── eu/        # AWS eu-central-1 (Frankfurt) for EU tenants
│   │   └── us/        # AWS us-east-1 (N. Virginia) for US / LATAM tenants
│   ├── modules/       # Reusable Terraform modules
│   └── envs/
│       ├── dev/
│       ├── staging/
│       └── prod/
├── kubernetes/
│   ├── base/          # Kustomize base manifests
│   └── overlays/      # Per-shard, per-env overlays
├── load-tests/
│   └── p3-gate-soak/  # FR-OBS-004 — 100 tenants × 10 users × 1,000 BRAIN ops × 7 days
│       ├── config.yaml
│       ├── seed.sh
│       ├── scenarios/
│       ├── cross-tenant-invariants/
│       └── reports/
└── anti-regression/   # FR-OBS-005 — CI-blocking regression suite
```

## Shards (FR-TEN-001)

CyberOS runs on a 4-shard residency-partitioned topology:

| Shard | AWS region | Bedrock region | Tenants |
|---|---|---|---|
| `vn` | `ap-southeast-1` | `ap-southeast-1` | Vietnam (PDPL) |
| `sg` | `ap-southeast-1` | `ap-southeast-1` | Singapore + ASEAN (non-VN) |
| `eu` | `eu-central-1` | `eu-central-1` | EU + UK + Switzerland (GDPR + EU AI Act) |
| `us` | `us-east-1` | `us-east-1` | US + Canada + LATAM |

Cross-shard reads/writes are forbidden. Cross-tenant invariants are tested in CI on every release.

## Why Hetzner appears in the PRD but not here

The PRD names Hetzner as a primary infra option. By P3 (multi-tenant), the residency requirement makes AWS the right primary across all 4 shards (Bedrock is AWS-native; Anthropic ZDR is also AWS-served). Hetzner remains an option for development and for shards where AWS is overpriced; that decision lands as `DEC-XXX` in the SRS Decisions Log when first provisioned. For now, Terraform is structured to allow swapping providers at the module level.

## load-tests/p3-gate-soak

Owns the soak rig consumed by FR-OBS-004's P3 → P4 gate evidence:

- 100 synthetic tenants × 10 users × 1,000 BRAIN ops/day for 7 days.
- Cross-tenant invariant tests every 6 hours.
- Audit-chain anomaly detection via `@cyberos/audit-chain` external CLI verifier.
- Runs on a dedicated `load-test` shard provisioned + decommissioned per soak.

## anti-regression

CI-blocking regression suite covering all 22 modules' critical paths. Specified in FR-OBS-005:

- Per-module critical-path Gherkin tests.
- Cross-tenant invariant tests.
- Cross-module integration tests (PROJ → TIME → INV → REW chain).
- Compensation/equity human-decision invariant (AI-only path returns 403).
- Persona-scope contract tests.
- EU AI Act high-risk human-in-the-loop tests.
- Audit-chain integrity verifier.

## Status

`stub` — created 2026-05-03.
