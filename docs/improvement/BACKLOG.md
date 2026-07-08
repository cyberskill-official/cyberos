# Improvement backlog - master index

Source: `docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md`. Status values: todo | doing | review | done | blocked. Only the human reviewer sets `done`. Task specs live in the wave files; this table is the single source of truth for status and eligibility.

Eligibility rule: a task is eligible when status is `todo` and every task in `depends_on` is `done` (or `review` if the reviewer has explicitly waived the wait in the ledger). Default pick order: wave, then ID.

## Wave 1 - see and survive (target: day 30)

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-001 | Dependency audit in CI (cargo-audit + cargo-deny) | R19 | p0 | s | - | todo |
| IMP-002 | Refuse dev CORS in production boot | R23 | p0 | xs | - | todo |
| IMP-003 | Secret scanning in CI and pre-push | R27 | p0 | xs | - | todo |
| IMP-004 | Deploy observability stack to P0 | R29 | p0 | m | - | todo |
| IMP-005 | External uptime probes and alerting | R30 | p0 | xs | - | todo |
| IMP-006 | Canary healthcheck and auto-rollback in deploy | R32 | p0 | s | - | todo |
| IMP-007 | apps/web test spine | R12, R46 | p0 | m | - | todo |
| IMP-008 | Goldensets as first-class gate inputs | R16 | p0 | s | - | todo |
| IMP-009 | LLM call ledger in ai-gateway | R41 | p0 | s | - | todo |
| IMP-010 | Telemetry-to-FR bridge | Stage 0 | p0 | m | IMP-004 | todo |
| IMP-011 | Structured gate-failure taxonomy | Stage 0 | p0 | s | - | todo |

## Wave 2 - measure and evaluate (target: day 60)

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-012 | Coverage measurement and ratchet | R11 | p1 | s | - | todo |
| IMP-013 | Cross-service contract tests | R14 | p1 | m | - | todo |
| IMP-014 | External audit-chain anchoring | R20 | p1 | s | - | todo |
| IMP-015 | Nightly chain-integrity monitor | R21 | p1 | s | - | todo |
| IMP-016 | Staging environment | R31 | p1 | m | - | todo |
| IMP-017 | OTLP tracing export | R37 | p1 | m | - | todo |
| IMP-018 | Prometheus metrics endpoints | R38 | p1 | s | - | todo |
| IMP-019 | SLO definitions and burn-rate alerts | R39 | p1 | s | IMP-004, IMP-018 | todo |
| IMP-020 | FR outcome scoring | Stage 1 | p1 | s | - | todo |
| IMP-021 | Rubric evals for LLM outputs, anchored judge | Stage 1 | p1 | m | IMP-008 | todo |
| IMP-022 | Ban defensive asserts | R13 | p1 | xs | - | todo |

## Wave 3 - widen the envelope (target: day 90)

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-023 | Groom draft FRs with value and confidence | R49 | p1 | m | - | todo |
| IMP-024 | Dream proposal ranking | Stage 2 | p1 | s | IMP-023 | todo |
| IMP-025 | Dream budget, latency and drift gates | Stage 2 | p1 | m | IMP-009 | todo |
| IMP-026 | Auto-revert on gate regression | Stage 2 | p1 | s | IMP-008 | todo |
| IMP-027 | Enable auto mode for docs/skills envelope | Stage 2 | p1 | s | IMP-020, IMP-021, IMP-024, IMP-025, IMP-026 | todo |
| IMP-028 | ACE-style skill curation loop | Stage 3 | p1 | l | IMP-008 | todo |
| IMP-029 | Paired-trajectory skill audits | Stage 3 | p1 | m | IMP-008 | todo |
| IMP-030 | QLoRA fine-tuning pilot (obs triage) | Stage 4 | p1 | l | IMP-008, IMP-009, IMP-021 | todo |

