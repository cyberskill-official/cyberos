# Compliance mapping — how AUDIT.md's rules support audit & control objectives

This one-pager maps the protocol's core rules and this repository's release
machinery to the change-management and evidence-integrity control objectives
that enterprise audits (SOC 2-style, ISO 27001-style) typically test. It is a
**support map for evidence collection, not a certification** and not legal
advice: certification belongs to the organization operating the process.

## Why this framework helps in an audited environment

AI coding agents create a new evidence problem: changes proposed and executed
by a non-human actor, with reasoning that lives in a conversation buffer.
AUDIT.md converts that into **files** — append-only backlogs, raw command
output, closed-vocabulary statuses — and the validator makes the evidence
trail *mechanically checkable*, offline, with zero telemetry.

## Rule-to-objective map

| Protocol mechanism | Control objective it supports |
|---|---|
| **R1 — evidence or nothing** (every metric = literal command + verbatim output, or an explicit `UNMEASURED (reason)`) | Integrity of monitoring evidence; no unverifiable performance claims enter the record |
| **R2 — honest targets** (cited-with-URL or declared `INTERNAL TARGET`) | Claims substantiation; prevents fabricated external benchmarks in client-facing reports |
| **R3 — protected areas** + validator auto-load from CONFIG | Change-scope control: declared no-touch zones (business logic, public contracts) with a tripwire when a completed change references them |
| **R4 — file is memory** (resume from artifacts, never restart) | Recordkeeping: process state is reconstructable from versioned files, not from a chat session |
| **R5 — one task at a time, closed status/severity sets** | Auditability of work items: every task carries a deterministic lifecycle (`OPEN → IN-PROGRESS → DONE/BLOCKED`) and a bounded severity vocabulary |
| **R6 — circuit breaker** (3 failures → revert + root cause) | Rollback discipline; failed changes leave a documented root cause, not a silent retry loop |
| **R7 — severity-weighted, no quotas** | Anti-gaming: findings cannot be manufactured to satisfy a count, so finding volume is meaningful signal |
| **R8 — secret redaction** (+ validator pattern tripwire) | Data protection in records: credentials never enter backlogs, handoffs, logs, or commits |
| **Phase 2 gated mode** (`Approved:` is a file artifact) | Human-in-the-loop authorization with durable, attributable approval records |
| **Waivers** (`docs/AUDIT-WAIVERS.yaml`: reason, approver, mandatory expiry) | Exception management: suppressions are audit-trailed and cannot rot silently — expiry re-raises both the finding and the stale waiver |

## Process-level mechanisms (this repository)

| Mechanism | Control objective |
|---|---|
| One protocol change per version, each citing a trigger (`CHANGELOG.md`, `core/improve/FAILURE_LOG.md`) | Change management with documented rationale |
| Immutable release snapshots (`core/improve/versions/`, CI-verified byte-exact) | Tamper-evidence for the governing document itself |
| Fault-injection regression suite (32 fixtures; traps must trip, precision fixtures must not) | Control effectiveness testing — every enforced rule is proven load-bearing |
| CI release invariants (suite, version sync, baseline sha256, snapshot match, docs sync, report schema) | Segregation of duty between "changing the rules" and "shipping the rules" |
| Blind-spot register (`core/improve/BLINDSPOTS.md`, statuses with evidence) | Honest control-limitation disclosure — what the tooling *cannot* see is documented, reviewed each campaign |
| Offline-by-design validator (stdlib-only, no network, no telemetry) | Data residency / air-gap compatibility: nothing about the audited codebase leaves the machine |

**Operating it in a regulated context:** pin a release tag per engagement,
commit `docs/BACKLOG.md` + `docs/HANDOFF.md` in the target repo, run the
validator in the target's CI (`uses: cyberskill-official/code-audit-framework@v1`),
and export `--report json` per run for retention.

*Maintained by CyberSkill — info@cyberskill.world.*
