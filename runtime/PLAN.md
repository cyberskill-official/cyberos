# CyberOS Runtime — Build Plan (v0.3.0 milestone)

> **This folder is a build plan, not a built system.** It documents what the runtime MUST satisfy when implemented, in a form an engineering team can pick up and execute. The CONTRACT-level source of truth is the registry under `cyberos/docs/skills/` + `cyberos/docs/contracts/`. This folder translates that into a concrete delivery plan.

## Why this exists

By registry v0.2.6 (the moment this folder is authored), CyberOS has:

- 7 skills scaffolded across cpo + cto personas, all carrying full v0.2.0 frontmatter.
- 5 contracts (`feature-request@1`, `nats-subjects@1`, `project-brief@1`, `prd@1`, `srs@1`).
- A complete chain from "human idea + BRAIN" through to "engineering tech-specs", documented end-to-end at the contract level.
- Zero executable code.

Every skill carries `gated_until_phase: runtime_v0_3_0`. The supervisor MUST NOT route to any of them until the runtime ships. This folder is the bridge from documented intent to running system.

## Source of truth

Everything in this folder DERIVES from the registry. If this folder ever contradicts the registry, the registry wins. Specifically:

- Skills + invariants live under `cyberos/docs/skills/`.
- Contracts live under `cyberos/docs/contracts/`.
- Architecture decisions live in PRD §5.10/5.11 + SRS §6.1–§6.16 (the .docx files at `cyberos/docs/CyberOS-PRD.docx` + `cyberos/docs/CyberOS-SRS.docx`).
- Audit ledger schema lives in SRS §6.7 + AGENTS.md §7.

This folder's job is to sequence the build, not redesign it.

## Build phases (registry README Part 9 mapping)

| Phase | Name | Deliverable | Estimate | Status |
| --- | --- | --- | --- | --- |
| **A** | CCSM canonicalisation | SKILL.md is already the CCSM (Canonical CyberSkill Skill Manifest); no work. | 0 | ✅ done at registry v0.2.0 |
| **B** | Transpilers | `ccsm-to-anthropic-skill`, `ccsm-to-mcp-tool`, `ccsm-to-claude-plugin`, `ccsm-to-antigravity`, `ccsm-to-codex`, `ccsm-to-cursor`. Pure functions `CCSM → host-artefact-tree`. | 2-3 weeks | 🔵 planned |
| **C** | Host shim library | `cyberos-skill-runtime` (Python) + `@cyberos/skill-runtime` (Node). Provides uniform `runtime.brain` / `runtime.audit` / `runtime.invariants` / `runtime.envelope` / `runtime.untrusted` semantics regardless of host. | 1-2 weeks | 🔵 planned |
| **D** | Equivalence test matrix | Golden input/output runs across every transpilation target. CI gate. | 1 week | 🔵 planned |
| **E** | Partner connector pipeline | Per-skill DEC required for `partner_connector: true`; build pipeline that emits the partner-side artefact. | 2 weeks | 🔵 planned (gated on first DEC) |
| **F** | LangGraph supervisor | Topology per SRS §6.1.1. classify-act node + conditional edges + checkpointing. | 2 weeks | 🔵 planned |
| **G** | `genie.action_log` | Postgres table + tamper detector + hash-chain validator. Schema in SRS §6.7. | 1 week | 🔵 planned |
| **H** | NATS event bus | JetStream config matching `nats-subjects@1` contract. Subjects + QoS + durability. | 0.5 week | 🔵 planned |
| **I** | Auto-refinement engine | Reads `INVARIANTS.md`, runs checks at declared `self_audit.check_at` checkpoints, emits `refinement_proposal` envelopes, pauses pipeline. | 1 week | 🔵 planned |
| **J** | Acceptance-test harness | Per Recipe 8. Loads fixtures from each skill's `acceptance/` folder; runs against transpiled artefact tree; asserts equivalence. | 1 week | 🔵 planned |
| **K** | BRAIN MCP server | `brain.search` + `brain.write_memory` + scope-contract enforcement. Filesystem-local for self-hosted; Postgres-backed for cloud. | 1.5 weeks | 🔵 planned |
| **L** | KB MCP server | `kb.read` + `kb.search`. Pluggable backends (Notion, Confluence, Google Docs, etc.). | 1 week | 🔵 planned |
| **M** | PROJ MCP server | `proj.read` + `proj.create_issue`. Pluggable backends (Linear, Jira, GitHub, etc.). | 1 week | 🔵 planned |
| **N** | CHAT MCP server | `chat.notify` + `chat.review_request`. Slack / Teams / Discord adapters. | 0.5 week | 🔵 planned |
| **O** | EMAIL MCP server | `email.draft` (drafts only — never auto-sends). Gmail / Outlook adapters. | 0.5 week | 🔵 planned |