## Wave 4 - hardening (continuous)

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-031 | Unified error envelope crate | R1 | p1 | m | - | todo |
| IMP-032 | Extract cyberos-service-kit | R2 | p1 | l | IMP-031 | todo |
| IMP-033 | Wire cloud router adapters in ai-gateway | R3 | p1 | m | - | todo |
| IMP-034 | Chat realtime fanout seam | R4 | p1 | m | - | todo |
| IMP-035 | unwrap/expect burn-down and panic removal | R5, R6 | p1 | m | - | todo |
| IMP-036 | Finish and property-test audit-chain crate | R7 | p1 | m | - | todo |
| IMP-037 | OpenAPI generation per service | R8 | p2 | m | - | todo |
| IMP-038 | Extend RLS property gate, cross-tenant probe | R15 | p1 | m | IMP-016 | todo |
| IMP-039 | Load and soak test suite | R17 | p2 | m | IMP-016 | todo |
| IMP-040 | Mutation testing pilot on shared crates | R18 | p2 | s | - | todo |
| IMP-041 | Secrets inventory and rotation runbook | R22 | p1 | s | - | todo |
| IMP-042 | Rate limits beyond login | R24 | p1 | s | - | todo |
| IMP-043 | Supply-chain hardening (pin, SBOM, sign) | R25 | p2 | m | - | todo |
| IMP-044 | Automated dependency updates | R26 | p2 | s | IMP-001 | todo |
| IMP-045 | Session and token security validation | R28 | p1 | m | - | todo |

## Wave 5 - platform and process (continuous)

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-046 | Backup independence and restore drill | R33 | p1 | m | - | todo |
| IMP-047 | Rebuild-in-60-minutes runbook | R34 | p2 | s | - | todo |
| IMP-048 | Build caching with cargo-chef | R35 | p2 | s | - | todo |
| IMP-049 | Deploy events into audit chain | R36 | p1 | s | - | todo |
| IMP-050 | Client and service error tracking | R40 | p2 | m | - | todo |
| IMP-051 | Least-privilege DB roles and layout doc | R42 | p2 | m | - | todo |
| IMP-052 | Migration discipline CI check | R43 | p2 | xs | - | todo |
| IMP-053 | pgvector operations plan | R44 | p2 | m | - | todo |
| IMP-054 | Generalized retention schedule | R45 | p2 | s | - | todo |
| IMP-055 | Spec-only module manifest | R9 | p2 | xs | - | todo |
| IMP-056 | API versioning and deprecation policy | R10 | p2 | xs | - | todo |
| IMP-057 | Frontend state and fetch consolidation | R47, R48 | p2 | m | IMP-007 | todo |
| IMP-058 | ADR backfill for irreversible decisions | R50 | p2 | s | - | todo |
| IMP-059 | Wiki link-integrity gate | R51 | p2 | s | - | todo |
| IMP-060 | Generated CONTINUE-HERE | R52 | p2 | s | - | todo |
| IMP-061 | BRAIN Phase 0 consent completion | Stage 5 | p0 | m | - | todo |
| IMP-062 | Quarterly envelope review ritual | Stage 5 | p2 | s | IMP-027 | todo |

## Wave 6 - go-live (operationalizes docs/deploy/go-live-guide.md)

Mostly operator work (accounts, server settings, governance); the agent's share is authoring, verification, and safety-gating. These tasks make the three go-live tracks trackable with the same acceptance/ledger discipline as the rest of the backlog. Cross-links: IMP-066 continues IMP-061; the go-live readiness gate (IMP-067) ties "flip everything on" to the Wave-1 safety nets.

| id | title | refs | prio | effort | depends_on | status |
|---|---|---|---|---|---|---|
| IMP-063 | Track A: serve a chat model in P0 and verify AI end-to-end | go-live A | p0 | s | - | todo |
| IMP-064 | Track B: desktop signing, auto-update, release verify | go-live B | p1 | m | - | todo |
| IMP-065 | Track B: mobile shells and store release pipeline | go-live B | p2 | m | IMP-064 | todo |
| IMP-066 | Track C: brain activation rollout (deploy, notice, ack, capture) | go-live C, Stage 5 | p0 | m | IMP-061 | todo |
| IMP-067 | Go-live readiness gate (safety nets before fully on) | go-live | p0 | xs | - | todo |
