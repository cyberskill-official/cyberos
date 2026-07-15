---
id: TASK-IMP-076
title: "Distribution expansion — root CLI .sh commands (init/update/changelog/help) + remote MCP connector transport for agent UIs"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (distribution channels)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-074, TASK-CUO-206]
depends_on: []
blocks: []
source_pages:
  - "tools/cyberos-init/ (init.sh exists with --check three-value report per TASK-IMP-070; update/changelog/help existed only as plugin commands - commands/*.md)"
  - "tools/cyberos-init/mcp/cyberos-mcp.mjs (184 lines, zero-dep stdio JSON-RPC; handle() dispatch is transport-agnostic - reused verbatim for http)"
  - "Stephen's screenshots 2026-07-13: Grok 'Custom Connector' (Name + MCP server URL, /sse placeholder) and Claude 'Add custom connector' (Name + Remote MCP server URL + optional OAuth client id/secret)"
source_decisions:
  - "2026-07-13 Stephen: init, update, changelog, help should have root CLI .sh versions to trigger directly; expand distribution channels with an MCP connector to integrate with agents (Grok + Claude UIs attached)."
language: bash (root CLI), node/mjs (http transport), markdown (connector runbook)
service: tools/cyberos-init
new_files:
  - tools/cyberos-init/update.sh
  - tools/cyberos-init/changelog.sh
  - tools/cyberos-init/help.sh
  - docs/deploy/mcp-connector.md
modified_files:
  - tools/cyberos-init/build.sh
  - tools/cyberos-init/mcp/cyberos-mcp.mjs
allowed_tools: [bash, node http core module - zero new dependencies]
disallowed_tools:
  - Exposing the http transport publicly without the auth/TLS checklist (docs/deploy/mcp-connector.md) - the tools execute repo workflows
effort_hours: 5
subtasks:
  - "help.sh / changelog.sh / update.sh at payload root; update.sh wraps init.sh (--check default, --apply executes) (2h)"
  - "build.sh: copy + chmod trio, npm files array, channels += mcp-connector + root-cli (1h)"
  - "cyberos-mcp.mjs --http [port]: streamable-HTTP POST endpoint + /healthz, reusing handle() dispatch; stdio unchanged as default (1.5h)"
  - "docs/deploy/mcp-connector.md: Claude/Grok hookup + production TLS/auth checklist (0.5h)"
risk_if_skipped: "update/changelog/help remain reachable only through an installed AI plugin - a shell-only operator or CI job cannot trigger them; CyberOS stays un-integratable with agent UIs' custom-connector dialogs (Grok, Claude web/desktop), which only accept a remote MCP URL, not a stdio binary."
---

## §1 — Description
1. `update.sh`, `changelog.sh`, `help.sh` **MUST** exist at the payload root beside the already-shipping `init.sh`, directly runnable (`bash update.sh`), mirroring the plugin commands' semantics: update = read-only check by default with `--apply` to execute (thin wrapper over init.sh, which owns the logic); changelog = installed version + `rules_sha` + pointers, read from the manifest beside the script; help = the command surface.
2. `build.sh` **MUST** ship the trio in every payload (copy + chmod + npm `files` array) and declare the two new channels in `manifest.yaml`: `root-cli`, `mcp-connector`.
3. `cyberos-mcp.mjs` **MUST** gain a `--http [port]` mode: MCP streamable-HTTP style endpoint (POST = one JSON-RPC message or batch → `application/json`; notifications → 202 empty; non-POST → 405; `GET /healthz` probe), reusing the existing `handle()` dispatch verbatim so stdio and http can never drift. stdio stays the default; zero new dependencies.
4. A `docs/deploy/mcp-connector.md` runbook **MUST** capture the agent-UI hookup (Claude: Name + Remote MCP server URL + optional OAuth; Grok: Name + Server URL) and the production checklist: reverse-proxy TLS, supervisor, and the explicit warning that the transport ships unauthenticated - public exposure requires proxy-level auth since the tools execute repo workflows.
5. Grok's dialog placeholder suggests legacy `/sse`; whether it accepts streamable HTTP is **confirmed at hookup time, not asserted** - if legacy SSE is required, that transport is a recorded follow-up (§9).

*Length note: sanctioned lean profile - every §5 check ran live in-session (scripts executed, endpoint curl-verified).*

## §2 — Why
update.sh wraps init.sh instead of duplicating logic (single source of truth for vendoring + --check). The http mode reuses `handle()` so a tool added once serves both transports. Auth is deliberately NOT hand-rolled into the node process: proxy-level auth at nginx is the checklist item - a bespoke token check in a zero-dep server is worse than the battle-tested proxy layer already fronting the VPS.

## §4/§5 — Acceptance + verification (all run live 2026-07-13)
1. Payload contains executable trio; `check-version-sync.sh` still green. ✅ (build to /tmp, ls + run)
2. `help.sh` prints the surface; `changelog.sh` prints version + rules_sha from its own manifest; `update.sh` (no args, offline) emits init.sh --check's three-value report + verdict. ✅ (all executed)
3. `--http`: `/healthz` 200 JSON; POST tools/list returns the 4 workflow tools; notification → 202; GET → 405. ✅ (curl-verified)
4. stdio unchanged (default branch untouched semantics). ✅ (code path conditional on --http only)
5. Runbook exists with hookup + security checklist. ✅

## §5b — Testing pass (2026-07-13, post gate-1 "approve all")
- Payload rebuilt to /tmp: help.sh 17-line surface exit 0; changelog.sh prints 1.0.0 + rules_sha + pointers; endpoint healthz 200 / tools-list 4 / notification 202 / GET 405. PASS
- DEFECT CAUGHT + FIXED IN-PHASE: `update.sh --check` passed the flag through twice (`exec init.sh --check "$@"` without shifting), so init.sh read the second `--check` as its TARGET ("cd: --: invalid option"; root detection broken, installed=none from an installed repo). The no-arg default path had masked it at implementation time. Fix: case/shift in update.sh. Re-run, all arg shapes green - live report: installed=1.0.0 payload=1.0.0 latest=1.0.0 (GitHub releases reachable) verdict=up_to_date.

## §9 — Open questions
- Grok legacy-SSE transport if streamable HTTP is rejected at hookup (recorded, not assumed).
- OAuth client id/secret support in the connector dialog - relevant only if Stephen wants Anthropic-managed auth instead of proxy auth; follow-up.
- Public URL + reverse-proxy wiring on the VPS (operator step - checklist in the runbook).

## §10 — Failure modes
| Failure | Detection | Recovery |
|---|---|---|
| endpoint exposed without auth | runbook checklist item 3 (explicit warning) | proxy auth before DNS |
| oversized POST | 1MB cap destroys request | client retries within cap |
| batch of notifications only | 202 empty per transport semantics | n/a - correct |
| trio run outside a payload | changelog/update print explicit error + exit 2 | run from payload/.cyberos |
| stdio/http drift | impossible by construction - one handle() | n/a |

*End of TASK-IMP-076.*