**Total estimate:** ~17 engineer-weeks for a single engineer; ~6-8 weeks with 2-3 engineers in parallel.

## Critical path

```
[A done] → [G action_log] ┐
                          ├→ [F supervisor] ┐
[H NATS] ─────────────────┘                 ├→ [I auto-refinement] ┐
[K BRAIN] ──────────────────────────────────┘                      ├→ [first chained run]
[C host shim] ┐                                                    │
[B transpilers] ┴→ [J acceptance-test harness] ────────────────────┘
[L/M/N/O peripheral MCPs]  (parallel; needed for end-to-end but not blocking the chain)
```

The blocking pieces are: **G (action_log) + H (NATS) + K (BRAIN) + F (supervisor) + I (auto-refinement)**. With those five, a chained skill run is observable end-to-end. Everything else (transpilers, peripheral MCPs, acceptance harness) can land in parallel.

## How to execute

1. **Read this folder + INTERFACES.md + BUILD_ORDER.md before writing any code.**
2. **Read PRD §5 + SRS §6** for architectural context the build plan summarises but doesn't replicate.
3. **Pick a phase, follow BUILD_ORDER.md.** Each phase has a "definition of done" inline.
4. **Write code under `cyberos/runtime/<component>/`** (Python under `cyberos/runtime/python/`, Node under `cyberos/runtime/node/`).
5. **Don't modify the registry while building.** The registry is the spec; the runtime IS the implementation. If the spec needs to change, the change goes through the registry CHANGELOG first, then implementation follows.
6. **Capture lessons learned in BRAIN under `memories/refinements/REF-NNN-*.md`** as you build. Future maintainers will thank you.

## Known unknowns (worth investigating first)

- **JetStream durability tuning** — actual retention budgets depend on production traffic; the contract sets minimums but ops will tune.
- **BRAIN scope-contract enforcement at the MCP layer** — needs careful design to enforce read_excluded patterns without leaking metadata.
- **Auto-refinement loop runaway** — what if INVARIANTS.md is wrong AND auto-refinement keeps proposing the same refinement? The escalation to manual fine-tune (signal: `self_audit_refinement_proposal_count_above`) is the answer; needs implementation.
- **Antigravity / Codex / Cursor adapter behaviours** — none of the three are documented in detail by their vendors; expect investigation cost in Phase B.

## When this folder retires

Once Phase J (acceptance-test harness) is green and at least one skill has run end-to-end through the chain in production, this folder becomes historical documentation. Move it to `cyberos/runtime/archive/` and replace with a `cyberos/runtime/README.md` pointing at the actual runtime code + its operations docs.

## Citations

- Registry README Part 9 — host-adapter strategy (Phases A-E).
- Registry README Part 12 — runtime architecture (LangGraph + action_log + NATS).
- Registry README Part 26 — honest inventory of what doesn't exist.
- SRS §6.1.1 — supervisor topology.
- SRS §6.7 — `genie.action_log` schema + tamper detector.
- SRS §6.13–§6.16 — runtime mechanisms (skills↔contracts split, dual-mode, self-audit, manual fine-tune, host adapter pipeline).
- AGENTS.md §7 — audit ledger semantics.
- DEC-090..093 — the four locked decisions the runtime implements.
