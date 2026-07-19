# Post-1.0.0 improvement backlog (opened 2026-07-19)

Successor to `IMPROVEMENT_HANDOFF.md`, which closed with: *"The handoff backlog
is empty: IMP-01..18 applied or drafted, IMP-19 applied. Remaining work is
release mechanics."* This file holds what came AFTER that line - the TRACE-006
corpus findings - so the release run stays clean.

## READ THIS FIRST - nothing here blocks the 1.0.0 tag

`IMPROVEMENT_HANDOFF.md` §9.1 already ruled on work of exactly this shape:

> **1.0.0 is none of that.** The release is the payload: `install.sh`, the
> vendored skill set, the three workflows, the npm package, the plugin zip.
> **Verdict: 0 of 67 are pre-1.0.0 blockers.** Shipping the payload does not
> require the platform's production hardening, and holding the payload for it
> would be a category error.

The TRACE-006 findings are overwhelmingly in `services/` - ai, auth, proj,
memory, chat, obs, mcp, email - which is the PLATFORM train, not the payload.
The same ruling applies. Do not escalate anything in this file into a release
blocker. If a future agent is tempted to, re-read §9.1 first.

**The payload surface passed its own audit.** In sweep terms the payload is the
`imp` module plus the `tools/install`, `tools/docs-site` and `scripts` suites:
0 INSUFFICIENT in both passes, 17 PASS acute / 6 PASS emit, 6 WEAK - and those
6 are test-strengthening, not payload defects.

## Provenance

Three committed reports carry the detail; this file is the program.

| artifact | commit | contents |
|---|---|---|
| `2026-07-19-trace-006-acute-sweep-results.md` | `481a4be6` | 234 acute clauses: 64 PASS / 31 WEAK / 112 INSUFFICIENT / 27 N/A |
| `2026-07-19-trace-006-emit-sweep-results.md` | `cd3a0f36` | 355 emit clauses: 16 PASS / 54 WEAK / 256 INSUFFICIENT / 29 N/A |
| `2026-07-19-trace-006-remediation-plan.md` | `8c50523a` | the cluster tables and task estimates (annex to this file) |

Totals across the full danger zone: **589 clauses - 80 PASS, 85 WEAK, 368
INSUFFICIENT, 56 N/A.** WEAK = a green test whose assertion is weaker than the
clause verb (true TRACE-006). INSUFFICIENT = no on-disk test asserts the verb at
all (a TRACE-004 integrity gap; most cite a test never written).

## The program

453 findings are NOT 453 tasks - they collapse to roughly 45-60 fix-tasks gated
by 6 dispositions. The binding constraint is the two HITL gates per task, not
agent throughput.

**Tier A - strengthen an existing green test (85 WEAK). No decisions needed.**
~20-25 tasks. Several collapse: chat's 4 findings are one fix (the integration
harness runs `audit_pool: None`, so no chat clause can reach its sink); auth
already has the strong `l1_audit_log` readback pattern in two places and porting
it closes 7. Start here - it also teaches the sink-assertion pattern Tier B reuses.

**Tier B - write a missing test for behaviour that DOES ship.** ~60-80 findings,
~15-20 tasks. Needs a per-clause pass to separate from Tier C.

**Tier C - behaviour never built as specified. ~250-280 findings, 6 dispositions.**
Each needs one operator call: BUILD it, or AMEND the spec / close the clause.
These are roadmap and cost decisions, not engineering ones.

1. **Observability layer** (~90-100). No OTel span or metric assertion exists
   anywhere in `services/auth`; `circuit_breaker_test.rs` is the only
   metric-registry read in all of ai-gateway. Is OTel wiring on the roadmap, or
   were those clauses aspirational?
2. **proj-sync + apps/web** (~82). `services/proj-sync/` does not exist;
   `apps/web` has zero test files. Build or descope?
3. **cuo supervisor + services/cuo** (~24). Never built; the real cuo is
   `modules/cuo`.
4. **skill never-built surface** (~40). Vietnam crates, skill-registry/OCI,
   migrate-wrap-in, sweep-placeholders. Also blocks skill's 9 WEAK - there is no
   memory writer in skill-broker, so no skill clause can currently reach the bar.
5. **auth security features** (~50-60). Mixed: some impl exists untested (-> B),
   some never built (-> C). Includes a live discrepancy: HIBP returns 409, not
   the spec's 422, with no threshold.
6. **mcp/email/obs named-but-unwritten suites** (~25).

## Named items worth doing regardless

