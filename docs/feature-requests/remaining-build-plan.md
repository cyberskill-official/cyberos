# Remaining-modules build plan

Written 2026-06-20 (branch auto/awh-absorb) to turn the 261-FR backlog into an executable plan for the
modules that are not yet shipped. It is grounded in `docs/feature-requests/BACKLOG.md` (the state
engine, 13-layer locked build order) and the per-FR `status:` frontmatter.

Update 2026-06-24: obs is feature-complete in-repo across FR-OBS-001..009 and now also exposes its
triage path as the mcp-gateway tool `cyberos.obs.execute_triage`, so the live front of the P0 path has moved to
mcp. A per-FR mcp plan is at `mcp/MCP-BUILD-PLAN.md`. Its headline: FR-MCP-001 (spec-compliance) and
FR-MCP-002 (heartbeat lifecycle) are already shipped in `services/mcp-gateway` but still read `draft`, so
the chain is blocked by a status-lag, not by missing code - reconcile those two to `done` via the gate
first (FR-AUTH-004, the root dep, is already `done`), then FR-MCP-003 (small, independent) and FR-MCP-004
(OAuth, the one ready FR) are buildable.

## The honest frontier

The spec corpus is closed: 261 FRs, all at 10/10 audit, 25 modules with full spec coverage. But two
things gate what can actually be built next:

1. `ship-feature-requests` only picks an FR whose status is `ready_to_implement` and whose `depends_on`
   rows are all `done`. Most remaining modules are still `draft`, so the workflow is not allowed to
   start them - they need the `draft -> ready_to_implement` spec audit first.
2. Building and verifying is a toolchain step. Every module's gate is `cargo test` (or pytest / make)
   plus `caf_gate.sh`, run on a machine with the toolchain. This sandbox has neither cargo nor Docker,
   so it can prepare and plan but cannot produce the GREEN+CLEAN evidence the gate requires. Code is
   never marked `done` without that evidence (the whole point of the gate).

So the work splits into three buckets.

## Bucket A - shipped, needs gate-verification (the 8 gated modules)

These have code on `main` and golden sets; the remaining work is running the awh+caf gates per the
local loop (`docs/verification/local-run-and-verify.md`), not writing new code.

| Module | FR status | Crate / impl | Action |
|---|---|---|---|
| memory | 20 done, 1 draft | `services/memory` (Rust) + `modules/memory` (Python) | verify suite green |
| auth | 15 done | `services/auth` (Rust) | verify suite green |
| proj | 18 done, 1 draft | `services/proj` (Rust lib) | verify suite green |
| skill | 16 done, 1 fixed, 7 needs_human | `modules/skill` catalog + `services/skill-broker` | verify; 7 `needs_human` FRs need your decision |
| cuo | 10 done | `modules/cuo` (Python) | verify suite green |
| chat | 12 done (+ready/delivered) | `services/chat` (Docker) | verify via `make chat-verify` |
| ai | 20 done, 2 ready_to_implement, 1 draft | `services/ai-gateway` (Rust) | verify (needs Redis); build the 2 ready FRs |
| email | 5 done, 6 draft | `services/email` (Rust) | verify the 5; the 6 draft need spec audit |

## Bucket B - buildable now (ready_to_implement, eligible for ship-feature-requests)

These can go through the workflow today on a toolchain machine. obs is the big one and the locked next
module; the others are stragglers inside otherwise-draft modules.

| Module | Ready FRs | Scaffold | Gate | Next |
|---|---|---|---|---|
| obs | 9 (FR-OBS-001..009) | `services/obs-collector` (FR-OBS-001 shipped) | gate-ready this session | build FR-by-FR - see `obs/OBS-BUILD-PLAN.md` |
| mcp | 1 ready + 7 draft | `services/mcp-gateway` (FR-MCP-001 shipped, tested) | gate-ready this session | build the ready FR; audit the 7 draft |
| ten | 2 ready + 12 draft | - | needs goldenset | build the 2 ready (incl. FR-TEN-104 offboarding FSM, Layer 2) |
| inv | 1 ready + 10 draft | - | needs goldenset | build the 1 ready; audit the rest |
| docs | 1 ready | - | n/a | small |

## Bucket C - spec-blocked (all draft - need the draft -> ready_to_implement audit first)

~120 FRs across these modules are authored at 10/10 but still `draft`, so the build workflow cannot pick
them up. The gating step is the `feature-request-audit` chain that flips `draft -> ready_to_implement`,
not implementation. Run that first, module by module, then they join Bucket B.

crm (10), doc (11), esop (7), hr (9), kb (9), learn (7), okr (7), plugin (8), portal (8), res (5),
rew (10), time (9) - plus the draft remainders of email (6), inv (10), ten (12), mcp (7).

plugin is the closest of these: its FR-PLUGIN-001..008 are authored at 10/10 and `services/plugin-host`
exists, so it is mostly an audit-flip + build away.

## The locked build order (BACKLOG appendix B - 13 layers)

Layers must be built in order; FRs inside a layer are independent and parallelizable. The P0 critical
path is explicit in the backlog: AI Gateway -> OBS -> AUTH stub -> MCP Gateway -> CHAT. AI, AUTH, and
CHAT are already done, so the live front of that path is OBS, then MCP. Layer 0 has 9 FRs, Layer 1 has
12, Layer 2 has 11 (incl. FR-MEMORY-101 ingest and FR-TEN-104 offboarding), Layer 3 has 22, and so on
through Layer 12. Read appendix B/C in `BACKLOG.md` for the full per-layer FR lists and the sprint cut.

## How to actually execute (per module)

1. If the module is in Bucket C, run the `draft -> ready_to_implement` audit on its FRs first.
2. Make sure the module is gated: `modules/<m>/.awh/goldenset.yaml` + `modules/<m>/audit-profile.yaml`
   exist (obs, mcp, and the 8 already do; ten/inv/etc. need a goldenset wired to their test command).
3. Run `ship-feature-requests` for the module's eligible FRs. The chain drives each FR through
   implement -> review -> test, then step 28 `awh-gate` (rerun the tests) and step 29 `caf-gate`
   (rerun build/lint/test + audit). `testing -> done` flips only on `awh GREEN AND caf CLEAN`.
4. Capture each module's awh baseline once (`awh eval ... --out` then `awh lock`), as in
   `docs/verification/local-run-and-verify.md`.

Because every new module ships through the same chain, the gate applies to it automatically - there is
no separate "is it verified" step to remember.