- **MEMORY-112 #12 - live bug, verified on disk.**
  `cyberos/core/dream/detectors.py:202` filters `row.get("op") != "episode.logged"`,
  but nothing in the repo produces that row - `episode.py::log` routes through
  `ops.put` and emits only `op="put"`; every other reference is a docstring.
  Consequence: the `cyberos dream` patterns detector matches zero rows and is
  silently dead, so TASK-MEMORY-115's patterns detector has never fired. Fix:
  emit the aux row per §1 #12 plus a sink assertion (HEAD +2, op + payload),
  which closes the WEAK finding in the same stroke.
- **AI-010 #3 - the sharpest calibration failure.** The test sends
  Token/Usage/Done into an mpsc channel and asserts they come back in that
  order. A tautology: no pipeline, no SSE bytes. AI-010 #4 binds
  `let _sse = ev.to_sse_event();` and asserts nothing.
- **AUTH-005 #6** asserts `n <= 1`, satisfied at n=0 - a test that passes when
  nothing happened.
- **Spec hygiene (cheap):** AUTH-104 §1 has two clauses numbered 7, which breaks
  traces_to mapping - worth a lint rule. SKILL-101 drift: spec says
  `skill.invoked_started`, code emits `skill.invocation_started`.

## Method note for whoever picks this up

The sweep was 8 independent read-only audit delegations calibrated on the
TASK-IMP-108 §1.7 anchor (its pre-fix grep-in-payload test FAILS TRACE-006; its
post-fix visible-markup test PASSES). Severe absence claims were re-verified
directly against the host repo rather than trusted from the delegations.

One disclosure: the acute pass ran off an extraction lost to a sandbox restart.
The regenerated parser (v2) reproduced the acute total EXACTLY (234) but shifts
per-module boundaries slightly and yields 355 emit-only clauses vs the 335
originally sized - wider, so nothing in scope was skipped. Clause ids in the
reports are transcribed from the per-module records; reconcile against the tally
tables when authoring fix-tasks.

## Findings from the B3 live-plugin run (2026-07-19)

Three items surfaced only because a real backlog was authored and driven end to
end. None blocks the tag; all are cheap.

- **STATUS-REFERENCE:89 overstates a conditional guarantee.** It reads "Every
  human verdict or override emits one `memory.status_overridden` aux audit row
  ... proves a human accepted each task at the two mandatory gates." That holds
  only when the MCP writer is wired. `docs-tools/memory-append.mjs` is the
  doc-driven fallback and its `KINDS` list is deliberately closed to four kinds,
  refusing `status_overridden` with "the MCP writer owns every other kind". In a
  doc-driven run the override is silently unlogged. Fix the sentence, not the
  tool - the division of labour is correct.
- **Nothing tests the KINDS boundary.** No suite asserts the closed list, nor
  that a refused kind writes nothing. An undefended boundary is how a closed list
  quietly stops being closed. Same shape as the TRACE-006 findings: the guard
  exists, the assertion for it does not.
- **Cone over-declaration serialises a batch.** All four eligible sachviet tasks
  declared `app/web/tests/i18n-completeness.spec.ts` in `new_files`, so
  batch-select excluded three on conflict and produced a batch of one from four
  independent modules. One task should own a shared spec file; the rest should
  drop it. Worth a task-author lint rule.

## Bin-name collision: two CLIs both called `cyberos` (2026-07-19)

Found while verifying that a global `cyberos` command reaches the installer from
any directory. On this machine it does not.

`command -v cyberos` resolves to `~/.pyenv/shims/cyberos`, which is the BRAIN
memory CLI - `modules/memory/cyberos/__main__.py`, installed into pyenv 3.12.13.
Its verbs are `init view put move delete verify export audit search ... dream
episode recall-similar`. It has no `install`, `version` or `status`, so
`cyberos version` fails with `invalid choice: 'version'` - an error raised by a
DIFFERENT tool in the same product, which reads as the installer being broken.

The installer CLI is fine when reached directly: `node
dist/cyberos/cli/bin/cli.mjs version` returns installed=1.0.0 payload=1.0.0 with
matching rules_sha. The documented path (`npx cyberos <command>`) bypasses PATH,
so documented usage is unaffected.

Not a 1.0.0 blocker - it only bites when the Python package is installed, and the
documented invocation works either way. But 1.0.0 is the moment both names become
public, so the naming decision is cheaper now than after. Options: rename one bin;
or teach the Python parser to recognise the installer verbs and point at
`npx cyberos`, so the error names the real problem instead of listing memory
subcommands.
